//! # Enishi API
//!
//! GraphQL and gRPC API interfaces for the Enishi database.
//!
//! Merkle DAG: enishi_api -> graphql_schema, grpc_services, http_handlers

use async_graphql::{Context, EmptySubscription, Object, Schema, SimpleObject, ID};
use fcdb_graph::{GraphDB, Rid, LabelId, Timestamp};
use fcdb_rdf::{RdfExporter, SparqlRunner};
use fcdb_shacl::{validate_shapes, ValidationConfig};
use fcdb_cypher::execute_cypher;
use fcdb_gremlin::{execute_traversal, Traversal, g};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// GraphQL node representation
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier
    pub id: ID,
    /// Node data as JSON string
    pub data: String,
    /// Creation timestamp
    pub created_at: String,
}

/// GraphQL edge representation
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub from: ID,
    /// Target node ID
    pub to: ID,
    /// Edge label
    pub label: String,
    /// Edge properties as JSON string
    pub properties: String,
}

/// GraphQL traversal result
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct TraversalResult {
    /// Node that was visited
    pub node: Node,
    /// Depth at which this node was found
    pub depth: i32,
}

/// GraphQL search result
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matching node
    pub node: Node,
    /// Search score
    pub score: f32,
}

/// Input for creating nodes
#[derive(async_graphql::InputObject)]
pub struct CreateNodeInput {
    /// Node data as JSON string
    pub data: String,
}

/// Input for updating nodes
#[derive(async_graphql::InputObject)]
pub struct UpdateNodeInput {
    /// Node ID
    pub id: ID,
    /// New node data as JSON string
    pub data: String,
}

/// Input for creating edges
#[derive(async_graphql::InputObject)]
pub struct CreateEdgeInput {
    /// Source node ID
    pub from: ID,
    /// Target node ID
    pub to: ID,
    /// Edge label
    pub label: String,
    /// Edge properties as JSON string
    pub properties: String,
}

/// Input for traversal queries
#[derive(async_graphql::InputObject)]
pub struct TraverseInput {
    /// Starting node ID
    pub from: ID,
    /// Edge labels to follow (empty means all)
    pub labels: Option<Vec<String>>,
    /// Maximum traversal depth
    pub max_depth: i32,
    /// Historical timestamp (optional)
    pub as_of: Option<String>,
}

/// SHACL validation input
#[derive(async_graphql::InputObject)]
pub struct ShaclValidateInput {
    /// SHACL shapes in RDF format (Turtle/JSON-LD)
    pub shapes: String,
    /// Validation configuration (optional)
    pub config: Option<ShaclValidationConfig>,
}

/// SHACL validation configuration
#[derive(async_graphql::InputObject)]
pub struct ShaclValidationConfig {
    /// Maximum violations to report
    pub max_violations: Option<usize>,
    /// Fail fast on first violation
    pub strict_mode: Option<bool>,
}

/// GraphQL representation of SHACL validation report
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct GraphQLValidationReport {
    /// Whether the data conforms to the shapes
    pub conforms: bool,
    /// Validation results
    pub results: Vec<GraphQLValidationResult>,
    /// Shape IDs that were validated
    pub shapes: Vec<String>,
}

/// GraphQL representation of validation result
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct GraphQLValidationResult {
    /// Whether this result is valid
    pub result: bool,
    /// Violations found
    pub violations: Vec<GraphQLViolation>,
    /// The node that was validated
    pub focus_node: String,
    /// The shape that was used
    pub shape_id: String,
}

/// GraphQL representation of violation
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct GraphQLViolation {
    /// Description of the constraint
    pub constraint: String,
    /// Human-readable error message
    pub message: String,
    /// The actual value that violated
    pub value: Option<String>,
    /// What was expected
    pub expected: Option<String>,
    /// Property path if applicable
    pub path: Option<String>,
}

/// GraphQL representation of Cypher query result
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct GraphQLCypherResult {
    /// Column names
    pub columns: Vec<String>,
    /// Result rows
    pub rows: Vec<serde_json::Value>,
    /// Query execution statistics
    pub stats: GraphQLQueryStats,
}

