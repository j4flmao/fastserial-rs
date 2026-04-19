# Contributing to FastSerial-RS

First off, thank you for considering contributing to FastSerial! It's people like you that make FastSerial such a great tool.

## 🚀 How Can I Contribute?

### Reporting Bugs
- Use the GitHub Issue Tracker.
- Describe the bug and include steps to reproduce.
- Mention your OS and Rust version.

### Suggesting Enhancements
- Check if the idea was already suggested.
- Open an issue with a clear title and description.

### Pull Requests
1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. If you've changed APIs, update the documentation.
4. Ensure the test suite passes (`cargo test`).
5. Make sure your code lints (`cargo clippy`).

## 🛠 Development Workflow

### Setup
```bash
git clone https://github.com/j4flmao/fastserial-rs.git
cd fastserial-rs
cargo build
```

### Git Hooks (Husky & Commitlint)
We use `husky` and `commitlint` to manage git hooks and ensure high-quality commits.
- **Pre-commit**: Runs `make check test` to ensure code quality and all tests pass before committing.
- **Commit-msg**: Ensures commit messages follow the [Conventional Commits](https://www.conventionalcommits.org/) specification (e.g., `feat: add new feature`, `fix: resolve bug`).

To set up hooks, ensure you have [Node.js](https://nodejs.org/) installed, then run:
```bash
npm install
```
The hooks will be automatically installed. If you need to bypass them (not recommended), use `git commit --no-verify`.

### Running Tests
```bash
# Run all tests in the workspace
cargo test --workspace
```

### Benchmarking
We use `criterion` for library benchmarks and `oha` for API benchmarks.
```bash
cargo bench
```

## 📜 Style Guide
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `cargo fmt` before committing.
- Document public functions with Rustdoc (Arguments, Returns, Examples).

## ⚖️ License
By contributing, you agree that your contributions will be licensed under its MIT License.
