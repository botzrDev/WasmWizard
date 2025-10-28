# Redis Setup and Configuration Guide

Redis is **required for production** deployments of WasmWizard to enable distributed rate limiting and caching across multiple application instances.

## Why Redis is Required for Production

### Rate Limiting Requirements

WasmWizard implements three-tier rate limiting (Free, Basic, Pro) that must work consistently across:
- Multiple application instances (horizontal scaling)
- Load balancer distributions
- Kubernetes pod replicas

**Without Redis:** Rate limits are stored in-memory per instance, leading to:
- ❌ Inconsistent rate limiting across instances
- ❌ Users can bypass limits by hitting different instances
- ❌ No shared state between pods

**With Redis:** Distributed rate limiting ensures:
- ✅ Consistent limits across all instances
- ✅ Shared state for accurate rate tracking
- ✅ Support for horizontal scaling

### Caching Benefits

Redis also provides:
- API key validation caching (reduces database load)
- Session management
- Real-time metrics aggregation

## Development vs Production

### Development (REDIS_ENABLED=false)

For local development, Redis is **optional**:
```bash
# .env file
REDIS_ENABLED=false
```

The application falls back to in-memory rate limiting:
- ✅ Works for single-instance development
- ✅ No external dependencies
- ❌ Not suitable for production
- ❌ Rate limits don't persist across restarts

### Production (REDIS_ENABLED=true)

For production, Redis is **mandatory**:
```bash
# .env file
REDIS_ENABLED=true
REDIS_URL=redis://redis-host:6379
```

The application will fail validation if Redis is not properly configured.

## Installation Options

### Option 1: Docker Compose (Recommended for Single Server)

WasmWizard includes Redis in the production Docker Compose stack:

```bash
cd wasmwiz
docker-compose -f docker-compose.production.yml up -d
```

This automatically starts:
- PostgreSQL database
- Redis cache
- WasmWizard application
- Prometheus + Grafana monitoring

**Configuration in docker-compose.production.yml:**
```yaml
redis:
  image: redis:7-alpine
  ports:
    - "6379:6379"
  volumes:
    - redis_data:/data
  healthcheck:
    test: ["CMD", "redis-cli", "ping"]
    interval: 10s
    timeout: 5s
    retries: 5
  restart: unless-stopped
  deploy:
    resources:
      limits:
        memory: 512M
      reservations:
        memory: 256M
```

### Option 2: Managed Redis Service (Recommended for Production)

For production deployments, use a managed Redis service:

#### AWS ElastiCache

```bash
# Create Redis cluster
aws elasticache create-cache-cluster \
  --cache-cluster-id wasm-wizard-redis \
  --engine redis \
  --cache-node-type cache.t3.micro \
  --num-cache-nodes 1 \
  --engine-version 7.0

# Get endpoint
aws elasticache describe-cache-clusters \
  --cache-cluster-id wasm-wizard-redis \
  --show-cache-node-info

# Set in environment
REDIS_URL=redis://wasm-wizard-redis.xxxxxx.0001.use1.cache.amazonaws.com:6379
```

#### Google Cloud Memorystore

```bash
# Create Redis instance
gcloud redis instances create wasm-wizard-redis \
  --size=1 \
  --region=us-central1 \
  --redis-version=redis_7_0

# Get connection info
gcloud redis instances describe wasm-wizard-redis \
  --region=us-central1

# Set in environment
REDIS_URL=redis://10.0.0.3:6379
```

#### Azure Cache for Redis

```bash
# Create Redis cache
az redis create \
  --name wasm-wizard-redis \
  --resource-group wasm-wizard-rg \
  --location eastus \
  --sku Basic \
  --vm-size c0

# Get connection string
az redis list-keys \
  --name wasm-wizard-redis \
  --resource-group wasm-wizard-rg

# Set in environment
REDIS_URL=redis://:PRIMARY_KEY@wasm-wizard-redis.redis.cache.windows.net:6379
```

