# Git Workflow

## Commit Standards
- Write clear, imperative commit messages (e.g., "Add item expiration tracking")
- Keep commits focused — one logical change per commit
- Use conventional commit prefixes: feat, fix, refactor, test, docs, chore

## Pre-Commit Checklist
Before every commit, verify:
1. `cargo fmt --check` passes
2. `cargo clippy --all-features -- -D warnings` passes
3. `cargo test --lib` passes
4. No compiler warnings remain

## Never Commit
- `config.toml` (contains local paths)
- `*.db`, `*.db-journal` (database files)
- `.env` (environment secrets)
- `target/` (build artifacts)

## Branch Naming
- `feat/<description>` for features
- `fix/<description>` for bug fixes
- `refactor/<description>` for refactors
