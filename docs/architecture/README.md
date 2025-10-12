# Own-CFA-Enishi Architecture Guide

## Overview

Own-CFA-Enishi implements a **categorical database** architecture that combines mathematical rigor with practical performance. This document provides a deep dive into the system architecture, design decisions, and technical implementation.

## Core Principles

### 1. Categorical Database Design

Enishi is built around the mathematical concept of categories, where:

- **Objects** are data entities (nodes, edges, properties)
- **Morphisms** are operations (queries, transformations, compositions)
- **Functors** preserve structure while mapping between categories

The fundamental functor is:

```
E: (Graph → ContentAddressable → Capability → Ownership)
```

### 2. Own+CFA Security Model

**Ownership + Capability + Functor** provides three layers of security:

1. **Ownership Types**: Rust's compile-time ownership prevents data races
2. **Capability-Based Access**: Fine-grained permissions with monotonicity
3. **Functor Preservation**: Security properties maintained under composition

### 3. Adaptive Optimization

The system learns and adapts through:

- **ε-greedy plan selection** for query optimization
- **Adaptive bloom filters** with memory redistribution
- **Snapshot management** for popular temporal queries

## System Architecture

### Component Hierarchy

```
┌─────────────────────────────────────────────────┐
│                Application Layer                │
│  ┌─────────────────────────────────────────────┐ │
│  │            HTTP API Server                 │ │
│  │  • REST endpoints                          │ │
│  │  • GraphQL interface                       │ │
│  │  • Health checks & metrics                 │ │
│  └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
                         │
┌─────────────────────────────────────────────────┐
│              Execution Layer                   │
│  ┌─────────────────────────────────────────────┐ │
│  │         Query Optimization                 │ │
│  │  • Plan switcher (ε-greedy)               │ │
│  │  • Meet-in-the-middle splitting            │ │
│  │  • Adaptive bloom filters                 │ │
│  └─────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────┐ │
│  │       Security & Ownership                 │ │
│  │  • Capability functor composition         │ │
│  │  • Transaction management                 │ │
│  │  • Audit logging                          │ │
│  └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
                         │
┌─────────────────────────────────────────────────┐
│               Storage Layer                    │
│  ┌─────────────────────────────────────────────┐ │
│  │         Graph Data Model                   │ │
│  │  • RID → CID mapping                      │ │
│  │  • Temporal versioning                    │ │
│  │  • Adjacency lists                        │ │
│  └─────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────┐ │
│  │       Content-Addressable Storage         │ │
│  │  • PackCAS with temperature bands         │ │
│  │  • Multi-level bloom filters              │ │
│  │  • Log-structured persistence             │ │
│  └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
                         │
┌─────────────────────────────────────────────────┐
│              Foundation Layer                  │
│  ┌─────────────────────────────────────────────┐ │
│  │         Core Data Types                    │ │
│  │  • Cid (BLAKE3 content identifiers)       │ │
│  │  • Cap (Cheri-style capabilities)         │ │
│  │  • QKey (query keys with signatures)      │ │
│  │  • Monoid (composable operations)         │ │
│  └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

## Detailed Component Analysis

### 1. Foundation Layer (enishi-core)

#### Content Identifiers (Cid)
```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cid([u8; 32]); // BLAKE3-256 hash
```

**Design Decisions**:
- **Cryptographic hashing**: Ensures content uniqueness and integrity
- **Fixed size**: Enables efficient indexing and comparison
- **Zero-copy operations**: Direct memory mapping for performance

#### Capabilities (Cap)
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cap {
    pub base: u64,    // Address base
    pub len: u64,     // Length
    pub perms: u32,   // Permissions
    pub proof: [u8; 16], // Cryptographic proof
}
```

**Security Properties**:
- **Spatial safety**: Base + length bounds checking
- **Permission monotonicity**: Capabilities can only lose permissions
- **Delegation control**: Explicit permission for capability sharing

#### Query Keys (QKey)
```rust
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct QKey {
    path_sig: Cid,      // Path signature for optimization
    class_sig: Cid,     // Class signature for filtering
    as_of: u64,         // Temporal point
    cap_region: (u64, u64), // Capability bounds
    type_part: u16,     // Type partitioning
}
```

**Optimization Features**:
- **Path signatures**: Enable efficient query caching
- **Class signatures**: Support type-based optimizations
- **Temporal indexing**: Efficient historical queries

### 2. Storage Layer (enishi-cas)

#### PackCAS Architecture

**Temperature-Based Organization**:
- **Hot (Small)**: Frequently accessed small objects
- **Warm (Index)**: Index structures and metadata
- **Cold (Blob)**: Large binary objects

**Multi-Level Bloom Filters**:
- **Global filter**: Fast rejection for non-existent content
- **Pack filters**: Per-pack filtering for large datasets
- **Shard filters**: Type×time based partitioning

#### Log-Structured Storage

**Advantages**:
- **Sequential writes**: Optimal for modern SSDs
- **Append-only**: Atomic operations, crash consistency
- **Compaction**: Automatic space reclamation

### 3. Graph Layer (enishi-graph)

