# Design Concept — BUG-0022

## Summary

BUG-0022 fixes a post-**Q0022** frontend regression on `/planning`: the plan dropdown cannot switch viewing context when a global active plan exists, so **Delete plan** stays permanently disabled for non-active plans. Backend **DEC-0082** (409 on active delete) is intact — fix is selector priority in `PlanningPage.tsx` only.

## Goals

- **BM:** Selecting a non-active plan in the dropdown enables **Delete plan**; confirmation removes the plan and refreshes the list
- **BN:** Active plan delete remains blocked in UI (disabled + tooltip) and via API **409** per **DEC-0082**
- Vitest coverage for selector resolution and delete enablement logic
- OIDC-enabled deploy regression pass

## Non-goals

- Backend delete guard changes
- PVA tab endpoint or behavior changes
- Sole-plan delete policy (create second plan first — **DEC-0082** §Risks)
- New DEC record
- Grafana Dashboard 3 overlay changes

## Key decisions

| Gate | Choice | Rationale |
|------|--------|-----------|
| GATE-SEL-1 | Invert useMemo: `selectedPlanId ?? globalActiveId ?? first` | Minimal diff; operator selection wins |
| GATE-DEC82-1 | Frontend-only | Backend 409 + tooltip logic already correct |
| GATE-TEST-1 | Vitest pure helper | Unit-test BM/BN guard without RTL |
| GATE-SCOPE-1 | `/quick` Q0031 (4 tasks) | Single-file primary touch |
| GATE-LABEL-1 | Rename "Active plan" → "Plan" (P2 defer OK) | Reduces confusion after BM fix |
| GATE-DEC-1 | No new DEC | Extends **DEC-0082** frontend contract |

**Spec-pack traceability:** `docs/engineering/spec-pack/BUG-0022-crs.md`, `docs/engineering/spec-pack/BUG-0022-technical-specification.md`
