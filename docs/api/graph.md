# FCDB Graph API

## Overview

`fcdb-graph` provides graph data structures and efficient traversal algorithms optimized for categorical database operations.

## GraphDB

Main graph database implementation:

```rust
use fcdb_graph::GraphDB;
use fcdb_cas::PackCAS;

// Create graph instance
let cas = PackCAS::new("./data").await?;
let graph = GraphDB::new(cas).await?;
```

## Node Operations

### Creating Nodes

```rust
// Create a node with data
let node_id = graph.create_node(b"User: Alice".to_vec()).await?;
println!("Created node: {:?}", node_id);

// Create multiple nodes
let users = vec![
    b"User: Bob".to_vec(),
    b"User: Carol".to_vec(),
    b"User: Dave".to_vec(),
];

let user_ids: Vec<_> = users.into_iter()
    .map(|data| graph.create_node(data))
    .collect::<Vec<_>>();

let user_ids = futures::future::join_all(user_ids).await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
```

### Retrieving Nodes

```rust
// Get node data
if let Some(data) = graph.get_node(node_id).await? {
    let content = String::from_utf8_lossy(&data);
    println!("Node data: {}", content);
}

// Check node existence
let exists = graph.has_node(node_id).await?;
if exists {
    println!("Node exists");
}
```

### Updating Nodes

```rust
// Update node data
let new_data = b"User: Alice (updated)".to_vec();
graph.update_node(node_id, new_data).await?;
```

### Deleting Nodes

```rust
// Delete a node (also removes connected edges)
graph.delete_node(node_id).await?;
```

## Edge Operations

### Creating Edges

```rust
use fcdb_graph::LabelId;

// Create labeled edges between nodes
let user_id = user_ids[0];
let post_id = graph.create_node(b"Post: Hello World!".to_vec()).await?;

// Create "authored" relationship
graph.create_edge(user_id, post_id, LabelId(1), b"authored").await?;

// Create "likes" relationship
let other_user = user_ids[1];
graph.create_edge(other_user, post_id, LabelId(2), b"likes").await?;
```

### Querying Edges

```rust
// Get all edges from a node
let outgoing = graph.get_edges_from(user_id).await?;
println!("User has {} outgoing relationships", outgoing.len());

// Get all edges to a node
let incoming = graph.get_edges_to(post_id).await?;
println!("Post has {} incoming relationships", incoming.len());

// Get edges with specific label
let authored_posts = graph.get_edges_with_label(user_id, LabelId(1)).await?;
```

### Edge Properties

```rust
// Edges can have properties
let properties = b"{\"timestamp\": \"2024-01-01T00:00:00Z\"}".to_vec();
graph.create_edge(user_id, post_id, LabelId(1), &properties).await?;

// Retrieve edge properties
let edges = graph.get_edges_from(user_id).await?;
for (to_node, label, props) in edges {
    if !props.is_empty() {
        let props_str = String::from_utf8_lossy(&props);
        println!("Edge to {:?} has properties: {}", to_node, props_str);
    }
}
```

## Graph Traversal

### Basic Traversal

```rust
// Traverse graph from a starting node
let result = graph.traverse(user_id, None, 2, None).await?;
println!("Found {} nodes within 2 hops", result.nodes.len());

// Traverse with specific edge labels
let labels = Some(vec![LabelId(1), LabelId(2)]); // authored, likes
let result = graph.traverse(user_id, labels, 3, None).await?;
```

### Advanced Traversal Options

```rust
use fcdb_graph::{TraversalConfig, TraversalDirection};

// Configure traversal
let config = TraversalConfig {
    max_depth: 3,
    max_nodes: 1000,
    direction: TraversalDirection::Outbound,
    labels: Some(vec![LabelId(1)]), // Only "authored" relationships
    ..Default::default()
};

let result = graph.traverse_with_config(user_id, config).await?;
```

### Path Finding

```rust
// Find shortest path between nodes
if let Some(path) = graph.shortest_path(start_node, end_node, None).await? {
    println!("Shortest path has {} hops", path.len() - 1);
    for (i, node_id) in path.iter().enumerate() {
        println!("Hop {}: {:?}", i, node_id);
    }
}
```

### Bidirectional Search

```rust
// Efficient bidirectional search for large graphs
let result = graph.bidirectional_search(start_node, end_node, None).await?;
if let Some(path) = result.path {
    println!("Found path with {} hops", path.len() - 1);
}
```

## Graph Algorithms

### Connected Components

```rust
// Find connected components
let components = graph.connected_components().await?;
println!("Graph has {} connected components", components.len());

// Get largest component
let largest = components.iter().max_by_key(|c| c.len()).unwrap();
println!("Largest component has {} nodes", largest.len());
```

### PageRank

```rust
// Calculate PageRank scores
let pagerank = graph.pagerank(0.85, 100, None).await?;
for (node_id, score) in pagerank.iter() {
    println!("Node {:?} has PageRank score: {:.4}", node_id, score);
}
```

### Community Detection

```rust
// Detect communities using label propagation
let communities = graph.label_propagation(100).await?;
println!("Found {} communities", communities.len());

// Get community membership
for (node_id, community_id) in communities.iter() {
    println!("Node {:?} belongs to community {}", node_id, community_id);
}
```

