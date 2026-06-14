# Technical Specification — BUG-0024

## Overview

Frontend-only UX fix: add a pure predicate and copy constant to `planSelector.ts`, wire an inline helper paragraph on `PlanningPage` when the sole globally active plan is selected and delete is disabled. Extends **DEC-0082** deactivate-first semantics — no backend or DELETE contract change.

## Components

| Layer | Change | Gate |
|-------|--------|------|
| `frontend/src/pages/planSelector.ts` | `SOLE_PLAN_DELETE_HINT`, `shouldShowSolePlanDeleteHint` | H1 |
| `frontend/src/pages/PlanningPage.tsx` | Conditional inline `<p>` below delete row | F1 |
| `frontend/src/pages/planSelector.test.ts` | Vitest predicate cases | T1 |

## Sole-plan hint contract (frozen)

```typescript
export const SOLE_PLAN_DELETE_HINT =
  "To delete this plan, create another scenario, set it active, then delete this one.";

export function shouldShowSolePlanDeleteHint(
  plans: PlanListItem[] | undefined,
  activePlanIsSelected: boolean,
): boolean {
  if (!plans || plans.length !== 1) return false;
  return plans[0].is_active === true && activePlanIsSelected === true;
}
```

## UI behavior

| Operator view | Expected |
|---------------|----------|
| 1 sole active plan, delete disabled | Inline hint visible below **Delete plan** row |
| 2+ plans, non-active selected | Delete **enabled**; **no** inline hint |
| 2+ plans, active selected | Delete disabled + existing tooltip; **no** inline hint |
| Empty plans | No hint (delete row hidden) |

**Helper text styling:** Reuse PlanningPage muted pattern — `fontSize: "0.85rem"`, `color: "#64748b"`, `margin: "0.5rem 0 0"`.

## API (unchanged)

- `GET /api/v1/plans` — plan list with `is_active`
- `DELETE /api/v1/plans/:id` — **409** when deleting globally active plan (**DEC-0082**)

## Non-functional

- **Compatibility:** localhost `:18080`, omniflow external profile; OIDC smoke unchanged
- **Testing:** Vitest helper cases; `npm test` frontend; no backend test changes
- **Deploy:** Frontend rebuild only — no migration; **FRONTEND_DEPLOY** required for omniflow **BR**

## Traceability

- [R-0096](docs/engineering/research.md#r-0096--bug-0024-plan-delete-still-disabled-live-post-q0031)
- `docs/engineering/architecture.md` § **BUG-0024**
- **DEC-0082**, **BUG-0022** / Q0031 (no new DEC)
