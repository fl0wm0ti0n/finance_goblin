# Sprint Release Notes — S0003

**Sprint:** S0003  
**Date:** 2026-05-31  
**Stories:** US-0003  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — `bash tests/run-tests.sh` (release run + QA/UAT evidence)
2. **QA completion gate:** PASS — `sprints/S0003/qa-findings.md`, no blockers
3. **UAT completeness gate:** PASS — `sprints/S0003/uat.json` (`status=pass`, 8/8 AC)
4. **Isolation compliance gate:** PASS — execute/qa/verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose --profile minimal up --build`
- `runtime_mode`: local
- `runtime_context_ref`: `docs/engineering/runbook.md` (Project run steps); external TimescaleDB required for subscription persistence

## Connect

- `service_url`: `http://localhost:8080` (Flow Finance AI UI/API)
- `service_port`: `8080` (`FLOW_PORT` env override)
- `health_endpoint`: `http://localhost:8080/health` (liveness); `http://localhost:8080/health/ready` (readiness + DB ping)

Additional services (minimal profile):

| Service | URL | Port env |
|---------|-----|----------|
| Firefly III | `http://localhost:8081` | `FIREFLY_PORT` (default 8081) |
| Grafana | `http://localhost:3000` | `GRAFANA_PORT` (default 3000) |

Subscriptions UI: `http://localhost:8080/subscriptions` (after auth or `AUTH_DEV_BYPASS=true`)

## Verify

- `verification_steps`:
  1. Copy `.env.example` to `.env`; set `DATABASE_*`, `FIREFLY_*`, and required compose placeholders.
  2. Provision external PostgreSQL **with TimescaleDB extension**; apply migrations including `backend/migrations/003_subscriptions.sql`.
  3. `docker compose --profile minimal up --build`.
  4. `curl -sf http://localhost:8080/health` and `curl -sf http://localhost:8080/health/ready`.
  5. Trigger Firefly sync from Sync Status; confirm sync phases include `"subscriptions"` before `"forecast"`.
  6. Open `http://localhost:8080/subscriptions`; verify **All | Pending review | Standing orders** tabs; confirm/reject pending cards; open detail drawer with price history chart.
  7. Open Grafana at `http://localhost:3000`; confirm dashboard **Subscriptions** (`uid=subscriptions`).
  8. Run `bash tests/run-tests.sh`; optional `DATABASE_URL=... cargo test --test subscriptions_integration` for pending persistence and `new_detection` alert proof.
- `expected_health_signal`: HTTP 200 from `/health`; `GET /api/v1/subscriptions` returns list after sync with recurring expense data; Grafana panels query `FlowFinancePostgreSQL` without provisioning errors; alerts banner shows unread `new_detection` or `price_change` when applicable.

## Credentials

- `credential_source_refs` (env names only):
  - `DATABASE_PASSWORD`, `DATABASE_USER`, `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`
  - `FIREFLY_PERSONAL_ACCESS_TOKEN`, `FIREFLY_APP_KEY`, `FIREFLY_DB_PASSWORD`
  - `OIDC_ISSUER_URL`, `OIDC_AUDIENCE`, `VITE_OIDC_AUTHORITY`, `VITE_OIDC_CLIENT_ID`
  - `GRAFANA_ADMIN_PASSWORD`
  - `AUTH_DEV_BYPASS` (dev-only; never production)
- `expected_value_source`: operator `.env` at repo root (from `.env.example`); Firefly PAT from Firefly UI; TimescaleDB on external PostgreSQL per `docs/user-guides/US-0003.md`.

## Known Issues

- External TimescaleDB (not embedded PostgreSQL) is an operator prerequisite for subscription persistence and live UI data.
- `subscriptions_integration`, `forecast_integration`, and `firefly_integration` tests skipped without `DATABASE_URL`.
- Live confirm/reject and price-change flows require synced recurring expense transactions (≥3 occurrences per pattern).
- Sync-triggered detection + forecast recompute latency should be monitored under production data volume (DEC-0018).
- OIDC live session requires IdP or `AUTH_DEV_BYPASS=true` for API/UI dev access.
- ECharts main bundle ~1 MB (PriceHistoryChart code-split; acceptable for MVP).

## Deliverables (US-0003)

- Shared recurrence core (`backend/src/recurrence/`) — confidence tiers 95/80/60 (DEC-0013, DEC-0014)
- Migration `003_subscriptions.sql` — subscription_patterns lifecycle + satellites (DEC-0015)
- Subscription engine — classify (Dauerauftrag), detection, price_change dual threshold (DEC-0016, DEC-0017)
- Sync hook — subscriptions phase before forecast; failure-tolerant (DEC-0018)
- Forecast override — confirmed replaces heuristic; rejected fingerprints excluded (AC-8)
- REST API — seven routes under `/api/v1/subscriptions/*`
- React `/subscriptions` page — tabs, confirm/reject, alerts banner, lazy ECharts price history
- Grafana — dashboard `subscriptions` in Analytics folder
- Operator guide: `docs/user-guides/US-0003.md`
- Tests — 18 unit tests; `subscriptions_integration` (optional with `DATABASE_URL`)

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0003 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.3.0-us0003`
