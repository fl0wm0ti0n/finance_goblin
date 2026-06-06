# Technical Specification — US-0009

## Overview

US-0009 adds an optional **ML-enhanced forecast overlay** on top of the released DEC-0007 baseline engine: Python StatsForecast sidecar, migration 009 schema, sync `forecast_ml` phase, REST API `variant`/`compare` extensions, React Compare UI, deterministic plan risk score, portfolio outlook, and Grafana Dashboard 5 ML panels.

**Dependencies:** US-0002 forecast hypertables, US-0004 plan computations, US-0007 `portfolio_pnl_snapshots`, US-0006 `get_forecast` tool (variant extension only).

**Frozen boundary:** DEC-0007 baseline algorithm, alert evaluation paths, plan hook baseline binding (DEC-0023), six AI tools (no new tools).

## Components

### StatsForecast sidecar (`stats-forecast/`)

```
stats-forecast/
  Dockerfile          # python:3.11-slim + statsforecast + numba
  app/main.py         # FastAPI: GET /health, POST /v1/forecast
  requirements.txt
```

**Compose (`docker-compose.yml`):**

```yaml
stats-forecast:
  build: ./stats-forecast
  profiles: [full]
  ports: ["8090:8090"]
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:8090/health"]
```

Per DEC-0049 / R-0044.

### Sidecar HTTP contract

```
POST /v1/forecast
Body: { series_id, freq, points[{ds,y}], horizon, level, model }
Response: { model_family, seasonal_periods, seasonal_strength, forecasts[{ds,y,y_lo,y_hi}],
            backtest_wmape, low_confidence }

Model ladder (internal):
  n < 12 → 422 insufficient_history
  12–23  → AutoETS(season_length=12)
  ≥24 + strength≥0.35 → MSTL
  failure → SeasonalNaive + low_confidence=true
```

Cross-validation: `h=6, n_windows=3, step_size=6` when ≥24 months available.

### Config (`[forecast_ml]`)

```toml
[forecast_ml]
enabled = false
sidecar_url = "http://stats-forecast:8090"
min_monthly_points = 12
min_portfolio_weeks = 8
interval_level = 90
sidecar_timeout_secs = 60
mstl_min_months = 24
mstl_seasonal_strength_threshold = 0.35
wmape_low_confidence_threshold = 0.35
```

Env: `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL`.

### Migration `009_forecast_ml.sql`

Per R-0049 / DEC-0050:

- `forecast_computations.model_kind` (default `'baseline'`)
- `forecast_computations.paired_baseline_id`
- `forecast_balance_daily.balance_p10`, `balance_p90` (nullable)
- `forecast_portfolio_weekly` hypertable
- `plan_risk_scores` table

Retention: 5 successful per `model_kind` (DEC-0011 extension).

### ForecastMlService (`backend/src/forecast_ml/service.rs`)

```
recompute(run_id, baseline_id) -> Result<Option<Uuid>, ForecastMlError>
  ├─ gate: enabled, sidecar health, min_monthly_points
  ├─ build monthly net-cashflow series (per account + household)
  ├─ call sidecar POST /v1/forecast (horizon=24, level=[90])
  ├─ integrate monthly deltas onto baseline daily path
  ├─ insert ml_enhanced computation + daily rows with bands
  ├─ optional: portfolio weekly forecast (crypto_value_eur, freq W)
  └─ trigger PlanRiskService::compute(active_plan_computation_id)

on skip/error: update baseline metadata { ml_status, ml_skipped_reason }
  — never fail sync (DEC-0052)
```

### Sync integration (`backend/src/sync/mod.rs`)

Insert between baseline forecast and `run_exchanges_and_alerts`:

```rust
*phase = Some("forecast_ml");
let _ = forecast_ml.recompute(run_id, baseline_id).await;
// alerts continue with baseline_id
```

Per DEC-0052 / R-0050.

### REST API

