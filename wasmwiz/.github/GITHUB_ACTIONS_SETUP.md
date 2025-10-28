# GitHub Actions CI/CD Setup Guide

This guide explains how to configure GitHub Actions for automated CI/CD deployment of WasmWizard.

## Prerequisites

- GitHub repository with admin access
- Container registry (Docker Hub, GitHub Container Registry, AWS ECR, etc.)
- Kubernetes cluster (optional, for automated deployment)

## Step 1: Create GitHub Environments

GitHub Environments provide deployment protection rules and environment-specific secrets.

### Create Staging Environment

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Environments**
3. Click **New environment**
4. Name it: `staging`
5. (Optional) Configure protection rules:
   - Required reviewers
   - Wait timer
   - Deployment branches (e.g., only `develop`)

### Create Production Environment

1. Click **New environment** again
2. Name it: `production`
3. **Recommended protection rules:**
   - ✅ Required reviewers (at least 1-2 reviewers)
   - ✅ Wait timer: 5 minutes
   - ✅ Deployment branches: only `main`

## Step 2: Configure Repository Secrets

Go to **Settings** → **Secrets and variables** → **Actions** → **New repository secret**

### Required Secrets

Add the following secrets:

#### Container Registry Credentials

| Secret Name | Description | Example |
|-------------|-------------|---------|
| `REGISTRY_URL` | Container registry URL | `docker.io`, `ghcr.io`, `123456789.dkr.ecr.us-east-1.amazonaws.com` |
| `REGISTRY_USERNAME` | Registry username | `your-username` or service account |
| `REGISTRY_PASSWORD` | Registry password/token | Personal access token or password |

**For Docker Hub:**
```
REGISTRY_URL=docker.io
REGISTRY_USERNAME=your-dockerhub-username
REGISTRY_PASSWORD=your-dockerhub-token
```

**For GitHub Container Registry (ghcr.io):**
```
REGISTRY_URL=ghcr.io
REGISTRY_USERNAME=your-github-username
REGISTRY_PASSWORD=your-github-personal-access-token
```

**For AWS ECR:**
```
REGISTRY_URL=123456789.dkr.ecr.us-east-1.amazonaws.com
REGISTRY_USERNAME=AWS
REGISTRY_PASSWORD=<temporary-token-from-aws-cli>
```
*Note: For ECR, consider using OIDC authentication instead*

### Environment-Specific Secrets (Optional)

If using different registries for staging/production, add secrets to each environment:

1. Go to **Settings** → **Environments** → **staging** → **Add secret**
2. Add environment-specific secrets that override repository secrets

## Step 3: Configure Kubernetes Deployment (Optional)

If deploying to Kubernetes, you'll need to add kubeconfig access.

### Option A: Using kubectl directly

Add to repository secrets:
```
KUBE_CONFIG_DATA=<base64-encoded-kubeconfig>
```

To generate:
```bash
cat ~/.kube/config | base64 -w 0
```

Then in your workflow, add:
```yaml
- name: Set up kubectl
  run: |
    mkdir -p ~/.kube
    echo "${{ secrets.KUBE_CONFIG_DATA }}" | base64 -d > ~/.kube/config
    kubectl version
```

### Option B: Using Helm

Add to repository secrets:
```
HELM_VALUES_STAGING=<base64-encoded-values.yaml>
HELM_VALUES_PRODUCTION=<base64-encoded-values.yaml>
```

### Option C: Using Cloud Provider CLI

**For AWS EKS:**
```
AWS_ACCESS_KEY_ID=<your-key-id>
AWS_SECRET_ACCESS_KEY=<your-secret-key>
AWS_REGION=us-east-1
EKS_CLUSTER_NAME=wasm-wizard-cluster
```

**For Google GKE:**
```
GCP_PROJECT_ID=<your-project-id>
GCP_SA_KEY=<base64-encoded-service-account-json>
GKE_CLUSTER_NAME=wasm-wizard-cluster
GKE_ZONE=us-central1-a
```

## Step 4: Update Deployment Steps in Workflow

Open `.github/workflows/ci.yml` and uncomment the deployment commands:

### For Kubernetes with kubectl:

