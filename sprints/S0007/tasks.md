# Tasks — Sprint S0007

**Story:** US-0007  
**Task count:** 12 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0073 | SQLx migration 007 exchanges portfolio schema | open | AC-1, AC-3, AC-5 |
| T-0074 | Exchange portfolio config and FxService EUR conversion | open | AC-3, AC-6 |
| T-0075 | ExchangeConnector trait HTTP layer and repository | open | AC-1, AC-6 |
| T-0076 | Binance Bybit and Bitunix connector implementations | open | AC-1 |
| T-0077 | Portfolio Engine hybrid PnL and baselines | open | AC-2, AC-3 |
| T-0078 | Sync pipeline exchanges phase and ExchangeService | open | AC-1, AC-3 |
| T-0079 | Extended wealth portfolio REST API and allocation_target template | open | AC-3, AC-4 |
| T-0080 | React /wealth Crypto tab and Overview extension | open | AC-3, AC-4 |
| T-0081 | Settings crypto exchanges and Sync Status per-exchange rows | open | AC-1, AC-6 |
| T-0082 | Grafana Dashboard 4 completion | open | AC-5 |
| T-0083 | Exchange portfolio and get_portfolio tests | open | AC-1, AC-2, AC-6 |
| T-0084 | Operator user guide | open | AC-1–AC-6 |

---

## T-0073 — SQLx migration 007 exchanges portfolio schema

**Status:** open  
**Depends on:** US-0001 (migration 001) through US-0006 (migration 006)  
**Decisions:** DEC-0042, R-0037

### Description

Add SQLx migration `007_exchanges_portfolio.sql` per architecture § migration 007:

| Object | Purpose |
|--------|---------|
| `exchange_connections` | Per-exchange enabled/state/last_sync metadata |
| `exchange_sync_state` | Watermarks JSON per exchange |
| `exchange_holdings` | Normalized balances/positions with EUR marks |
| `exchange_trades` | Trade history; unique `(exchange_id, external_id)` |
| `exchange_transfers` | Deposits/withdrawals |
| `exchange_funding_events` | Funding fee / income ledger rows |
| `portfolio_pnl_snapshots` | Daily PnL aggregate (`UNIQUE(snapshot_date)`) |
| `portfolio_baselines` | First-sync baseline per exchange for total return |
| `fx_rates` | Daily fiat pair cache |
| `net_worth_snapshots` **ALTER** | Add `crypto_value_eur`, `firefly_value_eur`, `total_return_pct`; extend `payload` |

Indexes:

- `exchange_holdings(exchange_id)`
- `exchange_trades(exchange_id, executed_at DESC)`
- Trade retention policy: 2 years (startup prune job hook for T-0078)

Seed:

```sql
INSERT INTO exchange_connections (id) VALUES ('binance'), ('bybit'), ('bitunix')
ON CONFLICT DO NOTHING;
```

Implement repository stubs in `exchanges/repository.rs` and `portfolio/repository.rs` for upsert/list used by later tasks.

### Done when

- [ ] Migration applies cleanly against external PostgreSQL
- [ ] All tables and ALTER columns match architecture schema
- [ ] Seed rows for three exchanges present
- [ ] Unique constraints on trades and PnL snapshot date enforced
- [ ] Repository upsert methods compile for holdings/trades/sync_state

---

## T-0074 — Exchange portfolio config and FxService EUR conversion

**Status:** open  
**Depends on:** T-0073  
**Decisions:** DEC-0039, DEC-0040, R-0034, R-0035

### Description

**TOML config additions:**

```toml
[exchanges]
enabled = true
interval_seconds = 3600

[exchanges.binance]
enabled = true
api_key_env = "BINANCE_API_KEY"
api_secret_env = "BINANCE_API_SECRET"
base_url = "https://api.binance.com"

[exchanges.bybit]
enabled = false
api_key_env = "BYBIT_API_KEY"
api_secret_env = "BYBIT_API_SECRET"

[exchanges.bitunix]
enabled = false
api_key_env = "BITUNIX_API_KEY"
api_secret_env = "BITUNIX_API_SECRET"
spot_base_url = "https://openapi.bitunix.com"
enabled_futures = false

[portfolio]
trade_retention_days = 730
frankfurter_base_url = "https://api.frankfurter.dev"
```

Load into `ExchangeConfig`, `PortfolioConfig` at startup. Secrets resolved from env via `*_env` names — never TOML plaintext.

**FxService (`backend/src/fx/`):**

