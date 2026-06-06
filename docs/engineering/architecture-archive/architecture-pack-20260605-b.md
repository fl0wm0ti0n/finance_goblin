# Architecture archive pack (2026-06-05)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 5
- First archived heading: `## US-0007 — Crypto exchange portfolio integration`
- Last archived heading: `## US-0007 — Crypto exchange portfolio integration`
- Verification tuple (mandatory):
  - archived_body_lines=406
  - retained_body_lines=2500
  - preamble_lines=865

---

## US-0007 — Crypto exchange portfolio integration

**Status:** architecture complete (2026-06-01)  
**Research:** R-0032, R-0033, R-0034, R-0035, R-0036, R-0037 (extends R-0021, R-0024, R-0026, R-0031, DEC-0004, DEC-0021, DEC-0028)  
**Decisions:** DEC-0037, DEC-0038, DEC-0039, DEC-0040, DEC-0041, DEC-0042  
**Spec-pack:** `docs/engineering/spec-pack/US-0007-{design-concept,crs,technical-specification}.md`  
**Depends on:** US-0001 platform + sync mutex; US-0005 `WealthService` + `net_worth_snapshots` + Dashboard 4 partial; US-0004 Plan Engine adjustments; US-0006 `get_portfolio` tool (extends, no new tools)

### System context

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Browser — /wealth (Overview | Crypto tabs) + Settings Crypto exchanges    │
│            Sync Status per-exchange rows + "Sync exchanges now"              │
│            /planning allocation gap card (optional link from /wealth)        │
└───────────────────────────────┬──────────────────────────────────────────────┘
                                │ JWT Bearer
                                ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum)                                                       │
│  Sync mutex: sync → subscriptions → forecast (+ plan hook)                   │
│              → exchanges → alerts (net worth + evaluate) → done               │
│                                                                               │
│  ┌─────────────────┐   ┌──────────────────┐   ┌─────────────────────────┐  │
│  │ ExchangeService │──▶│ PortfolioEngine  │──▶│ WealthService (extended)│  │
│  │ + 3 connectors  │   │ PnL + baselines  │   │ firefly + crypto EUR    │  │
│  └────────┬────────┘   └────────┬─────────┘   └───────────┬─────────────┘  │
│           │                     │                          │                 │
│           │              ┌──────▼──────┐                   │                 │
│           │              │  FxService  │◀── Frankfurter + exchange tickers   │
│           │              └─────────────┘                   │                 │
│  exchange_holdings/trades/pnl_snapshots ───────────────────┘                 │
│  AlertService::run_post_sync (unchanged contract; richer snapshot input)     │
└───────────────────────────────┬──────────────────────────────────────────────┘
                                │ HTTPS (server-side only; env secrets)
                                ▼
              Binance / Bybit / Bitunix REST APIs (read-only keys)
