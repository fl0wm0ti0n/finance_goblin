# CRS — BUG-0024 Plan delete still disabled (live post-Q0031)

## Purpose

Restore operator clarity on `/planning` when only one globally active plan exists: **Delete plan** stays disabled per **DEC-0082**, but inline copy explains how to delete (create another scenario, set it active, then delete the original). Multi-plan delete behavior from **BUG-0022** / Q0031 remains unchanged.

## Scope

### In scope

- **H1:** `planSelector.ts` — `shouldShowSolePlanDeleteHint`, `SOLE_PLAN_DELETE_HINT`
- **F1:** `PlanningPage.tsx` — conditional inline hint below plan-selector row
- **T1:** Vitest `shouldShowSolePlanDeleteHint` table-driven cases in `planSelector.test.ts`
- **G1:** `npm test` + `npm run build` gate
- **V1:** verify-work `/planning` smoke (**BR** post-deploy + **BS** sole-plan hint); OIDC smoke

### Out of scope

- `DELETE /api/v1/plans/:id`, **DEC-0082** 409 contract
- Create-plan API or CTA button from delete row
- Selector useMemo priority (**BUG-0022** shipped)
- Playwright suite

## Acceptance criteria ref

`docs/product/acceptance.md` — BUG-0024:

- **(BR)** With **two or more** plans on `/planning`, selecting a **non-active** plan enables **Delete plan** and successful confirmation removes the plan — not permanently disabled after **Q0031** deploy
- **(BS)** With **only one** globally active plan, **Delete plan** remains disabled per **DEC-0082** but UI shows **clear explanation** describing create→activate→delete workflow — not silent gray button; OIDC regression pass

## Dependencies

- **BUG-0022** / **Q0031** (`resolveDisplayedPlanId`, `isDeleteDisabled`)
- **DEC-0082** (active plan delete guard)
- [R-0096](docs/engineering/research.md#r-0096--bug-0024-plan-delete-still-disabled-live-post-q0031)
