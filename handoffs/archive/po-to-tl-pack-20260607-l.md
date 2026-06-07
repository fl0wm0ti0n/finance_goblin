# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## sprint-plan-20260607-q0023-bug0015 — BUG-0015 confirm persistence sprint-plan (hot pointer)`
- Last archived heading: `## sprint-plan-20260607-q0023-bug0015 — BUG-0015 confirm persistence sprint-plan (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=31
  - retained_body_lines=496

---

## sprint-plan-20260607-q0023-bug0015 — BUG-0015 confirm persistence sprint-plan (hot pointer)

**From:** Tech Lead  
**To:** Plan-verify / Dev  
**Date:** 2026-06-07  
**Bug:** BUG-0015  
**Orchestrator run:** `auto-20260607-bug0015-001`  
**Next phase:** `/plan-verify`

### Summary

**Q0023** materialized — 5 tasks (AU1–AU4 + V1); DEC-0084/0085/0086; acceptance **AU**–**AW**; no split.

### Tasks

| ID | Surface | AC |
|----|---------|-----|
| **AU1** | `normalize.rs` | AU, AV |
| **AU2** | `repository.rs` + migration | AU, AV |
| **AU3** | `detection.rs`, `service.rs` | AU, AV, AW |
| **AU4** | stale payee+interval | AV |
| **V1** | rebuild smoke | AU–AW |

**Ops gates:** BACKEND_FRONTEND_DEPLOY + POSTGRES_PERSISTENCE_PROBE + FULL_FIREFLY_SYNC before V1.

**Evidence:** `sprints/quick/Q0023/`, `handoffs/tl_to_dev.md` (`sprint-plan-20260607-q0023-bug0015`)

`triad_hot_surface`: sprint-plan prepended; --check PASS (2026-06-07T20:30:00Z)

---

