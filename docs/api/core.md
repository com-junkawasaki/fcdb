# FCDB Core API

## Overview

`fcdb-core` provides the fundamental data structures and cryptographic primitives used throughout FCDB.

## Key Types

### Cid (Content Identifier)

Cryptographic content identifier using BLAKE3 hash:

```rust
use fcdb_core::Cid;

// Create CID from data
let data = b"Hello World";
let cid = Cid::hash(data);

// Convert to/from bytes
let bytes: [u8; 32] = cid.as_bytes().clone();
let cid2 = Cid::from_bytes(bytes);

// Display formatting
println!("{}", cid); // Hex-encoded string
```

### Cap (Capability)

CHERI-style memory capability for fine-grained access control:

```rust
use fcdb_core::Cap;

// Create capability
let cap = Cap::new(0x1000, 0x1000, 0x07); // base, length, permissions

// Check permissions
assert!(cap.has_perm(0x01)); // read permission
assert!(cap.has_perm(0x02)); // write permission

// Check address bounds
assert!(cap.contains(0x1500)); // within bounds
assert!(!cap.contains(0x3000)); // outside bounds
```

### QKey (Query Key)

Caching and indexing key for query optimization:

```rust
use fcdb_core::{QKey, Cid};

// Create query key
let path_sig = Cid::hash(b"/users/123");
let class_sig = Cid::hash(b"User");
let qkey = QKey {
    path_sig,
    class_sig,
    as_of: 1640995200, // timestamp
    cap_region: (0x1000, 0x2000), // capability bounds
    type_part: 1, // type identifier
};
```

## Cryptographic Operations

### Content Hashing

```rust
use fcdb_core::Cid;

// Hash any data
let data = b"some data";
let cid = Cid::hash(data);

// Hash with specific algorithm (future extension)
let cid_blake3 = Cid::hash_blake3(data);
```

### Varint Encoding

Efficient variable-length integer encoding:

```rust
use fcdb_core::varint;

// Encode u64 to bytes
let mut buf = Vec::new();
varint::encode_u64(12345, &mut buf);

// Decode from bytes
let mut reader = &buf[..];
let value = varint::decode_u64(&mut reader)?;
assert_eq!(value, 12345);
```

## Error Handling

FCDB uses standard Rust error handling with `Result<T, E>` and `thiserror` for custom error types.

```rust
use fcdb_core::*;

// Most operations return Result
let result: Result<Cid, std::io::Error> = some_operation();
```

## Performance Characteristics

- **Hashing**: BLAKE3 provides high-speed cryptographic hashing (~1GB/s)
- **Varint**: Efficient encoding for variable-length integers
- **Capabilities**: Constant-time bounds checking
- **Memory**: Minimal memory footprint with zero-copy operations where possible

## Thread Safety

All types in `fcdb-core` are `Send + Sync` and can be safely shared between threads:

```rust
use std::sync::Arc;
use fcdb_core::Cid;

let cid = Arc::new(Cid::hash(b"shared data"));
// Safe to share across threads
```

## Migration Guide

### From Enishi

If migrating from the Enishi codebase:

- Replace `enishi_core::*` imports with `fcdb_core::*`
- Update capability creation syntax
- CID handling remains compatible
- Varint encoding API unchanged

## Examples

See [`examples/basic_usage.rs`](../../examples/basic_usage.rs) for complete usage examples.
