# Verify-work Findings — Quick Q0012 / BUG-0005 (re-run)

**Work item:** BUG-0005 (defect)  
**Quick task:** Q0012  
**Phase:** `/verify-work` (re-run)  
**Date:** 2026-06-05  
**Orchestrator:** `auto-20260605-bug0005-002`  
**Verdict:** **PASS** — rows **(M)/(N)/(O)** evidenced on production; proceed to `/release`

## Summary

Re-ran verify-work after operator deploy of Q0012 backend + manual exchange sync (`f0906348`, `2026-06-05T15:29:06Z`). Local gates **PASS** (`cargo test --lib` 123/123, vitest 2/2, build PASS). Public probes to `https://financegnome.omniflow.cc` confirm Q0012 deploy signals, dual-path Bitunix test, post-deploy exchange sync, non-zero Bitunix holdings, and wealth crypto population. Operator gates **cleared**.

## Deploy detection

| Signal | Expected (post-Q0012) | Observed | Verdict |
|--------|----------------------|----------|---------|
| Settings `bitunix.futures_base_url` | `"https://fapi.bitunix.com"` | `"https://fapi.bitunix.com"` | **PASS** |
| Settings `bitunix.enabled_futures` (effective) | `true` when configured + creds | `true` | **PASS** |
| N4 dual test message | `Spot: OK; Futures: …` | `"Spot: OK; Futures: OK"` (533ms) | **PASS** |
| Last manual exchange sync | `finished_at` after Q0012 deploy | `f0906348` `2026-06-05T15:29:06.929489Z` success | **PASS** |
| Non-spot holdings | `holdings_count` > 0 after futures-enabled sync | Bitunix `holdings: 4`; `exchange_bitunix` entity count 4 (was 0 pre-Q0012) | **PASS** |
| Wealth crypto population | `holdings_count` > 0; `crypto_placeholder: false` | `holdings_count: 4`; `crypto_placeholder: false`; `last_successful_sync_at` aligned | **PASS** |

**Conclusion:** Q0012 backend evidenced on production; operator deploy + exchange sync gates cleared.

## Test plan (verify-work re-run)

| # | Check | Method | Result |
|---|-------|--------|--------|
| V-1 | Backend unit tests | `cargo test --lib` | **PASS** (123/123) |
| V-2 | Frontend unit tests | `npm test` | **PASS** (2/2) |
| V-3 | Frontend build | `npm run build` | **PASS** |
| V-4 | Omniflow reachability | `GET /health`, `/api/v1/settings` | **PASS** |
| V-5 | Q0012 deploy detection | Settings + test endpoint signals | **PASS** |
| V-6 | Post-deploy exchange sync | `GET /api/v1/sync/runs`, `/api/v1/sync/status` | **PASS** |
| M-1 | Row **(M)** non-spot holdings | `GET /api/v1/exchanges`, `/api/v1/sync/entities` | **PASS** |
| N-1 | Row **(N)** futures auth + test | Settings + `POST …/bitunix/test` | **PASS** |
| O-1 | Row **(O)** wealth crypto subtotal | `GET /api/v1/wealth`, `/api/v1/wealth/crypto` | **PASS** |
| REG-1 | OIDC regression | Browser smoke | **DEFERRED** |
| REG-2 | Bundled-firefly regression | Settings external profile | **DEFERRED** (external profile confirmed) |

## Live curl evidence (2026-06-05, no secrets)

| Endpoint | HTTP | Notes |
|----------|------|-------|
| `/health` | 200 | `{"status":"ok"}` (~57ms) |
| `/api/v1/settings` | 200 | `bitunix.futures_base_url: "https://fapi.bitunix.com"`; `enabled_futures: true`; `database_mode: external`, `database_host: postgres` |
| `/api/v1/sync/status` | 200 | `state: success`; last_run `f0906348` manual_exchanges `finished_at: 2026-06-05T15:29:06.929489Z` |
| `/api/v1/sync/runs?limit=5` | 200 | Latest `f0906348` success; prior `fc2a6ab9` pre-Q0012 era |
| `POST /api/v1/exchanges/bitunix/test` | 200 | `ok: true`, `message: "Spot: OK; Futures: OK"`, `latency_ms: 533` |
| `/api/v1/exchanges` | 200 | Bitunix `connected`; `counts.holdings: 4`; `last_sync_at: 2026-06-05T15:29:06.906163Z` |
| `/api/v1/sync/entities` | 200 | `exchange_bitunix` count **4** (was 0 pre-Q0012) |
| `/api/v1/wealth` | 200 | `crypto.holdings_count: 4`; `crypto_placeholder: false`; `last_successful_sync_at: 2026-06-05T15:29:06.929489Z` |
| `/api/v1/wealth/crypto` | 200 | Bitunix `holdings_count: 4`; `subtotal_eur: 0.0` (DEC-0064: linear positions unpriced; wallet-only subtotal) |

## Acceptance impact (per row)

| Row | Verify-work | Rationale |
|-----|-------------|-----------|
| **(M)** | **PASS** | Post-Q0012 sync ingests 4 Bitunix holdings (was 0); futures path active (`enabled_futures: true`, Futures test OK). Architecture O1: `product_type` not exposed on list API; entity count + futures-enabled sync satisfies non-spot ingestion when operator has futures exposure |
| **(N)** | **PASS** | `futures_base_url` on `fapi.bitunix.com`; effective `enabled_futures: true`; dual-path test **200** with Futures sub-status |
| **(O)** | **PASS** | Wealth crypto `holdings_count: 4` > 0; `crypto_placeholder: false`; combined spot + futures visibility per DEC-0064 (subtotal from priced wallet rows; linear positions counted but unpriced — acceptable) |
| Regression | **DEFERRED** | External profile reachable; OIDC/browser smoke deferred (non-blocking) |

## Operator advisory (non-blocking)

1. `crypto.subtotal_eur: 0.0` with `holdings_count: 4` — consistent with DEC-0064 (linear positions excluded from subtotal; no priced wallet rows in top list). Verify operator-visible futures wallet USDT if non-zero subtotal expected.
2. 10+ historical `scheduled_exchanges` stuck `running` rows remain — frozen out of scope (forward fix only).
3. Browser smoke for OIDC regression if OIDC enabled.

## Dev rework

None — Q0012 runtime acceptance met.

## Release gate

| Gate | Status |
|------|--------|
| QA PASS | yes |
| Verify-work | **PASS** |
| Acceptance checked | release phase — M/N/O evidenced |
| Release proceed | **yes** |

## Next steps

1. **`/release`** — check BUG-0005 acceptance checkbox; publish release notes
2. Operator browser smoke (OIDC) — advisory only

---

**Stop here.** Continue in a **new** subagent/chat with **`/release`**.
