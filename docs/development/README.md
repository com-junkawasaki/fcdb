# Own-CFA-Enishi Development Guide

## Overview

This guide provides comprehensive information for developers working on or extending the Own-CFA-Enishi system. It covers architecture, coding standards, testing practices, and contribution guidelines.

## Development Environment

### Prerequisites

#### System Requirements
- **Rust**: 1.70+ with rustfmt and clippy
- **Docker**: 20.10+ for containerized development
- **Git**: 2.30+ with LFS support
- **Make**: Build automation
- **Python**: 3.8+ for tooling scripts

#### Installation
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install additional components
rustup component add rustfmt clippy
rustup component add llvm-tools-preview  # for profiling

# Install development tools
cargo install cargo-watch cargo-expand cargo-udeps cargo-outdated
cargo install flamegraph  # for profiling

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
```

### Project Structure
```
enishi/
├── src/                    # Main application
│   ├── main.rs            # Application entry point
│   ├── config.rs          # Configuration management
│   ├── server.rs          # HTTP server
│   ├── health.rs          # Health checking
│   └── metrics.rs         # Metrics collection
├── enishi-core/           # Core data types and algorithms
├── enishi-cas/            # Content-addressable storage
├── enishi-graph/          # Graph database operations
├── enishi-exec/           # Query execution and optimization
├── enishi-concur/         # Ownership and concurrency
├── enishi-api/            # API interfaces
├── enishi-tools/          # Utilities and benchmarks
├── validation/            # Validation suite
├── docs/                  # Documentation
└── docker/                # Docker configurations
```

## Building and Testing

### Basic Build
```bash
# Build debug version
cargo build

# Build release version
cargo build --release

# Build specific crate
cargo build -p enishi-core

# Run all tests
cargo test --workspace

# Run specific test
cargo test test_capability_functor
```

### Development Workflow
```bash
# Watch for changes and rebuild
cargo watch -x build

# Run tests on change
cargo watch -x test

# Check formatting and linting
cargo fmt --all --check
cargo clippy -- -D warnings

# Generate documentation
cargo doc --workspace --open

# Check for unused dependencies
cargo udeps --workspace
```

### Docker Development
```bash
# Build development image
docker build -t enishi:dev -f docker/Dockerfile.dev .

# Run with hot reload
docker run -v $(pwd):/enishi -p 8080:8080 enishi:dev

# Run tests in container
docker run --rm enishi:dev cargo test
```

## Architecture Deep Dive

### Core Abstractions

#### Content Identifiers (Cid)
```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cid([u8; 32]); // BLAKE3-256 hash

impl Cid {
    pub fn hash(data: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        Self(hash.into())
    }
}
```

**Design Principles:**
- **Cryptographic integrity**: Content uniqueness and tamper detection
- **Fixed size**: Efficient indexing and memory layout
- **Zero-copy operations**: Direct memory mapping for performance

#### Capabilities (Cap)
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cap {
    pub base: u64,     // Memory base address
    pub len: u64,      // Length bound
    pub perms: u32,    // Permission flags
    pub proof: [u8; 16], // Cryptographic proof
}
```

**Security Properties:**
- **Spatial safety**: Bounds checking prevents buffer overflows
- **Permission monotonicity**: Capabilities can only lose permissions
- **Delegation control**: Explicit control over capability sharing

#### Ownership Types
```rust
// Exclusive ownership
pub struct OwnedCapCid<T> {
    cap_cid: CapCid,
    data: T,
}

// Immutable borrow
pub struct BorrowCapCid<'a, T> {
    cap_cid: &'a CapCid,
    data: &'a T,
}

// Mutable borrow
pub struct BorrowMutCapCid<'a, T> {
    cap_cid: &'a mut CapCid,
    data: &'a mut T,
}
```

**Rust Safety Guarantees:**
- **No data races**: Compile-time borrow checking
- **No use-after-free**: Ownership tracking
- **No iterator invalidation**: Borrow checker enforcement

### Component Interactions

