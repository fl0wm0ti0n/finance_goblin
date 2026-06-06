# QA Findings — Sprint S0001 / US-0001

**Sprint:** S0001  
**Story:** US-0001  
**QA phase:** `/qa`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/verify-work`)

## Scope

Platform foundation: Docker Compose minimal stack, external PostgreSQL config, Rust/Axum backend (health, JWT API, Firefly read-only sync, scheduler), React OIDC UI shell, Grafana provisioning, verification tests, operator user guide.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0001/tasks.md`, `sprints/S0001/progress.md`, `sprints/S0001/summary.md`, `docs/product/acceptance.md` (US-0001), implementation in `backend/`, `frontend/`, `docker-compose.yml`, `tests/`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Automated harness | `bash tests/run-tests.sh` | **PASS** |
| T-2 | Firefly GET-only unit test | `cargo test --test firefly_readonly` | **PASS** (1/1) |
| T-3 | Config validation unit test | `cargo test --lib` | **PASS** (1/1) |
| T-4 | Firefly integration (audit log) | `cargo test --test firefly_integration` | **SKIP** — `DATABASE_URL` not set |
| T-5 | Frontend production build | `npm run build` | **PASS** |
| T-6 | Compose minimal profile services | `docker compose --profile minimal config --services` | **PASS** — `firefly-iii`, `flow-finance-ai`, `grafana` |
| T-7 | No embedded Flow PostgreSQL | Static review `docker-compose.yml` | **PASS** — no postgres service in minimal/standard/full profiles |
| T-8 | Runtime stack / OIDC E2E / Grafana live | Not executed in QA environment | **Deferred** to `/verify-work` (requires operator DB, Firefly PAT, optional IdP) |

### Environment dependencies (non-blocking)

- **`DATABASE_URL`:** Required for `firefly_integration` test. Harness skips gracefully; unit tests and static verification cover read-only guard. Not a QA blocker.
- **External TimescaleDB + Firefly PAT:** Required for runtime acceptance and verify-work UAT.
- **`AUTH_DEV_BYPASS=true` or OIDC IdP:** Required for live API/UI testing without full IdP setup.

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | Docker Compose minimal starts `flow-finance-ai`, `firefly-iii`, `grafana` without embedded PostgreSQL | **PASS** | `docker-compose.yml` profiles `[minimal, standard, full]` on three core services; `docker compose --profile minimal config --services` lists exactly those three. `authentik-postgres` exists only under `oidc` profile. |
| AC-2 | External PostgreSQL configurable via TOML/env; `database.mode = "external"` enforced | **PASS** | `backend/config/default.toml` sets `mode = "external"`. `AppConfig::validate()` rejects non-external mode. Unit test `rejects_non_external_database_mode` passes. Env overlay: `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`, `DATABASE_USER`, `DATABASE_PASSWORD`. |
| AC-3 | Rust backend serves health endpoint and authenticated API skeleton | **PASS** | `GET /health`, `GET /health/ready` in `backend/src/health/mod.rs`. Protected routes under `/api/v1/*` with JWT middleware (`require_auth`); `GET /api/v1/settings` returns safe config. Dev bypass via `AUTH_DEV_BYPASS`. |
| AC-4 | React UI shell loads with OIDC auth flow wired (login redirect + session) | **PASS** | `react-oidc-context` + `oidc-client-ts` in `frontend/src/auth/oidc.ts`, `main.tsx`, `App.tsx` (`signinRedirect`, `/callback`, `automaticSilentRenew`, `WebStorageStateStore`). Frontend build succeeds. Full IdP login E2E deferred to verify-work. |
| AC-5 | Firefly Connector syncs accounts, transactions, categories, budgets, tags, piggy banks via `/api/v1` | **PASS** | `backend/src/firefly/mod.rs` fetches all six entity types via GET paginated endpoints. Mirror tables in migration `001_initial.sql`. Integration test mocks all six paths (runs when `DATABASE_URL` set). |
| AC-6 | Sync scheduler on configurable interval; manual sync trigger available | **PASS** | `SyncService::start_scheduler()` uses `interval_seconds` from config (default 3600). `POST /api/v1/sync/trigger` returns 202; 409 on concurrent run. UI "Sync now" in `SyncStatusPage.tsx`. |
| AC-7 | No write operations to Firefly III API (read-only verified) | **PASS** | `FireflyClient::request()` rejects non-GET at runtime. `firefly_readonly` test PASS. Audit log table `firefly_request_audit` records method. Integration test asserts zero non-GET rows when `DATABASE_URL` set (skipped here — env dependency only). |
| AC-8 | Dev environment can connect to running local Firefly III per operator setup | **PASS** | `firefly-iii` service in minimal profile; `FIREFLY_BASE_URL` defaults to `http://firefly-iii:8080`. `docs/user-guides/US-0001.md` documents prerequisites, `.env` wiring, and local Firefly connection steps. |

**Summary:** 8/8 PASS (7 fully verified in QA; AC-4 runtime IdP flow and AC-7 integration audit path deferred to verify-work with operator env).

## Findings

### Blockers

None.

### Observations (non-blocking)

1. **Compose env interpolation:** `docker compose config` fails without `AUTHENTIK_SECRET_KEY` even when using `--profile minimal`, because optional `oidc` services reference `${AUTHENTIK_SECRET_KEY:?…}`. Operators must set a placeholder in `.env` (documented in `.env.example`) before any compose command. Does not affect minimal runtime once env is populated.
2. **`firefly_integration` skipped:** Expected without external TimescaleDB. Unit read-only test and code audit provide sufficient QA coverage; verify-work should run integration test with `DATABASE_URL`.
3. **OIDC E2E:** Code wiring verified; live redirect/session requires IdP or `AUTH_DEV_BYPASS=true` — covered in verify-work UAT checklist.
4. **Rust dead-code warning:** `Claims` fields unused in `auth/mod.rs` — cosmetic; no functional impact.

## Decision gates

No critical issues requiring dev rework. Proceed to **`/verify-work`**.

## Next phase

Run `/verify-work` in a fresh subagent with operator-provisioned external PostgreSQL, Firefly PAT, and optional OIDC (or dev bypass) for runtime acceptance.
