// Freight — Star Citizen Cargo Calculator
// ─────────────────────────────────────────────────────────────────

const API_BASE = '/api';

// ─── State ────────────────────────────────────────────────────────
const state = {
  scu: 256,
  systemId: 68,
  shipMaxContainer: 8,
  minMargin: 0,
  tab: 'all',
  routes: [],
  counts: { all: 0, intra_system: 0, interstellar: 0 },
  loading: false,
  error: null,
  lastUpdated: null,
};

// ─── DOM refs ─────────────────────────────────────────────────────
const $ = id => document.getElementById(id);
const scuInput       = $('scuInput');
const shipSelect     = $('shipSelect');
const systemSelect   = $('systemSelect');
const calcBtn        = $('calcBtn');
const refreshBtn     = $('refreshBtn');
const loading        = $('loading');
const loadingText    = $('loadingText');
const errorBox       = $('errorBox');
const results        = $('results');
const emptyState     = $('emptyState');
const routesList     = $('routesList');
const cacheDot       = $('cacheDot');
const cacheLabel     = $('cacheLabel');
const lastUpdated    = $('lastUpdated');
const routeCount     = $('routeCount');
const fuelTotal      = $('fuelTotal');
const tabLabel       = $('tabLabel');
const countAll       = $('countAll');
const countIntra     = $('countIntra');
const countInter     = $('countInter');
const tabBtns        = document.querySelectorAll('.tab-btn');

