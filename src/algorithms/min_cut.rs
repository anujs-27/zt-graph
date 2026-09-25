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
    /// Computes the maximum flow from source `s` to sink `t` using the Edmonds-Karp algorithm.
    pub fn max_flow_engine(&mut self, s: usize, t: usize) -> u64 {
        let mut max_flow = 0u64;

        loop {
            let mut parent: Vec<Option<(usize, usize)>> = vec![None; self.adj.len()];
            let mut visited = vec![false; self.adj.len()];
            let mut queue: VecDeque<usize> = VecDeque::new();

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

                    let residual_capacity = if edge.capacity > 0 {
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
                    let residual_capacity = if edge.capacity > 0 {
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

                    if edge.capacity > 0 {
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

    /// Computes the minimum cut partition boundary after running max flow.
    pub fn compute_min_cut(
        &mut self,
        s: usize,
        t: usize,
    ) -> Result<CutResult, ArchitecturalInvariantBreach> {
        // Run max flow engine first to saturate the network
        self.max_flow_engine(s, t);

        // Run BFS on the residual graph starting from s to find all reachable vertices (Set S)
        let mut visited = vec![false; self.adj.len()];
        let mut queue = VecDeque::new();

        visited[s] = true;
        queue.push_back(s);

        while let Some(u) = queue.pop_front() {
            for edge in &self.adj[u] {
                let residual_capacity = if edge.capacity > 0 {
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

        if visited[t] {
            return Err(ArchitecturalInvariantBreach::Message(format!(
                "Architectural deadlock: Sink {} remains reachable from source {} in the residual graph, indicating un-severable infrastructure paths.",
                t, s
            )));
        }

        let mut cut_edges = Vec::new();
        let mut total_capacity = 0u64;

        // Collect cut edges crossing from set S (visited) to set T (unvisited)
        for u in 0..self.adj.len() {
            if visited[u] {
                for edge in &self.adj[u] {
                    if !visited[edge.to] {
                        if edge.capacity > 0 {
                            // Check architectural invariants for cut boundaries
                            if edge.is_infrastructure || edge.capacity == u64::MAX {
                                return Err(ArchitecturalInvariantBreach::Message(format!(
                                    "Invariant breach: cut crosses critical infrastructure or infinite capacity edge from {} to {}",
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
        }

        Ok(CutResult {
            cut_edges,
            total_capacity,
        })
    }
}
