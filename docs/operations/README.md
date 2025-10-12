# Own-CFA-Enishi Operations Manual

## Overview

This manual provides operational procedures for deploying, monitoring, and maintaining Own-CFA-Enishi systems in production environments.

## Deployment

### Prerequisites

#### System Requirements
- **CPU**: 4+ cores (8+ recommended)
- **Memory**: 8GB minimum (16GB+ recommended)
- **Storage**: 100GB+ SSD storage
- **Network**: 1Gbps+ connectivity
- **OS**: Linux (Ubuntu 20.04+, RHEL 8+)

#### Dependencies
```bash
# Required packages
sudo apt-get install -y \
    curl \
    jq \
    prometheus \
    grafana \
    docker.io \
    docker-compose
```

### Single-Node Deployment

#### Docker Deployment
```bash
# Create data directory
mkdir -p ./data ./logs

# Run with Docker
docker run -d \
  --name enishi \
  -p 8080:8080 \
  -p 9090:9090 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/logs:/app/logs \
  -e RUST_LOG=info \
  -e ENISHI_STORAGE_PATH=/app/data \
  --restart unless-stopped \
  enishi/enishi:latest
```

#### Docker Compose
```yaml
# docker-compose.yml
version: '3.8'
services:
  enishi:
    image: enishi/enishi:latest
    ports:
      - "8080:8080"   # HTTP API
      - "9090:9090"   # Metrics
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
    environment:
      - RUST_LOG=info
      - ENISHI_PORT=8080
      - ENISHI_STORAGE_PATH=/app/data
      - ENISHI_MAX_SIZE_GB=100
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

#### Manual Installation
```bash
# Download binary
wget https://github.com/your-org/enishi/releases/latest/download/enishi-linux-x64.tar.gz
tar -xzf enishi-linux-x64.tar.gz
sudo mv enishi /usr/local/bin/

# Create user
sudo useradd -r -s /bin/false enishi

# Create directories
sudo mkdir -p /var/lib/enishi/data /var/log/enishi
sudo chown -R enishi:enishi /var/lib/enishi /var/log/enishi

# Create systemd service
sudo tee /etc/systemd/system/enishi.service > /dev/null <<EOF
[Unit]
Description=Own-CFA-Enishi Database
After=network.target

[Service]
User=enishi
Group=enishi
ExecStart=/usr/local/bin/enishi
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

# Start service
sudo systemctl daemon-reload
sudo systemctl enable enishi
sudo systemctl start enishi
```

### Configuration

#### Environment Variables
```bash
# Server
ENISHI_PORT=8080
ENISHI_HOST=0.0.0.0
ENISHI_WORKERS=8
ENISHI_MAX_CONNECTIONS=10000

# Storage
ENISHI_STORAGE_PATH=/var/lib/enishi/data
ENISHI_MAX_SIZE_GB=500
ENISHI_COMPRESSION=true

# Performance
ENISHI_CACHE_SIZE=10000000
ENISHI_BLOOM_SIZE=100000000
ENISHI_ADAPTIVE_OPTIMIZATION=true

# Security
ENISHI_ENABLE_AUDIT=true
ENISHI_AUDIT_LOG_PATH=/var/log/enishi/audit.log
ENISHI_SESSION_TIMEOUT=3600

# Monitoring
RUST_LOG=info,enishi=debug
ENISHI_METRICS_PORT=9090
```

#### Configuration File
```toml
# /etc/enishi/config.toml
[server]
port = 8080
host = "0.0.0.0"
workers = 8
max_connections = 10000
timeout_secs = 30

[storage]
path = "/var/lib/enishi/data"
max_size_gb = 500
compression = true
sync_writes = false

[performance]
query_cache_size = 10000000
bloom_filter_size = 100000000
max_concurrent_queries = 1000
adaptive_optimization = true

[security]
enable_audit = true
audit_log_path = "/var/log/enishi/audit.log"
max_sessions = 10000
session_timeout_secs = 3600

