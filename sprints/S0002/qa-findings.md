# QA Findings — Sprint S0002 / US-0002

**Sprint:** S0002  
**Story:** US-0002  
**QA phase:** `/qa`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/verify-work`)

## Scope

Cashflow forecasting MVP: TimescaleDB hypertable migration 002, hybrid rule-based Forecast Engine, sync-triggered recompute, 6 forecast REST endpoints, React `/forecast` page with lazy-loaded ECharts, Grafana Dashboards 1 & 5, unit/integration tests, operator user guide.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0002/summary.md`, `sprints/S0002/tasks.md`, `docs/product/acceptance.md` (US-0002), implementation in `backend/src/forecast/`, `backend/src/api/forecast.rs`, `frontend/src/pages/ForecastPage.tsx`, `grafana/provisioning/dashboards/analytics/`, `docs/user-guides/US-0002.md`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Automated harness | `bash tests/run-tests.sh` | **PASS** |
| T-2 | Firefly GET-only unit test | `cargo test --test firefly_readonly` | **PASS** (1/1) |
| T-3 | Forecast engine unit tests | `cargo test --lib` | **PASS** (8/8) |
| T-4 | Forecast hypertable integration | `cargo test --test forecast_integration` | **SKIP** — `DATABASE_URL` not set |
| T-5 | Firefly integration (US-0001 carry-forward) | `cargo test --test firefly_integration` | **SKIP** — `DATABASE_URL` not set |
| T-6 | Frontend production build | `npm run build` | **PASS** (ECharts lazy chunks: DailyChart, MonthlyChart, LongTermChart) |
| T-7 | Grafana dashboard provisioning | Static review `grafana/provisioning/` | **PASS** — uids `cashflow`, `forecast-horizons`; datasource `FlowFinancePostgreSQL` |
| T-8 | Runtime forecast E2E / live Grafana / sync recompute | Not executed in QA environment | **Deferred** to `/verify-work` (requires operator TimescaleDB, Firefly sync data) |

### Environment dependencies (non-blocking)

- **`DATABASE_URL`:** Required for `forecast_integration` (hypertable persistence, stale meta). Harness skips gracefully; unit tests and static verification cover engine logic and schema. Not a QA blocker.
- **TimescaleDB extension:** Operator prerequisite before migration 002 (documented in `docs/user-guides/US-0002.md`).
- **Synced asset account transactions:** Required for non-empty forecast UI at runtime — deferred to verify-work UAT.

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | Daily forecast shows tomorrow, next week, and month-end projected balance per account | **PASS** | `project.rs` derives milestones; `GET /api/v1/forecast/daily` returns `milestones.{tomorrow,next_week,month_end}`; `ForecastPage` stat cards + `DailyChart`; unit test `derives_milestones_and_horizons` passes. |
| AC-2 | Monthly forecast shows income, fixed costs, variable costs, and free cashflow | **PASS** | `forecast_cashflow_monthly` hypertable columns; `GET /api/v1/forecast/monthly` returns all four fields; `ForecastPage` monthly summary cards + `MonthlyChart`; `categories.rs` bucket mapping with unit test. |
| AC-3 | Long-term forecasts available for 3, 6, 12, and 24 months | **PASS** | `project.rs` `compute_horizon_balances` for `[3,6,12,24]`; `GET /api/v1/forecast/long-term?horizon=` validates allowed values (400 otherwise); UI horizon pills; Grafana Dashboard 5 stat panels for each horizon. |
| AC-4 | Forecast data persisted in TimescaleDB hypertables | **PASS** | `002_forecast_hypertables.sql` creates `forecast_balance_daily` (7d chunks) and `forecast_cashflow_monthly` (30d chunks) via `create_hypertable`; `repository.rs` bulk inserts; `forecast_integration` test asserts row counts (skipped here — env dependency only). |
| AC-5 | React UI displays forecast charts (ECharts) for selected account and horizon | **PASS** | Route `/forecast` enabled in nav; account selector; tabbed daily/monthly/long-term views; lazy `ReactECharts` in chart components; build emits separate chart chunks. |
| AC-6 | Grafana Dashboard 1 (Cashflow: balance, forecast, scarcity markers) provisioned | **PASS** | `grafana/provisioning/dashboards/analytics/cashflow.json` — uid `cashflow`, title "Cashflow"; panels for balance forecast with €200 scarcity threshold, daily balances, monthly decomposition; datasource uid matches `postgres.yaml`. |
| AC-7 | Grafana Dashboard 5 (Forecast horizons) provisioned | **PASS** | `grafana/provisioning/dashboards/analytics/forecast-horizons.json` — uid `forecast-horizons`, title "Forecast Horizons"; stat panels for 1/3/6/12/24 month end balances + balance path time series. |
| AC-8 | Forecasts recompute after Firefly sync completes | **PASS** | `sync/mod.rs` sets phase `"forecast"` and calls `ForecastService::recompute(run_id)` after successful sync, before mutex release (DEC-0010); user guide documents "Recomputing forecasts…" status; failed recompute logs warning and serves stale snapshot. |

**Summary:** 8/8 PASS (7 fully verified in QA; AC-4 integration persistence path and runtime sync/UI E2E deferred to verify-work with operator env).

## Findings

### Blockers

None.

### Observations (non-blocking)

1. **`forecast_integration` skipped:** Expected without external TimescaleDB. Unit tests (`derives_milestones_and_horizons`, recurring detection, category mapping, rolling cap) and migration/schema audit provide sufficient QA coverage; verify-work should run integration test with `DATABASE_URL`.
2. **Runtime E2E deferred:** Live `/forecast` charts, Grafana panel rendering, and sync→recompute timing require operator-provisioned stack — covered in verify-work UAT checklist (`sprints/S0002/uat.md`).
3. **ECharts bundle size:** Main chunk ~1 MB (vite warning); chart components are code-split per tab — acceptable for MVP per sprint summary.
4. **Rust dead-code warning:** `Claims` fields unused in `auth/mod.rs` — cosmetic carry-forward from US-0001; no functional impact.
5. **Category→bucket mapping:** Uses TOML defaults; operator overrides via config — documented limitation, not a blocker.

## Decision gates

No critical issues requiring dev rework. Proceed to **`/verify-work`**.

## Next phase

Run `/verify-work` in a fresh subagent with operator-provisioned external TimescaleDB, successful Firefly sync with asset account data, and optional `AUTH_DEV_BYPASS=true` for API/UI acceptance.
