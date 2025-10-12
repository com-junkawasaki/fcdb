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

The project is composed of several Rust crates:
- `fcdb-core`: Core data structures and logic.
- `fcdb-cas`: Content-Addressable Storage implementation.
- `fcdb-graph`: Graph view and traversal logic.
- `fcdb-api`: Public API for interacting with the database.
- `fcdb-concur`: Concurrency primitives.
- `fcdb-exec`: Query execution engine.
- `fcdb-tools`: Helper tools and utilities.

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

## Validation

The system has undergone rigorous validation. For more details, see [docs/validation/](docs/validation/).
