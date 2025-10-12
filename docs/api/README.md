# Own-CFA-Enishi API Reference

## Overview

Own-CFA-Enishi provides both REST HTTP APIs and GraphQL interfaces for database operations. All APIs are secured with Own+CFA capability-based access control.

## HTTP REST API

### Base URL
```
http://localhost:8080
```

### Authentication
All API requests require valid capability tokens in the `Authorization` header:
```
Authorization: Bearer <capability-token>
```

### Endpoints

#### Service Information

##### `GET /`
Returns basic service information.

**Response:**
```json
{
  "service": "Own-CFA-Enishi",
  "version": "0.1.0",
  "description": "Categorical Database with Ownership & Capability Security",
  "endpoints": {
    "health": "/health",
    "ready": "/ready",
    "metrics": "/metrics",
    "version": "/version",
    "status": "/status"
  }
}
```

#### Health Monitoring

##### `GET /health`
Performs comprehensive health check.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": 1640995200,
  "uptime_seconds": 3600,
  "checks": {
    "system": "healthy",
    "storage": "healthy",
    "memory": "healthy",
    "connections": "healthy"
  }
}
```

**Status Codes:**
- `200`: Healthy
- `503`: Unhealthy

##### `GET /ready`
Checks if service is ready to accept requests.

**Response:**
```json
{
  "status": "ready",
  "message": "System is ready to accept requests"
}
```

#### Metrics & Monitoring

##### `GET /metrics`
Returns Prometheus-format metrics.

**Response:**
```
# HELP enishi_query_count_total Total number of queries processed
# TYPE enishi_query_count_total counter
enishi_query_count_total 1500

# HELP enishi_query_duration_seconds Query duration in seconds
# TYPE enishi_query_duration_seconds histogram
enishi_query_duration_seconds_sum 125.5
enishi_query_duration_seconds_count 1500

# HELP enishi_memory_usage_bytes Current memory usage
# TYPE enishi_memory_usage_bytes gauge
enishi_memory_usage_bytes 2147483648
```

##### `GET /status`
Returns comprehensive system status.

**Response:**
```json
{
  "status": "operational",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "performance": {
    "queries_per_second": 1200.5,
    "cache_hit_ratio": 0.985,
    "memory_usage_mb": 2048,
    "active_connections": 150
  },
  "configuration": {
    "port": 8080,
    "workers": 8,
    "storage_path": "./data",
    "adaptive_optimization": true
  },
  "phases": {
    "A": "completed",
    "B": "completed",
    "C": "completed",
    "D": "completed",
    "PROD": "in_progress"
  }
}
```

##### `GET /version`
Returns version and build information.

**Response:**
```json
{
  "service": "Own-CFA-Enishi",
  "version": "0.1.0",
  "build_date": "2024-10-12T10:30:00Z",
  "git_commit": "a1b2c3d4e5f6",
  "rust_version": "1.70.0",
  "target": "x86_64-unknown-linux-gnu"
}
```

## GraphQL API

### Endpoint
```
POST /graphql
```

### Schema Definition

```graphql
type Query {
  node(id: ID!): Node
  nodeAt(id: ID!, asOf: String!): Node
  traverse(input: TraverseInput!): [TraversalResult!]!
  search(query: String!): [SearchResult!]!
}

type Mutation {
  createNode(input: CreateNodeInput!): Node!
  updateNode(input: UpdateNodeInput!): Node!
  createEdge(input: CreateEdgeInput!): GraphEdge!
}

type Node {
  id: ID!
  data: String!
  createdAt: String!
}

type GraphEdge {
  from: ID!
  to: ID!
  label: String!
  properties: String!
}

type TraversalResult {
  node: Node!
  depth: Int!
}

type SearchResult {
  node: Node!
  score: Float!
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
  label: String!
  properties: String!
}

