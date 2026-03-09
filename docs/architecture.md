# Architecture

## Module Overview

```
src/
  config.rs    — TOML config parsing
  item.rs      — GroceryItem model, ItemError, ItemRepository trait
  location.rs  — Location model, Shelf model, LocationRepository/ShelfRepository traits
  db.rs        — SqliteRepository (implements all repository traits)
  shopping.rs  — ShoppingListGenerator trait + DefaultShoppingListGenerator
  app.rs       — App<R, S> orchestration layer (generic over repo + shopping)
  main.rs      — CLI (clap subcommands)
  web.rs       — Axum web server + HTML frontend (feature-gated: "web")
  lib.rs       — Module exports
```

## Data Model

```
Location (1) ──< Shelf (many)
    │                │
    ▼                ▼
 location_id      shelf_id
    \              /
     GroceryItem
```

- **Location** has many **Shelves** (one-to-many)
- **GroceryItem** has an optional `location_id` and optional `shelf_id`
- Assigning a shelf auto-sets `location_id` to the shelf's parent
- Deleting a location cascade-deletes its shelves (`ON DELETE CASCADE`)
- Deleting a shelf/location clears item references (`ON DELETE SET NULL`)

## Key Design Decisions

### Trait-based repository pattern

`ItemRepository`, `LocationRepository`, and `ShelfRepository` are traits. `SqliteRepository` implements all three. This allows:

- In-memory SQLite for tests (`SqliteRepository::in_memory()`)
- Potential future backends without changing business logic

### Generic App struct

`App<R, S>` is generic over its repository and shopping list generator. Shelf methods are available via a separate `impl` block bounded on `R: ItemRepository + LocationRepository + ShelfRepository`.

### Feature-gated web module

The `web` feature gates axum/tower-http dependencies. The web module is compiled only when `--features web` is passed. This keeps the CLI binary small when the web UI isn't needed.

### Thread safety

`SqliteRepository` wraps `Connection` in a `Mutex`. The web server shares the app via `Arc<App<...>>`.

## Database

SQLite via `rusqlite` with bundled `libsqlite3`. Schema is auto-created on first run. Migrations (adding columns to existing databases) are handled in `initialize_schema()` by checking `PRAGMA table_info`.

### Tables

| Table | Columns |
|---|---|
| `locations` | id, name, temperature_f |
| `shelves` | id, location_id (FK → locations), name |
| `grocery_items` | id, name, quantity, unit, category, expiration_date, minimum_stock, location_id (FK → locations), shelf_id (FK → shelves) |
