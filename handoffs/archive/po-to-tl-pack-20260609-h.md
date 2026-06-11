# PO to TL archive pack (2026-06-09)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 11
- First archived heading: `## discovery-20260609-bug0017 — BUG-0017 post-sync forecast recompute cluster (hot pointer)`
- Last archived heading: `## discovery-20260609-bug0017 — BUG-0017 post-sync forecast recompute cluster (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=59
  - retained_body_lines=498

---

## discovery-20260609-bug0017 — BUG-0017 post-sync forecast recompute cluster (hot pointer)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-09  
**Bug:** BUG-0017  
**Orchestrator run:** `intake-20260609-ui-audit`  
**Next phase:** `/research`

### Summary

UI audit cluster **UI-002/006/009/010**: post-sync forecast recompute logs WARN on **ai_tool_audit** CHECK violations and **paired_baseline_id** FK delete, while sync status stays **success**. Operators see disabled ML controls, **Plan stale** on Planning Compare, and a transient Forecast empty state.

**Verdict:** **Two confirmed backend root defects** (audit CHECK gap + retention FK order), **one conditional ML gate** (verify after fix), **two downstream/UX symptoms** (plan stale, empty flash).

### Sub-defect verdicts

| AC | Verdict | Root cause | Research task |
|----|---------|------------|---------------|
| **AY** | **CONFIRMED** | `006_ai_audit.sql` CHECK never extended for `forecast_bucket_assignment` (US-0015 gap) | Migration enum extension pattern |
| **AZ** | **CONFIRMED** | CHECK rejects `low_confidence` / extended `result_status` values | Same migration or status mapping |
| **BA** | **CONFIRMED** | `enforce_retention()` deletes baseline before ML rows referencing `paired_baseline_id` | Delete order / CASCADE / SET NULL |
| **BB** | **VERIFY AFTER AY–BA** | `insufficient_history` with 922 txs — legitimate gate vs stale baseline | ML history threshold probe |
| **BC** | **DOWNSTREAM** | Plan stale from failed/skipped post-forecast refresh | Re-smoke after **BA** |
| **BD** | **CONFIRMED UX** | `ForecastPage` treats meta query pending as empty | `isPending` / placeholderData fix |

### Operator gates (mandatory before sprint)

1. **BACKEND_FRONTEND_DEPLOY** — apply BUG-0017 delta on localhost:18080 / omniflow
2. **Full sync + recompute** — `POST /api/v1/sync/trigger`; logs clean of audit/FK WARN
3. **forecast/meta** — `stale=false`; ML honest skip or `ml_computation_id` set
4. **Planning Compare** — no **Plan stale** badge after successful recompute
5. **Forecast nav** — no false **No forecast data yet** when meta has `computation_id`

### Research pointers (for `/research` — extend prior forecast audit work)

- `ai_tool_audit` CHECK migration strategy (compare US-0006 chat tools vs US-0015 S0016 T-0173 gap)
- FK retention: ML-before-baseline delete vs `ON DELETE CASCADE` on `paired_baseline_id`
- ML `insufficient_history` gate with 922 categorized transactions — threshold in `forecast_ml` vs sidecar
- ForecastPage react-query loading contract for **BD**
- Optional: sync phase should fail vs warn when recompute errors (product decision — default defer)

### Discovery decomposition evidence

- Workflow count: 1 operator gate (sync → recompute inspect) — **single bug retained**
- Cross-cutting: migrations, `forecast/repository.rs`, `forecast/service.rs`, optional `ForecastPage.tsx`
- Acceptance: 6 AC (**AY–BD**) unchanged
- Risk: do not mask true insufficient_history when data genuinely too short

### Artifacts updated

- `docs/product/vision.md` § BUG-0017 discovery
- `docs/product/backlog.md#BUG-0017`
- `handoffs/resume_brief.md`

**Evidence:** `handoffs/intake_evidence/intake-20260609-forecast-recompute.json`, `handoffs/intake_evidence/ui-audit-20260609-local.json` (UI-002, UI-006, UI-009, UI-010), `backend/migrations/006_ai_audit.sql`, `backend/migrations/009_forecast_ml.sql`, `backend/src/forecast/repository.rs`, `backend/src/forecast/service.rs`, `frontend/src/pages/ForecastPage.tsx`

`triad_hot_surface`: BUG-0017 discovery archived to hot pack; --rollover units=1 → `handoffs/archive/po-to-tl-pack-20260609-h.md` (+ prior unit → `po-to-tl-pack-20260609-g.md`); hot pointer prepended; --check PASS (2026-06-09T22:15:00Z)

---

