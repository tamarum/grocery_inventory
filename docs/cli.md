# CLI Reference

All commands require a config file: `cargo run -- -c config.toml <command>`

## Items

### Add an item

```bash
cargo run -- -c config.toml add "Milk" -q 2 -u gallons
```

Options:

| Flag | Description |
|---|---|
| `-q, --quantity` | Quantity (default: 1) |
| `-u, --unit` | Unit of measurement (default: "count") |
| `--category` | Category name |
| `--min-stock` | Minimum stock level (default: 0) |
| `--location` | Storage location ID |
| `--shelf` | Shelf ID (auto-sets location from shelf's parent) |

### List items

```bash
cargo run -- -c config.toml list
```

Displays a table with ID, Name, Qty, Unit, Category, Location, and Shelf columns.

### Update an item

```bash
cargo run -- -c config.toml update 1 -q 5
cargo run -- -c config.toml update 1 --shelf 3
cargo run -- -c config.toml update 1 --shelf 0   # clear shelf
cargo run -- -c config.toml update 1 --location 0 # clear location
```

### Remove an item

```bash
cargo run -- -c config.toml remove 1
```

## Shopping List

```bash
cargo run -- -c config.toml shop
```

Shows items where quantity is at or below `low_stock_threshold` (from config) or below the item's own `minimum_stock`.

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
