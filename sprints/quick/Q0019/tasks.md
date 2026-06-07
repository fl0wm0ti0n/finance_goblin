# Tasks — Q0019 (BUG-0011)

**Bug:** BUG-0011  
**Task count:** 11 (< `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260608-q0019-bug0011`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **AE1** | Task **AE1** | `monthly_overlay_delta_sum` via `build_overlay_deltas` |
| **AE2** | Task **AE2** | `version_metrics` + `project_adjustments_in_memory` |
| **AE3** | Task **AE3** | Zero overlay → 0.00; Leasing ~overlay unit tests |
| **AF1** | Task **AF1** | `PlanVsActualApiResponse` tagged enum; 200 `no_active_plan` |
| **AF2** | Task **AF2** | PVA guided empty state (mirror risk-score) |
| **AD1** | Task **AD1** | Template card grid + Create empty plan |
| **AD2** | Task **AD2** | Inline add/edit form → POST/PATCH adjustments |
| **AD3** | Task **AD3** | Custom Apply toast + query invalidation |
| **AD4** | Task **AD4** | Compare footnote + Set active banner |
| **T1** | Task **T1** | Compare + PVA integration tests |
| **V1** | Task **V1** | verify-work OIDC `/planning` smoke |

## Execute order

```text
AE1 → AE2 → AE3 → AF1 → AF2 → AD1 → AD2 → AD3 → AD4 → T1
→ deploy backend + frontend
→ V1 verify-work
```

**Sequencing:** AE-before-AF mandatory — AE1 helper before AF1 API freeze. AD after AF1 contract frozen.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **AD** | AD1–AD3, T1, V1 | Create empty plan + add-line UX; not silent no-op |
| **AE** | AE1–AE3, AD4, T1, V1 | Zero/neutral compare deltas; footnote for projected balance |
| **AF** | AF1–AF2, T1, V1 | PVA 200 JSON; guided tab when no active plan |
| Regression | T1, V1 | OIDC `/planning` three-tab smoke |

| ID | Title | Est. | Status | Acceptance |
|----|-------|------|--------|------------|
| AE1 | Overlay delta helper | 2h | open | **AE** |
| AE2 | Wire repository + service compare paths | 2h | open | **AE** |
| AE3 | Compare metric unit tests | 2h | open | **AE** |
| AF1 | Tagged PVA API 200 `no_active_plan` | 2h | open | **AF** |
| AF2 | PVA guided empty state | 2h | open | **AF** |
| AD1 | First-run Create empty plan | 2h | open | **AD** |
| AD2 | Inline add/edit adjustment form | 3h | open | **AD** |
| AD3 | Custom Apply toast + invalidation | 1h | open | **AD** |
| AD4 | Compare footnote + Set active banner | 1h | open | **AD**, **AE** |
| T1 | Compare + PVA integration tests | 2h | open | **AD/AE/AF** |
| V1 | verify-work OIDC `/planning` smoke | 1h | open | footer |

---

## AE1 — Overlay delta helper

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0011 **AE**

### Description

Add `monthly_overlay_delta_sum()` helper using existing `build_overlay_deltas` in `backend/src/plan/overlay.rs` (or `project.rs`). Sum overlay deltas from month start through `min(today, month_end)`. Empty adjustments → **0.00**.

**Files:** `backend/src/plan/overlay.rs`, `backend/src/plan/project.rs`

### Done when

- [ ] Helper exported and callable from repository/service
- [ ] Empty adjustments returns 0.00
- [ ] Does not alter `projected_month_end_balance` formula

---

## AE2 — Wire repository + service compare paths

**Status:** open  
**Depends on:** AE1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0011 **AE**

### Description

Replace full `planned_net` summation in `version_metrics` (`repository.rs`) and `project_adjustments_in_memory` (`service.rs`) with AE1 overlay helper for `monthly_delta_sum`. Leave projected month-end balance on full scenario path.

**Files:** `backend/src/plan/repository.rs`, `backend/src/plan/service.rs`

### Done when

- [ ] `/compare` endpoint returns overlay-only `monthly_delta_sum`
- [ ] Zero-adjustment plan returns 0.00 delta (not full forecast net)
- [ ] Non-empty Leasing plan returns ~overlay delta per DEC-0073 impact table

---

## AE3 — Compare metric unit tests

**Status:** open  
**Depends on:** AE2  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0011 **AE**

### Description

Unit tests: zero adjustments → `monthly_delta_sum` = 0.00; Leasing template ~-300/mo overlay; projected balance unchanged on empty overlay.

**Files:** `backend/src/plan/` (`#[cfg(test)]` or `backend/tests/`)

### Done when

- [ ] Zero-overlay fixture passes
- [ ] Leasing overlay fixture passes
- [ ] Architecture § Test strategy AE3 checks covered

---

## AF1 — Tagged PVA API 200 `no_active_plan`

**Status:** open  
**Depends on:** AE1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0011 **AF**

### Description

Introduce `PlanVsActualApiResponse` tagged enum in `backend/src/plan/types.rs` and route handler in `backend/src/api/plans.rs`. Mirror `RiskScoreApiResponse` pattern: HTTP **200** with `{ "status": "no_active_plan", "reason": "no_active_plan" }` when no active plan. Reject 404 via `plan_error_status` for this case.

**Files:** `backend/src/api/plans.rs`, `backend/src/plan/types.rs`, `backend/src/plan/service.rs`

### Done when

- [ ] No active plan returns 200 tagged JSON — not 404
- [ ] Active plan returns unchanged `ok` payload
- [ ] No auto-activate on create

---

## AF2 — PVA guided empty state

**Status:** open  
**Depends on:** AF1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0011 **AF**

### Description

Update `pvaQuery` in `PlanningPage.tsx`: `retry: false`; branch on `status === "no_active_plan"`. Render guided card with create plan + Set active CTA (mirror risk-score `no_score` pattern).

**Files:** `frontend/src/pages/PlanningPage.tsx`

### Done when

- [ ] Tab switch with no active plan shows guided UX — not blank/404
- [ ] Active plan renders existing chart/table
- [ ] User guide § Plan vs Actual aligned

---

## AD1 — First-run Create empty plan

**Status:** open  
**Depends on:** AF1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0011 **AD**

### Description

Replace Leasing-only first-run empty state with template card grid including **Create empty plan** (`POST /api/v1/plans` `{ name, template: "custom" }`). Wire navigation to new plan Scenarios tab.

**Files:** `frontend/src/pages/PlanningPage.tsx`

### Done when

- [ ] First visit offers Create empty plan — not Leasing-only
- [ ] POST creates plan and selects it for editing
- [ ] "Start empty and add lines" path functional

---

## AD2 — Inline add/edit adjustment form

**Status:** open  
**Depends on:** AD1  
**Estimate:** 3h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0011 **AD**

### Description

Add inline form above adjustments table: POST `.../adjustments` for new lines; PATCH for edits. Table rows editable (not read-only except delete).

**Files:** `frontend/src/pages/PlanningPage.tsx`

### Done when

- [ ] POST adjustment creates visible table row
- [ ] PATCH updates row fields
- [ ] Backend routes already exist — frontend wired

---

## AD3 — Custom Apply toast + invalidation

**Status:** open  
**Depends on:** AD2  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0011 **AD**

### Description

On Custom template Apply: toast **"Custom plan ready — add lines below"**; invalidate plan/version queries so table refreshes.

**Files:** `frontend/src/pages/PlanningPage.tsx`

### Done when

- [ ] Custom Apply no longer silent no-op
- [ ] Toast shown; queries invalidated
- [ ] Empty adjustments table visible after Apply

---

## AD4 — Compare footnote + Set active banner

**Status:** open  
**Depends on:** AD2  
**Estimate:** 1h  
**Acceptance hook:** BUG-0011 **AD**, **AE**

### Description

Compare tab: help footnote explaining projected balance may be negative from baseline forecast while delta is overlay-only. After first plan create: inline banner reminding operator to **Set active** for Plan vs Actual.

**Files:** `frontend/src/pages/PlanningPage.tsx`

### Done when

- [ ] Compare footnote present per user guide
- [ ] Set active banner after first create
- [ ] No auto-activate

---

## T1 — Compare + PVA integration tests

**Status:** open  
**Depends on:** AE3, AF1  
**Estimate:** 2h  
**Acceptance hook:** architecture § Test strategy

### Description

Integration tests: compare endpoint with zero/_nonempty adjustments; PVA route with and without active plan (200 both paths).

**Files:** `backend/tests/` or module `#[cfg(test)]`

### Done when

- [ ] Compare + PVA routes covered per architecture § Test strategy T1
- [ ] `cargo test` PASS (or documented DATABASE_URL prerequisite)

---

## V1 — verify-work OIDC `/planning` smoke

**Status:** open  
**Depends on:** AF2, AD4, T1, deploy  
**Estimate:** 1h  
**Acceptance hook:** BUG-0011 **AD**, **AE**, **AF** footer

### Description

Prepare `sprints/quick/Q0019/uat.md` smoke checklist. After backend + frontend deploy, probe `financegnome.omniflow.cc`:

1. **AD:** Create empty plan + add line
2. **AE:** Compare zero/neutral deltas on empty plan
3. **AF:** PVA guided state when no active plan
4. Regression: OIDC `/planning` three-tab smoke

**Files:** `sprints/quick/Q0019/uat.md`, `docs/user-guides/BUG-0011.md`

### Done when

- [ ] Row **AD**: empty create + add-line PASS
- [ ] Row **AE**: compare deltas PASS
- [ ] Row **AF**: PVA 200 + guided UX PASS
- [ ] Deploy regression footer PASS

**Operator gate:** Backend + frontend deploy after AE1–T1 + AF2/AD1–AD4 before V1 runtime probes.
