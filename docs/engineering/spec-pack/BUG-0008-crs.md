# CRS — BUG-0008 Subscription alerts vs list mismatch & under-detection

## Purpose

Close operator-reported defects **W** (subscription alert unread count diverges from `/subscriptions` list) and **X** (detection recall too low from 922+ synced transactions) without breaking US-0005 unified alert boundary or BUG-0007 AI coordinate table.

## Scope

### In scope

- **W (DEC-0071):** fingerprint dedup migration, `upsert_alert`, unread-count API, orphan lifecycle hooks, frontend banner/toast contract
- **X Phase 1 (DEC-0072):** payee normalization, transfer-type counterparty priority, `detection_window_days` 730
- **X Phase 2 (gated):** category-aware grouping ≥70% threshold — same sprint if capacity
- Integration tests for shared recurrence core (forecast regression guard)
- Operator verify on US-0010 external profile

### Out of scope

- Header bell badge merge with subscription unread
- AI-assisted in-pipeline detection
- `list_patterns` REST behavior changes
- min_emit_confidence 60 → 55 until W closed + FP review

## Acceptance criteria ref

`docs/product/acceptance.md` — BUG-0008:

- **(W)** Subscription-scoped alert unread count reconciles with visible `/subscriptions` list rows — reconciled semantics documented in UI
- **(X)** Detection surfaces materially more recurring patterns from 922+ txs without alert spam; rule improvements documented; AI deferred noted if not used

OIDC-enabled and bundled-firefly deploy regression checks pass.

## Dependencies

- US-0003 subscription engine (baseline)
- DEC-0071 before DEC-0072 execute slices
- R-0065 BUG-0008 coordinate (additive routes only)
