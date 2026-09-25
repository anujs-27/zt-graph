use crate::graph::topology::TopologyGraph;
use std::usize;

pub const INF_CAPACITY: u64 = u64::MAX;

pub struct FlowEdge {
    pub to: usize,
    pub rev: usize,
    pub capacity: u64,
    pub flow: u64,
    pub original_flow_index: Option<(usize, usize)>, // (from, edge_index_adjacency in TopologyGraph)
}

pub struct FlowNetwork {
    pub adj: Vec<Vec<FlowEdge>>,
}

impl FlowNetwork {
    pub fn new(n: usize) -> Self {
        FlowNetwork {
            adj: (0..n).map(|_| Vec::new()).collect(),
        }
    }

    pub fn add_edge(
        &mut self,
        from: usize,
        to: usize,
        cap: u64,
        orig_index: Option<(usize, usize)>,
    ) {
        let forward_idx = self.adj[from].len();
        let backward_idx = self.adj[to].len();

        let forward = FlowEdge {
            to,
            rev: backward_idx,
            capacity: cap,
            flow: 0,
            original_flow_index: orig_index,
        };

        let backward = FlowEdge {
            to: from,
            rev: forward_idx,
            capacity: 0,
            flow: 0,
            original_flow_index: None, // Residual edge
        };

        self.adj[from].push(forward);
        self.adj[to].push(backward);
    }

    pub fn reset_flows(&mut self) {
        for edges in &mut self.adj {
            for edge in edges {
                edge.flow = 0;
            }
        }
    }
}

impl FlowNetwork {
    pub fn from_topology(topo: &TopologyGraph) -> FlowNetwork {
        let n: usize = topo.adjacency.len();
        let mut net: FlowNetwork = FlowNetwork::new(n);

        for (from_idx, edges) in topo.adjacency.iter().enumerate() {
            for (edge_idx, edge) in edges.iter().enumerate() {
                let cap = if edge.is_infrastructure {
                    INF_CAPACITY
                } else {
                    edge.capacity
                };

                let orig_index = Some((from_idx, edge_idx));

                net.add_edge(from_idx, edge.to, cap, orig_index);
            }
        }

        net
    }
}
