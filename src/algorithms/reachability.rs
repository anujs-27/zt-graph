/*
[
 [a, b, c],
 [d, e, f],   -> [a, b, c, d, e, f, g, h, i]
 [g, h, i]
]
*/

use crate::{graph::topology::TopologyGraph, lattice::poset::flows_to};

pub struct ReachabilityMatrix {
    pub size: usize,
    pub matrix: Vec<bool>,
}

impl ReachabilityMatrix {
    pub fn new(size: usize) -> Self {
        ReachabilityMatrix {
            size,
            matrix: vec![false; size * size],
        }
    }

    pub fn get(&self, row: usize, col: usize) -> bool {
        let idx = row * self.size + col;
        self.matrix[idx]
    }

    pub fn set(&mut self, row: usize, col: usize, val: bool) {
        let idx = row * self.size + col;
        self.matrix[idx] = val;
    }

    pub fn is_reachable(&self, u: usize, v: usize) -> bool {
        if u >= self.size || v >= self.size {
            return false;
        }
        self.get(u, v)
    }

    #[allow(clippy::needless_range_loop)]
    pub fn compute_transitive_closure(adjacency_matrix: &[Vec<bool>]) -> Self {
        let size = adjacency_matrix.len();
        let mut matrix = ReachabilityMatrix::new(size);

        for i in 0..size {
            for j in 0..size {
                matrix.set(i, j, adjacency_matrix[i][j]);
            }
        }

        // Warshall's algorithm
        for k in 0..size {
            for i in 0..size {
                if matrix.get(i, k) {
                    for j in 0..size {
                        if !matrix.get(i, j) && matrix.get(k, j) {
                            matrix.set(i, j, true);
                        }
                    }
                }
            }
        }

        matrix
    }
}

pub fn audit_lattice_reachability(
    closure: &ReachabilityMatrix,
    topo: &TopologyGraph,
) -> Vec<(usize, usize)> {
    assert_eq!(
        closure.size,
        topo.zone_count(),
        "ReachabilityMatrix size must match TopologyGraph zone count"
    );

    let mut reachables: Vec<(usize, usize)> = Vec::new();

    for i in 0..closure.size {
        for j in 0..closure.size {
            if i == j {
                continue;
            }

            let label_u = &topo.zones[i].label;
            let label_v = &topo.zones[j].label;

            if closure.is_reachable(i, j) && !flows_to(label_u, label_v) {
                reachables.push((i, j));
            }
        }
    }

    reachables
}

