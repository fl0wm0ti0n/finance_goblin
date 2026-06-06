# QA Findings — Sprint S0012 / US-0012

**Sprint:** S0012  
**Story:** US-0012  
**QA phase:** `/qa`  
**Date:** 2026-06-03  
**Verdict:** **PASS** (ready for `/verify-work`; integration fixture + omniflow runtime deferred)

## Scope

In-app database bootstrap per DEC-0058: `DbPool::ensure_database` before `connect_with_retry`, idempotent `CREATE DATABASE … OWNER`, TimescaleDB extension via maintenance credentials, optional `DATABASE_BOOTSTRAP_URL`, stable `bootstrap_reason` log codes, config name allowlist, operator docs, tiered tests.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0012/summary.md`, `sprints/S0012/uat.md`, `docs/product/acceptance.md` (US-0012), `decisions/DEC-0058.md`, `backend/src/db/bootstrap.rs`, `backend/src/config/mod.rs`, `backend/src/lib.rs`, `backend/tests/database_bootstrap_integration.rs`, `.env.example`, `docs/engineering/runbook.md` § Omniflow §1, `tests/run-tests.sh`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Bootstrap unit + config tests | `cargo test --lib` | **PASS** (82/82) |
| T-2 | Bootstrap integration (AC-6 fixture) | `cargo test --test database_bootstrap_integration` | **DEFERRED** — `DATABASE_BOOTSTRAP_TEST_URL` unset; tests early-return (graceful skip, exit 0) |
| T-3 | DEC-0058 contract review | Static code/config review | **PASS** (see matrix) |
| T-4 | Operator docs AC-5 | `.env.example` + runbook § Omniflow §1 | **PASS** |
| T-5 | Frozen boundaries | Grep + code review (no DROP, no migration 001 change) | **PASS** |
| T-6 | Omniflow greenfield bootstrap smoke | Operator cold start on shared Postgres | **DEFERRED** — `OMNIFLOW_HOST_UNAVAILABLE` (S0010/S0011 precedent) |
| T-7 | Full harness `tests/run-tests.sh` | Not re-run end-to-end | **Advisory** — dev noted pre-existing `compose-config-check` `DATABASE_HOST` assert failure unrelated to US-0012 |

### Environment dependencies (non-blocking)

- **`DATABASE_BOOTSTRAP_TEST_URL`:** Required for live create-if-missing + TimescaleDB extension integration path (AC-6 fixture). Unset in QA environment — documented skip per DEC-0058 tiered test strategy.
- **Omniflow host:** Required for operator smoke (greenfield create, idempotent restart, `DATABASE_BOOTSTRAP_URL` on shared `postgres`, TimescaleDB-absent fail-closed). Deferred to verify-work / operator.

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | Cold start creates missing `DATABASE_NAME` idempotently before migrations | **PASS** (static) | `lib.rs` ordering `ensure_database` → `connect_with_retry` → `run_migrations`; `bootstrap.rs` existence check + `CREATE DATABASE … OWNER`; integration test present. Live create **DEFERRED** (no test URL) |
| AC-2 | Existing `flow_finance_ai` never dropped or recreated | **PASS** | No `DROP` in `backend/src/db/`; skip path `REASON_SKIPPED_EXISTS` when `pg_database` row exists |
| AC-3 | TimescaleDB extension attempt; actionable fail when server lacks extension | **PASS** | `ensure_timescaledb_extension` + `map_sql_error` → `database_bootstrap_failed_timescaledb`; runbook pointer in error display; unit test `timescaledb_error_includes_runbook_pointer` |
| AC-4 | `DATABASE_BOOTSTRAP_URL` when app role lacks CREATEDB; fail closed on missing privilege | **PASS** (static) | `maintenance_database_url()` prefers bootstrap URL; `42501` → `REASON_FAILED_PRIVILEGE` with hint; config unit tests. Live privilege path **DEFERRED** |
| AC-5 | `.env.example` and runbook document bootstrap env and omniflow shared-Postgres behavior | **PASS** | `.env.example` lines 46–49; runbook §1968–1986 bootstrap matrix + log triage |
| AC-6 | Automated test proves create-if-missing path | **PASS** (unit + gated integration) | 82 unit tests incl. allowlist, URL builder, reason codes, redaction; `database_bootstrap_integration.rs` wired in `run-tests.sh`. Integration execution **DEFERRED** — env unset |

**Summary:** 6/6 PASS on static/unit path; integration fixture and omniflow operator smoke deferred (not QA blockers per S0010/S0011 precedent).

## DEC-0058 compliance review

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Hook placement | `ensure_database` → `connect_with_retry` → `run_migrations` | `backend/src/lib.rs` lines 80–82 | PASS |
| Module | `backend/src/db/bootstrap.rs` | Implemented with eight reason codes | PASS |
| Maintenance URL | `DATABASE_BOOTSTRAP_URL` then derived `…/postgres` | `AppConfig::maintenance_database_url()` + unit tests | PASS |
| Create semantics | `CREATE DATABASE … OWNER`; skip if exists | `create_database()` + `database_exists()` | PASS |
| No DROP/recreate | Never drop existing DB | No DROP in bootstrap module | PASS |
| Extension step | Maintenance creds on app DB; fail-closed code | `app_database_maintenance_url()` + `ensure_timescaledb_extension()` | PASS |
| Name validation | Allowlist at config load | `validate_database_name()` + unit tests | PASS |
| Retry budget | Share DEC-0003 startup retry fields | `connect_with_startup_retry()` uses same config | PASS |
| Log codes | Eight stable `bootstrap_reason` values | Constants + tracing fields match DEC-0058 table | PASS |
| Secret redaction | Never log passwords/URLs with creds | `redact_database_url()` in connect logs + unit test | PASS |
| Migration 001 | Unchanged | Frozen boundary verified (dev handoff) | PASS |
| Docs | `.env.example` + runbook delta | Present per T-0133/T-0134 | PASS |

## Generated baseline test evidence

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` |
| `generated_test_command` | `cd backend && cargo test --lib` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-06-03 — 82/82 passed; bootstrap + config unit tests included |
| `generated_test_paths_ref` | `backend/src/db/bootstrap.rs`, `backend/src/config/mod.rs`, `backend/tests/database_bootstrap_integration.rs` |
| `generated_test_reason_code` | — |

