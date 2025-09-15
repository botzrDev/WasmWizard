# Contributing to Wasm Wizard

Thank you for your interest in contributing to Wasm Wizard! This document provides guidelines and information for contributors. Whether you're fixing bugs, adding features, improving documentation, or helping with testing, your contributions are welcome and appreciated.

## Table of Contents

- [Getting Started](#getting-started)
- [Contributing Process](#contributing-process)
- [Code Standards](#code-standards)
- [Issue Guidelines](#issue-guidelines)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Security](#security)

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- **Rust**: 1.81+ ([Install Rust](https://rustup.rs/))
- **Docker & Docker Compose**: For running the development environment
- **PostgreSQL**: 15+ (handled by Docker)
- **Redis**: 7+ (handled by Docker)
- **Git**: For version control

### Development Setup

1. **Fork and Clone the Repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/WasmWizard.git
   cd WasmWizard
   ```

2. **Start the Development Environment**
   ```bash
   cd wasm-wizard
   docker-compose -f docker-compose.dev.yml up -d
   ```

3. **Set Up Environment Variables**
   ```bash
   cp .env.development .env
   ```

4. **Run Database Migrations**
   ```bash
   sqlx migrate run
   ```

5. **Build and Run the Application**
   ```bash
   cargo run
   ```

6. **Verify Setup**
   - Frontend: http://localhost:8080
   - Health check: http://localhost:8080/health
   - Metrics: http://localhost:8080/metrics

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test --verbose

# Run specific test
cargo test test_name

# Run integration tests
cargo test --test integration_tests

# Run tests without output capture
cargo test -- --nocapture
```

## Contributing Process

### 1. Choose an Issue

- Check the [Issues](https://github.com/botzrDev/WasmWizard/issues) page for open tasks
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to indicate you're working on it

### 2. Create a Branch

Create a feature branch from `master`:

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 3. Make Changes

- Write clear, focused commits
- Follow the [commit message conventions](#commit-message-conventions)
- Test your changes thoroughly
- Update documentation if needed

### 4. Submit a Pull Request

1. **Push your branch** to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a Pull Request** on GitHub:
   - Use a descriptive title
   - Fill out the PR template
   - Reference any related issues
   - Provide a clear description of changes

3. **Code Review Process**:
   - Maintainers will review your PR
   - Address any feedback or requested changes
   - Once approved, your PR will be merged

### Commit Message Conventions

Follow conventional commit format:

```bash
type(scope): description

[optional body]

[optional footer]
```

**Types**:
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples**:
```bash
feat(auth): add JWT token validation
fix(api): resolve memory leak in WASM execution
docs(readme): update installation instructions
test(handlers): add unit tests for error handling
```

## Code Standards

### Rust Best Practices

- **Code Formatting**: Use `cargo fmt` for consistent formatting
- **Linting**: Run `cargo clippy` and fix all warnings
- **Documentation**: Document public APIs with `///` comments
- **Error Handling**: Use `Result` and `Option` appropriately
- **Performance**: Be mindful of allocations and performance implications

### Code Quality Checks

Before submitting a PR, run these commands:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Run security audit
cargo audit

# Run tests
cargo test --all-features
```

### Testing Requirements

- **Unit Tests**: Required for all new functions and significant changes
- **Integration Tests**: Required for API endpoints and database operations
- **Test Coverage**: Aim for high coverage, especially for critical paths
- **Edge Cases**: Test error conditions and edge cases

### Security Considerations

- **Input Validation**: Always validate and sanitize user inputs
- **Authentication**: Use proper JWT validation and role-based access
- **WASM Security**: Respect sandboxing limits and security boundaries
- **Dependencies**: Keep dependencies updated and audit for vulnerabilities
- **Secrets**: Never commit secrets or sensitive configuration

## Issue Guidelines

### Reporting Bugs

When reporting bugs, please provide:

1. **Clear Title**: Summarize the issue concisely
2. **Environment**: Rust version, OS, browser (if applicable)
3. **Steps to Reproduce**: Detailed steps to reproduce the issue
4. **Expected Behavior**: What should happen
5. **Actual Behavior**: What actually happens
6. **Screenshots/Logs**: If applicable
7. **Additional Context**: Any other relevant information

### Feature Requests

For feature requests, please provide:

1. **Clear Description**: What feature you want and why
2. **Use Case**: How this feature would be used
3. **Alternatives**: Any alternative solutions you've considered
4. **Mockups**: If applicable, provide mockups or examples

### Using Issue Templates

We use GitHub issue templates to ensure consistent reporting. Please use the appropriate template when creating issues:

- **Bug Report**: For reporting bugs and errors
- **Feature Request**: For proposing new features
- **Documentation**: For documentation improvements
- **Security Issue**: For reporting security vulnerabilities (see [Security](#security))

## Development Workflow

### Daily Development

1. **Pull latest changes**:
   ```bash
   git pull origin master
   ```

2. **Create feature branch**:
   ```bash
   git checkout -b feature/your-feature
   ```

3. **Make changes and commit**:
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

4. **Push and create PR**:
   ```bash
   git push origin feature/your-feature
   ```

### Code Review Guidelines

**For Contributors**:
- Be open to feedback and suggestions
- Explain your implementation decisions
- Update your PR based on review comments
- Mark resolved conversations as resolved

**For Reviewers**:
- Be constructive and respectful
- Explain reasoning for requested changes
- Focus on code quality and maintainability
- Acknowledge good work

### CI/CD Pipeline

Our CI/CD pipeline runs automatically on all PRs:

- **Build**: Compiles the project
- **Test**: Runs all tests
- **Lint**: Checks code quality with Clippy
- **Security**: Runs security audits
- **Format**: Verifies code formatting

All checks must pass before merging.

## Testing

### Running Tests Locally

```bash
# All tests
cargo test

# With coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html

# Integration tests
cargo test --test integration_tests

# Load testing
./scripts/load_test.sh
```

### Writing Tests

- **Unit Tests**: Place in the same file as the code being tested
- **Integration Tests**: Place in `tests/` directory
- **Test Naming**: Use descriptive names that explain what is being tested
- **Test Organization**: Group related tests with `#[cfg(test)]` modules

### Test Coverage

We aim for high test coverage. Focus on:
- Happy path scenarios
- Error conditions
- Edge cases
- Security boundaries

## Security

### Reporting Security Issues

If you discover a security vulnerability, please:

1. **DO NOT** create a public GitHub issue
2. **DO NOT** discuss it publicly
3. Email security concerns to: security@wasmwizard.dev
4. Provide detailed information about the vulnerability

### Security Best Practices

- **Dependencies**: Keep all dependencies updated
- **Secrets**: Never commit secrets to version control
- **Input Validation**: Always validate and sanitize inputs
- **Authentication**: Use secure authentication mechanisms
- **Authorization**: Implement proper access controls
- **Logging**: Be careful not to log sensitive information

## Getting Help

- **Documentation**: Check the [docs/](docs/) directory
- **Issues**: Search existing issues for similar problems
- **Discussions**: Use GitHub Discussions for questions
- **Code of Conduct**: Review our [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

## Recognition

Contributors are recognized through:
- GitHub contributor statistics
- Mention in release notes
- Attribution in documentation
- Community recognition

Thank you for contributing to Wasm Wizard! Your efforts help make this project better for everyone. 🚀