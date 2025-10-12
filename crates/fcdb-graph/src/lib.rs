//! # Enishi Graph
//!
//! Graph data structures and operations for the Enishi database.
//!
//! Merkle DAG: enishi_graph -> rid_to_cid, adjacency, postings, temporal

use fcdb_core::{Cid, varint, Monoid};
use fcdb_cas::{PackCAS, PackBand};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Resource ID (RID) - unique identifier for graph nodes
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Rid(pub u64);

impl Rid {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Debug for Rid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rid({})", self.0)
    }
}

impl std::fmt::Display for Rid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Edge label/type identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LabelId(pub u32);

impl LabelId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Temporal timestamp for versioning
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn now() -> Self {
        Self(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
        )
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Graph edge representation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub from: Rid,
    pub to: Rid,
    pub label: LabelId,
    pub properties: Cid, // CID of property data
    pub created_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
}

/// Adjacency list entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdjEntry {
    pub target: Rid,
    pub label: LabelId,
    pub properties: Cid,
    pub timestamp: Timestamp,
}

/// Posting list for full-text search and analytics
#[derive(Clone, Debug)]
pub struct Posting {
    pub term: String,
    pub rid: Rid,
    pub positions: Vec<u32>, // Term positions in the document
    pub timestamp: Timestamp,
}


/// RID to CID mapping with temporal support
#[derive(Clone, Debug)]
pub struct RidMapping {
    pub rid: Rid,
    pub cid: Cid,
    pub valid_from: Timestamp,
    pub valid_to: Option<Timestamp>,
}

/// Graph database core structure
pub struct GraphDB {
    cas: Arc<RwLock<PackCAS>>,

    // RID -> current CID mapping (in-memory cache)
    rid_to_cid: Arc<RwLock<HashMap<Rid, Cid>>>,

    // Temporal RID mappings (RID -> timeline of CIDs)
    temporal_rid_mappings: Arc<RwLock<HashMap<Rid, BTreeMap<Timestamp, Cid>>>>,

    // Adjacency lists (RID -> outgoing edges)
    adjacency: Arc<RwLock<HashMap<Rid, Vec<AdjEntry>>>>,

    // Reverse adjacency (RID -> incoming edges)
    reverse_adjacency: Arc<RwLock<HashMap<Rid, Vec<AdjEntry>>>>,

    // Posting lists for search
    postings: Arc<RwLock<HashMap<String, Vec<Posting>>>>,

    // Current timestamp for operations
    current_timestamp: Arc<RwLock<Timestamp>>,
}

impl GraphDB {
    /// Create a new graph database instance
    pub async fn new(cas: PackCAS) -> Self {
        Self {
            cas: Arc::new(RwLock::new(cas)),
            rid_to_cid: Arc::new(RwLock::new(HashMap::new())),
            temporal_rid_mappings: Arc::new(RwLock::new(HashMap::new())),
            adjacency: Arc::new(RwLock::new(HashMap::new())),
            reverse_adjacency: Arc::new(RwLock::new(HashMap::new())),
            postings: Arc::new(RwLock::new(HashMap::new())),
            current_timestamp: Arc::new(RwLock::new(Timestamp::now())),
        }
    }

    /// Set the current timestamp for operations (for testing/temporal control)
    pub async fn set_timestamp(&self, ts: Timestamp) {
        *self.current_timestamp.write().await = ts;
    }

    /// Create a new node with initial data
    pub async fn create_node(&self, data: &[u8]) -> Result<Rid, Box<dyn std::error::Error>> {
        let ts = *self.current_timestamp.read().await;

        // Generate new RID (simplified - in real impl, use proper ID generation)
        let rid = Rid(self.rid_to_cid.read().await.len() as u64 + 1);

        // Store data in CAS
        let cid = {
            let mut cas = self.cas.write().await;
            cas.put(data, 0, PackBand::Small).await?
        };

        // Update mappings
        {
            let mut rid_to_cid = self.rid_to_cid.write().await;
            let mut temporal = self.temporal_rid_mappings.write().await;

            rid_to_cid.insert(rid, cid);
            temporal.entry(rid).or_insert_with(BTreeMap::new).insert(ts, cid);
        }

        // Index for search if it's text data
        if let Ok(text) = std::str::from_utf8(data) {
            self.index_text(rid, text, ts).await;
        }

        info!("Created node {} with CID {:?}", rid, cid);
        Ok(rid)
    }

    /// Update a node's data
    pub async fn update_node(&self, rid: Rid, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let ts = *self.current_timestamp.read().await;

        let cid = {
            let mut cas = self.cas.write().await;
            cas.put(data, 0, PackBand::Small).await?
        };

        // Update mappings
        {
            let mut rid_to_cid = self.rid_to_cid.write().await;
            let mut temporal = self.temporal_rid_mappings.write().await;

            rid_to_cid.insert(rid, cid);
            temporal.entry(rid).or_insert_with(BTreeMap::new).insert(ts, cid);
        }

        // Re-index for search
        if let Ok(text) = std::str::from_utf8(data) {
            self.index_text(rid, text, ts).await;
        }

        debug!("Updated node {} to CID {:?}", rid, cid);
        Ok(())
    }

