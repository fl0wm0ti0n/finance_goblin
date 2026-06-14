# Q0031 Summary — BUG-0022 Plan delete selector regression

**Sprint:** Q0031 (`/quick`)  
**Bug:** BUG-0022  
**Orchestrator:** `auto-20260613-bug0022`  
**Phase:** execute **COMPLETE**  
**Date:** 2026-06-13

## Outcome

Fixed plan delete selector regression under **DEC-0082** / **DEC-0024** / **DEC-0074**:

- **BM1:** Inverted `activePlanId` resolution — `selectedPlanId ?? globalActiveId ?? firstPlanId` via pure helper `resolveDisplayedPlanId`.
- **T1:** Added `planSelector.ts` with `resolveDisplayedPlanId` + `isDeleteDisabled`; 8 vitest cases cover BM/BN delete enablement matrix.
- **G1:** `npm test` 17/17, `npm run build` PASS — frontend-only blast radius.

## Tasks

| ID | Status | Deliverable |
|----|--------|-------------|
| BM1 | done | `PlanningPage.tsx` selector priority fix |
| T1 | done | `planSelector.ts`, `planSelector.test.ts` |
| G1 | done | npm test + npm build PASS |
| V1 | deferred | Operator **FRONTEND_DEPLOY** required for runtime smoke |
| L1 | skipped | Optional P2 label rename |

## Test evidence

- `npm test` → **17/17** (+8 planSelector vs plan-verify baseline 9/9)
- `npm run build` → **PASS**

## Next phase

**`/qa`** — fresh subagent; V1 deferred to operator FRONTEND_DEPLOY.
