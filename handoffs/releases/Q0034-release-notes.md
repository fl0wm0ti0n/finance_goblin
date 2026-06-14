# Quick Release Notes — Q0034 / BUG-0025

**Quick task:** Q0034  
**Bug:** BUG-0025 — Firefly Stromkosten mirror lag (backdated import skip)  
**Date:** 2026-06-14  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0025 rows **BW**, **BX**, **BY**; live BW/BX/BY operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --lib` (221/221); `cargo test --test bug0025_sync_transaction_window` (3/3); `npm test` (31/31); `npm run build` PASS; `sprints/quick/Q0034/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/quick/Q0034/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS-WITH-PREREQUISITES — `sprints/quick/Q0034/uat.json`, `sprints/quick/Q0034/uat.md`, `sprints/quick/Q0034/verify-work-findings.md`; 8 steps — 2 pass, 6 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260614-bug0025-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Firefly mirror lag fix per **DEC-0002** extension (no new DEC): manual **Sync now** uses **365-day** transaction-date lookback; scheduled incremental path unchanged (`watermark − overlap_days`); Sync Status hero uses `last_firefly_run` distinct from exchange-only `last_run`; DEC-0002 callout + runbook remediation documented.

| Scope | Fix |
|-------|-----|
| **B1** | `backend/src/firefly/mod.rs` — `sync_transactions(..., trigger)` + `MANUAL_LOOKBACK_DAYS=365` |
| **B2** | `backend/src/sync/mod.rs` — `SyncStatusResponse.last_firefly_run` + `latest_firefly_run()` |
| **F1** | `frontend/src/pages/SyncStatusPage.tsx` — Last Firefly sync hero, trigger badge, exchange secondary, DEC-0002 callout |
| **D1** | `docs/engineering/runbook.md` — § Backdated Firefly imports (`#backdated-firefly-imports`) |
| **T1** | `backend/tests/bug0025_sync_transaction_window.rs` — 3 integration cases |
| **G1** | cargo lib 221/221; bug0025 3/3; npm 31/31; build PASS |
| **V1** | verify-work BW/BX/BY oracles — pass_with_prerequisites; deferred **BACKEND_REBUILD** + **FRONTEND_DEPLOY** |

**Code proof:** bug0025 integration 3/3 (manual 365d start + backdated ingest; scheduled watermark−7d; narrow-window skip); cargo lib 221/221; npm 31/31; runbook cursor-reset SQL documented.

**Operator post-release:** Rebuild backend + frontend (**BACKEND_REBUILD** + **FRONTEND_DEPLOY**) → manual **Sync now** → verify multi-month Stromkosten expense-series + `last_firefly_run` hero.

---

## Run

**Target service:** `flow-finance-ai` (backend + frontend — Firefly sync window + Sync Status UX).

**Deploy (backend + frontend rebuild — no migration):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

- `start_command`: docker compose commands above
- `runtime_mode`: local (`:18080`) and remote (omniflow external US-0010)
- `runtime_context_ref`: `docs/engineering/runbook.md` (§40 BUG-0025 hotfix; `#backdated-firefly-imports`)

**Operator gate — BACKEND_REBUILD (required before BW/BY API + manual 365d ingest smoke):**

Running container predates Q0034; `last_firefly_run` absent from API; expense-series category **146** shows 2026-05 only. Confirm tests pass before docker build:

```bash
cd backend && cargo test --lib && cargo test --test bug0025_sync_transaction_window
cd frontend && npm test && npm run build
```

**Operator gate — FRONTEND_DEPLOY (required before BX callout + BY hero smoke):**

Sync Status hero shows exchange `scheduled_exchanges` timestamp pre-deploy; DEC-0002 callout absent. Same docker compose rebuild satisfies both gates.

**Post-deploy — manual Full sync:**

On **Sync Status** (`/sync`), click **Sync now** — triggers manual Full Firefly ingest with 365-day lookback.

---

## Connect

