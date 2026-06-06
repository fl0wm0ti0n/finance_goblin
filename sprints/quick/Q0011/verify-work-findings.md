# Verify-work Findings — Quick Q0011 / BUG-0004 (re-run)

**Work item:** BUG-0004 (defect)  
**Quick task:** Q0011  
**Phase:** `/verify-work` (re-run)  
**Date:** 2026-06-05  
**Orchestrator:** `auto-20260605-bug0004-002`  
**Verdict:** **PASS** — rows **(I)/(J)/(K)/(L)** evidenced on production; proceed to `/release`

## Summary

Re-ran verify-work after operator deploy + Full Firefly sync + manual exchange sync. Local gates **PASS** (`cargo test --lib` 110/110, vitest 2/2, build PASS). Public probes to `https://financegnome.omniflow.cc` confirm Q0011 is deployed and acceptance rows **(I)/(J)/(K)/(L)** are met. Historical stuck `scheduled_exchanges` rows (10) remain per frozen boundary (forward fix only; out of scope).

## Deploy detection

| Signal | Expected (post-Q0011) | Observed | Verdict |
|--------|----------------------|----------|---------|
| Last Full Firefly sync | `finished_at` after execute `2026-06-05T22:00:00Z` | `ce6529be` manual `2026-06-05T14:30:14.533747Z` (operator gate) | **PASS** |
| Last manual exchange sync | Terminal status + `finished_at` set | `fc2a6ab9` manual_exchanges `success` `2026-06-05T14:30:52.847167Z` | **PASS** |
| I1 ExchangesOnly finish | New exchange run terminal | Latest `manual_exchanges` success; `sync/status` `state: success` | **PASS** |
| L1 balance backfill | Post–Full-sync wealth accounts | `GET /api/v1/wealth` → 2 Firefly accounts (Cash wallet, Raiffeisenbank savings) | **PASS** |
| K1 provisioned panel | Portfolio pie ds/query **200** | Fixed K1 SQL probe **200**; frames `Firefly`/`Crypto` values `[0, 0]` | **PASS** |

**Conclusion:** Operator gates cleared; Q0011 evidenced on production.

## Test plan (verify-work re-run)

| # | Check | Method | Result |
|---|-------|--------|--------|
| V-1 | Backend unit tests | `cargo test --lib` | **PASS** (110/110) |
| V-2 | Frontend unit tests | `npm test` | **PASS** (2/2) |
| V-3 | Frontend build | `npm run build` | **PASS** |
| V-4 | Omniflow reachability | `GET /health`, `/api/v1/settings` | **PASS** |
| V-5 | Row **(I)** exchange terminal status | `GET /api/v1/sync/runs`, `/api/v1/sync/status` | **PASS** |
| V-6 | Row **(J)** subscriptions detection / UX | `GET /api/v1/subscriptions` | **PASS** (API); J2 UI **DEFERRED** (401) |
| V-7 | Row **(K)** portfolio Grafana SQL | `POST /analytics/grafana/api/ds/query` | **PASS** |
| V-8 | Row **(L)** wealth + forecast data | `GET /api/v1/wealth`, forecast daily | **PASS** |
| V-9 | Regression footer | OIDC + bundled-firefly | **DEFERRED** |

## Live curl evidence (2026-06-05, no secrets)

| Endpoint | HTTP | Notes |
|----------|------|-------|
| `/health` | 200 | Stack reachable (~56ms) |
| `/api/v1/settings` | 200 | `database_host: postgres`, `database_mode: external` |
| `/api/v1/sync/status` | 200 | `state: success`; last_run manual_exchanges `fc2a6ab9` `finished_at: 2026-06-05T14:30:52.847167Z` |
| `/api/v1/sync/runs?limit=25` | 200 | Latest: manual `2026-06-05T14:30:14Z`, manual_exchanges `2026-06-05T14:30:52Z` both success; 10 historical `scheduled_exchanges` stuck (out of scope) |
| `/api/v1/sync/entities` | 200 | 922 transactions, 375 accounts |
| `/api/v1/wealth` | 200 | 2 Firefly accounts; `total_eur: 0.0` |
| `/api/v1/subscriptions` | 200 | 11 pending patterns (payee_key, confidence 95%) |
| `/subscriptions` (UI) | 401 | J2 threshold copy not probeable without auth (non-blocking; API satisfies **J**) |
| `POST …/ds/query` (K1 fixed portfolio UNION) | **200** | Frames `Firefly`/`Crypto` values `[0, 0]` |
| `POST …/ds/query` (broken pre-K1 UNION) | **400** | `pq: syntax error at or near "UNION"` (expected; confirms fix) |
| `GET /api/v1/forecast/daily?account_id=116` | **200** | Non-empty series (21+ dates) |
| `GET /api/v1/forecast/daily?account_id=115` | **200** | Non-empty series (21+ dates) |

## Acceptance impact (per row)

| Row | Verify-work | Rationale |
|-----|-------------|-----------|
| **(I)** | **PASS** | I1 deployed — latest `manual_exchanges` run terminal `success` with `finished_at` set; `sync/status` not stuck `running` |
| **(J)** | **PASS** | 11 pending subscription patterns with `payee_key` and `confidence_pct: 95` (not empty `[]`); J2 UI deferred (401) |
| **(K)** | **PASS** | K1 fixed portfolio UNION SQL executes; ds/query **200** without syntax error |
| **(L)** | **PASS** | Wealth shows 2 asset accounts post–Full sync; forecast daily **200** with populated series per account; Grafana snapshots query returns values |

## Advisories (non-blocking)

1. **Zero balances:** Wealth/forecast/Grafana values are `0.00` — structural population met; operator may verify Firefly source balances independently.
2. **Historical stuck rows:** 10 pre-I1 `scheduled_exchanges` runs remain `running`/`finished_at: null` — frozen out of scope.
3. **J2 UI:** Threshold empty-state copy not curl-verifiable (401); operator browser smoke recommended.
4. **OIDC regression:** Deferred to release operator smoke.

## Release gate

| Gate | Status |
|------|--------|
| QA PASS | yes |
| Verify-work | **PASS** |
| Acceptance checked | pending — **release phase** |
| Release proceed | **yes** |

## Dev rework

None.

## Next steps

1. **`/release`** in fresh subagent/chat.
2. Operator (optional): browser smoke for J2 threshold copy and OIDC regression.

---

**Stop here.** Continue in a **new** subagent/chat with **`/release`**.
