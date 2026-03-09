# Grocery Inventory

A home grocery inventory system with CLI and web interfaces. Track what's in your fridges, freezers, and pantries — down to the shelf. Built in Rust with SQLite storage.

## Features

- **Multi-location tracking** — manage multiple fridges, freezers, pantries with temperature data
- **Shelf-level organization** — assign items to specific shelves within each location
- **Expiration tracking** — warnings for expired and expiring items in CLI and web UI
- **Auto-fill categories** — automatically categorizes items (20+ groups) as you type
- **Auto-fill expiration dates** — estimates shelf life for 60+ item types
- **Receipt scanning** — photograph a grocery receipt and batch-import items via Claude Vision API
- **Shopping list generation** — flags low-stock and expiring items with suggested quantities
- **Web UI** — mobile-friendly single-page app with card layout for phones
- **CLI** — full-featured command-line interface for scripting and quick access
- **REST API** — JSON endpoints for all operations
- **Remote access** — share your inventory via Tailscale Funnel

## Quick Start

```bash
# Clone and build
git clone git@github.com:tamarum/grocery_inventory.git
cd grocery_inventory
cp config.example.toml config.toml
cargo build --features web

# Start the web server
cargo run --features web -- -c config.toml web
# Open http://127.0.0.1:3000
```

## CLI Usage

```bash
# Items (category and expiration auto-fill when not specified)
cargo run -- -c config.toml add "Milk" -q 2 -u gallons --shelf 1
cargo run -- -c config.toml add "Chicken Breast" -q 1 -u lbs --expires 2026-03-15
cargo run -- -c config.toml list
cargo run -- -c config.toml update 1 -q 5
cargo run -- -c config.toml update 1 --expires 2026-04-01
cargo run -- -c config.toml update 1 --expires none  # clear expiration
cargo run -- -c config.toml remove 1

# Locations
cargo run -- -c config.toml location add "Fridge" --temp 37.0
cargo run -- -c config.toml location list

# Shelves
cargo run -- -c config.toml shelf add 1 --name "Top Shelf"
cargo run -- -c config.toml shelf list 1

# Shopping list (includes expiring items)
cargo run -- -c config.toml shop
```

## Configuration

Edit `config.toml`:

```toml
[database]
path = "grocery_inventory.db"

[web]
host = "0.0.0.0"       # "127.0.0.1" for local only
port = 3000

[shopping]
low_stock_threshold = 0
include_out_of_stock = true

[anthropic]
api_key = "sk-ant-..."  # Optional: enables receipt scanning via Claude Vision
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

- Assigning a shelf to an item auto-sets its location
- Deleting a location cascade-deletes its shelves
- Deleting a shelf clears `shelf_id` on items but preserves `location_id`

## Remote Access

To access from your phone or share with others:

1. Set `host = "0.0.0.0"` in `config.toml`
2. Install [Tailscale](https://tailscale.com/) and run `tailscale funnel 3000`
3. Share the public HTTPS URL with anyone

Use `./start.sh` to launch both the web server and Tailscale Funnel together.

## Development

```bash
cargo fmt                                    # Format
cargo clippy --all-features -- -D warnings   # Lint
cargo test --lib                             # Unit tests
cargo test --all-features                    # All tests (including web)
```

## Documentation

- [Getting Started](docs/getting-started.md)
- [CLI Reference](docs/cli.md)
- [API Reference](docs/api.md)
- [Architecture](docs/architecture.md)
- [Roadmap](ROADMAP.md)

## Tech Stack

- **Language**: Rust (edition 2021)
- **Database**: SQLite via `rusqlite` (bundled)
- **CLI**: `clap` with subcommands
- **Web**: `axum` + `tower-http` (feature-gated)
- **Error handling**: `thiserror` (library) + `anyhow` (application)
