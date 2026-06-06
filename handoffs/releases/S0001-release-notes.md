# Sprint Release Notes — S0001

**Sprint:** S0001  
**Date:** 2026-05-31  
**Stories:** US-0001  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — `bash tests/run-tests.sh` (release run + QA evidence)
2. **QA completion gate:** PASS — `sprints/S0001/qa-findings.md`, no blockers
3. **UAT completeness gate:** PASS — `sprints/S0001/uat.json` (`status=pass`, 8/8 AC)
4. **Isolation compliance gate:** PASS — execute/qa/verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose --profile minimal up --build`
- `runtime_mode`: local
- `runtime_context_ref`: `docs/engineering/runtime-connectivity.md` (local Docker Compose; no remote targets enabled)

## Connect

- `service_url`: `http://localhost:8080` (Flow Finance AI UI/API)
- `service_port`: `8080` (`FLOW_PORT` env override)
- `health_endpoint`: `http://localhost:8080/health` (liveness); `http://localhost:8080/health/ready` (readiness + DB ping)

Additional services (minimal profile):

| Service | URL | Port env |
|---------|-----|----------|
| Firefly III | `http://localhost:8081` | `FIREFLY_PORT` (default 8081) |
| Grafana | `http://localhost:3000` | `GRAFANA_PORT` (default 3000) |

## Verify

- `verification_steps`:
  1. Copy `.env.example` to `.env`; set `DATABASE_*`, `FIREFLY_*`, and `AUTHENTIK_SECRET_KEY` placeholder (required for any compose config parse).
  2. Provision external TimescaleDB; run `docker compose --profile minimal up --build`.
  3. `curl -sf http://localhost:8080/health` and `curl -sf http://localhost:8080/health/ready` (ready may 503 until DB reachable).
  4. Open UI at `http://localhost:8080`; sign in via OIDC or set `AUTH_DEV_BYPASS=true` for dev-only API access.
  5. Trigger manual sync from Sync Status; confirm entity counts update.
  6. Open Grafana Platform Health dashboard at `http://localhost:3000`.
  7. Run `bash tests/run-tests.sh`; optional `DATABASE_URL=... cargo test --test firefly_integration` for audit-log proof.
- `expected_health_signal`: HTTP 200 from `/health`; `/health/ready` returns 200 when external PostgreSQL is reachable; sync status API shows last run without error.

## Credentials

- `credential_source_refs` (env names only):
  - `DATABASE_PASSWORD`, `DATABASE_USER`, `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`
  - `FIREFLY_PERSONAL_ACCESS_TOKEN`, `FIREFLY_APP_KEY`, `FIREFLY_DB_PASSWORD`
  - `OIDC_ISSUER_URL`, `OIDC_AUDIENCE`, `VITE_OIDC_AUTHORITY`, `VITE_OIDC_CLIENT_ID`
  - `GRAFANA_ADMIN_PASSWORD` (default change in production)
  - `AUTH_DEV_BYPASS` (dev-only; never production)
- `expected_value_source`: operator `.env` file at repo root (from `.env.example`); Firefly PAT from Firefly UI; OIDC client from IdP admin console.

## Known Issues

- External PostgreSQL and Firefly PAT are operator prerequisites; CI/release gates verify code and compose config, not live stack without operator env.
- `firefly_integration` test skipped without `DATABASE_URL`; run locally for full read-only audit-log verification.
- Compose config requires `AUTHENTIK_SECRET_KEY` placeholder even for `--profile minimal` (oidc service env interpolation).
- OIDC live redirect/session requires IdP or `AUTH_DEV_BYPASS=true`.

## Deliverables (US-0001)

- Docker Compose profiles (`minimal`, `standard`, `full`, `oidc`) without embedded Flow PostgreSQL
- Rust/Axum backend: health, JWT API, Firefly GET-only sync, scheduler
- React OIDC UI shell: Home, Sync Status, Settings
- Grafana provisioning + Platform Health dashboard
- Operator guide: `docs/user-guides/US-0001.md`
- Test harness: `tests/run-tests.sh`, `backend/tests/firefly_readonly*.rs`

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0001 `status=released`
- `release_notes_ref`: this file
