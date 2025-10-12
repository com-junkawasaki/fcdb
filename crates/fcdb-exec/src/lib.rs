//! # Enishi Execution Engine
//!
//! Phase C: P10+ Adaptation - Advanced query optimization and adaptive systems
//!
//! Merkle DAG: enishi_exec -> adaptive_bloom, plan_switcher, meet_in_middle

use fcdb_core::{Cid, QKey, compute_path_sig, compute_class_sig};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use std::time::{Duration, Instant};
use bloom::{BloomFilter, ASMS};
use rand::prelude::*;
use statrs::distribution::{Normal, ContinuousCDF};

// ===== PHASE C: Adaptive 三段Bloom Filters =====

/// Adaptive bloom filter configuration
#[derive(Clone, Debug)]
pub struct AdaptiveBloomConfig {
    pub target_fp_rate: f64,
    pub max_memory_mb: usize,
    pub adaptation_interval_secs: u64,
}

impl Default for AdaptiveBloomConfig {
    fn default() -> Self {
        Self {
            target_fp_rate: 1e-6, // Very low false positive rate
            max_memory_mb: 100,   // 100MB total
            adaptation_interval_secs: 300, // 5 minutes
        }
    }
}

/// 三段Bloomフィルタシステム (Global/Pack/Shard)
pub struct AdaptiveBloomSystem {
    global: BloomFilter,
    pack_filters: HashMap<u32, BloomFilter>,
    shard_filters: HashMap<(u16, u64), BloomFilter>, // (type, time_bucket)

    // Statistics for adaptation
    global_fps: Vec<f64>,
    pack_fps: HashMap<u32, Vec<f64>>,
    shard_fps: HashMap<(u16, u64), Vec<f64>>,

    config: AdaptiveBloomConfig,
    last_adaptation: Instant,
}

impl AdaptiveBloomSystem {
    pub fn new(config: AdaptiveBloomConfig) -> Self {
        let initial_capacity = 1_000_000;
        Self {
            global: BloomFilter::with_rate(config.target_fp_rate as f32, initial_capacity as u32),
            pack_filters: HashMap::new(),
            shard_filters: HashMap::new(),
            global_fps: Vec::new(),
            pack_fps: HashMap::new(),
            shard_fps: HashMap::new(),
            config,
            last_adaptation: Instant::now(),
        }
    }

    /// Insert with type and time bucket for sharding
    pub fn insert(&mut self, cid: &Cid, pack_id: u32, type_part: u16, time_bucket: u64) {
        // Global filter
        self.global.insert(cid.as_bytes());

        // Pack filter
        self.pack_filters
            .entry(pack_id)
            .or_insert_with(|| BloomFilter::with_rate(1e-7, 100_000))
            .insert(cid.as_bytes());

        // Shard filter - adaptive creation
        let shard_key = (type_part, time_bucket);
        self.shard_filters
            .entry(shard_key)
            .or_insert_with(|| BloomFilter::with_rate(1e-8, 10_000))
            .insert(cid.as_bytes());
    }

