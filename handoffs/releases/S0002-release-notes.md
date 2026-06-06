# Sprint Release Notes — S0002

**Sprint:** S0002  
**Date:** 2026-05-31  
**Stories:** US-0002  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — `bash tests/run-tests.sh` (release run + QA/UAT evidence)
2. **QA completion gate:** PASS — `sprints/S0002/qa-findings.md`, no blockers
3. **UAT completeness gate:** PASS — `sprints/S0002/uat.json` (`status=pass`, 8/8 AC)
4. **Isolation compliance gate:** PASS — execute/qa/verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose --profile minimal up --build`
- `runtime_mode`: local
- `runtime_context_ref`: `docs/engineering/runbook.md` (Project run steps); external TimescaleDB required for forecast persistence

## Connect

- `service_url`: `http://localhost:8080` (Flow Finance AI UI/API)
- `service_port`: `8080` (`FLOW_PORT` env override)
- `health_endpoint`: `http://localhost:8080/health` (liveness); `http://localhost:8080/health/ready` (readiness + DB ping)

Additional services (minimal profile):

| Service | URL | Port env |
|---------|-----|----------|
| Firefly III | `http://localhost:8081` | `FIREFLY_PORT` (default 8081) |
| Grafana | `http://localhost:3000` | `GRAFANA_PORT` (default 3000) |

Forecast UI: `http://localhost:8080/forecast` (after auth or `AUTH_DEV_BYPASS=true`)

## Verify

- `verification_steps`:
  1. Copy `.env.example` to `.env`; set `DATABASE_*`, `FIREFLY_*`, and required compose placeholders.
  2. Provision external PostgreSQL **with TimescaleDB extension**; apply migrations including `backend/migrations/002_forecast_hypertables.sql`.
  3. `docker compose --profile minimal up --build`.
  4. `curl -sf http://localhost:8080/health` and `curl -sf http://localhost:8080/health/ready`.
  5. Trigger Firefly sync from Sync Status; confirm forecast recompute completes (sync phase `"forecast"`).
  6. Open `http://localhost:8080/forecast`; select synced asset account; verify Daily | Monthly | Long-term tabs and ECharts render.
  7. Open Grafana at `http://localhost:3000`; confirm dashboards **Cashflow** (`uid=cashflow`) and **Forecast Horizons** (`uid=forecast-horizons`).
  8. Run `bash tests/run-tests.sh`; optional `DATABASE_URL=... cargo test --test forecast_integration` for hypertable persistence proof.
- `expected_health_signal`: HTTP 200 from `/health`; `/api/v1/forecast/meta` returns `last_computed_at` after sync; Grafana panels query `FlowFinancePostgreSQL` datasource without provisioning errors.

## Credentials

- `credential_source_refs` (env names only):
  - `DATABASE_PASSWORD`, `DATABASE_USER`, `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`
  - `FIREFLY_PERSONAL_ACCESS_TOKEN`, `FIREFLY_APP_KEY`, `FIREFLY_DB_PASSWORD`
  - `OIDC_ISSUER_URL`, `OIDC_AUDIENCE`, `VITE_OIDC_AUTHORITY`, `VITE_OIDC_CLIENT_ID`
  - `GRAFANA_ADMIN_PASSWORD`
  - `AUTH_DEV_BYPASS` (dev-only; never production)
- `expected_value_source`: operator `.env` at repo root (from `.env.example`); Firefly PAT from Firefly UI; TimescaleDB on external PostgreSQL per `docs/user-guides/US-0002.md`.

## Known Issues

- External TimescaleDB (not embedded PostgreSQL) is an operator prerequisite for forecast hypertables and live chart data.
- `forecast_integration` and `firefly_integration` tests skipped without `DATABASE_URL`.
- Sparse transaction history may yield `low_confidence` forecast metadata (DEC-0007); recurring detection improves in US-0003.
- Sync-triggered recompute latency should be monitored under production data volume (DEC-0010).
- OIDC live session requires IdP or `AUTH_DEV_BYPASS=true` for API/UI dev access.

## Deliverables (US-0002)

- Migration `002_forecast_hypertables.sql` — forecast computations + daily/monthly hypertables
- Forecast Engine (`backend/src/forecast/`) — hybrid rule-based daily/monthly/long-term projections
- Sync hook — `ForecastService::recompute` after successful Firefly sync (DEC-0010)
- REST API — six routes under `/api/v1/forecast/*`
- React `/forecast` page — account selector, tabbed horizons, lazy-loaded ECharts
- Grafana — dashboards `cashflow`, `forecast-horizons` in Analytics folder
- Operator guide: `docs/user-guides/US-0002.md`
- Tests — 8 unit tests; `forecast_integration` (optional with `DATABASE_URL`)

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0002 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.2.0-us0002`
