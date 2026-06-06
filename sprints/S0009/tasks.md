# Tasks — Sprint S0009

**Story:** US-0009  
**Task count:** 12 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0097 | Migration 009 forecast ML schema | open | AC-2, AC-3, AC-4, AC-5 |
| T-0098 | StatsForecast sidecar and Compose full profile | open | AC-1, AC-2 |
| T-0099 | Forecast ML overlay service | open | AC-1, AC-2, AC-3 |
| T-0100 | Sync forecast_ml phase integration | open | AC-1, AC-2, AC-4 |
| T-0101 | Forecast API variant compare and meta | open | AC-2, AC-6 |
| T-0102 | React forecast Compare UI with ECharts bands | open | AC-2, AC-6 |
| T-0103 | Plan risk score service and Planning UI | open | AC-4 |
| T-0104 | Portfolio outlook API and Wealth Crypto UI | open | AC-3 |
| T-0105 | Grafana Dashboard 5 ML extensions | open | AC-5 |
| T-0106 | Forecast ML unit and integration tests | open | AC-1–AC-6 |
| T-0107 | Monthly seasonal callout and get_forecast variant | open | AC-1, AC-2 |
| T-0108 | Operator user guide US-0009 | open | AC-1–AC-6 |

---

## T-0097 — Migration 009 forecast ML schema

**Status:** open  
**Depends on:** US-0002 migration 002/003 forecast hypertables  
**Decisions:** DEC-0050, DEC-0053, DEC-0054, R-0049

### Description

Add SQLx migration `009_forecast_ml.sql` per architecture:

```sql
ALTER TABLE forecast_computations
  ADD COLUMN IF NOT EXISTS model_kind TEXT NOT NULL DEFAULT 'baseline',
  ADD COLUMN IF NOT EXISTS paired_baseline_id UUID REFERENCES forecast_computations(id);

CREATE INDEX IF NOT EXISTS idx_forecast_computations_kind_computed
  ON forecast_computations (model_kind, computed_at DESC)
  WHERE status = 'success';

ALTER TABLE forecast_balance_daily
  ADD COLUMN IF NOT EXISTS balance_p10 NUMERIC(18,2),
  ADD COLUMN IF NOT EXISTS balance_p90 NUMERIC(18,2);

CREATE TABLE IF NOT EXISTS forecast_portfolio_weekly (...);
SELECT create_hypertable('forecast_portfolio_weekly', 'ts', ...);

CREATE TABLE IF NOT EXISTS plan_risk_scores (...);
```

Update `forecast::repository`:

- `latest_successful_by_kind(model_kind)` query
- Insert paths accept `model_kind`, `paired_baseline_id`, nullable band columns
- Retention job keeps last **5 successful per `model_kind`** (DEC-0011)

Existing baseline queries default `model_kind='baseline'` — backward compatible for Grafana alerts.

### Done when

- [ ] Migration applies on external PostgreSQL + TimescaleDB
- [ ] Index on `(model_kind, computed_at DESC)` for successful rows
- [ ] Repository methods for kind-specific latest + band inserts
- [ ] Retention updated to per-kind cap of 5
- [ ] Existing US-0002 integration tests pass with default baseline kind

---

## T-0098 — StatsForecast sidecar and Compose full profile

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0049, R-0044, R-0045

### Description

Create `stats-forecast/` Python FastAPI service:

| Endpoint | Purpose |
|----------|---------|
| `GET /health` | Liveness for backend ML gate |
| `POST /v1/forecast` | StatsForecast AutoETS/MSTL/SeasonalNaive + `level=[90]` intervals |

**Request/response** per architecture US-0009 sidecar contract (`series_id`, `freq`, `points`, `horizon`, `level`, `model` → `model_family`, `forecasts[]` with `y_lo`/`y_hi`, `backtest_wmape`, `low_confidence`).

**Model ladder (DEC-0051):** AutoETS `season_length=12` for 12–23 points; MSTL when ≥24 mo + seasonal strength ≥ threshold; SeasonalNaive fallback.

**Compose (`docker-compose.yml`):**

```yaml
stats-forecast:
  profiles: [full]
  build: ./stats-forecast
  ports: ["8090:8090"]
```

**Config (`backend/config/default.toml`):**

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

Env overrides: `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL`. Load in `ForecastMlConfig`.

Sidecar unit tests (Python): health + sample forecast with synthetic monthly series.

### Done when

