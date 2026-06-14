# Quick Release Notes — Q0033 / BUG-0024

**Quick task:** Q0033  
**Bug:** BUG-0024 — Plan delete sole-plan copy gap (live post-Q0031)  
**Date:** 2026-06-13  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0024 rows **BR**, **BS**; live BS browser operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `npm test` (31/31); `npm run build` PASS; `sprints/quick/Q0033/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/quick/Q0033/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS-WITH-PREREQUISITES — `sprints/quick/Q0033/uat.json`, `sprints/quick/Q0033/uat.md`, `sprints/quick/Q0033/verify-work-findings.md`; 5 steps — 3 pass, 2 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260613-bug0024-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Sole-plan delete guidance fix per **DEC-0082** (active plan delete guard unchanged). Frontend-only: when only one globally active plan exists, **Delete plan** stays disabled but inline copy explains create→activate→delete workflow. **Q0031** multi-plan selector regression preserved.

| Scope | Fix |
|-------|-----|
| **H1** | `frontend/src/pages/planSelector.ts` — `shouldShowSolePlanDeleteHint`, `SOLE_PLAN_DELETE_HINT` |
| **F1** | `frontend/src/pages/PlanningPage.tsx` — inline muted hint below **Delete plan** row |
| **T1** | `frontend/src/pages/planSelector.test.ts` — +7 vitest cases (15 total in suite; 31/31 npm) |
| **G1** | `npm test` 31/31; `npm run build` PASS — frontend-only blast radius |
| **V1** | verify-work BR/BS oracles — BR-UI + BR-API PASS; BS inline hint deferred **FRONTEND_DEPLOY** |

**Code proof:** planSelector 15/15; npm 31/31; npm build PASS; browser non-active delete enabled; DELETE active → 409 `active_plan_delete_forbidden`.

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
- `runtime_context_ref`: `docs/engineering/runbook.md` (§39 BUG-0024 hotfix)

**Operator gate — FRONTEND_DEPLOY (required before BS live UI smoke):**

Running container predates Q0033; sole-plan inline hint absent pre-deploy. Confirm tests pass before docker build:

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

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | External PostgreSQL — plan list for multi-plan smoke |
| `AUTHENTIK_SECRET_KEY` | External compose profile build gate (set dummy for local external profile) |
| OIDC provider config | Omniflow OIDC-1 regression — Compose/env only |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(BR)** | `/planning` with 2+ plans — select non-active | **Delete plan** enabled → confirm removes plan |
| **(BS)** | `/planning` with 1 sole active plan | Delete disabled + inline *To delete this plan, create another scenario, set it active, then delete this one.* |
| Regression | Active plan selected | Delete **disabled** + tooltip; API **409** |
| Regression | OIDC-1 omniflow `/planning` + `/api/v1/plans` | HTTP 200 after deploy |

**Automated (release):**

```bash
cd frontend && npm test && npm run build
```

**Live (operator):** UAT steps in `sprints/quick/Q0033/uat.json`.

---

## Credentials

- `DATABASE_URL` — external PostgreSQL (operator shell only — no inline secrets)
- OIDC provider config via Compose/env only

---

## Changes

| Area | Summary |
|------|---------|
| `frontend/src/pages/planSelector.ts` | H1 — sole-plan hint predicate + copy constant |
| `frontend/src/pages/PlanningPage.tsx` | F1 inline hint below Delete row when predicate true |
| `frontend/src/pages/planSelector.test.ts` | T1 +7 vitest cases for `shouldShowSolePlanDeleteHint` |
| Runbook | §39 BUG-0024 operator smoke |

**Linked decisions:** DEC-0082 (active plan delete guard)  
**Research fulfilled:** R-0096  
**Deferred:** BS live browser smoke until **FRONTEND_DEPLOY**

---

## Known Issues

- Running container serves pre-Q0033 frontend — sole-plan inline hint absent until rebuild
- Live `:18080` 2-plan env confirms BR pre-deploy; BS sole-plan fixture deferred deploy
- Omniflow **BR** full smoke recommended post-deploy if operator reported pre-Q0031 bundle

---

## Regression scope

- Plan delete selector (BUG-0022 / Q0031) unchanged
- Forecast Income card (BUG-0026 / Q0032) unchanged
- Crypto wealth EUR (BUG-0023) unchanged
- Backend DELETE 409 guard unchanged

---

## Rollback

```bash
git revert <Q0033-code-commits>
docker compose up -d --build flow-finance-ai
```

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0033 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0024-q0033`

## Milestone

**BUG-0024 released** — sole-plan inline delete guidance; BR code/browser PASS; BS code/vitest PASS; deploy operator-deferred per pass-with-prerequisites.