#### Query Processing Pipeline
```
Client Request → HTTP Server → Capability Check → Query Optimization → Execution → Response
                      ↓              ↓                   ↓             ↓
                Authentication → Permission Validation → Plan Selection → Data Access
```

#### Storage Hierarchy
```
Application → Query Cache → Bloom Filters → Pack Index → Log-Structured Storage
                     ↓            ↓             ↓            ↓
              Hot Data → Warm Index → Cold Blob → Persistent Storage
```

## Coding Standards

### Rust Best Practices

#### Naming Conventions
```rust
// Types and traits
struct GraphDB { /* ... */ }
trait Monoid { /* ... */ }
enum QueryPlan { /* ... */ }

// Functions and methods
fn compute_path_sig(path: &[&str]) -> Cid
fn execute_secure_operation(&self, actor: &str, operation: &str) -> Result<String, String>

// Variables
let query_cache_size = 1000000;
let adaptive_optimization = true;
```

#### Error Handling
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnishiError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Capability check failed")]
    CapabilityDenied,

    #[error("Transaction conflict")]
    TransactionConflict,
}
```

#### Documentation
```rust
/// Computes a path signature for query optimization.
///
/// This function creates a deterministic hash of a query path
/// that can be used for caching and indexing.
///
/// # Arguments
/// * `path` - Query path segments
///
/// # Returns
/// Content identifier representing the path signature
///
/// # Examples
/// ```
/// let path = &["user", "posts", "comments"];
/// let sig = compute_path_sig(path);
/// assert_eq!(sig.as_bytes().len(), 32);
/// ```
pub fn compute_path_sig(path: &[&str]) -> Cid {
    // Implementation...
}
```

### Testing Guidelines

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_creation() {
        let data = b"test data";
        let cid = Cid::hash(data);
        assert_eq!(cid.as_bytes().len(), 32);
        assert_ne!(cid, Cid::hash(b"different data"));
    }

    #[test]
    fn test_capability_bounds() {
        let cap = Cap::new(100, 50, 0b11); // read + write
        assert!(cap.contains(120));
        assert!(cap.contains(149));
        assert!(!cap.contains(150));
        assert!(cap.has_perm(0b01)); // read
        assert!(cap.has_perm(0b10)); // write
    }
}
```

#### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_full_query_pipeline() {
        let temp_dir = tempdir().unwrap();

        // Setup components
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;
        let executor = SafeExecutor::new();

        // Execute test query
        let result = executor.execute_secure_operation(
            "alice",
            "traverse",
            &["user", "posts"],
            &["User", "Post"]
        ).await;

        assert!(result.is_ok());
    }
}
```

#### Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_capability_composition_associativity(
        base1 in 0u64..1000,
        len1 in 1u64..500,
        perms1 in 0u32..15,
        base2 in 0u64..1000,
        len2 in 1u64..500,
        perms2 in 0u32..15
    ) {
        let cap1 = Cap::new(base1, len1, perms1);
        let cap2 = Cap::new(base2, len2, perms2);

        // Test associativity: (a ∩ b) ∩ c = a ∩ (b ∩ c)
        let left = cap1.intersect(&cap2);
        let right = cap2.intersect(&cap1);

        // Intersection should be commutative
        assert_eq!(left.base, right.base);
        assert_eq!(left.len, right.len);
        assert_eq!(left.perms, right.perms);
    }
}
```

## Extending the System

### Adding New Query Types

