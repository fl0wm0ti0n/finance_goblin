# Q0033 — BUG-0024 Plan delete sole-plan copy gap (live post-Q0031)

| Field | Value |
|-------|-------|
| **ID** | Q0033 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0024 |
| **Created** | 2026-06-13 |
| **Architecture** | `docs/engineering/architecture.md` § BUG-0024 (extends **DEC-0082**, **BUG-0022** / **Q0031**; **GATE-DEC-1** no new DEC) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260613-bug0024-q0033`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0024 rows **BR**, **BS** |
| **Task count** | 5 mandatory (5/12 under `SPRINT_MAX_TASKS=12`) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0024 on `/planning`: operator *immer ausgegraut* on sole-plan env is **BS** copy gap (disabled delete with tooltip-only guidance), not a **Q0031** selector regression (**H3 ruled out** on localhost). Add inline sole-plan hint when delete is correctly disabled per **DEC-0082**. **BR** (multi-plan non-active delete enabled) already **PASS** on localhost; omniflow verification deferred to **V1** after operator **FRONTEND_DEPLOY** (**GATE-DEPLOY-1**).

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| BS — sole-plan inline guidance (P0) | H1, F1, T1 | `planSelector.ts`, `PlanningPage.tsx` |
| BR — multi-plan delete regression (P0 verify) | G1, V1 | existing selector + post-deploy smoke |
| Regression + gates | G1, V1 | `npm test`, `npm run build`, uat |

**Ops-only (not execute tasks):** Operator **FRONTEND_DEPLOY** (Q0031/Q0032 bundles + this fix).

**Out of scope:** Backend `DELETE /api/v1/plans/:id`; **DEC-0082** 409 contract; create-plan API; Playwright E2E; sole-plan auto-deactivate delete (**Option C** rejected).

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| H1 | Pure helper `shouldShowSolePlanDeleteHint` + copy constant | 1h | — | **BS** | P0 |
| F1 | PlanningPage inline hint wire | 1h | H1 | **BS** | P0 |
| T1 | Vitest sole-plan predicate cases | 1.5h | H1 | **BS** | P0 |
| G1 | Automated gate | 0.5h | F1, T1 | **BR**, **BS** | P0 |
| V1 | verify-work `/planning` BR/BS + OIDC smoke | 1.5h | G1 + deploy | **BR**, **BS** | P0 |

**Total estimate:** ~4.5h dev + ~1.5h operator V1.

## Deploy order

```text
H1 (planSelector.ts — shouldShowSolePlanDeleteHint + SOLE_PLAN_DELETE_HINT)
  → F1 (PlanningPage inline hint below Delete plan row)
  → T1 (planSelector.test.ts sole-plan cases)
  → G1 (npm test + build)
  → operator: FRONTEND_DEPLOY (frontend rebuild only)
  → V1 verify-work
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **BS** | H1, F1, T1, G1, V1 | Sole active plan: delete disabled + inline hint *To delete this plan, create another scenario, set it active, then delete this one.* — not tooltip-only silent gray |
| **BR** | G1, V1 | 2+ plans: select non-active → Delete enabled → confirm removes plan; localhost PASS pre-fix; omniflow after **FRONTEND_DEPLOY** |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| H1 | Task **H1** — **GATE-COPY-1** |
| F1 | Task **F1** — **GATE-COPY-1** placement |
| T1 | Task **T1** — **GATE-TEST-1** |
| G1 | Task **G1** |
| BR/BS runtime gates | Task **V1** — **GATE-DEPLOY-1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.

## User guide (USER_GUIDE_MODE=1)

**Waived** — UX copy fix under existing Planning delete workflow; no new operator workflow.
