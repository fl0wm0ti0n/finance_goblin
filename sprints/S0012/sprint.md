# Sprint S0012

**ID:** S0012  
**Story:** US-0012 — Auto-provision application database on first start  
**Status:** PLANNED  
**Created:** 2026-06-03

## Goal

Deliver **DEC-0058** in-app database bootstrap: `DbPool::ensure_database` before `connect_with_retry`, idempotent `CREATE DATABASE … OWNER`, TimescaleDB extension via maintenance credentials, optional `DATABASE_BOOTSTRAP_URL`, stable `bootstrap_reason` log codes, operator docs (`.env.example` + runbook), and tiered automated tests — eliminating manual `CREATE DATABASE` on omniflow external Postgres while preserving fail-closed TimescaleDB preflight.

## Scope

- `backend/src/db/bootstrap.rs` — `ensure_database`, existence check, create, extension, reason codes
- `backend/src/config/mod.rs` — `DATABASE_BOOTSTRAP_URL`, `maintenance_database_url()`, `validate_database_name()`
- `backend/src/db/mod.rs` — `pub mod bootstrap;`
- `backend/src/lib.rs` — startup ordering: `ensure_database` → `connect_with_retry` → `run_migrations`
- `.env.example` omniflow block — `DATABASE_BOOTSTRAP_URL`; shrink manual CREATE DATABASE steps
- `docs/engineering/runbook.md` § Omniflow §1 — auto-provision path; when bootstrap URL required
- Unit tests — name allowlist, URL builder, reason-code mapping, log redaction
- Integration test — `database_bootstrap_integration.rs` gated on `DATABASE_BOOTSTRAP_TEST_URL`; wire in `tests/run-tests.sh`

**Out of scope:** Compose init service; embedded Postgres; auto-create `DATABASE_USER`; host TimescaleDB package install; remove migration 001 extension line; bootstrap URL password logging.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Secret leakage in bootstrap logs | Redact URL/password fields in tracing; unit test redaction | DEC-0058, T-0136 |
| Extension privilege on shared host | Extension step uses maintenance creds on app DB | DEC-0058 §4, T-0130 |
| TimescaleDB absent after DB create | Fail before migrations with `database_bootstrap_failed_timescaledb` + runbook pointer | R-0053 §1, T-0130, T-0134 |
| Integration test flakiness | Unique ephemeral DB names; teardown `DROP DATABASE` | T-0135 |
| Omniflow operator confusion | Runbook states when `DATABASE_BOOTSTRAP_URL` required vs optional | T-0133, T-0134 |
| Wrong app password after bootstrap | Unchanged behavior — bootstrap does not fix credential typos | DEC-0058 frozen boundary |

## Definition of Done

- All 7 sprint tasks complete (`T-0130` … `T-0136`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0012 (6 AC)
- Cold start against external Postgres creates missing `DATABASE_NAME` idempotently before migrations
- Existing database never dropped or recreated
- Bootstrap attempts TimescaleDB extension; fails closed with actionable log when server lacks extension
- `DATABASE_BOOTSTRAP_URL` enables bootstrap when app role lacks `CREATEDB`
- `.env.example` and runbook document bootstrap env and omniflow shared-Postgres behavior
- Unit tests pass in CI; integration test wired (gated or optional CI job documented)

## Architecture references

- `docs/engineering/architecture.md` — US-0012
- `decisions/DEC-0058.md`
- Research: R-0055, R-0053 §1
- Extends: DEC-0003; amends DEC-0056 DB-create preflight
- Discovery: `handoffs/po_to_tl.md#discovery-20260603-us0012`
- Acceptance: `docs/product/acceptance.md#US-0012`
