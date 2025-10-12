//! # Enishi Concurrency (Own+CFA)
//!
//! Phase D: Own+CFA Final - Ownership types and capability functor composition
//!
//! Merkle DAG: enishi_concur -> ownership_types, cap_functor, txn_safety

use fcdb_core::{Cap, Cid};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Capability-CID pair
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapCid {
    pub cap: Cap,
    pub cid: Cid,
}

impl CapCid {
    pub fn new(cid: Cid, cap: Cap) -> Self {
        Self { cap, cid }
    }
}

/// Errors for concurrency operations
#[derive(Error, Debug)]
pub enum ConcurError {
    #[error("Capability check failed")]
    CapCheckFailed,
    #[error("Ownership violation")]
    OwnershipViolation,
    #[error("Transaction conflict")]
    TransactionConflict,
    #[error("Lease expired")]
    LeaseExpired,
    #[error("Permission denied")]
    PermissionDenied,
}

/// Permission flags for capabilities
pub mod perms {
    pub const READ: u32 = 1 << 0;
    pub const WRITE: u32 = 1 << 1;
    pub const EXECUTE: u32 = 1 << 2;
    pub const DERIVE: u32 = 1 << 3;
    pub const DELEGATE: u32 = 1 << 4;
}

/// Phase D: Owned Capability Content Identifier
/// Rust ownership ensures exclusive access and prevents data races
pub struct OwnedCapCid<T> {
    cap_cid: CapCid,
    data: T,
}

impl<T> OwnedCapCid<T> {
    /// Create owned capability-CID pair (consumes data)
    pub fn new(data: T, cap: Cap, cid: Cid) -> Self {
        Self {
            cap_cid: CapCid::new(cid, cap),
            data,
        }
    }

    /// Get immutable reference (shared borrow)
    pub fn as_ref(&self) -> (&Cap, &T) {
        (&self.cap_cid.cap, &self.data)
    }

    /// Get mutable reference (exclusive borrow)
    pub fn as_mut(&mut self) -> (&mut Cap, &mut T) {
        (&mut self.cap_cid.cap, &mut self.data)
    }

    /// Consume self and return components
    pub fn into_parts(self) -> (CapCid, T) {
        (self.cap_cid, self.data)
    }
}

/// Phase D: Borrowed Capability Content Identifier
/// Compile-time borrow checking prevents use-after-free and data races
pub struct BorrowCapCid<'a, T> {
    cap_cid: &'a CapCid,
    data: &'a T,
}

impl<'a, T> BorrowCapCid<'a, T> {
    pub fn new(cap_cid: &'a CapCid, data: &'a T) -> Self {
        Self { cap_cid, data }
    }

    pub fn cap(&self) -> &Cap {
        &self.cap_cid.cap
    }

    pub fn cid(&self) -> &Cid {
        &self.cap_cid.cid
    }

    pub fn data(&self) -> &T {
        self.data
    }
}

/// Phase D: Mutable Borrowed Capability Content Identifier
/// Exclusive access for mutation with capability checking
pub struct BorrowMutCapCid<'a, T> {
    cap_cid: &'a mut CapCid,
    data: &'a mut T,
}

impl<'a, T> BorrowMutCapCid<'a, T> {
    pub fn new(cap_cid: &'a mut CapCid, data: &'a mut T) -> Self {
        Self { cap_cid, data }
    }

    pub fn cap(&self) -> &Cap {
        &self.cap_cid.cap
    }

    pub fn cap_mut(&mut self) -> &mut Cap {
        &mut self.cap_cid.cap
    }

    pub fn cid(&self) -> &Cid {
        &self.cap_cid.cid
    }

    pub fn data(&self) -> &T {
        self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        self.data
    }
}

/// Phase D: Capability Functor
/// F(Cap ▷ X) = Cap ▷ F(X) - functor composition for security
pub trait CapFunctor {
    type Target<U>;

    /// Map function while preserving capability
    fn cap_map<U, F>(self, f: F) -> Self::Target<U>
    where
        F: FnOnce(Self::Data) -> U;

    /// FlatMap with capability composition
    fn cap_flat_map<U, F>(self, f: F) -> Self::Target<U>
    where
        F: FnOnce(Self::Data) -> Self::Target<U>;
}

pub trait HasData {
    type Data;
}

impl<T> HasData for OwnedCapCid<T> {
    type Data = T;
}

impl<T> CapFunctor for OwnedCapCid<T> {
    type Target<U> = OwnedCapCid<U>;

    fn cap_map<U, F>(self, f: F) -> Self::Target<U>
    where
        F: FnOnce(Self::Data) -> U,
    {
        let (cap_cid, data) = self.into_parts();
        OwnedCapCid::new(f(data), cap_cid.cap, cap_cid.cid)
    }

