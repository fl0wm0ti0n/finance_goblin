# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## discovery-20260607-bug0015 — BUG-0015 confirmed subscriptions reconfirm after rebuild (hot pointer)`
- Last archived heading: `## discovery-20260607-bug0015 — BUG-0015 confirmed subscriptions reconfirm after rebuild (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=25
  - retained_body_lines=496

---

## discovery-20260607-bug0015 — BUG-0015 confirmed subscriptions reconfirm after rebuild (hot pointer)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-07  
**Bug:** BUG-0015  
**Orchestrator run:** `auto-20260607-bug0015-001`  
**Next phase:** `/research`  
**Full handoff:** `handoffs/archive/po-to-tl-pack-20260607-g.md`

### Verdict summary

| ID | Verdict | Boundary |
|----|---------|----------|
| **H1** fingerprint drift | **LIKELY PRIMARY** | Code |
| **H2** DB ephemeral | **UNLIKELY sole** | Ops gate |
| **H3** alert/UI desync | **REFUTED primary** | Secondary |
| **H4** detection re-run | **Subsumed by H1** | Code |

**Fix boundary:** code primary (fingerprint / merchant-identity); ops SQL gate rules out H2. Research: payee-level confirm inheritance vs stable fingerprint. Evidence: [R-0081](docs/engineering/research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild), backlog BUG-0015 discovery table.

`triad_hot_surface`: discovery prepended; rollover units=2,1 → `po-to-tl-pack-20260607-g.md`; --check PASS (2026-06-07T19:30:00Z)

---

