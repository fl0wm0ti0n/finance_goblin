# Tasks — Q0031 (BUG-0022)

**Bug:** BUG-0022  
**Task count:** 4 mandatory + 1 optional P2 (4/12 under `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260613-bug0022-q0031`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **BM1** | Task **BM1** | Invert useMemo: `selectedPlanId ?? globalActive ?? firstPlan` |
| **T1** | Task **T1** | Pure helper + vitest delete enablement cases |
| **G1** | Task **G1** | `npm test`, `npm run build` |
| **BM/BN runtime** | Task **V1** | verify-work after FRONTEND_DEPLOY |
| **L1** | Task **L1** | P2 optional label rename |

## Execute order

```text
BM1 → T1 → G1
  → operator: FRONTEND_DEPLOY
  → V1 verify-work
  → optional L1
```

**Parallelism:** Sequential — T1 depends on BM1 contract; G1 blocked on T1; V1 blocked on G1 + deploy.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **BM** | BM1, T1, G1, V1 | Non-active plan selected → Delete enabled → confirm → plan removed |
| **BN** | BM1, T1, G1, V1 | Active plan selected → delete disabled + tooltip; API DELETE active → **409** |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| BM1 | Invert selector useMemo priority | 1h | open | **BM** | P0 |
| T1 | Vitest selector + delete enablement | 2h | open | **BM**, **BN** | P0 |
| G1 | Automated gate | 0.5h | open | **BM**, **BN** | P0 |
| V1 | verify-work `/planning` + OIDC | 1.5h | open | **BM**, **BN** | P0 |
| L1 | Dropdown label rename | 0.5h | open | UX | P2 optional |

---

## BM1 — Invert selector useMemo priority

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0022 **BM** — **DEC-0082** §2 frontend

### Description

In `frontend/src/pages/PlanningPage.tsx`:

1. Change `activePlanId` useMemo priority to:
   `selectedPlanId ?? plans.find(is_active)?.id ?? plans[0]?.id ?? null`
2. Keep dropdown controlled: `value={activePlanId}` + `onChange` → `setSelectedPlanId`.
3. Ensure `activePlanIsSelected = plan(activePlanId)?.is_active` reflects **displayed** plan.
4. Verify set-active banner `!activePlanIsSelected` when viewing non-active plan.

**Files:** `frontend/src/pages/PlanningPage.tsx` (L110–113 useMemo primary; L489, L643–683 consumers)

### Done when

- [ ] useMemo priority inverted per frozen selector contract
- [ ] Dropdown selection drives displayed plan when global active exists
- [ ] Delete control enabled when displayed plan is non-active
- [ ] No backend file changes

### Verification

Manual: with 2+ plans and global active, select non-active → header/detail reflect selection; Delete enabled.

---

## T1 — Vitest selector + delete enablement

**Status:** open  
**Depends on:** BM1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0022 **BM**, **BN** — **R-0094** GATE-TEST-1

### Description

Extract pure helpers (suggested: `frontend/src/pages/planSelector.ts`):

```typescript
resolveDisplayedPlanId(plans, selectedPlanId): string | null
isDeleteDisabled(plans, displayedPlanId): boolean
```

Add `planSelector.test.ts` covering:

| Case | Expected |
|------|----------|
| Selected non-active + global active exists | Displayed = selected; delete **enabled** |
| Selected null + global active exists | Displayed = global active; delete **disabled** |
| Selected null + no global active | Displayed = first plan |
| Empty plans | Displayed = null; delete disabled |
| Displayed plan `is_active === true` | `isDeleteDisabled === true` |

Wire BM1 useMemo to use helper (or mirror contract exactly).

**Files:** `frontend/src/pages/planSelector.ts`, `frontend/src/pages/planSelector.test.ts`

### Done when

- [ ] All vitest cases PASS
- [ ] `planningFeedback.test.ts` 409 path unchanged (verify only)
- [ ] Helper contract matches architecture frozen selector

### Verification

`npm test planSelector` → all PASS.

---

## G1 — Automated gate

**Status:** open  
**Depends on:** T1  
**Estimate:** 0.5h  
**Acceptance hook:** BUG-0022 **BM**, **BN** — automated verification

### Description

Run and record automated checks in `sprints/quick/Q0031/progress.md`:

1. `npm test` → PASS (includes planSelector + planningFeedback).
2. `npm run build` → PASS.
3. `git diff --stat` blast radius matches frozen file list (frontend only).

**Files:** `sprints/quick/Q0031/progress.md`

### Done when

- [ ] All automated checks PASS, recorded in progress.md
- [ ] No forbidden paths touched (backend plans API, PVA endpoint, Grafana)

### Verification

Test output pasted in progress.md; diff stat confirms scope.

---

## V1 — verify-work `/planning` + OIDC smoke

**Status:** open  
**Depends on:** G1 + operator FRONTEND_DEPLOY  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0022 **BM**, **BN**

### Description

Populate `sprints/quick/Q0031/uat.md` and `uat.json` after deploy on
localhost:18080 (and optional omniflow OIDC):

1. **BM-UI** — `/planning` with 2+ plans, one global active: select non-active → Delete enabled → confirm → plan removed; list refreshes.
2. **BM-API** — `DELETE /api/v1/plans/:id` non-active → **204**.
3. **BN-UI** — Select active plan → delete disabled + tooltip *Set another plan active before deleting the active plan*.
4. **BN-API** — `DELETE /api/v1/plans/:id` active → **409** `active_plan_delete_forbidden`.
5. **OIDC-1** — `/planning`, `/api/v1/plans` smoke on omniflow profile.

**Files:** `sprints/quick/Q0031/uat.md`, `sprints/quick/Q0031/uat.json`

### Done when

- [ ] Rows **BM**, **BN** probed per acceptance.md matrix
- [ ] Regression gates documented
- [ ] `uat.md` and `uat.json` populated with results

**Operator gate:** **FRONTEND_DEPLOY** — frontend rebuild only (no migration).

---

## L1 — Dropdown label rename (optional P2)

**Status:** open  
**Depends on:** V1 (or skip if capacity tight)  
**Estimate:** 0.5h  
**Acceptance hook:** GATE-LABEL-1 — UX clarity

### Description

Rename dropdown label **"Active plan"** → **"Plan"** at `PlanningPage.tsx` ~L641.
Reduces operator confusion when viewing non-active scenario.

**Files:** `frontend/src/pages/PlanningPage.tsx`

### Done when

- [ ] Label updated; no functional regression
- [ ] Optional — skip without blocking P0 closure

### Verification

Visual check on `/planning` after deploy.
