# PO to TL archive pack (2026-06-05)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 7
- First archived heading: `## discovery-20260603-us0012 — US-0012 auto-provision application database discovery`
- Last archived heading: `## discovery-20260603-us0012 — US-0012 auto-provision application database discovery`
- Verification tuple (mandatory):
  - archived_body_lines=142
  - retained_body_lines=432

---

## discovery-20260603-us0012 — US-0012 auto-provision application database discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-03  
**Story:** US-0012  
**Next phase:** `/architecture` (optional `/research` for grant-pattern + CI fixture spikes)

### Summary

Discovery refines **automatic provisioning of `flow_finance_ai`** on first backend start when the configured `DATABASE_NAME` is absent on external PostgreSQL. Today (`US-0010` / runbook § Omniflow external deploy) operators run manual `CREATE DATABASE` + grants + `CREATE EXTENSION timescaledb` before `docker compose up`. **`backend/src/lib.rs`** connects via `DbPool::connect_with_retry` then `run_migrations` — connection retry exhausts on missing DB (`3D000`) with no bootstrap hook. Intake + **[R-0055](docs/engineering/research.md#r-0055--auto-provision-flow_finance_ai-database-on-external-postgres-startup)** recommend **in-app pre-migration bootstrap (option A)** with optional admin URL on shared homelab `postgres`.

**US-0010 context (unchanged posture):** external profile attaches to host container `postgres` on Docker network `traefik`; no `postgres` service in finance_goblin Compose; Firefly ledger DB remains separate; TimescaleDB **server packages** still operator-owned (R-0053 §1).

### Env contract (discovery canonical)

| Variable | Required | Purpose |
|----------|----------|---------|
| `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`, `DATABASE_USER`, `DATABASE_PASSWORD` | runtime (existing) | App connection + migrations — unchanged |
| `DATABASE_BOOTSTRAP_URL` | **optional** | Full postgres URL to **maintenance database** (typically `…/postgres`); admin/superuser credentials; **never committed** |
| _(derived, no new env)_ | when bootstrap URL unset | Build maintenance URL from `DATABASE_*` with db name **`postgres`** (not `template1`) |

**Resolution order:**

1. If `DATABASE_BOOTSTRAP_URL` set → use for maintenance connection only.
2. Else → derive `postgres://{USER}:{PASSWORD}@{HOST}:{PORT}/postgres` from runtime vars (same user/password as app).
3. Maintenance role must hold **`CREATEDB`** (or superuser) to create `DATABASE_NAME`.
4. Runtime `DATABASE_*` remain the app role after create — bootstrap must **`GRANT ALL PRIVILEGES ON DATABASE`** (or `ALTER DATABASE … OWNER TO`) for `DATABASE_USER` when admin URL ≠ app user.

**Security guidance (docs):** prefer dedicated bootstrap role with `CREATEDB` only; rotate bootstrap URL out of `.env` after first successful deploy if policy requires; never log URL passwords (redact user/host/db only).

**Out of scope env:** auto-create `DATABASE_USER` / role (defer per R-0055); `DATABASE_URL` override for bootstrap (use `DATABASE_BOOTSTRAP_URL` instead).

### Bootstrap sequence (idempotent)

Insert **`DbPool::ensure_database(config)`** in `backend/src/lib.rs` **before** `connect_with_retry`:

```
maintenance connect → existence check → [create DB + grants] → extension attempt on app DB → app connect_with_retry → run_migrations
```

| Step | Action | Idempotency |
|------|--------|-------------|
| 1 | Connect to maintenance DB (`postgres`) | retry with short backoff (reuse or subset of app retry budget) |
| 2 | `SELECT 1 FROM pg_database WHERE datname = $1` for `DATABASE_NAME` | portable (R-0055) |
| 3a | **Absent:** `CREATE DATABASE` (identifier from config only — validate `[a-zA-Z_][a-zA-Z0-9_]*` or quoted safe subset) | never `DROP` / `CREATE OR REPLACE` |
| 3b | **Present:** skip create | never recreate |
| 4 | Grant app role on new DB when bootstrap user ≠ app user | skip when already owner |
| 5 | Connect to **app DB** (maintenance or app creds per privilege) → `CREATE EXTENSION IF NOT EXISTS timescaledb` | run on **new DB** and on **existing DB** when extension missing (AC-3) |
| 6 | Fail closed if step 3 lacks privilege **or** step 5 fails because TimescaleDB not installed on server | do not proceed to migrations |
| 7 | `connect_with_retry` → `run_migrations` | migration 001 `CREATE EXTENSION IF NOT EXISTS timescaledb` remains — harmless duplicate |

**Wrong-password behavior (unchanged):** bootstrap may succeed via admin URL while app connect still fails — bootstrap does not fix credential typos (intake constraint).

### Structured log reason codes (canonical)

Use tracing field **`bootstrap_reason`** (stable contract for operators/CI):

| Code | Level | When |
|------|-------|------|
| `database_bootstrap_started` | info | Maintenance connection established |
| `database_bootstrap_created` | info | `CREATE DATABASE` executed |
| `database_bootstrap_skipped_exists` | info | DB already present |
| `database_bootstrap_grants_applied` | info | Grants/owner set for app role |
| `database_bootstrap_extension_ok` | info | TimescaleDB extension present or created |
| `database_bootstrap_failed_privilege` | error | `42501` / insufficient privilege on create — hint `DATABASE_BOOTSTRAP_URL` |
| `database_bootstrap_failed_timescaledb` | error | Extension unavailable — pointer runbook **US-0010 §1** / architecture TimescaleDB preflight |
| `database_bootstrap_failed_connect` | error | Maintenance DB unreachable |

Human-readable message must cite runbook section; do not echo bootstrap URL secrets.

### TimescaleDB + migration alignment

| Layer | Responsibility |
|-------|----------------|
| Host OS packages + `shared_preload_libraries` | Operator (R-0053 §1) — **out of scope** US-0012 |
| `CREATE EXTENSION timescaledb` on app DB | Bootstrap attempt **before** migrations (clear error) + migration `001_initial.sql` (released, keep) |
| Hypertables (002+) | Unchanged — require extension present |

**PO recommendation:** bootstrap TimescaleDB failure uses **`database_bootstrap_failed_timescaledb`** with same remediation text as runbook §1 (install packages → restart → extension). Avoid raw SQLx migration panic as first operator signal.

**Alternative considered:** remove extension from migration 001 for external mode — **rejected** (breaks US-0002–US-0009; violates released architecture).

### Privilege matrix (omniflow / greenfield)

| Deployment | Typical app role | Bootstrap path |
|------------|------------------|----------------|
| Greenfield dev (`DATABASE_USER=finance` with CREATEDB) | has CREATEDB | derived maintenance URL, no `DATABASE_BOOTSTRAP_URL` |
| Omniflow shared `postgres` | `finance` without CREATEDB | **`DATABASE_BOOTSTRAP_URL`** with admin/`postgres` superuser |
| CI / testcontainers | superuser in fixture | derived URL sufficient |
| DB already exists | any | skip create; extension attempt only if missing |

### Partial implementation review (repo-only; no host `.env`)

| Area | Status | Execute follow-up |
|------|--------|-------------------|
| `backend/src/lib.rs` startup | **Missing bootstrap** | call `ensure_database` before `connect_with_retry` |
| `backend/src/db/mod.rs` | connect + migrate only | add bootstrap module + reason codes |
| `backend/src/config/mod.rs` | no `DATABASE_BOOTSTRAP_URL` | parse optional URL; maintenance URL builder |
| `backend/migrations/001_initial.sql` | `CREATE EXTENSION IF NOT EXISTS timescaledb` | keep; align error messaging with bootstrap |
| `.env.example` omniflow block | manual SQL comments | add `DATABASE_BOOTSTRAP_URL` + shrink manual steps to TimescaleDB host install |
| `docs/engineering/runbook.md` § Omniflow §1 | manual `CREATE DATABASE` | auto-provision path + when bootstrap URL required |
| `docs/engineering/architecture.md` US-0010 TimescaleDB | manual DB create step 1 | cross-link US-0012 bootstrap |
| Automated test | none for create-if-missing | integration test or CI fixture (AC-6) |

### Discovery decomposition evidence

- Feature/workflow count: bootstrap hook + env/docs + test fixture (low–moderate — **single story retained**)
- Cross-cutting impact: `backend/src/db`, `config`, `.env.example`, runbook, optional compose comment only (no new services)
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md#US-0012`)
- Risk surface: bootstrap URL secret leakage in logs; grant/owner semantics on shared host; TimescaleDB absent after DB create; privilege false negatives

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#US-0012` | Discovery refinements + env contract | pass (updated) |
| `docs/product/acceptance.md#US-0012` | 6 criteria still valid | pass — no AC rewrite |
| `handoffs/intake_evidence/intake-20260602-auto-provision-db.json` | small-intake-pack complete | pass |
| R-0055 | In-app bootstrap + bootstrap URL | pass — linked |
| US-0010 / R-0053 | External postgres + TimescaleDB preflight | pass — bootstrap replaces manual CREATE DATABASE only |

`triad_hot_surface`: rollover 1 US-0011 research section → `handoffs/archive/po-to-tl-pack-20260603-a.md`; US-0012 discovery prepended; retained_body_lines=344, pack_ref=handoffs/archive/po-to-tl-pack-20260603-a.md

### Open questions (carry to `/research` or `/architecture`)

| Topic | Question |
|-------|----------|
| **Owner vs GRANT** | `CREATE DATABASE … OWNER = app_user` vs superuser create + `GRANT ALL` — least privilege on shared homelab `postgres`? |
| **Extension privilege** | App role often lacks `CREATE EXTENSION` — must bootstrap URL user run extension step even when app user creates DB? |
| **CI fixture** | Vanilla `postgres:16` vs `timescale/timescaledb` image for AC-6 — split tests for create path vs extension fail path? |
| **Identifier validation** | Allowlist `DATABASE_NAME` vs PostgreSQL quoted identifiers — injection guard for dynamic SQL |
| **Retry budget** | Share `startup_retry_*` config between maintenance and app connect or separate shorter maintenance budget? |

### Recommended next steps

1. `/architecture` — DEC for bootstrap env contract, reason codes, startup ordering, runbook delta (extends DEC-0056 external Postgres section)
2. `/research` (optional spike) — owner/grant pattern + CI testcontainers matrix if architecture needs evidence
3. `/sprint-plan` — Decompose 6 AC after architecture

---

