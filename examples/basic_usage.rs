//! # FCDB Basic Usage Example
//!
//! This example demonstrates the basic usage of FCDB components:
//! - Creating a CAS instance
//! - Initializing a GraphDB
//! - Creating nodes and edges
//! - Performing graph traversals

use fcdb_core::Cid;
use fcdb_cas::PackCAS;
use fcdb_graph::GraphDB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 FCDB Basic Usage Example");
    println!("==========================");

    // Initialize Content-Addressable Storage
    println!("📦 Initializing CAS...");
    let cas = PackCAS::new("./example_data").await?;
    println!("✅ CAS initialized successfully");

    // Create GraphDB instance
    println!("📊 Creating GraphDB instance...");
    let graph = GraphDB::new(cas).await?;
    println!("✅ GraphDB created successfully");

    // Create some nodes
    println!("🔵 Creating nodes...");
    let user_node = graph.create_node(b"User: alice".to_vec()).await?;
    let post_node = graph.create_node(b"Post: Hello World!".to_vec()).await?;
    let comment_node = graph.create_node(b"Comment: Great post!".to_vec()).await?;

    println!("✅ Created nodes:");
    println!("  - User: {:?}", user_node);
    println!("  - Post: {:?}", post_node);
    println!("  - Comment: {:?}", comment_node);

    // Create edges between nodes
    println!("🔗 Creating edges...");
    graph.create_edge(user_node, post_node, fcdb_graph::LabelId(1), b"authored").await?;
    graph.create_edge(post_node, comment_node, fcdb_graph::LabelId(2), b"has_comment").await?;
    graph.create_edge(user_node, comment_node, fcdb_graph::LabelId(3), b"commented_on").await?;
    println!("✅ Created relationships");

    // Perform graph traversal
    println!("🔍 Performing 2-hop traversal from user...");
    let traversal_result = graph.traverse(user_node, None, 2, None).await?;
    println!("✅ Found {} nodes in traversal", traversal_result.nodes.len());

    // Display traversal results
    println!("📋 Traversal results:");
    for node_id in &traversal_result.nodes {
        if let Some(node_data) = graph.get_node(*node_id).await? {
            let content = String::from_utf8_lossy(&node_data);
            println!("  - {}: {}", node_id, content);
        }
    }

    println!("🎉 FCDB basic usage example completed successfully!");
    println!("\n💡 Next steps:");
    println!("  - Explore fcdb-api for GraphQL/gRPC interfaces");
    println!("  - Check fcdb-tools for benchmarking utilities");
    println!("  - Read the research paper for theoretical background");

    Ok(())
}
