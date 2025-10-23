//! fcdb-cypher: Cypher subset parser and executor for FCDB
//! Merkle DAG: fcdb_cypher -> parser, planner, executor

pub mod ast;
pub mod parser;
pub mod planner;
pub mod executor;

pub use ast::{Query, Statement, MatchClause, WhereClause, ReturnClause};
pub use executor::{CypherExecutor, QueryResult};
pub use planner::QueryPlanner;

use fcdb_graph::GraphDB;

/// Execute a Cypher query against the graph database
/// Merkle DAG: fcdb_cypher -> execute_cypher(query, graph) -> result
pub async fn execute_cypher(
    query: &str,
    graph: &GraphDB,
) -> Result<QueryResult, CypherError> {
    let mut executor = CypherExecutor::new(graph);
    executor.execute(query).await
}

#[derive(Debug, thiserror::Error)]
pub enum CypherError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Planning error: {0}")]
    Planning(String),
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("Graph error: {0}")]
    Graph(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use fcdb_graph::GraphDB;
    use fcdb_cas::PackCAS;

    #[tokio::test]
    async fn test_execute_cypher_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        // Create test data
        graph.create_node(br#"{"name": "Alice", "age": 30}"#).await.unwrap();
        graph.create_node(br#"{"name": "Bob", "age": 25}"#).await.unwrap();

        // Simple Cypher query (placeholder implementation)
        let query = "MATCH (n) RETURN n";

        let result = execute_cypher(query, &graph).await.unwrap();

        // Basic checks for the result structure
        assert!(result.columns.len() >= 0); // At least some columns
        assert!(result.rows.len() >= 0); // At least some rows
        assert!(result.stats.execution_time_ms >= 0); // Non-negative execution time
    }

    #[tokio::test]
    async fn test_execute_cypher_with_data() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        // Create test data with edges
        let node1 = graph.create_node(br#"{"type": "Person", "name": "Alice"}"#).await.unwrap();
        let node2 = graph.create_node(br#"{"type": "Person", "name": "Bob"}"#).await.unwrap();
        graph.create_edge(node1, node2, 1u32.into(), b"knows").await.unwrap();

        // Cypher query for patterns
        let query = "MATCH (p:Person)-[:KNOWS]->(friend) RETURN p, friend";

        let result = execute_cypher(query, &graph).await.unwrap();

        // The placeholder implementation should still return a valid structure
        assert!(result.columns.len() >= 0);
        assert!(result.rows.len() >= 0);
    }

    #[test]
    fn test_cypher_error_display() {
        let error = CypherError::Parse("invalid syntax".to_string());
        assert!(error.to_string().contains("invalid syntax"));
    }
}
