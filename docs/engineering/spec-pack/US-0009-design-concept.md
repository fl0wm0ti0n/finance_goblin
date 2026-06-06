# Design Concept — US-0009

## Summary

US-0009 extends the released US-0002 **rule-based Forecast Engine** (DEC-0007) with an optional **ML-enhanced overlay**: Python StatsForecast sidecar, seasonal statistical models, 6–24 month projections with p10–p90 confidence bands, **Baseline vs ML vs Compare** on `/forecast` Long-term tab, portfolio performance outlook on `/wealth` Crypto tab, **plan-scenario risk score** (0–100) on `/planning`, and Grafana **Dashboard 5** panel extensions. Baseline computations remain the authoritative default; ML runs after baseline inside the sync mutex when history thresholds are met.

Builds on US-0002 hypertables, US-0004 plan overlays, US-0007 portfolio snapshots, US-0005 plan-viability semantics, and Dashboard 5 provisioning.

## Goals

- StatsForecast Python sidecar in Compose `full` profile; `[forecast_ml] enabled = false` default (DEC-0049, R-0044)
- Layered ML overlay via `model_kind` discriminator; baseline authority preserved (DEC-0050, R-0043)
- Seasonal model ladder: AutoETS / MSTL / SeasonalNaive fallback (DEC-0051, R-0045)
- Sync `forecast_ml` phase after baseline + plan hook; ML failure never fails sync (DEC-0052, R-0050)
- Migration 009: bands, portfolio weekly hypertable, plan_risk_scores (R-0049)
- API `variant` param + `/compare` endpoint + extended `/meta` (DEC-0053, R-0046)
- React Long-term Compare UI with ECharts bands + Monthly seasonal callout
- Deterministic plan risk score 0–100 with component breakdown (DEC-0054, R-0048)
- Portfolio outlook 3/6/12 mo on Wealth Crypto tab (R-0047)
- Grafana Dashboard 5 `$forecast_variant` + ML/risk panels (DEC-0055, R-0051)
- `get_forecast` AI tool optional `variant` param — no new tools

## Non-goals

- External cloud ML APIs; GPU training pipelines; in-app model training UI
- Replacing DEC-0007 baseline or US-0003 subscription engine
- Embedded Rust `augurs` primary path (deferred)
- Per-asset portfolio forecasts; trading signals; tax optimization
- New AI chat tools; new Grafana dashboards beyond Dashboard 5

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0049 | Python StatsForecast sidecar | Ecosystem maturity; seasonal + CV parity (R-0044) |
| DEC-0050 | `model_kind` layered overlay | Baseline authority; paired sync-cycle rows (R-0049) |
| DEC-0051 | AutoETS → MSTL ladder | Stability on short household series (R-0045) |
| DEC-0052 | Inline `forecast_ml` phase | Fresh baseline/ML pairs; plan hook unchanged (R-0050) |
| DEC-0053 | `variant` query param | Backward compatible API + Compare endpoint (R-0046) |
| DEC-0054 | Deterministic risk index | Explainable 0–100 score; not ML classifier (R-0048) |
| DEC-0055 | Dashboard 5 extensions | Stable uid; `$forecast_variant` variable (R-0051) |

**UX references:** US-0002 `/forecast` Long-term bands + Compare; US-0004 Planning risk badge; US-0007 Wealth Crypto outlook; Grafana Dashboard 5 — see `docs/product/vision.md` US-0009 discovery notes.

**Spec-pack traceability:** `docs/engineering/spec-pack/US-0009-crs.md`, `docs/engineering/spec-pack/US-0009-technical-specification.md`