```rust
FxService::to_eur(amount, asset, price_book: &ExchangePriceBook) -> Result<EurAmount, FxError>
```

- Frankfurter ECB daily for USD/USDT/USDC/GBP → EUR; cache in `fx_rates` with 24h TTL
- Crypto alts: exchange ticker `{ASSET}USDT` × USDT/EUR mark-to-market per sync
- Bybit unified wallet: `usdValue` × USD/EUR when available
- Missing price → `FxError::Unpriced`; caller sets `fx_incomplete=true`

**Extend `GET /api/v1/settings`:** include `[exchanges]` and `[portfolio]` sections; expose env var **names** and `configured: bool` badges — no secret values.

Wire `FxService` into `AppState`.

### Done when

- [ ] Config loads `[exchanges]` and `[portfolio]` from TOML + env overlay
- [ ] Settings API returns exchange sections without secrets
- [ ] `configured` badge reflects env key presence per exchange
- [ ] Frankfurter fetch caches rates in `fx_rates` table
- [ ] `to_eur` converts fiat/stablecoin and alt via price book
- [ ] Unpriced asset returns error for caller to handle
- [ ] Unit tests cover fiat conversion and unpriced alt path

---

## T-0075 — ExchangeConnector trait HTTP layer and repository

**Status:** open  
**Depends on:** T-0073, T-0074  
**Decisions:** DEC-0037, DEC-0040, DEC-0004, R-0032

### Description

Implement `backend/src/exchanges/` foundation:

| Submodule | Responsibility |
|-----------|----------------|
| `trait` | `ExchangeConnector` async trait |
| `types` | `ExchangeHolding`, `ExchangeTrade`, `ExchangeTransfer`, `ExchangeFundingEvent`, `ExchangeSyncState`, `ConnectionTest` |
| `http` | Shared reqwest client; HMAC signing helpers; 429 exponential backoff |
| `repository` | Upsert holdings/trades/transfers/funding; read/write sync watermarks |

**Trait contract:**

```rust
#[async_trait]
pub trait ExchangeConnector: Send + Sync {
    fn exchange_id(&self) -> &'static str;
    async fn test_connection(&self) -> Result<ConnectionTest, ExchangeError>;
    async fn sync_balances(&self, state: &mut ExchangeSyncState) -> Result<Vec<ExchangeHolding>, ExchangeError>;
    async fn sync_positions(&self, state: &mut ExchangeSyncState) -> Result<Vec<ExchangeHolding>, ExchangeError>;
    async fn sync_trades(&self, state: &mut ExchangeSyncState) -> Result<Vec<ExchangeTrade>, ExchangeError>;
    async fn sync_transfers(&self, state: &mut ExchangeSyncState) -> Result<Vec<ExchangeTransfer>, ExchangeError>;
    async fn sync_funding(&self, state: &mut ExchangeSyncState) -> Result<Vec<ExchangeFundingEvent>, ExchangeError>;
}
```

**Read-only enforcement (DEC-0037, DEC-0004):**

- HTTP layer allows **GET only**; compile-time + integration audit rejects POST/PUT/DELETE paths
- `test_connection` validates balance read; flag withdraw-enabled keys when detectable

**Repository methods:**

- `upsert_holdings`, `upsert_trades`, `upsert_transfers`, `upsert_funding`
- `get_sync_state`, `update_sync_state` with watermarks: `last_trade_time`, `last_transfer_time`, `last_funding_time`
- First run: 90-day backfill + 1-day overlap

### Done when

- [ ] `ExchangeConnector` trait and normalized DTOs defined
- [ ] HTTP client enforces GET-only with audit helper
- [ ] HMAC signing helper stubs per exchange (used by T-0076)
- [ ] Repository upserts holdings/trades and manages watermarks
- [ ] Unit test: GET-only audit rejects write methods
- [ ] 429 backoff implemented in shared HTTP layer

---

## T-0076 — Binance Bybit and Bitunix connector implementations

**Status:** open  
**Depends on:** T-0075  
**Decisions:** DEC-0037, R-0032

### Description

Implement three connectors under `backend/src/exchanges/`:

| Connector | Scope | Endpoints |
|-----------|-------|-----------|
| `binance` | Spot + USD-M futures | `api.binance.com`, `fapi.binance.com`, wallet `sapi` |
| `bybit` | V5 UNIFIED (spot + linear) | Configurable `base_url` for regional hosts |
| `bitunix` | Spot-first; futures stub | `openapi.bitunix.com`; `enabled_futures=false` default |

