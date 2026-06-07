# Tasks — Sprint S0015

**Story:** US-0014  
**Task count:** 8 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Slice | Acceptance refs |
|----|-------|--------|-------|-----------------|
| T-0155 | AC-1 empty-state onboarding regression verify | open | S1 | AC-1 |
| T-0156 | Set-active banner Dashboard 3 copy | open | S1 | AC-6 |
| T-0157 | Create/template success confirmations | open | S1 | AC-5 |
| T-0158 | planningFeedback helper module | open | S2 | AC-7 |
| T-0159 | onError on all 7 planning mutations | open | S2 | AC-7 |
| T-0160 | addAdjustment success + plan-vs-actual invalidation | open | S2 | AC-2 |
| T-0161 | Compare/PVA shipped UX verify | open | S3 | AC-3, AC-4 |
| T-0162 | User guide US-0014 finalize + UAT OIDC template | open | S3 | AC-8 |

---

## T-0155 — AC-1 empty-state onboarding regression verify

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0077 (verify-only)  
**Architecture slice:** US-0014-S1  
**Research:** R-0070, R-0072

### Description

Verify-first audit of Q0019-shipped empty-state onboarding — **do not rewrite** existing paths.

| Check | Contract |
|-------|----------|
| Empty branch | `plans.length === 0` shows template `card-grid` |
| Templates | Current, Leasing, Savings mode, House purchase, Custom |
| Primary CTA | **Create empty plan** visible and wired |
| Name field | Plan name input present |
| Reach add-lines | Create empty plan → editable plan with inline add form |

Document regression checklist in task completion note or `sprints/S0015/uat.md`.

### Done when

- [ ] Code audit confirms empty-state grid + Create empty plan (no functional regression)
- [ ] Six template cards present per R-0070 §4
- [ ] No rewrite of shipped Q0019 onboarding paths
- [ ] Regression steps documented for UAT

---

## T-0156 — Set-active banner Dashboard 3 copy

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0077, DEC-0024  
**Architecture slice:** US-0014-S1

### Description

Extend existing yellow `showSetActiveBanner` copy in `PlanningPage.tsx`:

| Element | Contract |
|---------|----------|
| Plan vs Actual | Existing — active plan required for PVA data |
| Grafana | Add cue: Dashboard 3 **Budgets** (`uid=budgets`) reflects the **active** plan |
| Trigger | After `createPlanMutation` success when `is_active=false` |
| Dismiss | Clears on successful `activateMutation` (existing) |

Reference `grafana/provisioning/dashboards/analytics/budgets.json` for copy only — no SQL edit.

### Done when

- [ ] Banner mentions Plan vs Actual tab requirement
- [ ] Banner mentions Grafana Dashboard 3 (Budgets) active-plan requirement
- [ ] Trigger/dismiss behavior unchanged from Q0019
- [ ] No auto-activate introduced (DEC-0074 frozen)

---

## T-0157 — Create/template success confirmations

**Status:** open  
**Depends on:** T-0158  
**Decisions:** DEC-0077  
**Architecture slice:** US-0014-S1  
**Research:** R-0073

### Description

Wire success confirmations via `showPlanningFeedback` (DEC-0077 success variant — green `#ecfdf5`, 4s auto-dismiss):

| Path | Message pattern |
|------|-----------------|
| Create empty plan | `Plan "{name}" created` |
| Create from template | `Plan "{name}" created from {template label}` |
| Apply template (existing plan) | `Template applied` or template-specific |

Extend beyond Q0019 Custom-only apply toast — **all** `applyTemplateMutation` success paths.

Invalidate `plan-vs-actual` on `createPlanMutation` success per DEC-0077 §4.

### Done when

- [ ] Empty-state create shows success card
- [ ] Create from template shows success card
- [ ] All template apply paths show confirmation (not Custom-only)
- [ ] `plan-vs-actual` invalidated on createPlan success
- [ ] Uses DEC-0077 helper (not ad-hoc duplicate UI)

---

## T-0158 — planningFeedback helper module

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0077  
**Architecture slice:** US-0014-S2  
**Research:** R-0073

### Description

Add page-local feedback helper — `frontend/src/pages/planningFeedback.ts` (or co-located in `PlanningPage.tsx`):

```typescript
showPlanningFeedback({ kind: 'success' | 'error', message: string })
formatPlanningError(err: unknown, fallback: string): string
```

