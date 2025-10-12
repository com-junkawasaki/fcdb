//! # Enishi CAS (Content Addressable Storage)
//!
//! PackCAS implementation with cidx indexing and bloom filters.
//!
//! Merkle DAG: enishi_cas -> pack_cas, cidx, bloom_filters, wal, gc

use fcdb_core::{Cid, varint};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use memmap2::Mmap;
use bloom::{BloomFilter, ASMS};
use crc32fast::Hasher as Crc32;
use tracing::{info, warn, error};

/// Pack size configuration (256-512MiB)
const PACK_SIZE_TARGET: u64 = 256 * 1024 * 1024; // 256 MiB
const PACK_SIZE_MAX: u64 = 512 * 1024 * 1024;    // 512 MiB

/// Temperature bands for pack organization
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackBand {
    Small,  // Small objects (< 4KB)
    Index,  // Index structures
    Blob,   // Large blobs (>= 4KB)
}

/// Pack metadata
#[derive(Clone, Debug)]
pub struct PackMeta {
    pub id: u32,
    pub band: PackBand,
    pub size: u64,
    pub object_count: u64,
    pub created_at: u64,
}

/// Content Index Record (64B fixed length)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CidxRec {
    pub cid: [u8; 32],      // CID
    pub pack_id: u32,       // Pack ID
    pub offset: u64,        // Offset in pack
    pub len: u32,           // Object length
    pub kind: u8,           // Object kind/type
    pub flags: u8,          // Flags
    pub crc: u32,           // CRC32 checksum
    pub _pad: [u8; 10],     // Padding to 64B
}

impl CidxRec {
    /// Create a new cidx record
    pub fn new(cid: Cid, pack_id: u32, offset: u64, len: u32, kind: u8, flags: u8) -> Self {
        let mut crc = Crc32::new();
        crc.update(cid.as_bytes());
        crc.update(&pack_id.to_le_bytes());
        crc.update(&offset.to_le_bytes());
        crc.update(&len.to_le_bytes());
        crc.update(&[kind, flags]);

        Self {
            cid: *cid.as_bytes(),
            pack_id,
            offset,
            len,
            kind,
            flags,
            crc: crc.finalize(),
            _pad: [0; 10],
        }
    }

    /// Verify CRC
    pub fn verify_crc(&self) -> bool {
        let mut crc = Crc32::new();
        crc.update(&self.cid);
        crc.update(&self.pack_id.to_le_bytes());
        crc.update(&self.offset.to_le_bytes());
        crc.update(&self.len.to_le_bytes());
        crc.update(&[self.kind, self.flags]);
        crc.finalize() == self.crc
    }
}

/// Bloom filter configuration for different levels
#[derive(Clone, Debug)]
pub struct BloomConfig {
    pub expected_items: usize,
    pub fp_rate: f64,
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            expected_items: 1_000_000,
            fp_rate: 1e-6, // Very low false positive rate
        }
    }
}

/// Multi-level bloom filter system
pub struct BloomFilters {
    global: BloomFilter,
    pack_filters: HashMap<u32, BloomFilter>,
    shard_filters: HashMap<(u16, u64), BloomFilter>, // (type, time_bucket) -> filter
}

impl BloomFilters {
    pub fn new() -> Self {
        Self {
            global: BloomFilter::with_rate(BloomConfig::default().fp_rate as f32, BloomConfig::default().expected_items as u32),
            pack_filters: HashMap::new(),
            shard_filters: HashMap::new(),
        }
    }

    pub fn insert(&mut self, cid: &Cid, pack_id: u32, type_part: u16, time_bucket: u64) {
        // Global filter
        self.global.insert(cid.as_bytes());

        // Pack filter
        self.pack_filters
            .entry(pack_id)
            .or_insert_with(|| BloomFilter::with_rate(1e-7, 100_000))
            .insert(cid.as_bytes());

        // Shard filter
        self.shard_filters
            .entry((type_part, time_bucket))
            .or_insert_with(|| BloomFilter::with_rate(1e-8, 10_000))
            .insert(cid.as_bytes());
    }