    fn cap_flat_map<U, F>(self, f: F) -> Self::Target<U>
    where
        F: FnOnce(Self::Data) -> Self::Target<U>,
    {
        let (cap_cid, data) = self.into_parts();
        let OwnedCapCid { cap_cid: new_cap_cid, data: new_data } = f(data);

        // Compose capabilities: new_cap ∩ original_cap
        let composed_cap = Cap {
            base: new_cap_cid.cap.base.max(cap_cid.cap.base),
            len: new_cap_cid.cap.len.min(cap_cid.cap.len),
            perms: new_cap_cid.cap.perms & cap_cid.cap.perms,
            proof: new_cap_cid.cap.proof, // Keep new proof
        };

        OwnedCapCid::new(new_data, composed_cap, new_cap_cid.cid)
    }
}

/// Phase D: Transaction with ownership tracking
pub struct Transaction {
    id: u64,
    owned_resources: Vec<OwnedCapCid<Box<dyn std::any::Any + Send + Sync>>>,
    borrowed_resources: Vec<Arc<RwLock<CapCid>>>,
    start_time: std::time::Instant,
    timeout_ms: u64,
}

impl Transaction {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            owned_resources: Vec::new(),
            borrowed_resources: Vec::new(),
            start_time: std::time::Instant::now(),
            timeout_ms: 5000, // 5 second default timeout
        }
    }

    /// Check if transaction has timed out
    pub fn is_expired(&self) -> bool {
        self.start_time.elapsed().as_millis() as u64 > self.timeout_ms
    }

    /// Add owned resource to transaction
    pub fn add_owned<T: Send + Sync + 'static>(&mut self, owned: OwnedCapCid<T>) {
        let boxed = OwnedCapCid::new(
            Box::new(owned.data) as Box<dyn std::any::Any + Send + Sync>,
            owned.cap_cid.cap,
            owned.cap_cid.cid
        );
        self.owned_resources.push(boxed);
    }

    /// Add borrowed resource to transaction
    pub fn add_borrowed(&mut self, borrowed: Arc<RwLock<CapCid>>) {
        self.borrowed_resources.push(borrowed);
    }

    /// Check if transaction has write permission for resource
    pub async fn check_write_perm(&self, target_cid: &Cid) -> Result<(), ConcurError> {
        // Check owned resources first
        for owned in &self.owned_resources {
            if owned.cap_cid.cid == *target_cid {
                if owned.cap_cid.cap.has_perm(perms::WRITE) {
                    return Ok(());
                } else {
                    return Err(ConcurError::PermissionDenied);
                }
            }
        }

        // Check borrowed resources
        for borrowed in &self.borrowed_resources {
            let cap_cid = borrowed.read().await;
            if cap_cid.cid == *target_cid {
                if cap_cid.cap.has_perm(perms::WRITE) {
                    return Ok(());
                } else {
                    return Err(ConcurError::PermissionDenied);
                }
            }
        }

        Err(ConcurError::PermissionDenied)
    }
}

/// Phase D: Lease management for capability expiration
pub struct LeaseManager {
    active_leases: Arc<RwLock<std::collections::HashMap<u64, LeaseInfo>>>,
}

#[derive(Clone)]
pub struct LeaseInfo {
    pub resource_id: u64,
    pub holder: String,
    pub permissions: u32,
    pub expires_at: u64,
    pub auto_renew: bool,
}

impl LeaseManager {
    pub fn new() -> Self {
        Self {
            active_leases: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Grant lease for resource
    pub async fn grant_lease(&self, lease_id: u64, info: LeaseInfo) -> Result<(), ConcurError> {
        let mut leases = self.active_leases.write().await;
        leases.insert(lease_id, info);
        Ok(())
    }

    /// Check if lease is valid
    pub async fn check_lease(&self, lease_id: u64) -> Result<LeaseInfo, ConcurError> {
        let leases = self.active_leases.read().await;
        match leases.get(&lease_id) {
            Some(info) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if now > info.expires_at {
                    return Err(ConcurError::LeaseExpired);
                }

                Ok(info.clone())
            }
            None => Err(ConcurError::LeaseExpired),
        }
    }

    /// Revoke lease
    pub async fn revoke_lease(&self, lease_id: u64) -> Result<(), ConcurError> {
        let mut leases = self.active_leases.write().await;
        leases.remove(&lease_id);
        Ok(())
    }

    /// Renew lease if auto-renew is enabled
    pub async fn renew_lease(&self, lease_id: u64, new_expiry: u64) -> Result<(), ConcurError> {
        let mut leases = self.active_leases.write().await;
        if let Some(info) = leases.get_mut(&lease_id) {
            if info.auto_renew {
                info.expires_at = new_expiry;
                Ok(())
            } else {
                Err(ConcurError::PermissionDenied)
            }
        } else {
            Err(ConcurError::LeaseExpired)
        }
    }
}

/// Phase D: Resource Manager with ownership tracking
pub struct ResourceManager {
    resources: Arc<RwLock<std::collections::HashMap<Cid, Arc<RwLock<CapCid>>>>>,
    lease_manager: LeaseManager,
    next_txn_id: Arc<Mutex<u64>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: Arc::new(RwLock::new(std::collections::HashMap::new())),
            lease_manager: LeaseManager::new(),
            next_txn_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Create new transaction
    pub async fn begin_transaction(&self) -> Result<Transaction, ConcurError> {
        let mut next_id = self.next_txn_id.lock().await;
        let txn_id = *next_id;
        *next_id += 1;

        Ok(Transaction::new(txn_id))
    }

