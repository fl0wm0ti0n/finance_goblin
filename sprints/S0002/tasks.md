# Tasks — Sprint S0002

**Story:** US-0002  
**Task count:** 12 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0013 | SQLx migration 002 forecast hypertables | open | AC-4 |
| T-0014 | Forecast repository and config | open | AC-4, AC-8 |
| T-0015 | Forecast engine core modules | open | AC-1, AC-2, AC-3 |
| T-0016 | ForecastService orchestration and retention | open | AC-4, AC-8 |
| T-0017 | Sync recompute hook integration | open | AC-8 |
| T-0018 | Forecast REST API endpoints | open | AC-1, AC-2, AC-3 |
| T-0019 | React forecast page shell and routing | open | AC-5 |
| T-0020 | React ECharts daily, monthly, and long-term views | open | AC-5 |
| T-0021 | Grafana datasource uid and Dashboard 1 Cashflow | open | AC-6 |
| T-0022 | Grafana Dashboard 5 Forecast horizons | open | AC-7 |
| T-0023 | Forecast unit and integration tests | open | AC-1–AC-4, AC-8 |
| T-0024 | Operator user guide and sync status UX | open | AC-5, AC-8 |

---

## T-0013 — SQLx migration 002 forecast hypertables

**Status:** open  
**Depends on:** US-0001 (migration 001)  
**Decisions:** DEC-0008

### Description

Add SQLx migration `002_forecast_hypertables.sql`: relational `forecast_computations` table; convert `forecast_balance_daily` and `forecast_cashflow_monthly` to TimescaleDB hypertables (7-day and 30-day chunks per R-0007). Include indexes on `(computation_id, account_id, ts)` and foreign-key from forecast rows to `forecast_computations.id`. Migration must fail clearly if TimescaleDB extension is missing.

### Done when

- [ ] Migration applies cleanly against external TimescaleDB-enabled PostgreSQL
- [ ] `forecast_computations`, `forecast_balance_daily`, `forecast_cashflow_monthly` exist with schema per architecture § hypertable migration
- [ ] Hypertable chunk intervals match R-0007 (7-day daily, 30-day monthly)

---

## T-0014 — Forecast repository and config

**Status:** open  
**Depends on:** T-0013  
**Decisions:** DEC-0008, DEC-0011

### Description

Implement `forecast::repository` and config additions: `[forecast]` TOML section (`rolling_window_days`, `sparse_history_days`, `retention_count`, `recurring_amount_tolerance_pct`, `[forecast.category_buckets]` defaults). Repository methods: insert computation metadata, bulk insert daily/monthly series, fetch latest successful `computation_id`, read series by account, cascade-delete rows beyond retention limit (keep last 5 successes per DEC-0011).

### Done when

- [ ] Config loads forecast settings from TOML + env overlay
- [ ] Repository writes and reads forecast rows against migration 002 schema
- [ ] Latest-computation query pattern matches architecture SQL example
- [ ] Retention cleanup deletes forecast rows for computations beyond `retention_count`

---

## T-0015 — Forecast engine core modules

**Status:** open  
**Depends on:** T-0014  
**Decisions:** DEC-0007

### Description

Implement pure projection modules under `forecast::`:
- `recurring` — lightweight payee/cadence detection with ±5% amount tolerance (R-0006)
- `rolling` — 3-month variable residual daily rate with 95th percentile cap
- `categories` — map transaction categories to income / fixed / variable buckets via config
- `project` — combine layers into day-by-day balance path (730 days); derive milestones (tomorrow, +7d, month-end; 3/6/12/24 month end balances); exclude internal transfers (R-0006)

Set `low_confidence: true` when account has fewer than 90 days of transactions (widen window to all available history).

### Done when

- [ ] Unit tests cover recurring detection, rolling average, transfer exclusion, and milestone derivation
- [ ] Monthly decomposition produces income, fixed_costs, variable_costs, free_cashflow per month
- [ ] Long-term end balances available for 3, 6, 12, and 24 month horizons
- [ ] Sparse-history accounts flagged with `low_confidence` without failing projection

---

## T-0016 — ForecastService orchestration and retention

**Status:** open  
**Depends on:** T-0015  
**Decisions:** DEC-0008, DEC-0011, DEC-0009

### Description

Implement `forecast::service::ForecastService` with `recompute(sync_run_id)`: iterate asset accounts from mirror table; run engine per account; persist daily and monthly hypertable rows under new `forecast_computations` record with `status = success | failed`. On failure, leave prior successful snapshot intact. After success, run retention cleanup (DEC-0011). Wire `ForecastService` into `AppState`.

