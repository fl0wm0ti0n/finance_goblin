# Sprint S0009

**ID:** S0009  
**Story:** US-0009 — Advanced forecasting with ML & risk assessment  
**Status:** PLANNED  
**Created:** 2026-06-01

## Goal

Extend the released US-0002 rule-based Forecast Engine (DEC-0007) with an optional **ML-enhanced overlay** via a Python StatsForecast sidecar (Compose `full` profile, disabled by default), **Baseline vs ML-enhanced Compare** on `/forecast`, deterministic **plan-scenario risk score** on `/planning`, **portfolio outlook** on `/wealth` Crypto, and **Grafana Dashboard 5** ML extensions — while keeping baseline authoritative for alerts, plan hook (DEC-0023), and AI `get_forecast` defaults (DEC-0050).

## Scope

- Migration `009_forecast_ml.sql` — `model_kind`, band columns, `forecast_portfolio_weekly`, `plan_risk_scores` (R-0049, DEC-0050)
- StatsForecast Python FastAPI sidecar + Compose `full` profile + `[forecast_ml]` config (DEC-0049, R-0044)
- `forecast_ml` Rust overlay service — sidecar client, seasonal model ladder, baseline integration (DEC-0050, DEC-0051, R-0045)
- Sync mutex `forecast_ml` sub-phase after baseline + plan hook; ML failure never fails sync (DEC-0052, R-0050)
- REST API — `variant` param, `/compare`, extended `/meta`, `/plan/risk-score`, `/wealth/portfolio-forecast` (DEC-0053)
- React `/forecast` Long-term Compare UI + ECharts p10–p90 bands; Monthly seasonal callout (DEC-0053)
- `PlanRiskService` deterministic 0–100 score + Planning Scenarios/Compare badges (DEC-0054, R-0048)
- Wealth Crypto portfolio outlook stat row 3/6/12 mo (R-0047)
- Grafana Dashboard 5 `$forecast_variant` + band/seasonal/portfolio/risk panels (DEC-0055, R-0051)
- Unit + integration tests (sidecar mock, baseline authority, compare API)
- Operator user guide (`docs/user-guides/US-0009.md`)
- `get_forecast` AI tool optional `variant` param (display extension only — no new tool)

**Out of scope:** External cloud ML APIs; MLOps/training UI; embedded Rust `augurs` primary path; replacing DEC-0007 baseline; new AI chat tools; new Grafana dashboards beyond Dashboard 5; per-asset portfolio forecasts.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Sidecar optional runtime | `[forecast_ml] enabled=false` default; graceful skip metadata | R-0044, DEC-0049 |
| Mutex latency growth | 30s combined budget; 60s sidecar timeout; log sub-phase ms | R-0050, DEC-0052 |
| Sparse history instability | ≥12 mo gate; WMAPE >0.35 → `low_confidence`; MSTL only ≥24 mo | R-0045, DEC-0051 |
| Baseline authority drift | Alerts/plan/AI default locked to baseline; regression tests | DEC-0050, DEC-0023 |
| ML/baseline divergence confusion | Compare UI + delta stat row mandatory; disable when skipped | AC6 |
| Symmetric prediction intervals | Document as prediction intervals; dim bands when low_confidence | R-0046 |
| FX incomplete crypto | Warning banner; do not hard-skip forecast | R-0034, R-0047 |
| numba JIT cold start | First sync after deploy may spike; acceptable once inside mutex | R-0044 |
| Grafana empty ML panels | Variant default baseline; dashboard description note | R-0051 |
| MLOps scope creep | No training UI or cloud APIs | backlog |

## Definition of Done

- All 12 sprint tasks complete (`T-0097` … `T-0108`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0009
- Baseline forecast unchanged as default; ML overlay optional via `[forecast_ml] enabled=true` + `--profile full`
- Compare UI shows Baseline \| ML \| Compare with bands on 6/12/24 mo horizons
- Plan risk score 0–100 on active scenario with component breakdown
- Portfolio outlook on Wealth Crypto when exchange snapshots exist
- Grafana Dashboard 5 extended with `$forecast_variant` (uid unchanged)
- Integration tests with sidecar mock; baseline authority regression tests pass
- User guide published at `docs/user-guides/US-0009.md`
- No Firefly write operations introduced

## Architecture references

- `docs/engineering/architecture.md` — US-0009
- Decisions: DEC-0049 … DEC-0055
- Research: R-0043 … R-0051
- Depends on: US-0002 baseline (released), US-0004 Plan Engine, US-0005 alerts, US-0007 portfolio snapshots, Grafana Dashboard 5 (DEC-0012)