**Per connector:**

- `test_connection` — balance/position read; connection latency
- `sync_balances` / `sync_positions` — normalized holdings
- `sync_trades` — incremental from watermark; Binance `myTrades` limited to symbols with non-zero balance or open positions (R-0032 fan-out mitigation)
- `sync_transfers` — deposits/withdrawals where API supports
- `sync_funding` — funding fee/income ledger (Bybit/Binance futures; Bitunix stub when futures disabled)

**ExchangeService shell (`exchanges/service.rs`):**

- Factory builds enabled connectors from config + env secrets
- `list_connections()` — metadata without secrets
- `test_connection(exchange_id)` — delegates to connector

Register connectors in `AppState.exchanges`.

### Done when

- [ ] All three connectors implement `ExchangeConnector`
- [ ] Binance spot + USD-M positions and trades import
- [ ] Bybit UNIFIED balances, positions, trades import
- [ ] Bitunix spot balances and trades import; futures stub returns empty when disabled
- [ ] Binance trade sync limited to active symbols
- [ ] `ExchangeService::test_connection` works for each enabled exchange
- [ ] Mock-server or recorded fixture tests for signing (especially Bitunix double SHA256)

---

## T-0077 — Portfolio Engine hybrid PnL and baselines

**Status:** open  
**Depends on:** T-0073, T-0074, T-0076  
**Decisions:** DEC-0038, DEC-0039, R-0033

### Description

Implement `backend/src/portfolio/`:

| Submodule | Responsibility |
|-----------|----------------|
| `pnl` | Hybrid realized/unrealized/total-return computation |
| `avg_cost` | Spot average-cost from imported trades when exchange lacks PnL fields |
| `baseline` | First-sync baseline per enabled exchange |
| `repository` | Upsert `portfolio_pnl_snapshots`; read holdings/trades |
| `service` | `PortfolioEngine::recompute_pnl(run_id)` |

**Hybrid methodology (DEC-0038):**

| Metric | Primary source | Fallback |
|--------|----------------|----------|
| Unrealized | Exchange position fields | `qty × mark − avg_cost × qty` |
| Realized | Exchange cumulative + funding ledgers | Local sum from trades since watermark |
| Total return | `(current_eur − baseline_eur) / baseline_eur` | Baseline = first successful sync snapshot |

- Funding fees: separate `ExchangeFundingEvent` rows; included in realized subtotal
- Reconciliation: when exchange cumulative ≠ local sum by >1%, prefer **exchange cumulative** + log `pnl_reconciliation_warning`
- Cross-exchange aggregation: sum **after** EUR conversion via `FxService`

**Baseline capture:** on first successful sync per exchange, record `portfolio_baselines` EUR total.

Wire `PortfolioEngine` into `AppState.portfolio`.

### Done when

- [ ] `recompute_pnl` produces realized, unrealized, total_return_pct per run
- [ ] Exchange-reported PnL preferred when available
- [ ] Avg-cost fallback computes spot unrealized when exchange fields missing
- [ ] Baseline captured on first sync; total return computed vs baseline
- [ ] Funding events included in realized subtotal
- [ ] Reconciliation warning logged when drift >1%
- [ ] Unit tests cover hybrid paths, avg-cost, and baseline capture

---

## T-0078 — Sync pipeline exchanges phase and ExchangeService

**Status:** open  
**Depends on:** T-0076, T-0077  
**Decisions:** DEC-0041, DEC-0042, R-0036

### Description

Extend sync pipeline (DEC-0028) with `"exchanges"` phase **before** `"alerts"`:

```
1. Firefly sync                 (phase: "sync")
2. Subscription detection       (phase: "subscriptions")
3. Forecast recompute           (phase: "forecast")
   └─ inline active plan refresh
4. Exchange sync                (phase: "exchanges")     ← NEW
   └─ PortfolioEngine::recompute_pnl
5. Net worth snapshot + alerts  (phase: "alerts")
6. Clear mutex
```

**ExchangeService::run_post_sync(run_id):**

```rust
for connector in enabled_connectors {
    connector.sync_balances → positions → trades → transfers → funding;
    upsert watermarks;
}
PortfolioEngine::recompute_pnl(run_id);
WealthService::upsert_daily_snapshot(/* includes crypto EUR */);
```

**Scheduler (DEC-0041):**

- Exchange interval: `[exchanges] interval_seconds` (default same as Firefly; operator may set longer e.g. 3600)
- Exchange-only tick → run phases **4–5** only; skip 1–3 when Firefly not due

