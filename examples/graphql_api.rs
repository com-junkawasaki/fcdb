//! # FCDB GraphQL API Example
//!
//! This example demonstrates how to create a GraphQL API using FCDB
//! with the fcdb-api crate. It provides:
//! - GraphQL schema with Node and Edge types
//! - Queries for nodes, edges, and traversals
//! - Mutations for creating nodes and edges

use async_graphql::{
    Context, EmptySubscription, Object, Schema, SimpleObject, ID,
};
use fcdb_api::GraphQLApi;
use fcdb_cas::PackCAS;
use fcdb_core::Rid;
use fcdb_graph::GraphDB;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(SimpleObject)]
struct Node {
    id: ID,
    data: String,
}

#[derive(SimpleObject)]
struct Edge {
    from_id: ID,
    to_id: ID,
    label: String,
    properties: Option<String>,
}

#[derive(SimpleObject)]
struct TraversalResult {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    total_nodes: usize,
    total_edges: usize,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get a node by ID
    async fn node(&self, ctx: &Context<'_>, id: ID) -> Option<Node> {
        let app_ctx = ctx.data::<AppContext>().ok()?;
        let graph = app_ctx.graph.read().await;

        let rid = Rid(id.parse().ok()?);
        if let Ok(Some(data)) = graph.get_node(rid).await {
            let content = String::from_utf8_lossy(&data).to_string();
            Some(Node {
                id,
                data: content,
            })
        } else {
            None
        }
    }

    /// Traverse the graph from a starting node
    async fn traverse(
        &self,
        ctx: &Context<'_>,
        from_id: ID,
        hops: Option<i32>,
    ) -> Option<TraversalResult> {
        let app_ctx = ctx.data::<AppContext>().ok()?;
        let graph = app_ctx.graph.read().await;

        let rid = Rid(from_id.parse().ok()?);
        let max_hops = hops.unwrap_or(2) as usize;

        if let Ok(result) = graph.traverse(rid, None, max_hops, None).await {
            let mut nodes = Vec::new();
            let mut edges = Vec::new();

            // Collect nodes
            for &node_id in &result.nodes {
                if let Ok(Some(data)) = graph.get_node(node_id).await {
                    let content = String::from_utf8_lossy(&data).to_string();
                    nodes.push(Node {
                        id: node_id.0.to_string().into(),
                        data: content,
                    });
                }
            }

            // Collect edges (simplified - would need to get actual edge data)
            for &(from, to, _) in &result.edges {
                edges.push(Edge {
                    from_id: from.0.to_string().into(),
                    to_id: to.0.to_string().into(),
                    label: "connected".to_string(), // Simplified
                    properties: None,
                });
            }

            Some(TraversalResult {
                nodes,
                edges,
                total_nodes: result.nodes.len(),
                total_edges: result.edges.len(),
            })
        } else {
            None
        }
    }

    /// Get database statistics
    async fn stats(&self, ctx: &Context<'_>) -> serde_json::Value {
        let app_ctx = ctx.data::<AppContext>().ok();

        serde_json::json!({
            "status": "operational",
            "service": "FCDB GraphQL API",
            "version": "0.1.0"
        })
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new node
    async fn create_node(&self, ctx: &Context<'_>, data: String) -> Option<Node> {
        let app_ctx = ctx.data::<AppContext>().ok()?;
        let mut graph = app_ctx.graph.write().await;

        if let Ok(rid) = graph.create_node(data.clone().into_bytes()).await {
            Some(Node {
                id: rid.0.to_string().into(),
                data,
            })
        } else {
            None
        }
    }

    /// Create an edge between nodes
    async fn create_edge(
        &self,
        ctx: &Context<'_>,
        from_id: ID,
        to_id: ID,
        label: String,
        properties: Option<String>,
    ) -> Option<Edge> {
        let app_ctx = ctx.data::<AppContext>().ok()?;
        let graph = app_ctx.graph.read().await;

        let from_rid = Rid(from_id.parse().ok()?);
        let to_rid = Rid(to_id.parse().ok()?);
        let label_id = fcdb_graph::LabelId(1); // Simplified label mapping
        let props = properties.unwrap_or_default().into_bytes();

        if graph.create_edge(from_rid, to_rid, label_id, &props).await.is_ok() {
            Some(Edge {
                from_id,
                to_id,
                label,
                properties,
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct AppContext {
    graph: Arc<RwLock<GraphDB>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 FCDB GraphQL API Example");
    println!("===========================");

    // Initialize FCDB components
    let cas = PackCAS::new("./graphql_example_data").await?;
    let graph = Arc::new(RwLock::new(GraphDB::new(cas).await?));

    let context = AppContext { graph };

    // Create GraphQL schema
    let schema = Schema::build(
        QueryRoot,
        MutationRoot,
        EmptySubscription,
    )
    .data(context)
    .finish();

    println!("🔗 GraphQL playground available at: http://localhost:8000");
    println!("\n📋 Example GraphQL queries:");
    println!("\n# Create a node:");
    println!("mutation {");
    println!("  createNode(data: \"Hello GraphQL!\") {");
    println!("    id");
    println!("    data");
    println!("  }");
    println!("}");
    println!("\n# Query a node:");
    println!("query {");
    println!("  node(id: \"1\") {");
    println!("    id");
    println!("    data");
    println!("  }");
    println!("}");
    println!("\n# Traverse graph:");
    println!("query {");
    println!("  traverse(fromId: \"1\", hops: 2) {");
    println!("    totalNodes");
    println!("    totalEdges");
    println!("    nodes {");
    println!("      id");
    println!("      data");
    println!("    }");
    println!("  }");
    println!("}");

    // Note: This is a simplified example. In production, you would use
    // a proper GraphQL server like async-graphql-axum
    println!("\n⚠️  This example shows the schema structure.");
    println!("   For a full server implementation, see fcdb-api crate.");

    Ok(())
}
