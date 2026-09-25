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
