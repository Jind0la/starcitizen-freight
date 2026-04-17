# Freight — Star Citizen Cargo Calculator

**Dead-simple cargo profit calculator. One input. One answer.**

Enter your cargo hold size, get the top 3 most profitable trade routes in Stanton or Pyro right now — with fuel costs already subtracted.

```
╔══════════════════════════════════════════════════════════╗
║  FREIGHT v0.3.0  ·  Stanton System  ·  Updated 2m ago  ║
╠══════════════════════════════════════════════════════════╣
║  #1 ★★★  IODINE  →  Admin - Seraphim      +3,666 CR/SCU║
║       HUR-OUTPOST → HUR-L1    500 SCU   +1,833,000 CR   ║
║       Margin 28.2%  ·  Stock: HIGH       Qty: 500 SCU  ║
╠══════════════════════════════════════════════════════════╣
║  ⚠ Fuel est: ~0 CR  ·  Based on avg 3.24.x prices       ║
╚══════════════════════════════════════════════════════════╝
```

## What Makes Freight Different

Most trade tools give you route tables — long lists of every commodity at every station. Freight does one thing: **tells you the single best route for your cargo hold right now**.

The difference:
- **One input.** Cargo size. That's it. No commodity filters, no station search, no Excel exports.
- **Fuel included.** Most calculators show gross profit. Freight subtracts hydrogen consumption based on route distance, giving you **net profit**.
- **Star ratings.** Confidence score from 1-3 stars based on user trade data volume and data freshness. More user trades = more reliable price.
- **Interstellar routes.** First tool to show Stanton→Pyro cross-system routes computed from live UEX data. Buy cheap in Stanton, sell high in Pyro — fuel cost included.

## Features

- **Top 3 net profit routes** — ranked by (sell - buy - fuel)
- **Fuel cost included** — hydrogen consumption estimated per route distance and quantum jump count
- **Star ratings** — confidence from user trade volume + data freshness
- **Stock level indicators** — HIGH/MED/LOW demand at destination
- **Interstellar (Pyro) routes** — cross-system Stanton→Pyro routes with jump fuel cost
- **System selector** — Stanton (default) or Pyro
- **Ship presets** — select your ship and container size is auto-detected
- **Cache** — API responses cached 30min, no redundant requests

## Installation

### Pre-built (recommended)

```bash
# macOS / Linux x86-64
curl -fsSL https://raw.githubusercontent.com/Jind0la/starcitizen-freight/main/install.sh | bash

# Or download from Releases: https://github.com/Jind0la/starcitizen-freight/releases
```

### Build from source

```bash
git clone https://github.com/Jind0la/starcitizen-freight.git
cd starcitizen-freight
cargo build --release
./target/release/freight
```

**Requirements:** Rust 1.75+ (install via [rustup](https://rustup.rs/))

### Optional: UEX API Token

Public endpoints work without a token. For higher rate limits:

```bash
# Get a token at: https://uexcorp.space/api/apps
echo "UEX_API_TOKEN=your_token" > .env
```

## Web UI

Start the web interface:

```bash
./target/release/freight --web --port 8081
```

Then open [http://localhost:8081](http://localhost:8081) in your browser.

## CLI

```bash
freight
```

**Controls:**
- `↑` / `↓` — navigate routes
- `Enter` — calculate profit for entered SCU
- `Click` route — expand details
- `Esc` or `q` — quit

## API

```bash
# Get top routes for 500 SCU in Stanton
curl "http://localhost:8081/api/routes?scu=500&system_id=68"

# Get top routes for 500 SCU in Pyro (includes interstellar Stanton→Pyro routes)
curl "http://localhost:8081/api/routes?scu=500&system_id=64"
```

## Architecture

```
src/
├── main.rs          # TUI event loop
├── api.rs           # UEX API client (reqwest, async, parallel commodity fetching)
├── models.rs        # API response types + domain types + ship database
├── calculation.rs   # Route ranking, fuel estimation, interstellar logic
├── cli.rs           # TUI rendering (ratatui)
├── error.rs         # Error types
└── web_server.rs    # Axum HTTP server for web UI

src/web/
├── index.html      # Web UI
├── styles.css      # Dark theme styling
└── app.js          # Vanilla JS frontend
```

## Data Sources

- Commodity prices & routes: [UEX API v2.0](https://uexcorp.space/api/documentation/)
- Star systems: Stanton (68), Pyro (64), Nyx (55)
- Fuel price estimation: ~800 aUEC/SCU hydrogen (in-game reference price)
- Quantum fuel per jump: ~30 SCU hydrogen (estimated for jump gate traversal)

## Limitations

- Prices are from UEX user-reported trades, not live in-game
- Pyro routes require selecting "Pyro" in the system filter — Pyro data is limited compared to Stanton
- Fuel estimates are based on average hydrogen prices and quantum travel distance — actual consumption varies by ship
- Nyx system data is sparse; most profitable routes will be in Stanton or Pyro

## License

MIT
