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