#### 1. Define Query Structure
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpatialQuery {
    pub center: (f64, f64),
    pub radius: f64,
    pub max_results: usize,
}
```

#### 2. Implement Query Optimization
```rust
impl QueryPlan {
    pub fn optimize_spatial(query: &SpatialQuery) -> Self {
        // Spatial indexing strategy
        if query.radius < 100.0 {
            QueryPlan::IndexLookup(format!("spatial_{}_{}",
                query.center.0 as i32, query.center.1 as i32))
        } else {
            QueryPlan::PathFirst(vec!["location".to_string()])
        }
    }
}
```

#### 3. Add Execution Logic
```rust
impl GraphDB {
    pub async fn execute_spatial(&self, query: &SpatialQuery) -> Result<Vec<Node>, Error> {
        // Implement spatial search logic
        // Use R-tree or similar spatial index
        todo!("Implement spatial query execution")
    }
}
```

### Adding Storage Backends

#### 1. Define Storage Trait
```rust
#[async_trait]
pub trait StorageBackend {
    async fn put(&self, cid: &Cid, data: &[u8]) -> Result<(), StorageError>;
    async fn get(&self, cid: &Cid) -> Result<Vec<u8>, StorageError>;
    async fn exists(&self, cid: &Cid) -> Result<bool, StorageError>;
    async fn delete(&self, cid: &Cid) -> Result<(), StorageError>;
}
```

#### 2. Implement Backend
```rust
pub struct S3Storage {
    bucket: String,
    client: aws_sdk_s3::Client,
}

#[async_trait]
impl StorageBackend for S3Storage {
    async fn put(&self, cid: &Cid, data: &[u8]) -> Result<(), StorageError> {
        let key = format!("{:x}", cid);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(data.to_vec().into())
            .send()
            .await?;
        Ok(())
    }

    // ... other methods
}
```

#### 3. Integrate with System
```rust
impl GraphDB {
    pub async fn with_storage_backend(backend: Arc<dyn StorageBackend>) -> Self {
        // Use custom storage backend
        Self {
            storage: backend,
            // ... other fields
        }
    }
}
```

### Adding Authentication Providers

#### 1. Define Auth Trait
```rust
#[async_trait]
pub trait AuthProvider {
    async fn authenticate(&self, token: &str) -> Result<UserInfo, AuthError>;
    async fn authorize(&self, user: &UserInfo, capability: &Cap) -> Result<(), AuthError>;
    fn generate_capability(&self, user: &UserInfo, resource: &str) -> Cap;
}
```

#### 2. Implement OAuth Provider
```rust
pub struct OAuthProvider {
    client_id: String,
    client_secret: String,
    issuer_url: String,
}

#[async_trait]
impl AuthProvider for OAuthProvider {
    async fn authenticate(&self, token: &str) -> Result<UserInfo, AuthError> {
        // Validate JWT token
        // Extract user information
        // Return user info
        todo!("Implement OAuth authentication")
    }

    async fn authorize(&self, user: &UserInfo, capability: &Cap) -> Result<(), AuthError> {
        // Check user permissions against capability
        // Implement RBAC or ABAC logic
        todo!("Implement OAuth authorization")
    }

    fn generate_capability(&self, user: &UserInfo, resource: &str) -> Cap {
        // Generate appropriate capability based on user roles
        match user.role.as_str() {
            "admin" => Cap::new(0, u64::MAX, 0b1111), // full permissions
            "user" => Cap::new(1000, 5000, 0b0011),   // read/write limited
            _ => Cap::new(0, 0, 0b0000),              // no permissions
        }
    }
}
```

## Performance Profiling

### CPU Profiling
```bash
# Install profiling tools
cargo install flamegraph

# Run with profiling
sudo flamegraph -- cargo test --release

# Analyze results
# Look for hot paths in query execution, storage operations, etc.
```

### Memory Profiling
```bash
# Use heaptrack
sudo apt-get install -y heaptrack

# Profile memory usage
heaptrack ./target/release/enishi

# Analyze results
heaptrack_gui heaptrack.enishi.*
```

### Benchmarking
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_path_signature(c: &mut Criterion) {
    let paths = vec![
        vec!["user"],
        vec!["user", "posts"],
        vec!["user", "posts", "comments"],
    ];

    c.bench_function("path_signature", |b| {
        b.iter(|| {
            for path in &paths {
                black_box(compute_path_sig(path));
            }
        })
    });
}

criterion_group!(benches, bench_path_signature);
criterion_main!(benches);
```

## Debugging Techniques

### Logging Configuration
```rust
// Enable detailed logging
export RUST_LOG=debug,enishi=trace

// Structured logging
use tracing::{info, warn, error, debug};

info!(operation = "query", user = %user_id, latency_ms = latency);
```

