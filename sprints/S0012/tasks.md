# Tasks — Sprint S0012

**Story:** US-0012  
**Task count:** 7 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0130 | Database bootstrap module | done | AC-1, AC-2, AC-3, AC-4 |
| T-0131 | Bootstrap config and env contract | done | AC-4 |
| T-0132 | lib.rs startup ordering | done | AC-1 |
| T-0133 | .env.example omniflow bootstrap block | done | AC-5 |
| T-0134 | Runbook Omniflow preflight delta | done | AC-5 |
| T-0135 | Bootstrap integration tests and run-tests.sh | done | AC-6 |
| T-0136 | Bootstrap unit tests | done | AC-6 |

---

## T-0130 — Database bootstrap module

**Status:** open  
**Depends on:** T-0131  
**Decisions:** DEC-0058, R-0055

### Description

Create `backend/src/db/bootstrap.rs` with `DbPool::ensure_database(config: &AppConfig) -> Result<(), BootstrapError>`:

| Step | Action | Idempotency |
|------|--------|-------------|
| 1 | Connect to maintenance DB (`postgres`) — share `startup_retry_*` (DEC-0003) | retry with backoff |
| 2 | `SELECT 1 FROM pg_database WHERE datname = $1` for `DATABASE_NAME` | parameterized |
| 3 | If absent: `CREATE DATABASE "{name}" OWNER "{app_user}"` — validated identifier only | never drop/recreate |
| 4 | If present: skip create | never recreate |
| 5 | Connect to app DB with **maintenance creds** → `CREATE EXTENSION IF NOT EXISTS timescaledb` when missing | run on new and existing DB |
| 6 | Fail closed: `42501` → `database_bootstrap_failed_privilege`; missing extension → `database_bootstrap_failed_timescaledb` | do not proceed to migrations |

Emit tracing field **`bootstrap_reason`** with stable codes per DEC-0058:

- `database_bootstrap_started`, `database_bootstrap_created`, `database_bootstrap_skipped_exists`
- `database_bootstrap_grants_applied` (when bootstrap user ≠ app user and OWNER applied)
- `database_bootstrap_extension_ok`
- `database_bootstrap_failed_privilege`, `database_bootstrap_failed_timescaledb`, `database_bootstrap_failed_connect`

Human-readable messages cite runbook § Omniflow §1 on TimescaleDB failures; never echo bootstrap URL passwords.

Use short-lived connections for maintenance/extension — not app pool.

Export from `backend/src/db/mod.rs` as `pub mod bootstrap;`.

### Done when

- [ ] `ensure_database` implements full idempotent sequence per DEC-0058
- [ ] All eight `bootstrap_reason` codes emitted at correct branches
- [ ] Fail-closed on privilege and TimescaleDB errors with actionable messages
- [ ] No password/secret values in log output
- [ ] `cargo build` succeeds with new module

---

## T-0131 — Bootstrap config and env contract

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0058, R-0055

### Description

Extend `backend/src/config/mod.rs`:

| Function / field | Purpose |
|------------------|---------|
| `database_bootstrap_url: Option<String>` | Parse optional `DATABASE_BOOTSTRAP_URL` env |
| `maintenance_database_url()` | Bootstrap URL if set; else derive `postgres://{USER}:{PASSWORD}@{HOST}:{PORT}/postgres` |
| `validate_database_name()` | Allowlist `^[a-zA-Z_][a-zA-Z0-9_]*$` at `AppConfig::load()` — fail before network |

Resolution order (frozen):

1. If `DATABASE_BOOTSTRAP_URL` set → use for maintenance connection only
2. Else → derive from runtime `DATABASE_*` with db name `postgres` (not `template1`)

Runtime `DATABASE_*` remain app connection after bootstrap. No `DATABASE_URL` override for bootstrap.

### Done when

- [ ] `DATABASE_BOOTSTRAP_URL` parsed as optional env var
- [ ] `maintenance_database_url()` implements resolution order
- [ ] Invalid `DATABASE_NAME` fails at config load with clear error
- [ ] Default `flow_finance_ai` passes validation
- [ ] Unit-testable without network (T-0136)

