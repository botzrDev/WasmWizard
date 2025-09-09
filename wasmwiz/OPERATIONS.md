# WasmWiz Production Operations Guide

This guide provides everything needed to operate WasmWiz in production environments.

## Quick Start

```bash
# Generate production secrets
./scripts/generate_secrets.sh

# Start production stack
docker-compose -f docker-compose.production.yml up -d

# Verify deployment
curl http://localhost:8080/health
```

## Production Checklist

### Pre-Deployment
- [ ] **Security**: Generate unique secrets with `./scripts/generate_secrets.sh`
- [ ] **Infrastructure**: Ensure adequate resources (8GB RAM, 4 CPU cores minimum)
- [ ] **Network**: Configure firewall rules and TLS certificates
- [ ] **Database**: Set up PostgreSQL with proper backups
- [ ] **Monitoring**: Configure Prometheus and Grafana dashboards

### Post-Deployment
- [ ] **Health Checks**: Verify all endpoints respond correctly
- [ ] **Performance**: Run load tests with `./scripts/load_test.sh`
- [ ] **Backups**: Test backup and restore procedures
- [ ] **Monitoring**: Configure alerts and notification channels
- [ ] **Documentation**: Update runbooks with environment-specific details

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Load Balancer │────│   WasmWiz App   │────│   PostgreSQL    │
│    (nginx)      │    │   (Rust/Actix)  │    │   (Database)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                       ┌─────────────────┐    ┌─────────────────┐
                       │      Redis      │    │   Prometheus    │
                       │    (Cache)      │    │  (Monitoring)   │
                       └─────────────────┘    └─────────────────┘
```

## Key Components

### WasmWiz Application
- **Technology**: Rust with Actix Web framework
- **Features**: WebAssembly execution, REST API, authentication
- **Port**: 8080 (internal), 443 (external via reverse proxy)
- **Health Check**: `GET /health`

### Database (PostgreSQL)
- **Version**: PostgreSQL 15+
- **Purpose**: User data, API keys, WASM modules metadata
- **Port**: 5432 (internal only)
- **Backup**: Automated daily backups with `./scripts/backup.sh`

### Cache (Redis)
- **Version**: Redis 7+
- **Purpose**: Rate limiting, session caching
- **Port**: 6379 (internal only)
- **Persistence**: RDB snapshots for durability

### Monitoring Stack
- **Prometheus**: Metrics collection on port 9090
- **Grafana**: Dashboards and visualization on port 3000
- **Alerts**: Configured for critical system events

## Environment Configuration

### Required Environment Variables

```bash
# Database
DATABASE_URL=postgresql://user:password@host:5432/wasmwiz

# Redis
REDIS_URL=redis://host:6379

# Security
API_SALT=your-secure-random-salt-minimum-32-characters

# Application
ENVIRONMENT=production
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
LOG_LEVEL=info

# Resource Limits
MAX_WASM_SIZE=10485760      # 10MB
MAX_INPUT_SIZE=1048576      # 1MB
EXECUTION_TIMEOUT=5         # seconds
MEMORY_LIMIT=134217728      # 128MB
```

### Optional Configuration

```bash
# TLS (if handling SSL termination in app)
TLS_CERT_PATH=/path/to/cert.pem
TLS_KEY_PATH=/path/to/key.pem

# Monitoring
PROMETHEUS_ENABLED=true
METRICS_ENDPOINT=/metrics

# Rate Limiting Tiers
FREE_TIER_RATE_MINUTE=10
FREE_TIER_RATE_DAY=500
BASIC_TIER_RATE_MINUTE=100
BASIC_TIER_RATE_DAY=10000
PRO_TIER_RATE_MINUTE=500
PRO_TIER_RATE_DAY=50000
```

## Deployment Options

### Docker Compose (Recommended for single-node)

```bash
# Production deployment
docker-compose -f docker-compose.production.yml up -d

# View logs
docker-compose logs -f wasmwiz

# Update application
docker-compose pull wasmwiz
docker-compose up -d wasmwiz
```

### Kubernetes (Recommended for multi-node)

```bash
# Deploy to Kubernetes
kubectl apply -f k8s/

# Check deployment status
kubectl get pods -n wasmwiz

# View logs
kubectl logs -l app=wasmwiz -n wasmwiz -f

# Update deployment
kubectl set image deployment/wasmwiz wasmwiz=wasmwiz:new-tag -n wasmwiz
```

## Monitoring and Alerting

### Key Metrics to Monitor

#### Application Metrics
- `wasmwiz_http_requests_total` - HTTP request count
- `wasmwiz_http_request_duration_seconds` - Response times
- `wasmwiz_wasm_executions_total` - WASM execution count
- `wasmwiz_database_connections_active` - Database connections

#### System Metrics
- CPU usage (target: <80%)
- Memory usage (target: <85%)
- Disk usage (target: <85%)
- Network I/O

#### Business Metrics
- API key generation rate
- User registrations
- WASM upload frequency
- Error rates by endpoint

### Alert Thresholds

| Metric | Warning | Critical |
|--------|---------|----------|
| Response Time (95th percentile) | >500ms | >1000ms |
| Error Rate | >5% | >10% |
| CPU Usage | >80% | >95% |
| Memory Usage | >85% | >95% |
| Disk Usage | >85% | >95% |

### Grafana Dashboards

Access Grafana at `http://localhost:3000` with admin credentials from secrets.

