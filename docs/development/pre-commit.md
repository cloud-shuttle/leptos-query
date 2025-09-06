# Pre-commit Hooks

This document describes the pre-commit hooks configured for the leptos-query project.

## Overview

Pre-commit hooks automatically run code quality checks and formatting before each commit. This ensures consistent code quality and prevents common issues from being committed.

## Installation

### Automatic Installation

Run the installation script:

```bash
./scripts/install-pre-commit.sh
```

### Manual Installation

1. Install pre-commit:
   ```bash
   pip install pre-commit
   # or
   brew install pre-commit
   ```

2. Install additional tools:
   ```bash
   # For security auditing
   cargo install cargo-audit
   
   # For markdown linting
   npm install -g markdownlint-cli
   ```

3. Install hooks:
   ```bash
   pre-commit install
   pre-commit install --hook-type pre-push
   ```

## Configured Hooks

### Rust Hooks

#### `rustfmt`
- **Purpose**: Format Rust code according to standard conventions
- **Command**: `cargo fmt --all`
- **Files**: `*.rs`

#### `clippy`
- **Purpose**: Lint Rust code for common issues and improvements
- **Command**: `cargo clippy --all-targets --all-features`
- **Files**: `*.rs`
- **Behavior**: Treats warnings as errors

#### `cargo check`
- **Purpose**: Verify code compiles without errors
- **Command**: `cargo check --all-targets --all-features`
- **Files**: `*.rs`

#### `cargo test`
- **Purpose**: Run all tests to ensure nothing is broken
- **Command**: `cargo test --quiet`
- **Files**: `*.rs`

#### `cargo audit`
- **Purpose**: Check for known security vulnerabilities
- **Command**: `cargo audit`
- **Files**: `Cargo.toml`, `Cargo.lock`

### File Quality Hooks

#### `trailing-whitespace`
- **Purpose**: Remove trailing whitespace
- **Files**: All text files

#### `end-of-file-fixer`
- **Purpose**: Ensure files end with a newline
- **Files**: All text files

#### `mixed-line-ending`
- **Purpose**: Ensure consistent line endings (LF)
- **Files**: All text files

### Validation Hooks

#### `check-yaml`
- **Purpose**: Validate YAML syntax
- **Files**: `*.yaml`, `*.yml`

#### `check-json`
- **Purpose**: Validate JSON syntax
- **Files**: `*.json`

#### `check-toml`
- **Purpose**: Validate TOML syntax
- **Files**: `*.toml`

#### `check-merge-conflict`
- **Purpose**: Detect merge conflict markers
- **Files**: All text files

### Security Hooks

#### `check-added-large-files`
- **Purpose**: Prevent large files from being committed
- **Limit**: 1000KB per file

#### `detect-secrets`
- **Purpose**: Detect potential secrets in code
- **Files**: All files
- **Baseline**: `.secrets.baseline`

### Formatting Hooks

#### `markdownlint`
- **Purpose**: Lint and format markdown files
- **Files**: `*.md`
- **Behavior**: Auto-fixes issues where possible

#### `prettier`
- **Purpose**: Format YAML files
- **Files**: `*.yaml`, `*.yml`
- **Config**: 2-space indentation

## Usage

### Automatic Usage

Hooks run automatically on every commit:

```bash
git commit -m "Add new feature"
# Pre-commit hooks run automatically
```

### Manual Usage

Run hooks on all files:

```bash
pre-commit run --all-files
```

Run specific hook:

```bash
pre-commit run rustfmt
pre-commit run clippy
```

Run hooks on specific files:

```bash
pre-commit run --files src/lib.rs
```

### Skipping Hooks

Skip hooks for a specific commit:

```bash
git commit --no-verify -m "Emergency fix"
```

**Note**: Only use `--no-verify` in emergencies. Regular commits should pass all hooks.

## Configuration

### `.pre-commit-config.yaml`

The main configuration file defines all hooks and their settings.

### Customizing Hooks

To modify hook behavior, edit `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/doublify/pre-commit-rust
    rev: v1.0
    hooks:
      - id: clippy
        args: [--manifest-path, Cargo.toml, --, -D, warnings, -A, clippy::too_many_arguments]
```

### Adding New Hooks

Add new hooks to `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.4.0
    hooks:
      - id: your-new-hook
        name: Your New Hook
        description: Description of what it does
```

## Troubleshooting

### Common Issues

#### Hook Fails on Commit

1. **Check the error message**:
   ```bash
   pre-commit run --all-files
   ```

2. **Fix the issues** and commit again

3. **If it's a false positive**, you can skip with `--no-verify`

#### Pre-commit Not Installed

```bash
pip install pre-commit
pre-commit install
```

#### Clippy Warnings

Fix clippy warnings or add `#[allow(clippy::warning_name)]` for false positives.

#### Formatting Issues

Run formatters manually:

```bash
cargo fmt
pre-commit run prettier --all-files
```

#### Security Audit Failures

Update dependencies or add exceptions to `Cargo.toml`:

```toml
[audit.allow]
CVE-2023-1234 = "Not applicable to our use case"
```

### Performance Issues

#### Slow Hooks

- Use `--hook-stage manual` for slow hooks
- Exclude large files from certain hooks
- Use `--all-files` sparingly

#### Memory Issues

- Increase available memory
- Exclude large files from hooks
- Use `--hook-stage manual` for memory-intensive hooks

## Best Practices

### Commit Messages

Write clear, descriptive commit messages:

```bash
git commit -m "feat: add caching support for queries

- Implement TTL-based cache invalidation
- Add cache size limits
- Update documentation"
```

### Regular Maintenance

1. **Update hooks regularly**:
   ```bash
   pre-commit autoupdate
   ```

2. **Run full checks before pushing**:
   ```bash
   pre-commit run --all-files
   cargo test
   cargo clippy
   ```

3. **Keep dependencies updated**:
   ```bash
   cargo update
   cargo audit
   ```

### Team Workflow

1. **Install hooks** when joining the project
2. **Don't skip hooks** unless absolutely necessary
3. **Fix issues** rather than ignoring them
4. **Update hooks** when adding new tools

## Integration with CI/CD

Pre-commit hooks complement CI/CD pipelines:

- **Pre-commit**: Fast, local checks
- **CI/CD**: Comprehensive, cross-platform checks

Both should pass for a successful build.

## Resources

- [Pre-commit Documentation](https://pre-commit.com/)
- [Rust Pre-commit Hooks](https://github.com/doublify/pre-commit-rust)
- [Pre-commit Hooks](https://github.com/pre-commit/pre-commit-hooks)
- [Markdownlint](https://github.com/igorshubovych/markdownlint-cli)
- [Cargo Audit](https://github.com/RustSec/cargo-audit)