| Method | Path | Params / body |
|--------|------|---------------|
| GET | `/api/v1/forecast/long-term` | `account_id`, `horizon=3\|6\|12\|24`, **`variant=baseline\|ml_enhanced`** |
| GET | `/api/v1/forecast/compare` | `account_id`, `horizon=6\|12\|24` |
| GET | `/api/v1/forecast/meta` | Extended ML fields |
| GET | `/api/v1/forecast/monthly` | Optional `seasonal` block from ML metadata |
| GET | `/api/v1/plan/risk-score` | Active plan score + components |
| GET | `/api/v1/wealth/portfolio-forecast` | 3/6/12 mo EUR outlook |

Per DEC-0053 / R-0046.

### PlanRiskService (`backend/src/plan/risk.rs`)

```
compute(plan_computation_id) -> PlanRiskScore
  balance_stress = f(planned_balance next 6 month-ends)
  plan_viability = mirror R-0022 rules (no alert emission)
  crypto_volatility = CV(last 12 weekly crypto_value_eur) or 0
  ml_divergence = ±5 when ML p10 < scarcity_threshold && baseline end above
  score = clamp(round(0.45*bs + 0.40*pv + 0.15*cv + ml_mod), 0, 100)
  band = low | medium | high
  upsert plan_risk_scores
```

Per DEC-0054 / R-0048.

### React frontend

| Component | Change |
|-----------|--------|
| `ForecastPage` Long-term tab | Segmented control: Baseline \| ML \| Compare |
| `LongTermChart` | ECharts area band (p10–p90); dual line Compare mode |
| `ForecastMonthlyTab` | Seasonal callout from meta |
| `PlanningScenarios` | Risk score badge + tooltip components |
| `PlanningCompare` | Risk column per version |
| `WealthCryptoTab` | Portfolio outlook stat row 3/6/12 mo |
| `useForecastLongTerm` | Query key includes `variant`, `horizon` |

When `ml_status=skipped`: disable ML/Compare tabs; show explainability skip reason.

### Grafana Dashboard 5

File: `grafana/provisioning/dashboards/analytics/forecast-horizons.json`

- Variable `$forecast_variant` (custom: baseline, ml_enhanced)
- Computation subquery filters `model_kind='$forecast_variant'`
- New row: band timeseries, seasonal stat, portfolio stats, risk score stat

Per DEC-0055 / R-0051. **uid unchanged:** `forecast-horizons`.

### AI tool extension

`get_forecast` — add optional `variant` string param (default `"baseline"`). When `ml_enhanced`, include band summary at requested horizon. No registry changes beyond param schema.

### Tests

| Test | Purpose |
|------|---------|
| `forecast_ml_sidecar_mock` | Mock sidecar response; overlay integration |
| `forecast_variant_api` | baseline default backward compat; ML bands present |
| `forecast_compare_api` | dual series + delta |
| `forecast_ml_sync_skip` | sidecar down → sync success + skip metadata |
| `plan_risk_score_deterministic` | fixed inputs → fixed score |
| `portfolio_forecast_gate` | <8 weeks → skipped |
| `grafana_dashboard_variant` | JSON contains `$forecast_variant` |
| `baseline_authority_alerts` | alerts use baseline computation_id after ML run |

### Operator user guide

`docs/user-guides/US-0009.md` (USER_GUIDE_MODE=1):

- Enable ML: `--profile full`, `[forecast_ml] enabled = true`
- Minimum history requirements
- Compare UI interpretation
- Risk score component breakdown
- Grafana `$forecast_variant` usage

## Acceptance coverage

| AC | Verification |
|----|--------------|
| AC1 | Integration test seasonal metadata; Monthly callout |
| AC2 | long-term variant=ml_enhanced returns p10/p90 |
| AC3 | portfolio-forecast API + Wealth tab with exchange data |
| AC4 | plan/risk-score + Planning badge |
| AC5 | Dashboard 5 JSON panels + manual Grafana check |
| AC6 | compare API + React Compare mode E2E |