Optional: implement aggregate series helper for same-currency accounts (DEC-0009) — used by aggregate API endpoint in T-0018.

### Done when

- [ ] `recompute` writes full 24-month daily path and monthly cashflow series per asset account
- [ ] Failed recompute records error on computation row without deleting prior snapshot
- [ ] Retention keeps at most 5 successful computations
- [ ] `ForecastService` available on shared application state

---

## T-0017 — Sync recompute hook integration

**Status:** open  
**Depends on:** T-0016  
**Decisions:** DEC-0010

### Description

Extend `SyncService::execute_run` lifecycle: after successful Firefly ingest and `finish_sync_run(success)`, call `ForecastService::recompute(sync_run_id)` **before** clearing `active_run` mutex. Forecast failure must not fail sync run; expose `stale: true` in forecast meta when latest computation failed. Sync Status `state: running` spans recompute duration; concurrent `POST /api/v1/sync/trigger` returns 409 during recompute.

### Done when

- [ ] Successful sync triggers forecast recompute inline
- [ ] Mutex covers sync and recompute; second trigger returns 409 throughout
- [ ] Sync run marked success even if forecast recompute fails
- [ ] `/api/v1/forecast/meta` returns `stale: true` when latest computation failed but prior snapshot exists

---

## T-0018 — Forecast REST API endpoints

**Status:** open  
**Depends on:** T-0016, T-0017  
**Decisions:** DEC-0006, DEC-0009

### Description

Add JWT-protected Axum handlers under `api::forecast`:

| Route | Purpose |
|-------|---------|
| `GET /api/v1/forecast/meta` | computation_id, computed_at, stale, low_confidence, sync_run_id |
| `GET /api/v1/forecast/accounts` | asset accounts for selector |
| `GET /api/v1/forecast/daily?account_id=` | series + milestones (tomorrow, next_week, month_end) |
| `GET /api/v1/forecast/monthly?account_id=` | monthly income/fixed/variable/free_cashflow series |
| `GET /api/v1/forecast/long-term?account_id=&horizon=3\|6\|12\|24` | series + end_balance |
| `GET /api/v1/forecast/aggregate` | optional summed series across asset accounts (same-currency MVP) |

All routes read latest successful `computation_id` from repository. Return 404 or empty state when no forecast exists.

### Done when

- [ ] All six endpoints registered and JWT-protected
- [ ] Daily, monthly, and long-term responses match architecture contracts
- [ ] Invalid `account_id` or missing data returns appropriate error/empty payload
- [ ] Meta endpoint reflects stale and low_confidence flags

---

## T-0019 — React forecast page shell and routing

**Status:** open  
**Depends on:** T-0018  
**Decisions:** —

### Description

Enable Forecast nav item (remove US-0001 disabled placeholder). Add `/forecast` route with shadcn layout: Card container, account Select (defaults to first asset account from `/forecast/accounts`), Tabs or ToggleGroup for Daily | Monthly | Long-term. Wire TanStack Query hooks for forecast API with bearer token. Show "Last computed" from `/forecast/meta` and link to Sync Status. Empty state when no transactions/forecast data.

Add `echarts` and `echarts-for-react` (or equivalent) to frontend dependencies.

### Done when

- [ ] Forecast nav enabled and route reachable when authenticated
- [ ] Account selector populated from API; tab switching works
- [ ] Meta timestamp and empty state render correctly
- [ ] Chart placeholder regions ready for T-0020

---

## T-0020 — React ECharts daily, monthly, and long-term views

**Status:** open  
**Depends on:** T-0019  
**Decisions:** —

### Description

Implement ECharts visualizations per `docs/product/vision.md`:
- **Daily:** stat cards (tomorrow, next week, month-end) + line chart for current-month projection
- **Monthly:** grouped bar chart (income, fixed, variable, free cashflow) + summary stat row
- **Long-term:** line/area chart with horizon pills 3 / 6 / 12 / 24 months calling `/forecast/long-term`

Lazy-load chart components per active tab to limit bundle impact.

### Done when

- [ ] Daily tab shows milestone stat cards and balance line chart for selected account
- [ ] Monthly tab shows four-bucket bar chart and summary stats
- [ ] Long-term tab switches 3/6/12/24 month horizons with correct API calls
- [ ] Charts update when account selector changes

