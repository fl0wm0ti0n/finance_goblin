# PO to TL archive pack (2026-06-13)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 20
- Retained units in hot file: 40
- First archived heading: `## Architecture summary`
- Last archived heading: `## Recommended next phase`
- Verification tuple (mandatory):
  - archived_body_lines=158
  - retained_body_lines=357

---

## Architecture summary

Frontend-only fix in `PlanningPage.tsx`: invert selector useMemo so **operator dropdown selection wins** over global active plan id. Backend **DEC-0082** 409 guard unchanged. Six gates frozen; **GATE-DEC-1 closed** (extends **DEC-0082**, no new DEC).

## Frozen selector contract

```text
displayedPlanId = selectedPlanId ?? globalActiveId ?? firstPlanId
deleteDisabled    = plan(displayedPlanId)?.is_active === true
```

PVA tab remains decoupled — `/api/v1/plans/active/plan-vs-actual`.

## Architecture gates (resolved)

| Gate | Decision |
|------|----------|
| **GATE-SEL-1** | Option A — invert useMemo priority (`selectedPlanId` first) |
| **GATE-DEC82-1** | No backend change; preserve 409 + tooltip |
| **GATE-TEST-1** | Vitest `resolveDisplayedPlanId` + delete enablement |
| **GATE-SCOPE-1** | `/quick` **Q0031** — 4 mandatory tasks (+ optional P2 label) |
| **GATE-LABEL-1** | Rename **"Active plan"** → **"Plan"** — P2 defer OK (**L1**) |
| **GATE-DEC-1** | No new DEC — extends **DEC-0082** frontend contract |

## Recommended `/quick` task tree (Q0031)

| ID | Task | Row |
|----|------|-----|
| **BM1** | Invert selector useMemo in `PlanningPage.tsx` | **BM** |
| **T1** | Vitest helper + delete enablement cases | **BM**, **BN** |
| **G1** | `npm test` + `npm run build` | all |
| **V1** | verify-work `/planning` 2+ plans + OIDC smoke | **BM**, **BN** |
| **L1** | Dropdown label rename (optional P2) | UX |

## Artifacts

- `docs/engineering/architecture.md` § **BUG-0022**
- `docs/engineering/spec-pack/BUG-0022-{design-concept,crs,technical-specification}.md`
- `docs/engineering/state.md` architecture checkpoint

## Recommended next phase

`/sprint-plan` (role: tech-lead) — materialize **Q0031** from architecture task table.

---

# research-20260613-bug0022 — BUG-0022 Plan delete selector regression

**From:** Tech Lead **To:** Tech Lead (architecture phase) **Bug:** BUG-0022 **Run:** `auto-20260613-bug0022`
**Date:** 2026-06-13 **Next phase:** `/architecture` (role: tech-lead)
**Discovery ref:** `discovery-20260613-bug0022` below
**Fresh context marker:** `research-20260613-bug0022-tl-fresh`

## Research summary

Post-**Q0022** AS1 frontend regression confirmed: `activePlanId` useMemo prefers global `is_active` plan over `selectedPlanId`, so the dropdown cannot switch viewing context and **Delete plan** stays disabled whenever a global active plan exists. Backend **DEC-0082** 409 guard intact (live DELETE non-active **204**, active **409**). Fix is **frontend-only** in `PlanningPage.tsx`.

## Selector contract (provisional — freeze at architecture)

| Concept | Contract |
|---------|----------|
| **Displayed plan** | `selectedPlanId ?? globalActiveId ?? firstPlanId` |
| **Delete enabled** | Displayed plan exists and `is_active === false` |
| **Delete disabled** | Displayed plan is globally active (tooltip per DEC-0082) |
| **PVA tab** | Unchanged — `/api/v1/plans/active/plan-vs-actual` (not tied to dropdown) |
| **Set active** | Operates on displayed plan id |

**Rejected:** useEffect sync of `selectedPlanId` from global active (dual source of truth); uncontrolled dropdown.

## Architecture gates (mandatory)

| Gate | Question | TL default |
|------|----------|------------|
| **GATE-SEL-1** | Selector priority | Option A — invert useMemo: `selectedPlanId` first |
| **GATE-DEC82-1** | Backend change? | **No** — preserve 409 + UI guard |
| **GATE-TEST-1** | Coverage | Vitest helper: selector resolution + delete enablement (BM/BN logic) |
| **GATE-SCOPE-1** | Sprint shape | `/quick` — 2–4 tasks; `PlanningPage.tsx` primary |
| **GATE-LABEL-1** | Dropdown label | Rename **"Active plan"** → **"Plan"** or **"Selected plan"** (P2 defer OK) |
| **GATE-DEC-1** | New DEC? | **No** — extends DEC-0082 frontend contract |

## Must-not-break (confirmed)

- **DEC-0082** — active DELETE **409** `active_plan_delete_forbidden`
- **DEC-0024** — single global active; Set active flow
- **DEC-0074** — PVA `no_active_plan` handling
- **deletePlanMutation** — invalidate queries; clear `selectedPlanId` on deleted id

