use clap::{Parser, Subcommand};
use std::path::PathBuf;
use zt_graph::{
    algorithms::{
        min_cut::ArchitecturalInvariantBreach,
        reachability::{ReachabilityMatrix, audit_lattice_reachability, audit_unmediated_flow},
    },
    graph::{flow::FlowNetwork, topology::TopologyGraph},
    policy::{
        compiler::{compile_cut_to_rules, render_nftables_script},
        parser::load_topology_from_json,
    },
};

#[derive(Parser)]
#[command(name = "zt-graph")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Audit {
        #[arg(short, long)]
        config: PathBuf,
    },
    Isolate {
        #[arg(short, long)]
        config: PathBuf,

        #[arg(short, long)]
        source: String,

        #[arg(short, long)]
        target: String,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Audit { config } => {
            audit_mode(config);
        }
        Commands::Isolate {
            config,
            source,
            target,
        } => {
            isolate_mode(&config, source, target);
        }
    }
}

fn audit_mode(config: PathBuf) {
    let topology: TopologyGraph = match load_topology_from_json(&config) {
        Ok(topo) => topo,
        Err(err) => {
            eprintln!("ERROR: Failed to load topology: {}", err);
            std::process::exit(1);
        }
    };

    let adjacency_matrix = topology.to_adjacency_matrix();
    let transitive_closure = ReachabilityMatrix::compute_transitive_closure(&adjacency_matrix);
    let violation_edges = audit_lattice_reachability(&transitive_closure, &topology);
    let unmediated_flows = audit_unmediated_flow(&topology);

    println!("=== AUDIT REPORT: {:?} ===", config);

    if violation_edges.is_empty() {
        println!("[PASS] Lattice Reachability: No unauthorized flows detected");
    } else {
        println!(
            "[FAIL] Lattice Reachability: {} unauthorized flow(s) detected",
            violation_edges.len()
        );
        for (u, v) in &violation_edges {
            let src = &topology.zones[*u];
            let dst = &topology.zones[*v];
            println!(
                "  - Unauthorized Flow: Zone '{}' -> Zone '{}'",
                src.id, dst.id
            );
        }
    }

    if unmediated_flows.is_empty() {
        println!("[PASS] Proxy Mediation: No unmediated cross-tier flows detected");
    } else {
        println!(
            "[FAIL] Proxy Mediation: {} unmediated flow(s) detected",
            unmediated_flows.len()
        );
        for (u, v) in &unmediated_flows {
            let src = &topology.zones[*u];
            let dst = &topology.zones[*v];
            println!(
                "  - Unmediated Leap: Zone '{}' -> Zone '{}'",
                src.id, dst.id
            );
        }
    }
}

fn isolate_mode(config: &PathBuf, source: String, target: String) {
    let topology: TopologyGraph = match load_topology_from_json(config) {
        Ok(topo) => topo,
        Err(err) => {
            eprintln!("ERROR: Failed to load topology: {}", err);
            std::process::exit(1);
        }
    };

    let source_index = match topology.get_zone_index(&source) {
        Some(index) => index,
        None => {
            eprintln!("ERROR: Source zone '{}' not found", source);
            std::process::exit(1);
        }
    };

    let target_index = match topology.get_zone_index(&target) {
        Some(index) => index,
        None => {
            eprintln!("ERROR: Target zone '{}' not found", target);
            std::process::exit(1);
        }
    };

    let mut flow_network = FlowNetwork::from_topology(&topology);
    let cut_res = match flow_network.compute_min_cut(source_index, target_index) {
        Ok(cut_result) => cut_result,
        Err(ArchitecturalInvariantBreach::Message(msg)) => {
            eprintln!("ERROR: Architectural invariant breach: {}", msg);
            std::process::exit(1);
        }
    };

    println!("=== ISOLATION CUT: '{}' -> '{}' ===", source, target);
    println!("Severed Edges: {}", cut_res.cut_edges.len());
    println!("Severed Capacity: {}", cut_res.total_capacity);

    let final_rules = compile_cut_to_rules(&cut_res, &topology);
    let nft_rules = render_nftables_script(&final_rules, "zt_filter", "forward");

    println!("\n=== SYNTHESIZED NFTABLES SCRIPT ===");
    print!("{}", nft_rules);
}