```

### Components

#### 1. Exchange connectors (`backend/src/exchanges/`)

Unified async trait per exchange (**DEC-0037**, **R-0032**). Three implementations: `binance`, `bybit`, `bitunix`.

| Submodule | Responsibility |
|-----------|----------------|
| `trait` | `ExchangeConnector` — `test_connection`, `sync_balances`, `sync_positions`, `sync_trades`, `sync_transfers`, `sync_funding` |
| `types` | Normalized DTOs: `ExchangeHolding`, `ExchangeTrade`, `ExchangeTransfer`, `ExchangeFundingEvent`, `ExchangeSyncState` |
| `http` | Shared reqwest client; HMAC signing helpers per exchange; 429 backoff |
| `binance` | Spot `api.binance.com`, USD-M `fapi.binance.com`, wallet `sapi` |
| `bybit` | V5 UNIFIED account; configurable `base_url` for regional hosts |
| `bitunix` | Spot-first `openapi.bitunix.com`; futures stub behind `enabled_futures=false` default |
| `repository` | Upsert holdings/trades; read/write `exchange_sync_state` watermarks |
| `service` | `ExchangeService` — orchestrates enabled connectors; aggregates sync metadata |

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

**Read-only enforcement (DEC-0037, DEC-0004 pattern):**
- Connector HTTP layer allows **GET only**; audit rejects POST/PUT/DELETE URL paths at compile-time + integration test
- Operator keys: Read-only mandatory (Binance "Enable Reading", Bybit `readOnly=1`, Bitunix no trade/withdraw)
- `test_connection` validates balance read + flags withdraw-enabled keys when detectable

**MVP product scope (DEC-0037):**
- Binance: spot wallet + USD-M futures
- Bybit: UNIFIED (spot + linear)
- Bitunix: **spot-first**; futures positions/funding behind config flag default off

**Incremental sync:** per-exchange watermarks `last_trade_time`, `last_transfer_time`, `last_funding_time` in `exchange_sync_state.payload`; first run 90-day backfill + 1-day overlap. Binance `myTrades` fan-out limited to symbols with non-zero balance or open positions.

**Alternative considered:** CCXT library — rejected (Rust ecosystem thin; hides PnL field mapping; R-0032).

#### 2. Portfolio Engine (`backend/src/portfolio/`)

Computes wealth analytics PnL — **not tax reporting** (**DEC-0038**, **R-0033**).

| Submodule | Responsibility |
|-----------|----------------|
| `pnl` | Hybrid realized/unrealized/total-return computation |
| `avg_cost` | Spot average-cost from imported trades when exchange lacks PnL fields |
| `baseline` | First-sync baseline per enabled exchange for total return % |
| `repository` | Upsert `portfolio_pnl_snapshots`; read holdings/trades |
| `service` | `PortfolioEngine::recompute_pnl(run_id)` called after exchange sync |

**Hybrid methodology (DEC-0038):**

| Metric | Primary source | Fallback |
|--------|----------------|----------|
| Unrealized | Exchange position fields (`unrealisedPnl`, `totalUnrealizedProfit`) | `qty × mark − avg_cost × qty` |
| Realized | Exchange cumulative + income/funding ledgers | Local sum from trades since watermark |
| Total return | `(current_eur − baseline_eur) / baseline_eur` | Baseline = first successful sync snapshot |

- Funding fees: separate `ExchangeFundingEvent` rows; included in realized subtotal, separate UI line
- Reconciliation: when exchange cumulative ≠ local sum by >1%, prefer **exchange cumulative** + log `pnl_reconciliation_warning` on Sync Status
- Cross-exchange aggregation: sum **after** EUR conversion via `FxService` (DEC-0039)

**Alternative considered:** Full local FIFO tax lots — rejected (out of scope; sparse first-sync history).

#### 3. FX service (`backend/src/fx/`)

Two-layer conversion to EUR reporting currency (**DEC-0039**, **R-0034**).

| Layer | Assets | Source | Cache |
|-------|--------|--------|-------|
| Fiat/stablecoin | USDT, USDC, USD, GBP | Frankfurter ECB daily | `fx_rates` table; 24h TTL |
| Crypto alts | BTC, ETH, alts | Exchange ticker `{ASSET}USDT` × USDT/EUR | Per sync mark-to-market |
| Bybit coins | Unified wallet | Response `usdValue` × USD/EUR | Per sync |

```rust
FxService::to_eur(amount, asset, price_book: &ExchangePriceBook) -> Result<EurAmount, FxError>
```

- Missing price for illiquid alt → exclude from crypto subtotal; set `fx_incomplete=true` + banner listing assets (extends US-0005 mixed-currency pattern)
- Frankfurter: default public `api.frankfurter.dev`; optional sidecar in Compose `standard` profile documented
- CoinGecko fallback: **deferred** post-MVP

**Alternative considered:** Manual TOML price map — rejected (operator burden).

#### 4. Migration `007_exchanges_portfolio.sql` (**DEC-0042**, **R-0037**)

| Object | Purpose |
|--------|---------|
| `exchange_connections` | Per-exchange enabled/state/last_sync metadata |
| `exchange_sync_state` | Watermarks JSON per exchange |
| `exchange_holdings` | Normalized balances/positions with EUR marks |
| `exchange_trades` | Trade history; unique `(exchange_id, external_id)` |
| `exchange_transfers` | Deposits/withdrawals (optional MVP stretch) |
| `exchange_funding_events` | Funding fee / income ledger rows |
| `portfolio_pnl_snapshots` | Daily PnL aggregate (`UNIQUE(snapshot_date)`) |
| `portfolio_baselines` | First-sync baseline per exchange for total return |
| `fx_rates` | Daily fiat pair cache |
| `net_worth_snapshots` **ALTER** | Add `crypto_value_eur`, `firefly_value_eur`, `total_return_pct`; extend `payload` |

Indexes: `exchange_holdings(exchange_id)`, `exchange_trades(exchange_id, executed_at DESC)`, `exchange_trades` retention policy 2 years (startup prune job).

Seed: `INSERT INTO exchange_connections (id) VALUES ('binance'), ('bybit'), ('bitunix') ON CONFLICT DO NOTHING`.

**Alternative considered:** Separate `portfolio_snapshots` hypertable — rejected (duplicate daily grain with `net_worth_snapshots`; R-0037).

#### 5. Sync pipeline — `"exchanges"` phase (**DEC-0041**, **R-0036**)

Extends DEC-0028 pipeline:

```
1. Firefly sync                 (phase: "sync")
2. Subscription detection       (phase: "subscriptions")
3. Forecast recompute           (phase: "forecast")
   └─ inline active plan refresh (DEC-0023)
