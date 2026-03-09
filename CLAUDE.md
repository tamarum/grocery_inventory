# Grocery Inventory

Home grocery inventory system with CLI and web interfaces. Built in Rust with SQLite storage.

## Quick Commands

```bash
cargo check --all-features      # Type-check everything
cargo clippy --all-features -- -D warnings  # Lint
cargo fmt                        # Format
cargo test --lib                 # Unit tests
cargo test --all-features        # All tests
cargo run -- --help              # CLI help
cargo run -- -c config.toml list # List inventory
cargo build --features web       # Build with web server
```

## Architecture

| Module | File | Purpose |
|---|---|---|
| Config | `src/config.rs` | TOML config parsing with `thiserror` errors |
| Item | `src/item.rs` | `GroceryItem` model + `ItemRepository` trait |
| Location | `src/location.rs` | `Location` + `Shelf` models, `LocationRepository` + `ShelfRepository` traits |
| Database | `src/db.rs` | `SqliteRepository` — `Mutex<Connection>` impl of all repository traits |
| Shopping | `src/shopping.rs` | `ShoppingListGenerator` trait + `DefaultShoppingListGenerator` |
| App | `src/app.rs` | `App<R, S>` — generic orchestration over repo + shopping |
| Web | `src/web.rs` | Axum routes + mobile-responsive HTML, feature-gated behind `web` |
| CLI | `src/main.rs` | Clap subcommands: add, list, update, remove, shop, location, shelf, web |

## Key Details

- **Database**: SQLite via `rusqlite` with bundled `libsqlite3`. Connection wrapped in `Mutex` for thread safety.
- **Feature gates**: `web` feature enables axum/tower-http. Use `--features web` or `--all-features`.
- **Error handling**: `thiserror` in library code (`ItemError`, `ConfigError`), `anyhow` in `main.rs`.
- **Config**: Copy `config.example.toml` to `config.toml` to run. Never commit `config.toml`.
- **Testing**: In-memory SQLite (`SqliteRepository::in_memory()`) for all tests. No external dependencies.

## Developer Infrastructure

- **Rules**: `.claude/rules/` — git-workflow, rust-style, testing conventions
- **Agents**: `.claude/agents/` — coder, code-reviewer, test-runner
- **Skills**: `.claude/skills/` — build, test, review, wrapup
