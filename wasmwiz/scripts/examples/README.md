# 📚 GitHub PAT Automation Examples

This directory contains practical examples demonstrating how to use the GitHub Personal Access Token (PAT) automation system in various scenarios.

## 🚀 Available Examples

### 1. **deployment-example.sh** - CI/CD Deployment
Demonstrates how to use temporary PATs for secure deployment workflows.

**Usage:**
```bash
./deployment-example.sh staging 1      # Deploy to staging with 1-hour PAT
./deployment-example.sh production 0.5 # Deploy to production with 30-minute PAT
```

**Features:**
- Automatic PAT creation and cleanup
- Deployment pipeline simulation
- Security best practices
- Error handling and logging

### 2. **contributor-access.sh** - Contributor Management
Shows how to provide temporary repository access to contributors with different permission levels.

**Usage:**
```bash
# Grant contributor access for 8 hours
./contributor-access.sh grant -n "john.doe" -t "contributor" -d 8

# Grant reviewer access for 24 hours  
./contributor-access.sh grant -n "jane.smith" -t "reviewer" -d 24

# List active contributor PATs
./contributor-access.sh list

# Revoke specific access
./contributor-access.sh revoke -i 12345
```

**Access Types:**
- `read-only`: Read-only repository access
- `contributor`: Standard read/write access
- `reviewer`: Includes PR review capabilities  
- `maintainer`: Advanced repository management

## 🔧 Prerequisites

Before running the examples, ensure you have:

1. **GitHub CLI**: Install and authenticate with `gh auth login`
2. **jq**: JSON processing tool
3. **PAT Manager**: The main `github-pat-manager.sh` script

## 🛡️ Security Features

All examples follow security best practices:

- **No Token Logging**: PAT values are never written to logs
- **Automatic Cleanup**: Expired tokens are cleaned up automatically
- **Minimal Scopes**: Request only the minimum required permissions
- **Time Limits**: Use short expiration times to limit exposure
- **Audit Trail**: Comprehensive logging of PAT lifecycle

## 📋 Example Workflows

### Quick Deployment
```bash
# Create a PAT for deployment
PAT=$(../github-pat-manager.sh create -d "Deploy $(date '+%Y-%m-%d')" -s "repo" -e 1)

# Use PAT for git operations
export GITHUB_TOKEN="$PAT"
git clone https://x-access-token:$PAT@github.com/botzrDev/WasmWizard.git

# Deploy application
./deploy.sh

# Clean up expired PATs
../github-pat-manager.sh cleanup
```

### Contributor Onboarding
```bash
# Grant temporary access to new contributor
./contributor-access.sh grant \
  -n "new.contributor" \
  -t "contributor" \
  -d 8 \
  -r "botzrDev/WasmWizard"

# Instructions are automatically generated for the contributor
# Share the generated instructions file securely
```

### Emergency Access
```bash
# Quick access for urgent fixes
PAT=$(../github-pat-manager.sh create \
  -d "Emergency fix - Issue #123" \
  -s "repo,packages:write" \
  -e 2)

# Use for emergency operations...
# Automatic cleanup after 2 hours
```

## 🧪 Testing Examples

All examples include validation and error handling. Test them safely:

```bash
# Test deployment example (dry run)
./deployment-example.sh staging 0.1  # 6-minute test PAT

# Test contributor access (minimal duration)
./contributor-access.sh grant -n "test.user" -t "read-only" -d 0.5
```

## 🆘 Troubleshooting

### Common Issues

**"PAT manager script not found"**
- Ensure you're running from the correct directory
- Check that `../github-pat-manager.sh` exists and is executable

**"GitHub CLI not authenticated"**
- Run `gh auth login` and complete authentication
- Verify with `gh auth status`

**"Failed to create PAT"**
- Check GitHub account permissions
- Verify organization policies allow PAT creation
- Ensure requested scopes are available

### Getting Help

- Review the main [PAT_AUTOMATION.md](../PAT_AUTOMATION.md) documentation
- Check the troubleshooting section in the main guide
- Open an issue on GitHub if you encounter problems

## 🤝 Contributing

To add new examples:

1. Follow the existing code style and structure
2. Include comprehensive error handling
3. Add security safeguards (no token logging)
4. Update this README with example documentation
5. Test thoroughly before submitting

## 📄 License

These examples are part of WasmWizard and are licensed under Apache 2.0.