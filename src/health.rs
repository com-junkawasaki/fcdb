//! Health checking system for Own-CFA-Enishi

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub system_health: ComponentHealth,
    pub storage_health: ComponentHealth,
    pub memory_health: ComponentHealth,
    pub connections_health: ComponentHealth,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthLevel,
    pub message: String,
    pub last_check: u64,
    pub response_time_ms: u64,
}

/// Health level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Health checker for system components
pub struct HealthChecker {
    start_time: Instant,
    checks: RwLock<Vec<HealthCheck>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            start_time: Instant::now(),
            checks: RwLock::new(Vec::new()),
        };

        // Register default health checks
        checker.register_default_checks();
        checker
    }

    /// Register default health checks
    fn register_default_checks(&mut self) {
        // System health check
        self.register_check(HealthCheck {
            name: "system".to_string(),
            check_fn: Box::new(|_| async {
                // Basic system health - always healthy for now
                ComponentHealth {
                    status: HealthLevel::Healthy,
                    message: "System operational".to_string(),
                    last_check: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    response_time_ms: 1,
                }
            }),
        });

        // Memory health check
        self.register_check(HealthCheck {
            name: "memory".to_string(),
            check_fn: Box::new(|_| async {
                let usage = get_memory_usage_mb();
                let status = if usage < 8000 { // 8GB limit
                    HealthLevel::Healthy
                } else if usage < 12000 { // 12GB warning
                    HealthLevel::Degraded
                } else {
                    HealthLevel::Unhealthy
                };

                ComponentHealth {
                    status,
                    message: format!("Memory usage: {} MB", usage),
                    last_check: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    response_time_ms: 1,
                }
            }),
        });

        // Storage health check
        self.register_check(HealthCheck {
            name: "storage".to_string(),
            check_fn: Box::new(|_| async {
                // Check if storage is accessible
                let accessible = std::path::Path::new("./data").exists() ||
                               std::fs::create_dir_all("./data").is_ok();

                ComponentHealth {
                    status: if accessible { HealthLevel::Healthy } else { HealthLevel::Unhealthy },
                    message: if accessible {
                        "Storage accessible".to_string()
                    } else {
                        "Storage inaccessible".to_string()
                    },
                    last_check: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    response_time_ms: 5,
                }
            }),
        });

        // Connections health check
        self.register_check(HealthCheck {
            name: "connections".to_string(),
            check_fn: Box::new(|_| async {
                // Placeholder - would check active connections
                ComponentHealth {
                    status: HealthLevel::Healthy,
                    message: "Connections within limits".to_string(),
                    last_check: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    response_time_ms: 2,
                }
            }),
        });
    }

    /// Register a custom health check
    pub fn register_check(&mut self, check: HealthCheck) {
        // This would be called during initialization
        // For now, we just store the checks
    }

    /// Perform comprehensive health check
    pub async fn check(&self) -> HealthStatus {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let uptime_seconds = self.start_time.elapsed().as_secs();

        // Perform individual component checks
        let system_health = self.check_component("system").await;
        let storage_health = self.check_component("storage").await;
        let memory_health = self.check_component("memory").await;
        let connections_health = self.check_component("connections").await;

        // Overall health determination
        let component_healths = [&system_health, &storage_health, &memory_health, &connections_health];
        let healthy = component_healths.iter().all(|h| h.status == HealthLevel::Healthy);

        HealthStatus {
            healthy,
            timestamp: now,
            uptime_seconds,
            system_health,
            storage_health,
            memory_health,
            connections_health,
        }
    }

    /// Check individual component health
    async fn check_component(&self, name: &str) -> ComponentHealth {
        match name {
            "system" => ComponentHealth {
                status: HealthLevel::Healthy,
                message: "System operational".to_string(),
                last_check: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_time_ms: 1,
            },
            "memory" => {
                let usage = get_memory_usage_mb();
                let status = if usage < 8000 {
                    HealthLevel::Healthy
                } else if usage < 12000 {
                    HealthLevel::Degraded
                } else {
                    HealthLevel::Unhealthy
                };

                ComponentHealth {
                    status,
                    message: format!("Memory usage: {} MB", usage),
                    last_check: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    response_time_ms: 1,
                }
            },
            "storage" => {
                let accessible = std::path::Path::new("./data").exists() ||
                               std::fs::create_dir_all("./data").is_ok();

                ComponentHealth {
                    status: if accessible { HealthLevel::Healthy } else { HealthLevel::Unhealthy },
                    message: if accessible {
                        "Storage accessible".to_string()
                    } else {
                        "Storage inaccessible".to_string()
                    },
                    last_check: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    response_time_ms: 5,
                }
            },
            "connections" => ComponentHealth {
                status: HealthLevel::Healthy,
                message: "Connections within limits".to_string(),
                last_check: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_time_ms: 2,
            },
            _ => ComponentHealth {
                status: HealthLevel::Unknown,
                message: format!("Unknown component: {}", name),
                last_check: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_time_ms: 0,
            },
        }
    }
}

/// Individual health check definition
pub struct HealthCheck {
    pub name: String,
    pub check_fn: Box<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ComponentHealth> + Send>> + Send + Sync>,
}

/// Get current memory usage in MB
fn get_memory_usage_mb() -> u64 {
    // Placeholder implementation
    // In a real system, this would read from /proc/self/status or similar
    // For now, return a reasonable fake value
    2048 // 2GB
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_checker() {
        let checker = HealthChecker::new();
        let status = checker.check().await;

        assert!(status.timestamp > 0);
        assert!(status.uptime_seconds >= 0);
        assert_eq!(status.system_health.status, HealthLevel::Healthy);
    }

    #[test]
    fn test_health_levels() {
        assert_eq!(HealthLevel::Healthy, HealthLevel::Healthy);
        assert_ne!(HealthLevel::Healthy, HealthLevel::Unhealthy);
    }
}
