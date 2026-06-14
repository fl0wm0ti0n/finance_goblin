# QA Findings — Quick Q0031 / BUG-0022

**Work item:** BUG-0022 (defect)  
**Quick task:** Q0031  
**QA phase:** `/qa`  
**Date:** 2026-06-13  
**Orchestrator:** `auto-20260613-bug0022`  
**Decisions:** DEC-0082, DEC-0024, DEC-0074  
**QA agent:** fresh subagent (`qa-20260613-bug0022-qa-fresh`)

## Verdict

**PASS** — Independent re-run confirms BM/BN implementation under frozen architecture gates **GATE-SEL-1** / **GATE-DEC82-1** / **GATE-TEST-1**: selector priority inverted to `selectedPlanId ?? globalActiveId ?? firstPlanId` via `resolveDisplayedPlanId`; delete guard uses `isDeleteDisabled` on displayed id; backend **DEC-0082** 409 guard unchanged. Automated gates **17/17** npm, **PASS** build; backend unit test **1/1** for 409 serialization; live API probe DELETE active → **409** `active_plan_delete_forbidden`. **V1** runtime smoke on `/planning` (BM/BN browser + OIDC) deferred to verify-work — pass-with-prerequisites.

**Blockers:** 0

## Scope

**Inputs reviewed:** `handoffs/dev_to_qa.md` (Q0031 top section), `sprints/quick/Q0031/{summary,progress,tasks,uat}.md`, `docs/product/acceptance.md` BUG-0022 row (BM/BN), `docs/engineering/architecture.md` § BUG-0022, `decisions/DEC-0082.md`, `frontend/src/pages/PlanningPage.tsx`, `frontend/src/pages/planSelector.ts`, `frontend/src/pages/planSelector.test.ts`, `frontend/src/pages/planningFeedback.test.ts`, `backend/src/api/plans.rs`, `backend/src/plan/service.rs`. No host `.env`/secret files read.

## Test plan

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | Frontend unit suite | `npm test` | **PASS** — 17/17 (2.04s) |
| T-2 | Frontend build | `npm run build` | **PASS** — tsc + vite build (12.45s) |
| T-3 | BM1 selector priority | Code review `planSelector.ts` + `PlanningPage.tsx` L111–114 | **PASS** — `resolveDisplayedPlanId(plans, selectedPlanId)` replaces inverted useMemo |
| T-4 | BM delete enablement | `planSelector.test.ts` + `PlanningPage.tsx` L490, L670–675 | **PASS** — non-active displayed → delete enabled; tooltip correct |
| T-5 | BN UI guard | `PlanningPage.tsx` L670–674 | **PASS** — active displayed → `disabled` + tooltip *Set another plan active before deleting the active plan* |
| T-6 | BN backend 409 unit | `cargo test --lib active_plan_delete` | **PASS** — 1/1 `active_plan_delete_returns_409_with_code` |
| T-7 | BN backend 409 live | `DELETE /api/v1/plans/:active_id` on `:18080` | **PASS** — HTTP 409, body `active_plan_delete_forbidden` |
| T-8 | T1 vitest matrix | `planSelector.test.ts` (8 cases) | **PASS** — selector priority + delete enablement per GATE-TEST-1 |
| T-9 | 409 feedback regression | `planningFeedback.test.ts` | **PASS** — 4/4; active delete message path unchanged |
| T-10 | Delete success cleanup | `PlanningPage.tsx` L375–384 | **PASS** — invalidates queries; clears `selectedPlanId` when deleted plan was selected |
| T-11 | Frozen boundary | git scope vs architecture § codebase map | **PASS** — frontend-only; no backend/PVA/Grafana edits |
| T-12 | V1 operator smoke | FRONTEND_DEPLOY + `/planning` BM/BN + OIDC | **DEFERRED** — verify-work |

### T-1 output

```
Test Files  5 passed (5)
Tests       17 passed (17)
  planSelector.test.ts       8 passed
  planningFeedback.test.ts   4 passed
  GoalStatsStrip.test.tsx    2 passed
  CategoryTrendChart.test.tsx 1 passed
  ChatPanel.test.tsx         2 passed
```

### T-7 output

```
{"error":"active_plan_delete_forbidden","message":"Cannot delete the active plan. Set another plan active first, then delete."}
HTTP:409
```

## Code review vs decisions

| Gate / Decision | Contract | Review |
|-----------------|----------|--------|
| **GATE-SEL-1** | `selectedPlanId ?? globalActiveId ?? firstPlanId` | **PASS** — `resolveDisplayedPlanId` L10–11; wired in `PlanningPage` useMemo |
| **GATE-DEC82-1** | No backend change; 409 + UI disabled on active | **PASS** — backend unchanged; UI uses `isDeleteDisabled`; live 409 confirmed |
| **GATE-TEST-1** | Vitest pure helpers + delete enablement cases | **PASS** — 8/8 cases cover architecture table |
| **GATE-SCOPE-1** | Frontend-only `/quick` blast radius | **PASS** — 3 frontend files + sprint artifacts |
| **GATE-LABEL-1** | Optional P2 label rename | **N/A** — L1 skipped (acceptable) |
| **DEC-0082** §2 | Delete disabled + tooltip on active; 409 on API | **PASS** — UI + API contract intact post-selector fix |
| **DEC-0024** | Set active on displayed plan | **PASS** — `activateMutation.mutate(activePlanId)` unchanged consumer |
| **DEC-0074** | PVA tab decoupled from dropdown | **PASS** — no PVA endpoint changes |

## Acceptance row status (qa-stage)

| Row | qa-stage evidence | Status |
|-----|-------------------|--------|
| **BM** | Selector priority fix + 8 vitest cases + delete button uses displayed id | **PASS** at qa — live 2+ plan delete flow deferred V1 |
| **BN** | `isDeleteDisabled` + tooltip + backend 409 unit + live probe | **PASS** at qa — OIDC browser regression deferred V1 |

## Task verdict matrix

| Task | Status (execute) | QA verdict | Notes |
|------|------------------|------------|-------|
| BM1 | done | **PASS** | `resolveDisplayedPlanId` wired; dropdown controlled on displayed id |
| T1 | done | **PASS** | 8/8 vitest; covers architecture case table |
| G1 | done | **PASS** | Independent 17/17 + build PASS |
| V1 | deferred | **DEFERRED** | FRONTEND_DEPLOY required |
| L1 | skipped | **N/A** | Optional P2 label rename |

## Non-blocking notes (carry to verify-work)

- Live environment has **1 plan** only — BM multi-plan delete flow requires operator to create 2+ plans at V1.
- Dropdown label still reads **"Active plan"** (L1 skipped per GATE-LABEL-1 P2 defer).
- V1 requires operator **FRONTEND_DEPLOY** (frontend rebuild only; no migration) before `/planning` smoke.
- Implementation files remain uncommitted per sprint policy.

## Handoff

- **Next phase:** `/verify-work` (role: qa)
- **No return items** — `handoffs/qa_to_dev.md` unchanged (PASS; 0 blockers)

`fresh_context_marker`: qa-20260613-bug0022-qa-fresh  
`runtime_proof_id`: runtime-proof-qa-20260613-bug0022-001  
`phase_boundary`: qa → verify-work
