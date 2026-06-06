# Tasks — Sprint S0001

**Story:** US-0001  
**Task count:** 12 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0001 | Docker Compose profiles and minimal stack | done | AC-1, AC-8 |
| T-0002 | External PostgreSQL configuration layer | done | AC-2, AC-8 |
| T-0003 | Rust/Axum backend project skeleton | done | AC-3 |
| T-0004 | Health endpoints and DB startup retry | done | AC-3 |
| T-0005 | SQLx migrations and mirror schema | done | AC-5, AC-6 |
| T-0006 | Firefly GET-only connector | done | AC-5, AC-7 |
| T-0007 | Sync scheduler and sync API endpoints | done | AC-6 |
| T-0008 | JWT auth middleware and protected API skeleton | done | AC-3 |
| T-0009 | React UI shell and OIDC integration | done | AC-4 |
| T-0010 | Home and Sync Status UI pages | done | AC-5, AC-6 |
| T-0011 | Grafana datasource and Platform Health dashboard | done | scope |
| T-0012 | Read-only integration test and operator user guide | done | AC-7, AC-8 |

---

## T-0001 — Docker Compose profiles and minimal stack

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0003, DEC-0001

### Description

Create `docker-compose.yml` with profiles `minimal`, `standard`, `full`, and `oidc`. Minimal profile runs `flow-finance-ai`, `firefly-iii`, and `grafana` with **no embedded PostgreSQL**. Wire external DB env vars for Flow and Firefly separately. Add `extra_hosts: host.docker.internal:host-gateway` for Linux dev. Standard adds `redis`; full adds `ollama`; `oidc` adds Authentik stack (optional, not required for minimal acceptance).

### Done when

- [ ] `docker compose --profile minimal up` starts three services without postgres container
- [ ] Profile definitions match architecture § Docker Compose profiles
- [ ] `.env.example` documents required external DB and Firefly vars

---

## T-0002 — External PostgreSQL configuration layer

**Status:** open  
**Depends on:** T-0001  
**Decisions:** DEC-0003

### Description

Implement backend `config` module: TOML defaults with `database.mode = "external"` (required), env overlay for `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`, `DATABASE_USER`, `DATABASE_PASSWORD`. Reject startup if mode is not external. Document operator wiring for host vs container reachability.

### Done when

- [ ] Config loads from TOML + env; `database.mode = "external"` enforced
- [ ] Missing required DB env vars produce clear startup error
- [ ] Operator can point at host or remote PostgreSQL without code changes

---

## T-0003 — Rust/Axum backend project skeleton

**Status:** open  
**Depends on:** T-0001  
**Decisions:** —

### Description

Scaffold Rust workspace: Axum + Tokio, module layout (`config`, `db`, `auth`, `firefly`, `sync`, `api`, `health`). Add multi-stage Dockerfile for `flow-finance-ai` service. Wire basic app bootstrap and graceful shutdown.

### Done when

- [ ] `cargo build` succeeds; container image builds from Dockerfile
- [ ] Module structure matches architecture § Rust/Axum backend
- [ ] Service starts and binds configured port inside Compose

---

## T-0004 — Health endpoints and DB startup retry

**Status:** open  
**Depends on:** T-0002, T-0003  
**Decisions:** DEC-0003

### Description

Implement `GET /health` (public liveness) and `GET /health/ready` (SQLx pool connectivity). On startup, retry external DB connection with exponential backoff, max ~60s, then fail with actionable log message.

### Done when

- [ ] `/health` returns 200 without auth
- [ ] `/health/ready` returns 503 when DB unreachable, 200 when connected
- [ ] Startup retry behavior matches DEC-0003 (~60s max)

---

## T-0005 — SQLx migrations and mirror schema

**Status:** open  
**Depends on:** T-0004  
**Decisions:** DEC-0005

### Description

Add SQLx migrations at startup (`sqlx::migrate!`). First migration: `CREATE EXTENSION IF NOT EXISTS timescaledb`. Create mirror tables: `sync_runs`, `sync_cursors`, `accounts`, `transactions`, `categories`, `budgets`, `tags`, `piggy_banks`, `firefly_request_audit`. Indexed columns + JSONB raw snapshots per architecture.

### Done when

- [ ] Migrations apply cleanly against external TimescaleDB-enabled PostgreSQL
- [ ] All mirror tables exist with Firefly `id` PKs and `synced_at` columns
- [ ] Repository traits or query helpers stubbed for sync module

---

## T-0006 — Firefly GET-only connector

**Status:** open  
**Depends on:** T-0005  
**Decisions:** DEC-0004, DEC-0002, DEC-0001

### Description

