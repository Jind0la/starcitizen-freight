/* Freight — SC Cargo Calculator Frontend v0.2.0 */

(function () {
    'use strict';

    // ── DOM refs ──────────────────────────────────────────────────────────────
    const shipSelect   = document.getElementById('shipSelect');
    const scuInput     = document.getElementById('scuInput');
    const systemSelect = document.getElementById('systemSelect');
    const calcBtn      = document.getElementById('calcBtn');
    const refreshBtn   = document.getElementById('refreshBtn');
    const loading      = document.getElementById('loading');
    const loadingText  = document.getElementById('loadingText');
    const errorBox     = document.getElementById('errorBox');
    const results      = document.getElementById('results');
    const routesList   = document.getElementById('routesList');
    const fuelTotal    = document.getElementById('fuelTotal');
    const routeCount   = document.getElementById('routeCount');
    const dataAge      = document.getElementById('dataAge');
    const lastUpdated  = document.getElementById('lastUpdated');
    const emptyState   = document.getElementById('emptyState');
    const cacheDot     = document.getElementById('cacheDot');
    const cacheLabel   = document.getElementById('cacheLabel');

    // ── State ─────────────────────────────────────────────────────────────────
    let cacheMs = null; // timestamp when data was loaded

    // ── Helpers ───────────────────────────────────────────────────────────────
    function fmt(n, decimals = 0) {
        return Number(n).toLocaleString('en-US', {
            minimumFractionDigits: decimals,
            maximumFractionDigits: decimals,
        });
    }

    function stars(score) {
        const filled = Math.min(3, Math.max(1, Math.round(score)));
        return Array.from({ length: 3 }, (_, i) =>
            `<span class="star ${i < filled ? 'on' : 'off'}">★</span>`
        ).join('');
    }

    function stockBadge(level) {
        const cls   = level?.toLowerCase() ?? 'low';
        const label = cls.toUpperCase();
        return `<span class="stock-badge ${cls}">${label}</span>`;
    }

    function profitBar(profit, maxProfit) {
        const isLoss = profit < 0;
        const pct = maxProfit > 0
            ? Math.min(100, Math.abs(profit / maxProfit) * 100)
            : 0;
        const cls  = isLoss ? 'loss' : '';
        const value = isLoss
            ? `-${fmt(Math.abs(profit))} aUEC`
            : `+${fmt(profit)} aUEC`;
        return `
            <div class="profit-bar-track">
                <div class="profit-bar-fill ${cls}" style="width:${pct}%"></div>
            </div>
            <div class="profit-bar-value ${cls}">${value}</div>
        `;
    }

    function shipMaxContainer(shipValue) {
        // shipValue format: "scu:maxContainer" e.g. "66:4"
        const parts = (shipValue || '').split(':');
        return parts.length === 2 ? parseInt(parts[1], 10) : null;
    }

    function routeCard(route, rank, maxProfit) {
        const invest    = route.buy_price * route.scu_to_trade;
        const containers = route.container_sizes || '';
        const containerLabels = containers
            .split('|')
            .map(s => s.trim())
            .filter(Boolean)
            .map(s => s + ' SCU')
            .join(' · ') || '—';

        const playerBadge = route.is_player_owned
            ? '<span class="player-badge">Player</span>'
            : '';

        const jumpBadge = route.is_interstellar
            ? `<span class="interstellar-badge" title="Cross-system route via jump point">` +
              `<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/><path d="M2 12h20"/></svg>` +
              ` ${route.jump_count === 1 ? '1 jump' : route.jump_count + ' jumps'}` +
              ` · ${route.destination_system || 'Pyro'}</span>`
            : '';

        const qlinkHref = route.destination_slug
            ? `https://er.key4.top/e/?s=${encodeURIComponent(route.destination_slug)}`
            : null;
        const qlinkHTML = qlinkHref
            ? `<a class="qlink-btn" href="${qlinkHref}" target="_blank" rel="noopener">` +
                `<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>` +
                ` Nav »</a>`
            : '';

        return `
        <article class="route-card">
            <div class="route-header">
                <div class="route-rank">
                    <span class="rank-badge">${rank}</span>
                    <span class="stars">${stars(route.stars)}</span>
                    <span class="route-commodity">${route.commodity}</span>
                    <span class="route-meta">${playerBadge}${jumpBadge}</span>
                </div>
                ${stockBadge(route.stock_level)}
            </div>

            <div class="route-path">
                <span class="path-from">${route.origin}</span>
                <svg class="path-arrow" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
                <span class="path-to">${route.destination}</span>
                ${route.distance_gm > 0 ? `<span class="path-distance">· ${fmt(route.distance_gm, 1)} GM</span>` : ''}
                ${qlinkHTML}
            </div>

            <div class="route-stats">
                <div class="stat">
                    <span class="stat-label">SCU</span>
                    <span class="stat-value neutral">${fmt(route.scu_to_trade)}</span>
                </div>
                <div class="stat">
                    <span class="stat-label">Buy</span>
                    <span class="stat-value buy">${fmt(route.buy_price)}</span>
                </div>
                <div class="stat">
                    <span class="stat-label">Sell</span>
                    <span class="stat-value sell">${fmt(route.sell_price)}</span>
                </div>
                <div class="stat">
                    <span class="stat-label">Margin</span>
                    <span class="stat-value neutral">${route.margin_pct.toFixed(1)}%</span>
                </div>
                <div class="stat">
                    <span class="stat-label">Profit/SCU</span>
                    <span class="stat-value profit">${fmt(route.profit_per_scu)}</span>
                </div>
                <div class="stat">
                    <span class="stat-label">Invest</span>
                    <span class="stat-value neutral">${fmt(invest)}</span>
                </div>
                <div class="stat">
                    <span class="stat-label">Fuel</span>
                    <span class="stat-value" style="color:var(--warning)">-${fmt(route.fuel_cost)}</span>
                </div>
                <div class="stat">
                    <span class="stat-label">Containers</span>
                    <span class="stat-value Containers">${containers || '—'}</span>
                </div>
            </div>

            <div class="route-profit-row">
                <div>
                    <div class="profit-bar-label">Net Profit (${fmt(route.scu_to_trade)} SCU)</div>
                    ${profitBar(route.total_profit, maxProfit)}
                </div>
            </div>
        </article>`;
    }

    // ── Cache status ──────────────────────────────────────────────────────────
    function setCacheStatus(state, msg) {
        cacheDot.className = 'cache-dot ' + state;
        cacheLabel.textContent = msg;
    }

    function updateCacheAge() {
        if (!cacheMs) return;
        const ageMs   = Date.now() - cacheMs;
        const ageMin  = Math.floor(ageMs / 60000);
        if (ageMin < 1) {
            setCacheStatus('live', 'Fresh');
        } else if (ageMin < 20) {
            setCacheStatus('stale', `${ageMin}m old`);
        } else {
            setCacheStatus('error', `${ageMin}m old`);
        }
    }

    // ── URL sync ──────────────────────────────────────────────────────────────
    function syncToURL() {
        const scu  = scuInput.value.trim();
        const sys  = systemSelect.value;
        const ship = shipSelect.value;
        const params = new URLSearchParams();
        if (scu)       params.set('scu', scu);
        if (sys !== '68') params.set('sys', sys);
        if (ship)      params.set('ship', ship);
        const search = params.toString();
        const newUrl = search ? `${location.pathname}?${search}` : location.pathname;
        history.replaceState(null, '', newUrl);
    }

    function loadFromURL() {
        const params = new URLSearchParams(location.search);
        if (params.has('scu'))  scuInput.value = params.get('scu');
        if (params.has('sys'))  systemSelect.value = params.get('sys');
        if (params.has('ship')) shipSelect.value = params.get('ship');
    }

    // ── API ──────────────────────────────────────────────────────────────────
    async function fetchRoutes(scu, systemId, shipMaxContainer) {
        const params = new URLSearchParams({ scu, system_id: systemId });
        if (shipMaxContainer) params.set('ship_max_container', shipMaxContainer);
        const url  = `/api/routes?${params.toString()}`;
        const res  = await fetch(url);
        if (!res.ok) {
            if (res.status === 429) throw new Error('Rate limited by UEX API. Try again later.');
            if (res.status === 400) throw new Error('Invalid SCU input (1–16000 allowed).');
            throw new Error(`API error ${res.status}`);
        }
        return res.json();
    }

    // Force-refresh by bypassing client cache
    async function refreshData() {
        const scu    = scuInput.value.trim() || (shipSelect.value ? shipSelect.value.split(':')[0] : '');
        if (!scu) {
            showError('Select a ship or enter SCU first.');
            return;
        }
        const systemId = systemSelect.value;
        const shipVal  = shipSelect.value;
        const shipMax  = shipVal ? parseInt(shipVal.split(':')[1], 10) : null;

        loadingText.textContent = 'Refreshing data...';
        loading.classList.remove('hidden');
        results.classList.add('hidden');
        errorBox.classList.add('hidden');
        emptyState.classList.add('hidden');
        calcBtn.disabled = true;
        refreshBtn.disabled = true;

        try {
            // Bypass browser cache with timestamp
            const ts = Date.now();
            const params = new URLSearchParams({ scu, system_id: systemId, _t: ts });
            if (shipMax) params.set('ship_max_container', shipMax);
            const url = `/api/routes?${params.toString()}`;
            const res = await fetch(url);
            if (!res.ok) throw new Error(`API error ${res.status}`);
            const data = await res.json();
            render(data);
        } catch (err) {
            showError(err.message || 'Failed to refresh. Check your connection.');
        } finally {
            loading.classList.add('hidden');
            calcBtn.disabled = false;
            refreshBtn.disabled = false;
        }
    }

    // ── Render ────────────────────────────────────────────────────────────────
    function render(data) {
        const { routes, total_fuel_estimate, last_updated, cached } = data;

        cacheMs = cached ? Date.now() - (30 * 60 * 1000 - (data.cache_age_ms || 0)) : Date.now();
        updateCacheAge();
        if (cached) {
            setInterval(updateCacheAge, 30000);
        }

        lastUpdated.textContent = last_updated ? `Updated ${last_updated}` : '';

        if (!routes || routes.length === 0) {
            errorBox.textContent = 'No profitable routes found for this SCU amount. Try a different ship or system.';
            errorBox.classList.remove('hidden');
            results.classList.add('hidden');
            emptyState.classList.add('hidden');
            return;
        }

        errorBox.classList.add('hidden');
        results.classList.remove('hidden');
        emptyState.classList.add('hidden');

        fuelTotal.textContent  = fmt(total_fuel_estimate);
        routeCount.textContent = routes.length;

        // Show data age from first route
        if (routes[0] && routes[0].data_age_days !== null && routes[0].data_age_days !== undefined) {
            const days = routes[0].data_age_days;
            dataAge.textContent = days === 0 ? 'Today' : `${days}d ago`;
        } else {
            dataAge.textContent = '—';
        }

        const maxProfit = Math.max(...routes.map(r => r.total_profit));
        routesList.innerHTML = routes.map((r, i) =>
            routeCard(r, i + 1, maxProfit)
        ).join('');
    }

    function showError(msg) {
        errorBox.textContent = msg;
        errorBox.classList.remove('hidden');
        results.classList.add('hidden');
        emptyState.classList.add('hidden');
    }

    // ── Main action ────────────────────────────────────────────────────────────
    async function calculate() {
        // Determine SCU: ship selection overrides manual input
        let scu, shipMax;
        if (shipSelect.value) {
            const parts = shipSelect.value.split(':');
            scu     = parts[0];
            shipMax = parseInt(parts[1], 10) || null;
            scuInput.value = ''; // clear manual override
        } else {
            scu     = scuInput.value.trim();
            shipMax = null;
        }

        if (!scu || isNaN(parseInt(scu, 10)) || parseInt(scu, 10) < 1 || parseInt(scu, 10) > 16000) {
            showError('Enter a cargo size between 1 and 16,000 SCU.');
            return;
        }

        const systemId = systemSelect.value;

        // UI: loading state
        loadingText.textContent = 'Fetching routes from UEX...';
        loading.classList.remove('hidden');
        results.classList.add('hidden');
        errorBox.classList.add('hidden');
        emptyState.classList.add('hidden');
        calcBtn.disabled = true;
        refreshBtn.disabled = true;

        syncToURL();

        try {
            const data = await fetchRoutes(scu, systemId, shipMax);
            render(data);
        } catch (err) {
            showError(err.message || 'Failed to fetch routes. Check your connection.');
        } finally {
            loading.classList.add('hidden');
            calcBtn.disabled = false;
            refreshBtn.disabled = false;
        }
    }

    // ── Ship select → auto-fill SCU ──────────────────────────────────────────
    shipSelect.addEventListener('change', () => {
        if (shipSelect.value) {
            const parts = shipSelect.value.split(':');
            scuInput.placeholder = `${parts[0]} SCU (${shipSelect.options[shipSelect.selectedIndex].text.split('(')[0].trim()})`;
            scuInput.value = '';
        } else {
            scuInput.placeholder = 'Manual SCU override';
        }
    });

    // ── Event listeners ──────────────────────────────────────────────────────
    calcBtn.addEventListener('click', calculate);
    refreshBtn.addEventListener('click', refreshData);

    scuInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            // Clear ship selection when manually entering SCU
            shipSelect.value = '';
            calculate();
        }
    });

    // Allow only digit keys + backspace in the input
    scuInput.addEventListener('keypress', (e) => {
        if (!/[\d]/.test(e.key) && e.key !== 'Enter') {
            e.preventDefault();
        }
    });

    // ── Init ──────────────────────────────────────────────────────────────────
    loadFromURL();
    scuInput.focus();

    // Auto-calculate if params in URL
    const params = new URLSearchParams(location.search);
    if (params.has('scu') || params.has('ship')) {
        calculate();
    }
})();