| Variant | Background | Dismiss |
|---------|------------|---------|
| success | `#ecfdf5` | Auto 4s |
| error | `#fef2f2` | Manual Dismiss button |

**Frozen:**

- Single active feedback slot — new message replaces prior
- Message extraction from `apiFetch` `Error.message`; truncate 240 chars; fallback `"Request failed"`
- Prepends mutation label when body empty: `"Could not add adjustment: {detail}"`
- **Rejected:** Global `MutationCache`, toast library (R-0073)

Wire helper into `PlanningPage.tsx` top-of-page card region (match `SubscriptionsPage.tsx` inline pattern).

### Done when

- [ ] Helper module exists with `showPlanningFeedback` + `formatPlanningError`
- [ ] Success and error card variants render at page top
- [ ] Single-slot replace behavior on rapid messages
- [ ] Error messages truncate at 240 chars with fallback
- [ ] No new npm toast dependency

---

## T-0159 — onError on all 7 planning mutations

**Status:** open  
**Depends on:** T-0158  
**Decisions:** DEC-0077  
**Architecture slice:** US-0014-S2  
**Research:** R-0073

### Description

Wire mandatory `onError` on every planning mutation in `PlanningPage.tsx`:

| Mutation | Endpoint | Default error label |
|----------|----------|---------------------|
| `createPlanMutation` | `POST /api/v1/plans` | "Could not create plan" |
| `activateMutation` | `POST /api/v1/plans/{id}/activate` | "Could not set active plan" |
| `applyTemplateMutation` | `POST .../apply-template` | "Could not apply template" |
| `createVersionMutation` | `POST .../versions` | "Could not create version" |
| `addAdjustmentMutation` | `POST .../adjustments` | "Could not add adjustment" |
| `updateAdjustmentMutation` | `PATCH .../adjustments/{id}` | "Could not update adjustment" |
| `deleteAdjustmentMutation` | `DELETE .../adjustments/{id}` | "Could not delete adjustment" |

Pattern:

```typescript
onError: (err) => showPlanningFeedback({
  kind: 'error',
  message: formatPlanningError(err, '<label>')
})
```

Also wire success toast on `activateMutation`: `"Plan set as active"` (clears set-active banner).

Invalidate `plan-vs-actual` on `activateMutation` success (immediate — do not wait for recompute badge).

### Done when

- [ ] All 7 mutations have `onError` handlers
- [ ] Forced 4xx/5xx shows red error card (not console-only)
- [ ] `activateMutation` success toast + banner clear
- [ ] `plan-vs-actual` invalidated on activate success
- [ ] No per-mutation duplicate error parsing bodies

---

## T-0160 — addAdjustment success + plan-vs-actual invalidation

**Status:** open  
**Depends on:** T-0158, T-0159  
**Decisions:** DEC-0077  
**Architecture slice:** US-0014-S2

### Description

Complete AC-2 add-lines polish — success feedback + query invalidation matrix:

| Mutation | Success toast | Invalidate |
|----------|----------------|------------|
| `addAdjustmentMutation` | `"Adjustment added"` (required) | `plan-version`, `plan-compare`, **`plan-vs-actual`** |
| `updateAdjustmentMutation` | optional `"Adjustment updated"` | `plan-version`, `plan-compare`, **`plan-vs-actual`** |
| `deleteAdjustmentMutation` | optional `"Adjustment removed"` | `plan-version`, `plan-compare`, **`plan-vs-actual`** |

**Rule:** Invalidate `plan-vs-actual` **immediately** on mutation success — badge advisory only; PVA tab may be open.

Existing `plan-compare` invalidation on adjustment success must remain.

### Done when

- [ ] Add adjustment shows success toast (no silent success)
- [ ] Compare tab reflects new line without manual refresh
- [ ] PVA tab reflects changes when open (immediate invalidation)
- [ ] Update/delete invalidate `plan-vs-actual` per matrix
- [ ] Inline add form remains visible after empty/custom plan create

---

## T-0161 — Compare/PVA shipped UX verify

**Status:** open  
**Depends on:** T-0160  
**Decisions:** DEC-0073, DEC-0074  
**Architecture slice:** US-0014-S3

### Description

Verify-first audit of Q0019-shipped Compare and PVA UX — **no formula or API changes**.

