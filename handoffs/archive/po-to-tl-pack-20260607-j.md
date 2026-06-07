# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## research-20260607-bug0015 — BUG-0015 confirm persistence fix research (hot pointer)`
- Last archived heading: `## research-20260607-bug0015 — BUG-0015 confirm persistence fix research (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=39
  - retained_body_lines=496

---

## research-20260607-bug0015 — BUG-0015 confirm persistence fix research (hot pointer)

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-07  
**Bug:** BUG-0015  
**Orchestrator run:** `auto-20260607-bug0015-001`  
**Next phase:** `/architecture`

### Summary

Discovery verdicts locked: **H1 fingerprint drift LIKELY PRIMARY**; H2 ops gate; H3 refuted primary; H4 subsumed. Extended **[R-0081](docs/engineering/research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild)** with fix matrix; added **[R-0082](docs/engineering/research.md#r-0082--card-billing-descriptor-normalization-for-subscription-identity)** (card descriptor normalization). **BUG-0008** prior art (DEC-0071 alert dedup, DEC-0072 payee normalization) necessary but insufficient when drift creates new `pattern_id`.

### Recommended approach (two-layer bundle)

| Layer | Option | Contract |
|-------|--------|----------|
| **1** | Card `payee_key` normalization (R-0082) | Left-prefix / domain collapse for Cursor/Apple-class descriptors |
| **2** | Payee+interval confirm inheritance (R-0081 §C) | Skip + merge on `(payee_key, interval_days ±3d)`; reject alert-only fix |

**Fallback:** skip-without-merge (R-0081 §D) if Layer 1 tests sufficient.

### Execute surfaces (P0)

| Task | Surface |
|------|---------|
| **AU1** | `recurrence/normalize.rs` |
| **AU2** | `subscriptions/repository.rs` |
| **AU3** | `subscriptions/detection.rs` |
| **V1** | verify-work rebuild smoke (AU–AW) |

**Ops gate:** `subscription_patterns` status counts + Cursor/Apple fingerprint probe pre-rebuild.

**Evidence:** [R-0081 §fix matrix](docs/engineering/research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild), discovery `handoffs/archive/po-to-tl-pack-20260607-h.md`

`triad_hot_surface`: research prepended; --rollover units=1,0; --check PASS (2026-06-07T19:30:00Z)

---