**Key Dashboards:**
1. **Application Overview** - Request rates, response times, error rates
2. **System Resources** - CPU, memory, disk, network usage
3. **Database Performance** - Connection pool, query performance
4. **Business Metrics** - User activity, API usage patterns

## Backup and Recovery

### Automated Backups

```bash
# Run daily backups (add to crontab)
0 2 * * * /opt/wasmwiz/scripts/backup.sh

# Manual backup
./scripts/backup.sh

# List available backups
ls -la /opt/wasmwiz/backups/
```

### Disaster Recovery

```bash
# Restore from latest backup
./scripts/restore.sh latest

# Restore from specific backup
./scripts/restore.sh /path/to/backup.sql.gz

# Verify restore
curl http://localhost:8080/health
```

### Backup Strategy
- **Frequency**: Daily automated backups
- **Retention**: 7 days local, 30 days remote
- **Storage**: Local disk + cloud storage (S3/GCS)
- **Testing**: Monthly restore verification

## Security

### TLS Configuration

```nginx
# nginx SSL configuration
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
ssl_prefer_server_ciphers off;
ssl_session_cache shared:SSL:10m;

# Security headers
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
add_header X-Frame-Options DENY always;
add_header X-Content-Type-Options nosniff always;
add_header X-XSS-Protection "1; mode=block" always;
```

### Firewall Rules

```bash
# Allow only necessary ports
ufw allow 22/tcp    # SSH
ufw allow 80/tcp    # HTTP (redirects)
ufw allow 443/tcp   # HTTPS
ufw deny 5432/tcp   # Block direct DB access
ufw deny 6379/tcp   # Block direct Redis access
ufw deny 8080/tcp   # Block direct app access
```

### API Security
- **Authentication**: JWT-based API keys
- **Rate Limiting**: Tiered limits per user
- **Input Validation**: All inputs sanitized
- **WASM Sandboxing**: Isolated execution environment

## Performance Tuning

### Application Optimization

```toml
# Cargo.toml production settings
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### Database Tuning

```sql
-- PostgreSQL performance settings
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
```

### Redis Optimization

```redis
# Redis configuration
maxmemory 512mb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
save 60 10000
```

## Troubleshooting

### Common Issues

**Application won't start:**
1. Check environment variables
2. Verify database connectivity
3. Check port availability
4. Review application logs

**High memory usage:**
1. Monitor WASM execution limits
2. Check for memory leaks
3. Review connection pool settings
4. Analyze long-running requests

**Database connection issues:**
1. Verify PostgreSQL is running
2. Check connection string format
3. Review firewall rules
4. Monitor connection pool exhaustion

### Log Analysis

```bash
# View application logs
docker-compose logs wasmwiz | grep ERROR

# Monitor real-time logs
docker-compose logs -f wasmwiz

# Search for specific patterns
docker-compose logs wasmwiz | grep "execution_timeout"

# Export logs for analysis
docker-compose logs --since 24h wasmwiz > /tmp/wasmwiz.log
```

### Performance Testing

```bash
# Run load tests
./scripts/load_test.sh

# Test specific endpoints
./scripts/load_test.sh -u https://your-domain.com -c 100 -n 5000

# Custom test scenarios
ab -n 1000 -c 50 http://localhost:8080/health
wrk -t12 -c400 -d30s http://localhost:8080/metrics
```

## Maintenance

### Daily Tasks
- [ ] Check application health and error rates
- [ ] Monitor resource usage trends
- [ ] Review security alerts
- [ ] Verify backup completion

### Weekly Tasks
- [ ] Update dependencies (after testing)
- [ ] Review performance metrics
- [ ] Clean up old logs and backups
- [ ] Security scan with `cargo audit`

### Monthly Tasks
- [ ] Disaster recovery testing
- [ ] Performance benchmark comparison
- [ ] Security audit and penetration testing
- [ ] Capacity planning review
- [ ] SSL certificate renewal check

## Support and Escalation

### Log Collection for Support

```bash
# Collect diagnostic information
./scripts/collect-logs.sh > wasmwiz-debug-$(date +%Y%m%d).tar.gz
```

### Emergency Contacts
- **On-call Engineer**: [Your emergency contact]
- **Database Administrator**: [DBA contact]
- **Security Team**: [Security contact]

### Escalation Procedures
1. **P1 (Critical)** - Complete service outage
   - Response time: 15 minutes
   - Contact on-call engineer immediately

2. **P2 (High)** - Significant performance degradation
   - Response time: 1 hour
   - Follow troubleshooting guide first

3. **P3 (Medium)** - Minor issues or questions
   - Response time: 24 hours
   - Create GitHub issue with details

## Additional Resources

- [Production Deployment Guide](PRODUCTION_DEPLOYMENT.md)
- [Troubleshooting Guide](TROUBLESHOOTING.md)
- [API Documentation](https://your-domain.com/api/docs)
- [Security Guidelines](SECURITY.md)
- [Contributing Guidelines](CONTRIBUTING.md)