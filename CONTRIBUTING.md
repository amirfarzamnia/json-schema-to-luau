# Contributing to json-schema-to-luau

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/amirfarzamnia/json-schema-to-luau.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Run formatting: `cargo fmt`
7. Run linting: `cargo clippy`
8. Commit your changes: `git commit -m "Description of your changes"`
9. Push to your fork: `git push origin feature/your-feature-name`
10. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_simple_object
```

### Running Examples

```bash
cargo run --example basic
```

### Using the CLI during development

```bash
# Run directly with cargo
cargo run -- input.json -o output.luau

# Or build and run
cargo build --release
./target/release/json-schema-to-luau input.json
```

## Code Style

This project follows standard Rust conventions:

- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Follow Rust naming conventions (snake_case for functions/variables, CamelCase for types)
- Add documentation comments for public APIs

## Testing Guidelines

- Add tests for all new features
- Ensure existing tests pass
- Include both positive and negative test cases
- Test edge cases and error conditions

## Pull Request Process

1. Update the README.md if you're adding new features
2. Add tests for new functionality
3. Ensure all tests pass
4. Update documentation as needed
5. Your PR should have a clear description of what it does

## Feature Requests and Bug Reports

- Use GitHub Issues to report bugs or request features
- Provide clear descriptions and examples
- For bugs, include steps to reproduce

## Areas for Contribution

Some areas where contributions would be particularly welcome:

- Additional JSON Schema features
- Better error messages
- Performance improvements
- Documentation improvements
- More examples
- CLI enhancements

## Questions?

Feel free to open an issue for any questions about contributing!
