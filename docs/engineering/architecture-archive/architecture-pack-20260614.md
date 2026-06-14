# Architecture archive pack (2026-06-14)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=120`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 12
- First archived heading: `## US-0014 — Planning mode intuitive UX completion`
- Last archived heading: `## US-0014 — Planning mode intuitive UX completion`
- Verification tuple (mandatory):
  - archived_body_lines=232
  - preamble_lines=10
  - retained_body_lines=2784

---

## US-0014 — Planning mode intuitive UX completion

**Status:** Architecture complete (2026-06-08)  
**Discovery:** `discovery-20260608-us0014` in `handoffs/po_to_tl.md` / `handoffs/archive/po-to-tl-pack-20260606-k.md`  
**Research:** [R-0072](research.md#r-0072--us-0014-planning-ux-epic-gap-beyond-bug-0011), [R-0073](research.md#r-0073--us-0014-planning-mutation-error-toast-patterns)  
**Decisions:** **DEC-0077** (planning mutation feedback); frozen **DEC-0073**, **DEC-0074**, **DEC-0024**  
**Depends on:** BUG-0011 DONE (Q0019), US-0004 (plan engine)  
**Sprint:** **S0015** recommended — slices US-0014-S1..S3  
**Acceptance:** `docs/product/acceptance.md` § US-0014 (9 rows)  
**Spec-pack:** `docs/engineering/spec-pack/US-0014-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**User guide:** `docs/user-guides/US-0014.md` (`USER_GUIDE_MODE=1`)

### Problem

Q0019 shipped **5 of 8** epic acceptance rows in `PlanningPage.tsx` (onboarding grid, Compare footnote, PVA guided card, template paths, set-active banner skeleton). US-0014 closes the remaining **polish + error surfaces + operator smoke** — not first-run greenfield.

| AC | Discovery verdict | Execute weight |
|----|-------------------|----------------|
| AC-1 Onboarding | Shipped | S1 verify |
| AC-2 Add-lines | Partial | S2 primary |
| AC-3 Compare UX | Shipped (DEC-0073) | S3 verify |
| AC-4 PVA guided | Shipped (DEC-0074) | S3 verify |
| AC-5 Templates | Partial | S1 confirmation toasts |
| AC-6 Set-active | Partial | S1 Dashboard 3 copy |
| AC-7 Errors | **Gap** | **S2 primary** |
| AC-8 OIDC | Verify | S3 smoke |

`isolation_scope`: artifact + repo source reads; no host `.env` / secrets read.

### System context (unchanged backend)

```text
┌─────────────────────────────────────────────────────────────────┐
│  /planning (PlanningPage.tsx) — three tabs                      │
├─────────────┬─────────────────────┬─────────────────────────────┤
│  Scenarios  │  Compare            │  Plan vs Actual             │
│  templates  │  overlay delta      │  200 no_active_plan         │
│  add form   │  DEC-0073 0.00      │  guided card DEC-0074       │
│  set active │  contextual footnote│  month selector             │
└─────────────┴─────────────────────┴─────────────────────────────┘
         │                    │                    │
         └────────────────────┴────────────────────┘
                              │
                              ▼
              POST/PATCH/DELETE /api/v1/plans/*  (contracts frozen)
                              │
                              ▼
              Grafana Dashboard 3 uid=budgets (active plan, DEC-0024)
```

**No backend metric or API contract changes** unless execute finds regression.

### Architecture contract (DEC-0077)

```text
US-0014
├── S1 — Verify shipped UX + banner + confirmation toasts (P1)
│   ├── V1 — AC-1 empty-state template grid regression verify
│   ├── T1 — Extend set-active banner: Plan vs Actual + Grafana Dashboard 3 (AC-6)
│   └── T2 — Success toasts on createPlan + Create from {template} (AC-5)
├── S2 — Mutation feedback + invalidation (P0 — primary)
│   ├── T1 — planningFeedback helper (success/error card variants)
│   ├── T2 — onError on all 7 mutations (AC-7)
│   └── T3 — addAdjustment success toast + plan-vs-actual invalidation (AC-2)
└── S3 — Verify + operator smoke + user guide (P1)
    ├── V1 — AC-3 Compare footnote + 0.00 overlay; AC-4 PVA guided card
    ├── T1 — docs/user-guides/US-0014.md (first-run, Set active, Compare semantics)
    └── V1 — OIDC `/planning` three-tab smoke (AC-8; BACKEND_FRONTEND_DEPLOY gate)
```

**Out of scope:** Compare formula changes (DEC-0073); PVA API shape (DEC-0074); auto-activate first plan; global toast library / MutationCache refactor.

### S1 — Verify + banner + confirmations (frozen)

#### AC-1 verify-only

Empty branch already ships: name field, primary **Create empty plan**, six-template `card-grid`. Execute **must not rewrite** — regression checklist only.

#### Set-active banner (AC-6)

Extend yellow banner copy:

| Element | Contract |
|---------|----------|
| Plan vs Actual | Existing — active plan required for PVA data |
| Grafana | Add cue: Dashboard 3 **Budgets** (`uid=budgets`) reflects the **active** plan |
| Trigger | `showSetActiveBanner` after `createPlanMutation` success when `is_active=false` |
| Dismiss | Clears on successful `activateMutation` (existing) |

#### Confirmation toasts (AC-5)

| Path | Message pattern |
|------|-----------------|
| Create empty plan | `Plan "{name}" created` |
| Create from template | `Plan "{name}" created from {template label}` |
| Apply template (existing plan) | `Template applied` or template-specific |

Use DEC-0077 success variant (green card, 4s auto-dismiss).

### S2 — Mutation feedback (frozen — DEC-0077)

#### Helper contract

```typescript
// frontend/src/pages/planningFeedback.ts (or inline in PlanningPage.tsx)
showPlanningFeedback({ kind: 'success' | 'error', message: string })
formatPlanningError(err: unknown, fallback: string): string
```

| Variant | Background | Dismiss |
|---------|------------|---------|
| success | `#ecfdf5` | Auto 4s |
| error | `#fef2f2` | Manual Dismiss button |

Single active slot — new feedback replaces prior (prevents toast flood on retry).

#### Mutation matrix

| Mutation | onError (required) | onSuccess toast | Extra invalidation |
|----------|-------------------|-----------------|-------------------|
| `createPlanMutation` | ✓ | ✓ create confirmation | `plan-vs-actual` |
| `activateMutation` | ✓ | ✓ "Plan set as active" | `plan-vs-actual` (existing `plans`) |
| `applyTemplateMutation` | ✓ | ✓ all templates | existing keys |
| `createVersionMutation` | ✓ | optional | — |
| `addAdjustmentMutation` | ✓ | ✓ "Adjustment added" | **`plan-vs-actual`** |
| `updateAdjustmentMutation` | ✓ | optional | **`plan-vs-actual`** |
| `deleteAdjustmentMutation` | ✓ | optional | **`plan-vs-actual`** |

**Rejected:** Global `MutationCache` — no toast library; one-page scope (R-0073).

#### Error message extraction

`apiFetch` throws `Error` with response body text. Helper truncates to 240 chars; prepends mutation label when empty.

### S3 — Verify + smoke + docs (frozen)

| Surface | Verify |
|---------|--------|
| Compare footnote L600–603 | Overlay-only delta vs projected balance |
| Compare `monthly_delta_sum` | **0.00** on zero-adjustment plan (DEC-0073) |
| PVA `no_active_plan` | Guided card + Scenarios / Set active buttons (DEC-0074) |
| OIDC smoke | All three tabs on US-0010 external profile |

**Operator gate:** **BACKEND_FRONTEND_DEPLOY** before V1 omniflow smoke (same as Q0019).

### Codebase map (planning UX slice)

| Path | Role | US-0014 touch |
|------|------|---------------|
| `frontend/src/pages/PlanningPage.tsx` | Primary UX surface | **S1–S3** — mutations, banner, toasts |
| `frontend/src/pages/planningFeedback.ts` | Feedback helper (new, optional extract) | **S2** — DEC-0077 |
| `frontend/src/lib/api.ts` | `apiFetch` error shape | Reference — no change expected |
| `backend/src/api/plans.rs` | Plans API | **Verify only** — frozen |
| `backend/src/plan/overlay.rs` | Compare delta | **Verify only** — DEC-0073 |
| `grafana/provisioning/dashboards/analytics/budgets.json` | Dashboard 3 | **Copy reference only** — no SQL edit |
| `docs/user-guides/US-0014.md` | End-user guide | **S3** — created at architecture |
| `docs/engineering/spec-pack/US-0014-*.md` | Spec-pack trio | Created at architecture |

### Recommended sprint S0015 (slices — sprint-plan materializes tasks)

| Slice | Boundary | Tasks (est.) | Acceptance rows |
|-------|----------|--------------|-----------------|
| **US-0014-S1** | Verify AC-1 + banner + create confirmations | ~3 | AC-1, AC-5, AC-6 |
| **US-0014-S2** | Mutation helper + errors + invalidation | ~3 | AC-2, AC-7 |
| **US-0014-S3** | Verify Compare/PVA + user guide + OIDC smoke | ~2 | AC-3, AC-4, AC-8 |

**Count:** ~8 tasks (< `SPRINT_MAX_TASKS` 12) → **single sprint S0015**; no split.

**Sequencing:** S2 helper (T1) before S2 onError wiring (T2); S1 verify may parallel S2; S3 after S2 lands (smoke validates error paths).

### Test strategy

| Check | Type | Pass criteria |
|-------|------|---------------|
| AC-1 regression | Manual / component | Empty state shows grid + Create empty plan |
| AC-7 error surface | Manual | Force 4xx/5xx (e.g. invalid amount) → red error card visible |
| AC-2 add-line | Manual | Submit adjustment → success toast + Compare/PVA refresh |
| AC-3/AC-4 | Manual | Zero-overlay 0.00; PVA guided when no active plan |
| AC-6 | Manual | Banner mentions Dashboard 3 after create |
| AC-8 | Operator V1 | OIDC `/planning` three-tab smoke on omniflow |
| Backend regression | `cargo test --test plans_integration` | Existing 5 tests pass — no API change |

### Triad check (architecture phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#US-0014` | Discovery audit + slice boundaries linked | pass |
| `docs/product/acceptance.md` § US-0014 | 9 rows unchanged; mapped to S1–S3 | pass |
| `frontend/src/pages/PlanningPage.tsx` | Gap matrix documented in codebase map | pass |
| R-0072 + R-0073 | Discovery questions resolved; DEC-0077 formalized | pass |
| DEC-0073 / DEC-0074 | Frozen — no architecture drift | pass |

`triad_hot_surface`: post-write `--check` required; architecture § US-0014 appended; DEC-0077 formalized; spec-pack + user guide created.

### Decisions (US-0014)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0077 | Planning mutation feedback | Page-local helper; mandatory onError; success confirmations; PVA invalidation |

Full record: `decisions/DEC-0077.md`

### Risks

| Risk | Mitigation |
|------|------------|
| Duplicate Q0019 work | S1/S3 verify-only for shipped AC-1/AC-3/AC-4 — do not rewrite empty state |
| Toast flood on retry | Single active feedback slot |
| Over-scoping global toast lib | DEC-0077 explicitly page-local; extract later if needed |
| OIDC smoke without deploy | BACKEND_FRONTEND_DEPLOY gate on V1 |
| Negative projected balance confusion | Help text only — DEC-0073 frozen |

### Acceptance mapping

| Row | Architecture slice | Verify |
|-----|-------------------|--------|
| Prerequisite | — | BUG-0011 DONE (checked) |
| AC-1 | S1 | Empty-state regression |
| AC-2 | S2 | Add-line success + invalidation |
| AC-3 | S3 | Compare footnote + 0.00 overlay |
| AC-4 | S3 | PVA guided card |
| AC-5 | S1 | Create confirmation toasts |
| AC-6 | S1 | Set-active + Dashboard 3 banner |
| AC-7 | S2 | All mutation onError surfaces |
| AC-8 | S3 | OIDC three-tab smoke |

### Next phase

`/sprint-plan` **S0015** — materialize US-0014-S1..S3 tasks from slice table; S2-weighted sequencing; then `/plan-verify` → `/execute`.

---

