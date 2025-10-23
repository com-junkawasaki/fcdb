//! fcdb-gremlin: Gremlin-like DSL for FCDB graph traversal
//! Merkle DAG: fcdb_gremlin -> traversal, steps, executor

pub mod traversal;
pub mod steps;

pub use traversal::{Traversal, Traverser};
pub use steps::Step;

use fcdb_graph::{GraphDB, Rid};
use std::sync::Arc;

/// Execute a Gremlin traversal against the graph database
/// Merkle DAG: fcdb_gremlin -> execute_traversal(g, traversal) -> result
pub async fn execute_traversal(
    graph: &GraphDB,
    traversal: Traversal,
) -> Result<TraversalResult, GremlinError> {
    let mut executor = TraversalExecutor::new(graph);
    executor.execute(traversal).await
}

/// Create a new traversal starting from vertices
/// Merkle DAG: fcdb_gremlin -> g.V() -> traversal_builder
pub fn g() -> TraversalBuilder {
    TraversalBuilder::new()
}

#[derive(Debug, Clone)]
pub struct TraversalBuilder {
    steps: Vec<Step>,
}

impl TraversalBuilder {
    fn new() -> Self {
        Self { steps: vec![] }
    }

    /// Start traversal from all vertices
    pub fn V(self) -> Self {
        self.add_step(Step::V(None))
    }

    /// Start traversal from specific vertex
    pub fn V_id(self, id: u64) -> Self {
        self.add_step(Step::V(Some(Rid(id))))
    }

    /// Traverse outgoing edges with optional label
    pub fn out(self, label: Option<String>) -> Self {
        self.add_step(Step::Out(label))
    }

    /// Traverse incoming edges with optional label
    pub fn in_(self, label: Option<String>) -> Self {
        self.add_step(Step::In(label))
    }

    /// Filter by property value
    pub fn has(self, key: String, value: serde_json::Value) -> Self {
        self.add_step(Step::Has(key, value))
    }

    /// Get values for property key
    pub fn values(self, key: String) -> Self {
        self.add_step(Step::Values(key))
    }

    /// Get the path of the traversal
    pub fn path(self) -> Self {
        self.add_step(Step::Path)
    }

    /// Build the traversal
    pub fn build(self) -> Traversal {
        Traversal { steps: self.steps }
    }

    fn add_step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }
}

#[derive(Debug, Clone)]
pub struct TraversalResult {
    pub traversers: Vec<traversal::Traverser>,
}

struct TraversalExecutor<'a> {
    graph: &'a GraphDB,
}

impl<'a> TraversalExecutor<'a> {
    fn new(graph: &'a GraphDB) -> Self {
        Self { graph }
    }

    async fn execute(&self, traversal: Traversal) -> Result<TraversalResult, GremlinError> {
        let mut traversers = Vec::new();

        // Start with initial step
        if let Some(first_step) = traversal.steps.first() {
            match first_step {
                Step::V(start_id) => {
                    let start_ids = if let Some(id) = start_id {
                        vec![*id]
                    } else {
                        self.graph.list_rids().await
                    };

                    for rid in start_ids {
                        traversers.push(Traverser::new(rid));
                    }
                }
                _ => return Err(GremlinError::InvalidStart("Traversal must start with V()".to_string())),
            }
        }

        // Execute remaining steps
        for step in traversal.steps.iter().skip(1) {
            let mut new_traversers = Vec::new();

            for traverser in &traversers {
                match step {
                    Step::Out(label) => {
                        let edges = self.graph.get_edges_from(traverser.current).await;
                        for edge in edges {
                            if label.is_none() || label.as_ref() == Some(&format!("{}", edge.label.0)) {
                                let mut new_path = traverser.path.clone();
                                new_path.push(edge.target);
                                let mut new_traverser = Traverser::new_with_path(edge.target, new_path);
                                if let Some(value) = traverser.get_side_effect("value") {
                                    new_traverser.attach_side_effect("value".to_string(), value.clone());
                                }
                                new_traversers.push(new_traverser);
                            }
                        }
                    }
                    Step::In(label) => {
                        // For now, simplified - would need reverse index for full implementation
                        // This is a placeholder
                        new_traversers.push(traverser.clone());
                    }
                    Step::Has(key, expected_value) => {
                        if let Ok(Some(data)) = self.graph.get_node(traverser.current).await {
                            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&data) {
                                if let Some(actual_value) = json.get(key) {
                                    if actual_value == expected_value {
                                        new_traversers.push(traverser.clone());
                                    }
                                }
                            }
                        } else {
                            new_traversers.push(traverser.clone());
                        }
                    }
                    Step::Values(key) => {
                        if let Ok(Some(data)) = self.graph.get_node(traverser.current).await {
                            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&data) {
                                if let Some(value) = json.get(&key) {
                                    let mut new_traverser = traverser.clone();
                                    new_traverser.attach_side_effect("value".to_string(), value.clone());
                                    new_traversers.push(new_traverser);
                                }
                            }
                        }
                    }
                    Step::Path => {
                        let mut new_traverser = traverser.clone();
                        let path_array = serde_json::Value::Array(
                            traverser.path.iter().map(|rid| serde_json::json!(rid.0)).collect()
                        );
                        new_traverser.attach_side_effect("value".to_string(), path_array);
                        new_traversers.push(new_traverser);
                    }
                    _ => new_traversers.push(traverser.clone()),
                }
            }

