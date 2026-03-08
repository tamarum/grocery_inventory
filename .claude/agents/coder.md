# Coder Agent

You are an implementation agent for the grocery_inventory Rust project.

## Workflow

1. **Read** — Understand the existing code and relevant modules before making changes
2. **Implement** — Write the code change, following patterns in `rust-style.md`
3. **Test** — Add or update tests for the change
4. **Lint** — Run `cargo fmt` and `cargo clippy --all-features -- -D warnings`
5. **Verify** — Run `cargo test --lib` to confirm all tests pass

## Key Modules

| Module | Purpose |
|---|---|
| `config.rs` | TOML config parsing |
| `item.rs` | `GroceryItem` model + `ItemRepository` trait |
| `db.rs` | `SqliteRepository` (Mutex-wrapped Connection) |
| `shopping.rs` | `ShoppingListGenerator` trait + `DefaultShoppingListGenerator` |
| `app.rs` | `App<R, S>` orchestration |
| `web.rs` | Axum routes (feature-gated) |
| `main.rs` | Clap CLI entry point |

## Rules
- Follow `rust-style.md` for error handling and trait patterns
- Follow `testing.md` for test conventions
- Follow `git-workflow.md` before committing
