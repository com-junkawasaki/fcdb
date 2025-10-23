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
| [`fcdb-rdf`](https://crates.io/crates/fcdb-rdf) | `0.1.0` | RDF mapping and SPARQL query engine for FCDB |
| [`fcdb-shacl`](https://crates.io/crates/fcdb-shacl) | `0.1.0` | SHACL Core subset validator for FCDB |
| [`fcdb-cypher`](https://crates.io/crates/fcdb-cypher) | `0.1.0` | Cypher query parser and executor for FCDB |
| [`fcdb-gremlin`](https://crates.io/crates/fcdb-gremlin) | `0.1.0` | Gremlin-like graph traversal DSL for FCDB |
| [`fcdb-owl`](https://crates.io/crates/fcdb-owl) | `0.1.0` | OWL ontology parser and RDFS reasoner for FCDB |

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

## Use Cases

FCDB supports multiple query languages and provides comprehensive graph database functionality. Here are key use cases and examples:

### 1. Basic Graph Operations

```rust
use fcdb_graph::GraphDB;
use fcdb_cas::PackCAS;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cas = PackCAS::open("data").await?;
    let graph = GraphDB::new(cas).await;

    // Create nodes
    let alice = graph.create_node(br#"{"name": "Alice", "age": 30}"#).await?;
    let bob = graph.create_node(br#"{"name": "Bob", "age": 25}"#).await?;

    // Create relationships
    graph.create_edge(alice, bob, 1u32.into(), b"knows").await?;

    // Query the graph
    let alice_data = graph.get_node(alice).await?;
    println!("Alice: {:?}", alice_data);

    Ok(())
}
```

### 2. RDF/SPARQL Integration

FCDB can export graph data to RDF and execute SPARQL queries:

```rust
use fcdb_rdf::{RdfExporter, SparqlRunner};
use fcdb_graph::GraphDB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cas = PackCAS::open("data").await?;
    let graph = GraphDB::new(cas).await;

    // Create RDF data
    let person = graph.create_node(br#"{"type": "Person", "name": "Alice"}"#).await?;

    // Export to RDF N-Triples
    let exporter = RdfExporter::new(&graph, "https://example.org/");
    let ntriples = exporter.export_ntriples().await?;
    println!("RDF Export:\n{}", ntriples);

    // Execute SPARQL queries
    let runner = SparqlRunner::new(exporter);
    let query = r#"
        SELECT ?s ?p ?o
        WHERE {
            ?s ?p ?o .
        }
        LIMIT 5
    "#;
    let results = runner.execute(query).await?;
    println!("SPARQL Results: {}", results);

    Ok(())
}
```

### 3. SHACL Validation

Validate RDF data against SHACL shapes:

```rust
use fcdb_shacl::validate_shapes;
use fcdb_graph::GraphDB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cas = PackCAS::open("data").await?;
    let graph = GraphDB::new(cas).await;

    // Create data to validate
    graph.create_node(br#"{"name": "Alice", "email": "alice@example.com"}"#).await?;

    // SHACL shape definition
    let shapes = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        <PersonShape> a sh:NodeShape ;
            sh:targetClass <Person> ;
            sh:property [
                sh:path <name> ;
                sh:datatype xsd:string ;
                sh:minCount 1 ;
            ] ;
            sh:property [
                sh:path <email> ;
                sh:datatype xsd:string ;
                sh:pattern ".+@.+" ;
            ] .
    "#;

    // Validate data against shapes
    let report = validate_shapes(&graph, shapes, Default::default()).await?;
    println!("Validation conforms: {}", report.conforms);

    Ok(())
}
```

### 4. Cypher Queries

Execute declarative graph queries using Cypher:

```rust
use fcdb_cypher::execute_cypher;
use fcdb_graph::GraphDB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cas = PackCAS::open("data").await?;
    let graph = GraphDB::new(cas).await;

    // Create a social network
    let alice = graph.create_node(br#"{"name": "Alice", "type": "Person"}"#).await?;
    let bob = graph.create_node(br#"{"name": "Bob", "type": "Person"}"#).await?;
    let charlie = graph.create_node(br#"{"name": "Charlie", "type": "Person"}"#).await?;

    graph.create_edge(alice, bob, 1u32.into(), b"FRIENDS_WITH").await?;
    graph.create_edge(bob, charlie, 1u32.into(), b"FRIENDS_WITH").await?;

    // Cypher query: Find Alice's friends of friends
    let query = r#"
        MATCH (alice:Person)-[:FRIENDS_WITH]->(friend:Person)-[:FRIENDS_WITH]->(foaf:Person)
        WHERE alice.name = "Alice"
        RETURN DISTINCT foaf.name as friend_of_friend
    "#;

    let result = execute_cypher(query, &graph).await?;
    println!("Alice's friends of friends: {:?}", result.rows);

    Ok(())
}
```

### 5. Gremlin Graph Traversals

Perform complex graph traversals with Gremlin's fluent API:

```rust
use fcdb_gremlin::{execute_traversal, g};
use fcdb_graph::GraphDB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cas = PackCAS::open("data").await?;
    let graph = GraphDB::new(cas).await;

    // Create a knowledge graph
    let ai = graph.create_node(br#"{"name": "AI", "type": "Technology"}"#).await?;
    let ml = graph.create_node(br#"{"name": "Machine Learning", "type": "Field"}"#).await?;
    let nn = graph.create_node(br#"{"name": "Neural Networks", "type": "Algorithm"}"#).await?;

    graph.create_edge(ai, ml, 1u32.into(), b"USES").await?;
    graph.create_edge(ml, nn, 1u32.into(), b"IMPLEMENTS").await?;

    // Gremlin traversal: AI -> technologies -> algorithms
    let traversal = g()
        .V()  // Start from all vertices
        .has("name", serde_json::json!("AI"))  // Find AI node
        .out(None)  // Traverse to related technologies
        .out(None)  // Traverse to algorithms
        .values("name".to_string())  // Extract names
        .build();

    let result = execute_traversal(&graph, traversal).await?;
    println!("AI-related algorithms: {:?}", result.traversers);

    Ok(())
}
```

### 6. OWL Ontology Reasoning

Perform logical reasoning with OWL ontologies:

```rust
use fcdb_owl::classify_ontology;
use fcdb_graph::GraphDB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cas = PackCAS::open("data").await?;
    let graph = GraphDB::new(cas).await;

    // Create ontology data
    let person = graph.create_node(br#"{"id": "Person", "type": "Class"}"#).await?;
    let student = graph.create_node(br#"{"id": "Student", "type": "Class"}"#).await?;
    let alice = graph.create_node(br#"{"id": "Alice", "type": "Individual"}"#).await?;

    // Define subclass relationship
    graph.create_edge(student, person, 1u32.into(), br#"{"predicate": "rdfs:subClassOf"}"#).await?;
    // Alice is a Student
    graph.create_edge(alice, student, 2u32.into(), br#"{"predicate": "rdf:type"}"#).await?;

    // OWL ontology with reasoning rules
    let ontology = r#"
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

        <Student> rdfs:subClassOf <Person> .
    "#;

    // Perform reasoning and get inferred triples
    let inferred = classify_ontology(ontology, &graph).await?;
    println!("Inferred triples: {}", inferred.len());

    // Now Alice is also inferred to be a Person
    for triple in inferred {
        if triple.o.contains("Person") {
            println!("Inferred: {} is a Person", triple.s.0);
        }
    }

    Ok(())
}
```

### 7. Real-World Applications

#### Knowledge Graph Management
```rust
// Building and querying enterprise knowledge graphs
// - RDF data integration from multiple sources
// - SHACL validation for data quality
// - SPARQL for complex federated queries
```

#### Social Network Analysis
```rust
// Analyzing social connections and influence
// - Cypher for pattern matching and recommendations
// - Gremlin for complex relationship traversals
// - Graph algorithms for centrality and clustering
```

#### Semantic Web Applications
```rust
// Building semantic web applications
// - OWL reasoning for inference and classification
// - RDF/SPARQL for linked data integration
// - SHACL for schema validation and data governance
```

#### Recommendation Systems
```rust
// Collaborative filtering and content-based recommendations
// - Graph traversal for finding similar items/users
// - Path analysis for recommendation explanations
// - Temporal queries for trend analysis
```

### 8. API Integration

FCDB provides both REST and GraphQL APIs for all query languages:

```bash
# REST API examples
curl -X POST http://localhost:8080/sparql \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"}'

curl -X POST http://localhost:8080/cypher \
  -H "Content-Type: application/json" \
  -d '{"query": "MATCH (n) RETURN count(n)"}'

curl -X POST http://localhost:8080/gremlin \
  -H "Content-Type: application/json" \
  -d '{"start": "V", "steps": ["has", "type", "Person", "values", "name"]}'

curl -X POST http://localhost:8080/shacl/validate \
  -H "Content-Type: application/json" \
  -d '{"shapes": "@prefix sh: <http://www.w3.org/ns/shacl#> ..."}'
```

```graphql
# GraphQL API example
query {
  sparql(query: "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 5") {
    results
  }

  cypher(query: "MATCH (n) RETURN n") {
    columns
    rows
  }

  gremlin(input: { start: "V", steps: ["values", "name"] }) {
    traversers {
      current
      path
      value
    }
  }

  classifyOwl(input: { ontology: "@prefix rdfs: ..." }) {
    inferredCount
    triples
  }
}
```

## Project Structure

```
fcdb/
├── src/                    # Main application source
├── crates/                 # All Rust crates
│   ├── fcdb-api/          # Public API (GraphQL, REST)
│   ├── fcdb-cas/          # Content-Addressable Storage
│   ├── fcdb-concur/       # Concurrency primitives
│   ├── fcdb-core/         # Core data structures
│   ├── fcdb-exec/         # Query execution engine
│   ├── fcdb-graph/        # Graph operations
│   ├── fcdb-tools/        # Helper tools and utilities
│   ├── fcdb-rdf/          # RDF mapping and SPARQL queries
│   ├── fcdb-shacl/        # SHACL validation
│   ├── fcdb-cypher/       # Cypher query language
│   ├── fcdb-gremlin/      # Gremlin graph traversal
│   ├── fcdb-owl/          # OWL ontology reasoning
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
- **[RDF/SPARQL API](docs/api/rdf.md)**: RDF export/import, SPARQL query execution, and semantic data integration
- **[SHACL Validation API](docs/api/shacl.md)**: Shape validation, constraint checking, and data quality assurance
- **[Cypher Query API](docs/api/cypher.md)**: Declarative graph queries, pattern matching, and result processing
- **[Gremlin Traversal API](docs/api/gremlin.md)**: Graph traversal DSL, fluent API, and complex navigation patterns
- **[OWL Reasoning API](docs/api/owl.md)**: Ontology classification, logical inference, and knowledge expansion

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
# Core crates
# fcdb-core: https://docs.rs/fcdb-core
# fcdb-cas: https://docs.rs/fcdb-cas
# fcdb-graph: https://docs.rs/fcdb-graph
# fcdb-api: https://docs.rs/fcdb-api

# Query language crates
# fcdb-rdf: https://docs.rs/fcdb-rdf
# fcdb-shacl: https://docs.rs/fcdb-shacl
# fcdb-cypher: https://docs.rs/fcdb-cypher
# fcdb-gremlin: https://docs.rs/fcdb-gremlin
# fcdb-owl: https://docs.rs/fcdb-owl
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
