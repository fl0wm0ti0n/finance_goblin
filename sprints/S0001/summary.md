# Summary — Sprint S0001

## Context pack (refresh 2026-06-09)

- **Latest release:** US-0018 / S0017 (`0.18.0-us0018`, 2026-06-09) — category filters & expense trend analytics; DEC-0087..0090
- **Open backlog:** US-0019, US-0020 — backlog drain active; next discovery US-0019
- **S0001 status:** CLOSED / released (`0.1.0-us0001`) — foundation segment complete

## Context pack (refresh 2026-05-31)

- **Status:** CLOSED / released (`0.1.0-us0001`)
- **Story:** US-0001 DONE — 8/8 acceptance, queue `released`
- **Evidence:** handoffs/releases/S0001-release-notes.md, sprints/S0001/release-findings.md
- **Next story:** US-0002 (forecasting & Grafana MVP) — discovery phase

## Goal
Deliver US-0001 platform foundation: Docker Compose minimal profile, external PostgreSQL config, Rust backend with read-only Firefly sync, OIDC React UI shell, Grafana provisioning, and read-only verification.

## Completed

| Task | Deliverable |
|------|-------------|
| T-0001 | `docker-compose.yml` — profiles `minimal`, `standard`, `full`, `oidc`; no embedded PostgreSQL |
| T-0002 | `backend/src/config/` — TOML + env overlay; rejects non-external DB mode |
| T-0003 | `backend/` Axum workspace, `backend/Dockerfile` multi-stage build |
| T-0004 | `backend/src/health/` — liveness + readiness with SQLx ping; DB retry in `db/` |
| T-0005 | `backend/migrations/001_initial.sql` — TimescaleDB extension + mirror tables |
| T-0006 | `backend/src/firefly/` — GET-only client, pagination, upsert sync |
| T-0007 | `backend/src/sync/` — scheduler, manual trigger, status/runs/entities API |
| T-0008 | `backend/src/auth/` — JWT/JWKS middleware; dev bypass for local test |
| T-0009 | `frontend/` — React/Vite shell, react-oidc-context, sidebar layout |
| T-0010 | Home, Sync Status, Settings pages with TanStack Query polling |
| T-0011 | `grafana/provisioning/` — PostgreSQL datasource + Platform Health dashboard |
| T-0012 | `backend/tests/firefly_readonly*.rs`, `tests/run-tests.sh`, `docs/user-guides/US-0001.md` |

## Test results

```
bash tests/run-tests.sh — PASS
  cargo test --test firefly_readonly — PASS (GET-only guard)
  cargo test --lib — PASS (config external mode)
  frontend npm run build — PASS
```

Runtime stack startup against live PostgreSQL/Firefly deferred to QA (operator must provision external DB).

## Release closure

- Released 2026-05-31; runtime E2E deferred to operator environment (documented in UAT)
- Carry-forward: OIDC E2E needs IdP or `AUTH_DEV_BYPASS=true`; `firefly_integration` needs `DATABASE_URL`
