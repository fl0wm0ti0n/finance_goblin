# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## discovery-20260608-bug0011 — BUG-0011 planning mode discovery → research`
- Last archived heading: `## discovery-20260608-bug0011 — BUG-0011 planning mode discovery → research`
- Verification tuple (mandatory):
  - archived_body_lines=58
  - retained_body_lines=483

---

## discovery-20260608-bug0011 — BUG-0011 planning mode discovery → research

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-08  
**Bug:** BUG-0011  
**Orchestrator run:** auto-20260608-bug0011-001  
**Next phase:** `/research`

### Summary

Discovery confirmed all three sub-defects via code audit (no host secrets). **AD:** custom/empty plan path is non-functional because React has **no add-adjustment UI** despite backend `POST .../adjustments`; Custom template Apply succeeds silently with zero lines. **AE:** Compare **"Monthly delta sum"** aggregates full forecast `planned_net`, not adjustment-only overlay delta — empty plans show illogical negatives (**-127489.44**). **AF:** `plan-vs-actual` returns **404** when no active plan (`is_active=false` default); frontend tab renders blank with no guided UX (contrast risk-score **200** `no_score`). Vision and backlog BUG-0011 blocks updated; acceptance AD/AE/AF unchanged. Epic polish deferred **US-0014**.

### Confirmed findings (carry to research)

| Item | Resolution |
|------|------------|
| AD root cause | Missing add-line form + first-run empty state only offers Leasing create — `PlanningPage.tsx` delete-only table; `apply-template` Custom → `[]` |
| AD backend | `create_plan` / `apply_template(Custom)` / `add_adjustment` API all exist and work — defect is frontend wiring + onboarding |
| AE root cause | `version_metrics` + `project_adjustments_in_memory` sum `planned_net` (baseline + overlay), not overlay-only delta |
| AE UI mismatch | Column label "Monthly delta sum" implies scenario impact; code emits forecast totals |
| AF root cause | `PlanError::NoActivePlan` → HTTP **404**; plans created with `is_active=false`; no auto-activate |
| AF frontend | `pvaQuery` no error/empty-state branch — tab broken on 404 |
| AF precedent | `risk-score` returns **200** `{ status: "no_score", reason: "no_active_plan" }` — candidate pattern for AF |
| Prior docs | BUG-0004 404 note superseded by acceptance **AF**; US-0004 user guide promises manual custom lines |

### Open questions for `/research`

1. **Compare delta contract** — sum overlay deltas only vs delta vs **Current (Ist)** baseline version vs per-version relative; impact on non-empty Leasing/Savings compare rows (acceptance **AE**).
2. **Empty-plan zero semantics** — when `adjustment_count === 0`, force **0.00** monthly delta and neutral projected balance vs inherit baseline — lock formula before architecture.
3. **plan-vs-actual empty API** — **200** tagged `{ status: "no_active_plan" }` (mirror risk-score) vs **200** empty `rows: []` vs auto-activate first plan on create — pick one contract (acceptance **AF**).
4. **First-run onboarding** — add **Create empty plan** CTA when `plans.length === 0` vs show template cards in empty state; auto-activate first plan vs explicit **Set active** step.
5. **Add-adjustment UX** — inline table row vs modal; required fields (amount, frequency, target, direction) per US-0004 spec; wire `POST .../adjustments` + invalidate queries.
6. **Regression scope** — OIDC deploy smoke for `/planning` tabs; whether compare metric change requires Grafana Dashboard 3 panel review.

### Risks (from intake, refined at discovery)

1. AE metric redefinition may shift compare numbers for existing non-empty plans — need migration note or versioned metric field.
2. AF 200 empty-state change is breaking for clients expecting 404 — document in API/decision; frontend-only fix insufficient per acceptance.
3. AD fix alone may still feel unintuitive without US-0014 onboarding — bound defect scope to functional add-line + first-run path.

### Recommended next steps

1. `/research` — new **R-0070** entry: compare delta definition + plan-vs-actual empty-state contract + first-run activate/create matrix; extend US-0004 research pointers as needed.
2. `/architecture` — DEC for chosen AE formula and AF API shape; sprint scope for AD frontend wiring.

### Evidence

- Intake: `handoffs/intake_evidence/intake-20260605-planning-mode-broken.json`
- Vision: `docs/product/vision.md` (Discovery notes BUG-0011, 2026-06-08)
- Backlog: `docs/product/backlog.md#BUG-0011` (#### Discovery notes 2026-06-08)
- Acceptance: `docs/product/acceptance.md` (BUG-0011 AD/AE/AF, unchanged)
- Code: `frontend/src/pages/PlanningPage.tsx`, `backend/src/plan/repository.rs` (`version_metrics`, `create_plan`), `backend/src/plan/service.rs` (`project_adjustments_in_memory`, `plan_vs_actual`), `backend/src/api/plans.rs` (`plan_error_status`, `add_adjustment`), `backend/src/plan/project.rs`, `backend/src/plan/templates.rs`
- User guide: `docs/user-guides/US-0004.md` (custom lines manual add)
- Related epic: **US-0014** (holistic UX if defect fixes insufficient)

---

