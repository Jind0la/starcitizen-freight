# Freight — Star Citizen Cargo Calculator

## 1. Concept & Vision

**Freight** is the cargo calculator that cuts through the noise. Other tools show you dashboards, charts, 15-filter forms, and data that takes 10 minutes to parse at a terminal. Freight shows you three things: *where to buy, where to sell, how much you'll make.*

One input — your cargo hold size. One output — the three best routes right now, with fuel cost already subtracted, with a profit number you can act on in 3 seconds.

**What makes Freight different from every other tool:**

The "En Route" feature. You're flying from Crusader to Hurston anyway with 60 SCU of empty cargo space. You fire up Freight, punch in your SCU, enable "En Route", and it shows you: *there's 40 SCU of Hydrogen at ArcCorp Mining that you can grab on the way and drop at Orison for +18,000 aUEC*. This is the feature no other tool makes easy.

**Interstellar routes as a first-class citizen.** Stanton→Pyro routes are real, they're profitable, and the data exists in the UEX API. But every tool hides cross-system trading behind extra clicks. Freight shows Stanton routes AND Pyro routes AND cross-system routes in one unified list, tagged clearly.

The personality is confident and direct — like a seasoned freight pilot reading a data tablet. No decorative UI. No marketing language. Just data.

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
- Accent red (loss/warning): `#EF4444`
- Accent amber (margin/caution): `#F59E0B`
- Accent blue (interactive/links): `#3B82F6`
- Interstellar badge: `#A855F7` (purple — distinct from intra-system)

**Typography:**
- Primary: `Exo 2` (geometric, slightly futuristic — evokes spacecraft HUD)
- Monospace (numbers, codes): `JetBrains Mono`
- Fallback: `Consolas`, `Monaco`, `monospace`
- Font sizes: 11px muted labels, 13px body, 16px route headers, 24px+ profit numbers

**Spatial System:**
- 4px base unit
- Compact rows: 8px padding
- Section gaps: 16px
- Cards: 12px padding, 6px border-radius

**Motion Philosophy:**
- Loading: skeleton pulse animation on route cards (not spinners)
- Route cards: fade in with 50ms stagger between cards
- Tab switches: cross-fade 150ms
- No decorative animations — every frame serves information delivery

---

## 3. Layout & Structure

### Web UI Layout

```
┌─────────────────────────────────────────────────────────┐
│  ◈ FREIGHT                    Stanton ▼ | 500 SCU [▾]  │
│  The 3-minute cargo calculator                          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  [Intra-System] [Interstellar] [En Route]               │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ ★★★  #1   Iodine  ·  Stanton → Pyro  ·  ⡿ 1 jump │ │
│  │        HUR-OUTPOST  →  Admin - Seraphim           │ │
│  │        ─────────────────────────────────────────── │ │
│  │   +1,833,000 aUEC          Margin: 28.2%         │ │
│  │   500 SCU @ 9,334 → 13,000  (+3,666/SCU)         │ │
│  │   STOCK ● LOW  ·  FUEL ⛽ ~0 CR  ·  AGE: today   │ │
│  │   Containers: 1|2|4|8|16|24|32                    │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ ★★☆  #2   Laranite  ·  Stanton  ·  ground        │ │
│  │        ArcCorp Mining  →  Admin - HUR-L1          │ │
│  │        ─────────────────────────────────────────── │ │
│  │   +1,376,431 aUEC          Margin: 28.1%         │ │
│  │   500 SCU @ 7,047 → 9,800  (+2,752/SCU)         │ │
│  │   STOCK ● LOW  ·  FUEL ⛽ ~2,100 CR  ·  AGE: today│ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ ★☆☆  #3   Consumer Goods  ·  Stanton  ·  ground │ │
│  │        ...                                        │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  ⚠ Net profit after ~2,100 CR fuel  ·  UEX data · 4.7 │
│  Quantum fuel estimated at ~800 aUEC/SCU H₂            │
└─────────────────────────────────────────────────────────┘
```

**Page structure:**
1. Header bar — logo, system selector (dropdown), SCU input
2. Tag tabs — [Intra-System] [Interstellar] [En Route]
3. Results — ranked route cards (max 3)
4. Footer — fuel disclaimer, data source, game version

**Responsive:** Single-column, works on mobile (375px+) and desktop. TUI via ratatui for local use.

---

## 4. Features & Interactions

### Core Feature: Profit Calculation

**Input:**
- Cargo hold size in SCU (integer, 1–16000)
- System selector: Stanton (default), Pyro, Nyx
- Tab: Intra-System | Interstellar | En Route

**Intra-System Tab:** Routes within a single star system (Stanton→Stanton). Shows routes from all terminals in Stanton, ranked by net profit.

**Interstellar Tab:** Cross-system routes only. Stanton→Pyro, Stanton→Nyx. Shows the jump count badge, the destination system name, and includes extra quantum fuel in the profit calculation.

**En Route Tab (the killer feature):** Takes current location and destination as inputs. Shows cargo opportunities along the quantum travel path. Example: flying Crusader→Hurston → shows Hydrogen available at ArcCorp Mining you can grab en route.

