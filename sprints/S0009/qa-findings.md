# QA Findings — Sprint S0009 / US-0009

**Sprint:** S0009  
**Story:** US-0009  
**QA phase:** `/qa`  
**Date:** 2026-06-01  
**Verdict:** **PASS** (ready for `/verify-work`)

## Scope

Advanced forecasting with ML & risk assessment: optional StatsForecast sidecar overlay (`[forecast_ml] enabled=false` default), migration 009 (`model_kind`, band columns, `forecast_portfolio_weekly`, `plan_risk_scores`), sync `forecast_ml` phase, Baseline | ML | Compare UI with ECharts bands, Monthly seasonal callout, plan risk score (0–100), Wealth Crypto portfolio outlook, Grafana Dashboard 5 extensions, operator guide. **Baseline (DEC-0007) remains authoritative** for alerts and plan hooks.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0009/summary.md`, `sprints/S0009/plan-verify.json`, `docs/product/acceptance.md` (US-0009), `backend/tests/forecast_ml_integration.rs`, `backend/src/forecast_ml/`, `backend/src/plan/risk.rs`, `backend/src/wealth/portfolio_forecast.rs`, `backend/src/api/forecast.rs`, `frontend/src/pages/{ForecastPage,PlanningPage,WealthPage}.tsx`, `frontend/src/components/forecast/LongTermChart.tsx`, `grafana/provisioning/dashboards/analytics/forecast-horizons.json`, `docs/user-guides/US-0009.md`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Automated harness | `bash tests/run-tests.sh` | **PASS** (exit 0, `All tests passed`) |
| T-2 | Firefly GET-only unit test | `cargo test --test firefly_readonly` | **PASS** (1/1) |
| T-3 | Backend unit tests | `cargo test --lib` | **PASS** (67/67; includes `overlay_applies_cumulative_delta`, `band_thresholds`, `skip_reason_disabled_by_default`) |
| T-4 | US-0009 integration | `cargo test --test forecast_ml_integration` | **PASS** (3/3 — sidecar mock, Grafana variant, ML skip metadata; DB portion skips without `DATABASE_URL`) |
| T-5 | Exchange signing | `cargo test --test exchange_signing` | **PASS** (4/4) |
| T-6 | AI frozen modules | `cargo test --test ai_frozen_modules` | **PASS** (2/2) |
| T-7 | AI local provider isolation | `cargo test --test ai_local_provider_isolation` | **PASS** (2/2) |
| T-8 | Integration suites | firefly/forecast/subscriptions/plans/wealth_alerts/ai_assistant/exchanges_portfolio | **SKIP** — `DATABASE_URL` not set (harness skips; S0001–S0008 pattern) |
| T-9 | Frontend build | `npm run build` (via harness) | **PASS** |
| T-10 | Sidecar Python tests | `pytest tests/` in `stats-forecast/` | **SKIP** — `pytest` not installed in QA environment |
| T-11 | Operator guide | Static review `docs/user-guides/US-0009.md` | **PASS** — enablement, horizons, bands, risk bands, Grafana variant, UAT checklist |
| T-12 | Runtime E2E (full profile ML sync, seasonal callout live, Compare UI, Grafana panels, portfolio outlook) | Not executed in QA environment | **Deferred** to `/verify-work` |

### Environment dependencies (non-blocking)

- **`DATABASE_URL`:** Required for full integration suites; harness skips without DB. ML skip-metadata test self-skips when unset.
- **`--profile full` + `[forecast_ml] enabled=true`:** Required for live ML overlay, seasonal detection, Compare dual series — deferred to verify-work.
- **Exchange history (US-0007):** Required for portfolio outlook E2E — deferred to verify-work.
- **Sidecar `pytest`:** Dev handoff reports PASS when Python deps installed; not re-run in QA env.

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | Seasonal patterns detected and applied to monthly cashflow forecasts | **PASS** (static) | Sidecar sets `seasonal_detected` / `seasonal_periods` (`stats-forecast/app/main.py`, `forecast_ml/service.rs`); `/forecast/meta` exposes fields; `ForecastPage.tsx` Monthly tab renders callout when `seasonal_detected=true`; overlay unit test applies ML monthly deltas onto baseline. Live MSTL detection deferred to verify-work. |
| AC-2 | ML-enhanced forecast model produces 6–24 month projections with confidence bands | **PASS** (static) | Migration 009 band columns; `variant=ml_enhanced` API returns `balance_p10`/`balance_p90`; horizon pills 6/12/24 for ML mode; `LongTermChart` p10–p90 area bands + low-confidence opacity; stat row shows end band. Sidecar mock integration test validates interval response. |
| AC-3 | Portfolio performance forecast available when US-0007 data present | **PASS** (static) | `PortfolioForecastService` + `GET /api/v1/wealth/portfolio-forecast`; `WealthPage.tsx` Crypto tab shows 3/6/12 mo projected EUR with optional bands and FX incomplete banner; Grafana panel "Portfolio 3/6/12 mo EUR". Live exchange-data E2E deferred to verify-work. |
| AC-4 | Risk assessment score displayed for active plan scenarios | **PASS** (static) | `PlanRiskService` persists 0–100 score with low/medium/high bands; `GET /api/v1/plans/risk-score`; `PlanningPage.tsx` badge with component tooltip (balance stress, plan viability, crypto vol, ML divergence); Compare tab risk column; unit tests `band_thresholds`, `raw_score_clamp`. |
| AC-5 | Grafana Dashboard 5 extended with ML forecast and risk panels | **PASS** | `forecast-horizons.json`: `$forecast_variant` (baseline/ml_enhanced) drives existing stat/path queries; new panels — Confidence band path (ML), Seasonal detected, Baseline vs ML 12mo end, Portfolio 3/6/12 mo EUR, Active plan risk score; uid `forecast-horizons` preserved; `grafana_dashboard_has_forecast_variant` test PASS. |
| AC-6 | User can compare baseline (US-0002) vs ML-enhanced forecast in UI | **PASS** (static) | `GET /api/v1/forecast/compare` returns dual series + `delta_end_balance`; `ForecastPage.tsx` Compare mode with Baseline/ML stat cards, dual-line `LongTermChart`, ML/Compare tabs disabled with tooltip when `ml_skipped_reason`; baseline authority preserved (`latest_successful_by_kind("baseline")` for alerts). |

**Summary:** 6/6 PASS on static/unit/harness path; live ML full-profile and operator UI E2E deferred to verify-work.

## Generated baseline test evidence

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` + `typescript` |
| `generated_test_command` | `bash tests/run-tests.sh` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-06-01 — exit 0, message `All tests passed` |
| `generated_test_paths_ref` | `backend/tests/forecast_ml_integration.rs`, `backend/src/forecast_ml/overlay.rs`, `backend/src/plan/risk.rs`, `frontend/src/pages/ForecastPage.tsx`, `grafana/provisioning/dashboards/analytics/forecast-horizons.json` |
| `generated_test_reason_code` | — |

