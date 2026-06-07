# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## research-20260608-bug0011 — BUG-0011 planning mode research → architecture`
- Last archived heading: `## research-20260608-bug0011 — BUG-0011 planning mode research → architecture`
- Verification tuple (mandatory):
  - archived_body_lines=49
  - retained_body_lines=483

---

## research-20260608-bug0011 — BUG-0011 planning mode research → architecture

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-08  
**Bug:** BUG-0011  
**Orchestrator run:** auto-20260608-bug0011-001  
**Next phase:** `/architecture`

### Summary

Code + prior-plan research completed for BUG-0011 discovery open questions. Added **[R-0070](docs/engineering/research.md#r-0070--bug-0011-planning-mode-compare-delta-empty-state-api-first-run-ux)** resolving all six questions. No host `.env` or secrets read. Recommend architecture formalize **DEC-0073** (compare overlay-delta metric, AE) and **DEC-0074** (plan-vs-actual 200 `no_active_plan`, AF) before sprint-plan/execute; AD inline add-line + first-run empty plan wiring is execute scope without a third DEC.

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| **Compare delta contract** | **Overlay-only monthly delta** — sum `build_overlay_deltas` for current month; reject summing `planned_net`. **Projected month-end balance** stays full scenario total (baseline + overlay). Aligns with R-0016 "net recurring impact €/month". |
| **Empty-plan zero semantics** | Zero adjustments → **`monthly_delta_sum = 0.00`** mandatory. Projected balance may remain baseline forecast (negative OK); AE targets mislabeled delta column. |
| **plan-vs-actual empty API** | **HTTP 200** `{ status: "no_active_plan", reason: "no_active_plan" }` — mirror `risk-score` `no_score`. Reject 404 and reject auto-activate on create. |
| **First-run onboarding** | Empty state: template cards + **Create empty plan** (`POST template=custom`) + keep Leasing quick path. Explicit **Set active** retained; banner after create. |
| **Add-adjustment UX** | **Inline form** above table; wire POST/PATCH adjustments; Custom Apply toast (no silent no-op). Fields per R-0015. |
| **Regression scope** | OIDC `/planning` tab smoke required. **No Grafana Dashboard 3 change** — AE fixes compare endpoint only. |

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| Compare math | R-0070 §2 | DEC-0073 overlay-only delta helper; release note for number shift on non-empty plans |
| Empty API | R-0070 §3 | DEC-0074 tagged 200 response + frontend guided UX |
| First-run / AD | R-0070 §4 | Sprint execute; bound polish to US-0014 |
| Grafana | R-0020 + R-0070 §1 | No panel SQL change |
| DEC ID note | R-0070 §5 | Runbook forward-ref DEC-0073 (US-0090) — resolve ID allocation at architecture |

### Risks (carried forward)

1. Compare numbers shift for existing Leasing/Savings plans — overlay-only vs full forecast sum.
2. DEC-0073 ID may collide with US-0090 runbook forward-ref — architecture must allocate formal IDs.
3. Negative projected balance on zero-overlay plans reflects baseline — help text, not zeroing.

### Evidence

- Research: [R-0070](docs/engineering/research.md#r-0070--bug-0011-planning-mode-compare-delta-empty-state-api-first-run-ux), [R-0015](docs/engineering/research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline), [R-0016](docs/engineering/research.md#r-0016--plan-scenario-versioning-immutable-snapshots-vs-editable-drafts)
- Code: `backend/src/plan/{repository,service,project,overlay}.rs`, `backend/src/api/plans.rs`, `frontend/src/pages/PlanningPage.tsx`
- Acceptance: `docs/product/acceptance.md` (BUG-0011 AD/AE/AF, unchanged)
- Prior handoff: `handoffs/archive/po-to-tl-pack-20260606-b.md#discovery-20260608-bug0011`

---

