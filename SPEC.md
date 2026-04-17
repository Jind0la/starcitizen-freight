# Freight — Star Citizen Cargo Calculator

## 1. Concept & Vision

**Freight** is the anti-overwhelming cargo tool. One input — your cargo hold size — and it tells you the three most profitable trade routes right now, with fuel cost subtracted, with clear profit numbers that a human can act on in under 5 seconds at a terminal.

No accounts. No dashboards. No 15 filters. No confusion about which price is which. Just: *I have X SCU of cargo space, what's my best move?*

The personality is confident and direct — like a seasoned freight pilot reading a data tablet. Information-dense but scannable. Dark theme because every Star Citizen player is flying at 2am anyway.

---

## 2. Design Language

**Aesthetic:** Military-industrial cargo manifest. Think cargo manifests, shipping labels, terminal readouts. Clean, tabular, high-information-density.

**Color Palette:**
- Background: `#0B0E14` (deep space black)
- Surface: `#151A23` (panel background)
- Border: `#1E2530` (subtle dividers)
- Primary text: `#E8EAED` (off-white)
- Muted text: `#6B7280` (labels, secondary)
- Accent green (profit): `#22C55E`
- Accent red (loss): `#EF4444`
- Accent amber (warning/margin): `#F59E0B`
- Accent blue (interactive): `#3B82F6`

**Typography:**
- Primary: `JetBrains Mono` (monospace — cargo manifests are monospace by tradition)
- Fallback: `Consolas`, `Monaco`, `monospace`
- Font sizes: 11px muted labels, 13px body, 16px route headers, 24px profit numbers

**Spatial System:**
- 4px base unit
- Compact rows: 8px padding
- Section gaps: 16px
- Max content width: 80 chars (matches terminal width philosophy)

**Motion Philosophy:**
- Minimal. Loading spinner during API fetch. Instant result render.
- No decorative animations — every frame serves information delivery.

---

## 3. Layout & Structure

```
╔══════════════════════════════════════════════════════════╗
║  FREIGHT v0.1.0  ·  Stanton System  ·  Updated HH:MM ago ║
╠══════════════════════════════════════════════════════════╣
║  CARGO (SCU)  [____42____]  [CALCULATE]                  ║
╠══════════════════════════════════════════════════════════╣
║  #1 ★★★  ARES STAR  →  ORISON           +12,847 CR/SCU   ║
║       Laranite    96 SCU @ buy → sell   +1,233,312 CR    ║
║       Margin 47%  ·  Stock: HIGH         Qty: 96 SCU     ║
╠══════════════════════════════════════════════════════════╣
║  #2 ★★☆  HURSTON   →  AREX DRILL         +8,234 CR/SCU  ║
║       Titanium   42 SCU @ buy → sell     +345,828 CR     ║
║       Margin 31%  ·  Stock: MED           Qty: 42 SCU     ║
╠══════════════════════════════════════════════════════════╣
║  #3 ★☆☆  CELLIN    →  DAYMAR REGION     +6,102 CR/SCU  ║
║       Diamond     8 SCU @ buy → sell      +48,816 CR      ║
║       Margin 28%  ·  Stock: LOW           Qty: 8 SCU      ║
╠══════════════════════════════════════════════════════════╣
║  ⚠ Fuel est: ~2,400 CR  ·  Based on avg 3.24.x prices   ║
╚══════════════════════════════════════════════════════════╝
```

**Page structure:**
1. Header bar — version, system, last updated
2. Input row — cargo SCU input + calculate button
3. Results — ranked routes (max 3 shown)
4. Footer note — fuel disclaimer + data source

**Responsive:** Single-column, works on mobile (375px+) and desktop. TUI via ratatui for local use.

---

## 4. Features & Interactions

### Core Feature: Instant Profit Calculation

**Input:** Cargo hold size in SCU (integer, 1–16000)

**Process:**
1. Fetch `/commodities_routes` with `investment` filter ≥ user's cargo value
2. Fetch fuel prices for relevant terminals (or use cached averages)
3. For each route:
   - Calculate: `(sell_price - buy_price) × min(scu_available, user_scu)`
   - Subtract estimated round-trip fuel cost
   - Rank by profit per SCU × available SCU (total profit, not per-SCU)
4. Return top 3 routes

**Output per route:**
- Rank + star rating (subjective confidence: 3★ = high stock + recent data, 1★ = low stock or stale)
- Commodity name + route origin → destination
- SCU to buy/sell (min of user input, stock available)
- Buy price → Sell price
- Total profit in CR
- Margin percentage
- Stock level indicator (HIGH/MED/LOW based on scu_sell_stock)
- Fuel estimate for the route

### Interaction Details

**Calculate button:**
- Disabled during fetch (show spinner)
- Disabled if input is empty or ≤ 0
- On Enter key in input field, triggers calculate

**Route row click/tap:**
- Expands to show additional details: container sizes accepted, game version of data, data freshness

**Error states:**
- API unreachable: "Cannot reach UEX API. Check your internet connection."
- No routes found for cargo size: "No profitable routes found for X SCU. Try a smaller cargo hold or check back later."
- API rate limit: "Rate limited by UEX API. Please wait a moment."
- Invalid input: Inline validation message below input field

