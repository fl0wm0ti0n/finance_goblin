# Technical Specification — BUG-0008

## Overview

Backend-first fix: migration adds `subscription_alerts.fingerprint` with partial unique index; detection pipeline switches to idempotent upsert; new unread-count route; lifecycle hooks mark-read orphans. Recurrence core gains SEPA-aware normalization and transfer-type counterparty priority. Frontend `/subscriptions` consumes unread-count API.

**Sequencing:** W1–W3 before X1–X3 (mandatory).

## Components

| Component | Change | Decision |
|-----------|--------|----------|
| `subscription_alerts` schema | `fingerprint TEXT NOT NULL`; partial unique index | DEC-0071 |
| `subscriptions/repository.rs` | `upsert_alert` ON CONFLICT | DEC-0071 |
| `subscriptions/detection.rs` | Emit gate: new pending or tier increase | DEC-0071 |
| `subscriptions/routes.rs` | `GET .../alerts/unread-count` | DEC-0071 |
| `subscriptions/service.rs` | Orphan mark-read on confirm/reject/inactive | DEC-0071 |
| `recurrence/normalize.rs` | SEPA token strip, suffix collapse | DEC-0072 |
| `recurrence/group.rs` | Transfer-type counterparty priority | DEC-0072 |
| `config/default.toml` | `detection_window_days = 730` | DEC-0072 |
| `SubscriptionsPage.tsx` | Banner/toast from unread-count | DEC-0071 |

## Interfaces

### GET /api/v1/subscriptions/alerts/unread-count

**Response (200):**

```json
{
  "unread_total": 2,
  "unread_new_detection": 2,
  "unread_price_change": 0,
  "pending_patterns": 6,
  "reconciled": true,
  "reconciliation_note": "unread_new_detection counts pending patterns with unread new_detection alerts; price_change alerts are informational"
}
```

**Computation:**

- `unread_new_detection` = COUNT unread `new_detection` JOIN `subscription_patterns` WHERE status = `pending`
- `reconciled: true` when `unread_new_detection <= pending_patterns` AND no orphan unread alerts
- `pending_patterns` = COUNT patterns status `pending`

### Fingerprint formulas

See DEC-0071 §1 — `sub_alert:{type}:{pattern_id}[:suffix]`.

### Upsert SQL

```sql
INSERT INTO subscription_alerts (pattern_id, alert_type, title, body, sync_run_id, fingerprint)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (fingerprint) WHERE read_at IS NULL
DO UPDATE SET body = EXCLUDED.body, sync_run_id = EXCLUDED.sync_run_id, created_at = NOW();
```

### Normalization (Phase 1)

- `payee_key()`: strip `SVWZ+`, reference tokens, card suffixes; collapse `GmbH`, `AB`
- Transfer guard: if description matches `SVWZ|UEBERWEISUNG|Lastschrift`, use `counterparty_name` before memo

### Phase 2 (gated)

When ≥70% txs in group share `category_id`, add grouping key `cat:{category_id}` for weak payee keys.

## Non-functional

- **Sequencing:** W dedup deployed before X recall in sprint task order
- **Regression:** OIDC + bundled-firefly deploy smoke; forecast integration tests after recurrence changes
- **Coordinate:** No `list_patterns` filter changes; BUG-0007 AI JSON additive-only
- **Privacy:** No host secrets; no raw transaction export changes
- **Performance:** Partial unique index on unread fingerprints; backfill one-time in migration

## User guide

`docs/user-guides/BUG-0008.md` — operator-facing alert reconciliation and recall expectations.
