# Wasm Wizard Production Deployment Checklist

## Pre-Deployment Requirements

### Infrastructure Prerequisites
- [ ] **Docker Engine 20.10+** installed and running
- [ ] **Docker Compose v2.10+** available
- [ ] **Kubernetes cluster** (v1.24+) for container orchestration (optional)
- [ ] **Load balancer** (Nginx, AWS ALB, etc.) configured with SSL/TLS termination
- [ ] **DNS** records pointing to load balancer
- [ ] **SSL certificates** (Let's Encrypt or commercial CA)

### Security Setup
- [ ] **Production secrets** generated using `./scripts/generate_secrets.sh`
- [ ] **Secrets stored securely** (not in version control)
- [ ] **API salt** changed from development default
- [ ] **Database credentials** are strong and unique
- [ ] **File permissions** set to 600 for secret files
- [ ] **Firewall rules** configured (allow only ports 80, 443, 22)

### Database Setup
- [ ] **PostgreSQL 15+** database server provisioned
- [ ] **Database migrations** tested and ready: `sqlx migrate run`
- [ ] **Database backups** configured and tested
- [ ] **Connection pooling** limits configured appropriately
- [ ] **Database monitoring** enabled (optional: pgAdmin)

### Redis Setup
- [ ] **Redis 7** server provisioned for rate limiting and caching
- [ ] **Redis persistence** configured (RDB + AOF recommended)
- [ ] **Redis authentication** enabled if exposed
- [ ] **Redis monitoring** configured

## Deployment Steps

### 1. Environment Configuration
```bash
# Copy and configure environment
cp .env.example .env.production
# Edit .env.production with production values
```

**Required Environment Variables:**
- [ ] `DATABASE_URL` - PostgreSQL connection string
- [ ] `REDIS_URL` - Redis connection string
- [ ] `API_SALT_FILE` - Path to API salt secret file
- [ ] `ENVIRONMENT=production`
- [ ] `LOG_LEVEL=info`
- [ ] `SERVER_HOST=0.0.0.0`
- [ ] `SERVER_PORT=8080`

### 2. Build and Test
```bash
# Lint and security audit
cargo clippy -- -D warnings
cargo audit  # If cargo-audit installed

# Build release binary
cargo build --release

# Run tests (optional, requires test database)
cargo test --release
```

### 3. Docker Deployment
```bash
# Generate secrets first
./scripts/generate_secrets.sh

# Start production stack
docker-compose -f docker-compose.production.yml up -d

# Check services are healthy
docker-compose -f docker-compose.production.yml ps
```

### 4. Kubernetes Deployment (Alternative)
```bash
# Apply Kubernetes manifests
kubectl apply -f k8s/

# Check deployment status
kubectl get pods -n wasm-wizard-production
kubectl get services -n wasm-wizard-production
kubectl get ingress -n wasm-wizard-production
```

## Post-Deployment Validation

### Health Checks
- [ ] **Basic health endpoint** responds: `curl https://yourdomain.com/health`
- [ ] **Liveness probe** responds: `curl https://yourdomain.com/health/live`
- [ ] **Readiness probe** responds: `curl https://yourdomain.com/health/ready`
- [ ] **Metrics endpoint** accessible: `curl https://yourdomain.com/metrics`

### Functional Testing
- [ ] **WASM upload and execution** works via API
- [ ] **API key generation** works: `POST /api/auth/keys`
- [ ] **Rate limiting** enforced correctly
- [ ] **Authentication** working properly
- [ ] **Database operations** functioning (user creation, logging)
- [ ] **File cleanup** working (temp WASM files deleted)

### Security Validation
- [ ] **HTTPS only** - HTTP redirects to HTTPS
- [ ] **Security headers** present (CSP, HSTS, etc.)
- [ ] **API keys** require authentication
- [ ] **Rate limiting** active on all endpoints
- [ ] **CORS** configured appropriately
- [ ] **Input validation** working (file size limits, etc.)

### Performance Validation
- [ ] **Response times** < 200ms for health endpoints
- [ ] **WASM execution** completes within timeout limits
- [ ] **Memory usage** stable under normal load
- [ ] **Database connections** not exhausted
- [ ] **File descriptors** not leaking

### Monitoring Setup
- [ ] **Prometheus** scraping metrics on port 9090
- [ ] **Grafana** dashboards accessible on port 3000
- [ ] **Alerting rules** configured and tested
- [ ] **Log aggregation** working (structured JSON logs)
- [ ] **Uptime monitoring** configured (external service recommended)

## Load Testing (Optional but Recommended)

### Automated Load Tests
```bash
# Run comprehensive load tests
./scripts/load_test.sh -u https://yourdomain.com -c 50 -n 1000

# Test specific scenarios
./scripts/load_test.sh -u https://yourdomain.com -c 10 -n 100 -d 60
```

**Load Test Targets:**
- [ ] **1000+ requests/second** on health endpoints
- [ ] **100+ concurrent WASM executions**
- [ ] **99th percentile** response time < 500ms
- [ ] **Error rate** < 1% under normal load
- [ ] **Memory usage** stable during sustained load
- [ ] **Rate limiting** properly enforced under load

## Operational Procedures

### Backup Procedures
```bash
# Database backup
./scripts/backup.sh

# Restore from backup (if needed)
./scripts/restore.sh latest
```

### Monitoring Alerts
**Critical Alerts:**
- Service down (> 30s)
- Database connection failures
- WASM execution timeouts (> 5 in 5min)
- High error rate (> 5%)

**Warning Alerts:**
- High response time (> 5s 95th percentile)
- High memory usage (> 512MB)
- High rate limit hits (> 10/sec)

### Scaling Procedures
- [ ] **Horizontal scaling** tested (multiple replicas)
- [ ] **Auto-scaling** configured (HPA in Kubernetes)
- [ ] **Database scaling** plan ready
- [ ] **Load balancer** health checks configured

### Disaster Recovery
- [ ] **Database backups** automated and tested
- [ ] **Secrets backup** stored securely offsite
- [ ] **Infrastructure as Code** (Terraform/CloudFormation)
- [ ] **Recovery procedures** documented and tested
- [ ] **RTO/RPO targets** defined (e.g., RTO < 15min, RPO < 5min)

## Security Hardening Checklist

### Application Security
- [ ] **API keys** use SHA-256 hashing
- [ ] **Rate limiting** with Redis/memory backends
- [ ] **Input validation** on all endpoints
- [ ] **File upload limits** enforced (10MB WASM, 1MB input)
- [ ] **WASM sandbox** limits (5s timeout, 128MB memory)
- [ ] **Temporary files** cleaned up automatically
- [ ] **No sensitive data** logged

### Infrastructure Security
- [ ] **TLS 1.2+** only for HTTPS
- [ ] **Strong ciphers** configured
- [ ] **Security headers** implemented
- [ ] **Container scanning** for vulnerabilities
- [ ] **Secrets management** (not in images/code)
- [ ] **Non-root containers** running
- [ ] **Read-only filesystems** where possible
- [ ] **Network policies** limiting access

## Final Deployment Validation

### Pre-Launch Checklist
- [ ] All health checks passing
- [ ] All functional tests passing
- [ ] All security validations complete
- [ ] Monitoring and alerting active
- [ ] Load testing completed successfully
- [ ] Backup/restore procedures tested
- [ ] Documentation updated
- [ ] Team trained on operations

### Post-Launch Monitoring (First 24 Hours)
- [ ] Monitor error rates and response times
- [ ] Watch for memory leaks or resource issues
- [ ] Verify rate limiting effectiveness
- [ ] Check database performance
- [ ] Monitor WASM execution patterns
- [ ] Review security logs for anomalies

### Weekly Operations Review
- [ ] Review performance metrics and trends
- [ ] Check backup integrity
- [ ] Update dependencies and security patches
- [ ] Review and tune alerting thresholds
- [ ] Capacity planning based on usage patterns

---

## Emergency Contacts and Procedures

**Escalation Path:**
1. On-call Engineer (immediate response)
2. Senior DevOps Engineer (< 30min)
3. Technical Lead (< 1hr)
4. Engineering Manager (< 2hr)

**Critical Incident Response:**
1. **Acknowledge** alert and start incident response
2. **Assess** impact and severity
3. **Mitigate** immediate impact (rollback if needed)
4. **Communicate** status to stakeholders
5. **Resolve** root cause
6. **Document** post-incident review

## Tools and Resources

**Production Tools:**
- Docker Compose: `docker-compose -f docker-compose.production.yml`
- Kubernetes: `kubectl` with production context
- Monitoring: Grafana (port 3000), Prometheus (port 9090)
- Load Testing: `./scripts/load_test.sh`
- Database: pgAdmin (optional, port 5050)

**Useful Commands:**
```bash
# Check service status
docker-compose -f docker-compose.production.yml ps

# View logs
docker-compose -f docker-compose.production.yml logs -f wasm-wizard

# Scale services
docker-compose -f docker-compose.production.yml up --scale wasm-wizard=3

# Database migration
cargo run -- --migrate

# Generate secrets
./scripts/generate_secrets.sh
```

This checklist ensures a comprehensive, production-ready deployment of Wasm Wizard with enterprise-grade security, monitoring, and operational procedures.