- [ ] Sidecar Dockerfile builds; `GET /health` returns 200
- [ ] `POST /v1/forecast` returns forecasts with p10/p90 via level=90
- [ ] Compose `full` profile includes `stats-forecast` on port 8090
- [ ] `[forecast_ml]` config loads with `enabled=false` default
- [ ] Runbook snippet: `--profile full` + `enabled=true` enablement
- [ ] Sidecar Python tests pass

---

## T-0099 — Forecast ML overlay service

**Status:** open  
**Depends on:** T-0097, T-0098  
**Decisions:** DEC-0050, DEC-0051, DEC-0053, R-0043, R-0045, R-0047

### Description

Implement `backend/src/forecast_ml/`:

| Module | Responsibility |
|--------|----------------|
| `sidecar.rs` | HTTP client (60s timeout), request/response types, health check |
| `service.rs` | Gate, series build, overlay integration, persist ML rows |

**Pipeline (`recompute(run_id, baseline_id)`):**

1. Skip when `!enabled`, sidecar unhealthy, monthly points < `min_monthly_points`
2. Build monthly net-cashflow series (freq `MS`) per account + household
3. Call sidecar; integrate monthly ML deltas onto **baseline daily balance path** (do not replace DEC-0007)
4. Map `y_lo`/`y_hi` → `balance_p10`/`balance_p90` on `forecast_balance_daily`
5. Insert `forecast_computations` with `model_kind=ml_enhanced`, `paired_baseline_id`, metadata JSON
6. Portfolio: when ≥8 weekly `crypto_value_eur` points — second sidecar call (freq `W`); persist `forecast_portfolio_weekly`
7. On skip: `record_skip_on_baseline(baseline_id, reason)` — metadata on baseline row, no failed ML row

**Metadata JSON:** `ml_status`, `ml_skipped_reason`, `model_family`, `seasonal_periods`, `seasonal_detected`, `seasonal_strength`, `backtest_wmape`, `low_confidence`, `portfolio_forecast_skipped`.

**Frozen:** do not modify DEC-0007 baseline engine algorithm in `forecast/engine.rs`.

### Done when

- [ ] Sidecar client with health gate and timeout
- [ ] ML computation persisted with `paired_baseline_id`
- [ ] Band columns populated on ML daily rows only
- [ ] Portfolio weekly hypertable populated when history sufficient
- [ ] Skip paths record metadata without failing baseline
- [ ] Unit tests: overlay integration math, skip gates, metadata shape

---

## T-0100 — Sync forecast_ml phase integration

**Status:** open  
**Depends on:** T-0099  
**Decisions:** DEC-0052, DEC-0023, R-0050

### Description

Extend `backend/src/sync/mod.rs` mutex pipeline:

```text
subscriptions → forecast (baseline) → plan refresh hook (baseline_id)
  → forecast_ml (NEW) → exchanges → alerts
```

**Implementation:**

```rust
let baseline_id = forecast.recompute(run_id, ctx).await?;
// plan hook unchanged — uses baseline_id only (DEC-0023)
if config.forecast_ml.enabled {
    *self.phase.lock().await = Some("forecast_ml".into());
    if let Err(e) = forecast_ml.recompute(run_id, baseline_id).await {
        warn!(?e, "ML forecast skipped; baseline unaffected");
        forecast_ml.record_skip_on_baseline(baseline_id, &e).await;
    }
}
self.run_exchanges_and_alerts(run_id, baseline_id).await?;
```

**Failure semantics (R-0050):**

| Outcome | Sync status | Baseline | ML |
|---------|-------------|----------|-----|
| ML fails | **success** (if ingest + baseline OK) | success | skip metadata only |
| Sidecar down | success | success | `ml_skipped_reason=sidecar_unavailable` |

Sync Status API/UI: phase `"forecast_ml"` displays **"ML forecast…"** sub-label.

Log sub-phase `duration_ms` for latency monitoring (30s combined budget).

Trigger `PlanRiskService::compute` after ML when divergence modifier may change (wire in T-0103 if service ready; else stub call site).

### Done when

- [ ] `forecast_ml` runs after baseline + plan hook, before exchanges
- [ ] ML error never marks sync failed
- [ ] Phase label exposed in sync status
- [ ] Alerts still use baseline `computation_id` only
- [ ] Sub-phase duration logged
- [ ] Integration test: ML failure → sync success + skip metadata

---

## T-0101 — Forecast API variant compare and meta

**Status:** open  
**Depends on:** T-0097, T-0099  
**Decisions:** DEC-0053, R-0046

### Description