// ─── Helpers ───────────────────────────────────────────────────────
function fmt(n) {
  if (n === null || n === undefined || isNaN(n)) return '—';
  if (Math.abs(n) >= 1_000_000) return (n / 1_000_000).toFixed(2) + 'M';
  if (Math.abs(n) >= 1_000) return (n / 1_000).toFixed(0) + 'K';
  return n.toFixed(0).replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

function fmtPct(n) {
  if (!n && n !== 0) return '—';
  return n.toFixed(1) + '%';
}

function starsHTML(n) {
  return '★'.repeat(Math.min(n, 3));
}

function stockDot(level) {
  // level: 0=none,1=low,2=medium,3=high
  if (!level) return '<span class="meta-dot dot-low"></span>';
  if (level === 1) return '<span class="meta-dot dot-low"></span>';
  if (level === 2) return '<span class="meta-dot dot-med"></span>';
  return '<span class="meta-dot dot-high"></span>';
}

function marginClass(pct) {
  if (pct >= 20) return 'hi';
  if (pct >= 8) return 'mid';
  return 'lo';
}

// ─── Tab handling ─────────────────────────────────────────────────
tabBtns.forEach(btn => {
  btn.addEventListener('click', () => {
    const t = btn.dataset.tab;
    state.tab = t;
    tabBtns.forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    renderRoutes(state.routes);
  });
});

// ─── Input handlers ────────────────────────────────────────────────
scuInput.addEventListener('input', () => {
  const v = parseInt(scuInput.value, 10);
  if (v > 0) state.scu = v;
});

shipSelect.addEventListener('change', () => {
  const val = shipSelect.value;
  if (!val) {
    state.shipMaxContainer = null;
  } else {
    const parts = val.split(':');
    state.scu = parseInt(parts[0], 10);
    state.shipMaxContainer = parseInt(parts[1], 10);
    // sync manual SCU input to ship value
    scuInput.value = state.scu;
  }
});

systemSelect.addEventListener('change', () => {
  state.systemId = parseInt(systemSelect.value, 10);
});

calcBtn.addEventListener('click', () => calculate());
refreshBtn.addEventListener('click', () => calculate());

// Allow Enter in inputs
scuInput.addEventListener('keydown', e => { if (e.key === 'Enter') calculate(); });

// ─── API ──────────────────────────────────────────────────────────
async function calculate() {
  setLoading(true);
  setError(null);

  const params = new URLSearchParams({
    scu: state.scu,
    systemId: state.systemId || 0,
    tab: state.tab,
    minMargin: state.minMargin,
  });
  if (state.shipMaxContainer) {
    params.set('shipMaxContainer', state.shipMaxContainer);
  }

  try {
    const res = await fetch(`${API_BASE}/routes?${params}`);
    if (res.status === 401 || res.status === 403) {
      throw new Error('UEX API auth failed — check your UEX_API_TOKEN');
    }
    if (!res.ok) {
      const text = await res.text();
      throw new Error(`API error ${res.status}: ${text}`);
    }

    const data = await res.json();
    state.routes = data.routes || [];
    state.counts = data.route_counts || { all: 0, intra_system: 0, interstellar: 0 };
    state.lastUpdated = data.last_updated;

    updateTabCounts(data.route_counts);
    renderRoutes(state.routes);
    setReady(data.last_updated);
  } catch (err) {
    setError(err.message || 'Failed to fetch routes');
    hideResults();
  } finally {
    setLoading(false);
  }
}

// ─── Render ───────────────────────────────────────────────────────
function renderRoutes(routes) {
  if (!routes || routes.length === 0) {
    hideResults();
    emptyState.classList.remove('hidden');
    return;
  }

  emptyState.classList.add('hidden');
  results.classList.remove('hidden');

  routeCount.textContent = routes.length;
  fuelTotal.textContent = fmt(routes.slice(0, 10).reduce((s, r) => s + r.fuelCost, 0));
  tabLabel.textContent = { all: 'All', intra: 'Intra', interstellar: 'Interstellar' }[state.tab] || '—';

  routesList.innerHTML = '';

  routes.forEach((r, i) => {
    const card = document.createElement('div');
    card.className = 'route-card' + (r.isInterstellar ? ' interstellar' : '');

    const interBadge = r.isInterstellar
      ? `<span class="route-system-badge badge-interstellar">Interstellar</span>`
      : `<span class="route-system-badge badge-intra">Stanton</span>`;

    const jumpHTML = r.isInterstellar && r.jumpCount > 0
      ? `<span class="route-jumps">⚡ ${r.jumpCount} jump${r.jumpCount > 1 ? 's' : ''}</span>`
      : '';

    const dataAge = r.dataAgeDays !== null ? `${r.dataAgeDays}d ago` : '—';
    const marginCls = marginClass(r.marginPct);

    card.innerHTML = `
      <div class="route-header" data-index="${i}">
        <span class="route-rank">#${r.rank}</span>
        <span class="route-stars">${starsHTML(r.stars)}</span>
        <span class="route-commodity">${escHtml(r.commodity)}</span>
        ${interBadge}
        ${jumpHTML}
      </div>
      <div class="route-body">
        <div class="route-terminals">
          <span class="terminal-name">${escHtml(r.origin)}</span>
          <span class="terminal-arrow">→</span>
          <span class="terminal-name">${escHtml(r.destination)}</span>
          ${r.isPlayerOwned ? '<span class="player-star">★</span>' : ''}
        </div>
        <div class="route-prices">
          <span class="profit-amount">+${fmt(r.totalProfit)} aUEC</span>
          <span class="margin-pct ${marginCls}">${fmtPct(r.marginPct)} margin</span>
        </div>
        <div class="route-trade">
          <span class="trade-scu">${r.scuToTrade} SCU</span>
          <span class="trade-arrow">·</span>
          <span class="trade-buy">${fmt(r.buyPrice)}</span>
          <span class="trade-arrow">→</span>
          <span class="trade-sell">${fmt(r.sellPrice)}</span>
        </div>
        <div class="route-meta">
          <span class="meta-item">
            ${stockDot(r.stockLevel)}
            Stock
          </span>
          <span class="meta-item">
            ⛽ ${fmt(r.fuelCost)} fuel
          </span>
          <span class="meta-item">
            💰 ${fmt(r.profitPerScu)}/SCU
          </span>
          <span class="meta-item">
            🕐 ${dataAge}
          </span>
        </div>
      </div>
      <div class="route-expanded">
        <div class="expanded-row">
          <span class="expanded-label">SCU to trade</span>
          <span class="expanded-value">${r.scuToTrade}</span>
        </div>
        <div class="expanded-row">
          <span class="expanded-label">Buy price</span>
          <span class="expanded-value">${fmt(r.buyPrice)} aUEC</span>
        </div>
        <div class="expanded-row">
          <span class="expanded-label">Sell price</span>
          <span class="expanded-value">${fmt(r.sellPrice)} aUEC</span>
        </div>
        <div class="expanded-row">
          <span class="expanded-label">Gross profit</span>
          <span class="expanded-value">${fmt(r.totalProfit + r.fuelCost)} aUEC</span>
        </div>
        <div class="expanded-row">
          <span class="expanded-label">Fuel cost</span>
          <span class="expanded-value warning">−${fmt(r.fuelCost)} aUEC</span>
        </div>
        <div class="expanded-row">
          <span class="expanded-label">Net profit</span>
          <span class="expanded-value">${fmt(r.totalProfit)} aUEC</span>
        </div>
        <div class="expanded-row">
          <span class="expanded-label">Distance</span>
          <span class="expanded-value">${r.distanceGm.toFixed(1)} GM</span>
        </div>
        ${r.isInterstellar ? `
        <div class="expanded-row">
          <span class="expanded-label">Destination system</span>
          <span class="expanded-value">${escHtml(r.destinationSystem || '—')}</span>
        </div>
        <div class="expanded-row">
          <span class="expanded-label">Quantum jumps</span>
          <span class="expanded-value">${r.jumpCount}</span>
        </div>` : ''}
        <div class="expanded-row">
          <span class="expanded-label">Container sizes</span>
          <span class="expanded-value">${r.containerSizes || '—'}</span>
        </div>
      </div>
    `;

    // Click to expand
    card.querySelector('.route-header').addEventListener('click', () => {
      card.classList.toggle('expanded');
    });

    routesList.appendChild(card);
  });
}

function escHtml(s) {
  if (!s) return '';
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function updateTabCounts(counts) {
  if (!counts) return;
  countAll.textContent   = counts.all || 0;
  countIntra.textContent = counts.intra_system || 0;
  countInter.textContent = counts.interstellar || 0;
}

function setLoading(on) {
  state.loading = on;
  if (on) {
    loading.classList.remove('hidden');
    results.classList.add('hidden');
    emptyState.classList.add('hidden');
    errorBox.classList.add('hidden');
    cacheDot.textContent = '◌';
    cacheDot.className = 'cache-dot';
    cacheLabel.textContent = 'Loading...';
  } else {
    loading.classList.add('hidden');
  }
}

function setError(msg) {
  if (!msg) {
    errorBox.classList.add('hidden');
    return;
  }
  errorBox.textContent = msg;
  errorBox.classList.remove('hidden');
}

function hideResults() {
  results.classList.add('hidden');
  emptyState.classList.remove('hidden');
}

function setReady(ts) {
  cacheDot.textContent = '●';
  cacheDot.className = 'cache-dot live';
  cacheLabel.textContent = 'Live';
  if (ts) lastUpdated.textContent = ts;
}

// ─── URL Param handling ─────────────────────────────────────────────
const SYSTEM_MAP = { stanton: 68, pyro: 64, nyx: 55 };
const TAB_MAP = { intra: 'intra', interstellar: 'interstellar', all: 'all' };

function readUrlParams() {
  const params = new URLSearchParams(window.location.search);

  const scu = params.get('scu');
  if (scu && !isNaN(parseInt(scu, 10))) {
    state.scu = parseInt(scu, 10);
    scuInput.value = state.scu;
  }

  const system = params.get('system');
  if (system && SYSTEM_MAP[system.toLowerCase()] !== undefined) {
    state.systemId = SYSTEM_MAP[system.toLowerCase()];
    systemSelect.value = state.systemId;
  }

  const tab = params.get('tab');
  if (tab && TAB_MAP[tab.toLowerCase()]) {
    const t = TAB_MAP[tab.toLowerCase()];
    state.tab = t;
    tabBtns.forEach(b => {
      b.classList.toggle('active', b.dataset.tab === t);
    });
  }
}

// ─── Auto-load on page load ─────────────────────────────────────────
window.addEventListener('DOMContentLoaded', () => {
  readUrlParams();
  calculate();
});
