# Technical Specification — US-0007

## Overview

US-0007 adds **ExchangeService** (three connectors), **PortfolioEngine**, **FxService**, migration `007_exchanges_portfolio.sql`, sync pipeline `"exchanges"` phase, extended wealth/portfolio REST API, React `/wealth` Crypto tab + Settings exchange section, plan `allocation_target` template, Grafana Dashboard 4 completion, and `get_portfolio` crypto extension.

**Dependencies:** US-0001 sync scheduler; US-0005 `WealthService` + Alert snapshot hook; US-0004 Plan Engine; US-0006 tool layer.

## Components

### ExchangeService (`backend/src/exchanges/`)

Per DEC-0037 / R-0032:

```
ExchangeService::run_post_sync(run_id)
  ├─ foreach enabled connector (binance, bybit, bitunix):
  │    sync_balances → positions → trades → transfers → funding
  │    upsert exchange_sync_state watermarks
  └─ PortfolioEngine::recompute_pnl(run_id)
```

`ExchangeConnector` trait — GET-only HTTP; HMAC signing per exchange; 429 backoff.

### PortfolioEngine (`backend/src/portfolio/`)

Per DEC-0038 / R-0033:

```
PortfolioEngine::recompute_pnl(run_id)
  ├─ load holdings + trades + exchange position fields
  ├─ compute unrealized (exchange primary; avg-cost fallback)
  ├─ compute realized (exchange cumulative + funding)
  ├─ compute total_return vs portfolio_baselines
  └─ upsert portfolio_pnl_snapshots
```

### FxService (`backend/src/fx/`)

Per DEC-0039 / R-0034:

```
FxService::to_eur(amount, asset, price_book) -> EurAmount
  ├─ stablecoin/fiat → Frankfurter USD/EUR (cached in fx_rates)
  └─ crypto → exchange ticker USDT price × USDT/EUR
```

Sets `fx_incomplete` when alt price missing.

### Database migration `007_exchanges_portfolio.sql`

Per DEC-0042 / R-0037:

- Tables: `exchange_connections`, `exchange_sync_state`, `exchange_holdings`, `exchange_trades`, `exchange_transfers`, `exchange_funding_events`, `portfolio_pnl_snapshots`, `portfolio_baselines`, `fx_rates`
- ALTER `net_worth_snapshots`: add `crypto_value_eur`, `firefly_value_eur`, `total_return_pct`
- Seed exchange connection rows for binance/bybit/bitunix

### Sync integration

Extend `SyncService::execute_run` (DEC-0041):

```rust
// After forecast success (plan hook awaited):
self.set_phase("exchanges").await;
if let Err(e) = self.exchanges.run_post_sync(run_id).await {
    tracing::warn!("exchange sync failed: {e}");
}
self.set_phase("alerts").await;
self.alerts.run_post_sync(run_id, eval_context).await;
// WealthService snapshot now includes crypto EUR
```

Exchange-only path: phases 4–5 via `POST /api/v1/sync/exchanges/trigger`.

### Config

```toml
[exchanges]
enabled = true
interval_seconds = 3600

[exchanges.binance]
enabled = true
api_key_env = "BINANCE_API_KEY"
api_secret_env = "BINANCE_API_SECRET"

[portfolio]
trade_retention_days = 730
frankfurter_base_url = "https://api.frankfurter.dev"
```

Per DEC-0040: secrets from env only; never TOML plaintext.

### REST API

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/exchanges` | Connection list (no secrets) |
| POST | `/api/v1/exchanges/{id}/test` | Read-only connection test |
| POST | `/api/v1/sync/exchanges/trigger` | Exchange-only sync |
| GET | `/api/v1/wealth` | Extended breakdown (firefly + crypto + PnL) |
| GET | `/api/v1/wealth/crypto` | Crypto holdings + per-exchange |
| GET | `/api/v1/wealth/history?days=90` | Trend with crypto columns |
| GET | `/api/v1/portfolio/pnl?days=90` | PnL snapshot series |

Settings API: `[exchanges]` read-only section.

### React UI

**`/wealth`** — Overview | Crypto tabs:
- Overview: combined total; Firefly + Crypto subtotals; remove placeholder when connected
- Crypto: per-exchange cards; holdings table; PnL summary; FX incomplete banner; allocation gap card

**Settings** — Crypto exchanges section (read-only configured badges; test connection button)

**Sync Status** — per-exchange rows; "Sync exchanges now" action

### Plan allocation

New adjustment kind `allocation_target` on `plan_adjustments` (DEC-0042):

```json
{ "kind": "allocation_target", "weights": { "etf_traditional_pct": 50, "crypto_pct": 50, "cash_pct": 0 } }
```

Compare current buckets from wealth API vs target; gap display on `/wealth` or `/planning`.

### Grafana Dashboard 4

File: `grafana/provisioning/dashboards/analytics/portfolio.json` (uid `portfolio`)

Panels (R-0026 completion):
- Total wealth stat (`total_eur` incl. crypto)
- Crypto value stat (`crypto_value_eur`)
- Allocation pie (Firefly vs crypto from payload)
- Portfolio performance time series (`total_return_pct` or `total_eur`)
- Remove crypto placeholder when data present

### AI tool extension

`get_portfolio` (DEC-0035): extend `WealthService::compute_breakdown` — add crypto totals + top 5 holdings; 8 KB cap.

## Interfaces

| Consumer | Interface |
|----------|-----------|
| React | JWT Bearer → `/api/v1/wealth/*`, `/api/v1/exchanges/*`, `/api/v1/sync/exchanges/trigger` |
| Grafana | PostgreSQL → extended `net_worth_snapshots`, `portfolio_pnl_snapshots` |
| Exchanges | HTTPS outbound from backend only; env secrets |
| US-0006 | `get_portfolio` tool reads extended WealthService output |

## Non-functional

- **Read-only exchanges:** GET-only connector; read-only API keys mandatory (DEC-0037, DEC-0004)
- **Secrets:** never in API responses, audit, or OpenAI path (DEC-0040)
- **Latency:** monitor phase 4 duration; recommend longer exchange interval if mutex >45s
- **Failure:** per-exchange error non-blocking for others; partial crypto in snapshot
- **Retention:** exchange trades 2 years; net worth snapshots 365 days
- **Auth:** JWT on all routes (DEC-0006)

## Verification

- Integration test: connector GET-only audit; secrets not in API responses
- Mock-server tests for Binance/Bybit/Bitbit signing
- Portfolio PnL unit tests: avg-cost, exchange-reported reconciliation
- FX incomplete banner E2E when alt unpriced
- Grafana Dashboard 4 provisioning smoke test
- User guide: `docs/user-guides/US-0007.md` (execute phase)
