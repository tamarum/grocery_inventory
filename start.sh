#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$HOME/.npm-global/bin:$PATH"

# Build if needed
cargo build --features web --release 2>&1

# Kill any existing instance
kill $(ss -tlnp 2>/dev/null | grep :3000 | grep -oP 'pid=\K\d+') 2>/dev/null || true
sleep 0.5

# Start the web server in the background
./target/release/grocery_inventory -c config.toml web &
WEB_PID=$!
echo "Web server started (pid $WEB_PID) at http://0.0.0.0:3000"

# Start Tailscale Funnel in the background
tailscale funnel 3000 &
FUNNEL_PID=$!
echo "Tailscale Funnel started (pid $FUNNEL_PID)"
echo "Public URL: https://tamari-brooks-thinkpad-p51.tailb5a00f.ts.net/"

echo ""
echo "Press Ctrl+C to stop both."

trap "kill $WEB_PID $FUNNEL_PID 2>/dev/null; echo 'Stopped.'; exit" INT TERM
wait