**Processing per route:**
1. Calculate: `(sell_price - buy_price) × min(scu_available, user_scu)`
2. Subtract estimated round-trip quantum fuel cost
3. For interstellar: add extra fuel for jump point traversal
4. Rank by net profit descending
5. Return top 3 routes

**Output per route:**
- Rank + star rating (confidence: 3★ = high stock + recent user data, 1★ = stale/low)
- Commodity name + route (origin → destination)
- System badge if cross-system (purple "STANTON → PYRO")
- Jump count badge if interstellar (purple "⡿ 1 jump")
- SCU to buy/sell (min of user input, stock available)
- Buy price → Sell price
- Total net profit in aUEC (fuel already subtracted)
- Profit per SCU
- Margin percentage
- Stock level indicator (HIGH/MED/LOW)
- Fuel estimate for the route (quantum fuel only, not hydrogen for maneuvering)
- Data age ("today" / "2 days ago" / "1 week ago")
- Container sizes accepted
- Player-owned terminal indicator (★ if destination is a player outpost)

### Star Rating Logic

- Base: 1 star
- +1 star if UEX score ≥ 7.0 (algorithmic quality rating)
- +1 star if ≥ 10 user-reported trades confirm the prices (price_origin_users_rows ≥ 10)
- Max: 3 stars

### Interactions

**Calculate button:**
- Disabled during fetch (show skeleton pulse)
- Disabled if input is empty or ≤ 0
- Enter key in SCU input triggers calculate

**Route card click:**
- Expands to show additional details: container sizes, fuel breakdown, data age, player-owned badge

**System selector:**
- Dropdown with Stanton, Pyro, Nyx
- Changing system re-triggers calculation with new system routes

**Tab switching:**
- Instant filter — no API re-fetch needed (data already loaded)
- Tab shows count badge: "Intra-System (847)" "Interstellar (22)"

**Error states:**
- API unreachable: "Cannot reach UEX API. Check your internet connection."
- No routes found: "No profitable routes for X SCU. Try a smaller cargo hold."
- Rate limited: "Rate limited by UEX API. Please wait a moment."
- Invalid input: Inline red text below input field

**Empty state:** Shows placeholder with suggested SCU values for popular ships (Cutlass Black: 66 SCU, Caterpillar: 576 SCU, Hull C: 1500 SCU).

### Data Freshness

- Show "Updated X minutes ago" based on when data was fetched
- Show game version tag (e.g., "4.7")
- Routes with >7 days since last user trade get amber "⚠ stale data" indicator

---

## 5. Gap Analysis: What Freight Does That Others Don't

### SC Trade Tools (sc-trade.tools)
**Strengths:** Best-in-class UI, massive dataset, "En Route" feature concept.
**Gaps Freight fills:**
- No cross-system (Stanton↔Pyro) routes surfaced clearly — Freight shows them as first-class results
- No fuel cost subtraction in displayed profit — Freight shows net profit (fuel pre-subtracted)
- Requires configuration/filters — Freight is zero-config
- "En Route" requires setting exact origin/destination — Freight's En Route just needs your cargo SCU

### UEX Website (uexcorp.space)
**Strengths:** Raw data, comprehensive.
**Gaps Freight fills:**
- Not a calculator — just data viewer
- Complex navigation
- No profit ranking
- No fuel cost calculation

### What Freight adds uniquely:
1. **Net profit display** — fuel cost pre-subtracted so you see what you actually take home
2. **Cross-system routes as first-class results** — not buried, not requiring extra API calls
3. **Simplest possible UX** — one number, three routes, done. No filters.
4. **Ship quick-select** — popular ships shown as one-click presets (Cutlass Black: 66 SCU, Caterpillar: 576 SCU, etc.)
5. **Zero-configuration En Route** — doesn't ask for origin/destination, just "where can I make money on my current trip"

---

## 6. Technical Approach

### Stack

**Core:** Rust (Axum for HTTP, Ratatui for TUI)

**API Client:** reqwest + tokio runtime

**TUI:** ratatui + crossterm (local terminal UI)

**Web UI:** embedded HTML/CSS/JS (Axum serves via rust-embed), single-file JS, no build step

**State:** All state is local to the session. No external state management.

### API Integration

**Endpoints used:**
- `GET /commodities_routes` (auth optional but required for full route data)
- `GET /commodities` (no auth — for hydrogen price reference)
- `GET /fuel_prices_all` (no auth — for hydrogen/H2 fuel prices)
- `GET /star_systems` (no auth — for system list with live status)

**Auth:** Bearer token via `UEX_API_TOKEN` env var. Required for `/commodities_routes`. Without it, routes endpoint returns limited data.

**Rate limits:** 172,800/day, 120/min with auth. Cache aggressively.

### Data Model

