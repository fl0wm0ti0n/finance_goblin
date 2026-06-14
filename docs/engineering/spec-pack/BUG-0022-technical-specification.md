# Technical Specification — BUG-0022

## Overview

Fix frontend plan selector priority so operator dropdown selection drives viewing/editing/delete context. Extends **DEC-0082** §2 frontend contract — no backend or API shape change.

## Components

| Layer | Change | Gate |
|-------|--------|------|
| `frontend/src/pages/PlanningPage.tsx` | Invert `activePlanId` useMemo: `selectedPlanId ?? globalActive ?? first` | BM1 |
| `frontend/src/pages/planSelector.ts` | Pure helper `resolveDisplayedPlanId`, `isDeleteDisabled` (new) | T1 |
| `frontend/src/pages/planSelector.test.ts` | Vitest cases for selector + delete enablement | T1 |

## Selector contract (frozen)

```typescript
function resolveDisplayedPlanId(
  plans: PlanListItem[],
  selectedPlanId: string | null,
): string | null {
  const globalActive = plans.find((p) => p.is_active)?.id ?? null;
  return selectedPlanId ?? globalActive ?? plans[0]?.id ?? null;
}

function isDeleteDisabled(
  plans: PlanListItem[],
  displayedPlanId: string | null,
): boolean {
  if (!displayedPlanId) return true;
  return plans.find((p) => p.id === displayedPlanId)?.is_active === true;
}
```

## UI behavior

| Operator action | Expected |
|-----------------|----------|
| Dropdown selects non-active plan (global active exists) | Detail/compare/adjustments show selected plan; **Delete plan** enabled |
| Dropdown selects active plan | **Delete plan** disabled; tooltip *Set another plan active before deleting the active plan* |
| Confirm delete non-active | `DELETE /api/v1/plans/:id` → **204**; invalidate queries; clear `selectedPlanId` if deleted |
| Set active | Operates on displayed plan id |
| PVA tab | Unchanged — uses active plan endpoint, not dropdown id |

## API (unchanged)

- `DELETE /api/v1/plans/:id` non-active → **204**
- `DELETE /api/v1/plans/:id` active → **409** `{ "error": "active_plan_delete_forbidden", … }`

## Non-functional

- **Compatibility:** localhost `:18080`, omniflow external profile; OIDC smoke unchanged
- **Testing:** Vitest helper cases; existing `planningFeedback.test.ts` 409 path; `cargo test` plan delete unchanged
- **Deploy:** Frontend rebuild only — no migration

## Traceability

- [R-0094](docs/engineering/research.md#r-0094--bug-0022-plan-delete-selector-regression-activeplanid-ignores-dropdown)
- `docs/engineering/architecture.md` § **BUG-0022**
- **DEC-0082**, **DEC-0024**, **DEC-0074** (no new DEC)
