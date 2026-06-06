# CRS — US-0002

## Purpose

Enable household budgeters to anticipate account balances and plan spending through daily, monthly, and long-term cashflow forecasts with Grafana analytics — built on US-0001 Firefly read-only sync.

## Scope

### In scope

- Forecast Engine: daily (tomorrow, next week, month-end), monthly (income/fixed/variable/free cashflow), long-term (3/6/12/24 months)
- TimescaleDB hypertables (`forecast_balance_daily`, `forecast_cashflow_monthly`) + `forecast_computations` metadata
- Recompute after successful Firefly sync completion
- REST API serving precomputed series
- React `/forecast` page with ECharts (account selector, horizon tabs)
- Grafana Dashboard 1 (Cashflow) and Dashboard 5 (Forecast horizons)
- Retention of last 5 forecast computations

### Out of scope

- ML forecasting (US-0009)
- Subscription-driven forecast adjustments with confirm/reject (US-0003)
- Plan scenario overlays (US-0004)
- Alert Engine rules (US-0005) — scarcity markers visual only in Grafana
- Grafana Dashboards 2–4

## Constraints

- Forecasts derived exclusively from Firefly-synced mirror data (US-0001 dependency)
- Read-only toward Firefly III (DEC-0004)
- External PostgreSQL with TimescaleDB extension (operator prerequisite)
- JWT-protected API (DEC-0006)

## Acceptance criteria ref

`docs/product/acceptance.md` — section **US-0002** (8 criteria):

1. Daily forecast: tomorrow, next week, month-end per account
2. Monthly forecast: income, fixed, variable, free cashflow
3. Long-term: 3, 6, 12, 24 months
4. Data in TimescaleDB hypertables
5. React ECharts for selected account/horizon
6. Grafana Dashboard 1 provisioned
7. Grafana Dashboard 5 provisioned
8. Recompute after Firefly sync completes

## Traceability

| Artifact | Path |
|----------|------|
| Backlog | `docs/product/backlog.md` — US-0002 |
| Architecture | `docs/engineering/architecture.md` — US-0002 |
| Research | R-0006, R-0007, R-0008 |
| Decisions | DEC-0007 … DEC-0012 |
| User guide (execute) | `docs/user-guides/US-0002.md` |
