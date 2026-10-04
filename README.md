# ZT-Graph: A zero-trust network microsegmentation audit and isolation tool 

[![CI](https://github.com/anujs-27/zt-graph/actions/workflows/ci.yml/badge.svg)](https://github.com/anujs-27/zt-graph/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/rust-stable-orange.svg?logo=rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
![Zero Trust](https://img.shields.io/badge/architecture-Zero%20Trust-blueviolet)

### What is it?
ZT-Graph is a static analysis CLI tool built in Rust for deterministic audit and automated isolation in a Zero Trust network topology.
It parses network architecture (provided via custom JSON schema), mathematically audits it to find policy violations and synthesize nftables firewall rules to sever unauthorized communication paths.

### Why it exists?
Many network configurations have a vulnerable network topology and manual verification of a complex topology to implement architecture like zero-trust may be error prone.
ZT-Graph tries to solve this by using mathematically grounded graph analysis ensuring lateral movement is contained and architectural invariants are upheld.

### Mathematical Model & Foundation
- Directed Graphs: The Network Topology is modeled as a directed graph G = (V, E) where vertices are network subnets and edges are allowed communication pathways.
- Lattice-based Access Control (Mandatory Access Control): The trust levels and flow policies are modeled as Posets forming a lattice. This ensures that data flows only in one direction according to the hierarchical clearances and horizontal compartments.
- Reachability & Blast Radius: Uses an adjacency matrix and computes the transitive closure via **Warshall's Algorithm** to map all possible direct/indirect communication paths (Blast Radius).
- Isolation: Uses flow networks to calculate the minimum capacity cut required to isolate two nodes. It applies the **Edmonds-Karp Max-Flow Min-Cut algorithm** to find the optimal edges to sever. Core invariants (paths that must never be cut) can be assigned infinite capacity so the algorithm is forced to find a safer topological path to sever.

### Commands
1. `audit`: This command checks for policy breaches. It computes the transitive closure (Blast Radius) to find indirect paths that shouldn't exist. It specifically looks for:
    * Lattice Reachability: Flows that violate the trust label hierarchy.
    * Proxy Mediation: Uses Breadth-First Search (BFS) and Lattice Covering relations to detect "multi-tier leaps"—flows that bypass firewalls and proxies by skipping trust levels unmediated.

2. `isolate`: This is the remediation command that requires `--source` and a `--target`. It calculates the optimal edges to cut to completely isolate the two without breaking critical architectural invariants, and outputs the required `nftables` rules directly to `stdout`.

### Architecture and Codebase structure
#### Dataset schema
##### Master JSON
```typescript
{
    zones: Zone[],          // Zones in the topology
    edges: Edge[]           // Edges connecting the zones
}
```
##### Zone structure
```typescript
{
    id: string;             // Identifier of the zone
    cidr: string;           // IP Range of the zone
    clearance: string;      // Trust level (Confidential, Restricted or Public)
    compartments: string[]; // Optional: Horizontal isolation tags
    is_proxy: boolean;      // True if the node acts as a Proxy / Firewall 
                            // mediating connections
}
```

##### Edge structure
```typescript
{
    from: string;               // The ID of source zone
    to: string;                 // The ID of to zone
    port: number;               // Destination network port
    capacity: number;           // The flow capacity used for min-cut. 
                                // Set to u64::MAX for uncuttable edge
    is_infrastructure: boolean; // If true, this is core routing edge and should 
                                // not be severed during targeted isolation.
}
```

#### Architecture
The codebase is organized into the following modules:
- `lattice/`: Data structures for security zone labels, clearance tiers, and access flow rules.
- `graph/`: Network topology definitions and flow network models.
- `algorithms/`: Graph traversal and analysis algorithms, including reachability checks, blast-radius estimation, and minimum-cut solvers.
- `policy/`: Configuration parsers and rule compilers (such as nftables output generation).

#### Codebase structure:
```ascii
zt-graph/
├── fixtures/         # Test scenarios and topology definitions (JSON)
└─ src/
   ├── lib.rs        # Core crate library exports
   ├── main.rs       # CLI entrypoint and reporting
   ├── lattice/      # Security lattice and flow comparators
   ├── graph/        # Topology and flow network models
   ├── algorithms/   # Graph algorithms and solvers
   └── policy/       # Parsers and rule compilers
```

### Test / edge-cases covered
The software as of now handles:
- Graph Anomalies: self loops, dangling edges and duplicate edges
- Schema Validation: Malformed CIDRs and Malformed Ports
- Isolation Logic: Impossible Isolation, and Architectural Invariants (Infra only paths): The engine halts and raises an `ArchitecturalInvariantBreach` rather than severing critical infrastructure or emitting dangerous rules.
- Scale and reference data: Scale tested till 5000 nodes and on **simulated network topologies** of College and Office infrastructure.

All the test fixtures can be tested using the files in `fixtures/` directory.

### Demo

Using the demo topology in `fixtures/demo.json`:

```console
$ ./target/release/zt-graph audit --config ./fixtures/demo.json 
=== AUDIT REPORT: "./fixtures/demo.json" ===
[FAIL] Lattice Reachability: 2 unauthorized flow(s) detected
  - Unauthorized Flow: Zone 'Security_Gateway' -> Zone 'Partner_Extranet'
  - Unauthorized Flow: Zone 'Internal_DB' -> Zone 'Partner_Extranet'
[FAIL] Proxy Mediation: 1 unmediated flow(s) detected
  - Unmediated Leap: Zone 'Public_WiFi' -> Zone 'Internal_DB'

$ ./target/release/zt-graph isolate --config ./fixtures/demo.json --source Public_WiFi --target Internal_DB
=== ISOLATION CUT: 'Public_WiFi' -> 'Internal_DB' ===
Severed Edges: 2
Severed Capacity: 60

=== SYNTHESIZED NFTABLES SCRIPT ===
table inet zt_filter {
        chain forward {
                type filter hook forward priority 0; policy accept;
                ct state established,related accept;

                ip saddr 10.0.0.0/24 ip daddr 10.0.1.0/24 th dport 443 drop
                ip saddr 10.0.0.0/24 ip daddr 10.0.2.0/24 th dport 5432 drop
        }
}
```

The direct route to `Internal_DB` through `Security_Gateway` is infrastructure-protected, so `isolate` cuts the edge leading into the gateway instead, plus the one unmediated skip-edge — both are needed to fully sever the path without touching the protected link.

### Limitations
- It is a static analysis tool so it assumes the given JSON topology is 100% accurate. It doesn't ingest live traffic or discover routes.

- The rule synthesis is limited to nftables.

- It is designed to mitigate lateral intra-movements. It can't prevent application level exploits nor save compromised networks. It'll try to prevent rest of network from getting compromised.

### Build and Run
#### Prerequisites
Standard Rust toolchain (cargo)

#### Build and run
```bash
# Build release version to ./target/release
cargo build --release

# Help
cargo run -- --help

# Usage (audit)
cargo run -- audit --config topology_file.json

# Usage (isolate)
cargo run -- isolate --config topology_file.json --source compromised_subnet --target protect_this
```

### Project Status
> This is a developed prototype and tested on simulated dataset.

> This software lacks testing on real world scale and topologies.

### Roadmap
- Auto generating JSON topologies from other formats.
- More policy generation like eBPF, Cilium, etc.