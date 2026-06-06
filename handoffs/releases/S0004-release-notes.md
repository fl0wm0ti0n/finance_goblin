# Sprint Release Notes — S0004

**Sprint:** S0004  
**Date:** 2026-05-31  
**Stories:** US-0004  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — `bash tests/run-tests.sh` (release run + QA/UAT evidence)
2. **QA completion gate:** PASS — `sprints/S0004/qa-findings.md`, no blockers
3. **UAT completeness gate:** PASS — `sprints/S0004/uat.json` (`status=pass`, 6/6 AC)
4. **Isolation compliance gate:** PASS — execute/qa/verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose --profile minimal up --build`
- `runtime_mode`: local
- `runtime_context_ref`: `docs/engineering/runbook.md` (Project run steps); external TimescaleDB required for plan persistence

## Connect

- `service_url`: `http://localhost:8080` (Flow Finance AI UI/API)
- `service_port`: `8080` (`FLOW_PORT` env override)
- `health_endpoint`: `http://localhost:8080/health` (liveness); `http://localhost:8080/health/ready` (readiness + DB ping)

Additional services (minimal profile):

| Service | URL | Port env |
|---------|-----|----------|
| Firefly III | `http://localhost:8081` | `FIREFLY_PORT` (default 8081) |
| Grafana | `http://localhost:3000` | `GRAFANA_PORT` (default 3000) |

Planning UI: `http://localhost:8080/planning` (after auth or `AUTH_DEV_BYPASS=true`)

## Verify

- `verification_steps`:
  1. Copy `.env.example` to `.env`; set `DATABASE_*`, `FIREFLY_*`, and required compose placeholders.
  2. Provision external PostgreSQL **with TimescaleDB extension**; apply migrations including `backend/migrations/004_plans.sql`.
  3. `docker compose --profile minimal up --build`.
  4. `curl -sf http://localhost:8080/health` and `curl -sf http://localhost:8080/health/ready`.
  5. Complete Firefly sync and forecast recompute (US-0002); confirm subscriptions (US-0003) for savings-mode suggestions.
  6. Open `http://localhost:8080/planning`; create plan from template (Leasing/Savings mode); add adjustments; create v2/v3; compare versions; set active plan; verify Plan vs Actual tab.
  7. Open Grafana at `http://localhost:3000`; confirm dashboard **Budgets** (`uid=budgets`) shows Plan/Ist/Abweichung for active plan.
  8. Run `bash tests/run-tests.sh`; optional `DATABASE_URL=... cargo test --test plans_integration` for plan CRUD/recompute/plan-vs-actual and Firefly write audit proof.
- `expected_health_signal`: HTTP 200 from `/health`; `GET /api/v1/plans` returns list after auth; active plan drives `GET /api/v1/plans/active/plan-vs-actual`; Grafana panels query `FlowFinancePostgreSQL` without provisioning errors; deviation = actual − planned per day.

## Credentials

- `credential_source_refs` (env names only):
  - `DATABASE_PASSWORD`, `DATABASE_USER`, `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`
  - `FIREFLY_PERSONAL_ACCESS_TOKEN`, `FIREFLY_APP_KEY`, `FIREFLY_DB_PASSWORD`
  - `OIDC_ISSUER_URL`, `OIDC_AUDIENCE`, `VITE_OIDC_AUTHORITY`, `VITE_OIDC_CLIENT_ID`
  - `GRAFANA_ADMIN_PASSWORD`
  - `AUTH_DEV_BYPASS` (dev-only; never production)
- `expected_value_source`: operator `.env` at repo root (from `.env.example`); Firefly PAT from Firefly UI; TimescaleDB on external PostgreSQL per `docs/user-guides/US-0004.md`.

## Known Issues

- External TimescaleDB (not embedded PostgreSQL) is an operator prerequisite for plan persistence and live UI data.
- `plans_integration`, `forecast_integration`, `subscriptions_integration`, and `firefly_integration` tests skipped without `DATABASE_URL`.
- Live plan compare and Plan vs Actual Ist series require synced asset transactions and successful forecast recompute.
- Savings mode template suggestions require US-0003 confirmed subscriptions.
- Plan recompute runs asynchronously after forecast sync; **Plan stale** badge may appear until snapshot completes.
- OIDC live session requires IdP or `AUTH_DEV_BYPASS=true` for API/UI dev access.
- ECharts main bundle ~1 MB (`CompareChart`, `PlanVsActualChart` code-split; acceptable for MVP).

## Deliverables (US-0004)

- Migration `004_plans.sql` — plans schema + hypertable (DEC-0022)
- Plan Engine — delta overlay on forecast baseline, project, templates (DEC-0019, DEC-0023)
- PlanService — plan-vs-Ist, compare, versioning cap v1/v2/v3 (DEC-0020, DEC-0021)
- Post-forecast recompute hook for active plan refresh
- REST API — 17 JWT-protected endpoints under `/api/v1/plans/*`
- React `/planning` page — Scenarios, Compare, Plan vs Actual tabs with lazy ECharts
- Grafana — dashboard `budgets` in Analytics folder (DEC-0024)
- Operator guide: `docs/user-guides/US-0004.md`
- Tests — 10 plan unit tests; `plans_integration` (optional with `DATABASE_URL`)

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0004 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.4.0-us0004`