| Surface | Verify |
|---------|--------|
| Compare footnote ~L600–603 | Overlay-only delta vs projected balance |
| Compare `monthly_delta_sum` | **0.00** on zero-adjustment plan (DEC-0073) |
| PVA `no_active_plan` | Guided card + Scenarios / Set active buttons (DEC-0074) |
| PVA with active plan | Month selector + plan vs actual rows |

Document smoke steps in `sprints/S0015/uat.md`.

### Done when

- [ ] Compare footnote explains overlay-only delta semantics
- [ ] Zero-adjustment plan shows 0.00 monthly delta
- [ ] PVA without active plan shows guided card (not blank tab)
- [ ] No Compare formula or PVA API changes introduced
- [ ] UAT steps documented for AC-3 and AC-4

---

## T-0162 — User guide US-0014 finalize + UAT OIDC template

**Status:** open  
**Depends on:** T-0161  
**Decisions:** DEC-0077, DEC-0059  
**Architecture slice:** US-0014-S3  
**Research:** R-0073 §5

### Description

Validate and finalize `docs/user-guides/US-0014.md` against shipped UI (`USER_GUIDE_MODE=1`):

| Section | Content |
|---------|---------|
| Purpose | First-run polish + mutation feedback |
| First visit | Template grid + Create empty plan |
| Set active | Plan vs Actual + Grafana Dashboard 3 |
| Compare | Overlay-only delta footnote (DEC-0073) |
| Troubleshooting | Red error card on mutation failure (DEC-0077) |

Add UAT template to `sprints/S0015/uat.md` for AC-8 OIDC smoke:

| Step | Contract |
|------|----------|
| Profile | US-0010 external (`financegnome.omniflow.cc`) |
| Tabs | Scenarios, Compare, Plan vs Actual |
| Gate | **BACKEND_FRONTEND_DEPLOY** before live smoke |
| Error path | After S2 — force failure shows red card |

Cross-link `docs/user-guides/US-0004.md`.

### Done when

- [ ] User guide sections match shipped PlanningPage behavior
- [ ] Troubleshooting documents visible mutation errors (AC-7)
- [ ] `uat.md` includes OIDC three-tab smoke template with operator gate
- [ ] No placeholder stubs remain in user guide
- [ ] `validate_doc_profile.py` unaffected (user guide path only)

---

## Execution order (recommended)

1. **S2 foundation:** T-0158 → T-0159 → T-0160
2. **S1 polish (after T-0158):** T-0155 ∥ T-0156; T-0157 after T-0158
3. **S3 verify (after S2):** T-0161 → T-0162
4. **Operator:** BACKEND_FRONTEND_DEPLOY → verify-work omniflow `/planning` smoke (UAT)

```text
T-0158 → T-0159 → T-0160
  ↓
(T-0155 + T-0156) ∥ T-0157
  ↓
T-0161 → T-0162
  ↓
Operator: deploy frontend → UAT OIDC smoke
```

## Acceptance coverage map

| AC | Tasks | Notes |
|----|-------|-------|
| Prerequisite | — | BUG-0011 AD/AE/AF DONE (checked) |
| AC-1 | T-0155 | Empty-state onboarding verify-only |
| AC-2 | T-0160, T-0159 | Add-line success + PVA invalidation |
| AC-3 | T-0161 | Compare 0.00 + footnote verify |
| AC-4 | T-0161 | PVA guided card verify |
| AC-5 | T-0157 | Create/template confirmations |
| AC-6 | T-0156 | Set-active banner Dashboard 3 |
| AC-7 | T-0158, T-0159 | Error surfaces on all mutations |
| AC-8 | T-0162 | OIDC smoke UAT template; live pass-with-prerequisites |

## Operator gate

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_FRONTEND_DEPLOY** | UAT AC-8 omniflow smoke | Deploy S1–S2 frontend changes on US-0010 external profile |

## Split decision

- **Why 8 tasks:** Architecture slices S1(3) + S2(3) + S3(2) = 8; under `SPRINT_MAX_TASKS=12`.
- **Why S2 first:** Helper (T-0158) prerequisite for toasts (T-0157) and onError (T-0159).
- **Why OIDC in UAT:** AC-8 verify-only; operator gate documented in T-0162 — mirrors S0014 pattern.
