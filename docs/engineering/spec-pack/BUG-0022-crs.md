# CRS — BUG-0022 Plan delete selector regression

## Purpose

Restore operator ability to delete non-active plans from `/planning` when a global active plan exists. Post-**Q0022** AS1 shipped **DEC-0082** correctly on the backend; frontend `activePlanId` useMemo incorrectly prefers global `is_active` over dropdown `selectedPlanId`.

## Scope

### In scope

- **BM1:** Invert selector useMemo priority in `PlanningPage.tsx`
- **T1:** Vitest `resolveDisplayedPlanId` + `isDeleteDisabled` helper tests
- **G1:** `npm test` + `npm run build` gate
- **V1:** verify-work on `/planning` with 2+ plans; OIDC smoke per **BN**

### Out of scope

- Backend `DELETE /api/v1/plans/:id` guard
- PVA tab (`/api/v1/plans/active/plan-vs-actual`)
- Sole-plan delete UX policy change
- Dropdown label rename (P2 optional **L1**)

## Acceptance criteria ref

`docs/product/acceptance.md` — BUG-0022:

- **(BM)** With 2+ plans, selecting non-active in dropdown enables **Delete plan**; confirmation removes plan and refreshes list
- **(BN)** Active plan delete blocked in UI (disabled + tooltip) and via API **409** per **DEC-0082**; OIDC regression pass

## Dependencies

- **DEC-0082** (active delete guard — extends frontend contract)
- **DEC-0024** (single global active; Set active flow)
- **DEC-0074** (PVA `no_active_plan` — unaffected)
- [R-0094](docs/engineering/research.md#r-0094--bug-0022-plan-delete-selector-regression-activeplanid-ignores-dropdown)
