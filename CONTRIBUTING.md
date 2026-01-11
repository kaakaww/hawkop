# Contributing to HawkOp

Thank you for your interest in contributing to HawkOp! This guide covers development setup, testing, and the release process.

## Table of Contents

- [Development Setup](#development-setup)
- [Build System](#build-system)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [Continuous Integration](#continuous-integration)
- [Release Process](#release-process)
- [Pull Request Guidelines](#pull-request-guidelines)

## Development Setup

### Prerequisites

- **Rust**: Latest stable (2024 edition)
- **Cargo**: Latest version
- **Make**: For build automation
- **Git**: For version control

### Initial Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/kaakaww/hawkop.git
   cd hawkop
   ```

2. Build the project:
   ```bash
   make build
   ```

3. Run the CLI:
   ```bash
   make run
   # or
   cargo run -- --help
   ```

## Build System

HawkOp uses a **Makefile** for build automation. Run `make help` to see all available targets.

### Common Development Commands

```bash
make build           # Build debug binary
make run             # Run in debug mode
make test            # Run all tests
make fmt             # Format code
make lint            # Run clippy lints
make pre-commit      # Run all checks (format, lint, test)
make clean           # Remove build artifacts
```

### Pre-Commit Checks

**Always run before committing:**

```bash
make pre-commit
```

This command:
1. Formats your code (`cargo fmt`)
2. Checks formatting (`cargo fmt --check`)
3. Runs lints (`cargo clippy -- -D warnings`)
4. Runs all tests (`cargo test`)

### Release Building

```bash
make build-release   # Build optimized binary
make build-all       # Build for all 6 platforms
make dist            # Create distribution archives
make checksums       # Generate SHA256 checksums
```

### Installation

```bash
make install              # Install to /usr/local/bin
make install PREFIX=~/bin # Install to custom location
```

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Follow existing code patterns
- Add tests for new functionality
- Update documentation as needed

### 3. Run Pre-Commit Checks

```bash
make pre-commit
```

### 4. Commit Your Changes

```bash
git add .
git commit -m "feat: add your feature description"
```

**Commit Message Prefixes:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Adding or updating tests
- `refactor:` - Code refactoring
- `chore:` - Maintenance tasks

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a PR on GitHub.

## Testing

### Running Tests

```bash
# All tests
make test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_name

# Specific module
cargo test module_name

# Integration tests only
cargo test --test cli
```

### Test Organization

- **Unit tests**: In the same file as the code, within `#[cfg(test)]` modules
- **Integration tests**: In `tests/` directory
- **Test fixtures**: In `src/client/fixtures.rs` for building test data

### Test Dependencies

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;  // CLI testing
    use mockito;              // HTTP mocking
    use tempfile;             // Temporary files

    // For API tests, use the fixtures module:
    use crate::client::fixtures::{OrganizationBuilder, ScanResultBuilder};

    #[test]
    fn test_something() {
        let org = OrganizationBuilder::new("org-1")
            .name("Test Org")
            .build();
        // ...
    }
}
```

### Integration Tests

Integration tests in `tests/cli.rs` use `mockito` for HTTP mocking:

```rust
#[cfg_attr(not(feature = "http-tests"), ignore)]
#[test]
fn my_integration_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();

    let _mock = server
        .mock("GET", "/api/v1/endpoint")
        .with_status(200)
        .with_body(r#"{"data": []}"#)
        .create();

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("hawkop"))
        .arg("--no-cache")  // Important: prevent cache interference
        .arg("command")
        .env("HAWKOP_API_BASE_URL", server.url())
        .assert()
        .success();

    Ok(())
}
```

## Code Quality

### Formatting

```bash
make fmt             # Format code
make check-fmt       # Check without modifying
```

### Linting

```bash
make lint            # Run clippy with warnings as errors
```

**Common clippy fixes:**
- Unused variables: Remove or prefix with `_`
- Unnecessary clones: Use references
- Complex expressions: Break into smaller functions

### Code Style Guidelines

- **Modules**: Focused and single-purpose
- **Functions**: Small, doing one thing well
- **Error Handling**: Use `Result` and `?` operator
- **Documentation**: Add doc comments for public items
- **Comments**: Explain "why", not "what"

## Continuous Integration

### GitHub Actions Workflows

#### CI Workflow (`.github/workflows/ci.yml`)

**Triggers:** Push to `main`, Pull Requests

**Jobs:**
1. **Check** - Format and lint checking
2. **Test Suite** - Build and test on Linux, macOS, Windows
3. **Security Audit** - Check for vulnerable dependencies

#### Release Workflow (`.github/workflows/release.yml`)

**Triggers:** Push of version tags (e.g., `v0.3.0`)

**Builds for 6 platforms:**
- Linux: x86_64, ARM64
- macOS: Intel, Apple Silicon
- Windows: x86_64, ARM64

**Creates:**
- Distribution archives (`.tar.gz`, `.zip`)
- SHA256 checksums
- GitHub Release with artifacts

### Local CI Simulation

```bash
# What CI runs
make pre-commit

# Security audit (optional)
cargo install cargo-audit
cargo audit
```

## Release Process

HawkOp uses an interactive release wizard.

### Prerequisites

- On `main` branch with clean working directory
- CHANGELOG.md has content in `[Unreleased]` section

### Creating a Release

1. **Run the Release Wizard**
   ```bash
   make release
   ```

2. **Follow Interactive Prompts**
   - Choose version bump type (patch/minor/major)
   - Review changelog content
   - Confirm pre-flight checks pass

3. **What the Wizard Does**
   - Updates version in `Cargo.toml`
   - Promotes `[Unreleased]` changelog to versioned section
   - Creates commit and tag

4. **Push to GitHub**
   ```bash
   git push origin main
   git push origin vX.Y.Z
   ```

5. **Verify Release**
   - Check [GitHub Actions](https://github.com/kaakaww/hawkop/actions)
   - Verify [Releases page](https://github.com/kaakaww/hawkop/releases)

### Changelog Management

Follow [Keep a Changelog](https://keepachangelog.com/) format.

**During development**, add entries to `[Unreleased]`:

```markdown
## [Unreleased]

### Added
- New feature X

### Fixed
- Bug Y
```

**Preview changelog:**

```bash
make changelog-preview
```

## Pull Request Guidelines

### Before Submitting

- [ ] Run `make pre-commit` - all checks pass
- [ ] Add tests for new functionality
- [ ] Update documentation if needed
- [ ] Write clear commit messages
- [ ] Rebase on latest `main`

### PR Description

Include:
- **What**: Brief description of changes
- **Why**: Motivation and context
- **How**: Implementation approach
- **Testing**: How you tested the changes

### Review Process

1. PR submitted
2. CI checks run automatically
3. Maintainers review code
4. Address feedback
5. Merge when approved and CI passes

## Getting Help

- **Issues**: https://github.com/kaakaww/hawkop/issues
- **Documentation**: https://docs.stackhawk.com/
- **Technical Planning**: [PLANNING.md](PLANNING.md)

## License

By contributing to HawkOp, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to HawkOp!**
