# Contributing to Namra

Thank you for your interest in contributing to Namra!

## Getting Started

### Prerequisites
- Rust 1.75 or later
- Git

### Development Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/namra
   cd namra
   ```
3. Build the project:
   ```bash
   cargo build
   ```
4. Run tests:
   ```bash
   cargo test --workspace
   ```

## How to Contribute

### Reporting Bugs

- Use [GitHub Issues](https://github.com/namra-agents/namra/issues)
- Include:
  - Operating system and version
  - Rust version (`rustc --version`)
  - Steps to reproduce the issue
  - Expected vs actual behavior
  - Relevant logs or error messages

### Suggesting Features

- Open a [GitHub Discussion](https://github.com/namra-agents/namra/discussions) first
- Explain the use case and why it would benefit the project
- Be open to feedback and alternative approaches

### Pull Requests

1. Create a feature branch:
   ```bash
   git checkout -b feature/my-feature
   ```

2. Make your changes and add tests

3. Ensure code quality:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --workspace
   ```

4. Commit with a descriptive message:
   ```bash
   git commit -m "Add feature: description of what was added"
   ```

5. Push and open a PR against `main`

### PR Guidelines

- Keep PRs focused on a single change
- Update documentation if needed
- Add tests for new functionality
- Ensure CI passes before requesting review

## Code Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Use meaningful variable and function names
- Add comments for complex logic

## Project Structure

```
namra/
├── namra-core/
│   ├── namra-config/    # Configuration parsing
│   ├── namra-llm/       # LLM provider adapters
│   ├── namra-tools/     # Built-in tools
│   ├── namra-memory/    # Memory implementations
│   ├── namra-middleware/# Middleware system
│   ├── namra-plugin/    # Plugin system
│   ├── namra-api/       # HTTP/gRPC API
│   ├── namra-runtime/   # Core runtime
│   └── namra-cli/       # CLI binary
├── examples/            # Example agent configs
└── docs/                # Documentation
```

## Testing

- Unit tests go in the same file as the code (`#[cfg(test)]`)
- Integration tests go in `tests/` directory
- Run specific tests: `cargo test test_name`
- Run tests with output: `cargo test -- --nocapture`

## License

By contributing, you agree that your contributions will be licensed under the Apache-2.0 License.