**Manual triggers:**

- `POST /api/v1/sync/trigger` — full pipeline 1–5
- `POST /api/v1/sync/exchanges/trigger` — phases 4–5 only

**Failure semantics:** single exchange failure → mark `connection_state=error`; continue others; alerts run with partial crypto + `fx_incomplete`.

**Trade retention startup job:** prune `exchange_trades` older than `[portfolio] trade_retention_days`.

Extend `WealthService::upsert_daily_snapshot` to populate `crypto_value_eur`, `firefly_value_eur`, `total_return_pct`, extended `payload.crypto`.

Log exchanges phase duration separately for mutex monitoring.

### Done when

- [ ] `"exchanges"` phase runs after forecast/plan hook and before alerts
- [ ] Exchange-only scheduler tick runs phases 4–5 only
- [ ] `POST /api/v1/sync/exchanges/trigger` triggers partial run
- [ ] Single exchange failure does not block others
- [ ] Daily net worth snapshot includes crypto EUR columns and payload
- [ ] Trade retention prune job runs at startup
- [ ] Phase timing logged for exchanges step

---

## T-0079 — Extended wealth portfolio REST API and allocation_target template

**Status:** open  
**Depends on:** T-0077, T-0078  
**Decisions:** DEC-0042, DEC-0035, R-0031, R-0037

### Description

