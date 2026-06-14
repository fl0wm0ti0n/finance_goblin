# Quick Release Notes — Q0031 / BUG-0022

**Quick task:** Q0031  
**Bug:** BUG-0022 — Plan delete selector regression (dropdown ignored)  
**Date:** 2026-06-13  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0022 rows **BM**, **BN**; live BM/BN browser operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `npm test` (17/17); `npm run build` PASS; `cargo test --lib active_plan_delete` (1/1); `sprints/quick/Q0031/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/quick/Q0031/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS-WITH-PREREQUISITES — `sprints/quick/Q0031/uat.json`, `sprints/quick/Q0031/uat.md`, `sprints/quick/Q0031/verify-work-findings.md`; 5 steps — 1 pass, 4 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260613-bug0022-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Plan delete selector regression fix per **DEC-0082** (active plan delete guard), **DEC-0024** (planning selector contract), **DEC-0074** (planning UX). Frontend-only: dropdown selection now drives displayed plan and delete enablement.

| Scope | Fix |
|-------|-----|
| **BM1** | `frontend/src/pages/PlanningPage.tsx` — `resolveDisplayedPlanId` (`selectedPlanId ?? globalActiveId ?? firstPlanId`) |
| **T1** | `frontend/src/pages/planSelector.ts` + `planSelector.test.ts` — 8 vitest cases for selector + `isDeleteDisabled` matrix |
| **G1** | `npm test` 17/17; `npm run build` PASS — frontend-only blast radius |
| **V1** | verify-work BM/BN oracles — BN-API live 409 PASS; BM/BN browser deferred **FRONTEND_DEPLOY** |
| **L1** | skipped — optional P2 dropdown label rename |

**Code proof:** planSelector 8/8; npm 17/17; npm build PASS; backend `active_plan_delete_returns_409_with_code` 1/1; live DELETE active → 409 `active_plan_delete_forbidden`.

**Operator post-release:** Rebuild frontend only (**FRONTEND_DEPLOY** — no migration).

---

## Run

**Target service:** `flow-finance-ai` (frontend SPA only — no backend changes).

**Deploy (frontend rebuild):**

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
- `runtime_context_ref`: `docs/engineering/runbook.md` (§36 BUG-0022 hotfix)

**Operator gate — FRONTEND_DEPLOY (required before BM/BN live UI smoke):**

Running container predates Q0031; `/planning` returns **404** pre-deploy. Confirm tests pass before docker build:

```bash
cd frontend && npm test && npm run build
```

No migration. Backend unchanged — **DEC-0082** 409 guard already live.

---

## Connect

- `service_url`: `http://localhost:18080` (local) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 18080 (local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET http://localhost:18080/health` → JSON 200
- Planning UI: `http://localhost:18080/planning`
- BN API oracle: `DELETE /api/v1/plans/:active_id` → **409** `active_plan_delete_forbidden`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | External PostgreSQL — multi-plan smoke requires 2+ plans |
| `AUTHENTIK_SECRET_KEY` | External compose profile build gate (set dummy for local external profile) |
| OIDC provider config | Omniflow OIDC-1 regression — Compose/env only |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(BM)** | `/planning` with 2+ plans (one global active) | Select non-active plan → **Delete plan** enabled |
| **(BM)** | Confirm delete modal | Non-active plan removed; list refreshes |
| **(BN)** | Select globally active plan | Delete **disabled** + tooltip *Set another plan active before deleting the active plan* |
| **(BN)** | `DELETE /api/v1/plans/:active_id` | **409** `active_plan_delete_forbidden` (live-confirmed pre-deploy) |
| Regression | OIDC-1 omniflow `/planning` + `/api/v1/plans` | HTTP 200 after deploy |

**Automated (release):**

```bash
cd frontend && npm test && npm run build
cd backend && cargo test --lib active_plan_delete
```

**Live (operator):** UAT steps in `sprints/quick/Q0031/uat.json`.

---

## Credentials

- `DATABASE_URL` — external PostgreSQL (operator shell only — no inline secrets)
- OIDC provider config via Compose/env only

---

## Changes

| Area | Summary |
|------|---------|
| `frontend/src/pages/PlanningPage.tsx` | BM1 selector priority fix via `resolveDisplayedPlanId` |
| `frontend/src/pages/planSelector.ts` | T1 pure helpers: `resolveDisplayedPlanId`, `isDeleteDisabled` |
| `frontend/src/pages/planSelector.test.ts` | T1 8 vitest cases BM/BN delete enablement matrix |
| Runbook | §36 BUG-0022 operator smoke |

**Linked decisions:** DEC-0082 (active plan delete 409); DEC-0024 (selector contract); DEC-0074 (planning UX)  
**Research fulfilled:** R-0094  
**Deferred:** BM/BN live multi-plan browser smoke until **FRONTEND_DEPLOY**

---

## Known Issues

- Running container serves pre-Q0031 frontend — selector fix not live until rebuild
- Live `:18080` confirms pre-deploy baseline: 1 plan only; `/planning` **404**
- BM multi-plan delete flow requires operator env with 2+ plans for full live oracle

---

## Regression scope

- Crypto wealth EUR (BUG-0023) unchanged
- CategoryFilter static import (BUG-0021) unchanged
- Subscription list (BUG-0020) unchanged
- Backend **DEC-0082** 409 guard unchanged

---

## Rollback

```bash
git revert <Q0031-code-commits>
docker compose up -d --build flow-finance-ai
```

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0031 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0022-q0031`

## Milestone

**BUG-0022 released** — selector priority fix + planSelector vitest; BM/BN code oracles PASS; deploy operator-deferred per pass-with-prerequisites.
