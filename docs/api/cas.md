# FCDB CAS API

## Overview

`fcdb-cas` implements Content-Addressable Storage (CAS) with advanced features like bloom filters, compaction, and concurrent access.

## PackCAS

Main CAS implementation using pack files with indexes:

```rust
use fcdb_cas::PackCAS;

// Create new CAS instance
let cas = PackCAS::new("./data").await?;

// Store data
let cid = cas.put(b"Hello FCDB!".to_vec(), 0).await?;
println!("Stored with CID: {:?}", cid);

// Retrieve data
let data = cas.get(&cid).await?;
assert_eq!(data, b"Hello FCDB!".to_vec());

// Check existence
let exists = cas.contains(&cid).await?;
assert!(exists);
```

## Advanced Operations

### Streaming Operations

Large data handling with streaming:

```rust
use tokio::fs::File;
use std::io::Read;

// Stream large file to CAS
let mut file = File::open("large_file.dat").await?;
let cid = cas.put_stream(&mut file, 0).await?;

// Stream data out of CAS
let mut output = Vec::new();
cas.get_stream(&cid, &mut output).await?;
```

### Batch Operations

Efficient batch processing:

```rust
// Store multiple items
let items = vec![
    (b"data1".to_vec(), 0u8),
    (b"data2".to_vec(), 1u8),
    (b"data3".to_vec(), 0u8),
];

let cids = cas.put_batch(items).await?;
println!("Stored {} items", cids.len());
```

## Bloom Filters

Automatic bloom filter management for fast existence checks:

```rust
// Bloom filters are automatically maintained
// Fast existence check without disk access
let exists = cas.contains(&cid).await?;
if exists {
    // High probability item exists, safe to retrieve
    let data = cas.get(&cid).await?;
}
```

## Compaction

Automatic and manual compaction for space optimization:

```rust
// Check compaction status
let stats = cas.stats().await?;
println!("Fragmentation: {:.2}%", stats.fragmentation_ratio * 100.0);

// Manual compaction
cas.compact().await?;
println!("Compaction completed");
```

## Configuration

CAS configuration options:

```rust
use fcdb_cas::{PackCAS, PackConfig};

let config = PackConfig {
    max_pack_size: 512 * 1024 * 1024, // 512MB
    bloom_fp_rate: 0.01,              // 1% false positive rate
    compaction_threshold: 0.7,        // Compact when 70% dead data
    ..Default::default()
};

let cas = PackCAS::with_config("./data", config).await?;
```

## Performance Tuning

### Memory Usage

```rust
// Monitor memory usage
let mem_usage = cas.memory_usage().await?;
println!("Index memory: {} MB", mem_usage.index_mb);
println!("Bloom filter memory: {} MB", mem_usage.bloom_mb);
```

### I/O Optimization

```rust
// Preload frequently accessed data
cas.prefetch(&[cid1, cid2, cid3]).await?;

// Optimize for read-heavy workloads
cas.optimize_for_reads().await?;
```

## Error Handling

CAS operations can fail with various error types:

```rust
use fcdb_cas::CasError;

match cas.get(&cid).await {
    Ok(data) => println!("Retrieved {} bytes", data.len()),
    Err(CasError::NotFound) => println!("Item not found"),
    Err(CasError::Io(e)) => println!("I/O error: {}", e),
    Err(CasError::Corruption) => println!("Data corruption detected"),
    Err(e) => println!("Other error: {}", e),
}
```

## Concurrent Access

Thread-safe concurrent operations:

```rust
use std::sync::Arc;

// Share CAS instance across threads
let cas = Arc::new(cas);

// Spawn multiple tasks
let handles: Vec<_> = (0..10).map(|i| {
    let cas = Arc::clone(&cas);
    tokio::spawn(async move {
        let data = format!("data from task {}", i).into_bytes();
        let cid = cas.put(data, 0).await?;
        Ok::<_, fcdb_cas::CasError>(cid)
    })
}).collect();

// Wait for all operations
for handle in handles {
    let cid = handle.await??;
    println!("Task completed with CID: {:?}", cid);
}
```

## Migration and Backup

### Data Migration

```rust
// Migrate data between CAS instances
let source_cas = PackCAS::new("./old_data").await?;
let target_cas = PackCAS::new("./new_data").await?;

source_cas.migrate_to(&target_cas).await?;
```

### Backup and Restore

```rust
// Create backup
cas.create_backup("./backup.tar.gz").await?;

// Restore from backup
PackCAS::restore_from_backup("./backup.tar.gz", "./data").await?;
```

## Monitoring

Built-in metrics and monitoring:

```rust
use std::time::Duration;

// Periodic stats reporting
tokio::spawn(async move {
    loop {
        let stats = cas.stats().await?;
        println!("CAS Stats:");
        println!("  Total items: {}", stats.total_items);
        println!("  Total size: {} MB", stats.total_size_mb);
        println!("  Read latency: {:.2}ms", stats.avg_read_latency_ms);
        println!("  Write throughput: {:.2} MB/s", stats.write_throughput_mbs);

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
});
```

## Best Practices

### Performance

1. **Batch Operations**: Use `put_batch()` for multiple items
2. **Streaming**: Use streaming APIs for large data
3. **Prefetching**: Preload frequently accessed data
4. **Compaction**: Monitor and schedule regular compaction

### Reliability

1. **Regular Backups**: Schedule automated backups
2. **Integrity Checks**: Run periodic integrity verification
3. **Monitoring**: Monitor error rates and performance metrics
4. **Resource Limits**: Set appropriate memory and disk limits

### Security

1. **Access Control**: Use file system permissions appropriately
2. **Encryption**: Consider encrypting data at rest
3. **Audit Logging**: Enable audit logging for sensitive operations

## Troubleshooting

### Common Issues

#### High Memory Usage
```
Cause: Large bloom filters or index caching
Solution: Adjust bloom filter parameters or reduce cache size
```

#### Slow Compaction
```
Cause: Large pack files or high fragmentation
Solution: Schedule compaction during low-traffic periods
```

#### I/O Bottlenecks
```
Cause: Disk I/O limitations
Solution: Use SSD storage or RAID configuration
```

## API Reference

See the generated Rustdoc documentation for complete API details:

```bash
cargo doc --open --package fcdb-cas
```
