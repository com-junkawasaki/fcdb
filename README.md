# FCDB (Enishi)

**A Functorial–Categorical Database**

## About

FCDB (Enishi) is a Functorial–Categorical Database model that separates graph responsibility (observation) from categorical authority (persistence), and composes Ownership, Capability, CAS, and Graph as a double categorical system. It constitutes a "9th lineage" of database systems, integrating features from Hash/Trie, Append-only, Graph, and Blob stores.

For a deep dive into the theory and architecture, please see the research paper: [FCDB: A Functorial–Categorical Database](./research/fcdb/main.tex).

## Project Status

- **Status:** Production Ready
- **Validation:** Passed

## Key Achievements

- **Mathematical:** A novel categorical database model with proven functorial preservation.
- **Performance:** High-throughput graph queries (e.g., 3-hop queries in ~9.6ms).
- **Security:** Guarantees memory and capability safety through an `Own+CFA` (Ownership + Control-Flow Attestation) model.
- **Architecture:** A self-learning adaptive optimization system.

## Crates

The project is composed of several Rust crates available on [crates.io](https://crates.io):

| Crate | Version | Description |
|-------|---------|-------------|
| [`fcdb-core`](https://crates.io/crates/fcdb-core) | `0.1.1` | Core data structures and utilities for FCDB |
| [`fcdb-cas`](https://crates.io/crates/fcdb-cas) | `0.1.0` | Content-Addressable Storage implementation |
| [`fcdb-graph`](https://crates.io/crates/fcdb-graph) | `0.1.0` | Graph operations and traversal for FCDB |
| [`fcdb-api`](https://crates.io/crates/fcdb-api) | `0.1.0` | Public API interfaces (GraphQL, gRPC, REST) |
| [`fcdb-concur`](https://crates.io/crates/fcdb-concur) | `0.1.0` | Concurrency primitives and async utilities |
| [`fcdb-exec`](https://crates.io/crates/fcdb-exec) | `0.1.0` | Query execution engine for FCDB |
| [`fcdb-tools`](https://crates.io/crates/fcdb-tools) | `0.1.0` | Helper tools, utilities, and CLI for FCDB |

### Quick Start

Add FCDB to your `Cargo.toml`:

```toml
[dependencies]
fcdb-core = "0.1"
fcdb-cas = "0.1"
fcdb-graph = "0.1"
```

Basic usage example:

```rust
use fcdb_core::{Cid, Cap};
use fcdb_cas::PackCAS;
use fcdb_graph::GraphDB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize CAS
    let cas = PackCAS::new("data").await?;

    // Create GraphDB instance
    let graph = GraphDB::new(cas).await?;

    // Create a node
    let node_id = graph.create_node(b"Hello FCDB!".to_vec()).await?;
    println!("Created node with ID: {:?}", node_id);

    Ok(())
}
```

## Project Structure

```
fcdb/
├── src/                    # Main application source
├── crates/                 # All Rust crates
│   ├── fcdb-api/          # Public API
│   ├── fcdb-cas/          # Content-Addressable Storage
│   ├── fcdb-concur/       # Concurrency primitives
│   ├── fcdb-core/         # Core data structures
│   ├── fcdb-exec/         # Query execution engine
│   ├── fcdb-graph/        # Graph operations
│   ├── fcdb-tools/        # Helper tools and utilities
│   └── fcdb-validation/   # Validation and benchmarking
├── examples/               # Demo and example code
├── docs/                   # Documentation
│   ├── api/               # API documentation
│   ├── architecture/      # Architecture docs
│   ├── development/       # Development guide
│   ├── operations/        # Operations guide
│   ├── research/          # Research papers and analysis
│   └── validation/        # Validation and testing docs
├── charts/                # Kubernetes Helm charts
├── deploy/                # Deployment configurations
├── loadtest/              # Load testing scripts
├── scripts/               # Build and utility scripts
├── Cargo.toml             # Workspace configuration
├── Dockerfile             # Container build
└── README.md              # This file
```

## Getting Started

### Prerequisites

- Rust 1.70+
- Docker (for container builds)

### Building

```bash
# Clone the repository
git clone https://github.com/com-junkawasaki/fcdb.git
cd fcdb

# Build all crates
cargo build --release

# Run the application
cargo run --bin fcdb

# Run validation suite
cargo run --package fcdb-validation -- full
```

### Development

See [docs/development/](docs/development/) for detailed development setup and contribution guidelines.

## Documentation

FCDB provides comprehensive documentation for all aspects of the system:

### API Documentation

Detailed API references for each crate:

- **[Core API](docs/api/core.md)**: Fundamental data structures, cryptographic primitives, and utilities
- **[CAS API](docs/api/cas.md)**: Content-Addressable Storage operations, performance tuning, and monitoring
- **[Graph API](docs/api/graph.md)**: Graph data structures, traversal algorithms, and indexing
- **[API Interface](docs/api/fcdb-api.md)**: REST and GraphQL interfaces, authentication, and middleware

### Additional Documentation

- **[Architecture](docs/architecture/)**: System architecture, design principles, and mathematical foundations
- **[Development](docs/development/)**: Development setup, contribution guidelines, and coding standards
- **[Operations](docs/operations/)**: Deployment, monitoring, and operational procedures
- **[Research](docs/research/)**: Research papers, theoretical foundations, and evaluation results
- **[Validation](docs/validation/)**: Validation procedures, benchmarking results, and testing

### Generated Documentation

For complete API documentation with examples and detailed type information:

```bash
# Generate and open Rustdoc
cargo doc --open

# Or view online at docs.rs
# fcdb-core: https://docs.rs/fcdb-core
# fcdb-cas: https://docs.rs/fcdb-cas
# fcdb-graph: https://docs.rs/fcdb-graph
# fcdb-api: https://docs.rs/fcdb-api
```

## Community & Support

- **GitHub Repository**: [com-junkawasaki/fcdb](https://github.com/com-junkawasaki/fcdb)
- **Crates.io**: [fcdb](https://crates.io/crates/fcdb-core)
- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/com-junkawasaki/fcdb/issues)
- **Discussions**: [GitHub Discussions](https://github.com/com-junkawasaki/fcdb/discussions)

## License

Licensed under Apache License 2.0. See [LICENSE](LICENSE) for details.

---

**FCDB (Enishi)**: A Functorial–Categorical Database 🚀