[monitoring]
metrics_port = 9090
enable_prometheus = true
log_level = "info"
health_check_interval_secs = 30
```

## Monitoring

### Health Checks

#### HTTP Endpoints
```bash
# Health check
curl -s http://localhost:8080/health | jq

# Readiness check
curl -s http://localhost:8080/ready | jq

# System status
curl -s http://localhost:8080/status | jq
```

#### Automated Monitoring
```bash
#!/bin/bash
# health_monitor.sh

ENDPOINT="http://localhost:8080"

while true; do
    # Health check
    if ! curl -s --max-time 5 "${ENDPOINT}/health" > /dev/null; then
        echo "$(date): Health check failed" >> /var/log/enishi/health.log
        # Send alert (email, Slack, etc.)
    fi

    # Performance metrics
    curl -s "${ENDPOINT}/metrics" >> /var/log/enishi/metrics.log

    sleep 30
done
```

### Metrics Collection

#### Prometheus Configuration
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'enishi'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
```

#### Key Metrics to Monitor

**Performance Metrics:**
```
enishi_query_duration_seconds{quantile="0.95"} < 0.01  # P95 < 10ms
enishi_queries_per_second_total > 1000               # Throughput
enishi_cache_hit_ratio > 0.98                        # Cache efficiency
```

**Resource Metrics:**
```
enishi_memory_usage_bytes < 8GB                     # Memory usage
enishi_cpu_usage_percent < 80                        # CPU usage
enishi_storage_used_bytes / enishi_storage_total_bytes < 0.9  # Storage usage
```

**Error Metrics:**
```
rate(enishi_error_count_total[5m]) < 0.01           # Error rate < 1%
enishi_last_error_timestamp offset 1h < time()      # Recent errors
```

### Grafana Dashboards

#### Sample Dashboard JSON
```json
{
  "dashboard": {
    "title": "Own-CFA-Enishi Overview",
    "panels": [
      {
        "title": "Query Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(enishi_query_duration_seconds_bucket[5m]))",
            "legendFormat": "P95 Latency"
          }
        ]
      },
      {
        "title": "System Resources",
        "type": "graph",
        "targets": [
          {
            "expr": "enishi_memory_usage_bytes / 1024 / 1024 / 1024",
            "legendFormat": "Memory (GB)"
          },
          {
            "expr": "enishi_cpu_usage_percent",
            "legendFormat": "CPU %"
          }
        ]
      }
    ]
  }
}
```

## Maintenance

### Backup Procedures

#### Full Backup
```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/var/backups/enishi"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/enishi_backup_${TIMESTAMP}.tar.gz"

# Stop service for consistent backup
sudo systemctl stop enishi

# Create backup
sudo tar -czf "$BACKUP_FILE" \
    --exclude='*.tmp' \
    --exclude='*.log' \
    /var/lib/enishi/data

# Restart service
sudo systemctl start enishi

# Verify backup
if [ $? -eq 0 ]; then
    echo "Backup completed: $BACKUP_FILE"
    # Upload to cloud storage
    aws s3 cp "$BACKUP_FILE" "s3://enishi-backups/"
else
    echo "Backup failed!"
    exit 1
fi
```

#### Incremental Backup
```bash
#!/bin/bash
# incremental_backup.sh

LAST_BACKUP=$(find /var/backups/enishi -name "*.tar.gz" | sort | tail -1)
rsync -av --link-dest="$LAST_BACKUP" /var/lib/enishi/data/ "/var/backups/enishi/incremental_$(date +%Y%m%d_%H%M%S)/"
```

### Log Management

#### Log Rotation
```bash
# /etc/logrotate.d/enishi
/var/log/enishi/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 enishi enishi
    postrotate
        systemctl reload enishi
    endscript
}
```