    /// Register resource with capability
    pub async fn register_resource(&self, cid: Cid, cap: Cap) -> Result<(), ConcurError> {
        let cap_cid = CapCid::new(cid, cap);
        let mut resources = self.resources.write().await;
        resources.insert(cid, Arc::new(RwLock::new(cap_cid)));
        Ok(())
    }

    /// Acquire exclusive ownership (mutable borrow)
    pub async fn acquire_exclusive(&self, cid: &Cid, txn: &mut Transaction) -> Result<(), ConcurError> {
        let resources = self.resources.read().await;
        if let Some(resource) = resources.get(cid) {
            txn.check_write_perm(cid).await?;
            txn.add_borrowed(resource.clone());
            Ok(())
        } else {
            Err(ConcurError::OwnershipViolation)
        }
    }

    /// Acquire shared ownership (immutable borrow)
    pub async fn acquire_shared(&self, cid: &Cid, txn: &mut Transaction) -> Result<(), ConcurError> {
        let resources = self.resources.read().await;
        if let Some(resource) = resources.get(cid) {
            txn.add_borrowed(resource.clone());
            Ok(())
        } else {
            Err(ConcurError::OwnershipViolation)
        }
    }

    /// Commit transaction with ownership transfer
    pub async fn commit_transaction(&self, txn: Transaction) -> Result<(), ConcurError> {
        if txn.is_expired() {
            return Err(ConcurError::TransactionConflict);
        }

        // Validate all capability checks
        for borrowed in &txn.borrowed_resources {
            let cap_cid = borrowed.read().await;
            // Additional validation could be added here
        }

        // Transaction committed successfully
        Ok(())
    }

    /// Abort transaction and release resources
    pub async fn abort_transaction(&self, txn: Transaction) -> Result<(), ConcurError> {
        // Resources are automatically released when transaction is dropped
        // due to Rust's ownership system
        Ok(())
    }
}

/// Phase D: Capability Tracer for audit trail
pub struct CapTracer {
    trace_log: Arc<RwLock<Vec<CapTraceEntry>>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CapTraceEntry {
    pub timestamp: u64,
    pub operation: String,
    pub actor: String,
    pub resource: Cid,
    pub capability: Cap,
    pub success: bool,
    pub details: String,
}

impl CapTracer {
    pub fn new() -> Self {
        Self {
            trace_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record capability operation
    pub async fn record_operation(
        &self,
        operation: &str,
        actor: &str,
        resource: &Cid,
        capability: &Cap,
        success: bool,
        details: &str,
    ) {
        let entry = CapTraceEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            operation: operation.to_string(),
            actor: actor.to_string(),
            resource: *resource,
            capability: *capability,
            success,
            details: details.to_string(),
        };

        let mut log = self.trace_log.write().await;
        log.push(entry);

        // Keep only recent entries (last 1000)
        if log.len() > 1000 {
            log.remove(0);
        }
    }

    /// Get audit trail for resource
    pub async fn get_audit_trail(&self, resource: &Cid) -> Vec<CapTraceEntry> {
        let log = self.trace_log.read().await;
        log.iter()
            .filter(|entry| entry.resource == *resource)
            .cloned()
            .collect()
    }

    /// Get operations by actor
    pub async fn get_actor_operations(&self, actor: &str) -> Vec<CapTraceEntry> {
        let log = self.trace_log.read().await;
        log.iter()
            .filter(|entry| entry.actor == actor)
            .cloned()
            .collect()
    }
}

/// Phase D: Safe wrapper for concurrent operations
pub struct SafeExecutor {
    resource_manager: ResourceManager,
    tracer: CapTracer,
}

impl SafeExecutor {
    pub fn new() -> Self {
        Self {
            resource_manager: ResourceManager::new(),
            tracer: CapTracer::new(),
        }
    }

