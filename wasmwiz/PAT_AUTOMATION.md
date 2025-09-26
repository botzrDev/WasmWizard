# 🔐 GitHub Personal Access Token (PAT) Automation

## Overview

WasmWizard's GitHub PAT automation provides a secure way to create, manage, and automatically revoke temporary Personal Access Tokens for short-lived repository access. This system addresses the security risks associated with long-lived credentials while maintaining the automation capabilities needed for CI/CD and contributor workflows.

## 🚀 Quick Start

### Prerequisites

1. **GitHub CLI**: Install the [GitHub CLI](https://cli.github.com/)
2. **jq**: Install [jq](https://stedolan.github.io/jq/) for JSON processing
3. **Authentication**: Run `gh auth login` to authenticate with GitHub

### Basic Usage

```bash
# Navigate to the wasmwiz directory
cd wasmwiz

# Create a PAT for CI/CD deployment (2 hours)
./scripts/github-pat-manager.sh create \
  -d "WasmWizard CI/CD Deploy" \
  -s "repo,read:org" \
  -e 2

# List active PATs
./scripts/github-pat-manager.sh list

# Clean up expired PATs
./scripts/github-pat-manager.sh cleanup
```

## 🔧 Features

### 🎯 Core Capabilities

- **Automated PAT Creation**: Generate PATs with custom scopes and expiration
- **Secure Storage**: Metadata-only storage (never stores actual tokens)
- **Automatic Revocation**: Clean up expired tokens automatically
- **Scope Limitation**: Restrict PAT permissions to minimum required
- **Repository Filtering**: Limit access to specific repositories
- **Comprehensive Logging**: Audit trail without token exposure

### 🛡️ Security Features

- **No Token Logging**: PAT values are never written to logs or files
- **Time-based Expiration**: Configurable short-lived tokens (hours/days)
- **Automatic Cleanup**: Scheduled cleanup of expired tokens
- **Secure Metadata Storage**: Protected file permissions (600)
- **Scope Validation**: Ensures minimal required permissions

## 📖 Command Reference

### Create PAT

Create a new temporary Personal Access Token:

```bash
./scripts/github-pat-manager.sh create [OPTIONS]

OPTIONS:
  -d, --description DESCRIPTION    Description for the PAT (required)
  -s, --scopes SCOPES             Comma-separated list of scopes (default: repo,read:org)
  -e, --expiration HOURS          Expiration time in hours (default: 24)
  -r, --repos REPOS              Comma-separated list of repositories (optional)
```

**Examples:**

```bash
# Basic deployment PAT
./scripts/github-pat-manager.sh create -d "Deploy to staging" -s "repo" -e 4

# Contributor access with repository restriction  
./scripts/github-pat-manager.sh create \
  -d "Contributor PR review" \
  -s "repo,read:org" \
  -e 8 \
  -r "botzrDev/WasmWizard"

# CI/CD with multiple scopes
./scripts/github-pat-manager.sh create \
  -d "CI/CD Pipeline" \
  -s "repo,packages:read,actions:read" \
  -e 1
```

### List PATs

List all active (non-expired) PATs:

```bash
./scripts/github-pat-manager.sh list
```

### Revoke PAT

Revoke a specific PAT by its ID:

```bash
./scripts/github-pat-manager.sh revoke -i TOKEN_ID
```

### Cleanup Expired PATs

Remove all expired PATs:

```bash
./scripts/github-pat-manager.sh cleanup
```

## 🔄 GitHub Workflow Integration

### Manual Workflow Dispatch

You can create PATs through GitHub Actions:

1. Go to **Actions** tab in the GitHub repository
2. Select **"GitHub PAT Automation"** workflow  
3. Click **"Run workflow"**
4. Fill in the required parameters:
   - **Action**: `create`, `list`, or `cleanup`
   - **Description**: Purpose of the PAT
   - **Scopes**: Required permissions
   - **Expiration**: Hours until expiration
   - **Repositories**: Optional repository restrictions

### Automated Cleanup

The workflow runs automatic cleanup daily at 2 AM UTC to remove expired PATs.

### CI/CD Integration Example

```yaml
name: Deploy with Temporary PAT

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Create deployment PAT
      id: create-pat
      run: |
        cd wasmwiz
        PAT=$(./scripts/github-pat-manager.sh create \
          -d "Deploy $(date '+%Y-%m-%d %H:%M')" \
          -s "repo,packages:write" \
          -e 1)
        echo "::add-mask::$PAT"
        echo "PAT=$PAT" >> $GITHUB_ENV
    
    - name: Deploy using PAT
      env:
        GITHUB_TOKEN: ${{ env.PAT }}
      run: |
        # Use the PAT for deployment operations
        git clone https://x-access-token:$PAT@github.com/botzrDev/WasmWizard.git deploy-repo
        # Perform deployment...
    
    - name: Cleanup PAT
      if: always()
      run: |
        cd wasmwiz
        ./scripts/github-pat-manager.sh cleanup
```

## ⚙️ Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PAT_EXPIRATION_HOURS` | Default expiration time in hours | `24` |
| `PAT_SCOPES` | Default comma-separated scopes | `repo,read:org` |
| `LOG_FILE` | Path to log file | `./pat-manager.log` |
| `PAT_STORE_FILE` | Path to metadata store | `/tmp/.wasmwiz-pats.json` |

### Scope Options

Common GitHub PAT scopes for different use cases:

| Use Case | Recommended Scopes | Description |
|----------|-------------------|-------------|
| **Repository Clone/Pull** | `repo` | Read access to repositories |
| **CI/CD Deploy** | `repo,packages:write` | Repository + package publishing |
| **Contributor Access** | `repo,read:org` | Repository + organization info |
| **Release Management** | `repo,packages:write,contents:write` | Full release pipeline |
| **Documentation** | `repo` | Documentation updates |

### Repository Restrictions

For enhanced security, you can restrict PATs to specific repositories:

```bash
# Limit to single repository
./scripts/github-pat-manager.sh create \
  -d "WasmWizard only access" \
  -r "botzrDev/WasmWizard"

# Limit to multiple repositories  
./scripts/github-pat-manager.sh create \
  -d "Multiple repo access" \
  -r "botzrDev/WasmWizard,botzrDev/OtherRepo"
```

## 🔒 Security Best Practices

### Token Handling

1. **Never Log Tokens**: The script is designed to never log actual PAT values
2. **Use Environment Variables**: Pass tokens through secure environment variables
3. **Mask in CI/CD**: Use `::add-mask::` in GitHub Actions to hide tokens
4. **Immediate Usage**: Use tokens immediately after creation
5. **Automatic Cleanup**: Always clean up tokens after use

### Scope Limitation

1. **Minimal Permissions**: Request only the minimum scopes needed
2. **Repository Restrictions**: Limit access to specific repositories when possible
3. **Time Limits**: Use the shortest reasonable expiration time
4. **Regular Audits**: Review and clean up tokens regularly

### Monitoring

1. **Audit Logs**: Review PAT creation and usage logs regularly
2. **Expiration Tracking**: Monitor token expiration dates
3. **Automated Alerts**: Set up alerts for unusual token usage
4. **Access Reviews**: Regularly review who has access to create tokens

## 🚨 Troubleshooting

### Common Issues

#### Authentication Errors

```bash
Error: GitHub CLI not authenticated
```

**Solution**: Run `gh auth login` and follow the authentication flow.

#### Permission Denied

```bash
Error: Failed to create GitHub PAT
```

**Solutions**:
- Ensure your GitHub account has necessary permissions
- Check if organization policies restrict PAT creation
- Verify the requested scopes are available to your account

#### Missing Dependencies

```bash
Error: Missing required dependencies: gh (GitHub CLI)
```

**Solution**: Install missing dependencies:
```bash
# GitHub CLI
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list
sudo apt update && sudo apt install gh

# jq
sudo apt install jq
```

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
export LOG_FILE="/tmp/pat-debug.log"
./scripts/github-pat-manager.sh create -d "Debug test" -s "repo" -e 1
cat /tmp/pat-debug.log
```

## 📋 Use Cases

### 1. CI/CD Deployment

Create short-lived PATs for deployment workflows:

```bash
# 1-hour deployment PAT
PAT=$(./scripts/github-pat-manager.sh create \
  -d "Production deployment $(date '+%Y-%m-%d')" \
  -s "repo,packages:write" \
  -e 1)

# Use for deployment
export GITHUB_TOKEN="$PAT"
./deploy.sh

# Clean up immediately
./scripts/github-pat-manager.sh cleanup
```

### 2. Contributor Onboarding

Provide temporary access for new contributors:

```bash
# 8-hour contributor access
./scripts/github-pat-manager.sh create \
  -d "New contributor: john.doe" \
  -s "repo" \
  -e 8 \
  -r "botzrDev/WasmWizard"
```

### 3. Scheduled Maintenance

Automated maintenance tasks with time-limited access:

```bash
# Daily backup PAT (30 minutes)
./scripts/github-pat-manager.sh create \
  -d "Daily backup $(date '+%Y-%m-%d')" \
  -s "repo" \
  -e 0.5
```

### 4. Emergency Access

Quick temporary access for urgent fixes:

```bash
# 2-hour emergency access
./scripts/github-pat-manager.sh create \
  -d "Emergency fix - Issue #123" \
  -s "repo,packages:write" \
  -e 2
```

## 🔮 Advanced Usage

### Custom PAT Store Location

```bash
export PAT_STORE_FILE="/secure/location/.wasmwiz-pats.json"
./scripts/github-pat-manager.sh create -d "Custom location test"
```

### Integration with Other Tools

#### Docker

```dockerfile
FROM ubuntu:latest
RUN apt-get update && apt-get install -y curl jq
# Install GitHub CLI...
COPY scripts/github-pat-manager.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/github-pat-manager.sh
```

#### Kubernetes

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: pat-cleanup
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: pat-cleanup
            image: wasmwizard/pat-manager:latest
            command: ["/usr/local/bin/github-pat-manager.sh", "cleanup"]
            env:
            - name: GITHUB_TOKEN
              valueFrom:
                secretKeyRef:
                  name: github-credentials
                  key: token
```

## 📞 Support

### Getting Help

- **Issues**: [Report bugs or request features](https://github.com/botzrDev/WasmWizard/issues)
- **Discussions**: [Ask questions](https://github.com/botzrDev/WasmWizard/discussions)
- **Security**: [Report security issues](./SECURITY.md)

### Contributing

Contributions to the PAT automation system are welcome! Please:

1. Follow the existing code style and security practices
2. Add comprehensive tests for new features
3. Update documentation for any changes
4. Ensure no sensitive information is logged or stored

## 📄 License

This PAT automation system is part of WasmWizard and is licensed under the Apache License 2.0. See [LICENSE](../LICENSE) for details.