input TraverseInput {
  from: ID!
  labels: [String!]
  maxDepth: Int!
  asOf: String
}
```

### Example Queries

#### Get Node by ID
```graphql
query {
  node(id: "123") {
    id
    data
    createdAt
  }
}
```

#### Temporal Query (as_of)
```graphql
query {
  nodeAt(id: "123", asOf: "2024-01-01T00:00:00Z") {
    id
    data
    createdAt
  }
}
```

#### Graph Traversal
```graphql
query {
  traverse(input: {
    from: "123"
    labels: ["follows", "friend"]
    maxDepth: 3
  }) {
    node {
      id
      data
    }
    depth
  }
}
```

#### Text Search
```graphql
query {
  search(query: "rust programming") {
    node {
      id
      data
    }
    score
  }
}
```

#### Create Node
```graphql
mutation {
  createNode(input: {
    data: "{\"name\": \"Alice\", \"age\": 30}"
  }) {
    id
    data
    createdAt
  }
}
```

#### Create Edge
```graphql
mutation {
  createEdge(input: {
    from: "123"
    to: "456"
    label: "follows"
    properties: "{\"since\": \"2024-01-01\"}"
  }) {
    from
    to
    label
    properties
  }
}
```

## Capability-Based Security

### Capability Tokens

All operations require valid capability tokens that specify:
- **Resource bounds**: Which data can be accessed
- **Permission flags**: What operations are allowed
- **Temporal limits**: When access expires
- **Delegation rights**: Whether capabilities can be shared

### Example Capability Token
```json
{
  "cid": "abc123...",
  "cap": {
    "base": 1000,
    "len": 5000,
    "perms": ["read", "write"],
    "expires": 1640995200
  },
  "proof": "def456..."
}
```

### Permission Flags
- `read`: Can read/query data
- `write`: Can create/modify data
- `execute`: Can execute operations
- `derive`: Can create derived capabilities
- `delegate`: Can delegate capabilities to others

## Error Handling

### HTTP Status Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | OK | Success |
| 400 | Bad Request | Invalid request parameters |
| 401 | Unauthorized | Missing or invalid capability |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource not found |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Error | Server error |
| 503 | Unavailable | Service temporarily unavailable |

### Error Response Format
```json
{
  "error": {
    "code": "CAPABILITY_DENIED",
    "message": "Insufficient permissions for operation",
    "details": {
      "required": ["read", "write"],
      "provided": ["read"]
    }
  },
  "request_id": "req_12345"
}
```

## Rate Limiting

### Default Limits
- **Queries**: 1000 requests/second
- **Mutations**: 100 requests/second
- **Administrative**: 10 requests/second

### Headers
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 950
X-RateLimit-Reset: 1640995200
```

## Data Formats

### JSON Data Format
All data is stored and returned as JSON strings:
```json
{
  "user_id": "12345",
  "name": "Alice Johnson",
  "email": "alice@example.com",
  "created_at": "2024-01-01T00:00:00Z",
  "metadata": {
    "source": "registration",
    "verified": true
  }
}
```

### Temporal Queries
Timestamps use RFC3339 format:
```json
{
  "as_of": "2024-01-01T00:00:00Z",
  "before": "2024-12-31T23:59:59Z",
  "after": "2024-01-01T00:00:00Z"
}
```

## SDKs and Libraries

### Official SDKs
- **Rust**: Full native implementation
- **TypeScript/JavaScript**: Node.js and browser support
- **Python**: Async/await support with type hints
- **Go**: High-performance concurrent operations

### Example Usage (TypeScript)
```typescript
import { EnishiClient } from '@enishi/client';

const client = new EnishiClient({
  endpoint: 'http://localhost:8080',
  capability: process.env.ENISHI_CAPABILITY
});

// Query node
const node = await client.getNode('123');

// Create node
const newNode = await client.createNode({
  name: 'Bob',
  age: 25
});

// Graph traversal
const results = await client.traverse({
  from: '123',
  labels: ['follows'],
  maxDepth: 2
});
```

## Best Practices

### Query Optimization
1. **Use path signatures** for repeated query patterns
2. **Specify temporal bounds** when possible
3. **Batch operations** to reduce round trips
4. **Leverage caching** through query keys

### Security Practices
1. **Minimal capabilities**: Request only needed permissions
2. **Short-lived tokens**: Use short expiration times
3. **Audit logging**: Monitor all operations
4. **Regular rotation**: Rotate capability tokens regularly

### Performance Tips
1. **Connection pooling**: Reuse connections
2. **Query batching**: Combine multiple operations
3. **Result pagination**: Limit result sizes
4. **Caching strategies**: Cache frequent queries

## Troubleshooting

### Common Issues

#### Authentication Errors
```
Error: CAPABILITY_DENIED
```
**Solution**: Verify capability token validity and permissions

#### Performance Issues
```
Error: QUERY_TIMEOUT
```
**Solution**: Simplify query, add indexes, or increase timeout

#### Connection Issues
```
Error: CONNECTION_REFUSED
```
**Solution**: Check server status and network connectivity

### Debug Mode

Enable debug logging:
```bash
export RUST_LOG=debug
./enishi
```

### Health Checks

Monitor system health:
```bash
# Continuous health monitoring
while true; do
  curl -s http://localhost:8080/health | jq .status
  sleep 30
done
```

## Version Compatibility

### API Versions
- **v1.0**: Current stable API
- **Breaking changes**: Major version increments only
- **Deprecation**: 6-month deprecation period

### Data Compatibility
- **Forward compatible**: New versions can read old data
- **Migration tools**: Automated data migration scripts
- **Backup requirements**: Always backup before upgrades

---

For more detailed technical information, see the [Architecture Guide](../architecture/README.md) and [Operations Manual](../operations/README.md).
