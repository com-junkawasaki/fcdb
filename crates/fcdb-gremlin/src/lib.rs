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
pub struct Traversal {
    pub steps: Vec<Step>,
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
                        traversers.push(Traverser {
                            current: rid,
                            path: vec![rid],
                            value: None,
                        });
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
                                new_traversers.push(Traverser {
                                    current: edge.target,
                                    path: new_path,
                                    value: traverser.value.clone(),
                                });
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
                        }
                    }
                    Step::Values(key) => {
                        if let Ok(Some(data)) = self.graph.get_node(traverser.current).await {
                            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&data) {
                                if let Some(value) = json.get(&key) {
                                    let mut new_traverser = traverser.clone();
                                    new_traverser.value = Some(value.clone());
                                    new_traversers.push(new_traverser);
                                }
                            }
                        }
                    }
                    Step::Path => {
                        let mut new_traverser = traverser.clone();
                        new_traverser.value = Some(serde_json::Value::Array(
                            traverser.path.iter().map(|rid| serde_json::json!(rid.0)).collect()
                        ));
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