pub fn audit_unmediated_flow(topo: &TopologyGraph) -> Vec<(usize, usize)> {
    let mut violating_edges: Vec<(usize, usize)> = Vec::new();
    let n = topo.zone_count();

    for u in 0..n {
        let mut visited = vec![false; n];
        let mut queue = std::collections::VecDeque::new();
        visited[u] = true;
        queue.push_back(u);

        while let Some(curr) = queue.pop_front() {
            for edge in &topo.adjacency[curr] {
                let v = edge.to;
                if visited[v] {
                    continue;
                }
                visited[v] = true;

                let zone_u = &topo.zones[u];
                let zone_v = &topo.zones[v];

                // Only inspect paths that flow legally but might skip a tier
                if flows_to(&zone_u.label, &zone_v.label) {
                    let multi_tier_leap = zone_u.label != zone_v.label
                        && !crate::lattice::poset::is_covered_by_assumes_flow(
                            &zone_v.label,
                            &zone_u.label,
                        );

                    let unmediated = !zone_u.is_proxy && !zone_v.is_proxy;

                    if multi_tier_leap && unmediated {
                        violating_edges.push((u, v));
                    }
                }

                // If v is a proxy, it mediates any further downstream leaps along this path.
                // We only continue exploring if it's NOT a proxy.
                if !zone_v.is_proxy {
                    queue.push_back(v);
                }
            }
        }
    }

    violating_edges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::topology::SecurityZone;
    use crate::lattice::labels::{Clearance, SecurityLabel};
    use ipnet::IpNet;
    use std::collections::BTreeSet;

    #[test]
    fn test_warshall_algorithm() {
        let adj_matrix: Vec<Vec<bool>> = vec![
            vec![false, true, false],  // A -> B
            vec![false, false, true],  // B -> C
            vec![false, false, false], // C has no outgoing edges
        ];

        let closure: ReachabilityMatrix =
            ReachabilityMatrix::compute_transitive_closure(&adj_matrix);

        assert!(closure.get(0, 1), "A should directly reach B");
        assert!(closure.get(1, 2), "B should directly reach C");
        assert!(closure.get(0, 2), "A should transitively reach C via B");
        assert!(!closure.get(2, 0), "C should not reach A (acyclic check)");
    }

    #[test]
    fn test_audit_lattice_reachability() {
        let mut topo = TopologyGraph::new();
        let cidr: IpNet = "10.0.0.0/24".parse().unwrap();

        // Zone 0: Confidential
        topo.add_zone(SecurityZone::new(
            "ZoneA".to_string(),
            cidr,
            SecurityLabel::new(Clearance::Confidential, BTreeSet::new()),
            false,
        ))
        .unwrap();

        // Zone 1: Public
        topo.add_zone(SecurityZone::new(
            "ZoneB".to_string(),
            cidr,
            SecurityLabel::new(Clearance::Public, BTreeSet::new()),
            false,
        ))
        .unwrap();

        let mut adj = vec![vec![false; 2]; 2];
        adj[0][1] = true;

        let closure = ReachabilityMatrix::compute_transitive_closure(&adj);
        let violations = audit_lattice_reachability(&closure, &topo);

        assert_eq!(violations.len(), 1, "Should detect 1 unauthorized flow");
        assert_eq!(
            violations[0],
            (0, 1),
            "Flow from Confidential to Public should be flagged"
        );
    }

    #[test]
    fn test_audit_unmediated_flow() {
        let mut topo = TopologyGraph::new();
        let cidr: IpNet = "10.0.0.0/24".parse().unwrap();

        // Zone 0: Public
        topo.add_zone(SecurityZone::new(
            "ZoneA".to_string(),
            cidr,
            SecurityLabel::new(Clearance::Public, BTreeSet::new()),
            false,
        ))
        .unwrap();

        // Zone 1: Confidential (Skipping Restricted)
        topo.add_zone(SecurityZone::new(
            "ZoneB".to_string(),
            cidr,
            SecurityLabel::new(Clearance::Confidential, BTreeSet::new()),
            false,
        ))
        .unwrap();

        // Connect A directly to B (No proxy mediation)
        topo.add_initiation_edge(0, 1, 80, 100, false).unwrap();

        let violations = audit_unmediated_flow(&topo);

        assert_eq!(violations.len(), 1, "Should detect 1 unmediated leap");
        assert_eq!(
            violations[0],
            (0, 1),
            "Public -> Confidential without proxy should be flagged"
        );
    }

    #[test]
    fn test_proxy_stop_condition_edge_case() {
        let mut topo = TopologyGraph::new();
        let cidr: IpNet = "10.0.0.0/24".parse().unwrap();

        // Zone 0: Public
        topo.add_zone(SecurityZone::new(
            "ZoneA".to_string(),
            cidr,
            SecurityLabel::new(Clearance::Public, BTreeSet::new()),
            false,
        ))
        .unwrap();

        // Zone 1: Restricted PROXY
        topo.add_zone(SecurityZone::new(
            "ZoneB-Proxy".to_string(),
            cidr,
            SecurityLabel::new(Clearance::Restricted, BTreeSet::new()),
            true, // is_proxy = true
        ))
        .unwrap();

        // Zone 2: Confidential
        topo.add_zone(SecurityZone::new(
            "ZoneC".to_string(),
            cidr,
            SecurityLabel::new(Clearance::Confidential, BTreeSet::new()),
            false,
        ))
        .unwrap();

        // Connect A -> Proxy -> C
        topo.add_initiation_edge(0, 1, 80, 100, false).unwrap();
        topo.add_initiation_edge(1, 2, 80, 100, false).unwrap();

        let violations = audit_unmediated_flow(&topo);

        assert!(
            violations.is_empty(),
            "Proxy mediated the flow, so there should be no violations"
        );
    }
}