Extend `backend/src/api/forecast.rs`:

| Method | Path | Change |
|--------|------|--------|
| GET | `/api/v1/forecast/long-term` | Query `variant=baseline\|ml_enhanced` (default `baseline`); ML adds `balance_p10`, `balance_p90`, explainability fields |
| GET | `/api/v1/forecast/compare` | **New** — both series + delta at horizon end + ML bands |
| GET | `/api/v1/forecast/meta` | Extend: `baseline_computation_id`, `ml_computation_id`, `ml_status`, `ml_skipped_reason`, `seasonal_detected` |

**Long-term horizons:** existing 3/6/12/24; ML bands primary on **6/12/24** (3 mo baseline-only acceptable).

**Compare response shape:**

```json
{
  "horizon_months": 12,
  "baseline": { "end_balance": "4200.00", "series": [...] },
  "ml_enhanced": { "end_balance": "4500.00", "end_balance_p10": "4100.00", "end_balance_p90": "4900.00", "series": [...] },
  "delta_end_balance": "300.00",
  "ml_available": true
}
```

When ML skipped: `ml_available=false` + `ml_skipped_reason`.

Register routes in `api/mod.rs`. OpenAPI/comments updated.

### Done when

- [ ] `variant` param on long-term with band fields for ML
- [ ] `/compare` returns dual series + delta
- [ ] `/meta` exposes ML status and paired computation IDs
- [ ] Default variant is `baseline` (alerts/AI compatibility)
- [ ] API unit/integration tests for variant and compare contracts
- [ ] 404/empty graceful when no ML computation exists

---

## T-0102 — React forecast Compare UI with ECharts bands

**Status:** open  
**Depends on:** T-0101  
**Decisions:** DEC-0053, DEC-0050

### Description

Extend `frontend/src/pages/ForecastPage.tsx` Long-term tab:

| Element | Implementation |
|---------|----------------|
| Segmented control | **Baseline** \| **ML-enhanced** \| **Compare** (ToggleGroup) |
| Baseline mode | Existing DEC-0007 line chart (unchanged) |
| ML mode | Line + area band (`balance_p10`–`balance_p90`); reduced opacity when `low_confidence` |
| Compare mode | Dual lines (baseline blue, ML orange) + ML band shading; stat row **delta at horizon end** |
| Horizon pills | 6 / 12 / 24 months (ML primary); 3 mo baseline-only |
| Explainability | Collapsible **"How this forecast works"** — `model_family`, `seasonal_periods`, `backtest_wmape`, skip reason |

TanStack Query keys include `variant` and `horizon`.

When ML skipped (`ml_available=false`): disable ML and Compare tabs with tooltip citing `ml_skipped_reason`.

### Done when

- [ ] Three-mode segmented control on Long-term tab
- [ ] ECharts band area renders p10–p90 for ML mode
- [ ] Compare mode shows dual series + delta stat
- [ ] Horizon pills 6/12/24 for ML; 3 mo baseline fallback
- [ ] Skipped ML disables ML/Compare with reason tooltip
- [ ] Explainability panel shows model metadata
- [ ] `npm run build` succeeds

---

## T-0103 — Plan risk score service and Planning UI

**Status:** open  
**Depends on:** T-0097, T-0100  
**Decisions:** DEC-0054, R-0048, R-0022

### Description

Implement `backend/src/plan/risk.rs` — `PlanRiskService`:

**Formula:**

```
raw = 0.45*balance_stress + 0.40*plan_viability + 0.15*crypto_volatility + ml_divergence_modifier
risk_score = clamp(round(raw), 0, 100)
```

| Component | Weight | Source |
|-----------|--------|--------|
| Balance stress | 45% | Month-ends < €0 in next 6 mo on active plan overlay |
| Plan viability | 40% | R-0022 rules without firing alerts |
| Crypto volatility | 15% | CV of last 12 weekly `crypto_value_eur`; 0 when no exchanges |
| ML divergence | ±5 cap | ML 6mo p10 below scarcity while baseline 6mo end above |

Upsert `plan_risk_scores` linked to active `plan_computation_id`.

**API:** `GET /api/v1/plan/risk-score` — score, band (`low`/`medium`/`high`), `components` JSON.

**React (`PlanningPage.tsx`):**

- Scenarios tab: risk badge 0–100 on active plan (green 0–29, amber 30–59, red 60–100)
- Compare tab: risk column per plan version
- Tooltip: `components` breakdown

**Not** a new Alert type (avoid duplicate inbox noise).

