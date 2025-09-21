# Contributing to RTodo

Thank you for your interest in contributing to RTodo! This document provides guidelines and information for contributors.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Guidelines](#contributing-guidelines)
- [Code Standards](#code-standards)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Issue Reporting](#issue-reporting)
- [Release Process](#release-process)

## Getting Started

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Git** - For version control
- **Text Editor/IDE** - VS Code with rust-analyzer extension recommended

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/your-username/rtodo.git
   cd rtodo
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/original-username/rtodo.git
   ```

## Development Setup

### Local Development

```bash
# Install dependencies and build
cargo build

# Run tests
cargo test

# Run with development flags
cargo run -- --help

# Install locally for testing
cargo install --path .
```

### Development Dependencies

```bash
# Install additional development tools
cargo install cargo-watch    # For file watching during development
cargo install cargo-audit    # For security auditing
cargo install clippy         # For linting (usually included with Rust)
```

### Running in Development

```bash
# Use cargo run for development
cargo run -- add "Test todo"
cargo run -- list

# Watch for changes during development
cargo watch -c -x "run -- list"
```

## Contributing Guidelines

### Types of Contributions

- **Bug Reports** - Report issues you've encountered
- **Feature Requests** - Suggest new functionality
- **Code Contributions** - Submit bug fixes or new features
- **Documentation** - Improve README, docs, or code comments
- **Testing** - Add or improve test coverage

### Contribution Workflow

1. **Check Issues First** - Look for existing issues related to your contribution
2. **Create Issue** - If none exists, create an issue describing the problem/feature
3. **Discuss** - Engage with maintainers about the approach
4. **Create Branch** - Create a feature branch for your work
5. **Make Changes** - Implement your changes with tests
6. **Submit PR** - Create a pull request with clear description

## Code Standards

### Rust Style Guidelines

- Follow [Rust's official style guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` to format code automatically
- Use `cargo clippy` to catch common mistakes
- Maximum line length: 100 characters

### Formatting

```bash
# Format code before committing
cargo fmt

# Check formatting without changing files
cargo fmt -- --check
```

### Linting

```bash
# Run clippy for linting
cargo clippy

# Run clippy with all targets
cargo clippy --all-targets --all-features
```

### Code Organization

- **One feature per file** when reasonable
- **Clear function names** that describe what they do
- **Comprehensive error handling** using `anyhow` or `Result` types
- **Unit tests** alongside the code they test

### Naming Conventions

- **Functions and variables**: `snake_case`
- **Types and traits**: `PascalCase`
- **Constants**: `UPPER_SNAKE_CASE`
- **Modules**: `snake_case`

### Documentation

- Use `///` for public API documentation
- Use `//` for implementation comments
- Document all public functions and types
- Include examples in documentation when helpful

```rust
/// Adds a new todo item to the list
///
/// # Arguments
///
/// * `title` - The title of the todo item
/// * `due_date` - Optional due date for the todo
///
/// # Examples
///
/// ```
/// let todo = add_todo("Buy groceries", None);
/// ```
pub fn add_todo(title: &str, due_date: Option<DateTime<Local>>) -> Result<Todo> {
    // Implementation here
}
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests and show coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Writing Tests

- **Unit tests** - Test individual functions and modules
- **Integration tests** - Test CLI functionality end-to-end
- **Property tests** - Test edge cases and invariants

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_todo() {
        // Arrange
        let title = "Test todo";

        // Act
        let result = add_todo(title, None);

        // Assert
        assert!(result.is_ok());
        let todo = result.unwrap();
        assert_eq!(todo.title, title);
    }
}
```

### Integration Tests

Place integration tests in `tests/` directory:

```rust
// tests/cli_tests.rs
use assert_cmd::Command;

#[test]
fn test_add_command() {
    let mut cmd = Command::cargo_bin("rtodo").unwrap();
    cmd.arg("add")
       .arg("Test todo")
       .assert()
       .success();
}
```

## Submitting Changes

### Commit Message Format

Use clear, descriptive commit messages:

```
type(scope): description

Longer description if needed

- Detail 1
- Detail 2

Fixes #123
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting, etc.)
- `refactor` - Code refactoring
- `test` - Adding or updating tests
- `chore` - Maintenance tasks

**Examples:**
```
feat(cli): add search command for filtering todos

Add new search subcommand that allows filtering todos by text content.
Supports case-insensitive matching across title, description, and category.

- Add search command with text parameter
- Implement case-insensitive text matching
- Add tests for search functionality

Fixes #45
```

### Pull Request Process

1. **Update Documentation** - Update README if needed
2. **Add Tests** - Include tests for new functionality
3. **Run Tests** - Ensure all tests pass locally
4. **Clean History** - Squash commits if needed
5. **Write Description** - Clear PR description explaining changes

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests pass locally
```

## Issue Reporting

### Bug Reports

Include the following information:

- **OS and version** (macOS, Linux, Windows)
- **Rust version** (`rustc --version`)
- **RTodo version** (`rtodo --version`)
- **Command that failed** (exact command line)
- **Expected behavior**
- **Actual behavior**
- **Error messages** (full text)
- **Steps to reproduce**

### Feature Requests

Include the following information:

- **Use case** - Why do you need this feature?
- **Proposed solution** - How should it work?
- **Alternatives** - What other solutions have you considered?
- **Examples** - Show how it would be used

## Release Process

### Versioning

RTodo follows [Semantic Versioning](https://semver.org/):

- **MAJOR** - Breaking changes
- **MINOR** - New features (backward compatible)
- **PATCH** - Bug fixes (backward compatible)

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Create release commit
5. Tag release: `git tag v1.2.3`
6. Push tags: `git push --tags`
7. Create GitHub release with notes

## Getting Help

- **GitHub Issues** - For bugs and feature requests
- **Discussions** - For questions and general discussion
- **Documentation** - Check README and inline docs first

## Recognition

Contributors will be recognized in:
- `CONTRIBUTORS.md` file
- Release notes for significant contributions
- GitHub contributors page

Thank you for contributing to RTodo!