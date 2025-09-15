# Branch Protection Rules Configuration

## Overview
This document provides instructions for setting up branch protection rules for the Wasm Wizard repository to ensure code quality and security.

## Required Branch Protection Rules

### Master Branch (Production)
**Branch:** `master`
**Protection Level:** Maximum

Required settings:
- [x] **Require pull request reviews before merging**
  - Required number of reviewers: 2
  - Dismiss stale reviews when new commits are pushed: ✓
  - Require review from code owners: ✓
  - Require approval of the most recent push: ✓

- [x] **Require status checks to pass before merging**
  - Require branches to be up to date: ✓
  - Required status checks:
    - `Security Audit`
    - `Lint and Format Check`
    - `Test Suite`
    - `Docker Build and Security Scan`

- [x] **Require conversation resolution before merging**

- [x] **Require signed commits**

- [x] **Require linear history**

- [x] **Include administrators** (applies rules to admins too)

- [x] **Restrict pushes that create files**

- [x] **Allow force pushes: NO**

- [x] **Allow deletions: NO**

### Development Branch (Staging)
**Branch:** `development`
**Protection Level:** High

Required settings:
- [x] **Require pull request reviews before merging**
  - Required number of reviewers: 1
  - Dismiss stale reviews when new commits are pushed: ✓

- [x] **Require status checks to pass before merging**
  - Require branches to be up to date: ✓
  - Required status checks:
    - `Security Audit`
    - `Lint and Format Check`
    - `Test Suite`
    - `Docker Build and Security Scan`

- [x] **Require conversation resolution before merging**

- [x] **Allow force pushes: NO**

- [x] **Allow deletions: NO**

### Research Branch (Experimental)
**Branch:** `research`
**Protection Level:** Medium

Required settings:
- [x] **Require status checks to pass before merging**
  - Required status checks:
    - `Security Audit` (allow failures for known issues)
    - `Lint and Format Check`
    - `Test Suite`

- [x] **Allow force pushes: YES** (for experimental work)

- [x] **Allow deletions: NO**

## Setup Instructions

### Via GitHub Web Interface:

1. Navigate to your repository on GitHub
2. Go to **Settings** → **Branches**
3. Click **Add rule** for each branch
4. Configure settings as specified above

### Via GitHub CLI:

```bash
# Install GitHub CLI if not already installed
# gh auth login

# Master branch protection
gh api repos/:owner/:repo/branches/master/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["Security Audit","Lint and Format Check","Test Suite","Docker Build and Security Scan"]}' \
  --field enforce_admins=true \
  --field required_pull_request_reviews='{"required_approving_review_count":2,"dismiss_stale_reviews":true,"require_code_owner_reviews":true,"require_last_push_approval":true}' \
  --field restrictions=null \
  --field allow_force_pushes=false \
  --field allow_deletions=false \
  --field required_linear_history=true \
  --field required_conversation_resolution=true

# Development branch protection
gh api repos/:owner/:repo/branches/development/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["Security Audit","Lint and Format Check","Test Suite","Docker Build and Security Scan"]}' \
  --field enforce_admins=false \
  --field required_pull_request_reviews='{"required_approving_review_count":1,"dismiss_stale_reviews":true}' \
  --field restrictions=null \
  --field allow_force_pushes=false \
  --field allow_deletions=false

# Research branch protection
gh api repos/:owner/:repo/branches/research/protection \
  --method PUT \
  --field required_status_checks='{"strict":false,"contexts":["Lint and Format Check","Test Suite"]}' \
  --field enforce_admins=false \
  --field required_pull_request_reviews=null \
  --field restrictions=null \
  --field allow_force_pushes=true \
  --field allow_deletions=false
```

## Workflow Integration

### Pull Request Flow:
1. **Feature branches** → `development` (1 reviewer, all checks)
2. **Development** → `master` (2 reviewers, all checks, linear history)
3. **Research** → `development` (via PR when ready)

### Emergency Hotfixes:
1. Create hotfix branch from `master`
2. Apply minimal fix
3. PR to `master` (can bypass some reviews with admin override if critical)
4. Cherry-pick to `development`

## Automated Quality Gates

All branches require these CI/CD checks to pass:
- ✅ Security audit (cargo-audit)
- ✅ Code formatting (rustfmt)
- ✅ Linting (clippy)
- ✅ Test suite (85%+ coverage)
- ✅ Docker security scan (Trivy)

## Branch Permissions

### Master Branch:
- **Write access:** Repository admins only
- **Merge access:** Via PR with 2 approvals only
- **Direct push:** Blocked for everyone

### Development Branch:
- **Write access:** Core team members
- **Merge access:** Via PR with 1 approval
- **Direct push:** Blocked

### Research Branch:
- **Write access:** All contributors
- **Merge access:** Direct or PR
- **Direct push:** Allowed for rapid iteration

## Monitoring & Compliance

- All protected branch violations are logged
- PR reviews are recorded for audit trails
- Failed security checks prevent merges
- Branch protection bypass events trigger alerts

## Emergency Procedures

In case of critical production issues:
1. Create hotfix branch from `master`
2. Implement minimal fix
3. Request admin override for accelerated merge
4. Deploy immediately
5. Follow up with proper testing and documentation
6. Conduct post-incident review

## Maintenance

- Review and update rules quarterly
- Monitor bypass usage and investigate
- Update required status checks when CI changes
- Regular audit of permissions and access