Compute after plan refresh (DEC-0023) and after ML pass (T-0100).

### Done when

- [ ] Deterministic score persisted in `plan_risk_scores`
- [ ] API returns score + components breakdown
- [ ] Planning Scenarios badge on active plan
- [ ] Compare tab risk column per version
- [ ] Component tooltip on hover
- [ ] Unit tests for formula edge cases (no crypto, ML divergence modifier)

---

## T-0104 — Portfolio outlook API and Wealth Crypto UI

**Status:** open  
**Depends on:** T-0097, T-0099  
**Decisions:** R-0047, R-0034

### Description

**Backend:** `backend/src/wealth/portfolio_forecast.rs` + `GET /api/v1/wealth/portfolio-forecast`:

```json
{
  "horizons": [
    { "months": 3, "value_eur": "12500.00", "value_p10": "11000.00", "value_p90": "14000.00" },
    { "months": 6, "value_eur": "..." },
    { "months": 12, "value_eur": "..." }
  ],
  "low_confidence": false,
  "fx_incomplete_warning": false,
  "skipped": false,
  "skip_reason": null
}
```

Source: `forecast_portfolio_weekly` linked to latest `ml_enhanced` computation.

**React (`WealthPage.tsx` Crypto tab):**

- Stat row: projected EUR at 3 / 6 / 12 months
- FX warning banner when >20% crypto value incomplete FX (R-0034)
- Hidden when `skipped` or insufficient history

### Done when

- [ ] API reads latest portfolio forecast from hypertable
- [ ] Wealth Crypto tab shows 3/6/12 mo stat row when data present
- [ ] FX incomplete warning banner when applicable
- [ ] Empty/hidden state when portfolio forecast skipped
- [ ] API tests with fixture weekly series

---

## T-0105 — Grafana Dashboard 5 ML extensions

**Status:** open  
**Depends on:** T-0097  
**Decisions:** DEC-0055, DEC-0012, R-0051

### Description

Extend `grafana/provisioning/dashboards/analytics/forecast-horizons.json`:

**Template variable `$forecast_variant`:**

```json
{ "options": [{"text": "Baseline", "value": "baseline"}, {"text": "ML Enhanced", "value": "ml_enhanced"}], "current": {"value": "baseline"} }
```

**Panel computation subquery (all panels):**

```sql
SELECT id FROM forecast_computations
WHERE status = 'success' AND model_kind = '$forecast_variant'
ORDER BY computed_at DESC LIMIT 1
```

**New panels:**

| Panel | Type | Notes |
|-------|------|-------|
| Confidence band path | timeseries | `balance`, `balance_p10`, `balance_p90`; hidden when variant=baseline |
| Seasonal detected | stat | `metadata->>'seasonal_detected'` from ml_enhanced |
| Baseline vs ML end balance | stat row | Fixed-variant subqueries side-by-side |
| Portfolio 3/6/12 mo | stat row | `forecast_portfolio_weekly` horizon offsets |
| Active plan risk score | stat | Join `plan_risk_scores` to active plan |

**uid `forecast-horizons` unchanged** (DEC-0012).

Compare overlay panel: baseline (blue) + ml_enhanced (orange) dual targets.

Dashboard description note: ML panels empty when no ml_enhanced computation.

### Done when

- [ ] `$forecast_variant` variable provisioned
- [ ] All existing panels filter by variant subquery
- [ ] Five new panels added per architecture
- [ ] uid unchanged; provisioning JSON valid
- [ ] Manual smoke: baseline default renders; ML variant when data exists

---

## T-0106 — Forecast ML unit and integration tests

**Status:** open  
**Depends on:** T-0099, T-0100, T-0101, T-0103  
**Decisions:** DEC-0050, DEC-0052, AC6

### Description

**Unit tests (Rust, no GPU):**

- `forecast_ml::service` — skip gates, overlay math, metadata JSON
- `plan::risk` — formula components, ML divergence modifier, no-crypto path
- Sidecar client — mock HTTP success/timeout/unavailable

**Integration tests (`backend/tests/forecast_ml_integration.rs`):**

- Wiremock sidecar returns synthetic forecasts
- Sync completes with ML failure → success + skip metadata
- `/forecast/compare` returns dual series when ML present
- Baseline authority: alert evaluation uses `model_kind=baseline` computation_id (grep/assert test)

**Regression:**

- Existing US-0002 forecast tests pass with default baseline
- `get_forecast` default variant baseline unchanged

