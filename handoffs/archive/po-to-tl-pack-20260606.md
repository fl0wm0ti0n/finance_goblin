# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## discovery-20260608-bug0008 — BUG-0008 subscription alerts discovery → research`
- Last archived heading: `## discovery-20260608-bug0008 — BUG-0008 subscription alerts discovery → research`
- Verification tuple (mandatory):
  - archived_body_lines=57
  - retained_body_lines=483

---

## discovery-20260608-bug0008 — BUG-0008 subscription alerts discovery → research

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-08  
**Bug:** BUG-0008  
**Orchestrator run:** auto-20260608-bug0008-001  
**Next phase:** `/research`

### Summary

Discovery confirmed both sub-defects on the US-0010 external profile. **W:** subscription-scoped unread alerts (**83** live; operator **33**) far exceed visible list rows (**6 pending** live; operator **11**) because `subscription_alerts` inserts on every detection pass with **no dedup** while patterns dedupe via fingerprint upsert. **X:** only **12** total patterns from **922+** transactions — detection gates (≥3 txs, 60% confidence, 365-day window, payee-only grouping) limit recall; AI enrichment is not wired into the detection pipeline. Vision and backlog BUG-0008 blocks updated; acceptance W/X unchanged. Coordinate with [R-0065 BUG-0008 isolation](docs/engineering/research.md#bug-0008-coordination-do-not-merge-scope) — BUG-0007 closed; REST list/alert semantics are BUG-0008 scope.

### Confirmed findings (carry to research)

| Item | Resolution |
|------|------------|
| W root cause | Alert row accumulation without dedup (`detection.rs` → `insert_alert` every sync per group) |
| W UI contract gap | Banner counts unread alert rows; list tabs count pattern rows — no reconciled unread-count API |
| W header bell | Unified `/api/v1/alerts/unread-count` = **0** on probe; subscription unread on `/subscriptions` banner only |
| X root cause | Hard detection gates + payee-only grouping; category signal unused; 12 patterns vs operator expectation |
| X AI path | Not in detection pipeline — optional research track, not BUG-0007 chat surface |
| Prior research | [R-0009](docs/engineering/research.md#r-0009--subscription-detection-engine-patterns--confidence-scoring)–[R-0013](docs/engineering/research.md#r-0013--post-sync-subscription-detection-pipeline--forecast-integration) detection schema/pipeline; [R-0011](docs/engineering/research.md#r-0011--subscription-price-change-detection--alert-thresholds) alert thresholds |
| Coordinate | [R-0065 § BUG-0008](docs/engineering/research.md#bug-0008-coordination-do-not-merge-scope) — no alert/list/detection threshold changes via BUG-0007 paths |

### Open questions for `/research`

1. **Alert dedup contract** — per `(pattern_id, alert_type, sync_run_id)` vs fingerprint lifecycle dedup vs mark-read on pattern confirm/reject; impact on acceptance **W**.
2. **Unread count API** — dedicated `GET /api/v1/subscriptions/alerts/unread-count` vs derive from deduped list; align with tab contract (pending + confirmed + standing orders per US-0003).
3. **Header bell scope** — include subscription unread in badge total vs keep US-0005-only badge with subscriptions-page banner (UX consistency).
4. **Recall levers** — threshold tuning (`min_emit_confidence`, tolerance pct), payee normalization improvements, category-aware grouping, standing-order split — compare false-positive risk vs **X** acceptance.
5. **AI-assisted detection** — in-pipeline enrichment vs post-detection async pass vs out-of-scope; document if used per acceptance **X** footer.
6. **Orphan/stale alerts** — behavior when patterns rejected/inactive but unread `new_detection` alerts remain.

### Risks (from intake, refined at discovery)

1. Fixing **X** recall without **W** dedup amplifies alert spam (83+ unread today).
2. Changing `SubscriptionService::list_patterns` REST behavior may regress BUG-0007 AI additive JSON — keep REST changes scoped per coordinate table.
3. Operator counts drift (11→6 pending) as patterns confirm/reject — acceptance must use reconciled semantics, not static numbers.

### Recommended next steps

1. `/research` — new **R-xxxx** entry: alert dedup + count contract + recall option matrix; extend R-0009/R-0011/R-0013 as needed.
2. `/architecture` — DEC for chosen W fix (dedup + count API) and X fix bundle; BUG-0008 coordinate table update if shared service touched.

### Evidence

- Intake: `handoffs/intake_evidence/intake-20260605-subscription-alerts-detection.json`
- Vision: `docs/product/vision.md` (Discovery notes BUG-0008, 2026-06-08)
- Backlog: `docs/product/backlog.md#BUG-0008` (#### Discovery notes 2026-06-08)
- Acceptance: `docs/product/acceptance.md` (BUG-0008 W/X, unchanged)
- Code: `backend/src/subscriptions/detection.rs`, `repository.rs`, `frontend/src/pages/SubscriptionsPage.tsx`, `frontend/src/components/AlertBell.tsx`
- Live probe: omniflow public API 2026-06-08 (6 pending, 83 unread alerts, 12 patterns, 0 unified unread)
- Research: [R-0009](docs/engineering/research.md#r-0009--subscription-detection-engine-patterns--confidence-scoring)–[R-0013](docs/engineering/research.md#r-0013--post-sync-subscription-detection-pipeline--forecast-integration), [R-0065 coordinate](docs/engineering/research.md#bug-0008-coordination-do-not-merge-scope)

---

