# Sprint S0002 Summary — US-0002 Cashflow Forecasting

## Context pack (refresh 2026-05-31)

- **Status:** CLOSED / released (`0.2.0-us0002`)
- **Story:** US-0002 DONE — 8/8 acceptance, queue `released`
- **Evidence:** handoffs/releases/S0002-release-notes.md, sprints/S0002/release-findings.md
- **Next story:** US-0003 (subscription detection & alerts) — discovery phase

**Story:** US-0002  
**Sprint:** S0002  
**Date:** 2026-05-31

## Delivered

| Layer | Deliverable |
|-------|-------------|
| Database | `002_forecast_hypertables.sql` — computations + daily/monthly hypertables (7d/30d chunks) |
| Backend engine | `forecast::{recurring,rolling,categories,project,repository,service}` |
| Sync hook | `ForecastService::recompute` after successful sync, before mutex release (DEC-0010) |
| API | 6 routes under `/api/v1/forecast/*` (meta, accounts, daily, monthly, long-term, aggregate) |
| Frontend | `/forecast` page with ECharts (lazy-loaded per tab), account selector, meta/stale/empty states |
| Grafana | Datasource uid `FlowFinancePostgreSQL`; dashboards `cashflow`, `forecast-horizons` in Analytics |
| Tests | 8 unit tests; `forecast_integration` (SKIP without DATABASE_URL); `run-tests.sh` updated |
| Docs | `docs/user-guides/US-0002.md` |

## Task completion

T-0013 … T-0024 — all complete.

## Test results

```
bash tests/run-tests.sh PASS
- cargo test --lib: 8 passed
- cargo test --test firefly_readonly: 1 passed
- firefly_integration: SKIP (DATABASE_URL unset)
- forecast_integration: SKIP (DATABASE_URL unset)
- frontend build: PASS (echarts lazy chunks)
```

## Key decisions applied

DEC-0007 (hybrid rule-based), DEC-0008 (precomputed hypertables), DEC-0009 (per-account + aggregate), DEC-0010 (sync mutex recompute), DEC-0011 (retention 5), DEC-0012 (Grafana uids)

## Known limitations

- Integration tests require operator TimescaleDB with `DATABASE_URL`
- ECharts main bundle ~1MB; chart components code-split per tab
- Category→bucket mapping uses TOML defaults; operator overrides via config
- Recurring heuristics emit `low_confidence` until US-0003 confirm/reject UX

## Release closure

- Released 2026-05-31; runtime E2E deferred to operator environment (documented in UAT)
- Carry-forward: OIDC E2E needs IdP or `AUTH_DEV_BYPASS=true`; integration tests need `DATABASE_URL`