## Indexing and Optimization

### Path Indexes

```rust
// Create path index for frequent query patterns
graph.create_path_index("/users/*/posts", "UserPosts").await?;

// Query using indexed paths
let user_posts = graph.query_path("/users/123/posts").await?;
println!("User 123 has {} posts", user_posts.len());
```

### Materialized Views

```rust
// Create materialized view for complex traversals
let view_name = "user_social_network";
let traversal_spec = r#"
{
  "start_pattern": "/users/*",
  "traversal": {
    "depth": 2,
    "labels": ["follows", "friends"]
  }
}
"#;

graph.create_materialized_view(view_name, traversal_spec).await?;
```

## Temporal Graph Operations

### Time-Travel Queries

```rust
use fcdb_graph::Timestamp;

// Query graph at specific point in time
let timestamp = Timestamp(1640995200); // 2024-01-01
let historical_graph = graph.at_time(timestamp).await?;

// Traverse historical state
let result = historical_graph.traverse(user_id, None, 2, None).await?;
```

### Versioned Operations

```rust
// Create versioned node
let versioned_node = graph.create_versioned_node(b"data v1".to_vec()).await?;

// Update with versioning
graph.update_versioned_node(versioned_node, b"data v2".to_vec()).await?;

// Get version history
let history = graph.get_version_history(versioned_node).await?;
println!("Node has {} versions", history.len());
```

## Bulk Operations

### Batch Node Creation

```rust
// Efficient batch node creation
let node_data = vec![
    b"Node 1".to_vec(),
    b"Node 2".to_vec(),
    b"Node 3".to_vec(),
];

let node_ids = graph.create_nodes_batch(node_data).await?;
println!("Created {} nodes", node_ids.len());
```

### Batch Edge Creation

```rust
// Batch edge creation
let edges = vec![
    (node1, node2, LabelId(1), b"rel1".to_vec()),
    (node2, node3, LabelId(2), b"rel2".to_vec()),
    (node3, node1, LabelId(3), b"rel3".to_vec()),
];

graph.create_edges_batch(edges).await?;
```

## Performance Optimization

### Memory Management

```rust
// Configure memory usage
graph.set_memory_limit(1024 * 1024 * 1024).await?; // 1GB

// Monitor memory usage
let mem_stats = graph.memory_stats().await?;
println!("Graph memory usage: {} MB", mem_stats.used_mb);
```

### Caching Strategies

```rust
// Enable query result caching
graph.enable_caching(true).await?;

// Configure cache size
graph.set_cache_size(100_000).await?; // 100k entries

// Cache statistics
let cache_stats = graph.cache_stats().await?;
println!("Cache hit rate: {:.2}%", cache_stats.hit_rate * 100.0);
```

### Parallel Processing

```rust
// Configure parallelism
graph.set_parallelism(8).await?; // 8 worker threads

// Parallel traversal
let result = graph.parallel_traverse(user_id, None, 3, None).await?;
```

## Monitoring and Statistics

### Graph Statistics

```rust
// Get comprehensive graph statistics
let stats = graph.stats().await?;
println!("Graph Statistics:");
println!("  Nodes: {}", stats.node_count);
println!("  Edges: {}", stats.edge_count);
println!("  Labels: {}", stats.label_count);
println!("  Density: {:.4}", stats.density);
println!("  Average degree: {:.2}", stats.avg_degree);
```

### Performance Metrics

```rust
// Query performance metrics
let metrics = graph.performance_metrics().await?;
println!("Query Performance:");
println!("  Avg traversal time: {:.2}ms", metrics.avg_traversal_ms);
println!("  Cache hit rate: {:.2}%", metrics.cache_hit_rate * 100.0);
println!("  I/O operations: {}", metrics.io_operations);
```

## Error Handling

Graph operations can return various error types:

```rust
use fcdb_graph::GraphError;

match graph.create_node(data).await {
    Ok(node_id) => println!("Created node: {:?}", node_id),
    Err(GraphError::NodeAlreadyExists) => println!("Node already exists"),
    Err(GraphError::InvalidNodeId) => println!("Invalid node ID"),
    Err(GraphError::StorageError(e)) => println!("Storage error: {}", e),
    Err(e) => println!("Other error: {}", e),
}
```

## Best Practices

### Data Modeling

1. **Consistent Labeling**: Use consistent edge labels across your graph
2. **Hierarchical IDs**: Design node IDs that reflect data hierarchy
3. **Property Schemas**: Establish consistent property formats for edge data

### Performance

1. **Index Frequently Queried Paths**: Create path indexes for common query patterns
2. **Use Appropriate Traversal Depths**: Limit traversal depth to prevent exponential explosion
3. **Batch Operations**: Use batch APIs for bulk operations
4. **Monitor Memory Usage**: Set appropriate memory limits and monitor usage

### Reliability

1. **Regular Backups**: Backup graph data regularly
2. **Version Control**: Use versioning for important data changes
3. **Error Handling**: Implement proper error handling for all operations
4. **Monitoring**: Monitor performance metrics and error rates

## API Reference

See the generated Rustdoc documentation for complete API details:

```bash
cargo doc --open --package fcdb-graph
```
