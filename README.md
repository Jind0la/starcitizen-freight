# Freight — Star Citizen Cargo Calculator

**Dead-simple cargo profit calculator. One input. One answer.**

Enter your cargo hold size, get the top 3 most profitable trade routes in Stanton right now — with fuel costs already subtracted.

```
╔══════════════════════════════════════════════════════════╗
║  FREIGHT v0.1.0  ·  Stanton System  ·  Updated 2m ago  ║
╠══════════════════════════════════════════════════════════╣
║  CARGO (SCU)  [____42____]  [CALCULATE]                  ║
╠══════════════════════════════════════════════════════════╣
║  #1 ★★★  ARES STAR  →  ORISON           +12,847 CR/SCU ║
║       Laranite    42 SCU @ buy → sell   +539,564 CR     ║
║       Margin 47%  ·  Stock: HIGH         Qty: 42 SCU   ║
╠══════════════════════════════════════════════════════════╣
║  ⚠ Fuel est: ~2,400 CR  ·  Based on avg 3.24.x prices   ║
╚══════════════════════════════════════════════════════════╝
```

## Features

- **Top 3 profit routes** — ranked by net profit (sell - buy - fuel)
- **Fuel cost included** — hydrogen consumption estimated per route distance
- **Stock level indicators** — HIGH/MED/LOW demand at destination
- **Star ratings** — confidence score based on data freshness + user trade volume
- **Expandable rows** — click a route to see fuel estimate, distance, container sizes
- **Caching** — API responses cached 30min to minimize requests
- **No account required** — public UEX API, no auth needed

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
echo "UEX_API_TOKEN=your_token_here" > .env
```

## Usage

```bash
freight
```

**Controls:**
- `↑` / `↓` — navigate routes (future: history)
- `Enter` — calculate profit for entered SCU
- `Click` route — expand details
- `Esc` or `q` — quit

## Architecture

```
src/
├── main.rs          # TUI event loop
├── api.rs           # UEX API client (reqwest, async, cached)
├── models.rs        # API response types + domain types
├── calculation.rs   # Route filtering, ranking, fuel math
├── cli.rs           # ratatui TUI layout
└── error.rs         # Error types
```

**API endpoints used:**
- `GET /2.0/commodities_routes` — pre-computed profitable routes
- `GET /2.0/fuel_prices` — hydrogen prices for fuel estimation
- `GET /2.0/terminals?id_star_system=1` — terminal metadata (cached 12h)

## Disclaimer

Data comes from community-reported prices via [UEX API 2.0](https://uexcorp.space/api/documentation/). Prices may be outdated or inaccurate. Always verify at a terminal before committing to a trade route.

## License

MIT
