# PO to TL archive pack (2026-06-05)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 7
- First archived heading: `## discovery-20260605-bug0012 — BUG-0012 forecast monthly bucket discovery (hot pointer)`
- Last archived heading: `## discovery-20260605-bug0012 — BUG-0012 forecast monthly bucket discovery (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=24
  - retained_body_lines=485

---

## discovery-20260605-bug0012 — BUG-0012 forecast monthly bucket discovery (hot pointer)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Bug:** BUG-0012  
**Orchestrator:** `auto-20260605-bug0012-001`  
**Next phase:** `/research`  
**Full handoff:** `handoffs/archive/po-to-tl-pack-20260605-b.md` (rollover archived full discovery section)

### Summary (condensed)

Both sub-defects **AG/AH confirmed**. Root cause: `categorize_delta` ignores `category_names` and passes `None` to `map_category` for negative deltas → Fixed never populated; Income only when net daily delta ≥ 0. `RecurringPattern` lacks `category_id`. Monthly API/UI read path OK; fix is projection decomposition + category wiring (DEC-0007). US-0015 AI out of scope.

### Triad check (discovery phase)

| Surface | Result |
|---------|--------|
| backlog / acceptance / vision / code trace | pass |
| `po_to_tl.md` / `state.md` rollover | pass |
| `architecture.md` 4624/3000 lines | **fail** — `STATE_ARCHIVE_REQUIRED` (pre-existing) |

---

