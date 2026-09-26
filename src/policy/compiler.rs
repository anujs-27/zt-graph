use ipnet::IpNet;

use crate::algorithms::min_cut::CutResult;
use crate::graph::topology::TopologyGraph;
use std::fmt::Write;

pub struct FireWallRules {
    pub src_cidr: IpNet,
    pub dst_cidr: IpNet,
    pub port: u16,
    pub action: String,
}

pub fn compile_cut_to_rules(cut: &CutResult, topo: &TopologyGraph) -> Vec<FireWallRules> {
    let mut rules: Vec<FireWallRules> = Vec::new();

    for &(from_zone_idx, edge_idx) in &cut.cut_edges {
        let source_zone = &topo.zones[from_zone_idx];
        let target_edge = &topo.adjacency[from_zone_idx][edge_idx];
        let target_zone = &topo.zones[target_edge.to];

        rules.push(FireWallRules {
            src_cidr: source_zone.cidr,
            dst_cidr: target_zone.cidr,
            port: target_edge.port,
            action: "drop".to_string(),
        });
    }

    rules
}

pub fn render_nftables_script(
    rules: &[FireWallRules],
    table_name: &str,
    chain_name: &str,
) -> String {
    let mut script = String::new();

    let _ = writeln!(script, "table inet {} {{", table_name);
    let _ = writeln!(script, "\tchain {} {{", chain_name);
    let _ = writeln!(
        script,
        "\t\ttype filter hook forward priority 0; policy accept;"
    );
    let _ = writeln!(script, "\t\tct state established,related accept;");
    let _ = writeln!(script);

    for rule in rules {
        let action = rule.action.to_lowercase();
        let src_dst_ip = (
            rule.src_cidr.addr().is_ipv4(),
            rule.dst_cidr.addr().is_ipv4(),
        );
        match src_dst_ip {
            (true, true) => {
                let _ = writeln!(
                    script,
                    "\t\tip saddr {} ip daddr {} th dport {} {}",
                    rule.src_cidr, rule.dst_cidr, rule.port, action
                );
            }
            (false, false) => {
                let _ = writeln!(
                    script,
                    "\t\tip6 saddr {} ip6 daddr {} th dport {} {}",
                    rule.src_cidr, rule.dst_cidr, rule.port, action
                );
            }
            _ => continue,
        }
    }

    let _ = writeln!(script, "\t}}");
    let _ = writeln!(script, "}}");

    script
}
