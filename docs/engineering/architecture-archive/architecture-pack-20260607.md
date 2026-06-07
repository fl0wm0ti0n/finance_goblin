# Architecture archive pack (2026-06-07)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 8
- First archived heading: `## US-0009 — Advanced forecasting with ML & risk assessment`
- Last archived heading: `## US-0009 — Advanced forecasting with ML & risk assessment`
- Verification tuple (mandatory):
  - archived_body_lines=431
  - preamble_lines=10
  - retained_body_lines=2728

---

## US-0009 — Advanced forecasting with ML & risk assessment

**Status:** architecture complete (2026-06-01)  
**Research:** R-0043, R-0044, R-0045, R-0046, R-0047, R-0048, R-0049, R-0050, R-0051 (extends R-0006, R-0007, R-0008, R-0022, DEC-0007, DEC-0010, DEC-0011, DEC-0023)  
**Decisions:** DEC-0049, DEC-0050, DEC-0051, DEC-0052, DEC-0053, DEC-0054, DEC-0055  
**Spec-pack:** `docs/engineering/spec-pack/US-0009-{design-concept,crs,technical-specification}.md`  
**Depends on:** US-0002 baseline Forecast Engine (DEC-0007), US-0004 Plan Engine, US-0005 alert/plan-viability semantics (R-0022), US-0007 portfolio snapshots, Grafana Dashboard 5 (DEC-0012)

### System context

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Browser — /forecast Long-term: Baseline | ML | Compare + band chart         │
│            /forecast Monthly: seasonal callout                               │
│            /planning Scenarios + Compare: risk score badge (0–100)           │
│            /wealth Crypto: portfolio outlook 3/6/12 mo                       │
└───────────────────────────────┬──────────────────────────────────────────────┘
                                │ JWT Bearer
                                ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum)                                                       │
│                                                                               │
│  Sync mutex (extends DEC-0010)                                               │
│    subscriptions → forecast (baseline DEC-0007)                              │
│      └─ plan refresh hook (DEC-0023, baseline computation_id)                │
│    → forecast_ml (NEW)                                                       │
│      ├─ ForecastMlService → stats sidecar HTTP                               │
│      ├─ portfolio weekly forecast (R-0047)                                   │
│      └─ PlanRiskService refresh (R-0048)                                     │
│    → exchanges → alerts (baseline computation_id unchanged)                  │
│                                                                               │
│  GET /forecast/long-term?variant=baseline|ml_enhanced                        │
│  GET /forecast/compare?account_id=&horizon=                                  │
│  GET /forecast/meta (+ ml_status, paired ids)                                │
│  GET /plan/risk-score (active plan)                                          │
│  GET /wealth/portfolio-forecast                                              │
└───────────────────────────────┬──────────────────────────────────────────────┘
                                │ POST /v1/forecast (60s timeout)
                                ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  stats-forecast (Python FastAPI + StatsForecast) — Compose profile [full]    │
│  AutoETS / MSTL / SeasonalNaive + level=[90] intervals + cross_validation    │
└───────────────────────────────┬──────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  External PostgreSQL + TimescaleDB                                           │
│  forecast_computations.model_kind (baseline | ml_enhanced)                   │
│  forecast_balance_daily.balance_p10 / balance_p90 (ML rows only)             │
│  forecast_portfolio_weekly (hypertable)                                      │
│  plan_risk_scores                                                            │
└───────────────────────────────┬──────────────────────────────────────────────┘
                                │ SQL datasource
                                ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  Grafana Dashboard 5 (uid forecast-horizons) — $forecast_variant variable    │
│  band panel, seasonal stat, portfolio row, risk score stat                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Baseline authority (DEC-0050):** Alerts, plan hook (DEC-0023), AI `get_forecast` default, and Grafana default variant remain **`model_kind=baseline`**. ML is an optional overlay — never replaces DEC-0007 computations.

### Components

#### 1. StatsForecast sidecar (`stats-forecast/`, Compose `full` profile)

Python FastAPI service calling Nixtla StatsForecast (**DEC-0049**, **R-0044**).