    /// Get current data for a node
    pub async fn get_node(&self, rid: Rid) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let cid = {
            let rid_to_cid = self.rid_to_cid.read().await;
            rid_to_cid.get(&rid).cloned()
        };

        if let Some(cid) = cid {
            let cas = self.cas.read().await;
            Ok(Some(cas.get(&cid).await?))
        } else {
            Ok(None)
        }
    }

    /// Get node data at a specific timestamp (temporal query)
    pub async fn get_node_at(&self, rid: Rid, as_of: Timestamp) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let cid = {
            let temporal = self.temporal_rid_mappings.read().await;
            if let Some(timeline) = temporal.get(&rid) {
                // Find the most recent CID valid at as_of
                timeline.range(..=as_of).next_back().map(|(_, cid)| *cid)
            } else {
                None
            }
        };

        if let Some(cid) = cid {
            let cas = self.cas.read().await;
            Ok(Some(cas.get(&cid).await?))
        } else {
            Ok(None)
        }
    }

    /// Create an edge between nodes
    pub async fn create_edge(&self, from: Rid, to: Rid, label: LabelId, properties: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let ts = *self.current_timestamp.read().await;

        let prop_cid = {
            let mut cas = self.cas.write().await;
            cas.put(properties, 1, PackBand::Small).await?
        };

        let entry = AdjEntry {
            target: to,
            label,
            properties: prop_cid,
            timestamp: ts,
        };

        // Update adjacency lists
        {
            let mut adj = self.adjacency.write().await;
            let mut rev_adj = self.reverse_adjacency.write().await;

            adj.entry(from).or_insert_with(Vec::new).push(entry.clone());
            rev_adj.entry(to).or_insert_with(Vec::new).push(AdjEntry {
                target: from,
                label,
                properties: prop_cid,
                timestamp: ts,
            });
        }

        debug!("Created edge {} --({})--> {}", from, label.0, to);
        Ok(())
    }

    /// Traverse graph from a starting node
    pub async fn traverse(&self, from: Rid, labels: Option<&[LabelId]>, max_depth: usize, as_of: Option<Timestamp>)
        -> Result<Vec<(Rid, usize)>, Box<dyn std::error::Error>>
    {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        let mut queue = vec![(from, 0)]; // (node, depth)

        let adj = self.adjacency.read().await;

        while let Some((current, depth)) = queue.pop() {
            if depth > max_depth || !visited.insert(current) {
                continue;
            }

            result.push((current, depth));

            if depth < max_depth {
                if let Some(edges) = adj.get(&current) {
                    for edge in edges {
                        // Check timestamp if as_of is specified
                        if let Some(as_of) = as_of {
                            if edge.timestamp > as_of {
                                continue;
                            }
                        }

                        // Check label filter
                        if let Some(labels) = labels {
                            if !labels.contains(&edge.label) {
                                continue;
                            }
                        }

                        queue.push((edge.target, depth + 1));
                    }
                }
            }
        }

        Ok(result)
    }

    /// Search nodes by text content
    pub async fn search(&self, query: &str) -> Result<Vec<(Rid, f32)>, Box<dyn std::error::Error>> {
        let postings = self.postings.read().await;
        let mut results = HashMap::new();

        // Simple term-based search (no ranking yet)
        if let Some(posts) = postings.get(query) {
            for post in posts {
                *results.entry(post.rid).or_insert(0.0) += 1.0; // Simple TF scoring
            }
        }

        let mut sorted_results: Vec<_> = results.into_iter().collect();
        sorted_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(sorted_results)
    }

    /// Index text content for search
    async fn index_text(&self, rid: Rid, text: &str, timestamp: Timestamp) {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut postings = self.postings.write().await;

        for (pos, word) in words.iter().enumerate() {
            let posting = Posting {
                term: word.to_lowercase(),
                rid,
                positions: vec![pos as u32],
                timestamp,
            };

            postings.entry(word.to_lowercase())
                .or_insert_with(Vec::new)
                .push(posting);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_graph_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        // Create nodes
        let node1 = graph.create_node(b"Hello World").await.unwrap();
        let node2 = graph.create_node(b"Foo Bar").await.unwrap();

        // Create edge
        let label = LabelId(1);
        graph.create_edge(node1, node2, label, b"connects to").await.unwrap();

        // Test that nodes exist in graph structure
        assert!(graph.rid_to_cid.read().await.contains_key(&node1));
        assert!(graph.rid_to_cid.read().await.contains_key(&node2));

        // Test that edges exist
        let edges_from_1 = graph.adjacency.read().await.get(&node1).cloned().unwrap_or_default();
        assert!(!edges_from_1.is_empty());

        // Search
        let search_results = graph.search("hello").await.unwrap();
        assert!(!search_results.is_empty());
    }

    #[tokio::test]
    async fn test_temporal_queries() {
        let temp_dir = tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        let node = graph.create_node(b"Version 1").await.unwrap();

        // Set timestamp to future
        let future_ts = Timestamp(1000000);
        graph.set_timestamp(future_ts).await;
        graph.update_node(node, b"Version 2").await.unwrap();

        // Test that temporal mapping was created
        let temporal_mappings = graph.temporal_rid_mappings.read().await;
        assert!(temporal_mappings.contains_key(&node));

        // Test timestamp was updated
        assert_eq!(*graph.current_timestamp.read().await, future_ts);
    }
}
