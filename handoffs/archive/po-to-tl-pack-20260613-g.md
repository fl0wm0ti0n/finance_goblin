# PO to TL archive pack (2026-06-13)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=650, PO_TO_TL_HOT_MAX_SECTIONS=60`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 8
- Retained units in hot file: 60
- First archived heading: `## Architecture summary`
- Last archived heading: `## Recommended next phase`
- Verification tuple (mandatory):
  - archived_body_lines=66
  - retained_body_lines=506

---

## Architecture summary

[R-0096 §1–9](docs/engineering/research.md#r-0096--bug-0024-plan-delete-still-disabled-live-post-q0031) gates formalized in architecture § BUG-0024. Five gates frozen; **GATE-DEC-1 closed — no new DEC**.

## Frozen gates (confirmed)

| Gate | Architecture verdict |
|------|---------------------|
| **GATE-COPY-1** | Inline `<p>` below plan-selector row when `shouldShowSolePlanDeleteHint(plans, activePlanIsSelected)`; copy `SOLE_PLAN_DELETE_HINT` |
| **GATE-DEPLOY-1** | Operator **FRONTEND_DEPLOY** then omniflow 2-plan `/planning` smoke; localhost **BR** PASS |
| **GATE-SCOPE-1** | Frontend-only; **DEC-0082** DELETE 409 unchanged; helper in `planSelector.ts` |
| **GATE-TEST-1** | Vitest `shouldShowSolePlanDeleteHint` in `planSelector.test.ts` |
| **GATE-DEC-1** | **No new DEC** — presentation layer on existing guard |

## Task tree (sprint-plan input)

| ID | Title | Row |
|----|-------|-----|
| H1 | `shouldShowSolePlanDeleteHint` + `SOLE_PLAN_DELETE_HINT` | **BS** |
| F1 | PlanningPage inline hint wire | **BS** |
| T1 | Vitest sole-plan predicate cases | **BS** |
| G1 | `npm test` + build | all |
| V1 | Post-**FRONTEND_DEPLOY** `/planning` smoke | **BR**, **BS** |

**Recommended sprint:** `/quick` **Q0033** — 5 tasks, 5/12 under `SPRINT_MAX_TASKS`.

## Recommended next phase

`/sprint-plan` — materialize Q0033; then `/plan-verify` → `/execute`.

---

# research-20260613-bug0024 — BUG-0024 Plan delete still disabled (live post-Q0031)

**From:** Tech Lead **To:** Tech Lead (architecture) **Bug:** BUG-0024 **Run:** `auto-20260613-bug0024`
**Date:** 2026-06-13 **Next phase:** `/architecture` (role: tech-lead)

## Research summary

[R-0096 §1–9](docs/engineering/research.md#r-0096--bug-0024-plan-delete-still-disabled-live-post-q0031) fulfilled. Localhost 2-plan probe confirms **Q0031** selector PASS (**H3 ruled out**). Operator *immer ausgegraut* on sole-plan env is **H1 copy gap (BS confirmed)**, not selector regression. Omniflow **BR** remains **OPEN** pending operator **FRONTEND_DEPLOY** (**H2 likely**).

## Frozen gates

| Gate | Research verdict |
|------|------------------|
| **GATE-COPY-1** | Inline hint below **Delete plan** row when `plans.length===1 && sole.is_active && activePlanIsSelected`; copy: *To delete this plan, create another scenario, set it active, then delete this one.* |
| **GATE-DEPLOY-1** | Operator **FRONTEND_DEPLOY** (Q0031/Q0032 bundles) then 2-plan `/planning` smoke on omniflow; localhost **BR** already PASS |
| **GATE-SCOPE-1** | Frontend-only; **DEC-0082** DELETE 409 unchanged; helper in `planSelector.ts` |
| **GATE-TEST-1** | Vitest `shouldShowSolePlanDeleteHint` predicate; Playwright not required |
| **GATE-DEC-1** | **No new DEC** — presentation layer on existing guard |

## Recommended execute shape

`/quick` — 2–4 tasks: `shouldShowSolePlanDeleteHint` + copy constant, PlanningPage inline hint wire, vitest, deferred V1 omniflow smoke.

## Recommended next phase

`/architecture` — formalize helper contract, acceptance trace **BR**/**BS**, size quick sprint.

---

# discovery-20260613-bug0024 — BUG-0024 Plan delete still disabled (live post-Q0031)

**From:** PO **To:** Tech Lead **Bug:** BUG-0024 **Run:** `auto-20260613-bug0024`
**Date:** 2026-06-13 **Next phase:** `/research` (role: tech-lead)