    /// Query with hierarchical filtering
    pub fn contains(&self, cid: &Cid, pack_id: Option<u32>, shard: Option<(u16, u64)>) -> bool {
        // Check global first (fast rejection)
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

    /// Record false positive for adaptation
    pub fn record_fp(&mut self, pack_id: Option<u32>, shard: Option<(u16, u64)>) {
        self.global_fps.push(1.0);

        if let Some(pack_id) = pack_id {
            self.pack_fps.entry(pack_id).or_insert_with(Vec::new).push(1.0);
        }

        if let Some(shard_key) = shard {
            self.shard_fps.entry(shard_key).or_insert_with(Vec::new).push(1.0);
        }

        // Trigger adaptation if interval passed
        if self.last_adaptation.elapsed() > Duration::from_secs(self.config.adaptation_interval_secs) {
            self.adapt_filters();
            self.last_adaptation = Instant::now();
        }
    }

    /// Adaptive filter reconfiguration
    fn adapt_filters(&mut self) {
        // Calculate current FP rates
        let global_fp_rate = self.global_fps.iter().sum::<f64>() / self.global_fps.len().max(1) as f64;

        // Redistribute memory based on FP rates and access patterns
        let total_memory = self.config.max_memory_mb * 1024 * 1024; // bytes

        // Global filter gets 40% baseline
        let global_memory = (total_memory as f64 * 0.4) as usize;

        // Pack filters get 40% distributed by access frequency
        let pack_memory = (total_memory as f64 * 0.4) as usize;

        // Shard filters get 20% distributed by FP rate
        let shard_memory = (total_memory as f64 * 0.2) as usize;

        // Reconfigure filters with new sizes
        self.reconfigure_filters(global_memory, pack_memory, shard_memory);

        // Reset statistics
        self.global_fps.clear();
        self.pack_fps.clear();
        self.shard_fps.clear();
    }

    fn reconfigure_filters(&mut self, global_mem: usize, pack_mem: usize, shard_mem: usize) {
        // Estimate optimal sizes based on memory and target FP rates
        // This is a simplified version - real implementation would use more sophisticated sizing
        let global_capacity = (global_mem / 16).min(10_000_000); // Rough estimate
        self.global = BloomFilter::with_rate(self.config.target_fp_rate as f32, global_capacity as u32);

        // Rebuild pack and shard filters with new sizing
        // (In practice, this would migrate existing data)
        self.pack_filters.clear();
        self.shard_filters.clear();
    }
}

// ===== PHASE C: Plan Switcher with ε-greedy =====

/// Query execution plan
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum QueryPlan {
    PathFirst(Vec<String>),        // Follow path first
    TypeFirst(Vec<String>),        // Filter by types first
    MeetInMiddle(String),          // Split query at midpoint
    IndexLookup(String),           // Direct index lookup
}

/// Plan performance statistics
#[derive(Clone, Debug)]
pub struct PlanStats {
    pub plan: QueryPlan,
    pub execution_time_ms: f64,
    pub result_count: usize,
    pub success: bool,
    pub timestamp: u64,
}

/// ε-greedy plan switcher
pub struct PlanSwitcher {
    plan_stats: HashMap<String, Vec<PlanStats>>, // plan_key -> stats
    epsilon: f64, // Exploration rate
    plan_timeout_ms: u64,
}

impl PlanSwitcher {
    pub fn new() -> Self {
        Self {
            plan_stats: HashMap::new(),
            epsilon: 0.1, // 10% exploration
            plan_timeout_ms: 1000, // 1 second timeout
        }
    }

    /// Choose best plan with ε-greedy exploration
    pub fn select_plan(&self, query_key: &str, available_plans: &[QueryPlan]) -> QueryPlan {
        if available_plans.is_empty() {
            return QueryPlan::PathFirst(vec![]);
        }

        // ε-greedy: explore or exploit
        if rand::random::<f64>() < self.epsilon {
            // Explore: random plan
            available_plans.choose(&mut rand::thread_rng()).unwrap().clone()
        } else {
            // Exploit: best performing plan
            self.select_best_plan(query_key, available_plans)
        }
    }

    fn select_best_plan(&self, query_key: &str, available_plans: &[QueryPlan]) -> QueryPlan {
        if let Some(stats) = self.plan_stats.get(query_key) {
            // Find plan with lowest average execution time
            let mut best_plan = &available_plans[0];
            let mut best_time = f64::INFINITY;

            for plan in available_plans {
                let plan_key = self.plan_key(plan);
                let avg_time = self.average_time_for_plan(&plan_key, stats);

                if avg_time < best_time {
                    best_time = avg_time;
                    best_plan = plan;
                }
            }

            best_plan.clone()
        } else {
            // No stats available, use first plan
            available_plans[0].clone()
        }
    }

