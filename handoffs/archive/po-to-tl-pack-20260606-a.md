# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## research-20260608-bug0008 — BUG-0008 subscription alerts research → architecture`
- Last archived heading: `## research-20260608-bug0008 — BUG-0008 subscription alerts research → architecture`
- Verification tuple (mandatory):
  - archived_body_lines=51
  - retained_body_lines=483

---

## research-20260608-bug0008 — BUG-0008 subscription alerts research → architecture

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-08  
**Bug:** BUG-0008  
**Orchestrator run:** auto-20260608-bug0008-001  
**Next phase:** `/architecture`

### Summary

Web + code research completed for BUG-0008 discovery open questions. Added **[R-0068](docs/engineering/research.md#r-0068--bug-0008-subscription-alert-dedup-unread-count-contract-orphan-lifecycle)** (alert dedup, unread-count API, orphan lifecycle, bell scope) and **[R-0069](docs/engineering/research.md#r-0069--bug-0008-detection-recall-levers--ai-path-boundary)** (recall matrix, AI boundary). Extended **R-0009–R-0013** with BUG-0008 addenda. Honored **[R-0065 § BUG-0008 coordinate](docs/engineering/research.md#bug-0008-coordination-do-not-merge-scope)** — no `list_patterns` filter changes; additive unread-count route only. No host `.env` or secrets read. Recommend architecture emit **DEC-0071** (W bundle: fingerprint dedup + unread-count contract + orphan hooks) and **DEC-0072** (X bundle: Phase 1 recall) with explicit **W-before-X** sequencing.

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| **1. Alert dedup contract** | **Lifecycle fingerprint dedup** per `(pattern_id, alert_type)` with partial unique index `WHERE read_at IS NULL`; `INSERT … ON CONFLICT DO UPDATE` (R-0023 pattern). Reject per-`sync_run_id` dedup. Upsert alert only on **new pending** or confidence tier increase. |
| **2. Unread count API** | Add **`GET /api/v1/subscriptions/alerts/unread-count`** with `{ unread_new_detection, pending_patterns, reconciled, … }`. Banner/toast consume this — not raw alert list length. Steady state: `unread_new_detection == pending_patterns`. |
| **3. Header bell scope** | **Keep US-0005-only badge** (R-0011/R-0023 boundary). Subscription unread on `/subscriptions` banner + existing popover link. Combined badge deferred. |
| **4. Recall levers** | **Phase 1:** SEPA payee normalization + transfer-type counterparty priority + `detection_window_days` 730. **Phase 2:** category-aware grouping. **Gate:** min_emit 55 / tolerance widen until W closed. Matrix in R-0069 §2. |
| **5. AI-assisted detection** | **Out of sync mutex** for MVP. BUG-0007 orchestrator does not feed detection. Optional async enrichment deferred to architecture gate. Acceptance **X** footer: document rule improvements; AI noted as future. |
| **6. Orphan/stale alerts** | Auto mark-read `new_detection` alerts on confirm/reject/inactive. One-time backfill dedupes 83→~6 live duplicates. |

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| W root cause | R-0068 §1 | Bare `insert_alert` every sync; patterns dedupe via fingerprint upsert — alerts do not |
| Count contract | R-0068 §2 | Reconciled semantics: actionable unread = pending-linked `new_detection`; not alert rows = list rows |
| Bell UX | R-0068 §3 | No header badge change; fix subscriptions-page banner source |
| X recall | R-0069 §2 | Normalization + category grouping before threshold tuning |
| Sequencing | R-0068 §5 + R-0069 §4 | **W dedup before X recall** — mandatory |
| Coordinate | R-0065 §4 | Additive API only; no REST list/alert filter regression |

### Risks (carried forward)

1. **X before W** — recall without dedup re-amplifies spam (83+ today).
2. **Migration backfill** — partial unique requires fingerprint backfill on existing alerts.
3. **Over-merge** — counterparty/category grouping false positives (R-0069 §5).
4. **Shared recurrence core** — normalization affects forecast — integration tests required.

### Evidence

- Research: [R-0068](docs/engineering/research.md#r-0068--bug-0008-subscription-alert-dedup-unread-count-contract-orphan-lifecycle), [R-0069](docs/engineering/research.md#r-0069--bug-0008-detection-recall-levers--ai-path-boundary), R-0009–R-0013 addenda, [R-0065 coordinate](docs/engineering/research.md#bug-0008-coordination-do-not-merge-scope)
- Code: `backend/src/subscriptions/detection.rs`, `repository.rs`, `backend/src/recurrence/group.rs`, `frontend/src/pages/SubscriptionsPage.tsx`, `frontend/src/components/AlertBell.tsx`
- Acceptance: `docs/product/acceptance.md` (BUG-0008 W/X, unchanged)
- Prior handoff: `#discovery-20260608-bug0008` (archive pack + state checkpoint)

---

