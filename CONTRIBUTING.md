# Contributing to Aevum

Thank you for your interest in contributing to Aevum! This document provides guidelines for contributing to the project.

## Code of Conduct

Be respectful, inclusive, and professional in all interactions.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/yourusername/aevum/issues)
2. If not, create a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - System information (OS, Rust version, etc.)
   - Relevant logs or error messages

### Suggesting Features

1. Open an issue with the `enhancement` label
2. Describe the feature and its use case
3. Explain why it would be valuable
4. Consider implementation approaches

### Pull Requests

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/my-feature`
3. **Make your changes**:
   - Follow the coding standards below
   - Add tests for new functionality
   - Update documentation as needed
4. **Test your changes**:
   ```bash
   cargo test --workspace
   cd coordinator && go test ./...
   cd ui && npm test
   ```
5. **Commit with clear messages**:
   ```
   feat: Add support for Ruby tracing
   fix: Resolve race condition in coordinator
   docs: Update installation instructions
   ```
6. **Push to your fork**: `git push origin feature/my-feature`
7. **Open a Pull Request**

## Coding Standards

### Rust

- Follow `rustfmt` formatting: `cargo fmt`
- Pass `clippy` lints: `cargo clippy`
- Write tests for new functionality
- Document public APIs with doc comments
- Use meaningful variable names
- Prefer explicit error handling over `.unwrap()`

### Go

- Follow `gofmt` formatting
- Use `golint` for linting
- Write table-driven tests
- Document exported functions
- Handle errors explicitly

### TypeScript

- Follow ESLint rules
- Use TypeScript strict mode
- Write JSDoc comments for complex functions
- Prefer functional components with hooks
- Use meaningful component and variable names

## Project Structure

```
Aevum/
├── trace-engine/      # Core Rust trace engine
├── replay-engine/     # Deterministic replay
├── cli/               # Command-line interface
├── agents/            # Language-specific agents
├── coordinator/       # Go distributed coordinator
├── ui/                # React UI
└── docs/              # Documentation
```

## Adding a New Language Agent

1. Create directory: `agents/your-language-agent/`
2. Implement the agent following existing patterns
3. Add event streaming to coordinator
4. Write tests and examples
5. Update documentation

Required functionality:
- Function call/return capture
- Thread/goroutine/async tracking
- Event serialization to JSON
- Connection to coordinator
- Hot attach/detach support

## Testing

### Unit Tests

```bash
# Rust
cargo test --workspace

# Go
cd coordinator && go test ./...

# TypeScript
cd ui && npm test
```

### Integration Tests

```bash
./scripts/integration-tests.sh
```

### Benchmarks

```bash
cargo bench -p trace-engine
```

## Documentation

- Update README.md for user-facing changes
- Update DEVELOPMENT.md for developer changes
- Add inline code comments for complex logic
- Create examples for new features

## Release Process

1. Update version numbers
2. Update CHANGELOG.md
3. Create git tag: `git tag v0.2.0`
4. Push tag: `git push origin v0.2.0`
5. Create GitHub release with notes

## Questions?

Open an issue or reach out to maintainers.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