#### DigitalOcean Managed Redis

```bash
# Create via web console or API
# Then set:
REDIS_URL=redis://default:password@db-redis-nyc1-12345.b.db.ondigitalocean.com:25061
```

### Option 3: Kubernetes with Helm

```bash
# Add Bitnami repository
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo update

# Install Redis
helm install redis bitnami/redis \
  --namespace production \
  --set auth.enabled=true \
  --set auth.password="SECURE_PASSWORD" \
  --set master.persistence.enabled=true \
  --set master.persistence.size=8Gi \
  --set replica.replicaCount=2

# Get Redis password
export REDIS_PASSWORD=$(kubectl get secret --namespace production redis -o jsonpath="{.data.redis-password}" | base64 -d)

# Update WasmWizard deployment
REDIS_URL=redis://:${REDIS_PASSWORD}@redis-master.production.svc.cluster.local:6379
```

### Option 4: Standalone Redis Server

#### Ubuntu/Debian

```bash
# Install Redis
sudo apt update
sudo apt install redis-server -y

# Configure Redis
sudo nano /etc/redis/redis.conf

# Recommended settings:
# maxmemory 512mb
# maxmemory-policy allkeys-lru
# bind 0.0.0.0  # Only if accessed remotely
# requirepass YOUR_SECURE_PASSWORD

# Start Redis
sudo systemctl enable redis-server
sudo systemctl start redis-server

# Test connection
redis-cli ping  # Should return PONG
```

#### Docker (Single Container)

```bash
docker run -d \
  --name wasm-wizard-redis \
  --restart unless-stopped \
  -p 6379:6379 \
  -v redis-data:/data \
  redis:7-alpine \
  redis-server --requirepass "YOUR_SECURE_PASSWORD"
```

## Configuration

### Environment Variables

```bash
# Enable Redis (required for production)
REDIS_ENABLED=true

# Redis connection URL
REDIS_URL=redis://hostname:6379

# With password
REDIS_URL=redis://:password@hostname:6379

# With username and password
REDIS_URL=redis://username:password@hostname:6379

# Redis Cluster
REDIS_URL=redis://node1:6379,node2:6379,node3:6379

# Redis Sentinel
REDIS_URL=redis://master-name?sentinel=sentinel1:26379,sentinel2:26379,sentinel3:26379
```

### Redis Configuration Recommendations

#### For Small Deployments (< 1000 users)

```redis
maxmemory 256mb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
save 60 10000
```

#### For Medium Deployments (1,000 - 10,000 users)

```redis
maxmemory 512mb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
appendonly yes
```

#### For Large Deployments (10,000+ users)

```redis
maxmemory 2048mb
maxmemory-policy allkeys-lru
save ""  # Disable RDB snapshots
appendonly yes
appendfsync everysec
```

## Security Hardening

### 1. Enable Authentication

```redis
# redis.conf
requirepass YOUR_VERY_STRONG_PASSWORD
```

### 2. Disable Dangerous Commands

```redis
# redis.conf
rename-command FLUSHDB ""
rename-command FLUSHALL ""
rename-command KEYS ""
rename-command CONFIG "CONFIG_SECRET_NAME"
```

### 3. Network Security

```redis
# Only allow specific IPs
bind 10.0.0.5 127.0.0.1

# Or use firewall rules
sudo ufw allow from 10.0.0.0/24 to any port 6379
```

### 4. TLS Encryption (Redis 6+)

```redis
# Generate certificates
openssl req -x509 -nodes -newkey rsa:4096 \
  -keyout redis.key -out redis.crt -days 365

# redis.conf
tls-port 6380
port 0
tls-cert-file /path/to/redis.crt
tls-key-file /path/to/redis.key
tls-ca-cert-file /path/to/ca.crt
```

Connection string:
```bash
REDIS_URL=rediss://hostname:6380  # Note: rediss (with double 's')
```

