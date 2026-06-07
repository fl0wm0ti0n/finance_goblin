# Technical Specification — US-0013

## Overview

Implement **DEC-0076**: extend `docker-compose.external.yml` to start `stats-forecast` on the external profile via additive profile merge and traefik network attachment; passthrough ML env vars on `flow-finance-ai`; update `.env.example`, `scripts/compose-config-check.sh`, and runbook; verify existing US-0009 backend/frontend/Grafana paths on omniflow.

## Components

| Layer | Change |
|-------|--------|
| `docker-compose.external.yml` | `stats-forecast`: `profiles: [external]`, `networks: [traefik]`, `${STATS_FORECAST_PORT:-8091}:8090` |
| `docker-compose.external.yml` | `flow-finance-ai.environment`: `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL` (documented defaults) |
| `.env.example` | Omniflow block: `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL`, existing `STATS_FORECAST_PORT=8091` |
| `scripts/compose-config-check.sh` | External service set → 3 services; `stats-forecast` traefik network assert |
| `docs/engineering/runbook.md` | § Omniflow ML enablement (US-0013) |
| `docs/user-guides/US-0013.md` | Operator enablement guide |
| `backend/src/forecast_ml/` | **Verify only** — health gate, skip metadata (no algorithm change) |
| `backend/src/sync/mod.rs` | **Verify only** — `forecast_ml` sub-phase |
| `frontend/src/pages/{ForecastPage,WealthPage}.tsx` | **Verify only** — Compare + portfolio overlay |
| `grafana/provisioning/dashboards/analytics/forecast-horizons.json` | **Verify only** — `$forecast_variant=ml_enhanced` |
| `backend/tests/forecast_ml_integration.rs` | Retain; optional external-compose assert in CI |

## Interfaces

### Compose merge (external profile)

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d
```

Expected services: `flow-finance-ai`, `grafana`, `stats-forecast`.

### Env contract (operator opt-in)

| Variable | Value (omniflow) |
|----------|------------------|
| `FORECAST_ML_ENABLED` | `true` |
| `STATS_FORECAST_URL` | `http://stats-forecast:8090` |
| `STATS_FORECAST_PORT` | `8091` (host publish) |

### Runtime health gate

- Backend `ForecastMlSidecar::health_ok()` GET `/health` before ML phase
- HTTP client timeout `sidecar_timeout_secs` (default 60)
- Compose healthcheck advisory; first sync may skip until warm (DEC-0052)

### API verification

- `GET /api/v1/forecast/long-term?variant=ml_enhanced` — non-empty 6–24 mo series after Full sync
- `GET /api/v1/forecast/compare` — baseline + ML overlay
- `GET /api/v1/wealth/portfolio-forecast` — horizons when US-0007 data present

## Non-functional

- **Security:** no production secrets in CI; wiremock sidecar in integration tests
- **Compatibility:** `full` and `full+external` dev paths unchanged; DEC-0056 profile guards preserved
- **Performance:** sidecar RSS ~80–120 MB on shared host (R-0044); monitor alongside postgres/grafana
- **Testing:** compose-config-check (no docker up) + `forecast_ml_integration` + operator V1 smoke on omniflow

## Traceability

- DEC-0076, R-0071, `docs/engineering/architecture.md` § US-0013
- Sprint: **S0014** slices US-0013-S1..S4
