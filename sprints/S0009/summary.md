# Sprint S0009 Summary — US-0009 ML forecasting & risk

**Status:** Released (`0.9.0-us0009`, 2026-06-01)  
**Tasks completed:** 12/12 (T-0097 … T-0108)

## Deliverables

| Task | Deliverable |
|------|-------------|
| T-0097 | Migration `009_forecast_ml.sql`; repository `model_kind`, bands, per-kind retention |
| T-0098 | `stats-forecast/` FastAPI sidecar; Compose `full` profile; `[forecast_ml]` config |
| T-0099 | `backend/src/forecast_ml/` overlay service + sidecar client |
| T-0100 | Sync `forecast_ml` phase; skip metadata; plan risk refresh hook |
| T-0101 | API `variant`, `/compare`, extended `/meta` |
| T-0102 | Forecast Long-term Baseline \| ML \| Compare UI + ECharts bands |
| T-0103 | `PlanRiskService` + `/plan/risk-score` + Planning badges |
| T-0104 | `/wealth/portfolio-forecast` + Crypto tab outlook |
| T-0105 | Grafana Dashboard 5 `$forecast_variant` + 5 new panels |
| T-0106 | Unit/integration tests; `tests/run-tests.sh` updated |
| T-0107 | Monthly seasonal callout; `get_forecast` variant param |
| T-0108 | `docs/user-guides/US-0009.md` |

## Frozen boundaries preserved

- DEC-0007 baseline algorithm unchanged; alerts/plan use `model_kind=baseline` only
- Six AI tools unchanged; `get_forecast` variant extension only
- Grafana uid `forecast-horizons` unchanged

## Test results

- `bash tests/run-tests.sh` — PASS (67 lib + 3 forecast_ml_integration + frontend build)
- DB integration portions skip without `DATABASE_URL`
- Live ML sidecar E2E deferred to operator (`--profile full`)

## Gate summary

- QA / verify-work / UAT: PASS (6/6 AC)
- Release: `0.9.0-us0009`; publish skipped (`RELEASE_PUBLISH_MODE=disabled`)

## Post-release refresh

- Context compacted 2026-06-01T24:00:00Z (`/refresh-context`); checkpoints archived to `docs/engineering/state-archive/state-pack-20260601-s0009.md`
- **AUTO_BACKLOG_DRAIN segment complete** — no next OPEN story in original backlog