### Debug Builds
```bash
# Build with debug symbols
cargo build

# Run with debugger
rust-gdb target/debug/enishi

# Set breakpoints
break enishi_graph::GraphDB::get_node
run
```

### Error Propagation
```rust
use anyhow::Context;

pub async fn complex_operation(&self, user_id: &str) -> Result<Node, anyhow::Error> {
    let user = self.authenticate(user_id)
        .context("Failed to authenticate user")?;

    let capability = self.authorize(&user, "read")
        .context("Failed to authorize user")?;

    let node = self.graph.get_node(&capability, "node123")
        .context("Failed to retrieve node")?;

    Ok(node)
}
```

## Contribution Guidelines

### Pull Request Process
1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Commit** changes: `git commit -m 'Add amazing feature'`
4. **Push** to branch: `git push origin feature/amazing-feature`
5. **Create** Pull Request with detailed description

### Code Review Checklist
- [ ] **Tests pass**: `cargo test --workspace`
- [ ] **Formatting correct**: `cargo fmt --all --check`
- [ ] **Linting clean**: `cargo clippy -- -D warnings`
- [ ] **Documentation updated**: `cargo doc --workspace`
- [ ] **Performance maintained**: Benchmarks show no regression
- [ ] **Security reviewed**: No new vulnerabilities introduced

### Commit Message Format
```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Testing
- `chore`: Maintenance

**Examples:**
```
feat(graph): add spatial query support

fix(security): prevent capability escalation in delegation

docs(api): update GraphQL schema documentation

test(performance): add benchmarks for query optimization
```

## Advanced Topics

### Formal Verification
```rust
// Use Kani for formal verification
#[cfg(kani)]
#[kani::proof]
fn verify_capability_monotonicity() {
    let cap1 = kani::any::<Cap>();
    let cap2 = kani::any::<Cap>();

    let intersect = cap1.intersect(&cap2);

    // Prove that intersection can only reduce permissions
    kani::assert(intersect.perms & cap1.perms == intersect.perms);
    kani::assert(intersect.perms & cap2.perms == intersect.perms);
}
```

### Category Theory Applications
```rust
// Functor implementation for capabilities
impl<T, U> Functor<U> for OwnedCapCid<T> {
    type Target = OwnedCapCid<U>;

    fn fmap<F>(self, f: F) -> Self::Target
    where
        F: FnOnce(T) -> U,
    {
        // Preserve capability while transforming data
        OwnedCapCid::new(f(self.data), self.cap_cid.cap, self.cap_cid.cid)
    }
}

// Monad implementation for secure composition
impl<T> Monad for OwnedCapCid<T> {
    fn bind<U, F>(self, f: F) -> OwnedCapCid<U>
    where
        F: FnOnce(T) -> Self,
    {
        let intermediate = f(self.data);
        // Compose capabilities: self.cap ∩ intermediate.cap
        let composed_cap = Cap {
            base: self.cap_cid.cap.base.max(intermediate.cap_cid.cap.base),
            len: self.cap_cid.cap.len.min(intermediate.cap_cid.cap.len),
            perms: self.cap_cid.cap.perms & intermediate.cap_cid.cap.perms,
            proof: intermediate.cap_cid.cap.proof,
        };
        OwnedCapCid::new(intermediate.data, composed_cap, intermediate.cap_cid.cid)
    }
}
```

### Research Directions

#### Ongoing Research
- **Homomorphic Encryption**: Privacy-preserving queries
- **Blockchain Integration**: Decentralized capability management
- **Quantum Resistance**: Post-quantum cryptographic capabilities
- **Machine Learning**: Query optimization using ML models

#### Future Enhancements
- **Multi-party Computation**: Secure collaborative queries
- **Differential Privacy**: Privacy-preserving analytics
- **Zero-Knowledge Proofs**: Verifiable query results
- **Federated Learning**: Distributed model training

---

This development guide provides comprehensive information for working with Own-CFA-Enishi. For questions or contributions, please see the main [README](../README.md) or create an issue in the repository.
