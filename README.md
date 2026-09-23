# ZT-Graph

A Rust library and CLI tool for modeling enterprise network topologies, access control policies, and network reachability.

## Architecture

The codebase is organized into the following modules:

* **`lattice/`**: Data structures for security zone labels, clearance tiers, and access flow rules.
* **`graph/`**: Network topology definitions and flow network models.
* **`algorithms/`**: Graph traversal and analysis algorithms, including reachability checks, blast-radius estimation, and minimum-cut solvers.
* **`policy/`**: Configuration parsers and rule compilers (such as `nftables` output generation).

## Project Structure

```text
zt-graph/
├── fixtures/         # Test scenarios and topology definitions (JSON)
├── src/
│   ├── lib.rs        # Core crate library exports
│   ├── main.rs       # CLI entrypoint and reporting
│   ├── lattice/      # Security lattice and flow comparators
│   ├── graph/        # Topology and flow network models
│   ├── algorithms/   # Graph algorithms and solvers
│   └── policy/       # Parsers and rule compilers
└── tests/            # Integration and operational test suites
```

## Building and Testing

```bash
cargo build --release
cargo test
```