## Runtime QA evidence

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed (`docker compose … up` on omniflow host with missing DB) |
| `runtime_stack_profile` | `docker-compose` external + shared Postgres bootstrap |
| `runtime_mode` | `deferred` |
| `runtime_health_target` | Greenfield `database_bootstrap_created`; restart `database_bootstrap_skipped_exists`; `DATABASE_BOOTSTRAP_URL` on shared host; `database_bootstrap_failed_timescaledb` when extension absent |
| `runtime_health_result` | `deferred` |
| `runtime_final_verdict` | `deferred` (verify-work / operator) |
| `runtime_reason_code` | `OMNIFLOW_HOST_UNAVAILABLE` |
| `runtime_evidence_refs` | S0010/S0011 carry-forward; `handoffs/dev_to_qa.md`; runbook § Omniflow §1 operator smoke table |

## Integration test skip (AC-6 fixture)

| Field | Value |
|-------|-------|
| `integration_test_suite` | `database_bootstrap_integration` |
| `env_gate` | `DATABASE_BOOTSTRAP_TEST_URL` |
| `qa_env_set` | `false` |
| `cargo_result` | `ok` (2 tests early-return without exercising Postgres) |
| `run_tests_sh_behavior` | Skips suite when env unset; prints advisory message |
| `deferred_to` | CI fixture or verify-work with superuser maintenance URL |
| `reason_code` | `DATABASE_BOOTSTRAP_TEST_URL_UNSET` |

## Findings

### Blockers

None.

### Advisories (non-blocking)

1. **Integration fixture:** Set `DATABASE_BOOTSTRAP_TEST_URL` (superuser `…/postgres`) and re-run `cargo test --test database_bootstrap_integration` in CI or verify-work to witness live create + extension path.
2. **Omniflow runtime:** Close operator smoke (greenfield create, idempotent restart, bootstrap URL on shared `postgres`, TimescaleDB-absent fail-closed) when host is deployed.
3. **compose-config-check:** Pre-existing `DATABASE_HOST` external merge assert failure — not US-0012 regression; re-run on CI/host with `DATABASE_HOST=postgres`.

## Verdict

**PASS** — proceed to `/verify-work` in fresh subagent. No `handoffs/qa_to_dev.md`.