**REST API handlers** (JWT-protected; secrets never in responses):

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/exchanges` | List connections (enabled, state, last_sync, counts) |
| POST | `/api/v1/exchanges/{id}/test` | Read-only connection test |
| POST | `/api/v1/sync/exchanges/trigger` | Manual exchange-only sync |
| GET | `/api/v1/wealth` | Extended breakdown (firefly + crypto + PnL) |
| GET | `/api/v1/wealth/crypto` | Crypto-focused payload |
| GET | `/api/v1/wealth/history?days=90` | Trend incl. `crypto_value_eur`, `total_return_pct` |
| GET | `/api/v1/portfolio/pnl?days=90` | PnL snapshot series for charts |

**Wealth breakdown response:**

```json
{
  "reporting_currency": "EUR",
  "firefly": { "subtotal_eur": 42000, "mixed_currency": false, "accounts": [...] },
  "crypto": { "subtotal_eur": 8500, "fx_complete": true, "exchanges": [...], "holdings_top": [...] },
  "total_eur": 50500,
  "pnl": { "realized_eur": 1200, "unrealized_eur": 800, "total_return_pct": 12.4 },
  "fx_incomplete": false
}
```

- Replace crypto placeholder when any exchange connected
- Headline `total_eur = firefly.subtotal_eur + crypto.subtotal_eur` when FX complete

**Plan allocation template (DEC-0042):**

Extend Plan Engine adjustment kind **`allocation_target`**:

```json
{ "kind": "allocation_target", "weights": { "etf_traditional_pct": 50, "crypto_pct": 50, "cash_pct": 0 } }
```

- Current allocation buckets from wealth API: `etf_traditional` (Firefly assets), `crypto`, `cash` (cashWallet + stablecoins)
- Add template preset on Planning Scenarios tab
- API endpoint or wealth payload includes current vs target gap when active plan has `allocation_target`

**Extend `get_portfolio` AI tool:** crypto totals + top **5** holdings; stay within 8 KB cap (DEC-0035).

### Done when

- [ ] All REST endpoints return expected payloads without secrets
- [ ] `/wealth` includes firefly + crypto + PnL breakdown
- [ ] `/wealth/crypto` returns per-exchange holdings and PnL summary
- [ ] `/wealth/history` includes crypto columns
- [ ] `allocation_target` plan kind persists and computes gap vs current allocation
- [ ] `get_portfolio` tool includes crypto totals + top-5 holdings within 8 KB
- [ ] Integration tests for wealth API shape (mock or DATABASE_URL)

---

## T-0080 — React /wealth Crypto tab and Overview extension

**Status:** open  
**Depends on:** T-0079  
**Decisions:** DEC-0042, R-0026

### Description

Extend existing `/wealth` page (US-0005) with **Crypto** tab (Overview | Crypto):

| UI element | Implementation |
|------------|----------------|
| Overview tab | Combined `total_eur` stat; Firefly + Crypto subtotal cards; remove placeholder when connected |
| Crypto tab | Per-exchange stat cards (connection state, last sync); holdings table (asset, qty, value EUR, unrealized PnL) |
| PnL summary row | Realized / unrealized / total return % |
| FX incomplete banner | Lists unpriced assets; crypto subtotal excludes them |
| Empty state | "No exchanges connected" + link to Settings |
| Allocation card | Current vs target weights when active plan has `allocation_target`; link to `/planning` |
| Grafana link | External link to Dashboard 4 |

**Data:** TanStack Query → `/api/v1/wealth`, `/api/v1/wealth/crypto`.

Preserve US-0005 Overview behavior (account breakdown, alert links) when no exchanges connected.

Optional ECharts: mini performance sparkline from `/wealth/history` (defer if tight — table sufficient for MVP).

### Done when

- [ ] Overview tab shows combined total when exchanges connected
- [ ] Crypto placeholder row removed when live data present
- [ ] Crypto tab renders per-exchange cards and holdings table
- [ ] PnL summary row displays realized/unrealized/total return
- [ ] FX incomplete banner shown when `fx_incomplete=true`
- [ ] Allocation gap card links to `/planning` when plan has target
- [ ] Empty state with Settings link when no exchanges
- [ ] `npm run build` succeeds

---

## T-0081 — Settings crypto exchanges and Sync Status per-exchange rows

**Status:** open  
**Depends on:** T-0079  
**Decisions:** DEC-0040, R-0035

### Description

**Settings — Crypto exchanges section** (read-only, mirror US-0006 AI & Privacy pattern):

| Display | Source |
|---------|--------|
| Binance/Bybit/Bitunix enabled | TOML `[exchanges.*].enabled` |
| Configured badge | env vars named in TOML resolve non-empty |
| Last test result | cached from `POST .../test` |
| Operator doc link | env setup + `.env.example` template |

**Actions:**

- "Test connection" button → `POST /api/v1/exchanges/{id}/test` (does not persist secrets)
- No in-browser secret entry; copy explains TOML/env edit + restart

**Sync Status extension** — per-exchange rows:

| Column | Source |
|--------|--------|
| Exchange name | `exchange_connections.id` |
| Status badge | `connection_state` |
| Last sync | `last_sync_at` |
| Entity counts | balances, positions, trades, transfers, funding |
| Error | redacted `last_error` |
| Action | "Sync exchanges now" → `POST /api/v1/sync/exchanges/trigger` |

Phase indicator: `"Syncing exchanges…"` during phase 4.

Update `.env.example` with `BINANCE_*`, `BYBIT_*`, `BITUNIX_*` placeholders.

### Done when

- [ ] Settings page includes Crypto exchanges section
- [ ] Enabled/configured badges accurate from settings API
- [ ] Test connection button works without exposing secrets
- [ ] Restart-required note present for credential changes
- [ ] Sync Status shows per-exchange rows with counts and status
- [ ] "Sync exchanges now" triggers partial sync
- [ ] `.env.example` documents exchange env vars

---

## T-0082 — Grafana Dashboard 4 completion

**Status:** open  
**Depends on:** T-0078  
**Decisions:** DEC-0042, R-0026

### Description

Replace US-0005 partial placeholder panels in `grafana/dashboards/portfolio.json` (uid `portfolio`):

| Panel | SQL / source |
|-------|--------------|
| Total wealth stat | `net_worth_snapshots.total_eur` (now includes crypto) |
| Crypto value stat | `net_worth_snapshots.crypto_value_eur` |
| Firefly vs crypto allocation pie | `payload->'allocation'` or computed from columns |
| Portfolio performance time series | `total_return_pct` or `total_eur` over time |
| Account breakdown table | Firefly accounts + exchange summary from payload |
| FX incomplete text panel | When `payload->>'fx_incomplete' = 'true'` |

Remove crypto placeholder text panel when data present.

Verify datasource reuse from US-0001; dashboard provisioning unchanged (DEC-0012 pattern).

### Done when

- [ ] Dashboard 4 JSON updated with crypto stat panel
- [ ] Allocation pie panel queries firefly vs crypto split
- [ ] Performance time series panel shows total return or total EUR trend
- [ ] FX incomplete panel displays when applicable
- [ ] Placeholder "Connect exchanges" panel removed or hidden when crypto data exists
- [ ] `docker compose --profile minimal config` validates Grafana mount

---

## T-0083 — Exchange portfolio and get_portfolio tests

**Status:** open  
**Depends on:** T-0075, T-0076, T-0077, T-0078, T-0079, T-0080, T-0081  
**Decisions:** DEC-0037, DEC-0038, DEC-0040, DEC-0004

### Description

Add Rust unit and integration tests:

**Exchange read-only:**

- GET-only HTTP audit rejects POST/PUT/DELETE paths
- Connector integration test with mock server (Binance HMAC, Bitunix double SHA256)

**FxService:**

- Frankfurter conversion; unpriced alt returns incomplete flag
- Stablecoin → EUR path

**Portfolio Engine:**

- Hybrid PnL: exchange-reported vs avg-cost fallback
- Baseline capture and total return calculation
- Reconciliation warning when drift >1%

**Sync pipeline:**

- Exchanges phase ordering: after forecast, before alerts
- Partial exchange-only trigger runs phases 4–5

**Wealth API:**

- Breakdown shape includes crypto subtotal
- Secrets absent from all exchange/wealth responses

**get_portfolio tool:**

- Crypto totals + top-5 holdings within 8 KB cap

**Integration test (`exchanges_portfolio_integration`):**

- With `DATABASE_URL`: migration 007 + exchange upsert + PnL recompute + wealth snapshot
- Skip without `DATABASE_URL` (same pattern as US-0001–US-0006)

Extend `tests/run-tests.sh` to include exchange/portfolio test targets.

### Done when

- [ ] GET-only audit test passes for exchange HTTP layer
- [ ] Mock signing tests pass for at least Binance and Bitunix
- [ ] Portfolio PnL unit tests cover hybrid methodology
- [ ] Sync phase ordering unit test passes
- [ ] Wealth API integration test validates crypto in snapshot (or SKIP without DATABASE_URL)
- [ ] Static audit: no secrets in API response fixtures
- [ ] `get_portfolio` 8 KB cap test passes
- [ ] `bash tests/run-tests.sh` includes new tests and passes

---

## T-0084 — Operator user guide

**Status:** open  
**Depends on:** T-0080, T-0081, T-0082, T-0083  
**Decisions:** —

### Description

Create `docs/user-guides/US-0007.md` per USER_GUIDE_MODE=1:

- Prerequisites: US-0001–US-0006 operational; read-only exchange API keys
- Enabling exchanges: TOML `[exchanges.*]` + env vars (`BINANCE_*`, `BYBIT_*`, `BITUNIX_*`); restart required
- Read-only key setup per exchange (Binance "Enable Reading", Bybit readOnly=1, Bitunix no trade/withdraw)
- Sync: full vs exchange-only trigger; recommended interval for rate limits
- `/wealth` Overview + Crypto tab; PnL summary interpretation (wealth analytics, not tax)
- FX incomplete banner meaning
- Allocation target template on `/planning`; gap display on `/wealth`
- Settings exchange section (read-only display; test connection)
- Sync Status per-exchange rows
- Grafana Dashboard 4 panels
- Troubleshooting: connection errors, rate limits, first-sync avg-cost limitation, mutex latency
- Secret rotation: env update + container restart
- AI `get_portfolio` now includes crypto when connected
- Out of scope: additional exchanges, on-chain wallets, tax reporting

### Done when

- [ ] User guide covers all six acceptance criteria from operator perspective
- [ ] Exchange env setup and read-only key requirements documented
- [ ] Sync intervals and manual trigger documented
- [ ] PnL methodology and FX incomplete banner explained
- [ ] Allocation scenario workflow documented
- [ ] Secret rotation and restart workflow documented

---

## Execution order (recommended)

1. **Database + config:** T-0073 → T-0074
2. **Exchange foundation:** T-0075 → T-0076
3. **Portfolio + sync:** T-0077 → T-0078
4. **API + plan:** T-0079
5. **Frontend:** T-0080 → T-0081
6. **Grafana:** T-0082 (parallel with frontend after T-0078)
7. **Verification:** T-0083 → T-0084

## Acceptance coverage map

| AC | Tasks |
|----|-------|
| Binance/Bybit/Bitunix connectors import data | T-0073, T-0075, T-0076, T-0078, T-0081, T-0083 |
| Portfolio Engine PnL (realized/unrealized/total return) | T-0077, T-0079, T-0080, T-0083 |
| Crypto in net worth view | T-0077, T-0078, T-0079, T-0080, T-0082 |
| Portfolio allocation scenarios | T-0079, T-0080, T-0084 |
| Grafana Dashboard 4 crypto + performance | T-0078, T-0082, T-0084 |
| API keys self-hosted secrets only | T-0074, T-0075, T-0079, T-0081, T-0083, T-0084 |