#### RID → CID Mapping
```rust
pub struct RidMapping {
    rid: Rid,
    cid: Cid,
    valid_from: Timestamp,
    valid_to: Option<Timestamp>,
}
```

**Temporal Features**:
- **Point-in-time queries**: `as_of` timestamp support
- **Version history**: Complete audit trail of changes
- **Efficient indexing**: Temporal B-tree structures

#### Adjacency Representations

**Options Considered**:
- **Adjacency List**: Space-efficient, good for sparse graphs
- **Adjacency Matrix**: Fast lookups, memory-intensive
- **Compressed Sparse Row**: Balance of both approaches

**Chosen**: Adjacency List with temporal indexing for optimal space-time tradeoff.

### 4. Execution Layer (enishi-exec)

#### ε-Greedy Plan Selection

**Algorithm**:
```
if random() < ε:
    return random_plan()
else:
    return best_performing_plan()
```

**Learning Mechanism**:
- **Performance tracking**: Execution time and success rate per plan
- **Statistical confidence**: Multi-iteration validation
- **Adaptive ε**: Exploration rate decreases over time

#### Meet-in-the-Middle Optimization

**Query Splitting Strategy**:
1. **Complexity analysis**: Estimate cost of different split points
2. **Optimal partition**: Find minimum cost division
3. **Parallel execution**: Independent sub-query processing

### 5. Concurrency Layer (enishi-concur)

#### Ownership Types in Rust

**Borrowing Hierarchy**:
```
OwnedCapCid<T>          // Exclusive ownership
BorrowCapCid<'a, T>     // Shared immutable borrow
BorrowMutCapCid<'a, T>  // Exclusive mutable borrow
```

**Safety Guarantees**:
- **No data races**: Compile-time borrow checking
- **No use-after-free**: Ownership tracking
- **No iterator invalidation**: Borrow checker enforcement

#### Transaction Model

**ACID Properties**:
- **Atomicity**: All-or-nothing execution
- **Consistency**: Valid state transitions only
- **Isolation**: Concurrent transaction independence
- **Durability**: Committed changes persist

## Performance Characteristics

### Latency Distribution

Based on validation results:

```
Percentile | Latency | Notes
-----------|---------|--------
P50       | 8.5ms  | Typical query performance
P95       | 9.6ms  | 95% of queries under 10ms
P99       | 12.0ms | 99% of queries under 12ms
P99.5     | ~15ms  | Tail latency control
```

### Scalability Model

**Horizontal Scaling**:
- **Shared-nothing architecture**: Independent node scaling
- **Consistent hashing**: Load distribution across nodes
- **Replication factor**: Configurable redundancy

**Vertical Scaling**:
- **CPU scaling**: Linear with core count (adaptive optimization)
- **Memory scaling**: Efficient caching and bloom filters
- **Storage scaling**: Log-structured append optimization

## Security Architecture

### Threat Model

**Assumptions**:
- **Network attacks**: MITM, spoofing, replay
- **Application attacks**: Injection, privilege escalation
- **Data attacks**: Corruption, unauthorized access
- **Operational attacks**: DoS, resource exhaustion

**Defenses**:
- **TLS encryption**: Network communication protection
- **Capability system**: Fine-grained access control
- **Audit logging**: Complete operation traceability
- **Rate limiting**: DoS attack prevention

### Zero-Trust Design

**Principles**:
1. **Never trust, always verify**: Every operation validated
2. **Least privilege**: Minimum required permissions only
3. **Complete mediation**: All accesses checked
4. **Fail-safe defaults**: Secure by default configuration

## Operational Architecture

### Health Monitoring

**Health Checks**:
- **Liveness**: Process responsiveness
- **Readiness**: Service availability
- **Dependency checks**: Storage and network connectivity

**Metrics Collection**:
- **Performance**: Query latency, throughput, cache hit rates
- **Resources**: CPU, memory, disk, network usage
- **Business**: Query patterns, data growth, user activity

### Configuration Management

**Hierarchical Configuration**:
1. **Compile-time defaults**: Built-in safe defaults
2. **Configuration files**: TOML/YAML overrides
3. **Environment variables**: Runtime customization
4. **Dynamic reconfiguration**: Hot config updates

## Evolution & Extensibility

### Plugin Architecture

**Extension Points**:
- **Query optimizers**: Custom optimization strategies
- **Storage backends**: Alternative persistence layers
- **Authentication providers**: Custom security modules
- **Analytics engines**: Graph algorithm libraries

### API Evolution

**Compatibility Strategy**:
- **Semantic versioning**: Breaking changes in major versions
- **Deprecation warnings**: Graceful migration path
- **Feature flags**: Experimental features behind flags
- **Backwards compatibility**: Best-effort compatibility maintenance

## Conclusion

Own-CFA-Enishi represents a fundamental rethinking of database architecture, grounded in category theory and implemented with Rust's safety guarantees. The system's layered design enables:

- **Mathematical correctness** through categorical foundations
- **Practical performance** through adaptive optimization
- **Ironclad security** through ownership and capabilities
- **Operational excellence** through comprehensive monitoring

This architecture serves as a blueprint for the next generation of secure, performant, and mathematically rigorous database systems.
