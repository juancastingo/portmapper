# Contributing to PortMapper

Thank you for contributing to `portmapper`!

## Building and Testing

1. Clone repository:
   ```bash
   git clone https://github.com/juancastingo/portmapper.git
   cd portmapper
   ```

2. Run test suite:
   ```bash
   cargo test
   ```

3. Run linter and formatting checks:
   ```bash
   cargo clippy --all-targets -- -D warnings
   cargo fmt -- --check
   ```

## Pull Request Guidelines

- Ensure tests pass with zero clippy warnings.
- Keep the tool fast and focused on socket mapping and snapshot diffing.