## Runtime QA evidence

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed |
| `runtime_stack_profile` | `rust` + `node` + `stats-forecast` (optional full profile) |
| `runtime_mode` | `local` |
| `runtime_health_target` | Deferred — `--profile full`, ML enable + sync, Forecast Compare UI, Planning risk badge, Wealth Crypto outlook, Grafana Dashboard 5 variant switch |
| `runtime_health_result` | `deferred` |
| `runtime_final_verdict` | `deferred` (verify-work) |
| `runtime_reason_code` | `RUNTIME_E2E_DEFERRED_VERIFY_WORK` |
| `runtime_evidence_refs` | `docs/user-guides/US-0009.md`, `handoffs/dev_to_qa.md` |

## Findings

### Blockers

None.

### Observations (non-blocking)

1. **Integration tests skipped without `DATABASE_URL`:** Same harness pattern as S0001–S0008; US-0009-specific coverage via `forecast_ml_integration` (wiremock + static Grafana JSON).
2. **Live ML / sidecar E2E not exercised:** Requires `--profile full`, `[forecast_ml] enabled=true`, ≥12 mo history — deferred to verify-work per operator guide.
3. **Sidecar pytest not run:** `pytest` unavailable in QA environment; dev handoff reports PASS when deps installed.
4. **Rust warnings:** Pre-existing unused imports/variables — cosmetic, non-blocking.
5. **Baseline authority verified in code:** Alerts/plan hook use `model_kind=baseline`; ML failure records skip metadata without failing sync (DEC-0052).

## Decision gates

No critical issues requiring dev rework. Proceed to **`/verify-work`**.

## Next phase

Run `/verify-work` in a fresh subagent with operator PostgreSQL (`DATABASE_URL`), optional Docker Compose `--profile full` for ML smoke per `docs/user-guides/US-0009.md`.
