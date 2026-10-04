use std::collections::VecDeque;

use crate::graph::flow::FlowNetwork;

#[derive(Debug, PartialEq, Eq)]
pub enum ArchitecturalInvariantBreach {
    Message(String),
}

pub struct CutResult {
    pub cut_edges: Vec<(usize, usize)>, // (from_zone, edge_index_topology)
    pub total_capacity: u64,
}

impl FlowNetwork {
    pub fn max_flow_engine(&mut self, s: usize, t: usize) -> u64 {
        if s == t {
            return 0;
        }
        if s >= self.adj.len() || t >= self.adj.len() {
            return 0;
        }

        self.reset_flows();

        let mut max_flow = 0u64;
        let n: usize = self.adj.len();
        let mut parent: Vec<Option<(usize, usize)>> = vec![None; n];
        let mut visited = vec![false; n];
        let mut queue: VecDeque<usize> = VecDeque::with_capacity(n);

        loop {
            parent.fill(None);
            visited.fill(false);
            queue.clear();

            visited[s] = true;
            queue.push_back(s);

            let mut found_path = false;

            // BFS to find the shortest augmenting path in the residual graph
            while let Some(u) = queue.pop_front() {
                if u == t {
                    found_path = true;
                    break;
                }

                for i in 0..self.adj[u].len() {
                    let edge = &self.adj[u][i];

                    let residual_capacity = if edge.original_flow_index.is_some() {
                        edge.capacity - edge.flow
                    } else {
                        self.adj[edge.to][edge.rev].flow
                    };

                    if residual_capacity > 0 && !visited[edge.to] {
                        visited[edge.to] = true;
                        parent[edge.to] = Some((u, i));
                        queue.push_back(edge.to);
                    }
                }
            }

            if !found_path {
                break;
            }

            // Find the bottleneck capacity (minimum residual capacity along the path)
            let mut path_flow = u64::MAX;
            let mut curr = t;

            while curr != s {
                if let Some((u, i)) = parent[curr] {
                    let edge = &self.adj[u][i];
                    let residual_capacity = if edge.original_flow_index.is_some() {
                        edge.capacity - edge.flow
                    } else {
                        self.adj[edge.to][edge.rev].flow
                    };
                    path_flow = path_flow.min(residual_capacity);
                    curr = u;
                } else {
                    break;
                }
            }

            // Augment flow along the path using asymmetric updates to prevent underflow
            curr = t;
            while curr != s {
                if let Some((u, i)) = parent[curr] {
                    let edge = &self.adj[u][i];
                    let rev_idx = edge.rev;
                    let to = edge.to;

                    if edge.original_flow_index.is_some() {
                        self.adj[u][i].flow += path_flow;
                    } else {
                        self.adj[to][rev_idx].flow -= path_flow;
                    }

                    curr = u;
                } else {
                    break;
                }
            }

            max_flow = max_flow.saturating_add(path_flow);
        }

        max_flow
    }

