# Contributing to HawkOp

Thank you for your interest in contributing to HawkOp! This guide will help you get started with development, testing, and releasing.

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

- **Rust**: 1.70 or later (2021 edition)
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
   cargo build
   ```

3. Run the CLI:
   ```bash
   cargo run -- --help
   ```

## Build System

HawkOp uses a **Makefile** for build automation and **GitHub Actions** for CI/CD.

### Makefile Targets

Run `make help` to see all available targets. Key commands:

#### Development

```bash
make build           # Build debug binary
make run             # Run in debug mode
make test            # Run all tests
make fmt             # Format code
make lint            # Run clippy lints
```

#### Pre-Commit Checks

**Before committing**, always run:

```bash
make pre-commit
```

This command:
1. Formats your code (`cargo fmt`)
2. Checks formatting (`cargo fmt --check`)
3. Runs lints (`cargo clippy -- -D warnings`)
4. Runs all tests (`cargo test`)

#### Release Building

```bash
make build-release   # Build optimized binary
make build-all       # Build for all 6 platforms
make dist            # Create distribution archives
make checksums       # Generate SHA256 checksums
```

#### Installation

```bash
make install              # Install to /usr/local/bin
make install PREFIX=~/bin # Install to custom location
```

#### Cleanup

```bash
make clean           # Remove build artifacts
```

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Write clear, idiomatic Rust code
- Follow existing code style and patterns
- Add tests for new functionality
- Update documentation as needed

### 3. Run Pre-Commit Checks

```bash
make pre-commit
```

This ensures your code:
- Is properly formatted
- Passes all lints
- Passes all tests

### 4. Commit Your Changes

```bash
git add .
git commit -m "feat: add your feature description"
```

**Commit Message Format:**
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
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific module
cargo test module_name
```

### Test Organization

- **Unit tests**: In the same file as the code being tested
- **Integration tests**: In `tests/` directory (to be added)
- **Doc tests**: In documentation comments

### Writing Tests

Use the existing test dependencies:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command; // For CLI testing
    use mockito;             // For API mocking
    use tempfile;            // For temporary files

    #[test]
    fn test_something() {
        // Your test here
    }
}
```

## Code Quality

### Formatting

HawkOp uses `rustfmt` with default settings:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

HawkOp uses `clippy` with warnings as errors:

```bash
# Run clippy
cargo clippy -- -D warnings
```

**Fix common issues:**
- Unused variables: Remove or prefix with `_`
- Missing documentation: Add doc comments for public items
- Unnecessary clones: Use references where possible
- Complex expressions: Simplify or break into smaller functions

### Code Style Guidelines

- **Modules**: Keep focused and single-purpose
- **Functions**: Small, doing one thing well
- **Error Handling**: Use `Result` and `?` operator, avoid `panic!`
- **Documentation**: Add doc comments for public items
- **Comments**: Explain "why", not "what"

## Continuous Integration

### GitHub Actions Workflows

#### CI Workflow (`.github/workflows/ci.yml`)

**Triggers:** Push to `main`, Pull Requests

**Jobs:**
1. **Check** (Ubuntu)
   - Format checking (`cargo fmt --check`)
   - Lint checking (`cargo clippy -- -D warnings`)

2. **Test Suite** (Linux, macOS, Windows)
   - Build project
   - Run all tests

3. **Security Audit** (Ubuntu)
   - Run `cargo audit` to check for vulnerable dependencies

**All CI checks must pass before a PR can be merged.**

#### Release Workflow (`.github/workflows/release.yml`)

**Triggers:** Push of version tags (e.g., `v0.1.0`)

**Builds for 6 platforms:**
- Linux: x86_64, ARM64
- macOS: Intel (x86_64), Apple Silicon (aarch64)
- Windows: x86_64, ARM64

**Creates:**
- `.tar.gz` archives (Linux/macOS)
- `.zip` archives (Windows)
- SHA256 checksums for all artifacts
- GitHub Release with all artifacts attached

### Local CI Simulation

Before pushing, simulate CI locally:

```bash
# What CI runs
make pre-commit

# Security audit (optional)
cargo install cargo-audit
cargo audit
```

## Release Process

### Creating a New Release

1. **Update Version**

   Edit `Cargo.toml`:
   ```toml
   [package]
   version = "0.2.0"  # Update version
   ```

   Update lockfile:
   ```bash
   cargo update -p hawkop
   ```

2. **Update CHANGELOG.md**

   Document all changes since last release:
   ```markdown
   ## [0.2.0] - 2025-11-20

   ### Added
   - New feature X
   - New feature Y

   ### Fixed
   - Bug fix A
   - Bug fix B

   ### Changed
   - Updated dependency Z
   ```

3. **Run Pre-Commit Checks**

   ```bash
   make pre-commit
   ```

   Ensure everything passes.

4. **View Release Instructions**

   ```bash
   make release
   ```

   This shows the complete release guide.

5. **Commit and Tag**

   ```bash
   git add Cargo.toml Cargo.lock CHANGELOG.md
   git commit -m "chore: release v0.2.0"
   git tag v0.2.0
   ```

6. **Push to GitHub**

   ```bash
   git push origin main
   git push origin v0.2.0
   ```

7. **Wait for GitHub Actions**

   The release workflow will automatically:
   - Build for all 6 platforms
   - Create distribution archives
   - Generate checksums
   - Create GitHub Release
   - Upload all artifacts

8. **Verify Release**

   Check:
   - GitHub Actions workflow completed successfully
   - Release appears at `https://github.com/kaakaww/hawkop/releases`
   - All 6 platform binaries are attached
   - Checksums are present

### Supported Platforms

| Platform | Architecture | Target Triple |
|----------|-------------|---------------|
| Linux | x86_64 | `x86_64-unknown-linux-gnu` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` |
| macOS | Intel | `x86_64-apple-darwin` |
| macOS | Apple Silicon | `aarch64-apple-darwin` |
| Windows | x86_64 | `x86_64-pc-windows-msvc` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` |

## Pull Request Guidelines

### Before Submitting

- [ ] Run `make pre-commit` and ensure all checks pass
- [ ] Add tests for new functionality
- [ ] Update documentation if needed
- [ ] Write clear commit messages
- [ ] Rebase on latest `main` branch

### PR Description

Include:
- **What**: Brief description of changes
- **Why**: Motivation and context
- **How**: Implementation approach
- **Testing**: How you tested the changes

### Review Process

1. PR is submitted
2. CI checks run automatically
3. Maintainers review code
4. Address feedback if needed
5. PR is merged once approved and CI passes

### After Merge

- Delete your feature branch
- Pull latest `main`
- Start next feature!

## Getting Help

- **Issues**: https://github.com/kaakaww/hawkop/issues
- **Discussions**: https://github.com/kaakaww/hawkop/discussions
- **Documentation**: https://docs.stackhawk.com/

## Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build great software together.

## License

By contributing to HawkOp, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to HawkOp!** 🎉