## Test / regression plan

- New vitest: `resolveDisplayedPlanId` (or equivalent) — non-active selection wins; delete enabled iff not globally active
- Existing: `planningFeedback.test.ts` 409 path; `active_plan_delete_returns_409_with_code`
- verify-work: BM/BN on `/planning` with 2+ plans; OIDC smoke per BN

## Research artifact

[R-0094](docs/engineering/research.md#r-0094--bug-0022-plan-delete-selector-regression-activeplanid-ignores-dropdown)

## Recommended next phase

`/architecture` (role: tech-lead) — freeze GATE-SEL-1 / GATE-LABEL-1, document selector contract in architecture.md, materialize `/quick` task tree.

---

# discovery-20260613-bug0022 — BUG-0022 Plan delete still broken (selector ignores dropdown)

**From:** PO **To:** Tech Lead **Bug:** BUG-0022 **Run:** `auto-20260613-bug0022`
**Date:** 2026-06-13 **Next phase:** `/research` (role: tech-lead)
**Intake evidence:** `handoffs/intake_evidence/intake-20260611-plan-delete-regression.json` (read-only)
**Fresh context marker:** `discovery-20260613-bug0022-po-fresh`

## Discovery summary

Post-**BUG-0014** AS1 / **Q0022** regression: operator cannot delete non-active plans from `/planning` when a global active plan exists. Code audit confirms intake hypothesis — `activePlanId` `useMemo` prefers `is_active` plan over `selectedPlanId`, so dropdown changes are ignored and **Delete plan** stays disabled. Backend **DEC-0082** 409 guard is intact; defect is frontend selector state only.

## Sub-defect confirmation (BM / BN)

| AC | Verdict | Key evidence |
|----|---------|--------------|
| **BM** | **CONFIRMED** | `PlanningPage.tsx` L110–113: `active?.id ?? selectedPlanId` — global active always wins. L643–647: dropdown `value={activePlanId}` + `onChange` sets `selectedPlanId` only. L489 + L669: `activePlanIsSelected` derived from `activePlanId` → delete **disabled** whenever active plan exists regardless of dropdown. Live API: created temp non-active plan, `DELETE` returned **204** (backend delete path works); UI/browser automation showed empty SPA shell — BM visual operator-deferred; code + API sufficient. |
| **BN** | **CONFIRMED (backend + guard logic); UI masked by BM** | `DELETE /api/v1/plans/:id` on active plan → **409** `{ "error": "active_plan_delete_forbidden", … }` (`plans.rs` L234–239; live probe 2026-06-13). Frontend L669–673: `disabled={activePlanIsSelected}` + tooltip *Set another plan active before deleting the active plan* — correct per **DEC-0082**, but `activePlanId` never reflects dropdown selection of active vs non-active until BM fixed. Unit test `active_plan_delete_returns_409_with_code` + `planningFeedback.test.ts` 409 message path present. |

## Root-cause chain (code + live)

| Layer | Finding |
|-------|---------|
| **Selector state** | `selectedPlanId` updated on dropdown change but not consumed when `is_active` plan exists |
| **Derived selection** | `activePlanIsSelected` checks `is_active` on plan matching `activePlanId` — always true for global active |
| **Delete mutation** | `deletePlanMutation` + confirm modal wired correctly; blocked before click by `disabled` |
| **Backend guard** | `service.rs` L268–274 `ActivePlanDeleteForbidden` → **409** — no regression |
| **Single-plan edge** | One active plan only → delete permanently disabled with no UI workaround (acceptable per **DEC-0082** §Risks; copy may need visibility) |

## Acceptance rows (unchanged)

- **(BM)** With 2+ plans, selecting non-active in dropdown enables **Delete plan**; confirmation removes plan and refreshes list
- **(BN)** Active plan delete blocked in UI (disabled + tooltip) and via API **409** per **DEC-0082**; OIDC regression pass

## Research questions (carry to `/research`)

1. **Selector contract:** Should `activePlanId` become `selectedPlanId ?? globalActiveId ?? first`, or split `displayedPlanId` vs `globalActivePlanId` for compare/PVA vs header dropdown?
2. **Dropdown label:** Rename *Active plan* → *Selected plan* when operator views non-active scenario?
3. **Test gap:** No `PlanningPage` selector/delete enablement test — add vitest or integration coverage in execute phase?
4. **DEC-0082 preservation:** Confirm no backend change; frontend-only fix scope?

## Must-not-break boundaries

| Contract | Boundary |
|----------|----------|
| **DEC-0082** | Active plan `DELETE` stays **409** `active_plan_delete_forbidden` |
| **DEC-0024** | Single global active plan; Set active flow unchanged |
| **DEC-0074** | `no_active_plan` PVA handling unchanged |
| **deletePlanMutation** | Invalidate `plans` / `plan-detail` / `plan-vs-actual`; clear `selectedPlanId` on deleted id |

## Recommended next phase

`/research` (role: tech-lead) — freeze selector contract, confirm single-file fix scope in `PlanningPage.tsx`, size `/quick` vs sprint.

---

