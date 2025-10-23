//! HTTP server implementation for Own-CFA-Enishi

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::metrics::MetricsCollector;
use crate::health::HealthChecker;
use fcdb_graph::GraphDB;
use fcdb_rdf::{RdfExporter, SparqlRunner};
use fcdb_shacl::{validate_shapes, ValidationConfig};
use fcdb_cypher::execute_cypher;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub metrics: Arc<MetricsCollector>,
    pub health: Arc<HealthChecker>,
    pub graph_db: Arc<RwLock<GraphDB>>,
}

/// HTTP server for Own-CFA-Enishi
pub struct Server {
    state: AppState,
}

impl Server {
    pub fn new(
        config: Config,
        metrics: Arc<MetricsCollector>,
        health: Arc<HealthChecker>,
        graph_db: Arc<RwLock<GraphDB>>,
    ) -> Self {
        Self {
            state: AppState {
                config,
                metrics,
                health,
                graph_db,
            },
        }
    }

    /// Start the HTTP server
    pub async fn run(self, addr: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.create_router();

        println!("🚀 Server starting on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Create the Axum router with all routes
    fn create_router(self) -> Router {
        Router::new()
            .route("/", get(root))
            .route("/health", get(health_check))
            .route("/ready", get(readiness_check))
            .route("/metrics", get(metrics_endpoint))
            .route("/version", get(version_info))
            .route("/status", get(system_status))
            .route("/rdf/export", get(rdf_export))
            .route("/sparql", post(sparql_query))
            .route("/shacl/validate", post(shacl_validate))
            .route("/cypher", post(cypher_query))
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::new().allow_origin(Any))
            .with_state(self.state)
    }
}

/// Root endpoint
async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "service": "Own-CFA-Enishi",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Categorical Database with Ownership & Capability Security",
        "endpoints": {
            "health": "/health",
            "ready": "/ready",
            "metrics": "/metrics",
            "version": "/version",
            "status": "/status"
        }
    }))
}

/// Health check endpoint
async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let health = state.health.check().await;

    let status = if health.healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = json!({
        "status": if health.healthy { "healthy" } else { "unhealthy" },
        "timestamp": health.timestamp,
        "checks": {
            "system": health.system_health,
            "storage": health.storage_health,
            "memory": health.memory_health,
            "connections": health.connections_health
        },
        "uptime_seconds": health.uptime_seconds
    });

    if health.healthy {
        Ok(Json(response))
    } else {
        Err(status)
    }
}

