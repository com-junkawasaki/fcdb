//! # Enishi Core
//!
//! Core data structures and algorithms for the Enishi graph database.
//!
//! Merkle DAG: enishi_core -> cid, cap, monoid, path_sig, class_sig, trace_normal_form

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;

/// Content Identifier (CID) - BLAKE3/256 hash
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cid([u8; 32]);

impl Cid {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn hash(data: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        Self(hash.into())
    }

    pub fn from_json<T: serde::Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        let canonical_json = serde_json::to_string(value)?;
        Ok(Self::hash(canonical_json.as_bytes()))
    }
}

impl fmt::Debug for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cid({})", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

/// Capability (Cap) - Cheri-style capability
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cap {
    pub base: u64,
    pub len: u64,
    pub perms: u32,
    pub proof: [u8; 16],
}

impl Cap {
    pub fn new(base: u64, len: u64, perms: u32) -> Self {
        let proof = rand::random::<[u8; 16]>();
        Self { base, len, perms, proof }
    }

    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.base && addr < self.base + self.len
    }

    pub fn has_perm(&self, perm: u32) -> bool {
        (self.perms & perm) != 0
    }
}

/// Query Key for caching and indexing
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QKey {
    pub path_sig: Cid,
    pub class_sig: Cid,
    pub as_of: u64,
    pub cap_region: (u64, u64),
    pub type_part: u16,
}

/// Monoid for deterministic composition
pub trait Monoid {
    fn empty() -> Self;
    fn combine(self, other: Self) -> Self;
}

/// Varint encoding utilities
pub mod varint {
    use integer_encoding::{VarInt, VarIntReader, VarIntWriter};
    use std::io::{Read, Write};

    pub fn encode_u64(value: u64, buf: &mut Vec<u8>) {
        buf.write_varint(value).unwrap();
    }

    pub fn decode_u64<R: Read>(reader: &mut R) -> Result<u64, std::io::Error> {
        reader.read_varint()
    }
}

// ===== PHASE B: Path/Class Signatures =====

/// Path signature for query optimization
/// Computes a compact representation of query paths for caching
pub fn compute_path_sig(path: &[&str]) -> Cid {
    let mut data = Vec::new();
    for segment in path {
        data.extend_from_slice(segment.as_bytes());
        data.push(0); // null terminator
    }
    Cid::hash(&data)
}

/// Class signature for type-based optimization
/// Sorts classes to ensure deterministic signatures
pub fn compute_class_sig(classes: &[&str]) -> Cid {
    let mut sorted_classes = classes.to_vec();
    sorted_classes.sort();
    let mut data = Vec::new();
    for class in sorted_classes {
        data.extend_from_slice(class.as_bytes());
        data.push(0);
    }
    Cid::hash(&data)
}

// ===== PHASE B: Trace Normal Form =====

/// Trace element representing a single operation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceOp {
    NodeCreate { id: u64, data: Cid },
    EdgeCreate { from: u64, to: u64, label: u32, props: Cid },
    PropertyUpdate { node: u64, key: String, value: Cid },
}

/// Trace sequence for commutative operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trace {
    pub ops: Vec<TraceOp>,
    pub timestamp: u64,
}

impl Trace {
    pub fn new(timestamp: u64) -> Self {
        Self {
            ops: Vec::new(),
            timestamp,
        }
    }

    pub fn add_op(&mut self, op: TraceOp) {
        self.ops.push(op);
    }
}

impl Monoid for Trace {
    fn empty() -> Self {
        Self::new(0)
    }

    fn combine(mut self, other: Self) -> Self {
        // Simple concatenation - in full implementation would handle commutativity
        self.ops.extend(other.ops);
        self.timestamp = self.timestamp.max(other.timestamp);
        self
    }
}

/// Trace normal form - canonical representation for key reduction
pub struct TraceNF {
    pub canonical_form: Cid,
    pub commutative_groups: Vec<Vec<TraceOp>>,
}

