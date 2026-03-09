# Getting Started

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2021)
- Git

### Optional

- [Tailscale](https://tailscale.com/) for remote access

## Setup

1. Clone the repository:

   ```bash
   git clone git@github.com:tamarum/grocery_inventory.git
   cd grocery_inventory
   ```

2. Copy the example config:

   ```bash
   cp config.example.toml config.toml
   ```

3. Build:

   ```bash
   cargo build --features web
   ```

4. Run the CLI:

   ```bash
   cargo run -- -c config.toml --help
   ```

5. Start the web server:

   ```bash
   cargo run --features web -- -c config.toml web
   ```

   Then open [http://127.0.0.1:3000](http://127.0.0.1:3000).

## Configuration

Edit `config.toml` to customize:

```toml
[database]
path = "grocery_inventory.db"

[web]
host = "127.0.0.1"   # Use "0.0.0.0" for LAN/remote access
port = 3000

[shopping]
low_stock_threshold = 0   # Items at or below this qty appear on shopping list
include_out_of_stock = true
```

## Remote Access

To access the web UI from your phone or share with others:

1. Change `host` to `"0.0.0.0"` in `config.toml`
2. Install [Tailscale](https://tailscale.com/) on your computer and phone
3. Run `tailscale funnel 3000` to get a shareable public URL

The included `start.sh` script handles starting both the web server and Tailscale Funnel together.
