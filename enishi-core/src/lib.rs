//! # Enishi Core
//!
//! Core data structures and algorithms for the Enishi graph database.
//!
//! Merkle DAG: enishi_core -> cid, cap, monoid, path_sig, class_sig, trace_normal_form

use serde::{Deserialize, Serialize};
use std::fmt;

/// Content Identifier (CID) - BLAKE3/256 hash
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cid([u8; 32]);

impl Cid {
    /// Create a CID from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create a CID by hashing data with BLAKE3
    pub fn hash(data: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        Self(hash.into())
    }

    /// Create a CID from JSON data using JCS (JSON Canonicalization Scheme)
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

/// Capability (Cap) - Cheri-style capability with base, length, permissions, and proof
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cap {
    pub base: u64,
    pub len: u64,
    pub perms: u32,
    pub proof: [u8; 16],
}

impl Cap {
    /// Create a new capability
    pub fn new(base: u64, len: u64, perms: u32) -> Self {
        let proof = rand::random::<[u8; 16]>();
        Self { base, len, perms, proof }
    }

    /// Check if an address is within the capability's bounds
    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.base && addr < self.base + self.len
    }

    /// Check if permission is granted
    pub fn has_perm(&self, perm: u32) -> bool {
        (self.perms & perm) != 0
    }
}

impl fmt::Debug for Cap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cap{{base: {}, len: {}, perms: {:#010b}, proof: {}}}",
               self.base, self.len, self.perms, hex::encode(&self.proof[..4]))
    }
}

/// Capability Content Identifier - CID with capability view
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapCid {
    pub cid: Cid,
    pub cap: Cap,
}

impl CapCid {
    pub fn new(cid: Cid, cap: Cap) -> Self {
        Self { cid, cap }
    }
}

impl fmt::Debug for CapCid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CapCid{{cid: {:?}, cap: {:?}}}", self.cid, self.cap)
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

impl QKey {
    pub fn new(path_sig: Cid, class_sig: Cid, as_of: u64, cap_region: (u64, u64), type_part: u16) -> Self {
        Self {
            path_sig,
            class_sig,
            as_of,
            cap_region,
            type_part,
        }
    }

    /// Compute hash for this query key
    pub fn hash(&self) -> Cid {
        let mut data = Vec::new();
        data.extend_from_slice(self.path_sig.as_bytes());
        data.extend_from_slice(self.class_sig.as_bytes());
        data.extend_from_slice(&self.as_of.to_le_bytes());
        data.extend_from_slice(&self.cap_region.0.to_le_bytes());
        data.extend_from_slice(&self.cap_region.1.to_le_bytes());
        data.extend_from_slice(&self.type_part.to_le_bytes());
        Cid::hash(&data)
    }
}

impl fmt::Debug for QKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QKey{{path_sig: {:?}, class_sig: {:?}, as_of: {}, cap_region: {:?}, type_part: {}}}",
               self.path_sig, self.class_sig, self.as_of, self.cap_region, self.type_part)
    }
}

/// Monoid for deterministic composition of postings/adj/frontier/p2-p3
pub trait Monoid {
    fn empty() -> Self;
    fn combine(self, other: Self) -> Self;
}

/// Varint encoding/decoding utilities
pub mod varint {
    use integer_encoding::{VarInt, VarIntReader, VarIntWriter};
    use std::io::{Read, Write};

    pub fn encode_u64(value: u64, buf: &mut Vec<u8>) {
        buf.write_varint(value).unwrap();
    }

    pub fn decode_u64<R: Read>(reader: &mut R) -> Result<u64, std::io::Error> {
        reader.read_varint()
    }

    pub fn encode_i64(value: i64, buf: &mut Vec<u8>) {
        buf.write_varint(value).unwrap();
    }

    pub fn decode_i64<R: Read>(reader: &mut R) -> Result<i64, std::io::Error> {
        reader.read_varint()
    }
}

/// Path signature computation for query optimization
pub fn compute_path_sig(path: &[&str]) -> Cid {
    let mut data = Vec::new();
    for segment in path {
        data.extend_from_slice(segment.as_bytes());
        data.push(0); // null terminator
    }
    Cid::hash(&data)
}

/// Class signature computation for type-based optimization
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
    fn test_cid_from_json() {
        let value = serde_json::json!({"key": "value", "number": 42});
        let cid = Cid::from_json(&value).unwrap();
        assert_eq!(cid.as_bytes().len(), 32);
    }

    #[test]
    fn test_cap_bounds_check() {
        let cap = Cap::new(100, 50, 0b111);
        assert!(cap.contains(120));
        assert!(cap.contains(149));
        assert!(!cap.contains(150));
        assert!(!cap.contains(99));
    }

    #[test]
    fn test_cap_permissions() {
        let cap = Cap::new(0, 100, 0b101); // perms: read and execute
        assert!(cap.has_perm(0b001)); // read
        assert!(!cap.has_perm(0b010)); // write
        assert!(cap.has_perm(0b100)); // execute
    }

    #[test]
    fn test_qkey_hash() {
        let path_sig = compute_path_sig(&["user", "posts"]);
        let class_sig = compute_class_sig(&["Post", "User"]);
        let qkey = QKey::new(path_sig, class_sig, 1234567890, (0, 1000), 42);
        let hash = qkey.hash();
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn test_varint_roundtrip() {
        let mut buf = Vec::new();
        varint::encode_u64(12345678901234567890, &mut buf);
        let mut reader = &buf[..];
        let decoded = varint::decode_u64(&mut reader).unwrap();
        assert_eq!(decoded, 12345678901234567890);
    }
}