- `service_url`: `http://localhost:18080` (local) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 18080 (local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET http://localhost:18080/health` → JSON 200
- Sync Status UI: `http://localhost:18080/sync`
- Forecast UI: `http://localhost:18080/forecast` (Category spending trend — **Wohnen - Stromkosten**)
- BW API oracle: `GET /api/v1/categories/expense-series?category_id=146` → multi-month bars after manual Sync now
- BY API oracle: `GET /api/v1/sync/status` → `last_firefly_run` distinct from exchange-only `last_run`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | External PostgreSQL — mirror + sync cursors |
| `FIREFLY_BASE_URL` | Firefly III API — transaction ingest |
| `AUTHENTIK_SECRET_KEY` | External compose profile build gate (set dummy for local external profile) |
| OIDC provider config | Omniflow OIDC-1 regression — Compose/env only |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(BW)** | Manual **Sync now** → `GET /api/v1/categories/expense-series?category_id=146` | Multi-month Stromkosten outflow bars — not 2026-05 only |
| **(BW)** | `/forecast` Category spending trend **Wohnen - Stromkosten** | Bars per month with Firefly data |
| **(BX)** | `/sync` DEC-0002 info callout + runbook link | Callout visible; links to `#backdated-firefly-imports` |
| **(BX)** | Runbook cursor reset | `DELETE FROM sync_cursors WHERE entity_type = 'transactions'` documented |
| **(BY)** | `/sync` hero **Last Firefly sync** + trigger badge | Hero uses `last_firefly_run`; exchange secondary when newer exchange-only run |
| **(BY)** | Sync history `trigger` column | `manual` / `scheduled` vs `scheduled_exchanges` distinguished |
| Regression | OIDC-1 `/sync` + `/forecast` | HTTP 200 after deploy |

**Automated (release):**

```bash
cd backend && cargo test --lib && cargo test --test bug0025_sync_transaction_window
cd frontend && npm test && npm run build
```

**Live (operator):** UAT steps in `sprints/quick/Q0034/uat.json`.

---

## Credentials

- `DATABASE_URL` — external PostgreSQL (operator shell only — no inline secrets)
- `FIREFLY_BASE_URL` + Firefly API token — via Compose/env only
- OIDC provider config via Compose/env only

---

## Changes

| Area | Summary |
|------|---------|
| `backend/src/firefly/mod.rs` | B1 — manual 365d lookback by trigger |
| `backend/src/sync/mod.rs` | B2 — `last_firefly_run` API split |
| `frontend/src/lib/api.ts` | `last_firefly_run: SyncRun \| null` |
| `frontend/src/pages/SyncStatusPage.tsx` | F1 hero + trigger badge + exchange secondary + DEC-0002 callout |
| `backend/tests/bug0025_sync_transaction_window.rs` | T1 — 3 integration cases |
| Runbook | §40 BUG-0025 hotfix + `#backdated-firefly-imports` |

**Linked decisions:** DEC-0002 (incremental overlap; manual 365d exception)  
**Research fulfilled:** R-0097  
**Deferred:** BW/BX/BY live browser/API smoke until **BACKEND_REBUILD** + **FRONTEND_DEPLOY**

---

## Known Issues

- Running container predates Q0034 — Stromkosten expense-series 2026-05 only; hero shows exchange timestamp; `last_firefly_run` absent until rebuild
- Live localhost reproduces pre-deploy symptom per verify-work browser probes
- Rows older than 365 days require cursor reset + manual Full sync per runbook

---

## Regression scope

- Plan delete sole-plan hint (BUG-0024 / Q0033) unchanged
- Forecast Income card (BUG-0026 / Q0032) unchanged
- Crypto wealth EUR (BUG-0023) unchanged
- Scheduled incremental sync (`watermark − overlap_days`) unchanged

---

## Rollback

```bash
git revert <Q0034-code-commits>
docker compose up -d --build flow-finance-ai
```

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0034 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0025-q0034`

## Milestone

**BUG-0025 released** — manual 365d lookback + `last_firefly_run` hero + DEC-0002 remediation; code/integration oracles PASS; deploy operator-deferred per pass-with-prerequisites.
