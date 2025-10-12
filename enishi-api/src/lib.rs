//! # Enishi API
//!
//! GraphQL and gRPC API interfaces for the Enishi database.
//!
//! Merkle DAG: enishi_api -> graphql_schema, grpc_services, http_handlers

use async_graphql::{Context, EmptySubscription, Object, Schema, SimpleObject, ID};
use enishi_graph::{GraphDB, Rid, LabelId, Timestamp};
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

    type Query {
        node(id: ID!): Node
        nodeAt(id: ID!, asOf: String!): Node
        traverse(input: TraverseInput!): [TraversalResult!]!
        search(query: String!): [SearchResult!]!
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
        let cas = enishi_cas::PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;
        let graph = Arc::new(RwLock::new(graph));

        let schema = create_schema(graph);
        assert!(schema.query(&"query { __typename }").await.is_ok());
    }
}
