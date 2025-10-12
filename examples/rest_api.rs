//! # FCDB REST API Example
//!
//! This example demonstrates how to create a simple REST API using FCDB
//! with the fcdb-api crate. It shows:
//! - Setting up an Axum-based REST server
//! - Basic CRUD operations via HTTP endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use fcdb_api::{GraphQLApi, RestApi};
use fcdb_cas::PackCAS;
use fcdb_core::Rid;
use fcdb_graph::GraphDB;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    graph: Arc<RwLock<GraphDB>>,
}

#[derive(Serialize, Deserialize)]
struct CreateNodeRequest {
    data: String,
}

#[derive(Serialize, Deserialize)]
struct CreateEdgeRequest {
    from: u64,
    to: u64,
    label: String,
    properties: Option<String>,
}

#[derive(Serialize)]
struct NodeResponse {
    id: Rid,
    data: String,
}

#[derive(Serialize)]
struct EdgeResponse {
    from: Rid,
    to: Rid,
    label: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 FCDB REST API Example");
    println!("========================");

    // Initialize FCDB components
    let cas = PackCAS::new("./api_example_data").await?;
    let graph = Arc::new(RwLock::new(GraphDB::new(cas).await?));

    let state = AppState { graph };

    // Create Axum router with REST endpoints
    let app = Router::new()
        .route("/nodes", post(create_node))
        .route("/nodes/:id", get(get_node))
        .route("/nodes/:id", delete(delete_node))
        .route("/edges", post(create_edge))
        .route("/traverse/:id/:hops", get(traverse))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(state);

    println!("🌐 Starting REST API server on http://localhost:3000");
    println!("📋 Available endpoints:");
    println!("  POST   /nodes              - Create a new node");
    println!("  GET    /nodes/:id          - Get node by ID");
    println!("  DELETE /nodes/:id          - Delete node by ID");
    println!("  POST   /edges              - Create an edge between nodes");
    println!("  GET    /traverse/:id/:hops - Traverse graph from node");
    println!("  GET    /health             - Health check");
    println!("\n🧪 Try these curl commands:");
    println!("  curl -X POST http://localhost:3000/nodes \\");
    println!("       -H 'Content-Type: application/json' \\");
    println!("       -d '{\"data\":\"Hello FCDB!\"}'");
    println!("\n  curl http://localhost:3000/health");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// REST API handlers

async fn create_node(
    State(state): State<AppState>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<Json<NodeResponse>, StatusCode> {
    let mut graph = state.graph.write().await;
    match graph.create_node(req.data.into_bytes()).await {
        Ok(id) => {
            let response = NodeResponse {
                id,
                data: req.data,
            };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_node(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<NodeResponse>, StatusCode> {
    let graph = state.graph.read().await;
    let rid = Rid(id);

    match graph.get_node(rid).await {
        Ok(Some(data)) => {
            let content = String::from_utf8_lossy(&data).to_string();
            let response = NodeResponse { id: rid, data: content };
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_node(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, StatusCode> {
    let mut graph = state.graph.write().await;
    let rid = Rid(id);

    match graph.delete_node(rid).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_edge(
    State(state): State<AppState>,
    Json(req): Json<CreateEdgeRequest>,
) -> Result<Json<EdgeResponse>, StatusCode> {
    let graph = state.graph.read().await;

    let from_rid = Rid(req.from);
    let to_rid = Rid(req.to);
    let label_id = fcdb_graph::LabelId(1); // Simple label mapping
    let properties = req.properties.unwrap_or_default().into_bytes();

    match graph.create_edge(from_rid, to_rid, label_id, &properties).await {
        Ok(_) => {
            let response = EdgeResponse {
                from: from_rid,
                to: to_rid,
                label: req.label,
            };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn traverse(
    State(state): State<AppState>,
    Path((id, hops)): Path<(u64, usize)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let graph = state.graph.read().await;
    let rid = Rid(id);

    match graph.traverse(rid, None, hops, None).await {
        Ok(result) => {
            let node_count = result.nodes.len();
            let edge_count = result.edges.len();

            let response = serde_json::json!({
                "start_node": id,
                "hops": hops,
                "nodes_found": node_count,
                "edges_found": edge_count,
                "nodes": result.nodes.iter().map(|&rid| rid.0).collect::<Vec<_>>()
            });

            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "FCDB REST API",
        "version": "0.1.0"
    }))
}
