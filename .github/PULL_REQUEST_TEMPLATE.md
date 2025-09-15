## 📋 Pull Request Checklist

Thank you for contributing to Wasm Wizard! Please ensure your PR meets the following requirements:

### 🔧 Code Quality
- [ ] **Code Formatting**: Run `cargo fmt` and ensure no formatting issues
- [ ] **Linting**: Run `cargo clippy` and fix all warnings
- [ ] **Security Audit**: Run `cargo audit` and address any vulnerabilities
- [ ] **Dependencies**: Update `Cargo.lock` if dependencies changed
- [ ] **Documentation**: Update relevant documentation and comments

### 🧪 Testing
- [ ] **Unit Tests**: Add or update unit tests for new functionality
- [ ] **Integration Tests**: Add integration tests if applicable
- [ ] **Test Coverage**: Ensure adequate test coverage (>80% preferred)
- [ ] **Manual Testing**: Test the changes manually
- [ ] **Edge Cases**: Test error conditions and edge cases

### 📚 Documentation
- [ ] **README**: Update README.md if user-facing changes
- [ ] **API Docs**: Update API documentation for new endpoints
- [ ] **Code Comments**: Add comments for complex logic
- [ ] **Changelog**: Update CHANGELOG.md with changes
- [ ] **Migration Guide**: Add migration guide for breaking changes

### 🔒 Security
- [ ] **Security Review**: Review code for security implications
- [ ] **Input Validation**: Ensure proper input validation and sanitization
- [ ] **Authentication**: Verify authentication/authorization if applicable
- [ ] **Secrets**: No secrets or sensitive data committed
- [ ] **Dependencies**: Check for known vulnerabilities in dependencies

### 🚀 Performance
- [ ] **Performance Impact**: Consider performance implications
- [ ] **Resource Usage**: Check memory and CPU usage
- [ ] **Scalability**: Ensure changes scale appropriately
- [ ] **Benchmarks**: Add benchmarks for performance-critical code

### 🐳 Infrastructure
- [ ] **Docker**: Update Dockerfile if needed
- [ ] **Kubernetes**: Update K8s manifests if needed
- [ ] **CI/CD**: Ensure CI/CD pipelines still pass
- [ ] **Database**: Update migrations if database schema changed
- [ ] **Environment**: Test in development environment

## 📝 Description

**What does this PR do?**
Provide a clear and concise description of the changes.

**Why are these changes needed?**
Explain the problem this PR solves or the feature it implements.

**How were these changes tested?**
Describe the testing approach and any relevant test results.

## 🔗 Related Issues

Closes # (issue number)
Related to # (issue number)

## 📋 Type of Change

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📚 Documentation update
- [ ] 🔧 Refactoring (no functional changes)
- [ ] 🧪 Tests (adding or updating tests)
- [ ] 🔒 Security (security-related changes)
- [ ] 🚀 Performance (performance-related changes)
- [ ] 🏗️ Infrastructure (deployment, CI/CD, etc.)

## 📊 Impact Assessment

**Breaking Changes**: Does this introduce breaking changes?
- [ ] Yes (explain below)
- [ ] No

**Migration Required**: Do users need to take action to migrate?
- [ ] Yes (explain below)
- [ ] No

**Performance Impact**: Any performance implications?
- [ ] Positive impact
- [ ] Negative impact (explain below)
- [ ] No significant impact

## 🎯 Checklist Verification

- [ ] I have read the [Contributing Guidelines](CONTRIBUTING.md)
- [ ] I have followed the [Code of Conduct](CODE_OF_CONDUCT.md)
- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Any dependent changes have been merged and published in downstream modules

## 📞 Additional Notes

Add any additional notes, considerations, or questions for reviewers.

---

**By submitting this pull request, I confirm that:**
- My contribution is made under the terms of the project's license
- I have the right to submit this contribution
- This contribution does not infringe on any third-party rights
- I understand and agree to the project's [Code of Conduct](CODE_OF_CONDUCT.md)