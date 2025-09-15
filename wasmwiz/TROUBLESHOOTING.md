# Wasm Wizard Troubleshooting Guide

This guide helps diagnose and resolve common issues with Wasm Wizard in production environments.

## Quick Diagnostics

### Health Check Commands

```bash
# Overall application health
curl -f http://localhost:8080/health

# Detailed health with dependencies
curl -f http://localhost:8080/health/ready

# Liveness check (for Kubernetes)
curl -f http://localhost:8080/health/live

# Database connectivity
curl -f http://localhost:8080/health/database

# Redis connectivity
curl -f http://localhost:8080/health/redis
```

### Service Status

```bash
# Docker Compose deployment
docker-compose ps
docker-compose logs --tail=50 wasm-wizard

# Kubernetes deployment
kubectl get pods -n wasm-wizard
kubectl logs -l app=wasm-wizard -n wasm-wizard --tail=50
```

## Common Issues and Solutions

### 1. Application Won't Start

#### Symptoms
- Container exits immediately
- "Connection refused" errors
- Health checks fail

#### Diagnosis
```bash
# Check container logs
docker-compose logs wasm-wizard

# Check environment variables
docker-compose exec wasm-wizard env | grep -E "(DATABASE_URL|API_SALT|REDIS_URL)"

# Verify configuration
docker-compose config
```

#### Common Causes and Solutions

**Missing Environment Variables:**
```bash
# Verify all required variables are set
cat .env
# Ensure API_SALT is at least 16 characters
echo $API_SALT | wc -c
```

**Database Connection Issues:**
```bash
# Test database connectivity
docker-compose exec postgres pg_isready -U wasm-wizard -d wasm-wizard
# Check if database exists
docker-compose exec postgres psql -U wasm-wizard -d wasm-wizard -c "\dt"
```

**Port Conflicts:**
```bash
# Check if port 8080 is already in use
netstat -tlnp | grep :8080
# Change port in docker-compose.yml if needed
```

### 2. Database Connection Failures

#### Symptoms
- "Failed to connect to database" errors
- Migrations fail
- Application crashes with database errors

#### Diagnosis
```bash
# Check PostgreSQL status
docker-compose logs postgres

# Test direct connection
docker-compose exec postgres psql -U wasm-wizard -d wasm-wizard -c "SELECT 1;"

# Check connection pool status
curl http://localhost:8080/metrics | grep database_connections
```

#### Solutions

**Database Not Ready:**
```bash
# Wait for database to fully start
docker-compose up -d postgres
sleep 30
docker-compose up wasm-wizard
```

**Wrong Credentials:**
```bash
# Verify credentials match between .env and docker-compose.yml
grep POSTGRES_PASSWORD .env
grep POSTGRES_PASSWORD docker-compose.yml
```

**Network Issues:**
```bash
# Check if containers can communicate
docker-compose exec wasm-wizard ping postgres
```

### 3. Redis Connection Issues

#### Symptoms
- Rate limiting not working
- Distributed cache errors
- "Failed to connect to Redis" logs

#### Diagnosis
```bash
# Check Redis status
docker-compose logs redis

# Test Redis connectivity
docker-compose exec redis redis-cli ping

# Test from application container
docker-compose exec wasm-wizard curl redis:6379
```

#### Solutions

**Redis Configuration:**
```bash
# Restart Redis with proper configuration
docker-compose restart redis

# Check Redis memory usage
docker-compose exec redis redis-cli info memory
```

### 4. High Memory Usage

#### Symptoms
- Container OOM kills
- Slow response times
- High swap usage

#### Diagnosis
```bash
# Monitor container resources
docker stats

# Check application memory usage
curl http://localhost:8080/metrics | grep memory

# Analyze memory distribution
docker-compose exec wasm-wizard ps aux --sort=-%mem
```

#### Solutions

**Adjust Resource Limits:**
```yaml
# In docker-compose.yml
deploy:
  resources:
    limits:
      memory: 2G  # Increase from 1G
    reservations:
      memory: 1G  # Increase from 512M
```