Implement `firefly` module: typed GET-only HTTP client (compile-time + runtime guard rejecting non-GET). PAT via `Authorization: Bearer`. Paginate with `page` + `limit=500`. Incremental transactions via watermark + 7-day overlap; reference entities full-list upsert by Firefly `id`. Persist optional `firefly_request_audit` rows. Exponential backoff on 5xx/429.

### Done when

- [ ] Connector fetches and upserts all six entity types from `/api/v1/*`
- [ ] Non-GET attempts rejected at wrapper layer
- [ ] Audit log records method, path, status, timestamp when enabled

---

## T-0007 — Sync scheduler and sync API endpoints

**Status:** open  
**Depends on:** T-0006  
**Decisions:** DEC-0002

### Description

Implement sync state machine (`idle → running → success | failed`) with `tokio-cron-scheduler` (or equivalent). Configurable `[sync] interval_seconds` (default 3600). Expose `POST /api/v1/sync/trigger` (202 + run id), `GET /api/v1/sync/status`, `GET /api/v1/sync/runs`, `GET /api/v1/sync/entities`. Mutex prevents overlapping runs (409 with active run id).

### Done when

- [ ] Scheduled sync runs on configured interval
- [ ] Manual trigger starts async job and returns 202
- [ ] Concurrent trigger returns 409; status endpoints reflect run progress

---

## T-0008 — JWT auth middleware and protected API skeleton

**Status:** open  
**Depends on:** T-0004  
**Decisions:** DEC-0006

### Description

Add JWT validation middleware via IdP JWKS (`OIDC_ISSUER_URL`). Protect all `/api/v1/*` except health routes; return 401 without valid bearer token. Implement `GET /api/v1/settings` (non-secret config display). OpenAPI-ready route registration skeleton.

### Done when

- [ ] Protected routes reject unauthenticated requests with 401
- [ ] Valid SPA bearer token passes JWKS validation
- [ ] `/api/v1/settings` returns safe operator-visible config

---

## T-0009 — React UI shell and OIDC integration

**Status:** open  
**Depends on:** T-0008  
**Decisions:** DEC-0001, DEC-0006

### Description

Scaffold React/TypeScript/Vite app with shadcn/ui. Implement `SidebarProvider` layout, collapsible sidebar, header with read-only badge placeholder, disabled nav placeholders (Forecast, Subscriptions, Planning, Wealth, AI). Wire `react-oidc-context` + `oidc-client-ts`: redirect login, `onSigninCallback`, silent renew, protected routes, user menu in sidebar footer.

### Done when

- [ ] Unauthenticated users redirect to IdP login
- [ ] Post-login session persists; logout works
- [ ] Layout matches architecture § React UI shell (routes stubbed)

---

## T-0010 — Home and Sync Status UI pages

**Status:** open  
**Depends on:** T-0007, T-0009  
**Decisions:** —

### Description

Implement Home (`/`): welcome card, sync summary stats, read-only badge, links to Sync Status and Settings. Implement Sync Status (`/sync`): entity count cards by Firefly type, last sync time, manual "Sync now" button, sync history table with status badges. Settings (`/settings`): Firefly URL/auth display, DB mode, sync interval (read-only), OIDC issuer display. Use TanStack Query polling sync endpoints with bearer token.

### Done when

- [ ] Home and Sync Status render live data from backend APIs
- [ ] Manual sync trigger invokes `POST /api/v1/sync/trigger` and refreshes status
- [ ] Read-only indicator visible in header on all pages

---

## T-0011 — Grafana datasource and Platform Health dashboard

**Status:** open  
**Depends on:** T-0001, T-0005  
**Decisions:** —

### Description

Add Grafana provisioning: PostgreSQL/TimescaleDB datasource YAML with env substitution pointing at external DB. Optional Platform Health dashboard JSON: sync duration, last successful sync, records per entity, sync error rate, backend health/uptime metrics emitted by sync/backend (thin dashboard acceptable).

### Done when

- [ ] Grafana starts in minimal profile with auto-provisioned datasource
- [ ] Datasource connects to external PostgreSQL (operator-configured)
- [ ] Platform Health dashboard loads (optional panels acceptable if metrics wired)

---

## T-0012 — Read-only integration test and operator user guide

**Status:** open  
**Depends on:** T-0006, T-0007, T-0010  
**Decisions:** DEC-0004

### Description

Add integration test that runs sync against mock or test Firefly instance and asserts **only GET** requests were issued (via audit log or HTTP mock recorder). Create `docs/user-guides/US-0001.md` covering: prerequisites (external TimescaleDB, Firefly PAT), Compose minimal startup, connecting to local Firefly III, OIDC setup options, verifying read-only sync.

### Done when

- [ ] Integration test fails if any non-GET Firefly request occurs
- [ ] Test runs in CI/local test harness (`tests/run-tests.sh` or equivalent)
- [ ] User guide published with operator setup for dev Firefly connection