#### Log Analysis
```bash
# Query performance analysis
grep "query" /var/log/enishi/enishi.log | \
    awk '{print $NF}' | \
    sort -n | \
    awk '
        BEGIN { sum=0; count=0; p95_idx=int(0.95*NR) }
        { sum+=$1; count++; latencies[count]=$1 }
        END {
            if (count > 0) {
                print "Average latency:", sum/count "ms"
                print "P95 latency:", latencies[int(0.95*count)] "ms"
            }
        }
    '
```

### Performance Tuning

#### Memory Optimization
```bash
# Adjust cache sizes based on workload
export ENISHI_CACHE_SIZE=50000000    # 50M entries
export ENISHI_BLOOM_SIZE=500000000   # 500M bits
```

#### Query Optimization
```bash
# Enable adaptive optimization
export ENISHI_ADAPTIVE_OPTIMIZATION=true

# Monitor plan selection
curl -s http://localhost:8080/metrics | grep plan
```

#### Storage Optimization
```bash
# Adjust compression settings
export ENISHI_COMPRESSION=true

# Monitor storage efficiency
curl -s http://localhost:8080/metrics | grep storage
```

## Troubleshooting

### Common Issues

#### High Memory Usage
**Symptoms:** Memory usage >80%
**Diagnosis:**
```bash
# Check memory metrics
curl -s http://localhost:8080/metrics | grep memory

# Check cache sizes
curl -s http://localhost:8080/status | jq .configuration
```

**Solutions:**
- Reduce cache sizes: `ENISHI_CACHE_SIZE=50000000`
- Enable compression: `ENISHI_COMPRESSION=true`
- Restart service: `sudo systemctl restart enishi`

#### Slow Queries
**Symptoms:** P95 latency >20ms
**Diagnosis:**
```bash
# Check query performance
curl -s http://localhost:8080/metrics | grep query_duration

# Check cache hit ratio
curl -s http://localhost:8080/metrics | grep cache_hit
```

**Solutions:**
- Increase cache sizes
- Enable adaptive optimization
- Check for hot spots in data access patterns

#### Storage Full
**Symptoms:** Storage usage >90%
**Diagnosis:**
```bash
# Check storage metrics
curl -s http://localhost:8080/metrics | grep storage

# Check disk usage
df -h /var/lib/enishi/data
```

**Solutions:**
- Clean up old data
- Increase storage capacity
- Implement data archival

#### Connection Issues
**Symptoms:** Active connections low, error rate high
**Diagnosis:**
```bash
# Check connection metrics
curl -s http://localhost:8080/metrics | grep connections

# Check network connectivity
netstat -tlnp | grep 8080
```

**Solutions:**
- Check firewall settings
- Verify network configuration
- Monitor for DDoS attacks

### Debug Procedures

#### Enable Debug Logging
```bash
# Temporary debug logging
export RUST_LOG=debug,enishi=trace
sudo systemctl restart enishi

# View logs
journalctl -u enishi -f
```

#### Performance Profiling
```bash
# Install profiling tools
sudo apt-get install -y linux-tools-common linux-tools-generic

# Profile CPU usage
sudo perf record -p $(pgrep enishi) -o perf.data
sudo perf report -i perf.data
```

#### Memory Leak Detection
```bash
# Install Valgrind
sudo apt-get install -y valgrind

# Run with memory checking
valgrind --leak-check=full ./enishi
```

## Scaling

### Horizontal Scaling

#### Load Balancer Configuration
```nginx
# nginx.conf
upstream enishi_cluster {
    server 10.0.0.1:8080;
    server 10.0.0.2:8080;
    server 10.0.0.3:8080;
}

server {
    listen 80;
    location / {
        proxy_pass http://enishi_cluster;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

#### Database Clustering
```yaml
# docker-compose.cluster.yml
version: '3.8'
services:
  enishi-1:
    image: enishi/enishi:latest
    environment:
      - CLUSTER_NODE_ID=1
      - CLUSTER_PEERS=enishi-2:9091,enishi-3:9092

  enishi-2:
    image: enishi/enishi:latest
    environment:
      - CLUSTER_NODE_ID=2
      - CLUSTER_PEERS=enishi-1:9090,enishi-3:9092

  enishi-3:
    image: enishi/enishi:latest
    environment:
      - CLUSTER_NODE_ID=3
      - CLUSTER_PEERS=enishi-1:9090,enishi-2:9091