/// GraphQL representation of query statistics
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct GraphQLQueryStats {
    /// Number of nodes created
    pub nodes_created: i32,
    /// Number of nodes deleted
    pub nodes_deleted: i32,
    /// Number of relationships created
    pub relationships_created: i32,
    /// Number of relationships deleted
    pub relationships_deleted: i32,
    /// Number of labels added
    pub labels_added: i32,
    /// Number of labels removed
    pub labels_removed: i32,
    /// Number of properties set
    pub properties_set: i32,
    /// Execution time in milliseconds
    pub execution_time_ms: i64,
}

/// GraphQL representation of Gremlin traversal result
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct GraphQLGremlinResult {
    /// Traversers that completed the traversal
    pub traversers: Vec<GraphQLTraverser>,
}

/// GraphQL representation of a traverser
#[derive(SimpleObject, Serialize, Deserialize)]
pub struct GraphQLTraverser {
    /// Current node ID
    pub current: String,
    /// Path of nodes traversed
    pub path: Vec<String>,
    /// Current value (if any)
    pub value: Option<serde_json::Value>,
}

/// Input for Gremlin traversal steps
#[derive(async_graphql::InputObject)]
pub struct GremlinTraversalInput {
    /// Starting point - "V" for all vertices, "V(id)" for specific vertex
    pub start: String,
    /// Sequence of traversal steps as strings
    pub steps: Vec<String>,
}

/// GraphQL query root
pub struct Query;