/// Readiness check endpoint
async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement proper readiness checks
    // For now, assume ready if health check passes

    let health = state.health.check().await;
    if health.healthy {
        Ok(Json(json!({
            "status": "ready",
            "message": "System is ready to accept requests"
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Metrics endpoint (Prometheus format)
async fn metrics_endpoint(
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
    let metrics = state.metrics.collect().await;

    // Format as Prometheus metrics
    let mut output = String::new();

    output.push_str("# HELP enishi_query_count_total Total number of queries processed\n");
    output.push_str(&format!("# TYPE enishi_query_count_total counter\n"));
    output.push_str(&format!("enishi_query_count_total {}\n", metrics.query_count));

    output.push_str("\n# HELP enishi_query_duration_seconds Query duration in seconds\n");
    output.push_str(&format!("# TYPE enishi_query_duration_seconds histogram\n"));
    output.push_str(&format!("enishi_query_duration_seconds_sum {}\n", metrics.query_duration_sum));
    output.push_str(&format!("enishi_query_duration_seconds_count {}\n", metrics.query_count));

    output.push_str("\n# HELP enishi_memory_usage_bytes Current memory usage\n");
    output.push_str(&format!("# TYPE enishi_memory_usage_bytes gauge\n"));
    output.push_str(&format!("enishi_memory_usage_bytes {}\n", metrics.memory_usage));

    output.push_str("\n# HELP enishi_cache_hit_ratio Cache hit ratio (0.0-1.0)\n");
    output.push_str(&format!("# TYPE enishi_cache_hit_ratio gauge\n"));
    output.push_str(&format!("enishi_cache_hit_ratio {}\n", metrics.cache_hit_ratio));

    Ok(output)
}

/// Version information endpoint
async fn version_info() -> Json<serde_json::Value> {
    Json(json!({
        "service": "Own-CFA-Enishi",
        "version": env!("CARGO_PKG_VERSION"),
        "build_date": env!("VERGEN_BUILD_DATE"),
        "git_commit": env!("VERGEN_GIT_SHA"),
        "rust_version": env!("VERGEN_RUSTC_SEMVER"),
        "target": env!("VERGEN_CARGO_TARGET_TRIPLE")
    }))
}

/// System status endpoint
async fn system_status(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let health = state.health.check().await;
    let metrics = state.metrics.collect().await;

    Json(json!({
        "status": if health.healthy { "operational" } else { "degraded" },
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": health.uptime_seconds,
        "performance": {
            "queries_per_second": metrics.queries_per_second,
            "cache_hit_ratio": metrics.cache_hit_ratio,
            "memory_usage_mb": metrics.memory_usage / 1024 / 1024,
            "active_connections": metrics.active_connections
        },
        "configuration": {
            "port": state.config.server.port,
            "workers": state.config.server.workers,
            "storage_path": state.config.storage.path.display().to_string(),
            "adaptive_optimization": state.config.performance.adaptive_optimization
        },
        "phases": {
            "A": "completed", // P4 Core
            "B": "completed", // P10 Optimization
            "C": "completed", // P10+ Adaptation
            "D": "completed", // Own+CFA Final
            "PROD": "in_progress" // Production Deployment
        }
    }))
}

/// RDF export endpoint (N-Triples)
async fn rdf_export(
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
    let graph = state.graph_db.read().await;
    let exporter = RdfExporter::new(&*graph, "https://enishi.local/");
    exporter.export_ntriples().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// SPARQL query endpoint (returns JSON for SELECT/Boolean, N-Triples for CONSTRUCT)
async fn sparql_query(
    State(state): State<AppState>,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> Result<String, StatusCode> {
    let query = body.get("query").and_then(|v| v.as_str()).unwrap_or("");
    if query.is_empty() { return Err(StatusCode::BAD_REQUEST); }
    let graph = state.graph_db.read().await;
    let exporter = RdfExporter::new(&*graph, "https://enishi.local/");
    let runner = SparqlRunner::new(exporter);
    runner.execute(query).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// SHACL validation endpoint
async fn shacl_validate(
    State(state): State<AppState>,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let shapes = body.get("shapes").and_then(|v| v.as_str()).unwrap_or("");
    let max_violations = body.get("maxViolations").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let strict_mode = body.get("strictMode").and_then(|v| v.as_bool()).unwrap_or(false);

    let config = ValidationConfig {
        max_violations,
        strict_mode,
    };

    let graph = state.graph_db.read().await;
    let report = validate_shapes(&*graph, shapes, config).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to JSON response
    let response = serde_json::json!({
        "conforms": report.conforms,
        "results": report.results.into_iter().map(|r| serde_json::json!({
            "result": r.result,
            "violations": r.violations.into_iter().map(|v| serde_json::json!({
                "constraint": v.constraint,
                "message": v.message,
                "value": v.value,
                "expected": v.expected,
                "path": v.path
            })).collect::<Vec<_>>(),
            "focusNode": r.focus_node,
            "shapeId": r.shape_id
        })).collect::<Vec<_>>(),
        "shapes": report.shapes
    });

    Ok(Json(response))
}

/// Cypher query endpoint
async fn cypher_query(
    State(state): State<AppState>,
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let query = body.get("query").and_then(|v| v.as_str()).unwrap_or("");
    if query.is_empty() { return Err(StatusCode::BAD_REQUEST); }

    let graph = state.graph_db.read().await;
    let result = execute_cypher(query, &*graph).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to JSON response
    let response = serde_json::json!({
        "columns": result.columns,
        "rows": result.rows,
        "stats": {
            "nodesCreated": result.stats.nodes_created,
            "nodesDeleted": result.stats.nodes_deleted,
            "relationshipsCreated": result.stats.relationships_created,
            "relationshipsDeleted": result.stats.relationships_deleted,
            "labelsAdded": result.stats.labels_added,
            "labelsRemoved": result.stats.labels_removed,
            "propertiesSet": result.stats.properties_set,
            "executionTimeMs": result.stats.execution_time_ms
        }
    });

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::MetricsCollector;
    use crate::health::HealthChecker;
    use crate::config::Config;

    #[tokio::test]
    async fn test_root_endpoint() {
        // This would test the actual HTTP endpoints
        // For now, just verify the response structure
        let response = root().await;
        let json = response.0;

        assert_eq!(json["service"], "Own-CFA-Enishi");
        assert!(json["version"].is_string());
    }

    #[tokio::test]
    async fn test_version_endpoint() {
        let response = version_info().await;
        let json = response.0;

        assert_eq!(json["service"], "Own-CFA-Enishi");
        assert!(json["version"].is_string());
    }
}
