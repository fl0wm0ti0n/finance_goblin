# Sprint S0012 summary — US-0012 database bootstrap

**Story:** US-0012  
**Sprint:** S0012  
**Phase:** released (`0.12.0-us0012`)  
**Decision:** DEC-0058

## Delivered

- **T-0131:** `DATABASE_BOOTSTRAP_URL`, `maintenance_database_url()`, `app_database_maintenance_url()`, `validate_database_name()` allowlist at config load
- **T-0130:** `backend/src/db/bootstrap.rs` — `ensure_database` idempotent CREATE DATABASE OWNER + TimescaleDB extension; eight `bootstrap_reason` codes; URL redaction
- **T-0132:** `lib.rs` startup — `ensure_database` before `connect_with_retry`
- **T-0133:** `.env.example` omniflow bootstrap block — optional `DATABASE_BOOTSTRAP_URL`; manual CREATE DATABASE demoted
- **T-0134:** Runbook § Omniflow §1 auto-provision delta + optional CI fixture note
- **T-0136:** Unit tests — name allowlist, URL builder, reason codes, redaction (`cargo test --lib`)
- **T-0135:** `database_bootstrap_integration.rs` gated on `DATABASE_BOOTSTRAP_TEST_URL`; wired in `tests/run-tests.sh`

## Tests

| Suite | Result |
|-------|--------|
| `cargo test --lib` | PASS (82, incl. bootstrap + config) |
| `cargo test --test database_bootstrap_integration` | SKIP (env unset) / PASS when `DATABASE_BOOTSTRAP_TEST_URL` set |
| Other backend integration tests | SKIP (no `DATABASE_URL`) |
| `npm run build` | PASS |
| `compose-config-check.sh` | FAIL — pre-existing `DATABASE_HOST` external merge assert (same as S0011; unrelated) |

## Frozen boundaries (verified)

- Migration 001 `CREATE EXTENSION` unchanged
- No Compose init service / embedded Postgres
- No auto-create PostgreSQL roles
- No password/URL logging in bootstrap tracing
- No DROP/recreate existing databases

## Release

- **Version:** `0.12.0-us0012` (2026-06-03)
- **Notes:** `handoffs/releases/S0012-release-notes.md`
- **Findings:** `sprints/S0012/release-findings.md`
- **Context refresh:** 2026-06-03T23:00:00Z — backlog drain complete; checkpoints archived to `docs/engineering/state-archive/state-pack-20260603-s0012.md`
