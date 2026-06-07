# Q0018 Summary — BUG-0008

**Sprint:** Q0018  
**Bug:** BUG-0008  
**Orchestrator:** `auto-20260608-bug0008-001`  
**Phase:** RELEASED  
**Last updated:** 2026-06-08 (refresh-context)

## Outcome

Implemented **DEC-0071** (W bundle) and **DEC-0072 Phase 1** (X recall) in W-before-X order. Backend migration adds alert `fingerprint` dedup; detection emit gate prevents resync spam; new unread-count API drives frontend banner/toast; orphan lifecycle hooks mark-read on confirm/reject/inactive; payee normalization + transfer counterparty priority + 730-day window improve recall without alert spam.

## Tasks

| ID | Status | Notes |
|----|--------|-------|
| W1 | DONE | `010_subscription_alert_fingerprint.sql` — column, backfill, dedupe, partial unique index |
| W2 | DONE | `upsert_alert` ON CONFLICT per DEC-0071 §1 |
| W3 | DONE | Emit gate: new pending or tier increase only |
| W4 | DONE | `GET /api/v1/subscriptions/alerts/unread-count` |
| W5 | DONE | Orphan mark-read on confirm/reject/inactive |
| W6 | DONE | Banner + toast from unread-count API |
| W7 | DONE | `bug0008_subscription_alerts.rs` + unit tests |
| X1 | DONE | SEPA token strip, legal suffix collapse in `normalize.rs` |
| X2 | DONE | Transfer guard counterparty priority in `group.rs` |
| X3 | DONE | `detection_window_days` 730 in TOML + config default |
| X4 | DONE | SEPA merge + forecast regression integration tests |
| V1 | DONE | UAT smoke checklist prepared — runtime probes pending deploy |

## Files changed

| Layer | Path |
|-------|------|
| Migration | `backend/migrations/010_subscription_alert_fingerprint.sql` |
| Backend | `backend/src/subscriptions/{repository,detection,service,types}.rs` |
| Backend | `backend/src/api/subscriptions.rs` |
| Backend | `backend/src/recurrence/{normalize,group}.rs` |
| Backend | `backend/src/config/mod.rs` |
| Backend | `backend/config/default.toml` |
| Backend tests | `backend/tests/bug0008_subscription_alerts.rs` |
| Frontend | `frontend/src/pages/SubscriptionsPage.tsx` |
| Frontend | `frontend/src/lib/api.ts` |
| UAT | `sprints/quick/Q0018/uat.md` |

## Tests run

| Command | Result |
|---------|--------|
| `cargo test --lib` | **PASS** (156 tests) |
| `cargo test --test bug0008_subscription_alerts` | **PASS** (8/8) |
| `cargo test --test subscriptions_integration` | **PASS** (1/1) |
| `python3 scripts/enforce-triad-hot-surface.py --check` | **PASS** (after rollover units=2) |
| `scripts/check-user-visible-metadata.py` | **SKIP** — script not present in repo |

## Governance

- **DEC-0071**, **DEC-0072**
- Triad rollover: `units=2` before state append
- Operator gate: **BACKEND_FRONTEND_DEPLOY** pending before V1 runtime probes

## Stop reason

`EXECUTE_COMPLETE` — hand off to `/qa`; do not begin QA in this subagent.
