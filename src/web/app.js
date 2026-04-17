/* Freight — SC Cargo Calculator Frontend */

(function () {
    'use strict';

    // ── DOM refs ──────────────────────────────────────────────────────────────
    const scuInput    = document.getElementById('scuInput');
    const calcBtn     = document.getElementById('calcBtn');
    const loading     = document.getElementById('loading');
    const errorBox    = document.getElementById('errorBox');
    const results     = document.getElementById('results');
    const routesList  = document.getElementById('routesList');
    const fuelTotal   = document.getElementById('fuelTotal');
    const lastUpdated = document.getElementById('lastUpdated');
    const emptyState  = document.getElementById('emptyState');

    // ── Helpers ────────────────────────────────────────────────────────────────
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
        const cls  = level?.toLowerCase() ?? 'low';
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

    function routeCard(route, rank, maxProfit) {
        const invest = route.buy_price * route.scu_to_trade;
        return `
        <article class="route-card">
            <div class="route-header">
                <div class="route-rank">
                    <span class="rank-badge">${rank}</span>
                    <span class="stars">${stars(route.stars)}</span>
                    <span class="route-commodity">${route.commodity}</span>
                </div>
                ${stockBadge(route.stock_level)}
            </div>

            <div class="route-path">
                <span class="path-from">${route.origin}</span>
                <svg class="path-arrow-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M5 12h14M12 5l7 7-7 7"/>
                </svg>
                <span class="path-to">${route.destination}</span>
                ${route.distance_gm > 0 ? `<span style="margin-left:6px;opacity:0.5">· ${fmt(route.distance_gm, 1)} GM</span>` : ''}
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
                    <span class="stat-label">Investment</span>
                    <span class="stat-value neutral">${fmt(invest)}</span>
                </div>
                <div class="stat">
                    <span class="stat-label">Fuel Cost</span>
                    <span class="stat-value" style="color:var(--warning)">-${fmt(route.fuel_cost)}</span>
                </div>
                <div class="stat">
                    <span class="stat-label">Containers</span>
                    <span class="stat-value neutral" style="font-size:0.75rem">${route.container_sizes}</span>
                </div>
            </div>

            <div class="route-profit-bar">
                <div>
                    <div class="profit-bar-label">Net Profit (${fmt(route.scu_to_trade)} SCU)</div>
                    ${profitBar(route.total_profit, maxProfit)}
                </div>
            </div>
        </article>`;
    }

    // ── API ───────────────────────────────────────────────────────────────────
    async function fetchRoutes(scu) {
        const url = `/api/routes?scu=${encodeURIComponent(scu)}`;
        const res  = await fetch(url);
        if (!res.ok) {
            if (res.status === 429) throw new Error('Rate limited by UEX API. Try again later.');
            if (res.status === 400) throw new Error('Invalid SCU input (1–16000 allowed).');
            throw new Error(`API error ${res.status}`);
        }
        return res.json();
    }

    // ── Render ───────────────────────────────────────────────────────────────
    function render(data) {
        const { routes, total_fuel_estimate, last_updated } = data;

        if (routes.length === 0) {
            errorBox.textContent = 'No profitable routes found for this SCU amount.';
            errorBox.classList.remove('hidden');
            results.classList.add('hidden');
            return;
        }

        errorBox.classList.add('hidden');
        results.classList.remove('hidden');
        emptyState.classList.add('hidden');

        fuelTotal.textContent = fmt(total_fuel_estimate);
        lastUpdated.textContent = last_updated ? `Updated ${last_updated}` : '';

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
        const raw = scuInput.value.trim();
        const scu = parseInt(raw, 10);

        if (!raw || isNaN(scu) || scu < 1 || scu > 16000) {
            showError('Enter a cargo size between 1 and 16,000 SCU.');
            return;
        }

        // UI: loading state
        loading.classList.remove('hidden');
        results.classList.add('hidden');
        errorBox.classList.add('hidden');
        emptyState.classList.add('hidden');
        calcBtn.disabled = true;

        try {
            const data = await fetchRoutes(scu);
            render(data);
        } catch (err) {
            showError(err.message || 'Failed to fetch routes. Check your connection.');
        } finally {
            loading.classList.add('hidden');
            calcBtn.disabled = false;
        }
    }

    // ── Event listeners ───────────────────────────────────────────────────────
    calcBtn.addEventListener('click', calculate);

    scuInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') calculate();
    });

    // Allow only digit keys + backspace in the input
    scuInput.addEventListener('keypress', (e) => {
        if (!/[\d]/.test(e.key) && e.key !== 'Enter') {
            e.preventDefault();
        }
    });

    // ── Init ───────────────────────────────────────────────────────────────────
    // Focus input on load
    scuInput.focus();

    // Auto-calculate if SCU param in URL
    const params = new URLSearchParams(window.location.search);
    const scuParam = params.get('scu');
    if (scuParam) {
        scuInput.value = scuParam;
        calculate();
    }
})();
