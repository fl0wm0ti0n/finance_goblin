# Sprint Release Notes — S0012

**Sprint:** S0012  
**Date:** 2026-06-03  
**Stories:** US-0012  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — verify-work re-ran `cargo test --lib` @ 2026-06-03 (82/82); release re-confirmed 82/82
2. **QA completion gate:** PASS — `sprints/S0012/qa-findings.md`, `sprints/S0012/qa.json` (6/6 AC; 0 blockers)
3. **UAT completeness gate:** PASS-with-prerequisites — `sprints/S0012/uat.json`; integration fixture deferred `DATABASE_BOOTSTRAP_TEST_URL_UNSET`; omniflow bootstrap smoke deferred `OMNIFLOW_HOST_UNAVAILABLE` (S0010/S0011 precedent)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build`
- `runtime_mode`: omniflow external (US-0010); database auto-provision on first backend start (DEC-0058 / US-0012)
- `runtime_context_ref`: `docs/engineering/runbook.md` § Omniflow external deploy §1 (Preflight — shared Postgres / TimescaleDB)

**Profile rule:** use **`external` only** — do not combine with `bundled-firefly`.

Greenfield dev (app role with `CREATEDB`): omit `DATABASE_BOOTSTRAP_URL` — bootstrap derives maintenance URL from `DATABASE_*`.

Shared homelab `postgres` (app role lacks `CREATEDB`): set **`DATABASE_BOOTSTRAP_URL`** to admin URL (`postgres://…@postgres:5432/postgres`); never commit.

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (Traefik + basic-auth middleware `auth`)
- `health_endpoint`: `https://financegnome.omniflow.cc/health`
- `bootstrap_logs`: backend container logs — filter `bootstrap_reason=` for triage codes

## Verify

- `verification_steps`:
  1. Deploy per US-0010 runbook (external profile); ensure TimescaleDB packages on host Postgres (R-0053).
  2. Confirm `.env` includes `DATABASE_*`; set `DATABASE_BOOTSTRAP_URL` when app role lacks `CREATEDB` (see `.env.example` omniflow bootstrap block).
  3. **Greenfield:** first start against missing `flow_finance_ai` → log `bootstrap_reason=database_bootstrap_created` then `database_bootstrap_extension_ok`.
  4. **Second start:** log `bootstrap_reason=database_bootstrap_skipped_exists` (no DROP/recreate).
  5. **TimescaleDB check:** `psql` on `flow_finance_ai`: `SELECT extversion FROM pg_extension WHERE extname='timescaledb';` → non-null.
  6. In-network: `curl -sf http://flow-finance-ai:8080/health` → OK JSON.
  7. Edge: `curl -sfI https://financegnome.omniflow.cc/health` (with Traefik basic-auth) → 200.
  8. Automated: `cargo test --lib` (82 tests); optional `cargo test --test database_bootstrap_integration` when `DATABASE_BOOTSTRAP_TEST_URL` set.
  9. **Privilege path:** when app role lacks `CREATEDB` and `DATABASE_BOOTSTRAP_URL` unset → `database_bootstrap_failed_privilege` (fail closed).
  10. **TimescaleDB absent:** host without extension → `database_bootstrap_failed_timescaledb` before migration panic.
- `expected_health_signal`: database exists; TimescaleDB extension present; migrations complete; `/health` OK; bootstrap logs show success-path reason codes; passwords never logged

## Credentials

- `credential_source_refs` (env names only):
  - US-0010 block: `COMPOSE_FILE`, `COMPOSE_PROFILES`, `DATABASE_PASSWORD`, `DATABASE_HOST`, `DATABASE_NAME`, `DATABASE_USER`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, `TRAEFIK_HOST`, `TRAEFIK_MIDDLEWARE`
  - US-0012: optional **`DATABASE_BOOTSTRAP_URL`** (maintenance DB `postgres`; admin/superuser when app role lacks `CREATEDB`)
- `expected_value_source`: operator `.env` at repo root on omniflow host (from `.env.example`)

## Known Issues

- Omniflow bootstrap smoke **PENDING** operator post-deploy — greenfield create, skip-on-restart, privilege path, TimescaleDB-absent (`OMNIFLOW_HOST_UNAVAILABLE` / `DATABASE_BOOTSTRAP_TEST_URL_UNSET` at QA/verify-work).
- Set `DATABASE_BOOTSTRAP_TEST_URL` in CI to witness `database_bootstrap_integration` live path.
- `compose-config-check.sh` may fail on env-dependent `DATABASE_HOST` external merge assert (unrelated; carry-forward from S0011).
- Integration tests require operator `DATABASE_URL` (carry-forward).
- US-0011 omniflow analytics smoke still deferred (carry-forward).

## Deliverables (US-0012)

- `backend/src/db/bootstrap.rs` — `ensure_database` idempotent `CREATE DATABASE … OWNER` + TimescaleDB extension via maintenance creds
- Eight stable `bootstrap_reason` log codes; URL redaction in tracing
- `DATABASE_BOOTSTRAP_URL`, `maintenance_database_url()`, `validate_database_name()` allowlist in config
- `lib.rs` startup — `ensure_database` before `connect_with_retry`
- `.env.example` omniflow bootstrap block; runbook § Omniflow §1 auto-provision delta
- Unit tests (82 lib); gated `database_bootstrap_integration` test
- Decision: DEC-0058

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0012 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.12.0-us0012`

## Milestone

**US-0012 released** — database auto-provision on first start; **backlog segment complete** (US-0001–US-0012 all released).