## Monitoring and Maintenance

### Health Check

```bash
# Simple ping
redis-cli ping

# With password
redis-cli -a YOUR_PASSWORD ping

# Check memory usage
redis-cli info memory

# Check connected clients
redis-cli client list
```

### Performance Monitoring

```bash
# Monitor real-time commands
redis-cli monitor

# Get slowlog
redis-cli slowlog get 10

# Check hit/miss ratio
redis-cli info stats | grep keyspace
```

### Backup and Recovery

#### Manual Backup

```bash
# Create RDB snapshot
redis-cli BGSAVE

# Copy the dump file
cp /var/lib/redis/dump.rdb /backup/redis-backup-$(date +%Y%m%d).rdb
```

#### Automated Backup (cron)

```bash
# Add to crontab
0 2 * * * redis-cli BGSAVE && \
  sleep 60 && \
  cp /var/lib/redis/dump.rdb /backup/redis-$(date +\%Y\%m\%d-\%H\%M).rdb && \
  find /backup -name "redis-*.rdb" -mtime +7 -delete
```

#### Restore from Backup

```bash
# Stop Redis
sudo systemctl stop redis-server

# Replace dump file
sudo cp /backup/redis-backup-20250101.rdb /var/lib/redis/dump.rdb
sudo chown redis:redis /var/lib/redis/dump.rdb

# Start Redis
sudo systemctl start redis-server
```

## Troubleshooting

### Connection Refused

```bash
# Check if Redis is running
sudo systemctl status redis-server

# Check port binding
sudo netstat -tlnp | grep 6379

# Test connection
redis-cli -h hostname -p 6379 ping
```

### Out of Memory

```bash
# Check memory usage
redis-cli info memory

# Check maxmemory setting
redis-cli config get maxmemory

# Increase maxmemory
redis-cli config set maxmemory 512mb
redis-cli config rewrite
```

### Authentication Failed

```bash
# Test with password
redis-cli -a YOUR_PASSWORD ping

# Check config
redis-cli config get requirepass

# Update password
redis-cli config set requirepass NEW_PASSWORD
redis-cli config rewrite
```

### High CPU Usage

```bash
# Check slow queries
redis-cli slowlog get 10

# Monitor commands
redis-cli --latency

# Check for blocking operations
redis-cli client list | grep blocked
```

## Migration from In-Memory to Redis

### Step 1: Deploy Redis

Choose one of the installation options above.

### Step 2: Update Configuration

```bash
# .env file
REDIS_ENABLED=true
REDIS_URL=redis://your-redis-host:6379
```

### Step 3: Rolling Update

For zero-downtime migration:

1. Deploy Redis
2. Update 1 application instance with new config
3. Test rate limiting
4. Update remaining instances
5. Monitor for errors

### Step 4: Verify

```bash
# Check Redis connections
redis-cli client list

# Monitor rate limit keys
redis-cli --scan --pattern "ratelimit:*"

# Check memory usage
redis-cli info memory
```

## Production Checklist

Before going to production, ensure:

- [ ] Redis deployed and accessible
- [ ] Authentication enabled (requirepass)
- [ ] Network security configured (firewall/security groups)
- [ ] Backups configured (RDB or AOF)
- [ ] Monitoring set up (Prometheus, CloudWatch, etc.)
- [ ] `REDIS_ENABLED=true` in production .env
- [ ] `REDIS_URL` correctly configured
- [ ] Connection tested from application server
- [ ] Persistence enabled (if needed)
- [ ] Resource limits configured (maxmemory)

## Support

For Redis-specific issues:
- Check Redis logs: `sudo journalctl -u redis-server -f`
- Official docs: https://redis.io/documentation
- WasmWizard config: See `src/config.rs` and `src/services/redis.rs`

For WasmWizard integration issues:
- Check application logs
- Verify `REDIS_ENABLED=true`
- Test Redis connection: `redis-cli -h HOST -p PORT ping`
