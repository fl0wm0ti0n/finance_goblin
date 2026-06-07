# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 8
- First archived heading: `## discovery-20260608-us0014 — US-0014 planning UX discovery (post-Q0019 audit)`
- Last archived heading: `## discovery-20260608-us0014 — US-0014 planning UX discovery (post-Q0019 audit)`
- Verification tuple (mandatory):
  - archived_body_lines=85
  - retained_body_lines=471

---

## discovery-20260608-us0014 — US-0014 planning UX discovery (post-Q0019 audit)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-08  
**Story:** US-0014  
**Orchestrator run:** auto-20260608-us0014-001  
**Next phase:** `/research`

### Summary

Discovery audits `frontend/src/pages/PlanningPage.tsx` **after Q0019 (BUG-0011)** against AC-1..AC-8. **Five acceptance rows are already shipped in code** (onboarding grid, compare help, PVA guided card, template paths, set-active banner skeleton). US-0014 execute **narrows to error surfaces (AC-7), mutation feedback/invalidation (AC-2/AC-5), Set-active + Grafana copy (AC-6), and omniflow OIDC smoke (AC-8)** — not first-visit greenfield.

### AC audit matrix (PlanningPage.tsx post-Q0019)

| AC | Verdict | Code evidence | Execute note |
|----|---------|---------------|--------------|
| **AC-1** | **Shipped** | `empty` branch: name field, primary **Create empty plan**, six-template `card-grid` | Verify regression only |
| **AC-2** | **Partial** | Inline add form + `addAdjustmentMutation`; custom create → Scenarios tab | Add success/error feedback; invalidate PVA/detail after add |
| **AC-3** | **Shipped** | Compare footnote overlay-only delta vs projected balance | Verify **0.00** overlay display (DEC-0073) |
| **AC-4** | **Shipped** | `no_active_plan` guided card + Scenarios / Set active buttons | Verify copy polish only |
| **AC-5** | **Partial** | Empty **Create from {label}** + existing **Apply** | Toast on create-from-template / createPlan success paths |
| **AC-6** | **Partial** | `showSetActiveBanner` after create | Extend banner for **Grafana Dashboard 3** active-plan cue |
| **AC-7** | **Gap** | No mutation `onError` / visible error UI | **Primary S2 work** — shared toast/inline helper |
| **AC-8** | **Verify** | N/A (runtime) | OIDC `/planning` three-tab smoke; **BACKEND_FRONTEND_DEPLOY** prerequisite |

### Partial implementation review

| Area | Status | Execute note |
|------|--------|--------------|
| First-run empty state (AC-1) | **Done** (Q0019) | S1 verify |
| Template grid empty + Scenarios (AC-5) | **Done** paths / **Partial** feedback | S1 confirmation toasts |
| Inline add adjustment form (AC-2) | **Done** wiring | S2 error + success + invalidation |
| Compare contextual help (AC-3) | **Done** (Q0019) | S3 verify |
| PVA guided empty state (AC-4) | **Done** (Q0019) | S3 verify |
| Set-active banner (AC-6) | **Partial** | S1 Dashboard 3 copy |
| Mutation error surfaces (AC-7) | **Missing** | S2 primary |
| OIDC regression (AC-8) | **Deferred** | S3 omniflow smoke |

### Discovery decomposition evidence

- Feature/workflow count: 6 UX surfaces — **3 shipped, 3 partial/gap** (low remaining breadth)
- Cross-cutting impact: `PlanningPage.tsx` only; DEC-0073/0074 frozen
- Acceptance breadth: **9 rows unchanged** (prerequisite checked + AC-1–AC-8)
- Risk surface: duplicate Q0019 work if S1/S3 rewrite shipped paths; toast flood on retry

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#US-0014` | Discovery notes + AC audit | pass |
| `docs/product/acceptance.md` US-0014 | 9 criteria still valid | pass |
| `frontend/src/pages/PlanningPage.tsx` | AC-1..AC-8 matrix documented | pass |
| `docs/product/vision.md` | Discovery notes US-0014 2026-06-08 | pass |
| R-0072 | Q0019 overlap resolved | pass |

`triad_hot_surface`: (recorded post `--rollover` / `--check`)

### Open questions (carry to `/research`)

| Topic | Question |
|-------|----------|
| **Error UX** | Single `planningToast` helper vs per-mutation inline — align with existing green success card pattern |
| **Invalidation** | Should add/adjustment invalidate `plan-vs-actual` immediately or after recompute badge clears? |
| **Confirmation scope** | Toast on every template create vs primary CTA only (avoid noise) |
| **User guide** | USER_GUIDE_MODE=1 — scope `docs/user-guides/US-0014.md` sections at architecture |

### Recommended next steps

1. `/research` — resolve toast/invalidation open questions; no web research required unless cross-page pattern emerges
2. `/architecture` — only if shared planning mutation helper crosses components
3. `/sprint-plan` — materialize US-0014-S1..S3 with **S2 weighted** (AC-7 primary)

### Evidence

- Vision: `docs/product/vision.md` (Discovery notes US-0014 2026-06-08)
- Backlog: `docs/product/backlog.md#US-0014` (#### Discovery notes 2026-06-08)
- Intake: `handoffs/intake_evidence/intake-20260608-us0014.json`
- Prior release: `handoffs/releases/Q0019-release-notes.md`, `frontend/src/pages/PlanningPage.tsx`
- Research: [R-0072](docs/engineering/research.md#r-0072--us-0014-planning-ux-epic-gap-beyond-bug-0011)
- Acceptance: `docs/product/acceptance.md` § US-0014 (9 rows, unchanged)
- Decisions: DEC-0073, DEC-0074

---

