# Contributing to Leptos Studio

Thank you for your interest in contributing to Leptos Studio! 🎉

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Style Guidelines](#style-guidelines)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Testing](#testing)
- [Documentation](#documentation)

## Code of Conduct

This project follows a Code of Conduct that all contributors are expected to adhere to. Please be respectful and constructive in all interactions.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/leptos-studio.git`
3. Add upstream remote: `git remote add upstream https://github.com/analisaperlengkapan/leptos-studio.git`
4. Create a new branch: `git checkout -b feature/your-feature-name`

## Development Setup

### Prerequisites

- Rust (stable) - Install via [rustup](https://rustup.rs/)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install --locked trunk`

### Running Locally

```bash
# Start the development server
trunk serve

# Run tests (this is what CI runs)
cargo test --workspace --lib --bins

# Check code quality
cargo clippy --all-targets --all-features

# Format code
cargo fmt
```

## How to Contribute

### Reporting Bugs

- Use the [Bug Report template](../.github/ISSUE_TEMPLATE/bug_report.yml)
- Include clear steps to reproduce
- Provide browser/OS information
- Add screenshots if applicable

### Suggesting Features

- Use the [Feature Request template](../.github/ISSUE_TEMPLATE/feature_request.yml)
- Explain the problem you're trying to solve
- Describe your proposed solution
- Consider alternatives

### Code Contributions

1. Pick an existing issue or create a new one
2. Comment on the issue to let others know you're working on it
3. Fork and create a branch
4. Make your changes
5. Add tests
6. Update documentation
7. Submit a pull request

## Style Guidelines

### Rust Code

- Follow the [Rust Style Guide](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Keep functions focused and well-documented
- Add doc comments for public APIs

### CSS

- Use consistent indentation (2 spaces)
- Follow existing naming conventions
- Use CSS variables for colors and spacing
- Keep specificity low

### Documentation

- Use clear, concise language
- Include code examples where helpful
- Keep README.md updated
- Document breaking changes

## Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test updates
- `build`: Build system changes
- `ci`: CI/CD changes
- `chore`: Other changes

### Examples

```
feat(canvas): add grid snapping functionality

Implements grid snapping for more precise component placement.
Users can toggle this feature via the settings panel.

Closes #123
```

```
fix(export): resolve HTML escaping in Leptos code generator

Previously, special characters in component props were not properly
escaped, leading to invalid generated code.

Fixes #456
```

## Pull Request Process

1. **Update documentation**: Ensure README.md and other docs reflect your changes
2. **Add tests**: All new features should have tests
3. **Pass CI**: All CI checks must pass
4. **Keep PRs focused**: One feature/fix per PR
5. **Fill out the template**: Use the [PR template](../.github/PULL_REQUEST_TEMPLATE.md)
6. **Request review**: Tag maintainers for review
7. **Address feedback**: Respond to review comments promptly
8. **Squash commits**: We prefer clean git history

### PR Checklist

- [ ] Code follows style guidelines
- [ ] Self-reviewed the code
- [ ] Commented hard-to-understand areas
- [ ] Updated documentation
- [ ] No new warnings
- [ ] Added tests
- [ ] All tests pass
- [ ] Meaningful commit messages

## Testing

### Unit Tests

```bash
cargo test --workspace --lib --bins
```

### WASM Tests

The browser-only suite (`tests/wasm_smoke.rs`) is compiled with
`wasm-bindgen-test`. Run it with:

```bash
wasm-pack test --headless --chrome
```

`cargo test --target wasm32-unknown-unknown` is **not** supported: the test
harness and some dependencies (such as `mio`) do not build for that target.

### Manual Testing

1. Test in multiple browsers (Chrome, Firefox, Safari, Edge)
2. Test responsive layouts
3. Verify export functionality
4. Check localStorage persistence

## Documentation

### Code Documentation

- Add doc comments (`///`) to public items
- Include examples in doc comments
- Explain complex algorithms

### Project Documentation

- Update README.md for user-facing changes
- Update ARCHITECTURE.md for structural changes
- Add entries to CHANGELOG.md
- Create/update guides in `/docs` if needed

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.

## Questions?

Feel free to:
- Open a discussion
- Ask in issues
- Reach out to maintainers

Thank you for contributing to Leptos Studio! 🚀
