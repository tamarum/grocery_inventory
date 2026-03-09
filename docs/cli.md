# CLI Reference

All commands require a config file: `cargo run -- -c config.toml <command>`

## Items

### Add an item

```bash
cargo run -- -c config.toml add "Milk" -q 2 -u gallons
cargo run -- -c config.toml add "Chicken Breast" -q 1 -u lbs --expires 2026-03-15
```

Options:

| Flag | Description |
|---|---|
| `-q, --quantity` | Quantity (default: 1) |
| `-u, --unit` | Unit of measurement (default: "count") |
| `--category` | Category name (auto-filled if omitted) |
| `--expires` | Expiration date in YYYY-MM-DD format (auto-filled if omitted) |
| `--min-stock` | Minimum stock level (default: 0) |
| `--location` | Storage location ID |
| `--shelf` | Shelf ID (auto-sets location from shelf's parent) |

When `--category` or `--expires` are omitted, they are automatically filled based on the item name. For example, adding "Milk" auto-sets category to "Dairy" and expiration to 10 days from today.

### List items

```bash
cargo run -- -c config.toml list
```

Displays a table with ID, Name, Qty, Unit, Category, Location, Shelf, and Expires columns.

Expiration warnings:
- `!!!` — expiring within 3 days
- `!` — expiring within 7 days
- `EXPIRED` — past expiration date

A summary of expired and expiring items is shown at the bottom.

### Update an item

```bash
cargo run -- -c config.toml update 1 -q 5
cargo run -- -c config.toml update 1 --shelf 3
cargo run -- -c config.toml update 1 --shelf 0     # clear shelf
cargo run -- -c config.toml update 1 --location 0   # clear location
cargo run -- -c config.toml update 1 --expires 2026-04-01
cargo run -- -c config.toml update 1 --expires none  # clear expiration
```

### Remove an item

```bash
cargo run -- -c config.toml remove 1
```

## Shopping List

```bash
cargo run -- -c config.toml shop
```

Shows items where quantity is at or below `low_stock_threshold` (from config) or below the item's own `minimum_stock`. Items expiring within 3 days are also included with an `[EXPIRING]` tag.

## Locations

### Add a location

```bash
cargo run -- -c config.toml location add "Fridge" --temp 37.0
```

### List locations

```bash
cargo run -- -c config.toml location list
```

### Remove a location

```bash
cargo run -- -c config.toml location remove 1
```

Removing a location clears `location_id` on its items and cascade-deletes its shelves.

## Shelves

### Add a shelf

```bash
cargo run -- -c config.toml shelf add 1 --name "Top Shelf"
```

The first argument is the location ID.

### List shelves for a location

```bash
cargo run -- -c config.toml shelf list 1
```

### Remove a shelf

```bash
cargo run -- -c config.toml shelf remove 1
```

Removing a shelf clears `shelf_id` on its items but preserves their `location_id`.