4. Exchange sync                (phase: "exchanges")     ← NEW
   └─ PortfolioEngine::recompute_pnl
5. Net worth snapshot + alerts  (phase: "alerts")
6. Clear mutex
```

**Why before alerts:** `WealthService::upsert_daily_snapshot` must include crypto EUR subtotal in `total_eur` and extended `payload.crypto` (R-0024).

**Scheduler (DEC-0041):**
- Firefly interval: existing `[sync] interval_seconds`
- Exchange interval: `[exchanges] interval_seconds` (default same as Firefly; operator may set longer e.g. 3600)
- Exchange-only tick → run phases **4–5** only (`exchanges` → `alerts`); skip 1–3 when Firefly not due

**Manual triggers:**
- `POST /api/v1/sync/trigger` — full pipeline 1–5
- `POST /api/v1/sync/exchanges/trigger` — phases 4–5 only

**Failure semantics:** single exchange failure → mark `connection_state=error`; continue others; alerts run with partial crypto + `fx_incomplete`; run `success_with_warnings` when Firefly+forecast OK.

**ExchangeService contract:**

```rust
ExchangeService::run_post_sync(run_id) -> ExchangeSyncResult {
    for connector in enabled_connectors {
        connector.sync_balances → positions → trades → transfers → funding;
        upsert watermarks;
    }
    PortfolioEngine::recompute_pnl(run_id);
}
```

**Alternative considered:** Exchanges after alerts — rejected (crypto excluded from snapshot; violates acceptance).

#### 6. Wealth extension (`backend/src/wealth/`)

Extends US-0005 `WealthService` (**DEC-0042**, **R-0021**, **R-0037**).

**Breakdown response:**

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

- Replace crypto placeholder row with live exchange totals when any exchange connected
- Headline `total_eur = firefly.subtotal_eur + crypto.subtotal_eur` (when FX complete)
- Daily snapshot `payload` adds `{ crypto_holdings[], allocation: { etf_traditional, crypto, cash }, exchanges[] }`

**`get_portfolio` tool (US-0006):** extend `WealthService::compute_breakdown` output — crypto totals + top **5** holdings; stay within 8 KB cap (DEC-0035, R-0031).

#### 7. Plan allocation scenarios (**DEC-0042**, **R-0037**, **R-0018**)

Extend Plan Engine adjustment kind **`allocation_target`** on existing `plan_adjustments`:

```json
{ "kind": "allocation_target", "weights": { "etf_traditional_pct": 50, "crypto_pct": 50, "cash_pct": 0 } }
```

**Current allocation buckets (from wealth API):**
- `etf_traditional` = Firefly asset subtotal EUR (non-cash asset accounts)
- `crypto` = crypto subtotal EUR
- `cash` = Firefly cashWalletAsset + stablecoin holdings

**MVP UX:** read-only gap display on `/wealth` Crypto tab or allocation card linking to `/planning`; Planning Scenarios tab supports allocation template preset.

**Alternative considered:** Dedicated `allocation_targets` table — rejected (plan adjustment kind sufficient for MVP).

#### 8. Exchange & portfolio REST API

All routes JWT-protected (DEC-0006). Secrets never in responses (DEC-0040).

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/exchanges` | List connections (enabled, state, last_sync, counts — no secrets) |
| POST | `/api/v1/exchanges/{id}/test` | Read-only connection test (balance endpoint) |
| POST | `/api/v1/sync/exchanges/trigger` | Manual exchange-only sync (phases 4–5) |
| GET | `/api/v1/wealth` | Extended breakdown (firefly + crypto + PnL) |
| GET | `/api/v1/wealth/crypto` | Crypto-focused payload (holdings, per-exchange, PnL) |
| GET | `/api/v1/wealth/history?days=90` | Trend incl. `crypto_value_eur`, `total_return_pct` |
| GET | `/api/v1/portfolio/pnl?days=90` | PnL snapshot series for charts |

Settings API extension: `[exchanges]` read-only display (enabled flags, env var **names**, masked configured badge — no secret values).

#### 9. React `/wealth` — Crypto tab (**R-0026** completion)

Enable **Crypto** tab on existing `/wealth` page (Overview | Crypto).

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

#### 10. Settings — Crypto exchanges (**DEC-0040**, **R-0035**)

New Settings section **Crypto exchanges** — **read-only** (mirror US-0006 AI & Privacy pattern):

| Display | Source |
|---------|--------|
| Binance/Bybit/Bitunix enabled | TOML `[exchanges.*].enabled` |
| Configured badge | env vars named in TOML resolve non-empty |
| Last test result | cached from `POST .../test` |
| Operator doc link | env setup + `.env.example` template |

**No in-browser secret entry.** Test connection + enable/disable toggles require TOML/env edit + container restart (document in user guide).

#### 11. Sync Status extension