**Optimize WASM Execution:**
```bash
# Reduce memory limits for WASM execution
export MEMORY_LIMIT=67108864  # 64MB instead of 128MB
export MAX_WASM_SIZE=5242880  # 5MB instead of 10MB
```

### 5. Performance Issues

#### Symptoms
- Slow API responses
- High CPU usage
- Request timeouts

#### Diagnosis
```bash
# Check response times
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8080/health

# Monitor metrics
curl http://localhost:8080/metrics | grep -E "(http_request_duration|cpu)"

# Check database performance
docker-compose exec postgres psql -U wasm-wizard -d wasm-wizard -c "
SELECT query, mean_time, calls 
FROM pg_stat_statements 
ORDER BY mean_time DESC 
LIMIT 10;"
```

#### Solutions

**Database Optimization:**
```sql
-- Add missing indexes
CREATE INDEX CONCURRENTLY idx_wasm_modules_user_id ON wasm_modules(user_id);
CREATE INDEX CONCURRENTLY idx_api_keys_user_id ON api_keys(user_id);

-- Analyze table statistics
ANALYZE;

-- Check for slow queries
SELECT query, mean_time FROM pg_stat_statements WHERE mean_time > 1000;
```

**Application Scaling:**
```yaml
# Scale horizontally with multiple replicas
wasm-wizard:
  deploy:
    replicas: 3
```

### 6. WASM Execution Failures

#### Symptoms
- "Invalid WASM module" errors
- Execution timeouts
- Memory allocation failures

#### Diagnosis
```bash
# Check WASM execution logs
docker-compose logs wasm-wizard | grep -E "(wasm|execute)"

# Verify WASM file integrity
hexdump -C your_module.wasm | head -1
# Should start with: 00 61 73 6d (WASM magic bytes)

# Check execution metrics
curl http://localhost:8080/metrics | grep wasm_execution
```

#### Solutions

**Timeout Issues:**
```bash
# Increase execution timeout
export EXECUTION_TIMEOUT=10  # seconds
```

**Memory Issues:**
```bash
# Reduce WASM memory limit
export MEMORY_LIMIT=33554432  # 32MB

# Check available system memory
free -h
```

### 7. SSL/TLS Issues

#### Symptoms
- Certificate errors
- HTTPS not working
- Mixed content warnings

#### Diagnosis
```bash
# Test SSL certificate
openssl s_client -connect your-domain.com:443 -servername your-domain.com

# Check certificate expiry
echo | openssl s_client -servername your-domain.com -connect your-domain.com:443 2>/dev/null | openssl x509 -noout -dates

# Verify nginx configuration
nginx -t
```

#### Solutions

**Certificate Renewal:**
```bash
# Let's Encrypt renewal
certbot renew --dry-run
certbot renew

# Restart nginx
systemctl reload nginx
```

### 8. Monitoring and Alerting Issues

#### Symptoms
- Metrics not updating
- Grafana dashboards empty
- Alerts not firing

#### Diagnosis
```bash
# Check Prometheus targets
curl http://localhost:9090/api/v1/targets

# Verify metrics endpoint
curl http://localhost:8080/metrics

# Check Grafana data sources
curl -u admin:password http://localhost:3000/api/datasources
```

#### Solutions

**Prometheus Configuration:**
```yaml
# In monitoring/prometheus.yml
scrape_configs:
  - job_name: 'wasm-wizard'
    static_configs:
      - targets: ['wasm-wizard:8080']  # Ensure correct hostname
    metrics_path: /metrics
    scrape_interval: 15s
```

## Advanced Debugging

### Enable Debug Logging

```bash
# Temporary debug logging
docker-compose exec wasm-wizard env RUST_LOG=debug wasm-wizard

# Persistent debug logging
echo "LOG_LEVEL=debug" >> .env
docker-compose restart wasm-wizard
```

### Database Debugging

