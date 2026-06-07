# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## architecture-20260607-bug0015 — BUG-0015 confirm persistence architecture (hot pointer)`
- Last archived heading: `## architecture-20260607-bug0015 — BUG-0015 confirm persistence architecture (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=41
  - retained_body_lines=496

---

## architecture-20260607-bug0015 — BUG-0015 confirm persistence architecture (hot pointer)

**From:** Tech Lead  
**To:** Sprint-plan  
**Date:** 2026-06-07  
**Bug:** BUG-0015  
**Orchestrator run:** `auto-20260607-bug0015-001`  
**Next phase:** `/sprint-plan`

### Summary

Two-layer bundle frozen: **DEC-0084** (card `payee_key` normalization) + **DEC-0085**/**DEC-0086** (payee+interval skip+merge, ±3d tolerance, in-place fingerprint rotation). H1 primary; H2 ops gate; H3 refuted; H4 subsumed. Extends DEC-0071/0072 — no BUG-0008 reopen.

### Decisions

| ID | Layer | Contract |
|----|-------|----------|
| **DEC-0084** | 1 | Comma/asterisk/domain rules in `normalize.rs` (R-0082) |
| **DEC-0085** | 2 | `load_confirmed_payee_intervals` + merge upsert; rejection by payee+interval; stale map |
| **DEC-0086** | 2 | ±3d `interval_matches`; rotate fingerprint on confirmed merge |

### Execute scope (P0)

| Task | Surface |
|------|---------|
| **AU1** | `recurrence/normalize.rs` |
| **AU2** | `subscriptions/repository.rs` + index migration |
| **AU3** | `subscriptions/detection.rs`, `service.rs` |
| **AU4** | `detection.rs` stale by payee+interval + wire |
| **V1** | verify-work rebuild smoke (AU–AW) |

**Ops gate:** `subscription_patterns` status counts + Cursor/Apple fingerprint probe **before** Full sync.

**Evidence:** [R-0081 §fix matrix](docs/engineering/research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild), `docs/engineering/architecture.md` § BUG-0015, `handoffs/archive/po-to-tl-pack-20260607-j.md` (research)

`triad_hot_surface`: architecture prepended; --rollover units=0,0; --check PASS (2026-06-07T20:00:00Z)

**Recommended sprint:** `/quick` **Q0023** (5 tasks ≤ 12)

---