    /// Record plan execution result
    pub fn record_result(&mut self, query_key: &str, plan: &QueryPlan, execution_time_ms: f64, result_count: usize, success: bool) {
        let stats = PlanStats {
            plan: plan.clone(),
            execution_time_ms,
            result_count,
            success,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.plan_stats.entry(query_key.to_string())
            .or_insert_with(Vec::new)
            .push(stats);

        // Keep only recent stats (last 100 per plan type)
        if let Some(stats_vec) = self.plan_stats.get_mut(query_key) {
            if stats_vec.len() > 100 {
                stats_vec.remove(0); // Remove oldest
            }
        }
    }

    fn plan_key(&self, plan: &QueryPlan) -> String {
        match plan {
            QueryPlan::PathFirst(path) => format!("path_first_{}", path.join("_")),
            QueryPlan::TypeFirst(types) => format!("type_first_{}", types.join("_")),
            QueryPlan::MeetInMiddle(split) => format!("meet_middle_{}", split),
            QueryPlan::IndexLookup(index) => format!("index_lookup_{}", index),
        }
    }

    fn average_time_for_plan(&self, plan_key: &str, all_stats: &[PlanStats]) -> f64 {
        let plan_stats: Vec<_> = all_stats.iter()
            .filter(|s| self.plan_key(&s.plan) == plan_key && s.success)
            .collect();

        if plan_stats.is_empty() {
            return f64::INFINITY;
        }

        plan_stats.iter().map(|s| s.execution_time_ms).sum::<f64>() / plan_stats.len() as f64
    }
}

// ===== PHASE C: Meet-in-the-middle Optimization =====

/// Meet-in-the-middle query splitter
pub struct MeetInMiddle {
    max_split_depth: usize,
    cost_estimator: CostEstimator,
}

impl MeetInMiddle {
    pub fn new() -> Self {
        Self {
            max_split_depth: 5,
            cost_estimator: CostEstimator::new(),
        }
    }

    /// Split complex query into two halves meeting in middle
    pub fn split_query(&self, query_path: &[&str], query_types: &[&str]) -> Option<QuerySplit> {
        if query_path.len() < 3 {
            return None; // Too simple for splitting
        }

        // Find optimal split point
        let path_len = query_path.len();
        let mut best_split = 0;
        let mut best_cost = f64::INFINITY;

        for split_point in 1..path_len {
            let left_cost = self.cost_estimator.estimate_cost(&query_path[0..split_point], &[]);
            let right_cost = self.cost_estimator.estimate_cost(&query_path[split_point..], &[]);
            let total_cost = left_cost + right_cost + 1.0; // Join cost

            if total_cost < best_cost {
                best_cost = total_cost;
                best_split = split_point;
            }
        }

        Some(QuerySplit {
            left_path: query_path[0..best_split].iter().map(|s| s.to_string()).collect(),
            right_path: query_path[best_split..].iter().map(|s| s.to_string()).collect(),
            join_key: query_path[best_split - 1].to_string(),
            estimated_cost: best_cost,
        })
    }
}

/// Query split result
#[derive(Clone, Debug)]
pub struct QuerySplit {
    pub left_path: Vec<String>,
    pub right_path: Vec<String>,
    pub join_key: String,
    pub estimated_cost: f64,
}

/// Cost estimation for query optimization
pub struct CostEstimator {
    // Simple cost model - in practice would be learned from execution stats
    pub base_selectivity: f64,
    pub path_expansion_factor: f64,
    pub type_filter_factor: f64,
}

impl CostEstimator {
    pub fn new() -> Self {
        Self {
            base_selectivity: 0.1,      // 10% selectivity baseline
            path_expansion_factor: 2.0,  // Each path step doubles work
            type_filter_factor: 0.5,     // Type filters reduce by half
        }
    }

    pub fn estimate_cost(&self, path: &[&str], types: &[&str]) -> f64 {
        let path_cost = path.len() as f64 * self.path_expansion_factor;
        let type_cost = if types.is_empty() { 1.0 } else {
            types.len() as f64 * self.type_filter_factor
        };

        path_cost * type_cost * self.base_selectivity
    }
}

// ===== PHASE C: Snapshot CID for Popular Temporal Points =====

/// Snapshot manager for popular as_of points
pub struct SnapshotManager {
    snapshots: BTreeMap<u64, Cid>, // timestamp -> snapshot_cid
    access_counts: HashMap<u64, u64>, // timestamp -> access_count
    max_snapshots: usize,
    snapshot_interval: u64, // seconds
}

impl SnapshotManager {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: BTreeMap::new(),
            access_counts: HashMap::new(),
            max_snapshots,
            snapshot_interval: 3600, // 1 hour
        }
    }

