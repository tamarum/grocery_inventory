# Testing Conventions

## Unit Tests
- Inline `#[cfg(test)] mod tests` in each module
- Use `SqliteRepository::in_memory()` — no test fixtures on disk
- Test both success and error paths

## Integration Tests
- Place in `tests/` directory
- Test full workflows: add → list → update → remove → shopping list
- Use `tempfile` for database paths in integration tests

## Test Naming
- Use descriptive snake_case: `add_and_get`, `low_stock_detection`
- Group related tests in the same module

## Running Tests
- `cargo test --lib` — unit tests only
- `cargo test --all-features` — all tests including feature-gated code
- `cargo test -- --nocapture` — show println output
