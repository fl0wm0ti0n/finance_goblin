# Summary — Sprint S0001

## Context pack (refresh 2026-06-11 — BUG-0021 released; intake bundle drain complete)

- **Active bug:** none (intake bundle bug queue drain complete)
- **Active sprint:** none (Q0029 released `bug0021-q0029`)
- **Latest release:** BUG-0021 / Q0029 (`bug0021-q0029`, 2026-06-11) — DEC-0110 static CategoryFilter + DEC-0111 account_role COALESCE
- **Bug queue:** (empty)
- **Open stories:** (empty)
- **Orchestrator:** `auto-20260611-bug0021`
- **S0001 status:** CLOSED / released (`0.1.0-us0001`) — foundation segment complete

## Context pack (refresh 2026-06-11 — BUG-0021 execute complete)

- **Active bug:** BUG-0021 — Frontend UX polish (OPEN; qa phase next)
- **Active sprint:** Q0029 (EXECUTE COMPLETE — static CategoryFilter + account_role COALESCE + label map)
- **Execute evidence:** `handoffs/dev_to_qa.md` Q0029 section; `sprints/quick/Q0029/progress.md`
- **Runtime proof:** `runtime-proof-execute-20260611-bug0021-001`

## Context pack (refresh 2026-06-11 — BUG-0020 released)

- **Active bug:** BUG-0021 — Frontend UX polish (OPEN; next discovery)
- **Active sprint:** none (Q0028 released `bug0020-q0028`)
- **Latest release:** BUG-0020 / Q0028 (`bug0020-q0028`, 2026-06-11) — DEC-0109 subscription list reconcile + display_category backfill
- **Bug queue:** BUG-0021 (1 OPEN)
- **Orchestrator:** `auto-20260610-bug0019`
- **S0001 status:** CLOSED / released (`0.1.0-us0001`) — foundation segment complete

## Context pack (refresh 2026-06-10 — BUG-0019 released)

- **Active bug:** BUG-0020 — Subscriptions list quality (OPEN; next discovery)
- **Active sprint:** none (Q0027 released `bug0019-q0027`)
- **Latest release:** BUG-0019 / Q0027 (`bug0019-q0027`, 2026-06-10) — DEC-0108 Grafana provisioning (sort:0 + current + mirror-count panel)
- **Bug queue:** BUG-0020, BUG-0021 (2 OPEN)
- **Orchestrator:** `auto-20260610-bug0019`
- **S0001 status:** CLOSED / released (`0.1.0-us0001`) — foundation segment complete

## Context pack (refresh 2026-06-10 — BUG-0018 released)

- **Active bug:** BUG-0019 — Grafana metrics wrong (OPEN; next discovery)
- **Active sprint:** none (Q0026 released `bug0018-q0026`)
- **Latest release:** BUG-0018 / Q0026 (`bug0018-q0026`, 2026-06-10) — DEC-0107 `fbd.balance`+`fbd.ts` qualification
- **Bug queue:** BUG-0019, BUG-0020, BUG-0021 (3 OPEN)
- **Orchestrator:** `intake-20260609-ui-audit`
- **S0001 status:** CLOSED / released (`0.1.0-us0001`) — foundation segment complete

## Context pack (refresh 2026-06-10 — BUG-0017 released)

- **Latest release:** BUG-0017 / Q0025 (`bug0017-q0025`, 2026-06-10) — post-sync forecast recompute cluster; DEC-0105 audit CHECK + DEC-0106 FK CASCADE/retention
- **Active bug:** BUG-0018 — alert evaluation SQL failure (OPEN; next discovery)
- **Bug queue:** BUG-0018, BUG-0019, BUG-0020, BUG-0021 (4 OPEN)
- **Orchestrator:** `intake-20260609-ui-audit`
- **Open stories:** (empty — intake bundle drain complete)
- **S0001 status:** CLOSED / released (`0.1.0-us0001`) — foundation segment complete

## Context pack (refresh 2026-06-09 — BUG-0016)

- **Latest release:** BUG-0016 / Q0024 (`bug0016-q0024`, 2026-06-09) — SPA deep-link HTTP 404; DEC-0104 Axum SPA fallback
- **Active bug:** BUG-0017 — post-sync forecast recompute cluster (OPEN; next discovery)
- **Bug queue:** BUG-0017, BUG-0018, BUG-0019, BUG-0020, BUG-0021 (5 OPEN)
- **Orchestrator:** `intake-20260609-ui-audit`
- **Open stories:** (empty — intake bundle drain complete)
- **S0001 status:** CLOSED / released (`0.1.0-us0001`) — foundation segment complete

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