#[Object]
impl Query {
    /// Get a node by ID
    async fn node(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Node>> {
        let graph = ctx.data::<Arc<RwLock<GraphDB>>>()?;
        let graph = graph.read().await;

        let rid = Rid(id.parse().map_err(|_| "Invalid node ID")?);

        match graph.get_node(rid).await {
            Ok(Some(data)) => {
                let data_str = String::from_utf8(data)
                    .map_err(|_| "Invalid UTF-8 data")?;
                Ok(Some(Node {
                    id,
                    data: data_str,
                    created_at: "2024-01-01T00:00:00Z".to_string(), // Simplified
                }))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(async_graphql::Error::new(format!("Database error: {}", e))),
        }
    }

    /// Get a node at a specific historical timestamp
    async fn node_at(&self, ctx: &Context<'_>, id: ID, as_of: String) -> async_graphql::Result<Option<Node>> {
        let graph = ctx.data::<Arc<RwLock<GraphDB>>>()?;
        let graph = graph.read().await;

        let rid = Rid(id.parse().map_err(|_| "Invalid node ID")?);
        let timestamp = Timestamp(as_of.parse().map_err(|_| "Invalid timestamp")?);

        match graph.get_node_at(rid, timestamp).await {
            Ok(Some(data)) => {
                let data_str = String::from_utf8(data)
                    .map_err(|_| "Invalid UTF-8 data")?;
                Ok(Some(Node {
                    id,
                    data: data_str,
                    created_at: as_of,
                }))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(async_graphql::Error::new(format!("Database error: {}", e))),
        }
    }

    /// Traverse the graph from a starting node
    async fn traverse(&self, ctx: &Context<'_>, input: TraverseInput) -> async_graphql::Result<Vec<TraversalResult>> {
        let graph = ctx.data::<Arc<RwLock<GraphDB>>>()?;
        let graph = graph.read().await;

        let from_rid = Rid(input.from.parse().map_err(|_| "Invalid node ID")?);
        let labels: Option<Vec<LabelId>> = input.labels.map(|ls|
            ls.into_iter().map(|l| LabelId(l.parse().unwrap_or(0))).collect()
        );
        let max_depth = input.max_depth as usize;
        let as_of = input.as_of.map(|ts| Timestamp(ts.parse().unwrap_or(0)));

        let traversal = graph.traverse(from_rid, labels.as_deref(), max_depth, as_of).await
            .map_err(|e| async_graphql::Error::new(format!("Traversal error: {}", e)))?;

        let mut results = Vec::new();
        for (rid, depth) in traversal {
            // Get node data for each result
            if let Ok(Some(data)) = graph.get_node(rid).await {
                if let Ok(data_str) = String::from_utf8(data) {
                    results.push(TraversalResult {
                        node: Node {
                            id: ID::from(rid.0.to_string()),
                            data: data_str,
                            created_at: "2024-01-01T00:00:00Z".to_string(),
                        },
                        depth: depth as i32,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Search nodes by text content
    async fn search(&self, ctx: &Context<'_>, query: String) -> async_graphql::Result<Vec<SearchResult>> {
        let graph = ctx.data::<Arc<RwLock<GraphDB>>>()?;
        let graph = graph.read().await;

        let search_results = graph.search(&query).await
            .map_err(|e| async_graphql::Error::new(format!("Search error: {}", e)))?;

        let mut results = Vec::new();
        for (rid, score) in search_results {
            if let Ok(Some(data)) = graph.get_node(rid).await {
                if let Ok(data_str) = String::from_utf8(data) {
                    results.push(SearchResult {
                        node: Node {
                            id: ID::from(rid.0.to_string()),
                            data: data_str,
                            created_at: "2024-01-01T00:00:00Z".to_string(),
                        },
                        score,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Execute a SPARQL query over the RDF projection
    async fn sparql(&self, ctx: &Context<'_>, query: String) -> async_graphql::Result<String> {
        let graph = ctx.data::<Arc<RwLock<GraphDB>>>()?;
        let graph = graph.read().await;
        let exporter = RdfExporter::new(&graph, "https://enishi.local/");
        let runner = SparqlRunner::new(exporter);
        runner.execute(&query).await.map_err(|e| async_graphql::Error::new(e))
    }

    /// Validate data against SHACL shapes
    async fn validate_shacl(&self, ctx: &Context<'_>, input: ShaclValidateInput) -> async_graphql::Result<GraphQLValidationReport> {
        let graph = ctx.data::<Arc<RwLock<GraphDB>>>()?;
        let graph = graph.read().await;

        let config = ValidationConfig {
            max_violations: input.config.as_ref().and_then(|c| c.max_violations).unwrap_or(100),
            strict_mode: input.config.as_ref().and_then(|c| c.strict_mode).unwrap_or(false),
        };

        let report = validate_shapes(&graph, &input.shapes, config).await
            .map_err(|e| async_graphql::Error::new(format!("SHACL validation error: {:?}", e)))?;

        // Convert internal report to GraphQL representation
        let graphql_report = GraphQLValidationReport {
            conforms: report.conforms,
            results: report.results.into_iter().map(|r| GraphQLValidationResult {
                result: r.result,
                violations: r.violations.into_iter().map(|v| GraphQLViolation {
                    constraint: v.constraint,
                    message: v.message,
                    value: v.value,
                    expected: v.expected,
                    path: v.path,
                }).collect(),
                focus_node: r.focus_node,
                shape_id: r.shape_id,
            }).collect(),
            shapes: report.shapes,
        };

        Ok(graphql_report)
    }

    /// Execute a Cypher query
    async fn cypher(&self, ctx: &Context<'_>, query: String) -> async_graphql::Result<GraphQLCypherResult> {
        let graph = ctx.data::<Arc<RwLock<GraphDB>>>()?;
        let graph = graph.read().await;

        let result = execute_cypher(&query, &graph).await
            .map_err(|e| async_graphql::Error::new(format!("Cypher execution error: {:?}", e)))?;

        // Convert internal result to GraphQL representation
        let graphql_result = GraphQLCypherResult {
            columns: result.columns,
            rows: result.rows,
            stats: GraphQLQueryStats {
                nodes_created: result.stats.nodes_created as i32,
                nodes_deleted: result.stats.nodes_deleted as i32,
                relationships_created: result.stats.relationships_created as i32,
                relationships_deleted: result.stats.relationships_deleted as i32,
                labels_added: result.stats.labels_added as i32,
                labels_removed: result.stats.labels_removed as i32,
                properties_set: result.stats.properties_set as i32,
                execution_time_ms: result.stats.execution_time_ms as i64,
            },
        };

        Ok(graphql_result)
    }

    /// Execute a Gremlin traversal
    async fn gremlin(&self, ctx: &Context<'_>, input: GremlinTraversalInput) -> async_graphql::Result<GraphQLGremlinResult> {
        let graph = ctx.data::<Arc<RwLock<GraphDB>>>()?;
        let graph = graph.read().await;

        // Build traversal from input
        let mut traversal_builder = g();

        // Parse start step
        if input.start == "V" {
            traversal_builder = traversal_builder.V();
        } else if input.start.starts_with("V(") && input.start.ends_with(")") {
            let id_str = &input.start[2..input.start.len() - 1];
            if let Ok(id) = id_str.parse::<u64>() {
                traversal_builder = traversal_builder.V_id(id);
            } else {
                return Err(async_graphql::Error::new("Invalid vertex ID in start"));
            }
        } else {
            return Err(async_graphql::Error::new("Invalid start step"));
        }

        // Parse and apply steps
        for step in &input.steps {
            traversal_builder = parse_and_apply_step(traversal_builder, step)?;
        }

        let traversal = traversal_builder.build();
        let result = execute_traversal(&graph, traversal).await
            .map_err(|e| async_graphql::Error::new(format!("Gremlin execution error: {:?}", e)))?;

        // Convert internal result to GraphQL representation
        let graphql_result = GraphQLGremlinResult {
            traversers: result.traversers.into_iter().map(|t| GraphQLTraverser {
                current: t.current.0.to_string(),
                path: t.path.iter().map(|rid| rid.0.to_string()).collect(),
                value: t.get_side_effect("value").cloned(),
            }).collect(),
        };

        Ok(graphql_result)
    }
}

fn parse_and_apply_step(builder: crate::fcdb_gremlin::TraversalBuilder, step: &str) -> Result<crate::fcdb_gremlin::TraversalBuilder, async_graphql::Error> {
    if step.starts_with("out(") && step.ends_with(")") {
        let label = &step[4..step.len() - 1];
        let label_opt = if label.is_empty() { None } else { Some(label.to_string()) };
        Ok(builder.out(label_opt))
    } else if step.starts_with("in(") && step.ends_with(")") {
        let label = &step[3..step.len() - 1];
        let label_opt = if label.is_empty() { None } else { Some(label.to_string()) };
        Ok(builder.in_(label_opt))
    } else if step.starts_with("has(") && step.ends_with(")") {
        let content = &step[4..step.len() - 1];
        let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let key = parts[0].trim_matches('"');
            let value_str = parts[1].trim_matches('"');
            // Simple parsing - in production, would need proper JSON parsing
            let value = if value_str.starts_with('"') && value_str.ends_with('"') {
                serde_json::Value::String(value_str[1..value_str.len()-1].to_string())
            } else if let Ok(num) = value_str.parse::<i64>() {
                serde_json::Value::Number(num.into())
            } else {
                serde_json::Value::String(value_str.to_string())
            };
            Ok(builder.has(key.to_string(), value))
        } else {
            Err(async_graphql::Error::new("Invalid has() syntax"))
        }
    } else if step == "values(name)" {
        Ok(builder.values("name".to_string()))
    } else if step == "path()" {
        Ok(builder.path())
    } else {
        Err(async_graphql::Error::new(format!("Unsupported step: {}", step)))
    }
}

/// GraphQL mutation root
pub struct Mutation;

#[Object]
impl Mutation {
    /// Create a new node
    async fn create_node(&self, ctx: &Context<'_>, input: CreateNodeInput) -> async_graphql::Result<Node> {
        let graph = ctx.data::<Arc<RwLock<GraphDB>>>()?;
        let mut graph = graph.write().await;

        let data_bytes = input.data.as_bytes();
        let rid = graph.create_node(data_bytes).await
            .map_err(|e| async_graphql::Error::new(format!("Create node error: {}", e)))?;

        Ok(Node {
            id: ID::from(rid.0.to_string()),
            data: input.data,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        })
    }

    /// Update an existing node
    async fn update_node(&self, ctx: &Context<'_>, input: UpdateNodeInput) -> async_graphql::Result<Node> {
        let graph = ctx.data::<Arc<RwLock<GraphDB>>>()?;
        let mut graph = graph.write().await;

        let rid = Rid(input.id.parse().map_err(|_| "Invalid node ID")?);
        let data_bytes = input.data.as_bytes();

        graph.update_node(rid, data_bytes).await
            .map_err(|e| async_graphql::Error::new(format!("Update node error: {}", e)))?;

        Ok(Node {
            id: input.id,
            data: input.data,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        })
    }

    /// Create an edge between nodes
    async fn create_edge(&self, ctx: &Context<'_>, input: CreateEdgeInput) -> async_graphql::Result<GraphEdge> {
        let graph = ctx.data::<Arc<RwLock<GraphDB>>>()?;
        let mut graph = graph.write().await;

        let from_rid = Rid(input.from.parse().map_err(|_| "Invalid from ID")?);
        let to_rid = Rid(input.to.parse().map_err(|_| "Invalid to ID")?);
        let label_id = LabelId(input.label.parse().map_err(|_| "Invalid label")?);
        let prop_bytes = input.properties.as_bytes();

        graph.create_edge(from_rid, to_rid, label_id, prop_bytes).await
            .map_err(|e| async_graphql::Error::new(format!("Create edge error: {}", e)))?;

        Ok(GraphEdge {
            from: input.from,
            to: input.to,
            label: input.label,
            properties: input.properties,
        })
    }
}

/// GraphQL schema type
pub type EnishiSchema = Schema<Query, Mutation, EmptySubscription>;

/// Create the GraphQL schema
pub fn create_schema(graph: Arc<RwLock<GraphDB>>) -> EnishiSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(graph)
        .finish()
}

/// GraphQL SDL (Schema Definition Language)
pub const GRAPHQL_SCHEMA: &str = r#"
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

    type ValidationReport {
        conforms: Boolean!
        results: [ValidationResult!]!
        shapes: [String!]!
    }

    type ValidationResult {
        result: Boolean!
        violations: [Violation!]!
        focusNode: String!
        shapeId: String!
    }

    type Violation {
        constraint: String!
        message: String!
        value: String
        expected: String
        path: String
    }

    type CypherResult {
        columns: [String!]!
        rows: [Json!]!
        stats: QueryStats!
    }

    type QueryStats {
        nodesCreated: Int!
        nodesDeleted: Int!
        relationshipsCreated: Int!
        relationshipsDeleted: Int!
        labelsAdded: Int!
        labelsRemoved: Int!
        propertiesSet: Int!
        executionTimeMs: Int!
    }

    scalar Json

    type GremlinResult {
        traversers: [Traverser!]!
    }

    type Traverser {
        current: String!
        path: [String!]!
        value: Json
    }

    input GremlinTraversalInput {
        start: String!
        steps: [String!]!
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

    input ShaclValidateInput {
        shapes: String!
        config: ShaclValidationConfig
    }

    input ShaclValidationConfig {
        maxViolations: Int
        strictMode: Boolean
    }

    type Query {
        node(id: ID!): Node
        nodeAt(id: ID!, asOf: String!): Node
        traverse(input: TraverseInput!): [TraversalResult!]!
        search(query: String!): [SearchResult!]!
        sparql(query: String!): String!
        validateShacl(input: ShaclValidateInput!): ValidationReport!
        cypher(query: String!): CypherResult!
        gremlin(input: GremlinTraversalInput!): GremlinResult!
    }

    type Mutation {
        createNode(input: CreateNodeInput!): Node!
        updateNode(input: UpdateNodeInput!): Node!
        createEdge(input: CreateEdgeInput!): GraphEdge!
    }
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_graphql_schema_creation() {
        let temp_dir = tempdir().unwrap();
        let cas = fcdb_cas::PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;
        let graph = Arc::new(RwLock::new(graph));

        let schema = create_schema(graph);
        let result = schema.execute("query { __typename }").await;
        assert!(result.is_ok());
    }
}
