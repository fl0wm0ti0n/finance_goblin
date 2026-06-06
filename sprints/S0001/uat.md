# UAT — Sprint S0001 / US-0001

**Sprint:** S0001  
**Story:** US-0001  
**Phase:** `/verify-work`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/release`)  
**Ready for release:** yes (operator runtime prerequisites documented below)

## Inputs

- QA PASS: `sprints/S0001/qa-findings.md`
- Acceptance: `docs/product/acceptance.md` (US-0001)
- Operator guide: `docs/user-guides/US-0001.md`
- Implementation: `backend/`, `frontend/`, `docker-compose.yml`

## Environment

| Prerequisite | Status in verify-work |
|--------------|----------------------|
| `.env` populated | **Not present** — no operator `.env` in workspace |
| `DATABASE_URL` (TimescaleDB) | **Unset** — integration test skipped |
| Firefly PAT + running stack | **Not provisioned** — live sync/OIDC E2E deferred |
| `AUTH_DEV_BYPASS` or OIDC IdP | **Unset** — live auth flow deferred |

Per workflow policy: code-level and automated verification **pass**; runtime E2E steps recorded as **PASS-with-prerequisites** where external infra is required.

## Automated checks

| ID | Check | Command | Result |
|----|-------|---------|--------|
| AUTO-1 | Test harness | `bash tests/run-tests.sh` | **PASS** |
| AUTO-2 | Firefly GET-only unit test | (via harness) `cargo test --test firefly_readonly` | **PASS** (1/1) |
| AUTO-3 | Config validation unit test | (via harness) `cargo test --lib` | **PASS** (1/1) |
| AUTO-4 | Firefly integration (audit log) | (via harness) `cargo test --test firefly_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-5 | Frontend production build | (via harness) `npm run build` | **PASS** |
| AUTO-6 | Compose minimal services | `docker compose --profile minimal config --services` (with placeholder env) | **PASS** — `firefly-iii`, `flow-finance-ai`, `grafana` |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | Minimal Compose profile lists three core services; no embedded Flow PostgreSQL | **PASS** | `docker-compose.yml` profiles `[minimal, standard, full]` on `flow-finance-ai`, `firefly-iii`, `grafana`. `authentik-postgres` only under `oidc` profile. AUTO-6 services list matches. |
| UAT-2 | AC-2 | External PostgreSQL via TOML/env; `database.mode = "external"` enforced | **PASS** | `backend/config/default.toml` `mode = "external"`. Unit test `rejects_non_external_database_mode` PASS. Env overlay: `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`, `DATABASE_USER`, `DATABASE_PASSWORD`. |
| UAT-3 | AC-3 | Backend health endpoints and authenticated API skeleton | **PASS-with-prerequisites** | Code: `GET /health`, `GET /health/ready` in `backend/src/health/mod.rs`; protected `/api/v1/*` with JWT middleware. **Operator prerequisite:** start stack with external DB; curl `/health` (200) and `/health/ready` (200 when DB up); call `/api/v1/settings` with JWT or `AUTH_DEV_BYPASS=true`. |
| UAT-4 | AC-4 | React UI shell with OIDC login redirect + session | **PASS-with-prerequisites** | Code: `react-oidc-context`, `oidc-client-ts`, `signinRedirect`, `/callback`, `WebStorageStateStore`, `automaticSilentRenew` in `frontend/src/`. Build PASS. **Operator prerequisite:** configure `VITE_OIDC_*` or use `AUTH_DEV_BYPASS=true`; open `http://localhost:8080` and complete login redirect. |
| UAT-5 | AC-5 | Firefly Connector syncs six entity types via `/api/v1` | **PASS-with-prerequisites** | Code: GET paginated sync for accounts, transactions, categories, budgets, tags, piggy banks in `backend/src/firefly/mod.rs`; mirror tables in `001_initial.sql`. **Operator prerequisite:** Firefly PAT + `docker compose --profile minimal up`; trigger sync; verify entities in Sync Status UI / DB. |
| UAT-6 | AC-6 | Configurable sync interval; manual sync trigger | **PASS** | `sync.interval_seconds` default 3600 in config; `SyncService::start_scheduler()`; `POST /api/v1/sync/trigger` (202/409); UI **Sync now** in `SyncStatusPage.tsx`. Live trigger validation covered under UAT-5 operator steps. |
| UAT-7 | AC-7 | No write operations to Firefly III API | **PASS-with-prerequisites** | Unit test `rejects_non_get_methods` PASS. Runtime guard in `FireflyClient::request()`. Integration audit test **SKIP** without `DATABASE_URL`. **Operator prerequisite:** set `DATABASE_URL` and run `cargo test --test firefly_integration` to assert zero non-GET rows in `firefly_request_audit`. |
| UAT-8 | AC-8 | Dev environment connects to local Firefly III | **PASS-with-prerequisites** | `firefly-iii` in minimal profile; `FIREFLY_BASE_URL=http://firefly-iii:8080`; `docs/user-guides/US-0001.md` documents `.env`, host DB, PAT, and startup steps. **Operator prerequisite:** follow user guide §Usage steps 1–6. |

## Summary

| Metric | Count |
|--------|-------|
| Acceptance criteria passed (code/automated) | 8/8 |
| Full runtime E2E executed | 0/8 (blocked by missing operator infra) |
| Automated checks passed | 5/6 (1 SKIP — expected) |
| Failed | 0 |

## Operator prerequisites (post-release smoke)

1. Copy `.env.example` → `.env`; set `DATABASE_*`, `FIREFLY_*`, optional `AUTHENTIK_SECRET_KEY` placeholder for compose config.
2. Provision external TimescaleDB; enable extension; create `flow_finance_ai` and `firefly` databases.
3. `docker compose --profile minimal up --build`
4. Complete Firefly setup at `:8081`; create PAT.
5. Sign in at `:8080` (OIDC or `AUTH_DEV_BYPASS=true`); run **Sync now**.
6. Optional: `DATABASE_URL=... cargo test --test firefly_integration` for audit-log read-only proof.

## Findings

### Blockers

None.

### Observations

1. Compose commands require populated `.env` (e.g. `DATABASE_PASSWORD`, `AUTHENTIK_SECRET_KEY` placeholder) — documented in QA and user guide.
2. Full stack smoke test remains operator responsibility; structural and automated verification sufficient for release gate.

## Next phase

Run `/release` in a fresh subagent.