impl TraceNF {
    /// Convert trace to normal form for key explosion reduction
    pub fn from_trace(trace: &Trace) -> Self {
        // Group commutative operations
        let mut node_ops = Vec::new();
        let mut edge_ops = Vec::new();
        let mut prop_ops = Vec::new();

        for op in &trace.ops {
            match op {
                TraceOp::NodeCreate { .. } => node_ops.push(op.clone()),
                TraceOp::EdgeCreate { .. } => edge_ops.push(op.clone()),
                TraceOp::PropertyUpdate { .. } => prop_ops.push(op.clone()),
            }
        }

        // Sort within each commutative group
        node_ops.sort_by_key(|op| match op {
            TraceOp::NodeCreate { id, .. } => *id,
            _ => 0,
        });

        edge_ops.sort_by_key(|op| match op {
            TraceOp::EdgeCreate { from, to, .. } => (*from, *to),
            _ => (0, 0),
        });

        prop_ops.sort_by_key(|op| match op {
            TraceOp::PropertyUpdate { node, key, .. } => (*node, key.clone()),
            _ => (0, String::new()),
        });

        let commutative_groups = vec![node_ops, edge_ops, prop_ops];

        // Compute canonical form
        let mut canonical_data = Vec::new();
        for group in &commutative_groups {
            for op in group {
                let op_json = serde_json::to_string(op).unwrap();
                canonical_data.extend_from_slice(op_json.as_bytes());
            }
        }

        Self {
            canonical_form: Cid::hash(&canonical_data),
            commutative_groups,
        }
    }
}

// ===== PHASE B: Manifest Diffing =====

/// Manifest entry for query result caching
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub qkey: QKey,
    pub result_cid: Cid,
    pub last_accessed: u64,
    pub access_count: u64,
}

/// Manifest with diff support for efficient updates
#[derive(Clone, Debug)]
pub struct Manifest {
    pub base_version: u64,
    pub entries: HashMap<QKey, ManifestEntry>,
    pub diffs: Vec<ManifestDiff>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestDiff {
    pub version: u64,
    pub timestamp: u64,
    pub added: Vec<ManifestEntry>,
    pub removed: Vec<QKey>,
    pub updated: Vec<(QKey, Cid)>, // qkey -> new_result_cid
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            base_version: 0,
            entries: HashMap::new(),
            diffs: Vec::new(),
        }
    }

    /// Apply diff to manifest
    pub fn apply_diff(&mut self, diff: ManifestDiff) {
        // Remove entries
        for qkey in &diff.removed {
            self.entries.remove(qkey);
        }

        // Update entries
        for (qkey, new_cid) in &diff.updated {
            if let Some(entry) = self.entries.get_mut(qkey) {
                entry.result_cid = *new_cid;
            }
        }

        // Add new entries
        for entry in &diff.added {
            self.entries.insert(entry.qkey.clone(), entry.clone());
        }

        self.diffs.push(diff);
    }

    /// Get result for query key, checking diffs
    pub fn get_result(&self, qkey: &QKey) -> Option<Cid> {
        self.entries.get(qkey).map(|entry| entry.result_cid)
    }

    /// Create diff from current state to new state
    pub fn create_diff(&self, new_entries: HashMap<QKey, ManifestEntry>) -> ManifestDiff {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut updated = Vec::new();

        // Find added entries
        for (qkey, entry) in &new_entries {
            if !self.entries.contains_key(qkey) {
                added.push(entry.clone());
            }
        }

        // Find removed and updated entries
        for (qkey, old_entry) in &self.entries {
            if let Some(new_entry) = new_entries.get(qkey) {
                if old_entry.result_cid != new_entry.result_cid {
                    updated.push((qkey.clone(), new_entry.result_cid));
                }
            } else {
                removed.push(qkey.clone());
            }
        }

        ManifestDiff {
            version: self.base_version + self.diffs.len() as u64 + 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            added,
            removed,
            updated,
        }
    }
}

