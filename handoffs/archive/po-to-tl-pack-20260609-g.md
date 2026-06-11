# PO to TL archive pack (2026-06-09)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 11
- First archived heading: `## sprint-plan-20260609-bug0016 — BUG-0016 SPA deep-link fallback sprint (hot pointer)`
- Last archived heading: `## sprint-plan-20260609-bug0016 — BUG-0016 SPA deep-link fallback sprint (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=27
  - retained_body_lines=498

---

## sprint-plan-20260609-bug0016 — BUG-0016 SPA deep-link fallback sprint (hot pointer)

**From:** Tech Lead  
**To:** Plan-verify / Dev  
**Date:** 2026-06-09  
**Bug:** BUG-0016  
**Orchestrator run:** `intake-20260609-ui-audit`  
**Next phase:** `/plan-verify`

### Summary

Quick sprint **Q0024** materialized — 3 tasks (3/12 under `SPRINT_MAX_TASKS`; no split): **AX1** `ServeDir::fallback(ServeFile)` in `build_router` (**DEC-0104**), **AX2** integration tests, **V1** verify-work curl + browser smoke. Acceptance **AX** traced to all three tasks.

### Tasks

| ID | Surface | Gate |
|----|---------|------|
| **AX1** | `backend/src/lib.rs` | — |
| **AX2** | `backend/tests/` | after AX1 |
| **V1** | `uat.md` curl + browser | BACKEND_FRONTEND_DEPLOY |

**Evidence:** `sprints/quick/Q0024/`, `handoffs/tl_to_dev.md` sprint-plan-20260609-q0024-bug0016

**Recommended next:** `/plan-verify` (qa) → `/execute` (dev)

---

