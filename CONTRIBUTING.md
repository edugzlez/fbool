# Contributing to FBool Entanglement

Thank you for your interest in contributing to FBool Entanglement! This document provides guidelines and instructions for contributing to this project.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Running Tests](#running-tests)
- [Code Style](#code-style)
- [Pull Request Process](#pull-request-process)
- [Reporting Issues](#reporting-issues)

## Getting Started

FBool Entanglement is a Rust workspace project that provides tools for analyzing entanglement in boolean functions. Before contributing, please:

1. Read the [README.md](README.md) to understand the project's purpose and structure
2. Check existing [issues](../../issues) and [pull requests](../../pulls) to avoid duplicating work
3. For major changes, open an issue first to discuss your proposed changes

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Python 3.8+ (for Python bindings)
- C++ compiler (GCC or Clang for optimal5 crate)

### Building the Project

```bash
# Clone the repository
git clone https://github.com/edugzlez/fbool-entanglement.git
cd fbool-entanglement

# Build the entire workspace
cargo build

# Build in release mode
cargo build --release

# Build a specific crate
cargo build -p fbool
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p fbool

# Run tests with output
cargo test -- --nocapture
```

### Running the CLI

```bash
# Build and run the CLI
cargo run --bin fbool-cli -- --help

# Example: Calculate entanglement
cargo run --bin fbool-cli -- entanglement majority -n 4
```

## Project Structure

The project is organized as a Rust workspace with the following crates:

- **fbool**: Core library for boolean functions
- **fbool-entanglement**: Entanglement analysis traits and algorithms
- **fbool-frontier**: Frontier analysis for boolean functions
- **fbool-cli**: Command-line interface
- **fbool-py**: Python bindings (PyO3)
- **clique_solver**: CLIQUE problem solver
- **optimal5**: Logic gate optimizer for 5-variable functions (Rust + C++)

## Running Tests

All contributions should include appropriate tests:

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test --verbose

# Run a specific test
cargo test test_name

# Run doctests
cargo test --doc
```

## Code Style

We follow Rust's official style guidelines:

### Formatting

```bash
# Check formatting
cargo fmt --all -- --check

# Format code
cargo fmt --all
```

### Linting

```bash
# Run clippy for all targets
cargo clippy --all-targets --all-features

# Fix clippy warnings automatically (where possible)
cargo clippy --fix
```

### Documentation

- Add documentation comments (`///`) for all public items
- Include examples in documentation where appropriate
- Use `cargo doc --open` to preview documentation

Example:

````rust
/// Calculates the entanglement of a boolean function.
///
/// # Arguments
///
/// * `partition` - The partition of variables to analyze
///
/// # Returns
///
/// The entanglement value as a floating-point number.
///
/// # Examples
///
/// ```
/// use fbool_entanglement::Entanglement;
/// // Example usage here
/// ```
pub fn calculate_entanglement(&self, partition: &[bool]) -> f64 {
    // Implementation
}
````

## Pull Request Process

1. **Fork the repository** and create your branch from `main`:

   ```bash
   git checkout -b feature/my-new-feature
   ```

2. **Make your changes**:
   - Write clear, concise commit messages
   - Follow the code style guidelines
   - Add tests for new functionality
   - Update documentation as needed

3. **Ensure all tests pass**:

   ```bash
   cargo test --all
   cargo clippy --all-targets --all-features
   cargo fmt --all -- --check
   ```

4. **Update the documentation**:
   - Update README.md if adding new features
   - Add docstrings to new functions
   - Update examples if needed

5. **Create a pull request**:
   - Provide a clear description of the changes
   - Reference any related issues
   - Include any breaking changes in the description

6. **Respond to review feedback**:
   - Address reviewer comments
   - Make requested changes
   - Keep the PR focused on a single concern

## Reporting Issues

When reporting issues, please include:

1. **Description**: Clear and concise description of the problem
2. **Steps to reproduce**: Detailed steps to reproduce the issue
3. **Expected behavior**: What you expected to happen
4. **Actual behavior**: What actually happened
5. **Environment**:
   - Rust version (`rustc --version`)
   - Operating system
   - Relevant dependencies

### Example Issue

```markdown
**Description**
The entanglement calculation returns incorrect values for certain partitions.

**Steps to Reproduce**

1. Create a majority function with n=4
2. Calculate entanglement with partition [true, false, true, false]
3. Observe incorrect value

**Expected Behavior**
Should return entanglement value of approximately 2.5

**Actual Behavior**
Returns 0.0

**Environment**

- Rust version: 1.70.0
- OS: Ubuntu 22.04
- fbool-entanglement version: 0.1.0
```

## Code Review Guidelines

When reviewing pull requests, consider:

- **Correctness**: Does the code do what it's supposed to?
- **Tests**: Are there sufficient tests?
- **Performance**: Are there any obvious performance issues?
- **Documentation**: Is the code well-documented?
- **Style**: Does it follow project conventions?

## Getting Help

If you need help:

- Check the [README.md](README.md) documentation
- Look at existing code for examples
- Open an issue with the "question" label
- Reach out to maintainers

## License

By contributing to FBool Entanglement, you agree that your contributions will be licensed under the same license as the project.

## Thank You!

Your contributions make this project better. Thank you for taking the time to contribute!
