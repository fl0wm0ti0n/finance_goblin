# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 8
- First archived heading: `## intake-20260608-us0014 — US-0014 planning UX epic (post-BUG-0011)`
- Last archived heading: `## intake-20260608-us0014 — US-0014 planning UX epic (post-BUG-0011)`
- Verification tuple (mandatory):
  - archived_body_lines=59
  - retained_body_lines=471

---

## intake-20260608-us0014 — US-0014 planning UX epic (post-BUG-0011)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-08  
**Story:** US-0014  
**Orchestrator:** `auto-20260608-us0014-001`  
**Next phase:** `/discovery`

### Summary

Re-intake refines the **P2 planning UX epic** deferred from **BUG-0011**. Functional gates **AD/AE/AF** are **released** (Q0019, DEC-0073, DEC-0074). US-0014 captures holistic first-visit polish: onboarding template grid, add-lines discoverability, Compare contextual help, PVA guided UX, template discoverability, Set-active guidance, and visible error surfaces.

### Scope (bounded)

| In | Out |
|----|-----|
| First-run template grid + **Create empty plan** primary CTA | Compare metric formula changes (DEC-0073 frozen) |
| Inline add-lines polish + mutation error toasts | PVA API contract changes (DEC-0074 frozen) |
| Compare help copy (overlay-only delta vs projected balance) | Auto-activate first plan |
| PVA guided card polish (Set active / Scenarios links) | AI plan simulation chat (US-0006) |
| Set-active banner after first create | Crypto allocation scenarios (US-0007) |
| Template discoverability from empty state + Scenarios | |

### Research pointers

- [R-0070](docs/engineering/research.md#r-0070--bug-0011-planning-mode-compare-delta-empty-state-api-first-run-ux) — fulfilled Q0019; §4 first-run onboarding spec
- [R-0072](docs/engineering/research.md#r-0072--us-0014-planning-ux-epic-gap-beyond-bug-0011) — post-BUG-0011 gap analysis + slice map

### Decomposition

**Single epic retained** — sprint-plan slices **US-0014-S1..S3**:

| Slice | Boundary |
|-------|----------|
| S1 | Onboarding + templates + Set-active banner (AC-1, AC-5, AC-6) |
| S2 | Add-lines polish + error surfaces (AC-2, AC-7) |
| S3 | Compare help + PVA guided + OIDC smoke (AC-3, AC-4, AC-8) |

### Risks for discovery

1. **Q0019 overlap** — audit shipped `PlanningPage.tsx` vs AC-1/AC-2 to avoid duplicate execute work
2. **Negative projected balance** — help text only; do not zero baseline month-end (DEC-0073)
3. **USER_GUIDE_MODE=1** — `docs/user-guides/US-0014.md` required at release

### Acceptance traceability

Nine rows in `docs/product/acceptance.md` § US-0014 — prerequisite BUG-0011 checked; 8 open AC-1–AC-8.

### Intake evidence

- `intake_run_id`: `intake-20260608-us0014`
- `selected_pack`: `small-intake-pack`
- Evidence bundle: `handoffs/intake_evidence/intake-20260608-us0014.json`
- `prior_intake_ref`: `handoffs/intake_evidence/intake-20260605-planning-mode-broken.json`
- Split: single epic → sprint-plan slices S1..S3

---