| Aspect | Design |
|--------|--------|
| Image | `python:3.11-slim` + `statsforecast` + `numba` (~180–250 MB pull; ~80–120 MB RSS idle) |
| Compose | Service `stats-forecast`, `profiles: [full]`, port `8090`, env `STATS_FORECAST_URL=http://stats-forecast:8090` |
| Default | **`[forecast_ml] enabled = false`** — minimal/standard profiles baseline-only; no error when sidecar absent |
| Health | `GET /health` — backend gates ML pass on reachability |
| Timeout | Backend HTTP client **60s** hard cap; log `duration_ms` per call |

**Sidecar contract:**

```json
POST /v1/forecast
{
  "series_id": "household",
  "freq": "MS",
  "points": [{"ds": "2024-01-01", "y": 1200.0}],
  "horizon": 24,
  "level": [90],
  "model": "auto"
}

→ {
  "model_family": "AutoETS",
  "seasonal_periods": [12],
  "seasonal_strength": 0.42,
  "forecasts": [{"ds": "2025-01-01", "y": 1180.0, "y_lo": 950.0, "y_hi": 1410.0}],
  "backtest_wmape": 0.18,
  "low_confidence": false
}
```

**Alternative considered:** Embedded Rust `augurs` — rejected (incomplete seasonal ETS, no cross_validation parity; R-0044). Subprocess CLI — rejected (ops fragility).

#### 2. ML overlay service (`backend/src/forecast_ml/`)

Rust module orchestrating sidecar calls and persisting ML computation rows (**DEC-0050**, **R-0043**, **R-0045**).

| Step | Behavior |
|------|----------|
| 1. Gate | Skip when `!config.forecast_ml.enabled`, sidecar unhealthy, or monthly points < `min_monthly_points` (default 12) |
| 2. Series build | Monthly net-cashflow per account + household aggregate from mirror aggregates (freq `MS`) |
| 3. Model ladder | AutoETS(`season_length=12`) for 12–23 mo; MSTL when ≥24 mo + `seasonal_strength ≥ 0.35`; SeasonalNaive fallback with `low_confidence=true` (**DEC-0051**) |
| 4. Overlay | Integrate monthly ML net-cashflow deltas onto **baseline daily balance path** — do not replace DEC-0007 day mechanics |
| 5. Bands | Map sidecar `y_lo`/`y_hi` → `balance_p10`/`balance_p90` on `forecast_balance_daily` (**DEC-0053**, R-0046) |
| 6. Persist | Insert `forecast_computations` with `model_kind=ml_enhanced`, `paired_baseline_id`, metadata JSON |
| 7. Portfolio | When ≥8 weekly `crypto_value_eur` points — second sidecar call (freq `W`); persist `forecast_portfolio_weekly` (R-0047) |
| 8. Risk | Trigger `PlanRiskService::compute` after ML (divergence modifier may change; R-0048) |

**Metadata JSON (computation row):**

```json
{
  "ml_status": "success",
  "ml_skipped_reason": null,
  "model_family": "MSTL+AutoETS",
  "seasonal_periods": [12],
  "seasonal_detected": true,
  "seasonal_strength": 0.42,
  "backtest_wmape": 0.21,
  "low_confidence": false,
  "portfolio_forecast_skipped": false
}
```

Skip reasons: `insufficient_history`, `sidecar_disabled`, `sidecar_unavailable`, `sidecar_error`.

#### 3. Migration `009_forecast_ml.sql` (**R-0049**, **DEC-0050**)

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

CREATE TABLE IF NOT EXISTS forecast_portfolio_weekly (
  ts TIMESTAMPTZ NOT NULL,
  computation_id UUID NOT NULL REFERENCES forecast_computations(id) ON DELETE CASCADE,
  value_eur NUMERIC(18,2) NOT NULL,
  value_p10 NUMERIC(18,2),
  value_p90 NUMERIC(18,2)
);
SELECT create_hypertable('forecast_portfolio_weekly', 'ts',
  chunk_time_interval => INTERVAL '30 days', if_not_exists => TRUE);

