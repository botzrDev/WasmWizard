# Git Workflow Quick Reference Card

**Wasm Wizard Project** | **Last Updated:** June 20, 2025

## 🚀 Daily Development Commands

### Start New Work
```bash
git checkout master && git pull origin master
git checkout -b feature/your-feature-name
```

### Commit & Push
```bash
git add .
git commit -m "feat: your description"
git push origin feature/your-feature-name
```

### Before Committing (Quality Checks)
```bash
cargo fmt                    # Format code
cargo clippy --fix          # Fix lint issues
cargo test                  # Run all tests
cargo audit                 # Security check
```

## 🔄 CI/CD Pipeline Flow

```
Push → Tests → Security → Quality → Docker → Staging → [Approval] → Production
```

**Triggers**: 
- Push to `master` → Full pipeline
- Pull Request → Tests + Quality checks only

## 🎯 Branch Strategy

| Branch Type | Naming | Purpose |
|------------|---------|---------|
| `master` | `master` | Production-ready code |
| Feature | `feature/description` | New features |
| Hotfix | `hotfix/description` | Critical fixes |
| Docs | `docs/description` | Documentation updates |

## 📋 Commit Message Format

```
type(scope): description

Examples:
feat(auth): add JWT token validation
fix(api): resolve memory leak in WASM execution
docs(readme): update installation instructions
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## 🚦 Pipeline Status

### Test Requirements (41 tests must pass)
- Unit tests: 9 tests
- Integration tests: 18 tests  
- Functional tests: 14 tests

### Quality Gates
- ✅ Formatting (`cargo fmt --check`)
- ✅ Linting (`cargo clippy` - zero warnings)
- ✅ Security (`cargo audit` - no vulnerabilities)
- ✅ Dependencies (`cargo deny` - compliance)

## 🎪 Deployment Environments

| Environment | Trigger | URL | Purpose |
|------------|---------|-----|---------|
| **Staging** | Automatic | `staging.wasm-wizard.example.com` | Testing |
| **Production** | Manual Approval | `wasm-wizard.example.com` | Live |

### Approval Process
1. Go to GitHub Actions
2. Find completed workflow
3. Click "Review deployments"
4. Select "production"
5. Click "Approve and deploy"

## 🔙 Rollback Process

### Manual Rollback
1. GitHub Actions → "Manual Rollback" workflow
2. Click "Run workflow"
3. Choose environment + version
4. Confirm rollback

### Emergency Kubectl Rollback
```bash
kubectl rollout undo deployment/wasm-wizard -n wasm-wizard-production
```

## 🛠️ Troubleshooting

### Common Fixes

**Tests Failing?**
```bash
cargo test --all-features --verbose
```

**Format Issues?**
```bash
cargo fmt
```

**Lint Problems?**
```bash
cargo clippy --fix --all-targets --all-features
```

**Security Alerts?**
```bash
cargo audit
cargo update
```

### Check Status
```bash
# Local health
cargo check

# Deployment status
kubectl get pods -n wasm-wizard-production

# Logs
kubectl logs -l app=wasm-wizard -n wasm-wizard-production --tail=50
```

## 📞 Health Checks

| Endpoint | Purpose |
|----------|---------|
| `/health` | Basic health status |
| `/ready` | Readiness for traffic |
| `/metrics` | Prometheus metrics |

## 🔐 Security Checklist

- [ ] No secrets in code
- [ ] Dependencies updated
- [ ] Security scan passes
- [ ] Code review completed
- [ ] Tests cover security scenarios

## 📝 Pull Request Template

```markdown
## Description
What changes were made and why?

## Type
- [ ] Bug fix
- [ ] New feature  
- [ ] Breaking change
- [ ] Documentation

## Testing
- [ ] Tests added/updated
- [ ] Manual testing completed
- [ ] Security implications reviewed

## Checklist
- [ ] Code formatted (`cargo fmt`)
- [ ] Lints pass (`cargo clippy`)
- [ ] Tests pass (`cargo test`)
- [ ] Documentation updated
```

## 🚨 Emergency Contacts

**Pipeline Issues**: Check GitHub Actions logs  
**Deployment Issues**: Check Kubernetes status  
**Security Issues**: Run `cargo audit` and update dependencies  

---

**Keep this handy!** Pin to your workspace or bookmark this reference.

**File Location**: `DOCS/GIT_WORKFLOW_QUICK_REFERENCE.md`
