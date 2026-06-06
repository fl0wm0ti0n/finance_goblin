# Technical Specification — US-0004

## Overview

US-0004 adds a **Plan Engine** to the Rust/Axum backend, migration `004_plans.sql`, plan REST endpoints, post-forecast recompute hook, React `/planning` page, and Grafana Dashboard 3 (`uid: budgets`). Plan projections are delta overlays on the latest successful forecast computation (DEC-0019).

**Dependencies:** US-0001 mirror tables + sync; US-0002 `forecast_computations` + forecast engine; US-0003 confirmed `subscription_patterns` for savings-mode suggestions.

## Components

### Plan Engine (`backend/src/plan/`)

```
PlanService::recompute_version(version_id, forecast_computation_id)
  ├─ repository::load_adjustments(version_id)
  ├─ overlay::apply(baseline_forecast, adjustments, confirmed_subs)
  ├─ project::daily_net_cashflow + optional balance path
  ├─ repository::insert_plan_computation + plan_daily_cashflow rows
  └─ return computation_id + stale flags
```

| Submodule | Responsibility |
|-----------|----------------|
| `overlay` | Map adjustments to daily deltas; subscription removal by payee_key |
| `project` | Merge with forecast baseline household series |
| `templates` | Preset bundles: current, leasing, savings_mode, house_purchase |
| `repository` | SQLx CRUD |
| `service` | CRUD, activate, versioning, compare, plan-vs-actual aggregation |
| `types` | Enums, DTOs, `PlanVsActualRow` |

### Database migration `004_plans.sql`

Per DEC-0022 / R-0018 (see `architecture.md` for full DDL):

- Enums: `plan_template`, `plan_adjustment_direction`, `plan_adjustment_frequency`, `plan_adjustment_target`
- Tables: `plans`, `plan_versions`, `plan_adjustments`, `plan_computations`, `plan_daily_cashflow` (hypertable)
- Partial unique: one active plan globally; one latest version per plan

### Forecast integration

At end of `ForecastService::recompute` success (DEC-0023):

```rust
if let Some(active) = self.plans.active_plan().await? {
    self.plans.recompute_version(active.latest_version_id, computation_id).await?;
}
```

Plan mutations spawn async recompute outside sync mutex.

### Plan REST API

See architecture.md §7 for full route table. Key endpoints:

- `POST /api/v1/plans/{id}/activate` — global active plan
- `GET /api/v1/plans/active/plan-vs-actual?month=YYYY-MM`
- `GET /api/v1/plans/{id}/compare` — v1/v2/v3 metrics
- `GET /api/v1/plans/templates/savings-mode/suggestions`

### React `/planning`

- Route: `/planning` (enable nav placeholder)
- Tabs: Scenarios | Compare | Plan vs Actual
- TanStack Query → plan API
- ECharts: grouped bar (compare), dual line (plan vs actual)
- Stale badges from `actuals_stale` / `plan_stale`

### Grafana Dashboard 3

File: `grafana/provisioning/dashboards/analytics/budgets.json`

- uid `budgets`, folder Analytics
- Variable `$active_plan_version`
- Panels: Plan, Ist, Abweichung, MTD table, active plan stat
- Ist SQL: asset accounts, exclude transfers (R-0017)

## Interfaces

| Consumer | Interface |
|----------|-----------|
| React | JWT Bearer → `/api/v1/plans/*` |
| Grafana | PostgreSQL datasource → `plan_daily_cashflow` + `transactions` |
| US-0006 (future) | `simulate_plan` tool reads plan API (no direct DB) |

## Non-functional

- **Read-only Firefly:** No Firefly writes (DEC-0004); Ist from mirror only
- **Latency:** Plan recompute outside sync mutex; monitor async task duration
- **Retention:** 3 successful `plan_computations` per version
- **Currency:** MVP single `plans.reporting_currency` (default EUR)
- **Auth:** JWT on all plan routes (DEC-0006)

## Verification

- Unit tests: overlay math, template presets, version cap 409
- Integration test: create plan → apply leasing template → plan-vs-actual rows
- Grafana: provision `budgets.json`; manual check with active plan
- User guide: `docs/user-guides/US-0004.md` (execute phase, USER_GUIDE_MODE=1)
