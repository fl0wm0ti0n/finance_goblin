# Verify-work Findings — Quick Q0009 / BUG-0003

**Work item:** BUG-0003  
**Quick task:** Q0009  
**Phase:** `/verify-work` (re-run)  
**Date:** 2026-06-05  
**Orchestrator:** `auto-20260605-bug0003-002`  
**Verdict:** **PASS** — acceptance rows **(F)**, **(G)**, **(H)** evidenced on deployed omniflow stack; release may proceed

## Summary

Re-ran verify-work after operator F1 recovery (prior BLOCKED 2026-06-04: `host.docker.internal` mis-host, disk full, stack 404). Local gates PASS (`cargo test --lib` 103/103, vitest 2/2, build PASS). Live curl on `https://financegnome.omniflow.cc` confirms Q0009 fixes deployed and F1 complete: `database_host: postgres`, representative GET APIs **200** &lt;0.1s (not 500 ~30s), Bitunix test **200**, Grafana `ds/query` **200** with `SELECT 1`.

## Test plan (verify-work re-run)

| # | Check | Method | Result |
|---|-------|--------|--------|
| V-1 | Backend unit tests | `cargo test --lib` | **PASS** (103/103) |
| V-2 | Frontend unit tests | `npm test` | **PASS** (2/2) |
| V-3 | Frontend build | `npm run build` | **PASS** |
| V-4 | F2 doc guardrail | `.env.example` + runbook + compose comment | **PASS** (static) |
| V-5 | G1 `effective_enabled` | Unit test + live bitunix test | **PASS** |
| V-6 | Row **(F)** live smoke | Public curl | **PASS** |
| V-7 | Row **(G)** Bitunix test | `POST …/bitunix/test` | **PASS** |
| V-8 | Row **(H)** Grafana SQL | `POST …/analytics/grafana/api/ds/query` | **PASS** |
| V-9 | Regression footer | OIDC + bundled-firefly | **DEFERRED** (non-blocking) |

## Live curl evidence (2026-06-05 re-run, no Traefik credentials)

| Endpoint | HTTP | Latency | Notes |
|----------|------|---------|-------|
| `/health` | 200 | ~97ms | Stack reachable |
| `/api/v1/settings` | 200 | ~57ms | `database_host`: **`postgres`**, `database_mode`: `external` |
| `/api/v1/alerts/unread-count` | **200** | ~74ms | `{"count":0}` — not 500 ~30s |
| `/api/v1/sync/entities` | **200** | ~73ms | 922 transactions, 375 accounts |
| `/api/v1/sync/runs` | **200** | ~58ms | Recent success run |
| `/api/v1/exchanges` | **200** | ~63ms | Bitunix connected |
| `/api/v1/subscriptions` | **200** | ~60ms | DB-backed list |
| `/api/v1/ai/audit` | **200** | ~56ms | DB-backed audit log |
| `/api/v1/plans/risk-score` | **200** | ~57ms | `no_score` (regression sanity) |
| `POST /api/v1/exchanges/bitunix/test` | **200** | ~333ms | `{"ok":true,"message":"Spot balance read OK"}` |
| `POST /analytics/grafana/api/ds/query` | **200** | ~95ms | `SELECT 1 AS ok` → frame value `[[1]]` |

**Note:** `GET /api/v1/forecast/daily` without `account_id` returns **400** validation error (~58ms) — expected contract, not DB timeout cascade.

## Acceptance impact

| Row | Verify-work | `acceptance.md` checkbox |
|-----|-------------|---------------------------|
| **(F)** | **PASS** | Ready for release checkbox |
| **(G)** | **PASS** | Ready for release checkbox |
| **(H)** | **PASS** | Ready for release checkbox |
| Regression | **DEFERRED** | Operator browser advisory |

## Prior baseline resolution

| Prior issue (2026-06-04) | Status |
|--------------------------|--------|
| `database_host: host.docker.internal` | **Resolved** → `postgres` |
| GET APIs **500** ~30s DB timeout | **Resolved** → **200** &lt;0.1s |
| Bitunix test **400** unknown exchange | **Resolved** → **200** connection OK |
| Grafana ds/query **400/404** | **Resolved** → **200** SQL executes |
| Host disk full / stack **404** | **Resolved** — stack healthy |

## Release gate

| Gate | Status |
|------|--------|
| QA PASS | yes |
| Verify-work PASS | **yes** |
| Acceptance checked | **pending** `/release` |
| Release proceed | **yes** |

## Next steps

1. **`/release`** in fresh subagent — check BUG-0003 acceptance checkbox.
2. Operator advisory: OIDC regression browser smoke (non-blocking).
