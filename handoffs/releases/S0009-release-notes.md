# Sprint Release Notes — S0009

**Sprint:** S0009  
**Date:** 2026-06-01  
**Stories:** US-0009  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — `bash tests/run-tests.sh` @ 2026-06-01T23:50:00Z (67 lib + 3 forecast_ml_integration + frontend build)
2. **QA completion gate:** PASS — `sprints/S0009/qa-findings.md`, no blockers
3. **UAT completeness gate:** PASS — `sprints/S0009/uat.json`, `sprints/S0009/uat.md` (6/6 AC; live ML sidecar E2E PASS-with-prerequisites)
4. **Isolation compliance gate:** PASS — execute/qa/verify-work/release checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose --profile minimal up --build` (baseline-only ML disabled); `docker compose --profile full up -d` when `[forecast_ml] enabled = true`
- `runtime_mode`: local
- `runtime_context_ref`: `docs/engineering/runbook.md` (Project run steps — ML forecasting verification); external TimescaleDB required; migration `009_forecast_ml.sql` at backend startup; StatsForecast sidecar on port 8090 in full profile

## Connect

- `service_url`: `http://localhost:8080` (Flow Finance AI UI/API)
- `service_port`: `8080` (`FLOW_PORT` env override)
- `health_endpoint`: `http://localhost:8080/health` (liveness); `http://localhost:8080/health/ready` (readiness + DB ping)

Additional services:

| Service | URL | Notes |
|---------|-----|-------|
| Firefly III | `http://localhost:8081` | `FIREFLY_PORT` |
| Grafana | `http://localhost:3000` | `GRAFANA_PORT` |
| StatsForecast sidecar (full profile) | `http://localhost:8090/health` | `stats-forecast` service; ML overlay disabled by default |
| Ollama (full profile) | `http://ollama:11434` (in-network) | US-0008 local AI; optional alongside ML |

ML surfaces: `/forecast` Long-term Baseline | ML | Compare; Monthly seasonal callout; `/planning` risk badge; `/wealth` Crypto portfolio outlook; Grafana Dashboard 5 `$forecast_variant`

## Verify

- `verification_steps`:
  1. Copy `.env.example` to `.env`; set `DATABASE_*`, `FIREFLY_*`; optional `AUTH_DEV_BYPASS=true` for API-only dev.
  2. Provision external PostgreSQL with TimescaleDB; confirm migration `009_forecast_ml.sql` applies at backend startup.
  3. **ML path:** `docker compose --profile full up -d`; set `[forecast_ml] enabled = true` (or `FORECAST_ML_ENABLED=true`); restart backend; verify `curl http://localhost:8090/health` → `{"status":"ok"}`.
  4. Run sync with ≥12 mo Firefly history (≥24 mo for MSTL seasonal callout on Monthly tab).
  5. Forecast → Long-term: Baseline | ML | Compare at 6/12/24 mo; confirm p10–p90 bands in ML mode and dual series in Compare.
  6. Forecast → Monthly: seasonal badge when `seasonal_detected=true` in `/api/v1/forecast/meta`.
  7. Planning → active plan risk badge 0–100 with component tooltip; Compare tab risk column.
  8. Wealth → Crypto: 3/6/12 mo projected EUR when US-0007 exchanges connected (≥8 weekly snapshots).
  9. Grafana Dashboard 5 (`uid=forecast-horizons`): switch `$forecast_variant` Baseline / ML Enhanced; confirm Confidence band, Seasonal detected, Portfolio 3/6/12 mo EUR, Active plan risk score panels.
  10. Confirm alerts and plan hooks still use `model_kind=baseline` only (DEC-0007 authority preserved).
  11. Run `bash tests/run-tests.sh`; optional `DATABASE_URL=... cargo test --test forecast_ml_integration` for DB skip-metadata persistence.
- `expected_health_signal`: HTTP 200 from `/health`; sidecar health OK when full profile running; ML Compare returns dual series when enabled and history sufficient; baseline served when ML skipped with `ml_skipped_reason`

## Credentials

- `credential_source_refs` (env names only):
  - `DATABASE_PASSWORD`, `DATABASE_USER`, `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`
  - `FIREFLY_PERSONAL_ACCESS_TOKEN`, `FIREFLY_APP_KEY`, `FIREFLY_DB_PASSWORD`
  - `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL` (optional ML enablement)
  - `OPENAI_API_KEY` (US-0006 cloud AI only)
  - `BINANCE_*`, `BYBIT_*`, `BITUNIX_*` (US-0007 portfolio outlook prerequisite)
  - `OIDC_ISSUER_URL`, `OIDC_AUDIENCE`, `VITE_OIDC_AUTHORITY`, `VITE_OIDC_CLIENT_ID`
  - `GRAFANA_ADMIN_PASSWORD`
  - `AUTH_DEV_BYPASS` (dev-only)
- `expected_value_source`: operator `.env` at repo root (from `.env.example`); see `docs/user-guides/US-0009.md`

## Known Issues

- ML overlay **disabled by default** (`[forecast_ml] enabled = false`); operators on minimal/standard profile see baseline-only (DEC-0049).
- External TimescaleDB and migration 009 are operator prerequisites; integration tests skip without `DATABASE_URL`.
- Live StatsForecast sidecar sync, seasonal MSTL detection, Compare dual-series UI, Grafana variant switch, and portfolio outlook E2E not executed in CI/agent environment — operator steps above required for full runtime proof.
- Sparse history → unstable seasonality; WMAPE gate + `low_confidence` UI opacity (DEC-0051).
- FX incomplete crypto portfolio forecast — warning banner, not hard skip (R-0034).
- Sync mutex latency grows with baseline + ML + exchanges phases — monitor 30s budget (DEC-0052).
- Sidecar Python `pytest` not run in QA environment when deps absent.

## Deliverables (US-0009)

- Migration `009_forecast_ml.sql` — `model_kind`, band columns, portfolio weekly, plan risk scores
- `stats-forecast/` FastAPI sidecar; Compose `full` profile; `[forecast_ml]` config
- `backend/src/forecast_ml/` overlay service + sidecar client; sync `forecast_ml` phase
- API `variant`, `/compare`, extended `/meta`; plan `/risk-score`; wealth `/portfolio-forecast`
- React: Forecast Long-term Baseline | ML | Compare + ECharts bands; Planning risk badges; Wealth Crypto outlook
- Grafana Dashboard 5 `$forecast_variant` + 5 new panels (uid `forecast-horizons` unchanged)
- Tests: `forecast_ml_integration` (3/3); 67 lib tests; harness updated
- Operator guide: `docs/user-guides/US-0009.md`
- Decisions: DEC-0049 through DEC-0055

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0009 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.9.0-us0009`

## Milestone

**Original 9-story backlog drain complete** — US-0001 through US-0009 released.
