# Wasm Wizard Production Deployment Guide

This guide covers the complete production deployment of Wasm Wizard, a secure WebAssembly execution platform.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Security Setup](#security-setup)
3. [Docker Deployment](#docker-deployment)
4. [Kubernetes Deployment](#kubernetes-deployment)
5. [Monitoring Setup](#monitoring-setup)
6. [Backup and Recovery](#backup-and-recovery)
7. [Performance Tuning](#performance-tuning)
8. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

**Minimum Production Setup:**
- CPU: 4 cores
- RAM: 8GB
- Storage: 50GB SSD
- Network: 1Gbps

**Recommended Production Setup:**
- CPU: 8 cores
- RAM: 16GB
- Storage: 100GB SSD (with backup storage)
- Network: 1Gbps

### Software Dependencies

- Docker 24.0+ with Docker Compose
- PostgreSQL 15+ (if not using containerized)
- Redis 7+ (if not using containerized)
- nginx or similar reverse proxy for TLS termination
- Let's Encrypt or valid TLS certificates

## Security Setup

### 1. Generate Production Secrets

```bash
# Create secrets directory
mkdir -p /opt/wasm-wizard/secrets
cd /opt/wasm-wizard/secrets

# Generate secure database password
openssl rand -base64 32 > db_password.txt

# Generate API salt (minimum 32 characters)
openssl rand -base64 48 > api_salt.txt

# Generate Grafana admin password
openssl rand -base64 32 > grafana_password.txt

# Set proper permissions
chmod 600 *.txt
chown root:docker *.txt
```

### 2. TLS Configuration

```bash
# If using Let's Encrypt with certbot
certbot certonly --nginx -d your-domain.com

# Or place your certificates
cp your-cert.pem /opt/wasm-wizard/secrets/tls_cert.pem
cp your-key.pem /opt/wasm-wizard/secrets/tls_key.pem
chmod 600 /opt/wasm-wizard/secrets/tls_*
```

### 3. Firewall Configuration

```bash
# Allow only necessary ports
ufw allow 22/tcp    # SSH
ufw allow 80/tcp    # HTTP (for redirects)
ufw allow 443/tcp   # HTTPS
ufw deny 5432/tcp   # Block direct PostgreSQL access
ufw deny 6379/tcp   # Block direct Redis access
ufw deny 8080/tcp   # Block direct app access
ufw enable
```

## Docker Deployment

### 1. Environment Setup

```bash
# Create production directory
sudo mkdir -p /opt/wasm-wizard
cd /opt/wasm-wizard

# Clone the repository
git clone https://github.com/your-org/wasm-wizard.git .
cd wasm-wizard

# Copy production configuration
cp docker-compose.production.yml docker-compose.yml

# Set environment variables
cat > .env.production << EOF
POSTGRES_PASSWORD=$(cat secrets/db_password.txt)
API_SALT=$(cat secrets/api_salt.txt)
GRAFANA_ADMIN_PASSWORD=$(cat secrets/grafana_password.txt)
ENVIRONMENT=production
LOG_LEVEL=info
DOMAIN=your-domain.com
EOF
```

### 2. Build and Deploy

```bash
# Build the application
docker-compose build

# Start services
docker-compose up -d

# Run database migrations
docker-compose exec wasm-wizard wasm-wizard migrate

# Verify deployment
docker-compose ps
docker-compose logs wasm-wizard
```

### 3. nginx Reverse Proxy

```nginx
# /etc/nginx/sites-available/wasm-wizard
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    
    # SSL Configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeout settings
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Monitoring endpoints
    location /metrics {
        proxy_pass http://127.0.0.1:8080;
        allow 127.0.0.1;
        allow 10.0.0.0/8;
        deny all;
    }
}
```

## Kubernetes Deployment

### 1. Prepare Secrets

```bash
# Create namespace
kubectl create namespace wasm-wizard

# Create secrets
kubectl create secret generic wasm-wizard-secrets \
  --from-file=api-salt=secrets/api_salt.txt \
  --from-file=database-url=<(echo "postgresql://wasm-wizard:$(cat secrets/db_password.txt)@postgres-service:5432/wasm-wizard") \
  -n wasm-wizard

kubectl create secret generic postgres-secret \
  --from-file=password=secrets/db_password.txt \
  -n wasm-wizard
```

### 2. Deploy Services

```bash
# Deploy PostgreSQL and Redis
kubectl apply -f k8s/dependencies.yaml -n wasm-wizard

# Wait for dependencies to be ready
kubectl wait --for=condition=ready pod -l app=postgres -n wasm-wizard --timeout=300s
kubectl wait --for=condition=ready pod -l app=redis -n wasm-wizard --timeout=300s

# Deploy Wasm Wizard application
kubectl apply -f k8s/wasm-wizard-deployment.yaml -n wasm-wizard

# Verify deployment
kubectl get pods -n wasm-wizard
kubectl logs -l app=wasm-wizard -n wasm-wizard
```

### 3. Setup Ingress and TLS

```yaml
# Update k8s/wasm-wizard-deployment.yaml ingress section
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
```

## Monitoring Setup

### 1. Prometheus Configuration

The monitoring stack is included in the Docker Compose setup. Access:

- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000 (admin password in secrets)

### 2. Configure Alerts

```yaml
# monitoring/alert_rules.yml
groups:
  - name: wasm-wizard.rules
    rules:
    - alert: Wasm WizardDown
      expr: up{job="wasm-wizard"} == 0
      for: 1m
      labels:
        severity: critical
      annotations:
        summary: "Wasm Wizard instance is down"
        
    - alert: HighErrorRate
      expr: rate(wasm-wizard_http_requests_total{status=~"5.."}[5m]) > 0.1
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High error rate detected"
        
    - alert: DatabaseConnectionFailure
      expr: wasm-wizard_database_connections_failed_total > 10
      for: 1m
      labels:
        severity: critical
      annotations:
        summary: "Database connection failures"
```

## Backup and Recovery

### 1. Automated Backups

```bash
# Setup daily backups via cron
echo "0 2 * * * /opt/wasm-wizard/scripts/backup.sh" | crontab -

# Setup backup rotation and monitoring
echo "0 3 * * 0 /opt/wasm-wizard/scripts/cleanup-backups.sh" | crontab -
```

### 2. Disaster Recovery Testing

```bash
# Test restore procedure monthly
/opt/wasm-wizard/scripts/restore.sh latest

# Verify data integrity
docker-compose exec wasm-wizard wasm-wizard health-check
```

## Performance Tuning

### 1. Database Optimization

```sql
-- PostgreSQL configuration for production
ALTER SYSTEM SET max_connections = 100;
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
```

### 2. Application Tuning

```toml
# Cargo.toml production profile
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### 3. Container Resource Limits

```yaml
# docker-compose.yml resources
deploy:
  resources:
    limits:
      memory: 1G
      cpus: '1.0'
    reservations:
      memory: 512M
      cpus: '0.5'
```

## Troubleshooting

### Common Issues

1. **Database Connection Failures**
   ```bash
   # Check database status
   docker-compose logs postgres
   docker-compose exec postgres pg_isready -U wasm-wizard
   ```

2. **High Memory Usage**
   ```bash
   # Monitor container resources
   docker stats
   # Check for memory leaks
   docker-compose exec wasm-wizard ps aux
   ```

3. **WASM Execution Timeouts**
   ```bash
   # Check execution logs
   docker-compose logs wasm-wizard | grep "execution_timeout"
   # Adjust timeout in environment variables
   ```

### Log Analysis

```bash
# Real-time monitoring
docker-compose logs -f wasm-wizard

# Error analysis
docker-compose logs wasm-wizard | grep "ERROR"

# Performance metrics
curl http://localhost:8080/metrics
```

### Health Checks

```bash
# Application health
curl -f http://localhost:8080/health

# Database health
curl -f http://localhost:8080/health/database

# Dependencies health
curl -f http://localhost:8080/health/dependencies
```

## Security Checklist

- [ ] All secrets are generated and stored securely
- [ ] TLS certificates are valid and auto-renewing
- [ ] Firewall rules are configured correctly
- [ ] Database access is restricted to application only
- [ ] All containers run as non-root users
- [ ] Security headers are configured in reverse proxy
- [ ] Rate limiting is enabled
- [ ] Log aggregation is configured
- [ ] Backup encryption is enabled
- [ ] Security scanning is automated in CI/CD

## Maintenance Schedule

### Daily
- Monitor application logs and metrics
- Check backup completion
- Review security alerts

### Weekly
- Update dependencies (after testing)
- Review and rotate logs
- Performance analysis

### Monthly
- Security audit
- Disaster recovery testing
- Capacity planning review
- SSL certificate renewal check

For additional support, consult the [troubleshooting guide](TROUBLESHOOTING.md) or create an issue in the repository.