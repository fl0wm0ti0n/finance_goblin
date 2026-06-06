# PO to TL archive pack (2026-06-05)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 6
- Retained units in hot file: 7
- First archived heading: `## intake-20260605-bug0007 — BUG-0007 AI merchant/category discovery`
- Last archived heading: `## intake-20260605-bug0005 — BUG-0005 exchange sync multi-product (Bitunix futures)`
- Verification tuple (mandatory):
  - archived_body_lines=388
  - retained_body_lines=485

---

## intake-20260605-bug0007 — BUG-0007 AI merchant/category discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Work item:** BUG-0007 (defect)  
**Next phase:** `/discovery`

### Summary

Operator report (production AI chat on `financegnome.omniflow.cc`, post-BUG-0006): AI finds ~**200 €/month** cancelable streaming via **`get_subscriptions`**, but when asked *"liste mir die dienste auf"* cannot enumerate specific services—only **Netflix ~6,37 €** plus generic list; asks user which services they have. Separate queries for **Strom** (electricity) and **Amazon Jan–Oct 2023** return "no expenses / miscategorized / no bookings" despite operator expectation that mirror data should support discovery from **categories, transaction names, accounts, and amounts**.

| Sub | Symptom | Intake hypothesis |
|-----|---------|-------------------|
| **S** | Subscription list failure after cancelable total | `get_subscriptions` contract lacks merchant enumeration for pending/confirmed; AI does not chain transaction payee search |
| **T** | Strom / Amazon keyword misses | Aggregate-only tools lack description/counterparty dimensions; category label mismatch; period/filter gap |
| **U** | User must name merchants | Orchestrator/tool contracts do not require cross-field fusion |
| **V** | RAG question | Discovery/architecture only — not intake fix prescription |

### Operator examples (translated intent)

1. **Streaming:** ~200 €/month cancelable → follow-up list request → "cannot retrieve specific streaming services"; Netflix ~6,37 € only; generic Netflix/Prime/Disney+ list.
2. **Strom:** No electricity expenses recorded — possibly miscategorized or no bookings.
3. **Amazon:** No Amazon expenses Jan–Oct 2023 — miscategorized or no bookings.

**Tools used:** `get_subscriptions` (2×), `get_transactions`

### Discovery guidance

1. Audit `get_subscriptions` JSON for pending vs confirmed — payee/merchant fields exposed to model.
2. Audit `get_transactions` for description/counterparty/category dimensions under `allow_raw_transactions=false`.
3. Reproduce three operator prompts on omniflow; compare tool payloads vs mirror SQL (names only, no secrets).
4. Trace subscription detection output (11 pending post-BUG-0004) vs AI enumeration path.
5. Document **RAG vs tool-enhancement** tradeoff in discovery — do not prescriptively choose at intake.

### Related (do not merge scope)

- **BUG-0006** DONE — empty aggregates/category ingest; distinct surface (merchant intelligence)
- **BUG-0004** DONE — subscription detection patterns; 0 confirmed may affect naming
- **US-0006** — six-tool registry exists; orchestration/intelligence gap

### Intake evidence

- `intake_run_id`: `intake-20260605-ai-merchant-category-discovery`
- Bundle: `handoffs/intake_evidence/intake-20260605-ai-merchant-category-discovery.json`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`

### Triad check (intake phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#BUG-0007` | Canonical bug + sub-defects S/T/U/V | pass |
| `docs/product/acceptance.md` BUG-0007 | Rows S/T/U (+ V discovery note) | pass |
| `handoffs/intake_evidence/intake-20260605-ai-merchant-category-discovery.json` | US-0078 validation OK | pass |

---

## discovery-20260605-bug0005 — BUG-0005 exchange sync multi-product (Bitunix futures)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Work item:** BUG-0005 (defect)  
**Orchestrator:** `auto-20260605-bug0005-001`  
**Next phase:** `/architecture`

### Summary

