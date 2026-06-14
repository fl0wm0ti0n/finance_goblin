# Tasks — Q0033 (BUG-0024)

**Bug:** BUG-0024  
**Task count:** 5 mandatory (5/12 under `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260613-bug0024-q0033`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **H1** | Task **H1** | `shouldShowSolePlanDeleteHint` + `SOLE_PLAN_DELETE_HINT` — **GATE-COPY-1** |
| **F1** | Task **F1** | PlanningPage inline hint below Delete plan row |
| **T1** | Task **T1** | Vitest table-driven predicate cases — **GATE-TEST-1** |
| **G1** | Task **G1** | `npm test`, `npm run build` |
| **BR/BS runtime** | Task **V1** | verify-work after **FRONTEND_DEPLOY** — **GATE-DEPLOY-1** |

## Execute order

```text
H1 → F1 → T1 → G1
  → operator: FRONTEND_DEPLOY
  → V1 verify-work
```

**Parallelism:** F1 and T1 both depend on H1; G1 blocked on F1 + T1; V1 blocked on G1 + deploy.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **BS** | H1, F1, T1, G1, V1 | Sole active plan: disabled delete + inline hint with create→activate→delete guidance |
| **BR** | G1, V1 | Multi-plan non-active selection enables delete; confirm removes plan; OIDC smoke |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| H1 | Pure helper sole-plan delete hint | 1h | open | **BS** | P0 |
| F1 | PlanningPage inline hint wire | 1h | open | **BS** | P0 |
| T1 | Vitest sole-plan predicate cases | 1.5h | open | **BS** | P0 |
| G1 | Automated gate | 0.5h | open | **BR**, **BS** | P0 |
| V1 | verify-work `/planning` + OIDC | 1.5h | open | **BR**, **BS** | P0 |

---

## H1 — Pure helper sole-plan delete hint

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0024 **BS** — **GATE-COPY-1**

### Description

Extend `frontend/src/pages/planSelector.ts` with frozen contract:

```typescript
export const SOLE_PLAN_DELETE_HINT =
  "To delete this plan, create another scenario, set it active, then delete this one.";

export function shouldShowSolePlanDeleteHint(
  plans: PlanSummary[] | undefined,
  activePlanIsSelected: boolean,
): boolean {
  return (
    (plans?.length ?? 0) === 1 &&
    plans![0].is_active === true &&
    activePlanIsSelected === true
  );
}
```

Mirror **BUG-0022** / **BUG-0026** colocated pure-helper pattern alongside existing `resolveDisplayedPlanId` / `isDeleteDisabled`.

**Files:** `frontend/src/pages/planSelector.ts`

### Done when

- [ ] Both exports implemented per frozen predicate
- [ ] Copy constant matches R-0096 §3 frozen text exactly
- [ ] No backend file changes

### Verification

Typecheck + importable from PlanningPage.

---

## F1 — PlanningPage inline hint wire

**Status:** open  
**Depends on:** H1  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0024 **BS** — **GATE-COPY-1** placement

### Description

In `frontend/src/pages/PlanningPage.tsx`:

1. Import `shouldShowSolePlanDeleteHint` and `SOLE_PLAN_DELETE_HINT` from `planSelector.ts`
2. When predicate true, render inline help text **immediately below** the **Delete plan** button within the plan-selector row
3. Use existing muted helper class (e.g. `help-text` or page-consistent muted `<p>`)
4. **Do not** replace tooltip on multi-plan active selection — existing *Set another plan active…* tooltip unchanged when `plans.length >= 2`
5. Delete button disabled state unchanged — still gated by `activePlanIsSelected` / **DEC-0082**

**Files:** `frontend/src/pages/PlanningPage.tsx` (L667–675 delete row vicinity)

### Done when

- [ ] Inline hint visible on sole active plan when delete disabled
- [ ] Hint hidden when 2+ plans or non-active plan selected
- [ ] Keyboard/screen-reader discoverable (not tooltip-only)
- [ ] No backend or DELETE contract changes

### Verification

Manual localhost: 1 plan active → hint visible; 2 plans non-active selected → hint hidden, delete enabled.

---

## T1 — Vitest sole-plan predicate cases

**Status:** open  
**Depends on:** H1  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0024 **BS** — **GATE-TEST-1**, **R-0096** §6

### Description

Extend `frontend/src/pages/planSelector.test.ts` with table-driven cases:

| Case | Expected |
|------|----------|
| Sole plan active + `activePlanIsSelected=true` | `shouldShowSolePlanDeleteHint` → **true** |
| Sole plan active + `activePlanIsSelected=false` | **false** (guard) |
| Two plans, active selected | **false** (multi-plan uses tooltip only) |
| Two plans, non-active selected | **false** (delete enabled; no hint) |
| Empty plans | **false** |
| `SOLE_PLAN_DELETE_HINT` | Contains *create another scenario* |

**Regression:** existing 8/8 `resolveDisplayedPlanId` / `isDeleteDisabled` cases remain green.

**Files:** `frontend/src/pages/planSelector.test.ts`

### Done when

- [ ] All new vitest cases PASS
- [ ] Existing planSelector cases PASS
- [ ] `planningFeedback.test.ts` 409 path unchanged (verify only)

### Verification

`npm test planSelector` → all PASS.

---

## G1 — Automated gate

**Status:** open  
**Depends on:** F1, T1  
**Estimate:** 0.5h  
**Acceptance hook:** BUG-0024 **BR**, **BS** — automated verification

### Description

Run and record automated checks in `sprints/quick/Q0033/progress.md`:

1. `npm test` → PASS (includes new sole-plan cases + existing suites).
2. `npm run build` → PASS.
3. `git diff --stat` blast radius matches frozen file list (frontend only).

**Files:** `sprints/quick/Q0033/progress.md`

### Done when

- [ ] All automated checks PASS, recorded in progress.md
- [ ] No forbidden paths touched (backend plans API, DELETE handler)

### Verification

Test output pasted in progress.md; diff stat confirms scope.

---

## V1 — verify-work `/planning` BR/BS + OIDC smoke

**Status:** open  
**Depends on:** G1 + operator FRONTEND_DEPLOY  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0024 **BR**, **BS**

### Description

Populate `sprints/quick/Q0033/uat.md` and `uat.json` after deploy on
localhost:18080 (and optional omniflow OIDC):

1. **BS-UI** — `/planning` with **1** sole active plan: delete disabled + **inline hint** visible with create→activate→delete copy.
2. **BR-UI** — `/planning` with **2+** plans: select non-active → **Delete plan** enabled → confirm removes plan.
3. **BR-API** — `DELETE` on active plan → **409** `active_plan_delete_forbidden` (**DEC-0082** regression).
4. **BN-regression** — Select globally active plan → delete disabled + tooltip (unchanged from **Q0031**).
5. **OIDC-1** — `/planning`, `/api/v1/plans` smoke on omniflow profile.

**Files:** `sprints/quick/Q0033/uat.md`, `sprints/quick/Q0033/uat.json`

### Done when

- [ ] Rows **BR**, **BS** probed per acceptance.md matrix
- [ ] Regression gates documented
- [ ] `uat.md` and `uat.json` populated with results

**Operator gate:** **FRONTEND_DEPLOY** — frontend rebuild (Q0031/Q0032 + this fix).
