use std::{fs::File, io::BufReader};

use serde::{Deserialize, Serialize};

use crate::graph::topology::{SecurityZone, TopologyGraph};
use crate::lattice::labels::{Clearance, SecurityLabel};

use ipnet::IpNet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawZone {
    pub id: String,
    pub cidr: IpNet,
    pub clearance: Clearance,
    pub compartments: Vec<String>,
    pub is_proxy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEdge {
    #[serde(rename = "from")]
    pub from_zone: String,

    #[serde(rename = "to")]
    pub to_zone: String,

    pub port: u16,
    pub capacity: u64,
    pub is_infrastructure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawConfig {
    pub zones: Vec<RawZone>,
    pub edges: Vec<RawEdge>,
}

pub fn load_topology_from_json(path: &std::path::Path) -> Result<TopologyGraph, String> {
    let file =
        File::open(path).map_err(|e| format!("Failed to open config file at {:?}: {}", path, e))?;

    let reader = BufReader::new(file);
    let raw_json: RawConfig = serde_json::from_reader(reader)
        .map_err(|e| format!("Failed to parse JSON topology: {}", e))?;
    let mut topo: TopologyGraph = TopologyGraph::new();

    for zone in raw_json.zones {
        let sec_zone = SecurityZone::new(
            zone.id,
            zone.cidr,
            SecurityLabel::new(zone.clearance, zone.compartments.into_iter().collect()),
            zone.is_proxy,
        );
        topo.add_zone(sec_zone)?;
    }

    for edge in raw_json.edges {
        let from_index = match topo.get_zone_index(&edge.from_zone) {
            Some(idx) => idx,
            None => {
                return Err(format!(
                    "ERROR: Unknown source zone '{}' referenced in edge",
                    edge.from_zone
                ));
            }
        };

        let to_index = match topo.get_zone_index(&edge.to_zone) {
            Some(idx) => idx,
            None => {
                return Err(format!(
                    "ERROR: Unknown target zone '{}' referenced in edge",
                    edge.to_zone
                ));
            }
        };

        topo.add_initiation_edge(
            from_index,
            to_index,
            edge.port,
            edge.capacity,
            edge.is_infrastructure,
        )?;
    }

    Ok(topo)
}