    pub fn compute_min_cut(
        &mut self,
        s: usize,
        t: usize,
    ) -> Result<CutResult, ArchitecturalInvariantBreach> {
        if s == t {
            return Err(ArchitecturalInvariantBreach::Message(
                "Source and sink cannot be identical".into(),
            ));
        }
        if s >= self.adj.len() || t >= self.adj.len() {
            return Err(ArchitecturalInvariantBreach::Message(format!(
                "Invalid node index: source {} or sink {} out of bounds (max {})",
                s,
                t,
                self.adj.len() - 1
            )));
        }

        // Run max flow engine first to saturate the network
        self.max_flow_engine(s, t);

        // Run BFS on the residual graph starting from s to find all reachable vertices (Set S)
        let n: usize = self.adj.len();
        let mut visited = vec![false; n];
        let mut queue = VecDeque::with_capacity(n);

        visited[s] = true;
        queue.push_back(s);

        while let Some(u) = queue.pop_front() {
            for edge in &self.adj[u] {
                let residual_capacity = if edge.original_flow_index.is_some() {
                    edge.capacity - edge.flow
                } else {
                    self.adj[edge.to][edge.rev].flow
                };

                if residual_capacity > 0 && !visited[edge.to] {
                    visited[edge.to] = true;
                    queue.push_back(edge.to);
                }
            }
        }

        let mut cut_edges = Vec::new();
        let mut total_capacity = 0u64;

        // Collect cut edges crossing from set S (visited) to set T (unvisited)
        for u in 0..self.adj.len() {
            if visited[u] {
                for edge in &self.adj[u] {
                    if !visited[edge.to] && edge.original_flow_index.is_some() {
                        // Check architectural invariants for cut boundaries
                        if edge.capacity == u64::MAX {
                            return Err(ArchitecturalInvariantBreach::Message(format!(
                                "Invariant breach: cut crosses infinite capacity edge from {} to {}",
                                u, edge.to
                            )));
                        }
                        if let Some(orig_idx) = edge.original_flow_index {
                            cut_edges.push(orig_idx);
                        }

                        total_capacity = total_capacity.saturating_add(edge.capacity);
                    }
                }
            }
        }

        Ok(CutResult {
            cut_edges,
            total_capacity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_min_cut_valid() {
        let mut net = FlowNetwork::new(3);
        net.add_edge(0, 1, 10, Some((0, 0)));
        net.add_edge(1, 2, 5, Some((1, 0))); // Bottleneck

        let cut = net.compute_min_cut(0, 2).unwrap();
        assert_eq!(cut.total_capacity, 5);
        assert_eq!(cut.cut_edges, vec![(1, 0)]);
    }

    #[test]
    fn test_compute_min_cut_parallel_paths() {
        let mut net = FlowNetwork::new(4);
        // S (0) -> A (1) -> T (3) [capacity 10, bottleneck 10]
        net.add_edge(0, 1, 10, Some((0, 0)));
        net.add_edge(1, 3, 20, Some((1, 0)));
        // S (0) -> B (2) -> T (3) [capacity 5, bottleneck 5]
        net.add_edge(0, 2, 5, Some((0, 1)));
        net.add_edge(2, 3, 10, Some((2, 0)));

        let cut = net.compute_min_cut(0, 3).unwrap();
        assert_eq!(cut.total_capacity, 15);
        assert_eq!(cut.cut_edges.len(), 2);
    }

    #[test]
    fn test_compute_min_cut_with_cycles() {
        let mut net = FlowNetwork::new(4);
        net.add_edge(0, 1, 10, Some((0, 0))); // S -> A
        net.add_edge(1, 2, 10, Some((1, 0))); // A -> B
        net.add_edge(2, 1, 10, Some((2, 0))); // B -> A (Cycle)
        net.add_edge(2, 3, 5, Some((2, 1))); // B -> T (Bottleneck)

        let cut = net.compute_min_cut(0, 3).unwrap();
        assert_eq!(cut.total_capacity, 5);
        assert_eq!(cut.cut_edges, vec![(2, 1)]);
    }

    #[test]
    fn test_compute_min_cut_disconnected() {
        let mut net = FlowNetwork::new(4);
        net.add_edge(0, 1, 10, Some((0, 0))); // S -> A
        net.add_edge(2, 3, 10, Some((2, 0))); // B -> T (No path from S to T)

        let cut = net.compute_min_cut(0, 3).unwrap();
        assert_eq!(cut.total_capacity, 0);
        assert!(cut.cut_edges.is_empty());
    }

    #[test]
    fn test_compute_min_cut_errors() {
        let mut net = FlowNetwork::new(2);

        // Identical nodes
        assert!(matches!(
            net.compute_min_cut(0, 0),
            Err(ArchitecturalInvariantBreach::Message(_))
        ));
        // Out of bounds
        assert!(matches!(
            net.compute_min_cut(0, 5),
            Err(ArchitecturalInvariantBreach::Message(_))
        ));
    }

    #[test]
    fn test_compute_min_cut_invariant_breach() {
        let mut net = FlowNetwork::new(2);
        net.add_edge(0, 1, u64::MAX, Some((0, 0))); // Infrastructure edge

        assert!(matches!(
            net.compute_min_cut(0, 1),
            Err(ArchitecturalInvariantBreach::Message(_))
        ));
    }
}