```yaml
- name: Deploy to staging environment
  run: |
    kubectl set image deployment/wasm-wizard \
      wasm-wizard=${{ secrets.REGISTRY_URL }}/wasm-wizard:staging-${{ github.sha }} \
      --namespace=staging
    kubectl rollout status deployment/wasm-wizard --namespace=staging
```

### For Helm:

```yaml
- name: Deploy to staging environment
  run: |
    helm upgrade --install wasm-wizard ./helm/wasm-wizard \
      --namespace staging \
      --set image.repository=${{ secrets.REGISTRY_URL }}/wasm-wizard \
      --set image.tag=staging-${{ github.sha }} \
      --values helm/values-staging.yaml
```

### For Docker Compose (simple deployments):

```yaml
- name: Deploy to staging environment
  run: |
    ssh deploy@staging-server << 'EOF'
      cd /opt/wasm-wizard
      docker pull ${{ secrets.REGISTRY_URL }}/wasm-wizard:staging-${{ github.sha }}
      docker-compose down
      docker-compose up -d
    EOF
```

## Step 5: Test the Workflow

### Test Staging Deployment

1. Create a feature branch
2. Make a commit
3. Push to `develop` branch:
   ```bash
   git checkout develop
   git merge feature-branch
   git push origin develop
   ```
4. Watch the workflow run in **Actions** tab
5. Approve the deployment if you configured reviewers

### Test Production Deployment

1. Merge `develop` into `main`:
   ```bash
   git checkout main
   git merge develop
   git push origin main
   ```
2. Watch the workflow run
3. Approve the production deployment

## Step 6: Monitor and Maintain

### Monitoring Workflow Runs

- Check **Actions** tab for build status
- Set up notifications: **Settings** → **Notifications** → **Actions**
- Configure Slack/Discord webhooks for deployment notifications

### Security Best Practices

1. **Rotate secrets regularly** (every 90 days recommended)
2. **Use scoped tokens** with minimum required permissions
3. **Enable branch protection** on `main` and `develop`
4. **Require status checks** before merging
5. **Enable Dependabot** for security updates

### Troubleshooting

**Problem:** `Error: Unable to find image '...'`
- **Solution:** Check REGISTRY_URL format and credentials

**Problem:** `Error: Unable to connect to cluster`
- **Solution:** Verify KUBE_CONFIG_DATA is correctly base64 encoded

**Problem:** `403 Forbidden` when pushing to registry
- **Solution:** Check REGISTRY_PASSWORD has write permissions

**Problem:** Deployment approved but not running
- **Solution:** Check branch protection rules and deployment branch restrictions

## Advanced Configuration

### Using OIDC Authentication (Recommended)

For AWS, GCP, or Azure, use OIDC instead of long-lived credentials:

#### AWS OIDC Setup:

```yaml
- name: Configure AWS credentials
  uses: aws-actions/configure-aws-credentials@v4
  with:
    role-to-assume: arn:aws:iam::123456789:role/GitHubActionsRole
    aws-region: us-east-1
```

Benefits:
- No long-lived credentials
- Automatic token rotation
- Better security

### Multi-Region Deployments

```yaml
- name: Deploy to multiple regions
  run: |
    for region in us-east-1 eu-west-1 ap-southeast-1; do
      kubectl set image deployment/wasm-wizard \
        wasm-wizard=${{ secrets.REGISTRY_URL }}/wasm-wizard:${{ github.sha }} \
        --namespace=production \
        --context=$region-context
    done
```

### Blue-Green Deployments

```yaml
- name: Blue-Green deployment
  run: |
    # Deploy green environment
    kubectl apply -f k8s/deployment-green.yaml

    # Wait for green to be ready
    kubectl wait --for=condition=ready pod -l app=wasm-wizard,slot=green

    # Switch traffic
    kubectl patch service wasm-wizard -p '{"spec":{"selector":{"slot":"green"}}}'

    # Scale down blue
    kubectl scale deployment wasm-wizard-blue --replicas=0
```

## Support

For issues with GitHub Actions setup:
- Check workflow logs in Actions tab
- Review [GitHub Actions Documentation](https://docs.github.com/en/actions)
- Open an issue in the repository

For WasmWizard deployment issues:
- See `PRODUCTION_DEPLOYMENT.md`
- Check `k8s/` directory for Kubernetes manifests
- Review `docker-compose.production.yml` for Docker deployments
