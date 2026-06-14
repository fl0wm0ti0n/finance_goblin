# Q0033 Summary — BUG-0024 Plan delete sole-plan copy gap

**Sprint:** Q0033 (`/quick`)  
**Bug:** BUG-0024  
**Orchestrator:** `auto-20260613-bug0024`  
**Phase:** release **COMPLETE** — `bug0024-q0033`  
**Date:** 2026-06-13

## Goal

Close BUG-0024 on `/planning`: sole-plan operator sees permanently gray **Delete plan** with tooltip-only copy (**BS confirmed**). **Q0031** selector fix is **PASS** on localhost for multi-plan delete (**BR** / **H3 ruled out**). Add inline sole-plan guidance when delete is correctly disabled per **DEC-0082**. Omniflow **BR** verification deferred to **V1** after **FRONTEND_DEPLOY**.

## Tasks

| ID | Title | Status | Acceptance | Priority |
|----|-------|--------|------------|----------|
| H1 | Pure helper `shouldShowSolePlanDeleteHint` + copy constant | **done** | **BS** | P0 |
| F1 | PlanningPage inline hint wire | **done** | **BS** | P0 |
| T1 | Vitest sole-plan predicate cases | **done** | **BS** | P0 |
| G1 | Automated gate (`npm test`, `npm run build`) | **done** | **BR**, **BS** | P0 |
| V1 | verify-work `/planning` BR/BS + OIDC smoke | **deferred** | **BR**, **BS** | P0 |

**Task count:** 5 mandatory (5/12 under `SPRINT_MAX_TASKS=12`; no split).

## Deliverables

- `frontend/src/pages/planSelector.ts` — `SOLE_PLAN_DELETE_HINT`, `shouldShowSolePlanDeleteHint`
- `frontend/src/pages/PlanningPage.tsx` — inline muted hint below **Delete plan** when predicate true
- `frontend/src/pages/planSelector.test.ts` — +7 vitest cases (15 total in suite; 31/31 npm)

## Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **BS** | H1, F1, T1, G1, V1 | Sole active plan: disabled delete + inline create→activate→delete hint |
| **BR** | G1, V1 | Multi-plan non-active → delete enabled → plan removed; omniflow post-deploy |

## Frozen boundaries

- **GATE-SCOPE-1:** frontend-only — **DEC-0082** DELETE 409 unchanged
- **GATE-COPY-1:** inline hint below Delete row when sole active plan selected
- **GATE-DEC-1:** no new DEC
- **Out of scope:** backend delete API, sole-plan auto-deactivate, Playwright E2E

## Operator gate

**FRONTEND_DEPLOY** — frontend rebuild (Q0031/Q0032 bundles + this fix) before V1 runtime smoke.

## Next phase

**`/qa`** — fresh subagent (role: qa).