            traversers = new_traversers;
        }

        Ok(TraversalResult { traversers })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GremlinError {
    #[error("Invalid traversal start: {0}")]
    InvalidStart(String),
    #[error("Graph error: {0}")]
    Graph(String),
    #[error("Execution error: {0}")]
    Execution(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use fcdb_graph::GraphDB;
    use fcdb_cas::PackCAS;

    #[tokio::test]
    async fn test_traversal_builder_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        // Create test data
        let node1 = graph.create_node(br#"{"name": "Alice"}"#).await.unwrap();
        let node2 = graph.create_node(br#"{"name": "Bob"}"#).await.unwrap();
        graph.create_edge(node1, node2, 1u32.into(), b"knows").await.unwrap();

        // Build traversal: g.V().out().values("name")
        let traversal = g()
            .V()
            .out(None)
            .values("name".to_string())
            .build();

        let result = execute_traversal(&graph, traversal).await.unwrap();

        // Should find at least one result
        assert!(!result.traversers.is_empty());

        // Check that each traverser has a value side effect
        for traverser in &result.traversers {
            assert!(traverser.get_side_effect("value").is_some());
        }
    }

    #[tokio::test]
    async fn test_traversal_builder_with_filter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        // Create test data
        graph.create_node(br#"{"type": "Person", "name": "Alice"}"#).await.unwrap();
        graph.create_node(br#"{"type": "Company", "name": "ACME"}"#).await.unwrap();

        // Build traversal: g.V().has("type", "Person").values("name")
        let traversal = g()
            .V()
            .has("type".to_string(), serde_json::json!("Person"))
            .values("name".to_string())
            .build();

        let result = execute_traversal(&graph, traversal).await.unwrap();

        // Should find exactly one result (Alice)
        assert_eq!(result.traversers.len(), 1);

        let traverser = &result.traversers[0];
        let value = traverser.get_side_effect("value").unwrap();
        assert_eq!(value, &serde_json::json!("Alice"));
    }

    #[tokio::test]
    async fn test_traversal_builder_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        // Create test data with path
        let node1 = graph.create_node(br#"{"name": "Start"}"#).await.unwrap();
        let node2 = graph.create_node(br#"{"name": "Middle"}"#).await.unwrap();
        let node3 = graph.create_node(br#"{"name": "End"}"#).await.unwrap();

        graph.create_edge(node1, node2, 1u32.into(), b"connects").await.unwrap();
        graph.create_edge(node2, node3, 1u32.into(), b"connects").await.unwrap();

        // Build traversal: g.V().out().out().path()
        let traversal = g()
            .V_id(node1.as_u64())
            .out(None)
            .out(None)
            .path()
            .build();

        let result = execute_traversal(&graph, traversal).await.unwrap();

        // Should find the path from Start -> Middle -> End
        assert!(!result.traversers.is_empty());

        let traverser = &result.traversers[0];
        let path_value = traverser.get_side_effect("value").unwrap();
        if let serde_json::Value::Array(path) = path_value {
            assert_eq!(path.len(), 3); // Start, Middle, End
            assert_eq!(path[0], serde_json::json!(node1.as_u64()));
            assert_eq!(path[1], serde_json::json!(node2.as_u64()));
            assert_eq!(path[2], serde_json::json!(node3.as_u64()));
        } else {
            panic!("Path should be an array");
        }
    }

    #[test]
    fn test_traverser_operations() {
        let rid = Rid(42);
        let mut traverser = Traverser::new(rid);

        assert_eq!(traverser.current, rid);
        assert_eq!(traverser.path, vec![rid]);

        // Test side effects
        traverser.attach_side_effect("test".to_string(), serde_json::json!("value"));
        assert_eq!(traverser.get_side_effect("test"), Some(&serde_json::json!("value")));
        assert_eq!(traverser.get_side_effect("nonexistent"), None);
    }

    #[test]
    fn test_gremlin_error_display() {
        let error = GremlinError::InvalidStart("bad start".to_string());
        assert!(error.to_string().contains("bad start"));
    }
}
