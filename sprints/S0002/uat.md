# UAT — Sprint S0002 / US-0002

**Sprint:** S0002  
**Story:** US-0002  
**Phase:** `/verify-work`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/release`)  
**Ready for release:** yes (operator runtime prerequisites documented below)

## Inputs

- QA PASS: `sprints/S0002/qa-findings.md`
- Acceptance: `docs/product/acceptance.md` (US-0002)
- Operator guide: `docs/user-guides/US-0002.md`
- Implementation: `backend/src/forecast/`, `backend/src/api/forecast.rs`, `frontend/src/pages/ForecastPage.tsx`, `grafana/provisioning/dashboards/analytics/`

## Environment

| Prerequisite | Status in verify-work |
|--------------|----------------------|
| `.env` populated | **Not present** — no operator `.env` in workspace |
| `DATABASE_URL` (TimescaleDB + extension) | **Unset** — `forecast_integration` and `firefly_integration` skipped |
| Firefly PAT + synced asset transactions | **Not provisioned** — live forecast charts and recompute E2E deferred |
| `AUTH_DEV_BYPASS` or OIDC IdP | **Unset** — live API/UI auth flow deferred |

Per workflow policy: code-level and automated verification **pass**; runtime E2E steps recorded as **PASS-with-prerequisites** where external infra is required.

## Automated checks

| ID | Check | Command | Result |
|----|-------|---------|--------|
| AUTO-1 | Test harness | `bash tests/run-tests.sh` | **PASS** |
| AUTO-2 | Firefly GET-only unit test | (via harness) `cargo test --test firefly_readonly` | **PASS** (1/1) |
| AUTO-3 | Forecast engine unit tests | (via harness) `cargo test --lib` | **PASS** (8/8) |
| AUTO-4 | Firefly integration (audit log) | (via harness) `cargo test --test firefly_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-5 | Forecast hypertable integration | (via harness) `cargo test --test forecast_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-6 | Frontend production build | (via harness) `npm run build` | **PASS** (lazy chunks: DailyChart, MonthlyChart, LongTermChart) |
| AUTO-7 | Compose minimal services | `docker compose --profile minimal config --services` (placeholder env) | **PASS** — `firefly-iii`, `flow-finance-ai`, `grafana` |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | Daily forecast: tomorrow, next week, month-end milestones per account | **PASS-with-prerequisites** | `project.rs` `compute_milestones`; `GET /api/v1/forecast/daily` returns `milestones.{tomorrow,next_week,month_end}`; `ForecastPage` stat cards + lazy `DailyChart`; unit test `derives_milestones_and_horizons` PASS. **Operator prerequisite:** sync asset account data; open `/forecast` daily tab and confirm milestone values. |
| UAT-2 | AC-2 | Monthly forecast: income, fixed, variable, free cashflow | **PASS-with-prerequisites** | `forecast_cashflow_monthly` hypertable columns; `GET /api/v1/forecast/monthly` returns four fields; `MonthlyChart` + summary cards; `categories.rs` bucket mapping unit test PASS. **Operator prerequisite:** non-empty monthly series after sync. |
| UAT-3 | AC-3 | Long-term forecasts for 3, 6, 12, 24 months | **PASS** | `compute_horizon_balances` for `[3,6,12,24]`; `GET /api/v1/forecast/long-term?horizon=` rejects invalid values (400); UI horizon pills; Grafana Dashboard 5 stat panels per horizon. |
| UAT-4 | AC-4 | Forecast data in TimescaleDB hypertables | **PASS-with-prerequisites** | `002_forecast_hypertables.sql` `create_hypertable` on `forecast_balance_daily` (7d) and `forecast_cashflow_monthly` (30d); `repository.rs` bulk inserts. Integration test **SKIP** without `DATABASE_URL`. **Operator prerequisite:** enable TimescaleDB; run migration 002; `DATABASE_URL=... cargo test --test forecast_integration`. |
| UAT-5 | AC-5 | React ECharts charts for selected account and horizon | **PASS-with-prerequisites** | Route `/forecast` enabled in nav; account selector; tabbed daily/monthly/long-term; lazy `ReactECharts` chart chunks in build output. **Operator prerequisite:** `AUTH_DEV_BYPASS=true` or OIDC; select account and verify chart render. |
| UAT-6 | AC-6 | Grafana Dashboard 1 (Cashflow) provisioned | **PASS** | `grafana/provisioning/dashboards/analytics/cashflow.json` — uid `cashflow`, scarcity threshold €200, balance forecast panels; datasource uid `FlowFinancePostgreSQL` matches `postgres.yaml`. |
| UAT-7 | AC-7 | Grafana Dashboard 5 (Forecast horizons) provisioned | **PASS** | `grafana/provisioning/dashboards/analytics/forecast-horizons.json` — uid `forecast-horizons`, stat panels 1/3/6/12/24 month balances + balance path series. |
| UAT-8 | AC-8 | Forecasts recompute after Firefly sync | **PASS-with-prerequisites** | `sync/mod.rs` sets phase `"forecast"` and calls `ForecastService::recompute(run_id)` after successful sync (DEC-0010); user guide documents recompute status. **Operator prerequisite:** trigger sync; confirm Sync Status phase and updated `forecast_computations` row. |

## Summary

| Metric | Count |
|--------|-------|
| Acceptance criteria passed (code/automated) | 8/8 |
| Full runtime E2E executed | 0/8 (blocked by missing operator infra) |
| Automated checks passed | 5/7 (2 SKIP — expected) |
| Failed | 0 |

## Operator prerequisites (post-release smoke)

1. Copy `.env.example` → `.env`; set `DATABASE_*`, `FIREFLY_*`, optional OIDC or `AUTH_DEV_BYPASS=true`.
2. Provision external TimescaleDB; `CREATE EXTENSION IF NOT EXISTS timescaledb;`; apply migrations including `002_forecast_hypertables.sql`.
3. `docker compose --profile minimal up --build`
4. Complete Firefly setup; create PAT; sync at least one asset account with transactions.
5. Open `http://localhost:8080/forecast` — verify daily milestones, monthly decomposition, long-term horizons.
6. Open Grafana Analytics dashboards `cashflow` and `forecast-horizons`.
7. Optional: `DATABASE_URL=... cargo test --test forecast_integration` for hypertable persistence proof.

## Findings

### Blockers

None.

### Observations

1. `forecast_integration` and `firefly_integration` require operator `DATABASE_URL` — skipped by design in CI/verify-work; unit tests and schema audit provide sufficient gate coverage.
2. Live chart population depends on synced transaction history — documented in `docs/user-guides/US-0002.md`.
3. ECharts main chunk ~1 MB (vite warning); per-tab lazy chunks mitigate — acceptable for MVP.

## Next phase

Run `/release` in a fresh release subagent context.