// ===== PHASE B: Query Optimization =====

/// Query plan with optimization hints
#[derive(Clone, Debug)]
pub struct QueryPlan {
    pub qkey: QKey,
    pub estimated_cost: f64,
    pub use_path_sig: bool,
    pub use_class_sig: bool,
    pub trace_optimized: bool,
    pub manifest_cached: bool,
}

impl QueryPlan {
    /// Create optimized plan for query
    pub fn optimize(path: &[&str], classes: &[&str], as_of: u64) -> Self {
        let path_sig = compute_path_sig(path);
        let class_sig = compute_class_sig(classes);

        let qkey = QKey {
            path_sig,
            class_sig,
            as_of,
            cap_region: (0, u64::MAX), // Full range
            type_part: 0,
        };

        // Estimate cost based on path complexity
        let path_complexity = path.len() as f64;
        let class_selectivity = classes.len() as f64;

        let estimated_cost = path_complexity * class_selectivity * 10.0; // Simplified

        Self {
            qkey,
            estimated_cost,
            use_path_sig: path.len() > 1,
            use_class_sig: classes.len() > 1,
            trace_optimized: true, // Phase B feature
            manifest_cached: true, // Assume manifest caching
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_creation() {
        let data = b"hello world";
        let cid = Cid::hash(data);
        assert_eq!(cid.as_bytes().len(), 32);
    }

    #[test]
    fn test_path_signature() {
        let path1 = &["user", "posts"];
        let path2 = &["user", "posts"];
        let path3 = &["posts", "user"];

        let sig1 = compute_path_sig(path1);
        let sig2 = compute_path_sig(path2);
        let sig3 = compute_path_sig(path3);

        assert_eq!(sig1, sig2); // Same path
        assert_ne!(sig1, sig3); // Different path
    }

    #[test]
    fn test_class_signature() {
        let classes1 = &["User", "Post"];
        let classes2 = &["Post", "User"]; // Different order
        let classes3 = &["User", "Comment"];

        let sig1 = compute_class_sig(classes1);
        let sig2 = compute_class_sig(classes2);
        let sig3 = compute_class_sig(classes3);

        assert_eq!(sig1, sig2); // Same classes, different order
        assert_ne!(sig1, sig3); // Different classes
    }

    #[test]
    fn test_trace_normal_form() {
        let mut trace = Trace::new(1234567890);
        trace.add_op(TraceOp::NodeCreate { id: 1, data: Cid::hash(b"node1") });
        trace.add_op(TraceOp::NodeCreate { id: 2, data: Cid::hash(b"node2") });

        let nf = TraceNF::from_trace(&trace);
        assert!(!nf.commutative_groups.is_empty());
    }

    #[test]
    fn test_manifest_diffing() {
        let mut manifest = Manifest::new();

        let qkey1 = QKey {
            path_sig: compute_path_sig(&["test"]),
            class_sig: compute_class_sig(&["Test"]),
            as_of: 1000,
            cap_region: (0, 100),
            type_part: 1,
        };

        let entry1 = ManifestEntry {
            qkey: qkey1.clone(),
            result_cid: Cid::hash(b"result1"),
            last_accessed: 1000,
            access_count: 1,
        };

        let mut new_entries = HashMap::new();
        new_entries.insert(qkey1.clone(), entry1);

        let diff = manifest.create_diff(new_entries);
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.removed.len(), 0);

        manifest.apply_diff(diff);
        assert!(manifest.get_result(&qkey1).is_some());
    }

    #[test]
    fn test_query_plan_optimization() {
        let path = &["user", "posts", "comments"];
        let classes = &["User", "Post", "Comment"];

        let plan = QueryPlan::optimize(path, classes, 1234567890);

        assert!(plan.use_path_sig);
        assert!(plan.use_class_sig);
        assert!(plan.trace_optimized);
        assert!(plan.manifest_cached);
        assert!(plan.estimated_cost > 0.0);
    }
}