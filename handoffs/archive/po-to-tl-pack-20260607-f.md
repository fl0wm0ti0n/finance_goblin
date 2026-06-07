# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## intake-20260607-bug0015 — BUG-0015 confirmed subscriptions reappear after rebuild`
- Last archived heading: `## intake-20260607-bug0015 — BUG-0015 confirmed subscriptions reappear after rebuild`
- Verification tuple (mandatory):
  - archived_body_lines=53
  - retained_body_lines=496

---

## intake-20260607-bug0015 — BUG-0015 confirmed subscriptions reappear after rebuild

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-07  
**Bug:** BUG-0015  
**Intake run:** `intake-20260607-subscription-reconfirm`  
**Next phase:** `/research` or `/discovery`

### Summary

Operator reports **previously confirmed** subscriptions (Cursor €17.18/mo, Apple €9.99/mo) **reappearing as pending** with Confirm/Reject after post-**BUG-0014** container rebuild on `financegnome.omniflow.cc`. P1 data-integrity / operator-annoyance defect on **US-0003** confirm contract.

### Acceptance (open)

| Row | Contract |
|-----|----------|
| **AU** | Confirmed patterns stay confirmed through rebuild + Full sync — no re-prompt for Cursor/Apple |
| **AV** | Detection rerun skips confirmed fingerprints; no duplicate pending for same merchant identity |
| **AW** | Subscription alert unread reconciles with list — no spurious `new_detection` for confirmed merchants |

### Intake hypotheses (discovery)

| ID | Hypothesis | AC |
|----|------------|-----|
| **H1** | Fingerprint drift (`payee_key` / amount / interval → new fingerprint) | AU, AV |
| **H2** | DB ephemeral — postgres volume lost on rebuild | AU |
| **H3** | Alert/UI desync (confirmed in DB, pending in UI) | AW |
| **H4** | Detection re-run post-sync with changed grouping | AV |

### Code surfaces (delegation)

- `backend/src/subscriptions/repository.rs` — `upsert_pending_pattern` preserves `confirmed` on same fingerprint only
- `backend/src/subscriptions/detection.rs` — `confirmed_fps` skip before emit
- `compute_fingerprint(payee_key, interval_days, median_amount)` — drift creates new pending row

### Operator gates (discovery)

1. Confirm Cursor + Apple → record fingerprints
2. Rebuild app containers (not postgres) per **BUG-0014** pattern
3. Check `subscription_patterns` status **before** Full sync
4. Full sync → re-check statuses + alert rows

### Evidence

- Intake: `handoffs/intake_evidence/intake-20260607-subscription-reconfirm.json` (validation **PASS**)
- Research: [R-0081](docs/engineering/research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild)
- Related: **BUG-0008** DONE (alert dedup — coordinate H3), **BUG-0014** DONE (rebuild context)

**Recommended next phase:** `/discovery` (rank H1–H4 with DB probes before architecture)

---

