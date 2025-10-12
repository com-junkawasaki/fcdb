# FCDB API Interface

## Overview

`fcdb-api` provides REST and GraphQL interfaces for FCDB, built on top of the core components.

## REST API

### Server Setup

```rust
use fcdb_api::rest::RestServer;
use fcdb_cas::PackCAS;
use fcdb_graph::GraphDB;

// Initialize components
let cas = PackCAS::new("./data").await?;
let graph = GraphDB::new(cas).await?;

// Create REST server
let server = RestServer::new(graph);

// Start server
server.start("127.0.0.1:8080").await?;
```

### REST Endpoints

#### Health Check

```http
GET /health
```

Response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

#### Node Operations

```http
# Create node
POST /api/v1/nodes
Content-Type: application/json

{
  "data": "SGVsbG8gV29ybGQ="  // base64 encoded
}

# Get node
GET /api/v1/nodes/{id}

# Update node
PUT /api/v1/nodes/{id}
Content-Type: application/json

{
  "data": "VXBkYXRlZCBkYXRh"  // base64 encoded
}

# Delete node
DELETE /api/v1/nodes/{id}
```

#### Edge Operations

```http
# Create edge
POST /api/v1/edges
Content-Type: application/json

{
  "from": "node_id_1",
  "to": "node_id_2",
  "label": 1,
  "properties": "eyJrZXkiOiJ2YWx1ZSJ9"  // base64 encoded JSON
}

# Get edges from node
GET /api/v1/nodes/{id}/edges/outgoing

# Get edges to node
GET /api/v1/nodes/{id}/edges/incoming
```

#### Graph Traversal

```http
# Traverse graph
POST /api/v1/traverse
Content-Type: application/json

{
  "start_node": "node_id",
  "max_depth": 3,
  "labels": [1, 2],
  "direction": "outgoing"
}
```

## GraphQL API

### Schema Definition

```rust
use fcdb_api::graphql::{create_schema, GraphQLContext};
use async_graphql::Schema;

// Create GraphQL schema
let schema = create_schema();

// Create context
let context = GraphQLContext { graph };

// Execute query
let query = r#"
{
  node(id: "node_123") {
    id
    data
  }
}
"#;

let result = schema.execute(query).await?;
println!("{}", result.data);
```

### GraphQL Schema

```graphql
type Query {
  # Node operations
  node(id: ID!): Node
  nodes(limit: Int, offset: Int): [Node!]!

  # Edge operations
  edges(from: ID, to: ID, label: Int): [Edge!]!

  # Traversal operations
  traverse(
    startNode: ID!
    maxDepth: Int
    labels: [Int!]
    direction: TraversalDirection
  ): TraversalResult!

  # Search operations
  search(query: String!, limit: Int): [SearchResult!]!
}

type Mutation {
  # Node mutations
  createNode(input: CreateNodeInput!): Node!
  updateNode(input: UpdateNodeInput!): Node!
  deleteNode(id: ID!): Boolean!

  # Edge mutations
  createEdge(input: CreateEdgeInput!): Edge!
  deleteEdge(from: ID!, to: ID!, label: Int!): Boolean!
}

type Node {
  id: ID!
  data: String!
  createdAt: String!
  outgoingEdges: [Edge!]!
  incomingEdges: [Edge!]!
}

type Edge {
  id: ID!
  from: ID!
  to: ID!
  label: Int!
  properties: String
  createdAt: String!
}

type TraversalResult {
  nodes: [Node!]!
  edges: [Edge!]!
  totalNodes: Int!
  totalEdges: Int!
  maxDepth: Int!
}

type SearchResult {
  node: Node!
  score: Float!
  highlights: [String!]
}

enum TraversalDirection {
  OUTGOING
  INCOMING
  BOTH
}

input CreateNodeInput {
  data: String!
}

input UpdateNodeInput {
  id: ID!
  data: String!
}

input CreateEdgeInput {
  from: ID!
  to: ID!
  label: Int!
  properties: String
}
```

## Authentication & Authorization

### Capability-Based Access Control

```rust
use fcdb_api::auth::CapabilityAuth;
use fcdb_core::Cap;

// Create capability for user
let user_cap = Cap::new(0x1000, 0x1000, 0x07); // read, write, execute

// Create auth middleware
let auth = CapabilityAuth::new(user_cap);

// Use in request handling
let result = auth.check_access(&request, &resource_id).await?;
if result.allowed {
    // Proceed with operation
} else {
    return Err(AuthError::InsufficientPermissions);
}
```

### JWT Authentication

```rust
use fcdb_api::auth::JwtAuth;

// Create JWT authenticator
let jwt_auth = JwtAuth::new("your-secret-key".to_string());

// Validate JWT token
let claims = jwt_auth.validate_token(token).await?;

// Extract user capabilities from claims
let user_caps = claims.capabilities;
```

## Middleware

### CORS Configuration

```rust
use fcdb_api::middleware::CorsConfig;

// Configure CORS
let cors = CorsConfig {
    allowed_origins: vec!["https://example.com".to_string()],
    allowed_methods: vec!["GET", "POST", "PUT", "DELETE".to_string()],
    allowed_headers: vec!["Content-Type", "Authorization".to_string()],
    allow_credentials: true,
};

// Apply to server
server.with_cors(cors);
```

### Rate Limiting

