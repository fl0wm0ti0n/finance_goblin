# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## discovery-20260607-bug0015 — BUG-0015 confirmed subscriptions reconfirm after rebuild`
- Last archived heading: `## discovery-20260607-bug0015 — BUG-0015 confirmed subscriptions reconfirm after rebuild`
- Verification tuple (mandatory):
  - archived_body_lines=71
  - retained_body_lines=496

---

## discovery-20260607-bug0015 — BUG-0015 confirmed subscriptions reconfirm after rebuild

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-07  
**Bug:** BUG-0015  
**Orchestrator run:** `auto-20260607-bug0015-001`  
**Intake run:** `intake-20260607-subscription-reconfirm`  
**Next phase:** `/research`

### Summary

Post-rebuild re-confirm prompts for Cursor (€17.18/mo) and Apple (€9.99/mo) on omniflow trace to **fingerprint drift on detection re-run** (H1), not alert/UI desync (H3) or missing `confirmed_fps` skip (H4). Confirm state binds to `payee_key + interval_days + median_amount` hash; status preservation applies only on **exact** fingerprint match. Post-rebuild Full sync is expected to re-run detection; skip logic works when fingerprint is stable.

### Hypothesis verdicts

| ID | Verdict | Confidence | Root cause | Boundary |
|----|---------|------------|------------|----------|
| **H1** | **LIKELY PRIMARY** | high (mechanism) / medium (operator symptom) | `compute_fingerprint` includes amount + interval + normalized payee; re-group after sync can drift → new pending row; prior confirmed row orphaned on old fingerprint | **Code** |
| **H2** | **UNLIKELY sole** | medium | External profile postgres on traefik; BUG-0014 app-only rebuild; merchant-specific symptom inconsistent with total DB wipe | **Ops gate** |
| **H3** | **REFUTED primary** | high | UI Confirm/Reject requires API `status=pending`; cannot explain without new pending DB rows | Secondary only |
| **H4** | **Subsumed by H1** | high | `load_confirmed_fingerprints` before emit confirmed (`service.rs` L40–44); skip at `detection.rs` L43–44 — failure is fingerprint change, not pipeline ordering | **Code** (H1) |

### Boundary split

| Class | Scope | Action |
|-------|-------|--------|
| **Code (primary)** | Fingerprint / merchant-identity contract | Research fix options before architecture |
| **Ops (gate)** | Postgres persistence across rebuild | Operator SQL before execute closes H2 |
| **Out of scope** | Reopen BUG-0008, merge BUG-0014 | Intake decisions retained |

### Code surfaces (confirmed)

- `backend/src/recurrence/detect.rs` — `compute_fingerprint`, `median_amount` from recent 6 txs
- `backend/src/recurrence/group.rs` — `extract_payee_source` (description vs counterparty)
- `backend/src/recurrence/normalize.rs` — `payee_key` SEPA/reference stripping
- `backend/src/subscriptions/repository.rs` — `upsert_pending_pattern` status CASE on conflict
- `backend/src/subscriptions/detection.rs` — `confirmed_fps` skip, `emit_detection_alert` on new fingerprint only
- `backend/src/sync/mod.rs` — subscriptions phase after Firefly sync success

### Operator gates (research / verify-work)

1. Confirm Cursor + Apple → record `fingerprint`, `payee_key`, `current_amount`, `interval_days`.
2. Rebuild app containers only (not postgres) per BUG-0014 pattern.
3. **Before Full sync:** `SELECT fingerprint, status, payee_key, current_amount FROM subscription_patterns WHERE display_name ILIKE '%cursor%' OR payee_key ILIKE '%apple%';`
4. Full sync → re-query; expect duplicate rows (old `confirmed` + new `pending`) if H1 confirmed.

### Open questions for `/research`

1. **Stable fingerprint contract** — exclude `median_amount` from hash vs round-bucket tolerance vs payee-only identity key.
2. **Confirm propagation** — on detect, match by `payee_key` and inherit `confirmed` from any prior confirmed row for same merchant.
3. **Duplicate cleanup** — migration to merge orphan confirmed + new pending pairs for known merchants.
4. **AW secondary** — confirm H1 fix eliminates spurious `new_detection` for drifted fingerprints (pattern_id changes).

### Risks

- Payee-only fingerprint increases false-merge risk (distinct merchants with same normalized key).
- Amount-excluded fingerprint may miss legitimate price-change alerts on same merchant.
- Operator SQL probe required before closing H2 definitively.

### Evidence

- Intake: `handoffs/intake_evidence/intake-20260607-subscription-reconfirm.json`
- Research: [R-0081](docs/engineering/research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild) — extend with discovery verdicts
- Backlog: `docs/product/backlog.md` § BUG-0015 discovery table
- Related: **US-0003** confirm contract; **BUG-0008** DONE (alert dedup — secondary AW only)

**Recommended next phase:** `/research` (fingerprint fix option matrix + operator SQL gate)

---

