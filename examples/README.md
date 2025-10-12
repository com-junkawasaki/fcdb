# FCDB Examples

This directory contains example code demonstrating various ways to use FCDB components.

## Examples

### Basic Usage (`basic_usage.rs`)

A simple example showing the fundamental operations:
- Initialize Content-Addressable Storage (CAS)
- Create a GraphDB instance
- Create nodes and edges
- Perform basic graph traversals

Run with:
```bash
cargo run --example basic_usage
```

### REST API (`rest_api.rs`)

Demonstrates how to build a REST API using FCDB:
- Axum-based HTTP server
- CRUD operations for nodes and edges
- Graph traversal endpoints
- Health check endpoint

Run with:
```bash
cargo run --example rest_api
```

Then test with:
```bash
# Create a node
curl -X POST http://localhost:3000/nodes \
  -H 'Content-Type: application/json' \
  -d '{"data":"Hello FCDB!"}'

# Get health status
curl http://localhost:3000/health
```

### GraphQL API (`graphql_api.rs`)

Shows how to set up a GraphQL API:
- GraphQL schema definition
- Query and Mutation resolvers
- Node and edge operations
- Graph traversal queries

This example demonstrates the schema structure. For a full GraphQL server, use the `fcdb-api` crate.

### Legacy Examples

- `phase_c_demo.rs` - Phase C implementation demo
- `phase_d_demo.rs` - Phase D implementation demo
- `simple_validate.rs` - Basic validation example
- `validate_system.rs` - System validation utilities

## Running Examples

All examples can be run using:

```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example basic_usage
cargo run --example rest_api
```

## Dependencies

Examples use the following FCDB crates:
- `fcdb-core` - Core data structures
- `fcdb-cas` - Content-addressable storage
- `fcdb-graph` - Graph operations
- `fcdb-api` - API interfaces (REST/GraphQL)

Make sure to add these to your `Cargo.toml` when creating your own applications.

## Contributing

When adding new examples:
1. Follow Rust naming conventions
2. Include comprehensive documentation
3. Add error handling
4. Update this README

## Support

For questions or issues, please check:
- [FCDB Documentation](../docs/)
- [GitHub Issues](https://github.com/com-junkawasaki/fcdb/issues)
- [Crates.io](https://crates.io/crates/fcdb-core)
