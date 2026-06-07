# CRS — US-0013

## Purpose

Enable **ML-enhanced forecasting and wealth analytics** on the omniflow production external profile so Forecast and Wealth deliver the full US-0009 capability — not baseline-only fallback after BUG-0010 baseline fixes.

## Scope

**In scope**

- `stats-forecast` sidecar on external profile (`docker-compose.external.yml` overlay per DEC-0076)
- `FORECAST_ML_ENABLED` / `STATS_FORECAST_URL` env contract on `flow-finance-ai`
- Verification of existing sync `forecast_ml` phase, API `variant=ml_enhanced`, React Compare, wealth overlay, Grafana ML panels
- Operator runbook: compose union, health probe, min history, degraded troubleshooting
- CI: compose-config-check extension + retain `forecast_ml_integration`

**Out of scope**

- New StatsForecast models beyond US-0009 (R-0043 ladder)
- Raw transaction ML training
- Monthly bucket attribution (BUG-0012 / US-0015)
- Grafana honest empty-state (BUG-0009 — banner remains when ML off)

## Acceptance criteria ref

See `docs/product/acceptance.md` § US-0013 — 10 rows (9 open + BUG-0010 prerequisite checked).

## Dependencies

- US-0009 DONE (ML feature stack)
- US-0010 DONE (external profile, DEC-0056)
- BUG-0010 DONE (baseline numbers prerequisite)
- DEC-0049, DEC-0052, DEC-0055, DEC-0065, DEC-0066
