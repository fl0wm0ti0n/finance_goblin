# Sprint Release Notes — S0010

**Sprint:** S0010  
**Date:** 2026-06-02  
**Stories:** US-0010  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — `bash tests/run-tests.sh` @ 2026-06-02T22:36:00Z (includes `scripts/compose-config-check.sh` external-only, anti-combination, overlay validation)
2. **QA completion gate:** PASS — `sprints/S0010/qa-findings.md`, `sprints/S0010/qa.json` (6/6 AC; 0 blockers)
3. **UAT completeness gate:** PASS-with-prerequisites — verify-work not executed; omniflow host unavailable; release proceeds on QA PASS per operator contract (`handoffs/qa_to_release.md`, `sprints/S0010/smoke-evidence.md`)
4. **Isolation compliance gate:** PASS — execute/qa/release checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build`
- `runtime_mode`: omniflow external (shared host `postgres` / `firefly` / `traefik`)
- `runtime_context_ref`: `docs/engineering/runbook.md` (Omniflow external deploy — US-0010); operator guide `docs/user-guides/US-0010.md`

**Profile rule:** use **`external` only** — do not combine with `minimal`, `standard`, `full`, or `bundled-firefly`.

Greenfield dev (unchanged): `docker compose --profile minimal --profile bundled-firefly up --build`

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (Traefik + basic-auth middleware `auth`)
- `service_port`: none published on host (`ports: !reset` in external overlay)
- `health_endpoint`: in-network `http://flow-finance-ai:8080/health`; edge `https://financegnome.omniflow.cc/health` (with basic-auth)

Additional services (external profile):

| Service | Access | Notes |
|---------|--------|-------|
| Flow Finance AI | `https://financegnome.omniflow.cc` | Traefik router `financegnome`; TLS via `myresolver` |
| Grafana | internal `http://grafana:3000` on `traefik` network | optional public host via `GRAFANA_TRAEFIK_HOST` |
| Firefly III (host) | `http://firefly:8080` in-network | public ledger `https://finance.omniflow.cc` unchanged |
| PostgreSQL (host) | `postgres:5432` on `traefik` network | DB `flow_finance_ai`; TimescaleDB required |

## Verify

- `verification_steps`:
  1. Copy `.env.example` → `.env` on omniflow host; set `COMPOSE_FILE=docker-compose.yml:docker-compose.external.yml`, `COMPOSE_PROFILES=external`.
  2. Set `DATABASE_PASSWORD`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, `GRAFANA_ADMIN_PASSWORD` (replace default `admin`).
  3. Preflight TimescaleDB on `flow_finance_ai`: `SELECT extversion FROM pg_extension WHERE extname='timescaledb';` — non-null required.
  4. Deploy: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build`.
  5. Confirm services: `docker compose … ps --services` → `flow-finance-ai`, `grafana` only (no `firefly-iii`, no `postgres` from this repo).
  6. From `traefik` network: `curl -sf http://firefly:8080/api/v1/about` → HTTP 200.
  7. From `traefik` network: `curl -sf http://flow-finance-ai:8080/health` → OK JSON.
  8. Edge TLS + auth: `curl -sfI https://financegnome.omniflow.cc/health -u '<user>:<pass>'` → 200; without credentials → 401.
  9. Backend logs: no migration panic after fresh DB wiring.
  10. Run `bash tests/run-tests.sh` locally/CI (compose-config-check regression).
  11. Optional OIDC: register redirect `https://financegnome.omniflow.cc/callback` when moving off `AUTH_DEV_BYPASS`.
- `expected_health_signal`: in-network `/health` OK; edge route 200 with basic-auth; 401 without auth; no duplicate Firefly containers

## Credentials

- `credential_source_refs` (env names only):
  - `COMPOSE_FILE`, `COMPOSE_PROFILES`
  - `DATABASE_PASSWORD`, `DATABASE_HOST` (default `postgres`), `DATABASE_USER`, `DATABASE_NAME`
  - `FIREFLY_PERSONAL_ACCESS_TOKEN`, `FIREFLY_BASE_URL` (default `http://firefly:8080`)
  - `TRAEFIK_HOST` (default `financegnome.omniflow.cc`), `TRAEFIK_MIDDLEWARE` (default `auth`)
  - `GRAFANA_ADMIN_PASSWORD`, optional `GRAFANA_TRAEFIK_HOST`
  - `VITE_OIDC_*`, `OIDC_*` (when auth enabled for public URL)
  - `AUTH_DEV_BYPASS` (dev/smoke only)
- `expected_value_source`: operator `.env` at repo root on omniflow host (from `.env.example`); Traefik basic-auth credentials on host Traefik stack only (out of scope)

## Known Issues

- Omniflow host smoke steps 1–6 and 8 **PENDING** operator post-merge — documented in `sprints/S0010/smoke-evidence.md` (step 7 local PASS).
- Verify-work phase not executed; runtime TLS/Firefly DNS/migration proof deferred to operator checklist above.
- Integration tests skip without operator `DATABASE_URL` (S0001–S0009 pattern; US-0010 is compose/docs-only).
- Host port **8090** bound by `firefly_product_manager` — use `STATS_FORECAST_PORT=8091` if adding `full` profile on same host.
- TimescaleDB extension must exist on shared `postgres` before first backend start on `flow_finance_ai`.
- Do not combine `external` profile with `bundled-firefly` (starts duplicate Firefly).

## Deliverables (US-0010)

- `bundled-firefly` profile split in `docker-compose.yml` (DEC-0056)
- Env-parameterized Traefik labels in `docker-compose.external.yml` (`TRAEFIK_HOST`, `TRAEFIK_MIDDLEWARE`)
- Grafana internal-only default: `ports: !reset`, optional `GRAFANA_TRAEFIK_HOST` gate
- `.env.example` omniflow block (`COMPOSE_FILE`, external vars, OIDC notes)
- `scripts/compose-config-check.sh` wired into `tests/run-tests.sh`
- Runbook § Omniflow external deploy (eight-step AC-6 table)
- Operator guides: `docs/user-guides/US-0010.md`; `docs/user-guides/US-0001.md` start-command update
- Smoke evidence: `sprints/S0010/smoke-evidence.md`
- Decision: DEC-0056

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0010 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.10.0-us0010`

## Milestone

**US-0010 released** — first omniflow external deploy story; backlog drain segment continues (10/10 stories released through US-0010).
