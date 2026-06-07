# Architecture archive pack (2026-06-06)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 14
- First archived heading: `## BUG-0005 — Exchange sync multi-product (Bitunix futures)`
- Last archived heading: `## BUG-0005 — Exchange sync multi-product (Bitunix futures)`
- Verification tuple (mandatory):
  - archived_body_lines=272
  - preamble_lines=10
  - retained_body_lines=2922

---

## BUG-0005 — Exchange sync multi-product (Bitunix futures)

**Status:** architecture complete (2026-06-05)  
**Discovery:** `discovery-20260605-bug0005` in `handoffs/po_to_tl.md`  
**Research:** [R-0058](research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation), [R-0059](research.md#r-0059--exchange-multi-product-sync-scope-bitunix-futures)  
**Decisions:** **DEC-0062** (dual REST auth); **DEC-0063** (`effective_enabled_futures`); **DEC-0064** (wallet vs position wealth accounting); extends **DEC-0037** (connector trait), **DEC-0038** (PnL/wealth boundary), **DEC-0039** (FX), **DEC-0041** (exchanges sync phase)  
**Sprint:** `/quick` **Q0012** (recommended)  
**Acceptance:** `docs/product/acceptance.md` rows **M**, **N**, **O**  
**Related:** BUG-0003 G2 fulfilled separately (registry); BUG-0004 DONE — wealth pipeline ready; do not merge BUG-0006

### Symptom chain (frozen)

Operator on US-0010 external profile: Bitunix credentials configured; spot test **200**; post-sync holdings and wealth crypto slice **empty or spot-only**. Three connector gaps; wealth under-report is downstream, not a separate aggregation bug:

| Sub | Gap | Effect |
|-----|-----|--------|
| **M** | `sync_balances` → `/api/spot/v1/user/account` only; all rows `product_type: "spot"` | Futures/margin wallet never ingested |
| **N** | `enabled_futures=false` default; `sync_positions`/`sync_funding` no-op stubs; spot query-sign client incompatible with `fapi.bitunix.com` header auth | Futures REST never called |
| **O** | `WealthService` sums all holdings `market_value_eur` — spot-only DB rows | `crypto.subtotal_eur` under-reports operator futures exposure |

`isolation_scope`: artifact + repo source reads + public curl probes (R-0058/R-0059); no host `.env` / `.env_prod` secrets read.

### Fix slices

```text
BUG-0005
├── N — Futures REST infrastructure (P0, first)
│   ├── N1 — Header-auth HTTP client + futures_base_url config (DEC-0062)
│   └── N3 — effective_enabled_futures policy + settings exposure (DEC-0063)
├── M — Futures wallet ingestion (P0)
│   └── M1 — sync_balances futures account path → product_type futures
├── N — Positions + connectivity (P0)
│   ├── N2 — sync_positions via get_pending_positions → product_type linear (DEC-0064)
│   └── N4 — Dual-path test_connection (spot + futures sub-status)
└── O — Operator verify (P1)
    └── O1 — omniflow probes rows M/N/O (post deploy + exchange sync)
```

**Deploy order:** N1 + N3 + M1 + N2 + N4 in one PR (backend config + connector); deploy image; operator **manual exchange sync**; O1 verify-work.

**Deferred (out of sprint):** `sync_funding` implementation; Bybit `product_type` relabeling; Binance futures balance completeness; multi margin-coin iteration beyond USDT.

### Endpoint map (frozen)

| Purpose | Host | Method | Path | Auth |
|---------|------|--------|------|------|
| Spot balance (existing) | `openapi.bitunix.com` | GET | `/api/spot/v1/user/account` | Query `timestamp` + `sign` (`bitunix_sign`) |
| Futures wallet | `fapi.bitunix.com` | GET | `/api/v1/futures/account?marginCoin=USDT` | Headers per DEC-0062 |
| Open positions | `fapi.bitunix.com` | GET | `/api/v1/futures/position/get_pending_positions` | Headers per DEC-0062 |

Config: add `futures_base_url = "https://fapi.bitunix.com"` to `[exchanges.bitunix]` in `default.toml` and `BitunixConfig`.

### N1 — Futures header-auth client — DEC-0062 (frozen)

**Problem:** Shipped `signed_get` builds spot query-string auth (`bitunix_sign` on query + `sign` param). Futures private REST requires header auth on `fapi.bitunix.com` per [R-0058](research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation) and [official sign doc](https://www.bitunix.com/api-docs/futures/common/sign.html).

**Contract:**

| Concern | Spot path (unchanged) | Futures path (new) |
|---------|----------------------|-------------------|
| Base URL | `spot_base_url` | `futures_base_url` |
| Sign input | Query string incl. `timestamp` | `digest = SHA256(nonce + timestamp + api-key + queryParams + body)` |
| Sign output | `bitunix_sign(secret, query)` → query param | `sign = SHA256(hex(digest) + secretKey)` → header |
| Headers | `api-key` only | `api-key`, `nonce` (32 chars), `timestamp` (ms), `sign` |
| HTTP | GET via `ExchangeHttpClient::get_with_backoff` | Same client; GET-only audit unchanged (DEC-0037) |

**Implementation:**

1. Add `bitunix_futures_sign(secret, nonce, timestamp_ms, api_key, query_params, body) -> String` in `backend/src/exchanges/http.rs`.
2. Add `BitunixConnector::futures_signed_get(path, query) -> Result<Value, ExchangeError>` — generate 32-char nonce (e.g. alphanumeric), ms timestamp, empty body for GET.
3. Add `futures_base_url: String` to `BitunixConfig` with default `https://fapi.bitunix.com`.
4. **Do not** reuse `bitunix_sign` for futures — different canonical string (R-0058).

**Alternatives rejected:**

- *Single client with host sniffing* — fragile; explicit dual-path matches Binance spot/fapi split pattern.
- *CCXT* — rejected (DEC-0037 / R-0032).
- *Futures-only rewrite* — rejected; spot path works on omniflow; preserve regression when futures opt-out.

**Files:** `backend/src/exchanges/http.rs`, `backend/src/exchanges/bitunix.rs`, `backend/src/config/mod.rs`, `backend/config/default.toml`

**Risks:** Sign canonical string drift vs Bitunix doc — unit test with published fixture; API error bodies may leak key hints — log status only.

### N3 — effective_enabled_futures policy — DEC-0063 (frozen)

**Problem:** `enabled_futures = false` in `default.toml` gates all futures sync despite operator credentials present; settings expose raw TOML flag → operator sees `enabled_futures: false` while expecting multi-product sync.

**Contract:**

```rust
// BitunixConfig
pub fn effective_enabled_futures(&self) -> bool {
    // Env override (highest priority)
    if let Ok(v) = std::env::var("BITUNIX_ENABLED_FUTURES") {
        if matches!(v.to_ascii_lowercase().as_str(), "false" | "0" | "no") {
            return false;
        }
        if matches!(v.to_ascii_lowercase().as_str(), "true" | "1" | "yes") {
            return true;
        }
    }
    if self.enabled_futures {
        return true;
    }
    // Auto-enable when exchange effectively enabled with credentials
    self.effective_enabled() && self.credentials().is_some()
}
```

| Surface | Expose |
|---------|--------|
| `GET /api/v1/settings` → `exchanges.bitunix.enabled_futures` | **`effective_enabled_futures()`** (runtime truth) |
| TOML `enabled_futures` | Remains `false` default — explicit opt-in still works |
| Env `BITUNIX_ENABLED_FUTURES` | Document in `.env.example`; explicit opt-out for spot-only operators |

**Gate usage:** `sync_balances` futures branch, `sync_positions`, `sync_funding` stub guard, and N4 futures probe all use `effective_enabled_futures()` — not raw `enabled_futures`.

**Alternatives rejected:**

- *TOML default flip to `true`* — breaks spot-only deploys without env; effective override is safer.
- *Require manual operator flag only* — rejected; root cause is silent default blocking ingestion (acceptance **N**).

**Files:** `backend/src/config/mod.rs`, `backend/config/default.toml`, settings view assembly, `.env.example`

**Risks:** Operator with spot-only intent must set `BITUNIX_ENABLED_FUTURES=false` — document in runbook; auto-enable only when credentials present limits blast radius.

### M1 — Futures wallet balance ingestion (frozen)

**Problem:** `sync_balances` (`bitunix.rs` L77–119) calls spot account only; all holdings tagged `product_type: "spot"`.

**Contract:**

When `effective_enabled_futures()`:

1. Call `futures_signed_get("/api/v1/futures/account", "marginCoin=USDT")`.
2. Parse wallet fields from `data` (tolerate nested `account` object per API variance):
   - Margin coin asset (default **USDT** MVP — single coin per discovery defer list)
   - Equity quantity from `available` + `frozen` + `margin` or `accountEquity` / `totalEquity` (use first present numeric field; log warn on ambiguity)
3. Emit `ExchangeHolding` per non-zero margin coin:

| Field | Value |
|-------|-------|
| `asset` | Margin coin symbol (e.g. `USDT`) |
| `quantity` | Wallet equity qty |
| `product_type` | `"futures"` |
| `market_value_usd` | `quantity` for stablecoins; else None pending FX |
| `unrealized_pnl` | From API field if present (`unrealizedPnl` / `crossUnPnl`) |
| `payload` | Raw JSON fragment |

4. **Append** to spot holdings from existing spot path (do not replace spot sync).
5. When `effective_enabled_futures()` is false, spot-only path unchanged (regression).

**Alternatives rejected:**

- *Positions-only, skip wallet* — fails acceptance **M** (margin account balances).
- *Multi margin-coin loop* — deferred; USDT-only MVP.

**Files:** `backend/src/exchanges/bitunix.rs`

**Risks:** API field rename — defensive parsing + fixture tests; zero wallet with open positions still valid (N2 covers exposure rows).

### N2 — sync_positions — DEC-0064 (frozen)

**Problem:** `sync_positions` (`bitunix.rs` L122–129) returns `Ok(vec![])` even when `enabled_futures` true.

**Contract:**

When `effective_enabled_futures()`:

1. `futures_signed_get("/api/v1/futures/position/get_pending_positions", "")`.
2. For each open position with non-zero size:

| Field | Value |
|-------|-------|
| `asset` | Symbol / pair identifier from API |
| `quantity` | `abs(position size)` |
| `product_type` | `"linear"` (Binance parity) |
| `market_value_usd` | **`None`** (DEC-0064 — avoid double-count with wallet equity) |
| `unrealized_pnl` | From API unrealized field |
| `payload` | Raw position JSON |

3. Push symbols to `state.active_symbols` for trade sync watermark (existing pattern).
4. When futures disabled via `effective_enabled_futures()`, return `Ok(vec![])` unchanged.

**Wealth accounting (DEC-0064):**

- **Wealth crypto subtotal** sums `market_value_eur` on holdings — futures **wallet** rows (M1) contribute; **linear** position rows do not (null market value → excluded from subtotal, visible in holdings list / PnL via DEC-0038 portfolio engine).
- Aligns with Binance pattern: spot balances priced; positions carry unrealized PnL without duplicating wallet equity.

**Alternatives rejected:**

- *Price positions into wealth subtotal* — double-counts wallet equity that already embeds unrealized PNL.
- *Skip positions, wallet only* — fails acceptance **M** when operator holds open contracts.

**Files:** `backend/src/exchanges/bitunix.rs`

**Risks:** Operator sees holdings_count > 0 but subtotal from wallet only — acceptable; O1 verifies combined spot + futures visibility.

### N4 — Dual-path test_connection (frozen)

**Problem:** `test_connection` probes spot only; message `"Spot balance read OK"` masks futures auth failure.

**Contract:**

1. Always probe spot (`/api/spot/v1/user/account`) — existing behavior.
2. When `effective_enabled_futures()`, additionally probe futures account (`/api/v1/futures/account?marginCoin=USDT`).
3. Compose `ConnectionTest.message`:

| Spot | Futures | `ok` | Message pattern |
|------|---------|------|-----------------|
| OK | OK | `true` | `Spot: OK; Futures: OK` |
| OK | fail | `true` | `Spot: OK; Futures: {err}` (partial — acceptance **N** allows read-only key scope) |
| fail | * | `false` | `Spot: {err}` (futures skipped or appended) |

4. `latency_ms` = total elapsed for both probes.
5. **Do not** extend `ConnectionTest` struct — message string sufficient for MVP (alternatives rejected: structured sub-status JSON — heavier API churn).

**Files:** `backend/src/exchanges/bitunix.rs`

**Risks:** Partial OK may hide misconfigured futures keys — O1 + settings `enabled_futures` effective flag surfaces runtime truth.

### O1 — Operator verify-work (frozen)

**Precondition:** Deploy N1–N4; operator runs **manual exchange sync** on `financegnome.omniflow.cc`.

| Row | Probe | Pass |
|-----|-------|------|
| **M** | `GET /api/v1/exchanges` → bitunix holdings after sync | ≥1 holding with `product_type` ∈ `{futures, linear}` when operator has futures exposure |
| **N** | `POST /api/v1/exchanges/bitunix/test` | **200**; message mentions Futures; settings `enabled_futures: true` (effective) |
| **O** | `GET /api/v1/wealth` | `crypto.subtotal_eur` reflects futures wallet rows; bitunix `holdings_count` > 0 when exposure exists |
| Regression | footer | OIDC + bundled-firefly checks |

**Files:** `sprints/quick/Q0012/uat.md`

**Operator gate:** Exchange sync required before O1 (not Full Firefly sync — unlike BUG-0004).

### Task map (Q0012)

| Order | Task | Layer | Acceptance |
|-------|------|-------|------------|
| 1 | **N1** futures header-auth client (DEC-0062) | backend exchanges | **N** |
| 2 | **N3** effective_enabled_futures (DEC-0063) | backend config | **N** |
| 3 | **M1** futures wallet ingestion | backend bitunix | **M** |
| 4 | **N2** sync_positions (DEC-0064) | backend bitunix | **M**, **N** |
| 5 | **N4** dual-path test_connection | backend bitunix | **N** |
| 6 | **O1** verify-work omniflow probes | verify-work | **M**, **N**, **O** |

**Count:** 6 tasks (≤ `SPRINT_MAX_TASKS` 12) → **`/quick` Q0012**; no split.

### Test strategy

| Check | Type | Pass criteria |
|-------|------|---------------|
| N1 | Unit | Futures sign canonical string matches fixture; spot sign unchanged |
| N1 | Mock HTTP | Futures GET receives `api-key`, `nonce`, `timestamp`, `sign` headers |
| N3 | Unit | Creds present + TOML false → effective true; env `false` → false |
| M1 | Mock HTTP | Account JSON → holding `product_type: "futures"` |
| N2 | Mock HTTP | Positions JSON → holding `product_type: "linear"`, `market_value_usd: None` |
| N4 | Unit | Spot OK + futures fail → `ok: true`, message contains both |
| Regression | Unit | `effective_enabled_futures()` false → spot-only sync unchanged |
| O1 | verify-work | Omniflow rows M/N/O + regression footer |

### Frozen boundaries

- No merge with BUG-0006 transaction ingest
- No `sync_funding` implementation this sprint
- USDT margin coin only — no multi-coin config loop
- Read-only GET-only connector guarantee unchanged (DEC-0037)
- Do not modify Binance/Bybit connectors except import patterns as reference

---

