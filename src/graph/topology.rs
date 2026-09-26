use crate::lattice::labels::SecurityLabel;
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityZone {
    pub id: String,
    pub cidr: IpNet,
    pub label: SecurityLabel,
    pub is_proxy: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitiationEdge {
    pub from: usize,
    pub to: usize,
    pub port: u16,
    pub capacity: u64,
    pub is_infrastructure: bool,
}

#[derive(Debug)]
pub struct TopologyGraph {
    pub zones: Vec<SecurityZone>,
    pub node_index_map: HashMap<String, usize>,
    pub adjacency: Vec<Vec<InitiationEdge>>,
}

impl SecurityZone {
    pub fn new(id: String, cidr: IpNet, label: SecurityLabel, is_proxy: bool) -> Self {
        SecurityZone {
            id,
            cidr,
            label,
            is_proxy,
        }
    }
}

impl TopologyGraph {
    pub fn new() -> Self {
        TopologyGraph {
            zones: Vec::new(),
            node_index_map: HashMap::new(),
            adjacency: Vec::new(),
        }
    }

    pub fn add_zone(&mut self, sec_zone: SecurityZone) -> Result<usize, &'static str> {
        if self.node_index_map.contains_key(&sec_zone.id) {
            return Err("ERROR: Key already exists");
        }

        let index = self.zones.len();
        self.node_index_map.insert(sec_zone.id.clone(), index);
        self.zones.push(sec_zone);
        self.adjacency.push(Vec::new());

        Ok(index)
    }

    pub fn add_initiation_edge(
        &mut self,
        from: usize,
        to: usize,
        port: u16,
        capacity: u64,
        is_infra: bool,
    ) -> Result<(), &'static str> {
        // Validate that both nodes exist in the graph
        if from >= self.zones.len() || to >= self.zones.len() {
            return Err("ERROR: Zone index out of bounds!");
        }

        if from == to {
            return Err("ERROR: Self-loops are not allowed");
        }

        if !is_infra && capacity == u64::MAX {
            return Err("ERROR: Non-infrastructure edge cannot have infinite capacity (u64::MAX)");
        }

        let edge: InitiationEdge = InitiationEdge {
            from,
            to,
            port,
            capacity,
            is_infrastructure: is_infra,
        };

        self.adjacency[from].push(edge);

        Ok(())
    }

    pub fn to_adjacency_matrix(&self) -> Vec<Vec<bool>> {
        let n = self.zones.len();
        let mut matrix: Vec<Vec<bool>> = vec![vec![false; n]; n];

        for (u, edges) in self.adjacency.iter().enumerate() {
            for edge in edges {
                let v = edge.to;
                matrix[u][v] = true;
            }
        }

        matrix
    }

    pub fn zone_count(&self) -> usize {
        self.zones.len()
    }

    pub fn get_zone_index(&self, id: &str) -> Option<usize> {
        self.node_index_map.get(id).copied()
    }

    pub fn get_zone(&self, index: usize) -> Option<&SecurityZone> {
        self.zones.get(index)
    }
}
