# Rust Style Guide

## Error Handling
- Use `thiserror` for library error types (e.g., `ItemError`, `ConfigError`)
- Use `anyhow` in `main.rs` and application-level code
- Propagate errors with `?`; avoid `.unwrap()` outside tests

## Traits and Dependency Injection
- Define behavior traits: `ItemRepository`, `ShoppingListGenerator`
- Accept trait objects (`&dyn Trait`) or generics (`impl Trait`) for testability
- The `App<R, S>` struct is generic over repository and shopping generator

## Concurrency
- `SqliteRepository` uses `Mutex<Connection>` for thread safety
- Web handlers use `Arc<App<...>>` as shared state
- Keep lock scopes minimal

## Testing
- Use `SqliteRepository::in_memory()` for test databases
- Test through public API (`App` methods), not internal details
- Mock traits when testing components in isolation

## Feature Gates
- `web` feature gates axum/tower-http dependencies
- Use `#[cfg(feature = "web")]` for web module code
- Always test with `--all-features` in CI