CREATE TABLE IF NOT EXISTS plan_risk_scores (
  plan_computation_id UUID PRIMARY KEY REFERENCES plan_computations(id) ON DELETE CASCADE,
  score SMALLINT NOT NULL CHECK (score BETWEEN 0 AND 100),
  band TEXT NOT NULL,
  components JSONB NOT NULL DEFAULT '{}',
  computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Retention (DEC-0011):** keep last **5 successful per `model_kind`** — cascade deletes portfolio weekly rows.

**Latest queries (backward compatible):**

```sql
-- Baseline (existing Grafana + alerts — unchanged default)
SELECT id FROM forecast_computations
WHERE status = 'success' AND model_kind = 'baseline'
ORDER BY computed_at DESC LIMIT 1;

-- ML
SELECT id FROM forecast_computations
WHERE status = 'success' AND model_kind = 'ml_enhanced'
ORDER BY computed_at DESC LIMIT 1;
```

#### 4. Sync `forecast_ml` phase (**DEC-0052**, **R-0050**)

Extends sync mutex after baseline forecast, before exchanges:

```text
1. sync → 2. subscriptions → 3. forecast (baseline)
     └─ inline: active plan refresh (DEC-0023, baseline computation_id)
   → 4. forecast_ml (NEW sub-phase)
     └─ portfolio forecast + risk score refresh
   → 5. exchanges → 6. alerts → clear mutex
```

**Implementation sketch (`backend/src/sync/mod.rs`):**

```rust
let baseline_id = forecast.recompute(run_id, ctx).await?;
// plan hook uses baseline_id (unchanged DEC-0023)
if config.forecast_ml.enabled {
    *self.phase.lock().await = Some("forecast_ml".into());
    if let Err(e) = forecast_ml.recompute(run_id, baseline_id).await {
        warn!(?e, %run_id, "ML forecast skipped; baseline unaffected");
        forecast_ml.record_skip_on_baseline(baseline_id, &e).await;
    }
}
self.run_exchanges_and_alerts(run_id, baseline_id).await?;
```

**Failure semantics:**

| Outcome | Sync status | Baseline | ML row |
|---------|-------------|----------|--------|
| Baseline fails | may succeed with stale | failed/stale | skipped |
| ML fails | **success** (if ingest + baseline OK) | success | skip metadata on baseline; **no failed ML row** (R-0050) |
| Sidecar down | success | success | `ml_skipped_reason=sidecar_unavailable` |

Sync Status UI: phase `"forecast_ml"` displays sub-label **"ML forecast…"** when active.

**Latency budget:** baseline + ML + portfolio **<30s combined** on reference hardware; sidecar timeout 60s.

#### 5. Config (`[forecast_ml]` TOML)

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

**Env override:** `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL`.

Operator enablement (full profile):

```bash
docker compose --profile full up -d
# TOML: [forecast_ml] enabled = true
```

#### 6. REST API — `variant` param and compare (**DEC-0053**, **R-0046**)

| Method | Path | Change |
|--------|------|--------|
| GET | `/api/v1/forecast/long-term` | Add `variant=baseline\|ml_enhanced` (default `baseline`); ML adds `balance_p10`, `balance_p90` per point + explainability fields |
| GET | `/api/v1/forecast/compare` | **New** — both series + delta at horizon end + ML bands |
| GET | `/api/v1/forecast/meta` | Extend: `baseline_computation_id`, `ml_computation_id`, `ml_status`, `ml_skipped_reason`, `seasonal_detected` |
| GET | `/api/v1/forecast/monthly` | Add optional `seasonal` metadata block from latest ML computation when present |
| GET | `/api/v1/plan/risk-score` | **New** — active plan score + `components` breakdown |
| GET | `/api/v1/wealth/portfolio-forecast` | **New** — 3/6/12 mo EUR outlook from `forecast_portfolio_weekly` |

**Long-term response (ML variant excerpt):**

```json
{
  "variant": "ml_enhanced",
  "model_family": "AutoETS",
  "seasonal_periods": [12],
  "backtest_wmape": 0.21,
  "low_confidence": false,
  "series": [
    { "date": "2026-07-01", "balance": "4200.00", "balance_p10": "3800.00", "balance_p90": "4600.00" }
  ],
  "end_balance": "4500.00",
  "end_balance_p10": "4100.00",
  "end_balance_p90": "4900.00"
}
```

**Horizons:** long-term API accepts **6, 12, 24** months for ML (extends existing 3/6/12/24 — ML bands primary on 6–24 per AC2).

**`get_forecast` AI tool:** add optional `variant` query param (default `baseline`); include band summary at horizon end when `ml_enhanced` — **no new tool** (display extension only).

#### 7. React compare UI (`/forecast` Long-term tab)

| Element | Design |
|---------|--------|
| **Segmented control** | **Baseline** \| **ML-enhanced** \| **Compare** (ToggleGroup / Tabs) |
| **Baseline mode** | Existing DEC-0007 line chart (unchanged) |
| **ML mode** | Line + area band (`balance_p10`–`balance_p90`); dim opacity when `low_confidence` |
| **Compare mode** | Dual lines (baseline blue, ML orange) + ML band shading; stat row **delta at horizon end** |
| **Horizon pills** | 6 / 12 / 24 months (ML primary); 3 mo baseline-only fallback |
| **Explainability** | Collapsible **"How this forecast works"** — `model_family`, `seasonal_periods`, `backtest_wmape`, skip reason when ML unavailable |
| **Monthly tab** | Seasonal callout badge when `seasonal_detected=true` in meta |

TanStack Query keys include `variant` and `horizon`. When ML skipped, Compare tab disabled with tooltip citing `ml_skipped_reason`.

#### 8. Plan risk score (`/planning`) (**DEC-0054**, **R-0048**)

Deterministic **0–100** weighted index — not ML classifier.

**Formula:**

```
raw = 0.45*balance_stress + 0.40*plan_viability + 0.15*crypto_volatility + ml_divergence_modifier
risk_score = clamp(round(raw), 0, 100)
```

| Component | Weight | Source |
|-------------|--------|--------|
| Balance stress | 45% | Count of projected month-ends < €0 in next 6 months on active plan overlay |
| Plan viability | 40% | R-0022 rules (month-end < 0 → 80; consecutive → 100) without firing alerts |
| Crypto volatility | 15% | CV of last 12 weekly `crypto_value_eur` changes; **0 weight** when no exchanges |
| ML divergence | ±5 cap | ML 6mo p10 below scarcity threshold while baseline 6mo end above |

**UI bands:** 0–29 Low (green), 30–59 Medium (amber), 60–100 High (red).

**Surfaces:** Planning **Scenarios** tab badge on active plan; **Compare** tab risk column per version. Tooltip shows `components` JSON breakdown. **Not** a new Alert type (avoid duplicate inbox noise with plan_viability alerts).

**Compute trigger:** after active plan refresh (DEC-0023) and again after ML pass when divergence modifier may change.

#### 9. Portfolio outlook (`/wealth` Crypto tab) (**R-0047**)

When US-0007 exchange snapshots exist and portfolio forecast succeeded:

| Element | Design |
|---------|--------|
| Stat row | Projected EUR value at **3 / 6 / 12 months** |
| Source | `forecast_portfolio_weekly` linked to latest `ml_enhanced` computation |
| FX warning | Banner when >20% crypto value has incomplete FX (R-0034) — forecast still shown with `low_confidence` |
| Empty state | Hidden when `portfolio_forecast_skipped` or <8 weekly points |

#### 10. Grafana Dashboard 5 extensions (**DEC-0055**, **R-0051**)

Extend `grafana/provisioning/dashboards/analytics/forecast-horizons.json` — **`uid: forecast-horizons` unchanged** (DEC-0012).

**Template variable:**

```json
{
  "name": "forecast_variant",
  "type": "custom",
  "options": [
    {"text": "Baseline", "value": "baseline"},
    {"text": "ML Enhanced", "value": "ml_enhanced"}
  ],
  "current": {"value": "baseline"}
}
```

**Panel computation subquery (all panels):**

```sql
SELECT id FROM forecast_computations
WHERE status = 'success' AND model_kind = '$forecast_variant'
ORDER BY computed_at DESC LIMIT 1
```

**New panels (AC5):**

| Panel | Type | Notes |
|-------|------|-------|
| Confidence band path | timeseries | `balance`, `balance_p10`, `balance_p90`; hidden when variant=baseline |
| Seasonal detected | stat | `metadata->>'seasonal_detected'` from ml_enhanced computation |
| Baseline vs ML end balance | stat row | Side-by-side fixed-variant subqueries |
| Portfolio 3/6/12 mo | stat row | `forecast_portfolio_weekly` at horizon offsets; N/A when empty |
| Active plan risk score | stat | Join `plan_risk_scores` to latest active plan computation |

**Compare overlay:** timeseries with two targets — baseline (blue) + ml_enhanced (orange); band fill on ML target only.

### Backend module layout

| Module | Change |
|--------|--------|
| `forecast_ml::service` | Sidecar client, series build, overlay integration, skip metadata |
| `forecast_ml::sidecar` | HTTP client + request/response types |
| `forecast::repository` | `latest_successful_by_kind`, band column inserts, retention per kind |
| `plan::risk` | `PlanRiskService` deterministic score + upsert |
| `wealth::portfolio_forecast` | Read path for Crypto tab + API |
| `sync/mod.rs` | `forecast_ml` phase between baseline and exchanges |
| `api::forecast` | `variant` param, `/compare`, extended `/meta` |
| `api::plan` | `/risk-score` endpoint |
| `api::wealth` | `/portfolio-forecast` endpoint |
| `ai::tools::get_forecast` | Optional `variant` param |
| `migrations/009_*` | Schema per R-0049 |
| `stats-forecast/` | New Python sidecar service + Dockerfile |

### Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Sidecar optional runtime | `[forecast_ml] enabled=false` default; graceful skip metadata | R-0044, DEC-0049 |
| Mutex latency growth | 30s combined budget; 60s sidecar timeout; log sub-phase ms | R-0050, DEC-0052 |
| Sparse history instability | ≥12 mo gate; WMAPE >0.35 → `low_confidence`; MSTL only ≥24 mo | R-0045, DEC-0051 |
| Symmetric prediction intervals | Document as prediction intervals; dim bands when low_confidence | R-0046 |
| Baseline authority drift | Alerts/plan/AI default locked to baseline; code review gate | DEC-0050, DEC-0023 |
| ML/baseline divergence confusion | Compare UI + delta stat row mandatory | discovery AC6 |
| FX incomplete crypto | Warning banner; do not hard-skip forecast | R-0034, R-0047 |
| numba JIT cold start | First sync after deploy may spike; acceptable inside mutex once | R-0044 |
| Grafana empty ML panels | Dashboard description note; variant default baseline | R-0051 |
| MLOps scope creep | No training UI, cloud APIs, or new AI tools | backlog |
| Risk score over-trust | Component breakdown tooltip; not alert duplicate | R-0048, DEC-0054 |

### Decisions (US-0009)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0049 | StatsForecast sidecar | Python FastAPI in Compose `full`; `[forecast_ml] enabled=false` default |
| DEC-0050 | ML overlay model_kind | Layered overlay; baseline authoritative; `paired_baseline_id` linkage |
| DEC-0051 | Seasonal model ladder | AutoETS 12–23 mo; MSTL ≥24 mo + strength; SeasonalNaive fallback |
| DEC-0052 | Sync forecast_ml phase | After baseline + plan hook; ML failure never fails sync |
| DEC-0053 | API variant + bands | `variant` query param; compare endpoint; nullable p10/p90 columns |
| DEC-0054 | Plan risk score | Deterministic 0–100 weighted index; `plan_risk_scores` table |
| DEC-0055 | Grafana Dashboard 5 | `$forecast_variant`; band + seasonal + portfolio + risk panels |

Full records: `decisions/DEC-0049.md` … `decisions/DEC-0055.md`

### Out of scope (US-0009)

- External cloud ML APIs (OpenAI/Azure ML/SageMaker)
- GPU training pipelines; in-app model training UI; MLOps
- Embedded Rust `augurs` primary path (deferred spike)
- Replacing DEC-0007 baseline engine or US-0003 subscription heuristics
- New AI chat tools (only `get_forecast` variant extension)
- New Grafana dashboards beyond Dashboard 5 extensions
- Real-time trading signals; tax optimization
- Per-asset portfolio forecasts (household crypto EUR series only)

### Next phase

`/sprint-plan` — decompose 6 acceptance criteria (expect ≥10 tasks; single sprint unless separable deploy paths found).

---