```sql
-- Enable query logging
ALTER SYSTEM SET log_statement = 'all';
ALTER SYSTEM SET log_min_duration_statement = 0;
SELECT pg_reload_conf();

-- Monitor connections
SELECT 
    pid,
    usename,
    application_name,
    client_addr,
    state,
    query_start,
    query
FROM pg_stat_activity 
WHERE datname = 'wasm-wizard';
```

### Network Debugging

```bash
# Test internal container networking
docker-compose exec wasm-wizard nslookup postgres
docker-compose exec wasm-wizard nc -zv postgres 5432

# Check external connectivity
docker-compose exec wasm-wizard curl -I google.com

# Monitor network traffic
docker-compose exec wasm-wizard netstat -tlnp
```

### Performance Profiling

```bash
# Enable application profiling
export RUST_LOG=debug,wasm-wizard=trace

# Monitor system resources
top -p $(docker-compose exec wasm-wizard pgrep wasm-wizard)

# Analyze memory usage
docker-compose exec wasm-wizard valgrind --tool=massif wasm-wizard
```

## Log Analysis

### Important Log Patterns

```bash
# Error patterns to watch
grep -E "(ERROR|FATAL|panic)" /var/log/wasm-wizard.log

# Performance patterns
grep -E "(slow|timeout|high_memory)" /var/log/wasm-wizard.log

# Security patterns
grep -E "(unauthorized|forbidden|rate_limit)" /var/log/wasm-wizard.log
```

### Structured Log Queries

```bash
# Using jq for JSON logs
cat /var/log/wasm-wizard.log | jq '.level="ERROR"'

# Filter by component
cat /var/log/wasm-wizard.log | jq '.target="wasm-wizard::handlers::execute"'

# Performance analysis
cat /var/log/wasm-wizard.log | jq '.message | contains("execution_time")' | jq '.execution_time'
```

## Recovery Procedures

### Emergency Response

1. **Application Down:**
   ```bash
   # Quick restart
   docker-compose restart wasm-wizard
   
   # If that fails, full redeploy
   docker-compose down
   docker-compose up -d
   ```

2. **Database Corruption:**
   ```bash
   # Stop application
   docker-compose stop wasm-wizard
   
   # Restore from backup
   ./scripts/restore.sh latest
   
   # Restart application
   docker-compose start wasm-wizard
   ```

3. **High Load:**
   ```bash
   # Scale horizontally
   docker-compose up --scale wasm-wizard=3
   
   # Enable maintenance mode
   echo "maintenance" > /tmp/maintenance
   ```

### Rollback Procedures

```bash
# Git-based rollback
git log --oneline -10  # Find previous stable version
git checkout <commit-hash>
docker-compose build
docker-compose up -d

# Database migration rollback
docker-compose exec wasm-wizard wasm-wizard migrate-down
```

## Performance Baselines

### Expected Metrics

| Metric | Healthy Range | Warning Threshold | Critical Threshold |
|--------|---------------|-------------------|-------------------|
| Response Time | < 100ms | > 500ms | > 1000ms |
| CPU Usage | < 50% | > 80% | > 95% |
| Memory Usage | < 70% | > 85% | > 95% |
| Database Connections | < 20 | > 80 | > 95 |
| Error Rate | < 1% | > 5% | > 10% |

### Load Testing

```bash
# Simple load test
ab -n 1000 -c 10 http://localhost:8080/health

# WASM execution load test
curl -X POST http://localhost:8080/api/wasm/execute \
  -F "wasm=@test.wasm" \
  -F "input=test"
```

## Getting Help

### Log Collection

```bash
# Collect all relevant logs
./scripts/collect-logs.sh > wasm-wizard-debug.tar.gz
```

### System Information

```bash
# Gather system info for support
uname -a
docker version
docker-compose version
free -h
df -h
```

### Contact Support

1. Create a GitHub issue with:
   - Detailed problem description
   - Steps to reproduce
   - Relevant log snippets
   - System information
   - Configuration (without secrets)

2. For urgent production issues:
   - Use emergency contact procedures
   - Include business impact assessment
   - Attach debug bundle