Extend `tests/run-tests.sh` to include new test targets.

Skip pattern: DB-dependent portions skip without `DATABASE_URL`; sidecar mock tests run in CI without Python sidecar.

### Done when

- [ ] Unit tests for overlay, risk formula, sidecar client
- [ ] Integration test: ML failure does not fail sync
- [ ] Integration test: compare API dual series
- [ ] Baseline authority regression test passes
- [ ] `bash tests/run-tests.sh` includes new targets
- [ ] All tests pass via harness (or documented SKIP without DATABASE_URL)

---

## T-0107 — Monthly seasonal callout and get_forecast variant

**Status:** open  
**Depends on:** T-0101, T-0099  
**Decisions:** DEC-0051, DEC-0053

### Description

**Monthly tab (`ForecastPage.tsx`):**

- Fetch extended `/meta` for `seasonal_detected`, `seasonal_periods`, `seasonal_strength`
- When `seasonal_detected=true`: Badge/callout **"Seasonal pattern detected"** with period summary (e.g. "12-month cycle")
- When ML skipped: no callout (baseline monthly unchanged)

**AI tool (`backend/src/ai/tools/get_forecast.rs`):**

- Add optional `variant` query param (default `baseline`)
- When `variant=ml_enhanced`: include band summary at horizon end in tool payload
- **No new tool** — display extension only (AC boundary)

Update tool JSON schema description for variant param.

### Done when

- [ ] Monthly tab seasonal callout when meta indicates detection
- [ ] Callout hidden when no ML metadata
- [ ] `get_forecast` accepts optional `variant`
- [ ] Default variant baseline preserves US-0006 behavior
- [ ] Tool unit test for variant param passthrough
- [ ] Six-tool registry count unchanged

---

## T-0108 — Operator user guide US-0009

**Status:** open  
**Depends on:** T-0098, T-0102, T-0103, T-0104, T-0105, T-0106  
**Decisions:** —

### Description

Create `docs/user-guides/US-0009.md` per USER_GUIDE_MODE=1:

- Prerequisites: US-0002 baseline operational; US-0007 for portfolio outlook
- Enabling ML: Compose `--profile full`, `[forecast_ml] enabled=true`, sidecar health
- `/forecast` Long-term: Baseline \| ML \| Compare modes; band chart interpretation; low_confidence meaning
- Monthly seasonal callout explanation
- `/planning` risk score bands and component tooltip
- `/wealth` Crypto portfolio outlook 3/6/12 mo; FX warning
- Grafana Dashboard 5 `$forecast_variant` usage
- Troubleshooting: insufficient history, sidecar unavailable, sparse data instability
- Boundaries: baseline authoritative for alerts; no cloud ML; restart for config change
- Operator UAT checklist mirroring 6 acceptance criteria

### Done when

- [ ] User guide covers all six acceptance criteria from operator perspective
- [ ] Full profile + enablement workflow documented step-by-step
- [ ] Compare UI and risk score interpretation documented
- [ ] Grafana variant variable documented
- [ ] Troubleshooting for common skip reasons included

---

## Execution order (recommended)

1. **Schema + sidecar:** T-0097 ∥ T-0098 (parallel)
2. **ML core:** T-0099 (after T-0097, T-0098)
3. **Sync:** T-0100 (after T-0099)
4. **API:** T-0101 (after T-0099)
5. **Backend risk:** T-0103 (after T-0097, T-0100)
6. **Frontend forecast:** T-0102 → T-0107 (after T-0101)
7. **Portfolio:** T-0104 (after T-0099)
8. **Grafana:** T-0105 (after T-0097)
9. **Verification:** T-0106 → T-0108

## Acceptance coverage map

| AC | Tasks |
|----|-------|
| AC1 Seasonal patterns on monthly forecasts | T-0098, T-0099, T-0100, T-0107, T-0108 |
| AC2 ML 6–24 mo projections with confidence bands | T-0097, T-0098, T-0099, T-0100, T-0101, T-0102, T-0106, T-0107, T-0108 |
| AC3 Portfolio performance forecast (US-0007 data) | T-0097, T-0099, T-0104, T-0105, T-0108 |
| AC4 Risk assessment score for active plan scenarios | T-0097, T-0100, T-0103, T-0105, T-0106, T-0108 |
| AC5 Grafana Dashboard 5 ML and risk panels | T-0097, T-0105, T-0108 |
| AC6 Compare baseline vs ML-enhanced in UI | T-0101, T-0102, T-0106, T-0108 |