```rust
// Raw API models (from UEX)
struct Route {
    id: u32,
    commodity_id: u32,
    id_star_system_origin: u32,
    id_star_system_destination: u32,
    id_terminal_origin: u32,
    id_terminal_destination: u32,
    commodity_name: String,
    origin_star_system_name: Option<String>,
    destination_star_system_name: Option<String>,
    terminal_origin_name: String,
    terminal_destination_name: String,
    terminal_origin_slug: Option<String>,
    terminal_destination_slug: Option<String>,
    origin_terminal_is_player_owned: Option<i32>,
    price_origin: f64,
    price_destination: f64,
    price_margin: f64,
    price_roi: f64,
    scu_origin: Option<f64>,
    scu_destination: Option<f64>,
    status_origin: Option<i32>,
    status_destination: Option<i32>,
    investment: f64,
    profit: f64,
    distance: Option<f64>,  // in GM
    score: Option<f64>,
    container_sizes_origin: Option<String>,
    container_sizes_destination: Option<String>,
    game_version_origin: Option<String>,
    game_version_destination: Option<String>,
    date_added: Option<u64>,
}

struct Commodity {
    id: u32,
    name: String,
    price_sell: Option<f64>,
    is_fuel: Option<i32>,
    // ...
}

struct FuelPrice {
    commodity_id: u32,
    commodity_name: String,
    terminal_name: String,
    star_system_name: Option<String>,
    price_buy: f64,
    price_buy_avg: Option<f64>,
}

struct StarSystem {
    id: u32,
    name: String,
    is_available_live: Option<i32>,
}

// Enriched domain model (what the UI sees)
struct RankedRoute {
    rank: u8,
    stars: u8,              // 1-3 confidence stars
    commodity: String,
    commodity_slug: Option<String>,
    origin: String,
    destination: String,
    scu_to_trade: u32,
    buy_price: f64,
    sell_price: f64,
    total_profit: f64,      // net after fuel
    profit_per_scu: f64,
    margin_pct: f64,
    stock_level: StockLevel,
    fuel_cost: f64,         // quantum fuel only
    container_sizes: String,
    distance_gm: f64,
    data_age_days: Option<u32>,
    is_player_owned: bool,
    destination_slug: Option<String>,
    is_interstellar: bool,  // crosses star systems
    jump_count: u8,         // 0=intra, 1-2=interstellar
    destination_system: Option<String>,
}
```

### Caching Strategy

- In-memory cache with TTL:
  - Routes: 30 minutes
  - Fuel prices: 30 minutes
  - Star systems: 1 day
- No persistent cache (stateless between runs)

### Calculation Formula

```
For each route:
  max_scu = min(user_cargo_scu, route.scu_origin)
  gross_profit = (price_destination - price_origin) × max_scu
  fuel_cost = estimate_quantum_fuel(route.distance, is_interstellar)
  net_profit = gross_profit - fuel_cost
  profit_per_scu = net_profit / max_scu
  margin_pct = ((price_destination - price_origin) / price_origin) × 100

Rank by: net_profit descending
```

### Fuel Estimation

```
// Hydrogen consumption: ~10 SCU H2 per 100 GM quantum travel
// Hydrogen price: fetched from /fuel_prices_all or commodity list
// Jump point traversal: +30 SCU H2 per jump

hydrogen_price = fuel_prices_all.find("Hydrogen").price_buy_avg ?? 400 CR/SCU
hydrogen_per_100gm = 10 SCU
distance_gm = route.distance ?? 0
fuel_cost = (distance_gm / 100) × hydrogen_per_100gm × hydrogen_price × 2 (round trip)
if is_interstellar:
  fuel_cost += QUANTUM_JUMP_COST (30 SCU × hydrogen_price × 2)
```

### Known Limitations

- Cross-system routes in UEX data are limited: only Stanton→Pyro commodity routes exist
- Pyro itself has 211 terminals but no commodity_routes data for Pyro→Pyro in the API
- Nyx has 103 terminals but no commodity_routes data
- The `rank_interstellar_routes` function is a best-effort approach that combines intra-system sell prices from one system with buy prices from another — this is an approximation since the UEX API doesn't provide explicit cross-system routes
- Fuel estimation uses average hydrogen price, not location-specific prices
- En Route feature requires origin/destination inputs (not yet in v1)

---

## 7. Non-Goals (Out of Scope)

- User accounts / authentication
- Persistent trade history
- Price history charts
- Ship-specific loadout optimization
- Refinery/mining calculations
- Mobile native apps
- Trade alerts / notifications
- Price prediction / trend analysis

---

## 8. File Structure

```
src/
├── main.rs          # Entry: TUI or HTTP mode, dotenv loading
├── api.rs           # UEX API client (reqwest, caching, parallel commodity fetching)
├── models.rs        # Type-safe API response structs + RankedRoute
├── calculation.rs   # Route filtering, ranking, fuel estimation, star rating
├── error.rs         # AppError enum
├── cli.rs           # Ratatui TUI layout and event loop
├── web_server.rs    # Axum HTTP server + JSON API endpoint
├── web_ui.rs        # rust-embed for static web assets
web/
├── index.html       # Web UI entry point
├── styles.css       # Dark theme CSS
└── app.js           # Single-file JS, no dependencies
Cargo.toml
.env.example
README.md
SPEC.md
```