---

## T-0132 — lib.rs startup ordering

**Status:** open  
**Depends on:** T-0130, T-0131  
**Decisions:** DEC-0058, DEC-0003

### Description

Wire bootstrap into `backend/src/lib.rs` `run()` **before** existing `connect_with_retry`:

```
AppConfig::load()  [DATABASE_NAME allowlist via T-0131]
  → DbPool::ensure_database(&config)     // NEW
  → DbPool::connect_with_retry(&config)  // existing ~line 80
  → db.run_migrations()                  // existing ~line 81
  → (existing service wiring)
```

Bootstrap errors propagate as startup failure — no silent skip.

### Done when

- [ ] `ensure_database` called before `connect_with_retry` in `run()`
- [ ] Bootstrap failure prevents migrations and service start
- [ ] Existing startup retry behavior unchanged for app connect phase
- [ ] `cargo build` and existing startup tests unaffected (aside from new bootstrap path)

---

## T-0133 — .env.example omniflow bootstrap block

**Status:** open  
**Depends on:** T-0131  
**Decisions:** DEC-0058

### Description

Update `.env.example` omniflow / external Postgres section:

| Change | Detail |
|--------|--------|
| Add `DATABASE_BOOTSTRAP_URL` | Optional; full URL to maintenance DB (`…/postgres`); admin/superuser; **never commit** |
| Shrink manual steps | Remove or demote manual `CREATE DATABASE` as **required** pre-step |
| Retain TimescaleDB host install | Comment pointer to runbook §1 — operator-owned server packages |
| Document when required | App role lacks `CREATEDB` on shared homelab `postgres` → set bootstrap URL |
| Document when optional | Greenfield dev with `CREATEDB` app role — derived URL sufficient |

No literal secrets. Cross-reference runbook auto-provision path (T-0134).

### Done when

- [ ] `DATABASE_BOOTSTRAP_URL` documented with purpose and security note
- [ ] Manual `CREATE DATABASE` no longer listed as required operator step
- [ ] TimescaleDB host install prerequisite retained in comments
- [ ] Omniflow shared-Postgres bootstrap URL requirement explained

---

## T-0134 — Runbook Omniflow preflight delta

**Status:** open  
**Depends on:** T-0130, T-0133  
**Decisions:** DEC-0058, DEC-0056 (amended preflight)

### Description

Update `docs/engineering/runbook.md` § Omniflow external deploy §1:

- **Auto-provision path:** backend creates `flow_finance_ai` on first start when absent
- **`DATABASE_BOOTSTRAP_URL`:** when app role (`finance`) lacks `CREATEDB` on shared host `postgres`
- **Derived maintenance URL:** when app role has `CREATEDB` (greenfield dev, CI superuser)
- **TimescaleDB host install:** retained as operator prerequisite — bootstrap fails with `database_bootstrap_failed_timescaledb` when server packages missing
- **Bootstrap log codes:** reference `bootstrap_reason` field for operator triage
- **Security:** prefer dedicated bootstrap role with `CREATEDB` only; rotate bootstrap URL after first deploy if policy requires

Optional: footnote in `decisions/DEC-0056.md` that DB-create preflight is automated by DEC-0058 (TimescaleDB preflight unchanged).

Cross-link `docs/engineering/architecture.md` US-0012 section.

### Done when

- [ ] Runbook §1 describes auto-provision flow and bootstrap URL requirement matrix
- [ ] TimescaleDB host install block retained with remediation text matching bootstrap error
- [ ] Manual `CREATE DATABASE` SQL demoted to optional/troubleshooting only
- [ ] `bootstrap_reason` codes referenced for operator log triage

---

## T-0135 — Bootstrap integration tests and run-tests.sh

**Status:** open  
**Depends on:** T-0130, T-0132  
**Decisions:** DEC-0058, R-0055