    /// Get or create snapshot for timestamp
    pub fn get_snapshot(&mut self, as_of: u64) -> Option<Cid> {
        // Record access
        *self.access_counts.entry(as_of).or_insert(0) += 1;

        // Find closest snapshot
        self.snapshots.range(..=as_of)
            .next_back()
            .map(|(_, cid)| *cid)
    }

    /// Create new snapshot at timestamp
    pub fn create_snapshot(&mut self, as_of: u64, data_cid: Cid) {
        // Remove old snapshots if over limit
        while self.snapshots.len() >= self.max_snapshots {
            if let Some(oldest_ts) = self.snapshots.iter().next().map(|(k, _)| *k) {
                self.snapshots.remove(&oldest_ts);
                self.access_counts.remove(&oldest_ts);
            }
        }

        self.snapshots.insert(as_of, data_cid);
    }

    /// Get popular snapshot timestamps for precomputation
    pub fn get_popular_timestamps(&self, top_k: usize) -> Vec<u64> {
        let mut popular: Vec<(u64, u64)> = self.access_counts.iter()
            .map(|(ts, count)| (*ts, *count))
            .collect();

        popular.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by access count descending

        popular.into_iter()
            .take(top_k)
            .map(|(ts, _)| ts)
            .collect()
    }
}

// ===== PHASE C: SIMD VarInt (Placeholder for future SIMD implementation) =====

/// SIMD-accelerated VarInt encoding/decoding
pub mod simd_varint {
    use fcdb_core::varint;

    /// SIMD VarInt encoder (placeholder - would use SIMD instructions)
    pub fn encode_simd(values: &[u64]) -> Vec<u8> {
        let mut result = Vec::new();
        for &value in values {
            varint::encode_u64(value, &mut result);
        }
        result
    }

    /// SIMD VarInt decoder (placeholder)
    pub fn decode_simd(data: &[u8]) -> Vec<u64> {
        let mut result = Vec::new();
        let mut reader = data;
        while !reader.is_empty() {
            if let Ok(value) = varint::decode_u64(&mut reader) {
                result.push(value);
            } else {
                break;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_bloom() {
        let mut bloom = AdaptiveBloomSystem::new(AdaptiveBloomConfig::default());
        let cid = Cid([1u8; 32]);

        bloom.insert(&cid, 1, 100, 1234567890);
        assert!(bloom.contains(&cid, Some(1), Some((100, 1234567890))));
        assert!(!bloom.contains(&Cid([2u8; 32]), None, None));
    }

    #[test]
    fn test_plan_switcher() {
        let mut switcher = PlanSwitcher::new();

        let plans = vec![
            QueryPlan::PathFirst(vec!["user".to_string()]),
            QueryPlan::TypeFirst(vec!["User".to_string()]),
        ];

        let selected = switcher.select_plan("test_query", &plans);
        // Should select one of the available plans
        assert!(matches!(selected, QueryPlan::PathFirst(_) | QueryPlan::TypeFirst(_)));

        // Record a result
        switcher.record_result("test_query", &plans[0], 10.0, 100, true);

        // Should prefer the faster plan on subsequent calls
        let selected2 = switcher.select_plan("test_query", &plans);
        assert!(matches!(selected2, QueryPlan::PathFirst(_)));
    }

    #[test]
    fn test_meet_in_middle() {
        let mim = MeetInMiddle::new();
        let query_path = &["user", "posts", "comments", "replies"];

        if let Some(split) = mim.split_query(query_path, &[]) {
            assert_eq!(split.left_path.len() + split.right_path.len(), query_path.len());
            assert!(split.estimated_cost > 0.0);
        } else {
            panic!("Should split this query");
        }
    }

    #[test]
    fn test_snapshot_manager() {
        let mut manager = SnapshotManager::new(10);
        let cid = Cid([42u8; 32]);

        manager.create_snapshot(1000, cid);
        assert_eq!(manager.get_snapshot(1000), Some(cid));
        assert_eq!(manager.get_snapshot(1500), Some(cid)); // Should find closest

        let popular = manager.get_popular_timestamps(5);
        assert!(!popular.is_empty());
    }
}
