# QA Findings — Quick Q0018 / BUG-0008

**Work item:** BUG-0008 (defect)  
**Quick task:** Q0018  
**QA phase:** `/qa`  
**Date:** 2026-06-08  
**Orchestrator:** `auto-20260608-bug0008-001`  
**Decisions:** DEC-0071, DEC-0072  
**QA agent:** fresh subagent (`qa-20260608-q0018-bug0008`)  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — Acceptance rows **W** and **X** satisfied at code/test level. Zero blocking findings. Hand off to `/verify-work` (V1 omniflow runtime probes deferred until **BACKEND_FRONTEND_DEPLOY**).

## Scope

BUG-0008 subscription alert dedup/reconciliation (W bundle) and detection recall (X Phase 1) per `handoffs/dev_to_qa.md` and `sprints/quick/Q0018/summary.md`.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0018/summary.md`, `sprints/quick/Q0018/tasks.md`, `sprints/quick/Q0018/uat.md`, `backend/migrations/010_subscription_alert_fingerprint.sql`, `backend/src/subscriptions/{repository,detection,service}.rs`, `backend/src/api/subscriptions.rs`, `backend/src/recurrence/{normalize,group}.rs`, `backend/tests/bug0008_subscription_alerts.rs`, `frontend/src/pages/SubscriptionsPage.tsx`, `frontend/src/lib/api.ts`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | BUG-0008 contract suite | `cd backend && cargo test --test bug0008_subscription_alerts` | **PASS** (8/8) |
| T-2 | Full lib regression | `cd backend && cargo test --lib` | **PASS** (156/156) |
| T-3 | Subscriptions unit scope | `cargo test --lib subscriptions` | **PASS** (13/13) |
| T-4 | Recurrence unit scope | `cargo test --lib recurrence` | **PASS** (13/13) |
| T-5 | W — fingerprint migration + upsert dedup | Code review + `upsert_alert_dedupes_unread_fingerprints` | **PASS** |
| T-6 | W — emit gate (no resync spam) | Code review + `unchanged_resync_does_not_spam_alerts` | **PASS** |
| T-7 | W — unread-count reconciled API | Code review + `unread_count_api_reconciled_semantics` | **PASS** |
| T-8 | W — orphan lifecycle on confirm | Code review + `confirm_marks_read_orphan_alerts` | **PASS** |
| T-9 | W — frontend banner/toast wiring | Code review `SubscriptionsPage.tsx` | **PASS** |
| T-10 | X — SEPA + legal suffix normalization | Code review + `sepa_transfer_fixtures_merge_under_single_payee_key` + normalize unit tests | **PASS** |
| T-11 | X — transfer counterparty priority | Code review `group.rs` + `transfer_shaped_description_prefers_counterparty` | **PASS** |
| T-12 | X — 730-day detection window | Code review + `detection_window_defaults_to_730_days` | **PASS** |
| T-13 | X — forecast regression guard | `forecast_recurring_still_detects_after_normalization` | **PASS** |
| T-14 | V1 omniflow smoke (W/X live) | `sprints/quick/Q0018/uat.md` runtime probes | **DEFERRED** — verify-work after deploy |
| T-15 | User-visible metadata guard | `scripts/check-user-visible-metadata.py` | **SKIP** — entrypoint absent (same as execute) |
| T-16 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |

### Test output (T-1)

```
running 8 tests
test unread_count_api_reconciled_semantics ... ok
test detection_window_defaults_to_730_days ... ok
test unchanged_resync_does_not_spam_alerts ... ok
test confirm_marks_read_orphan_alerts ... ok
test upsert_alert_dedupes_unread_fingerprints ... ok
test subscription_detection_persists_pending_pattern ... ok
test forecast_recurring_still_detects_after_normalization ... ok
test sepa_transfer_fixtures_merge_under_single_payee_key ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Test output (T-2)

```
test result: ok. 156 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Acceptance criteria matrix (BUG-0008)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **W** | Reconciled unread-count; banner uses `unread_new_detection`; confirm/reject cleans orphans; no resync alert spam | **PASS** (code) / **DEFERRED** (runtime) | Migration `010_subscription_alert_fingerprint.sql` + partial unique index; `upsert_alert` ON CONFLICT; emit gate `emit_detection_alert` only on new pending or tier increase; `unread_alert_counts()` returns `reconciled` with orphan JOIN guard; `mark_read_unread_alerts_for_pattern` on confirm/reject/inactive; frontend banner/toast from unread-count API. Tests T-5–T-9 PASS. Live W-1–W-4 **DEFERRED** |
| **X** | Improved recall (>12 patterns post-deploy) without alert spam | **PASS** (code) / **DEFERRED** (runtime) | SEPA/legal normalization in `normalize.rs`; transfer counterparty guard in `group.rs`; `detection_window_days=730`; spam invariant via emit gate + fingerprint dedup. Tests T-10–T-13 PASS. Live X-1/X-2 **DEFERRED** |
| Regression | OIDC + bundled-firefly deploy smoke | **DEFERRED** | V1 checklist in `uat.md`; REG-1/REG-2 pending verify-work |

## Code review highlights

| Area | Finding | Severity |
|------|---------|----------|
| W1–W2 | Fingerprint column, backfill, dedupe, NOT NULL, partial unique index `subscription_alerts_unread_fingerprint` | **PASS** |
| W3 | `upsert_pending_pattern` sets `emit_detection_alert` only for new pending or confidence tier increase | **PASS** |
| W4 | `GET /api/v1/subscriptions/alerts/unread-count` with reconciled semantics and orphan guard | **PASS** |
| W5 | `confirm_pattern` / `reject_pattern` / `mark_inactive` call `mark_read_unread_alerts_for_pattern` | **PASS** |
| W6 | Banner count from `unread_new_detection`; toast uses sessionStorage delta; header bell unchanged | **PASS** |
| X1–X3 | Normalization, transfer guard, 730-day window in TOML + config default | **PASS** |

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

## Verify-work readiness

| Gate | Status |
|------|--------|
| Code QA (W1–W7, X1–X4) | **READY** |
| `cargo test --test bug0008_subscription_alerts` | **READY** — 8/8 PASS |
| `cargo test --lib` | **READY** — 156/156 PASS |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** |
| V1 omniflow smoke (W/X rows) | **PENDING** — blocked on deploy |

## Next phase

**`/verify-work`** — operator **BACKEND_FRONTEND_DEPLOY** then V1 omniflow probes per `sprints/quick/Q0018/uat.md`.

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