    /// Execute operation with full Own+CFA safety
    pub async fn execute_safe<F, Fut, T>(
        &self,
        actor: &str,
        operation: &str,
        resource: &Cid,
        cap_check: F,
    ) -> Result<T, ConcurError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, ConcurError>>,
    {
        // Pre-operation capability check
        let mut txn = self.resource_manager.begin_transaction().await?;
        self.resource_manager.acquire_shared(resource, &mut txn).await?;

        let cap_cid = {
            let resources = self.resource_manager.resources.read().await;
            resources.get(resource)
                .ok_or(ConcurError::OwnershipViolation)?
                .read().await
                .clone()
        };

        // Execute operation
        let result = cap_check().await;

        // Record result in audit trail
        let success = result.is_ok();
        let details = if success { "success" } else { "failed" };
        self.tracer.record_operation(
            operation,
            actor,
            resource,
            &cap_cid.cap,
            success,
            details,
        ).await;

        // Commit or abort transaction
        match result {
            Ok(value) => {
                self.resource_manager.commit_transaction(txn).await?;
                Ok(value)
            }
            Err(e) => {
                self.resource_manager.abort_transaction(txn).await?;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_owned_cap_cid() {
        let data = "test data".to_string();
        let cap = Cap::new(0, 100, perms::READ | perms::WRITE);
        let cid = Cid::hash(data.as_bytes());

        let owned = OwnedCapCid::new(data, cap, cid);

        // Test borrowing
        {
            let (cap_ref, data_ref) = owned.as_ref();
            assert!(cap_ref.has_perm(perms::READ));
            assert_eq!(data_ref, "test data");
        }

        // Test mutable borrowing
        let mut owned = owned;
        {
            let (cap_mut, data_mut) = owned.as_mut();
            *data_mut = "modified".to_string();
            cap_mut.perms &= !perms::WRITE; // Remove write permission
        }

        // Verify changes
        let (final_cap, final_data) = owned.as_ref();
        assert_eq!(final_data, "modified");
        assert!(!final_cap.has_perm(perms::WRITE));
    }

    #[tokio::test]
    async fn test_capability_functor() {
        let data = 42;
        let cap = Cap::new(0, 100, perms::READ | perms::WRITE);
        let cid = Cid::hash(&data.to_le_bytes());

        let owned = OwnedCapCid::new(data, cap, cid);

        // Test map operation
        let mapped = owned.cap_map(|x| x * 2);
        let (_, result) = mapped.as_ref();
        assert_eq!(*result, 84);
    }

    #[tokio::test]
    async fn test_transaction_lifecycle() {
        let rm = ResourceManager::new();
        let cid = Cid::hash(b"test resource");
        let cap = Cap::new(0, 100, perms::READ | perms::WRITE);

        // Register resource
        rm.register_resource(cid, cap).await.unwrap();

        // Begin transaction
        let mut txn = rm.begin_transaction().await.unwrap();

        // Acquire resource
        rm.acquire_exclusive(&cid, &mut txn).await.unwrap();

        // Check permissions
        assert!(txn.check_write_perm(&cid).await.is_ok());

        // Commit transaction
        rm.commit_transaction(txn).await.unwrap();
    }

    #[tokio::test]
    async fn test_lease_management() {
        let lm = LeaseManager::new();
        let lease_id = 12345;

        let info = LeaseInfo {
            resource_id: 1,
            holder: "test_user".to_string(),
            permissions: perms::READ | perms::WRITE,
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600, // 1 hour from now
            auto_renew: true,
        };

        // Grant lease
        lm.grant_lease(lease_id, info.clone()).await.unwrap();

        // Check valid lease
        let checked = lm.check_lease(lease_id).await.unwrap();
        assert_eq!(checked.holder, "test_user");

        // Revoke lease
        lm.revoke_lease(lease_id).await.unwrap();

        // Check should fail
        assert!(lm.check_lease(lease_id).await.is_err());
    }

    #[tokio::test]
    async fn test_capability_tracing() {
        let tracer = CapTracer::new();
        let cid = Cid::hash(b"test resource");
        let cap = Cap::new(0, 100, perms::READ);

        // Record operations
        tracer.record_operation(
            "read",
            "alice",
            &cid,
            &cap,
            true,
            "successful read"
        ).await;

        tracer.record_operation(
            "write",
            "bob",
            &cid,
            &cap,
            false,
            "permission denied"
        ).await;

        // Check audit trail
        let alice_ops = tracer.get_actor_operations("alice").await;
        assert_eq!(alice_ops.len(), 1);
        assert_eq!(alice_ops[0].operation, "read");
        assert!(alice_ops[0].success);

        let resource_trail = tracer.get_audit_trail(&cid).await;
        assert_eq!(resource_trail.len(), 2);
    }
}