---

## T-0021 — Grafana datasource uid and Dashboard 1 Cashflow

**Status:** open  
**Depends on:** T-0013, T-0016  
**Decisions:** DEC-0012, R-0008

### Description

Update `grafana/provisioning/datasources/postgres.yaml` with explicit `uid: FlowFinancePostgreSQL`. Add `grafana/provisioning/dashboards/analytics/cashflow.json` with uid `cashflow`, folder Analytics. Panels: balance time series + forecast overlay from `forecast_balance_daily`; static scarcity threshold reference line (default €200, visual only until US-0005). Template variable `$account_id` from asset accounts query. Filter by latest successful `computation_id`.

### Done when

- [ ] Datasource provisioning includes uid `FlowFinancePostgreSQL`
- [ ] Dashboard 1 loads in Grafana minimal profile with account variable
- [ ] Balance and forecast overlay panels query latest computation
- [ ] Scarcity threshold line visible at €200 (configurable in JSON)

---

## T-0022 — Grafana Dashboard 5 Forecast horizons

**Status:** open  
**Depends on:** T-0021  
**Decisions:** DEC-0012, R-0008

### Description

Add `grafana/provisioning/dashboards/analytics/forecast-horizons.json` with uid `forecast-horizons`, folder Analytics. Panels for 1 / 3 / 6 / 12 month horizons (required) plus optional 24-month panel aligned with React long-term selector. Reuse `$account_id` variable and latest `computation_id` query pattern. Register Analytics dashboard provider if not present from T-0021.

### Done when

- [ ] Dashboard 5 loads with uid `forecast-horizons`
- [ ] Horizon panels (1, 3, 6, 12, 24 months) display forecast end balances or series
- [ ] Queries filter by `$account_id` and latest successful computation
- [ ] Platform Health dashboard (US-0001) unchanged

---

## T-0023 — Forecast unit and integration tests

**Status:** open  
**Depends on:** T-0015, T-0017, T-0018  
**Decisions:** DEC-0007, DEC-0010

### Description

Add Rust unit tests for projection modules (recurring, rolling, transfer exclusion, milestones). Add integration test with fixture mirror data: run `recompute`, assert hypertable rows and API responses. Extend `tests/run-tests.sh` to include forecast test targets. Integration test skips without `DATABASE_URL` (same pattern as US-0001).

### Done when

- [ ] Unit tests pass for core projection logic
- [ ] Integration test validates recompute → hypertable → API read path (or SKIP without DATABASE_URL)
- [ ] `bash tests/run-tests.sh` includes forecast tests and passes
- [ ] Sync hook test or mock verifies recompute invoked after successful sync

---

## T-0024 — Operator user guide and sync status UX

**Status:** open  
**Depends on:** T-0017, T-0020, T-0022  
**Decisions:** DEC-0010

### Description

Create `docs/user-guides/US-0002.md`: TimescaleDB hypertable prerequisite, triggering sync to populate forecasts, using `/forecast` page, accessing Grafana Dashboards 1 & 5, interpreting `low_confidence` and `stale` flags. Optionally extend Sync Status UI to show forecast recompute sub-state while sync mutex is held (recommended in architecture risks).

### Done when

- [ ] User guide covers prerequisites, forecast UI, and Grafana dashboards
- [ ] Guide documents recompute-on-sync behavior and stale/low_confidence semantics
- [ ] Sync Status reflects running state during forecast recompute (or documented limitation)

---

## Execution order (recommended)

1. **Database:** T-0013 → T-0014
2. **Engine:** T-0015 → T-0016
3. **Integration:** T-0017 → T-0018
4. **Frontend (parallel after T-0018):** T-0019 → T-0020
5. **Grafana (parallel after T-0016):** T-0021 → T-0022
6. **Verification:** T-0023 → T-0024

## Acceptance coverage map

| AC | Tasks |
|----|-------|
| Daily milestones (tomorrow, next week, month-end) | T-0015, T-0018, T-0020 |
| Monthly income/fixed/variable/free cashflow | T-0015, T-0018, T-0020 |
| Long-term 3/6/12/24 months | T-0015, T-0018, T-0020 |
| TimescaleDB hypertables | T-0013, T-0014, T-0016 |
| React ECharts charts | T-0019, T-0020 |
| Grafana Dashboard 1 | T-0021 |
| Grafana Dashboard 5 | T-0022 |
| Recompute after sync | T-0016, T-0017, T-0024 |