```rust
use fcdb_api::middleware::RateLimitConfig;

// Configure rate limiting
let rate_limit = RateLimitConfig {
    requests_per_minute: 1000,
    burst_limit: 100,
};

// Apply to server
server.with_rate_limiting(rate_limit);
```

### Logging

```rust
use fcdb_api::middleware::LoggingConfig;

// Configure logging
let logging = LoggingConfig {
    level: "info".to_string(),
    format: "json".to_string(),
    include_body: false,
};

// Apply to server
server.with_logging(logging);
```

## Error Handling

### REST API Errors

```json
{
  "error": {
    "code": "NODE_NOT_FOUND",
    "message": "Node with ID '123' not found",
    "details": {
      "node_id": "123",
      "timestamp": "2024-01-01T00:00:00Z"
    }
  },
  "request_id": "req_abc123"
}
```

### GraphQL Errors

```json
{
  "errors": [
    {
      "message": "Node not found",
      "locations": [{"line": 3, "column": 5}],
      "path": ["node"],
      "extensions": {
        "code": "NODE_NOT_FOUND",
        "node_id": "123"
      }
    }
  ]
}
```

## Client Libraries

### Rust Client

```rust
use fcdb_api::client::FcdbClient;

// Create client
let client = FcdbClient::new("http://localhost:8080").await?;

// Node operations
let node = client.create_node(b"Hello FCDB!".to_vec()).await?;
let retrieved = client.get_node(node.id).await?;

// Graph operations
let result = client.traverse(node.id, 2, None).await?;
println!("Found {} nodes", result.nodes.len());
```

### JavaScript/TypeScript Client

```typescript
import { FcdbClient } from '@fcdb/client';

const client = new FcdbClient('http://localhost:8080');

// Node operations
const node = await client.createNode('Hello FCDB!');
const retrieved = await client.getNode(node.id);

// GraphQL queries
const result = await client.graphql(`
  {
    node(id: "${node.id}") {
      id
      data
    }
  }
`);
```

## Performance Optimization

### Connection Pooling

```rust
// Configure connection pool
let pool_config = ConnectionPoolConfig {
    max_connections: 100,
    min_connections: 10,
    connection_timeout: Duration::from_secs(30),
};

let client = FcdbClient::with_pool("http://localhost:8080", pool_config).await?;
```

### Caching

```rust
// Enable response caching
let cache_config = CacheConfig {
    ttl: Duration::from_secs(300), // 5 minutes
    max_size: 1000, // 1000 entries
};

let client = client.with_caching(cache_config);
```

### Compression

```rust
// Enable gzip compression
let client = client.with_compression(true);
```

## Monitoring

### Metrics Collection

```rust
use fcdb_api::metrics::MetricsCollector;

// Create metrics collector
let metrics = MetricsCollector::new();

// Collect metrics
let snapshot = metrics.collect().await?;
println!("Requests per second: {}", snapshot.requests_per_second);
println!("Average response time: {:.2}ms", snapshot.avg_response_time_ms);
println!("Error rate: {:.2}%", snapshot.error_rate * 100.0);
```

### Health Checks

```rust
// Comprehensive health check
let health = server.health_check().await?;
println!("Status: {}", health.status);
println!("Checks: {}", health.checks.len());

// Individual component checks
for check in &health.checks {
    println!("{}: {}", check.name, check.status);
}
```

## Configuration

### Server Configuration

```toml
[server]
host = "127.0.0.1"
port = 8080
workers = 8

[api]
max_request_size = "10MB"
timeout_seconds = 30

[security]
jwt_secret = "your-secret-key"
capability_cache_size = 10000

[monitoring]
metrics_enabled = true
health_check_interval = 30
```

### Client Configuration

```rust
let config = ClientConfig {
    endpoint: "http://localhost:8080".to_string(),
    timeout: Duration::from_secs(30),
    retries: 3,
    retry_delay: Duration::from_millis(100),
    ..Default::default()
};

let client = FcdbClient::with_config(config).await?;
```

## Best Practices

### API Design

1. **Use appropriate HTTP methods**: GET for reads, POST for creates, PUT for updates, DELETE for deletes
2. **Consistent error responses**: Use standard HTTP status codes and error formats
3. **Version your APIs**: Use URL versioning (e.g., `/api/v1/`)
4. **Pagination**: Implement pagination for list endpoints

### Security

1. **Validate inputs**: Always validate and sanitize user inputs
2. **Use HTTPS**: Encrypt all communications in production
3. **Rate limiting**: Implement appropriate rate limits
4. **Audit logging**: Log all important operations

### Performance

1. **Connection pooling**: Reuse connections to reduce overhead
2. **Caching**: Cache frequently accessed data
3. **Compression**: Enable response compression
4. **Async operations**: Use async/await for non-blocking operations

## Troubleshooting

### Common Issues

#### Connection Refused
```
Cause: Server not running or wrong address
Solution: Check server status and configuration
```

#### Authentication Failed
```
Cause: Invalid or expired credentials
Solution: Refresh tokens or check capability permissions
```

#### Timeout Errors
```
Cause: Slow queries or network issues
Solution: Optimize queries or increase timeout values
```

#### Rate Limited
```
Cause: Too many requests
Solution: Implement exponential backoff or reduce request frequency
```

## API Reference

See the generated Rustdoc documentation for complete API details:

```bash
cargo doc --open --package fcdb-api
```