Discovery **confirms all three sub-defects M/N/O** with code traces, research [R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation) / [R-0059](docs/engineering/research.md#r-0059--exchange-multi-product-sync-scope-bitunix-futures), and public omniflow curl probes (no secrets). Shipped Bitunix connector is **spot-first MVP** — futures wallet and positions never ingest. Wealth under-report is a **downstream symptom**, not a separate aggregation bug.

### Confirmed root causes

| Sub | Root cause | Fix task |
|-----|------------|----------|
| **M** | `sync_balances` → `/api/spot/v1/user/account` only; all rows `product_type: "spot"` | **M1** |
| **N** | `enabled_futures=false` default; `sync_positions`/`sync_funding` no-op stubs; spot query-sign client incompatible with `fapi.bitunix.com` header auth | **N1**, **N2**, **N3**, **N4** |
| **O** | `WealthService` / `PortfolioEngine` sum all holdings — spot-only DB rows → low `crypto_value_eur` | **O1** (verify-work) |

### Futures endpoint map (frozen for architecture)

| Purpose | Host | Method | Path |
|---------|------|--------|------|
| Wallet balance | `fapi.bitunix.com` | GET | `/api/v1/futures/account?marginCoin=USDT` |
| Open positions | `fapi.bitunix.com` | GET | `/api/v1/futures/position/get_pending_positions` |
| Spot balance (existing) | `openapi.bitunix.com` | GET | `/api/spot/v1/user/account` |

**Auth split:** Spot = query `timestamp` + `sign` (`bitunix_sign` on query string). Futures = headers `api-key`, `nonce` (32 chars), `timestamp` (ms), `sign` where `digest=SHA256(nonce+timestamp+api-key+queryParams+body)`, `sign=SHA256(digest+secretKey)` per official futures sign doc.

### Live probe summary (2026-06-05, financegnome.omniflow.cc)

| Probe | Result |
|-------|--------|
| `GET /health` | **200** |
| `GET /api/v1/settings` → `exchanges.bitunix` | `enabled: true`, `configured: true`, `enabled_futures: false`, `spot_base_url: openapi.bitunix.com` |
| `POST /api/v1/exchanges/bitunix/test` | **200** `ok: true`, `"Spot balance read OK"` (~493 ms) |
| `GET /api/v1/exchanges` → bitunix | `connected`, `holdings: 0`, last_sync `2026-06-05T14:30:52Z` |
| `GET /api/v1/wealth` | bitunix `holdings_count: 0`, `crypto.subtotal_eur: 0` — consistent with spot-only + empty spot wallet or futures exposure absent from sync |

### Recommended sprint shape (post-architecture)

| Order | Task | Files (primary) | Acceptance |
|-------|------|-------------------|------------|
| 1 | **N1** Futures header-auth HTTP client + `futures_base_url` config | `backend/src/exchanges/http.rs`, `backend/src/exchanges/bitunix.rs`, `backend/src/config/mod.rs`, `backend/config/default.toml` | N |
| 2 | **M1** Futures account balance ingestion (`product_type: futures`/`margin`) | `backend/src/exchanges/bitunix.rs` (`sync_balances` or parallel futures fetch) | M |
| 3 | **N2** `sync_positions` via `get_pending_positions` (`product_type: linear`) | `backend/src/exchanges/bitunix.rs` | M, N |
| 4 | **N3** `enabled_futures` default policy + settings exposure | `backend/config/default.toml`, `backend/src/config/mod.rs`, settings API | N |
| 5 | **N4** Dual-path `test_connection` (spot + futures status) | `backend/src/exchanges/bitunix.rs` | N (partial failure messaging) |
| 6 | **O1** verify-work omniflow post-deploy | acceptance rows M/N/O | O |

**Tests:** unit test futures sign canonical string; mock HTTP fixtures for account + positions JSON; integration test holdings include non-spot `product_type` when futures enabled; regression spot-only path unchanged when `enabled_futures=false`.

**Deferred (out of sprint unless architecture expands):** `sync_funding` implementation; Bybit `product_type` relabeling; Binance futures balance completeness.

### Architecture open questions

1. **`enabled_futures` default:** auto-enable when Bitunix credentials present (parity with `effective_enabled`) vs explicit operator opt-in via env/TOML?
2. **Balance vs position split:** map futures wallet USDT (`available` + unrealized PNL fields) as `product_type: futures` holdings vs position rows only — avoid double-count in wealth?
3. **Margin coins:** USDT-only MVP vs iterate configured margin coins?
4. **test_connection contract:** single combined message vs structured spot/futures sub-status?

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#BUG-0005` | Discovery table + fix tasks | pass |
| `docs/product/acceptance.md` BUG-0005 | Rows M/N/O still valid | pass — no AC rewrite |
| `handoffs/intake_evidence/intake-20260605-exchange-futures-multi-product.json` | small-intake-pack complete | pass |
| [R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation), [R-0059](docs/engineering/research.md#r-0059--exchange-multi-product-sync-scope-bitunix-futures) | Discovery confirmation appended | pass |

### Related (do not merge scope)

- **BUG-0003** G2 — futures auth spike fulfilled separately; this track is product holdings ingestion
- **BUG-0004** — analytics pipeline DONE; wealth now shows Firefly accounts but crypto slice still spot-only for Bitunix
- **BUG-0006** — AI transaction path separate

---

## discovery-20260605-bug0004 — BUG-0004 post-sync pipeline empty analytics

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Work item:** BUG-0004 (defect)  
**Orchestrator:** `auto-20260605-bug0004-001`  
**Next phase:** `/architecture`

### Summary

Discovery **confirms all four sub-defects I/J/K/L** with code traces and public omniflow curl probes (no secrets). Symptoms share post-sync / analytics wiring but fixes decompose into **seven independent tasks** (I1, K1, L1, L2, J1, J2, L3 verify-work). Primary wealth/forecast emptiness is **NULL account balances** from Firefly string `current_balance` parse gap — not Grafana variable mismatch.

### Confirmed root causes

| Sub | Root cause | Fix task |
|-----|------------|----------|
| **I** | `ExchangesOnly` (`manual_exchanges`, `scheduled_exchanges`) never calls `finish_sync_run`; DB rows stuck `running` | **I1** |
| **J** | Payee grouping uses `description` only; detection only on Full sync; 0 confirmed until operator action | **J1**, **J2** |
| **K** | Invalid `UNION ALL` SQL in portfolio pie panel (ORDER BY/LIMIT per branch) | **K1** |
| **L1** | `current_balance.as_f64()` → NULL mirror balances | **L1** |
| **L2** | Wealth `load_asset_accounts` excludes NULL with `balance >= 0` | **L2** |
| **L3** | Forecast projects from `starting_balance` 0 → flat zero series | **L1** + **L2** (downstream) |

### Live probe summary (2026-06-05, financegnome.omniflow.cc)

| Probe | Result |
|-------|--------|
| `GET /api/v1/sync/status` | `success` (last Full manual run) |
| `GET /api/v1/sync/runs` | 10× `scheduled_exchanges` + 2× `manual_exchanges` stuck `running` |
| `GET /api/v1/subscriptions` | 11 pending, 0 confirmed |
| `GET /api/v1/wealth` | `accounts: []`, `total_eur: 0` |
| `GET /api/v1/forecast/daily?account_id=116` | 26 points, all balance `0.00` |
| Grafana portfolio UNION SQL | 500 `pq: syntax error at or near "UNION"` |
| Asset accounts (ds/query) | 3 asset rows, all `balance: null` |

### Recommended sprint shape (post-architecture)

| Order | Task | Files (primary) |
|-------|------|-----------------|
| 1 | **I1** Exchange sync terminal status | `backend/src/sync/mod.rs` |
| 2 | **K1** Portfolio Grafana SQL | `grafana/provisioning/dashboards/analytics/portfolio.json` |
| 3 | **L1** Account balance parse | `backend/src/firefly/mod.rs` |
| 4 | **L2** Wealth NULL balance filter | `backend/src/wealth/repository.rs` |
| 5 | **J1** Payee key fallbacks | `backend/src/recurrence/group.rs` |
| 6 | **J2** Subscriptions empty-state UX | `frontend/src/pages/SubscriptionsPage.tsx` |
| 7 | **L3** verify-work omniflow | acceptance rows I–L |

**Tests:** unit test ExchangesOnly finishes sync run; SQL fixture for portfolio pie; balance parse string/number; payee grouping with counterparty payload.

### Architecture open questions

1. Payee extraction priority: description → `payload.attributes` counterparty name → split `destination_name`?
2. Re-sync backfill: upsert overwrites NULL balances on next Full sync vs one-shot migration?
3. Stuck `sync_runs` cleanup: mark orphaned `running` rows failed on deploy or leave historical?
4. Coordinate with BUG-0006 Q2/Q3 transaction ingest fixes for subscription expense filter (`amount < 0`)?

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#BUG-0004` | Discovery table + fix tasks | pass |
| `docs/product/acceptance.md` BUG-0004 | Rows I/J/K/L still valid | pass — no AC rewrite |
| `handoffs/intake_evidence/intake-20260605-omniflow-post-sync-pipeline.json` | small-intake-pack complete | pass |
| [R-0061](docs/engineering/research.md#r-0061--post-sync-analytics-pipeline-empty-data-paths) | Discovery findings persisted | pass |

### Related (do not merge scope)

- **BUG-0006** — transaction date/amount/category ingest (overlaps subscription expense filter only)
- **BUG-0005** — exchange multi-product (crypto slice of L, separate track)

---

## discovery-20260605-bug0006 — BUG-0006 AI get_transactions empty despite sync

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Work item:** BUG-0006 (defect)  
**Next phase:** `/architecture`

### Summary

Discovery confirms **three mirror ingest gaps** and **one aggregate contract gap** that together produce the operator symptom (German "no expenses in categories" after `get_transactions` despite 922 synced transactions). No local DB probe (`DATABASE_URL` unset); root causes established by code analysis + Firefly API field conventions.

### Confirmed root causes

| Sub | Root cause | Fix task |
|-----|------------|----------|
| **Q** | `category_id` never extracted from `attributes.transactions[].category_id` and never written by `upsert_transaction` | **Q1** |
| **Q2** | Firefly ISO datetime strings fail strict `%Y-%m-%d` parse → `transactions.date` NULL → period filter returns empty `by_category` | **Q2** |
| **Q3** | Firefly stores positive split amounts; aggregates use `amount < 0` for outflow → zero outflow with non-zero count | **Q3** |
| **R** | `TransactionAggregates` lacks period totals, uncategorized bucket, and empty-state semantics | **R1** |
| **P** | LLM misreads empty/misleading tool JSON (downstream of Q/Q2/Q3/R) | **P1** (verify-work) |

### Firefly payload contract (for architecture)

| Field | Firefly path | Mirror column | Today |
|-------|--------------|---------------|-------|
| Category | `attributes.transactions[].category_id` (string) | `transactions.category_id` | **not written** |
| Date | `attributes.transactions[].date` (ISO datetime) | `transactions.date` | **parse fails → NULL** |
| Amount | `attributes.transactions[].amount` (positive) + `type` (`withdrawal`/`deposit`/`transfer`) | `transactions.amount` | **positive only; no sign norm** |
| Full journal | top-level `item` JSON | `transactions.payload` | written |

Categories reference table is synced (`sync_categories`); join key on transactions never populated.

### Aggregate contract gap (R1 scope for architecture)

Current `TransactionAggregates` (`backend/src/transactions/types.rs`):

```json
{ "period_start", "period_end", "group_by", "by_category": [...], "by_month": null, "raw_rows": null }
```

Recommended additions (privacy-safe, DEC-0032 compatible):

- `total_transaction_count`, `total_outflow`, `total_inflow` (period-level)
- `uncategorized_transaction_count` (rows with NULL `category_id`)
- `period_status`: `no_rows_in_period` | `rows_uncategorized` | `rows_with_outflow` | `rows_zero_outflow`

### Recommended quick sprint shape (Q0010)

| Order | Task | Files (primary) |
|-------|------|-------------------|
| 1 | **Q1** Category sync | `backend/src/firefly/mod.rs`, `backend/src/db/mod.rs` |
| 2 | **Q2** ISO date parse | `backend/src/firefly/mod.rs` |
| 3 | **Q3** Amount sign normalization (type-aware) | `backend/src/firefly/mod.rs`, `backend/src/transactions/repository.rs` (if query semantics change) |
| 4 | **R1** Aggregate contract + service assembly | `backend/src/transactions/types.rs`, `service.rs`, `repository.rs`, `backend/src/ai/tools/transactions.rs` |
| 5 | **P1** Operator E2E + SQL probe on omniflow | verify-work checklist |

**Tests:** unit tests for date parse, category extraction, amount sign; integration test for aggregates with fixture mirror rows.

### Architecture open questions

1. Amount normalization: negate on `withdrawal` source leg only, or use Firefly `type` + account role heuristic?
2. Re-sync strategy: one-time backfill on next sync (upsert overwrites) vs migration script for existing 922 rows?
3. Uncategorized bucket: separate `by_category` row with `category_id: null` label `"Uncategorized"` vs top-level count only?
4. Privacy: ensure new summary fields pass `PrivacyLayer` unchanged (no raw row exposure).

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#BUG-0006` | Discovery notes table + fix tasks | pass |
| `docs/product/acceptance.md` BUG-0006 | P/Q/R criteria still valid | pass — no AC rewrite |
| `handoffs/intake_evidence/intake-20260605-ai-get-transactions-empty.json` | small-intake-pack complete | pass |
| R-0060 | Findings confirmed/extended (Q2 date, Q3 amount) | pass |

### Related (do not merge scope)

- **BUG-0004** — forecast/subscriptions/Grafana pipeline
- **BUG-0002/0003/0005** — separate OPEN tracks

### Evidence

- Bundle: `handoffs/intake_evidence/intake-20260605-ai-get-transactions-empty.json`
- Research: [R-0060](docs/engineering/research.md#r-0060--ai-get_transactions-empty-aggregates-vs-mirror-sync)
- Code: `backend/src/firefly/mod.rs`, `backend/src/db/mod.rs`, `backend/src/transactions/repository.rs`, `backend/src/ai/tools/transactions.rs`

---

## intake-20260605-bug0006 — BUG-0006 AI get_transactions empty despite sync

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Work item:** BUG-0006 (defect)  
**Next phase:** `/discovery`

### Summary

Operator report: AI Chat claims *no expenses this month in categories / data unavailable* after invoking **`get_transactions`** (~23:30:13), while Sync Status shows **922 transactions** loaded.

| Sub | Symptom | Intake hypothesis |
|-----|---------|-------------------|
| **P** | AI "no expenses" answer | Tool returns empty/misleading aggregates for requested period |
| **Q** | Categories never mapped | `upsert_transaction` omits `category_id`; sync does not extract Firefly category |
| **R** | Model misreads payload | No top-level period totals; uncategorized vs empty period indistinguishable |

### Operator example (verbatim intent)

> Es wurden keine Ausgaben für diesen Monat in den Kategorien aufgezeichnet. Möglicherweise gab es keine Transaktionen oder die Daten sind nicht verfügbar.  
> Tools used: `get_transactions` — 23:30:13

### Discovery guidance

1. SQL: `SELECT COUNT(*), COUNT(category_id), MIN(date), MAX(date) FROM transactions` on production mirror.
2. Compare aggregate output for operator's month vs sync entity count.
3. Inspect Firefly transaction JSON for category field paths; fix ingest.
4. Extend `TransactionAggregates` with `total_transaction_count`, `total_outflow`, `uncategorized_count` (privacy-safe).
5. Verify amount sign convention from Firefly splits.

### Related

- **BUG-0004** — broader analytics pipeline (forecast/subscriptions/Grafana)
- **US-0006** — tool registry exists; data path defect

### Intake evidence

- Bundle: `handoffs/intake_evidence/intake-20260605-ai-get-transactions-empty.json`
- Research: [R-0060](docs/engineering/research.md#r-0060--ai-get_transactions-empty-aggregates-vs-mirror-sync)

---

## intake-20260605-bug0005 — BUG-0005 exchange sync multi-product (Bitunix futures)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Work item:** BUG-0005 (defect)  
**Next phase:** `/architecture` (discovery complete 2026-06-05 — see `discovery-20260605-bug0005` above)

### Summary

Operator requirement: **Bitunix exchange sync must not be spot-only** — futures and other account types are important for accurate wealth. Shipped connector (`bitunix.rs`) syncs only `/api/spot/v1/user/account`; futures paths are stubbed with `enabled_futures=false` default.

| Sub | Symptom | Intake hypothesis |
|-----|---------|-------------------|
| **M** | Only spot holdings after sync | `sync_balances` spot endpoint only; all `product_type: "spot"` |
| **N** | Futures wallet never populated | Separate `fapi.bitunix.com` + header auth ([R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation)); stubs in `sync_positions`/`sync_funding` |
| **O** | Wealth/crypto totals too low | Portfolio/wealth consume connector output — spot-only → under-report |

### Alternatives considered

- **Extend BUG-0003 G2 only** — rejected: G2 targets test-connection auth spike, not multi-product holdings ingestion
- **New US story** — rejected: defect vs US-0007 spot-first MVP; operator missing data today

### Intake evidence

- `intake_run_id`: `intake-20260605-exchange-futures-multi-product`
- Bundle: `handoffs/intake_evidence/intake-20260605-exchange-futures-multi-product.json`
- Research: [R-0059](docs/engineering/research.md#r-0059--exchange-multi-product-sync-scope-bitunix-futures)

### Discovery guidance

1. Map Bitunix futures account/balance/position endpoints on `fapi.bitunix.com`.
2. Implement header-auth HTTP client (or dual client) without breaking spot path.
3. Decide default for `enabled_futures` (operator opt-in vs auto when credentials present).
4. Verify wealth snapshot aggregation includes futures `product_type`.
5. Note: Binance `sync_positions` already calls `fapi` — use as pattern; Bybit unified wallet may need `product_type` labeling only.

### Related

- **BUG-0003** G1/G2 — registry + auth (prerequisite, not substitute)
- **BUG-0004** — analytics pipeline separate

---