Per-exchange rows on existing Sync Status page:

| Column | Source |
|--------|--------|
| Exchange name | `exchange_connections.id` |
| Status badge | `connection_state` |
| Last sync | `last_sync_at` |
| Entity counts | balances, positions, trades, transfers, funding |
| Error | redacted `last_error` |
| Action | "Sync exchanges now" → `POST /api/v1/sync/exchanges/trigger` |

Phase indicator: `"Syncing exchanges…"` during phase 4.

#### 12. Grafana Dashboard 4 completion (**DEC-0042**, **R-0026**)

Replace US-0005 partial placeholder panels in `portfolio.json` (uid `portfolio`):

| Panel | SQL / source |
|-------|--------------|
| Total wealth stat | `net_worth_snapshots.total_eur` (now includes crypto) |
| Crypto value stat | `net_worth_snapshots.crypto_value_eur` |
| Firefly vs crypto allocation pie | `payload->'allocation'` or computed from columns |
| Portfolio performance time series | `total_return_pct` or `total_eur` over time |
| Account breakdown table | Firefly accounts + exchange summary from payload |
| Mixed-currency / FX incomplete | Text panel when `payload->>'fx_incomplete' = 'true'` |

Remove crypto placeholder text panel when data present.

### Backend module layout

| Module | Responsibility |
|--------|----------------|
| `exchanges::{trait,types,http,binance,bybit,bitunix,repository,service}` | Connectors + sync orchestration |
| `portfolio::{pnl,avg_cost,baseline,repository,service}` | Portfolio Engine |
| `fx::{repository,service}` | EUR conversion + rate cache |
| `wealth::service` | Extend breakdown + snapshot payload |
| `plan::templates` | Add `allocation_target` template preset |
| `api::exchanges`, `api::wealth` (extend), `api::portfolio` | Axum handlers |
| `sync` | Add `"exchanges"` phase; exchange-only partial run |
| `config` | `[exchanges]`, `[portfolio]` TOML sections |

`AppState` gains `exchanges: ExchangeService`, `portfolio: PortfolioEngine`, `fx: FxService`.

### Config additions (TOML)

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
# price_fallback = "coingecko"  # deferred post-MVP

[wealth]
snapshot_retention_days = 365
```

Compose env example: `BINANCE_API_KEY`, `BINANCE_API_SECRET`, etc. — never in TOML plaintext.

### Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Binance symbol fan-out latency | Sync symbols with balance/positions only; longer exchange interval | R-0032, R-0036 |
| Bitunix signing complexity | Spot-first; futures stub default off; dedicated integration test | R-0032 |
| Avg-cost inaccuracy first sync | 90-day backfill; document limitation; prefer exchange PnL when available | R-0033 |
| FX incomplete illiquid alts | Exclude from subtotal + banner | R-0034 |
| Mutex duration >45s | Log phase timing; recommend longer exchange interval | R-0036, DEC-0018 |
| Secret rotation | Document env update + restart flow | R-0035 |
| PnL reconciliation drift | Prefer exchange cumulative; operator warning on Sync Status | R-0033 |
| USDT≠USD peg drift | Document ±0.5% wealth-view caveat | R-0034 |
| `get_portfolio` payload overflow | Top-5 holdings + summarize; 8 KB cap | R-0031, DEC-0035 |

### Decisions (US-0007)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0037 | Exchange connectors | Unified `ExchangeConnector` trait; GET-only; spot+linear MVP; Bitunix spot-first |
| DEC-0038 | PnL methodology | Hybrid exchange-reported + avg-cost spot; wealth analytics not tax |
| DEC-0039 | FX conversion | Frankfurter fiat/stablecoin; exchange tickers for alts; `fx_incomplete` banner |
| DEC-0040 | Exchange secrets | TOML env names + Compose env; Settings read-only; test-connection only |
| DEC-0041 | Sync exchanges phase | Inline `"exchanges"` before `"alerts"`; independent interval + partial trigger |
| DEC-0042 | Migration 007 schema | Holdings/trades/PnL tables; extend `net_worth_snapshots`; `allocation_target` kind |

Full records: `decisions/DEC-0037.md` … `decisions/DEC-0042.md`

### Out of scope (US-0007)

- Additional exchanges (Kraken, Coinbase, Bitpanda, OKX)
- On-chain wallet tracking
- Tax reporting / FIFO tax lots
- Trading execution or order placement
- Encrypted DB secret vault
- Settings runtime credential edit in browser
- CoinGecko price fallback (defer post-MVP)
- Binance Portfolio Margin (`papi`), options, inverse-only advanced modes
- New AI tools (extends `get_portfolio` only)
- Any write to Firefly III

### Next phase

`/sprint-plan` — S0007 task decomposition against 6 acceptance criteria.

---

