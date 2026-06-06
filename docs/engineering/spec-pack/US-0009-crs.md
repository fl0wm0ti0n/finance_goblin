# CRS — US-0009

## Purpose

Canonical requirements snapshot for **US-0009 — Advanced forecasting with ML & risk assessment**. Enables long-term planners to view ML-enhanced seasonal forecasts with confidence bands, compare against the US-0002 baseline, assess plan-scenario risk, and monitor portfolio outlook — all self-hosted without cloud ML APIs.

See `docs/product/backlog.md#us-0009` and `docs/product/acceptance.md#US-0009`.

## Scope

**In:** StatsForecast Python sidecar (`full` profile), `[forecast_ml]` TOML config, migration 009 (`model_kind`, band columns, `forecast_portfolio_weekly`, `plan_risk_scores`), sync `forecast_ml` phase, ML overlay service, seasonal detection metadata, long-term API `variant` param + `/compare`, React Baseline | ML | Compare UI with ECharts bands, Monthly seasonal callout, plan risk score on Planning Scenarios/Compare, portfolio outlook on Wealth Crypto tab, Grafana Dashboard 5 extensions (`$forecast_variant`, band/seasonal/portfolio/risk panels), `get_forecast` variant extension, operator user guide.

**Out:** Cloud ML APIs, MLOps/training UI, augurs-in-Rust primary path, baseline replacement, new AI tools, per-asset portfolio forecasts, trading/tax features.

## Acceptance criteria ref

`docs/product/acceptance.md` — section **US-0009**:

1. Seasonal patterns detected and applied to monthly cashflow forecasts
2. ML-enhanced forecast model produces 6–24 month projections with confidence bands
3. Portfolio performance forecast available when US-0007 data present
4. Risk assessment score displayed for active plan scenarios
5. Grafana Dashboard 5 extended with ML forecast and risk panels
6. User can compare baseline (US-0002) vs ML-enhanced forecast in UI

## Architecture ref

`docs/engineering/architecture.md` — section **US-0009**  
**Decisions:** DEC-0049–DEC-0055  
**Research:** R-0043–R-0051

## AC mapping

| AC | Primary deliverables |
|----|---------------------|
| AC1 Seasonal | Sidecar model ladder (DEC-0051); metadata + Monthly callout |
| AC2 ML bands | Sidecar intervals; migration 009 bands; long-term `variant=ml_enhanced` |
| AC3 Portfolio | `forecast_portfolio_weekly`; Wealth Crypto outlook API + UI |
| AC4 Risk score | `PlanRiskService`; `plan_risk_scores`; Planning badges |
| AC5 Grafana | Dashboard 5 `$forecast_variant`; new panel row (DEC-0055) |
| AC6 Compare | `/forecast/compare`; React Baseline \| ML \| Compare control |

## Dependencies

- US-0002 baseline Forecast Engine (DEC-0007) — must remain default
- US-0004 Plan Engine — active plan overlay inputs for risk score
- US-0007 portfolio snapshots — `crypto_value_eur` weekly series
- US-0005 R-0022 plan_viability semantics — risk component, not duplicate alerts
- Grafana Dashboard 5 uid `forecast-horizons` (DEC-0012)
