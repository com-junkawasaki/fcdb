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

## Getting Started

*(TODO: Add instructions for building and running the project.)*

## Validation

The system has undergone rigorous validation. For more details, see [VALIDATION.md](./VALIDATION.md).