```

### Vertical Scaling

#### Resource Allocation
```yaml
# Kubernetes resource limits
resources:
  requests:
    memory: "4Gi"
    cpu: "2"
  limits:
    memory: "8Gi"
    cpu: "4"
```

#### Performance Tuning
```bash
# Scale with CPU cores
export ENISHI_WORKERS=$(nproc)

# Adjust memory settings
export ENISHI_CACHE_SIZE=$((50 * 1024 * 1024))  # 50M * cores
```

## Security

### Access Control

#### Capability Management
```bash
# Generate capability tokens
curl -X POST http://localhost:8080/admin/capabilities \
  -H "Content-Type: application/json" \
  -d '{
    "resource": "/data/users",
    "permissions": ["read", "write"],
    "expires_in": 3600
  }'
```

#### Audit Logging
```bash
# View audit logs
tail -f /var/log/enishi/audit.log

# Search for specific operations
grep "user_id=123" /var/log/enishi/audit.log
```

### Network Security

#### TLS Configuration
```nginx
# nginx.conf with TLS
server {
    listen 443 ssl http2;
    server_name api.enishi.dev;

    ssl_certificate /etc/ssl/certs/enishi.crt;
    ssl_certificate_key /etc/ssl/private/enishi.key;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

#### Firewall Rules
```bash
# Allow only necessary ports
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP (redirect to HTTPS)
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 9090/tcp  # Metrics (internal only)
sudo ufw --force enable
```

## Disaster Recovery

### Backup Strategy

#### Automated Backups
```bash
# Daily full backup
0 2 * * * /usr/local/bin/enishi-backup.sh

# Hourly incremental backup
0 * * * * /usr/local/bin/enishi-incremental-backup.sh
```

#### Backup Verification
```bash
#!/bin/bash
# verify_backup.sh

BACKUP_FILE=$1

# Extract and verify
tar -tzf "$BACKUP_FILE" > /dev/null
if [ $? -eq 0 ]; then
    echo "Backup integrity verified"
else
    echo "Backup corrupted!"
    # Send alert
fi
```

### Recovery Procedures

#### Data Recovery
```bash
#!/bin/bash
# recover.sh

BACKUP_FILE=$1

# Stop service
sudo systemctl stop enishi

# Restore data
sudo tar -xzf "$BACKUP_FILE" -C /

# Verify data integrity
/usr/local/bin/enishi --verify-data

# Start service
sudo systemctl start enishi
```

#### Service Recovery
```bash
# Automatic restart on failure
sudo systemctl enable enishi
sudo systemctl set-property enishi Restart=always
sudo systemctl set-property enishi RestartSec=5
```

## Compliance

### Data Protection

#### GDPR Compliance
- **Data minimization**: Only collect necessary data
- **Purpose limitation**: Clear data usage policies
- **Storage limitation**: Automatic data deletion
- **Integrity & confidentiality**: Encryption and access controls

#### Audit Requirements
```bash
# Enable comprehensive auditing
export ENISHI_ENABLE_AUDIT=true
export ENISHI_AUDIT_LOG_PATH=/var/log/enishi/audit.log

# Regular audit log rotation
sudo logrotate /etc/logrotate.d/enishi
```

### Security Standards

#### CIS Benchmarks
- **File permissions**: Restrictive file access
- **Service accounts**: Dedicated system users
- **Network security**: Minimal exposed services
- **Logging**: Comprehensive audit trails

#### Penetration Testing
```bash
# Automated security scanning
sudo apt-get install -y openvas
sudo openvas-start
# Run regular vulnerability scans
```

---

This operations manual provides comprehensive procedures for running Own-CFA-Enishi in production. For additional support, see the [development guide](../development/README.md) or create an issue in the project repository.