    pub fn contains(&self, cid: &Cid, pack_id: Option<u32>, shard: Option<(u16, u64)>) -> bool {
        // Check global first
        if !self.global.contains(cid.as_bytes()) {
            return false;
        }

        // Check pack filter if specified
        if let Some(pack_id) = pack_id {
            if let Some(filter) = self.pack_filters.get(&pack_id) {
                if !filter.contains(cid.as_bytes()) {
                    return false;
                }
            }
        }

        // Check shard filter if specified
        if let Some((type_part, time_bucket)) = shard {
            if let Some(filter) = self.shard_filters.get(&(type_part, time_bucket)) {
                if !filter.contains(cid.as_bytes()) {
                    return false;
                }
            }
        }

        true
    }
}

/// PackCAS - Content Addressable Storage with pack files
pub struct PackCAS {
    base_path: PathBuf,
    current_pack: Option<PackWriter>,
    packs: HashMap<u32, PackMeta>,
    cidx_file: File,
    bloom_filters: BloomFilters,
    next_pack_id: u32,
}

impl PackCAS {
    /// Open or create a PackCAS instance
    pub async fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let base_path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_path)?;

        let cidx_path = base_path.join("cidx.dat");
        let cidx_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(cidx_path)?;

        let mut cas = Self {
            base_path,
            current_pack: None,
            packs: HashMap::new(),
            cidx_file,
            bloom_filters: BloomFilters::new(),
            next_pack_id: 0,
        };

        cas.load_existing_packs().await?;
        cas.load_cidx().await?;

        Ok(cas)
    }

    /// Load existing pack metadata
    async fn load_existing_packs(&mut self) -> io::Result<()> {
        let mut pack_id = 0;
        loop {
            let pack_path = self.base_path.join(format!("pack_{:08}.dat", pack_id));
            if !pack_path.exists() {
                break;
            }

            // Load pack metadata (simplified - in real impl, read from manifest)
            let meta = PackMeta {
                id: pack_id,
                band: PackBand::Blob, // Default
                size: std::fs::metadata(&pack_path)?.len(),
                object_count: 0, // Would be loaded from manifest
                created_at: 0,
            };

            self.packs.insert(pack_id, meta);
            pack_id += 1;
        }
        self.next_pack_id = pack_id;

        Ok(())
    }

    /// Load content index
    async fn load_cidx(&mut self) -> io::Result<()> {
        let file_size = self.cidx_file.metadata()?.len();
        let record_count = file_size / std::mem::size_of::<CidxRec>() as u64;

        // Memory map the cidx file for fast access
        let mmap = unsafe { Mmap::map(&self.cidx_file)? };
        let records = unsafe {
            std::slice::from_raw_parts(
                mmap.as_ptr() as *const CidxRec,
                record_count as usize,
            )
        };

        // Rebuild bloom filters from cidx
        for record in records {
            if !record.verify_crc() {
                warn!("Cidx record CRC mismatch, skipping");
                continue;
            }

            let cid = Cid::from_bytes(record.cid);
            let pack_id = record.pack_id;
            let type_part = (record.kind as u16) << 8; // Simplified type extraction
            let time_bucket = 0; // Would be derived from metadata

            self.bloom_filters.insert(&cid, pack_id, type_part, time_bucket);
        }

        info!("Loaded {} cidx records", record_count);
        Ok(())
    }

    /// Store data and return CID
    pub async fn put(&mut self, data: &[u8], kind: u8, band: PackBand) -> io::Result<Cid> {
        let cid = Cid::hash(data);

        // Check if already exists
        if self.bloom_filters.contains(&cid, None, None) {
            // Could do a full lookup here, but for now assume it's there
            return Ok(cid);
        }

        // Ensure we have a pack writer
        self.ensure_pack_writer(band).await?;

        let (offset, pack_id) = if let Some(writer) = &mut self.current_pack {
            let offset = writer.current_offset;
            let pack_id = writer.pack_id;
            writer.file.write_all(data)?;
            writer.current_offset += data.len() as u64;
            (offset, pack_id)
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "No current pack writer"));
        };

        // Add to cidx
        let record = CidxRec::new(cid, pack_id, offset, data.len() as u32, kind, 0);
        self.append_cidx_record(&record).await?;

        // Update bloom filters
        let type_part = (kind as u16) << 8;
        let time_bucket = 0; // Would be current time bucket
        self.bloom_filters.insert(&cid, pack_id, type_part, time_bucket);

        // Check if pack is full
        if offset + data.len() as u64 >= PACK_SIZE_TARGET {
            self.close_current_pack().await?;
        }

        Ok(cid)
    }

    /// Retrieve data by CID
    pub async fn get(&self, cid: &Cid) -> io::Result<Vec<u8>> {
        // Use bloom filters to narrow search
        if !self.bloom_filters.contains(cid, None, None) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "CID not found"));
        }

        // For now, do a linear search through packs
        // In real implementation, would use cidx for direct lookup
        for (pack_id, _meta) in &self.packs {
            let pack_path = self.base_path.join(format!("pack_{:08}.dat", pack_id));
            let mut file = File::open(pack_path)?;

            // This is highly inefficient - real impl would use cidx for direct access
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;

            // Check if this pack contains our data (simplified)
            if data.len() > 32 && &data[..32] == cid.as_bytes() {
                return Ok(data[32..].to_vec()); // Remove CID prefix if stored
            }
        }

        Err(io::Error::new(io::ErrorKind::NotFound, "CID not found"))
    }

    /// Ensure we have an active pack writer
    async fn ensure_pack_writer(&mut self, band: PackBand) -> io::Result<()> {
        if self.current_pack.is_none() {
            let pack_id = self.next_pack_id;
            self.next_pack_id += 1;

            let pack_path = self.base_path.join(format!("pack_{:08}.dat", pack_id));
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(pack_path)?;

            self.current_pack = Some(PackWriter {
                pack_id,
                file,
                current_offset: 0,
                band,
            });

            // Record pack metadata
            self.packs.insert(pack_id, PackMeta {
                id: pack_id,
                band,
                size: 0,
                object_count: 0,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
        Ok(())
    }

    /// Close current pack
    async fn close_current_pack(&mut self) -> io::Result<()> {
        if let Some(mut writer) = self.current_pack.take() {
            writer.file.flush()?;
            info!("Closed pack {}", writer.pack_id);
        }
        Ok(())
    }

    /// Append record to cidx file
    async fn append_cidx_record(&mut self, record: &CidxRec) -> io::Result<()> {
        self.cidx_file.seek(SeekFrom::End(0))?;
        let bytes = unsafe {
            std::slice::from_raw_parts(
                record as *const CidxRec as *const u8,
                std::mem::size_of::<CidxRec>(),
            )
        };
        self.cidx_file.write_all(bytes)?;
        self.cidx_file.flush()?;
        Ok(())
    }
}

/// Pack writer for building pack files
struct PackWriter {
    pack_id: u32,
    file: File,
    current_offset: u64,
    band: PackBand,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_pack_cas_basic() {
        let temp_dir = tempdir().unwrap();
        let mut cas = PackCAS::open(temp_dir.path()).await.unwrap();

        let data = b"Hello, PackCAS!";
        let cid = cas.put(data, 1, PackBand::Small).await.unwrap();

        let retrieved = cas.get(&cid).await.unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_cidx_record() {
        let cid = Cid::hash(b"test data");
        let record = CidxRec::new(cid, 42, 1024, 100, 1, 0);

        assert!(record.verify_crc());
        assert_eq!(record.pack_id, 42);
        assert_eq!(record.offset, 1024);
        assert_eq!(record.len, 100);
    }

    #[test]
    fn test_bloom_filters() {
        let mut filters = BloomFilters::new();
        let cid = Cid::hash(b"test");

        filters.insert(&cid, 1, 100, 1234567890);

        assert!(filters.contains(&cid, None, None));
        assert!(filters.contains(&cid, Some(1), None));
        assert!(filters.contains(&cid, Some(1), Some((100, 1234567890))));

        let other_cid = Cid::hash(b"other");
        assert!(!filters.contains(&other_cid, None, None));
    }
}
