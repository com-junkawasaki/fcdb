# FCDB Query Languages Support

FCDB provides comprehensive support for multiple graph query languages, with GraphDB as the canonical data model and RDF as a projection layer.

## Supported Query Languages

### 1. SPARQL (RDF Query Language)

**Status**: ✅ Implemented with oxigraph

**Features**:
- SELECT queries with projections and filters
- CONSTRUCT queries for RDF generation
- ASK queries for boolean results
- Full SPARQL 1.1 syntax support via oxigraph

**API Endpoints**:
- `POST /sparql` - Execute SPARQL queries
- GraphQL: `sparql(query: String!): String!`

**Example**:
```sparql
PREFIX foaf: <http://xmlns.com/foaf/0.1/>
SELECT ?name ?email
WHERE {
    ?person foaf:name ?name .
    ?person foaf:mbox ?email .
}
```

### 2. SHACL (Shapes Constraint Language)

**Status**: ✅ Implemented (Core subset)

**Features**:
- Node shapes and property shapes
- Datatype constraints (xsd:string, xsd:integer, etc.)
- Cardinality constraints (minCount, maxCount)
- Value constraints (in, pattern, class)
- Node kind constraints (IRI, blank node, literal)

**API Endpoints**:
- `POST /shacl/validate` - Validate RDF data against SHACL shapes
- GraphQL: `validateShacl(input: ShaclValidateInput!): ValidationReport!`

**Example**:
```turtle
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

person:PersonShape a sh:NodeShape ;
    sh:targetClass foaf:Person ;
    sh:property [
        sh:path foaf:name ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
    ] .
```

### 3. Cypher (Graph Query Language)

**Status**: ✅ Implemented (subset)

**Features**:
- MATCH patterns with nodes and relationships
- WHERE clauses with equality and range comparisons
- RETURN with property access
- Basic path traversal
- Node and relationship pattern matching

**API Endpoints**:
- `POST /cypher` - Execute Cypher queries
- GraphQL: `cypher(query: String!): CypherResult!`

**Example**:
```cypher
MATCH (p:Person)-[:KNOWS]->(friend)
WHERE p.age > 25
RETURN p.name, friend.name
```

### 4. Gremlin (Traversal Language)

**Status**: ✅ Implemented (DSL subset)

**Features**:
- Fluent traversal API in Rust
- Vertex and edge traversal (out, in)
- Property filtering (has)
- Value extraction (values)
- Path computation (path)
- Step composition

**API Endpoints**:
- `POST /gremlin` - Execute Gremlin traversals
- GraphQL: `gremlin(input: GremlinTraversalInput!): GremlinResult!`

**Example**:
```rust
use fcdb_gremlin::g;

let traversal = g()
    .V()
    .has("type".to_string(), json!("Person"))
    .out(Some("knows".to_string()))
    .values("name".to_string())
    .build();

let result = execute_traversal(&graph, traversal).await?;
```

### 5. OWL (Web Ontology Language)

**Status**: ✅ Implemented (RDFS/OWL-RL subset)

**Features**:
- RDFS vocabulary support (subClassOf, domain, range)
- Transitive subclass reasoning
- Property domain/range inference
- RDF materialization of inferred triples
- Ontology parsing (simplified)

**API Endpoints**:
- `POST /owl/classify` - Perform OWL reasoning and classification
- GraphQL: `classifyOwl(input: OwlClassifyInput!): OwlResult!`

**Example**:
```turtle
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:Person rdfs:subClassOf :Agent .
:Student rdfs:subClassOf :Person .

:hasName rdfs:domain :Agent .
:hasName rdfs:range xsd:string .
```

## Architecture Principles

### GraphDB as Canonical Model

All query languages operate on the same underlying `GraphDB` instance:
- **Nodes**: Represented as RIDs with associated data (JSON/CBOR)
- **Edges**: Represented as labeled relationships between nodes
- **Properties**: Stored as key-value pairs on nodes and edges

### RDF as Projection Layer

RDF serves as a bridge between different query paradigms:
- GraphDB ↔ RDF bidirectional mapping
- SPARQL/SHACL operate on RDF projection
- OWL reasoning materializes back to RDF triples

### Modular Implementation

Each query language is implemented in separate crates:
- `fcdb-rdf`: RDF/SPARQL support
- `fcdb-shacl`: SHACL validation
- `fcdb-cypher`: Cypher subset
- `fcdb-gremlin`: Gremlin DSL
- `fcdb-owl`: OWL reasoning

## Performance Considerations

### SPARQL
- Projection-based execution (only relevant data loaded)
- oxigraph provides optimized query evaluation
- Memory usage scales with query scope

### SHACL
- Validation performed against GraphDB directly
- Early termination on first violation (configurable)
- Efficient constraint checking via indexed access

### Cypher/Gremlin
- Direct GraphDB traversal (no projection overhead)
- Iterator-based evaluation for memory efficiency
- Optimized for graph-native operations

### OWL
- Rule-based forward chaining
- Fixed-point iteration with convergence detection
- Minimal memory overhead for RDFS reasoning

## Integration with REST/GraphQL APIs

All query languages are accessible via:
- **REST endpoints**: Direct HTTP POST with JSON payloads
- **GraphQL mutations/queries**: Typed schema integration
- **Unified error handling**: Consistent error responses
- **Metrics collection**: Query execution statistics

## Future Extensions

### Planned Enhancements
- **Full Cypher**: Complete Cypher 9 specification
- **Gremlin Language**: String-based Gremlin queries
- **SHACL Advanced**: SPARQL-based constraints
- **OWL Full**: Complete OWL 2 DL reasoning
- **Query Federation**: Cross-database queries

### Integration Opportunities
- **Apache TinkerPop**: Full Gremlin ecosystem
- **Neo4j**: Enterprise Cypher features
- **Jena/Fuseki**: Full SPARQL/SPARQL Update
- **HermiT/Pellet**: Complete OWL reasoning
