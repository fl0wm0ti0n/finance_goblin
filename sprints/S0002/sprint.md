# Sprint S0002

**ID:** S0002  
**Story:** US-0002 — Cashflow forecasting & Grafana analytics (MVP)  
**Status:** PLANNED  
**Created:** 2026-05-31

## Goal

Deliver the forecasting MVP on top of US-0001: hybrid rule-based Forecast Engine, TimescaleDB hypertable persistence, JWT-protected forecast API, inline recompute after Firefly sync, React `/forecast` page with ECharts, and Grafana Dashboards 1 (Cashflow) and 5 (Forecast horizons).

## Scope

- Migration `002_forecast_hypertables.sql` — `forecast_computations`, `forecast_balance_daily`, `forecast_cashflow_monthly` (DEC-0008)
- Forecast Engine: recurring inference, rolling variable residual, category buckets, day-by-day projection (DEC-0007)
- Forecast repository: hypertable writes, latest computation reads, retention of last 5 successes (DEC-0011)
- Sync recompute hook: inline after successful ingest, extends sync mutex (DEC-0010)
- REST API: `/api/v1/forecast/{meta,accounts,daily,monthly,long-term,aggregate}` (DEC-0009)
- React: enable Forecast nav; account selector; Daily | Monthly | Long-term tabs; ECharts charts
- Grafana: datasource uid `FlowFinancePostgreSQL`; dashboards uid `cashflow` and `forecast-horizons` (DEC-0012)
- Tests and operator user guide (`docs/user-guides/US-0002.md`)

**Out of scope:** ML forecasting (US-0009), subscription-adjusted forecasts (US-0003), plan overlays (US-0004), Alert Engine (US-0005), Grafana Dashboards 2–4, async Redis job queue.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Recompute blocks sync mutex | Accept for MVP; log duration per account | DEC-0010, R-0007 |
| Recurring heuristic false positives | `low_confidence` metadata on computation/API | DEC-0007, R-0006 |
| Hypertable migration fails | Operator TimescaleDB prerequisite in user guide | R-0004, R-0007 |
| Grafana uid mismatch | Set datasource uid + dashboard uids in same PR | DEC-0012, R-0008 |
| Sparse transaction history | Widen rolling window; flag; do not fail recompute | R-0006 |
| ECharts bundle size | Lazy-load chart components per tab | — |

## Definition of Done

- All 12 sprint tasks complete (`T-0013` … `T-0024`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0002
- Forecast recomputes after successful Firefly sync
- Hypertables populated with precomputed series
- `/forecast` renders charts for selected account and horizon
- Grafana Dashboards 1 & 5 load with account template variable
- User guide published at `docs/user-guides/US-0002.md`

## Architecture references

- `docs/engineering/architecture.md` — US-0002
- Decisions: DEC-0007 … DEC-0012
- Research: R-0006, R-0007, R-0008
- Depends on: US-0001 mirror tables, sync scheduler, Grafana datasource provisioning