**Empty state:** Initial state shows placeholder text: "Enter your cargo capacity above to find the best trade routes right now."

### Data Freshness

- Show "Updated X minutes ago" based on API cache headers
- Show game version (e.g., "3.24.x") as a subline
- Routes with >7 days of no user trade reports get a ⚠️ indicator

---

## 5. Component Inventory

### CargoInput
- Text input, numeric only, placeholder "e.g. 96"
- States: default (blue border), focused (blue glow), error (red border + message), loading (disabled)
- "Calculate" button: default blue, hover slightly lighter, loading shows spinner, disabled grayed out

### RouteCard
- Rank badge (#1, #2, #3) with star rating
- Commodity name (bold) + route "ORIGIN → DEST" (muted)
- Profit per SCU: large green number, right-aligned
- Total profit: medium text below
- Stock indicator: pill badge (HIGH=green, MED=amber, LOW=red)
- Expanded state: shows buy/sell prices individually, container sizes, fuel estimate, data age

### StatusBar
- Last updated timestamp
- API status indicator (● green = live, ● amber = stale, ● red = error)
- Game version tag

### ErrorMessage
- Red-bordered box with icon
- Message text
- Optional retry button

---

## 6. Technical Approach

### Stack

**Core:** Rust (no framework, vanilla Axum for optional HTTP server)

**API Client:** reqwest with tokio runtime

**TUI:** ratatui + crossterm (local terminal UI)

**Web UI:** minimal HTML/CSS/JS static files served by Axum

**State:** No external state management. All state is local to the request or session.

### Architecture

```
src/
├── main.rs          # Entry point: chooses TUI or HTTP mode
├── api.rs           # UEX API client (reqwest)
├── models.rs        # Type-safe response structs
├── calculation.rs   # Route filtering, ranking, profit math
├── error.rs         # AppError enum
└── cli.rs           # TUI layout and event loop (ratatui)

Cargo.toml
.env.example
README.md
SPEC.md
```

### API Integration

**Endpoints used:**
- `GET /commodities_routes` — pre-computed profitable routes (no auth)
- `GET /fuel_prices` — for fuel cost estimation (no auth)
- `GET /terminals` — for terminal metadata (no auth, cached 12h)

**Rate limit handling:** 172,800 daily / 120/min. Respect `Retry-After` headers. Cache aggressively (use API cache TTLs as cache durations).

**Error handling:** All API errors surface as user-friendly messages. No stack traces in CLI output.

### Data Model

```rust
// From API response
struct Route {
    id: u32,
    commodity_name: String,
    origin_terminal_name: String,
    destination_terminal_name: String,
    price_origin: f64,         // buy price per SCU
    price_destination: f64,    // sell price per SCU
    scu_origin: Option<f64>,   // stock available
    scu_destination: Option<f64>,
    scu_margin: f64,
    price_margin: f64,
    price_roi: f64,
    investment: f64,
    profit: f64,
    distance: f64,             // GM
    score: f64,
    container_sizes_origin: String,
    container_sizes_destination: String,
    game_version_origin: String,
    date_added: u64,
}

struct FuelPrice {
    price_buy: f64,
    terminal_name: String,
}

struct AppState {
    cargo_scu: u32,
    routes: Vec<RankedRoute>,
    fuel_estimate: f64,
}

struct RankedRoute {
    rank: u8,
    stars: u8,           // 1-3
    commodity: String,
    origin: String,
    destination: String,
    scu_to_trade: u32,
    buy_price: f64,
    sell_price: f64,
    total_profit: f64,
    profit_per_scu: f64,
    margin_pct: f64,
    stock_level: StockLevel,
    fuel_cost: f64,
    container_sizes: String,
}
```

### Caching Strategy

- In-memory cache with TTL:
  - Routes: 30 minutes (matches API cache TTL)
  - Fuel prices: 30 minutes
  - Terminals: 12 hours
- No persistent cache (app is stateless between runs)

### Calculation Formula

```
For each route:
  max_scu = min(user_cargo_scu, route.scu_available)
  gross_profit = (price_destination - price_origin) × max_scu
  fuel_cost = estimate_fuel_cost(route.distance)
  net_profit = gross_profit - fuel_cost
  profit_per_scu = net_profit / max_scu
  margin_pct = ((price_destination - price_origin) / price_origin) × 100

Rank by: net_profit descending
```

### Fuel Estimation

```
// Average fuel consumption ~10 SCU Hydrogen per 100 GM (rough estimate)
// Hydrogen avg price from API or fallback: 15 CR/SCU
fuel_cost = (distance_gm / 100) × 10 × hydrogen_price
round_trip_multiplier = 2.0
total_fuel_cost = fuel_cost × 2
```

---

## 7. Non-Goals (Out of Scope)

- User accounts / authentication
- Persistent history of trades
- Price history charts
- Multiple star systems (Stanton only, at least initially)
- Ship-specific loadout optimization
- Refinery/mining calculations
- Mobile native apps
- Trade alerts / notifications