### Description

Add `backend/tests/database_bootstrap_integration.rs` (or equivalent integration test target):

| Scenario | Assertion |
|----------|-----------|
| Create-if-missing | Ephemeral unique `DATABASE_NAME`; bootstrap creates DB; idempotent second run skips create |
| Extension present | TimescaleDB extension created or confirmed on app DB |
| Privilege fail (optional variant) | Role without `CREATEDB` and no bootstrap URL → `database_bootstrap_failed_privilege` |

Gate on env var `DATABASE_BOOTSTRAP_TEST_URL` (superuser maintenance access) — skip gracefully in CI when unset.

Wire into `tests/run-tests.sh`:

- Run integration test when `DATABASE_BOOTSTRAP_TEST_URL` set
- Document optional CI jobs: `postgres:16` for create path; `timescale/timescaledb` for extension path (runbook reference)

Use unique ephemeral DB names; teardown with `DROP DATABASE` after test.

### Done when

- [ ] Integration test proves create-if-missing path (AC-6)
- [ ] Idempotent skip-on-exists covered
- [ ] Test gated on `DATABASE_BOOTSTRAP_TEST_URL` with clear skip message when unset
- [ ] `tests/run-tests.sh` invokes bootstrap integration when env available
- [ ] Runbook documents optional CI postgres/timescaledb fixture jobs

---

## T-0136 — Bootstrap unit tests

**Status:** open  
**Depends on:** T-0131, T-0130  
**Decisions:** DEC-0058

### Description

Add `cargo test --lib` coverage in bootstrap/config modules:

| Area | Tests |
|------|-------|
| Name allowlist | Valid names pass; invalid chars / empty rejected |
| Maintenance URL builder | Bootstrap URL precedence; derived URL format from `DATABASE_*` |
| Reason-code mapping | Error variants map to correct `bootstrap_reason` strings |
| Log redaction | Bootstrap URL / password fields redacted in display/log helpers |

No network required — pure config and error-mapping tests.

### Done when

- [ ] Name validation unit tests pass
- [ ] URL builder resolution order tested (bootstrap URL vs derived)
- [ ] Reason-code mapping tested for privilege, TimescaleDB, connect failures
- [ ] Redaction tests confirm no password echo in log formatting
- [ ] Tests pass in `cargo test --lib` without `DATABASE_BOOTSTRAP_TEST_URL`

---

## Execution order (recommended)

1. **Config foundation:** T-0131
2. **Bootstrap module:** T-0130 (after T-0131)
3. **Startup wiring:** T-0132 (after T-0130)
4. **Docs (parallel):** T-0133 ∥ T-0134 (after T-0131/T-0130)
5. **Unit tests:** T-0136 (after T-0130, T-0131)
6. **Integration tests:** T-0135 (last — after T-0132, T-0136)

## Acceptance coverage map

| AC | Tasks |
|----|-------|
| AC1 Cold start creates missing DB idempotently before migrations | T-0130, T-0132, T-0135 |
| AC2 Existing DB never dropped or recreated | T-0130, T-0135 |
| AC3 TimescaleDB extension attempt; actionable fail when server lacks extension | T-0130, T-0134, T-0135 |
| AC4 DATABASE_BOOTSTRAP_URL when app role lacks CREATEDB; fail closed on missing privilege | T-0130, T-0131, T-0135 |
| AC5 .env.example and runbook document bootstrap env and omniflow behavior | T-0133, T-0134 |
| AC6 Automated test proves create-if-missing path | T-0135, T-0136 |

## Split decision

- **Why 7 tasks:** Maps DEC-0058 execute outline (bootstrap module, config, lib wiring, env, runbook, integration + unit tests); within SPRINT_MAX_TASKS=12.
- **Why not S0012a/b:** Bootstrap module, config, and lib wiring share one startup contract; splitting create from extension would allow broken startup between sprints.
- **USER_GUIDE_MODE:** No separate user guide — operator docs covered by runbook + `.env.example` (AC-5).
