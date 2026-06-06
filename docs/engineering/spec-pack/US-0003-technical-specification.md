# Technical Specification — US-0003

## Overview

US-0003 adds a **Subscription Engine** to the Rust/Axum backend, migration 003 for subscription lifecycle tables, subscription REST endpoints, sync pipeline extension (subscriptions phase before forecast), forecast override hook, React `/subscriptions` page, and Grafana Dashboard 2. Detection runs inline after successful Firefly sync (DEC-0018, extends DEC-0010).

**Dependencies:** US-0001 mirror tables + sync scheduler; US-0002 forecast engine + `forecast/recurring.rs` (refactored to shared `recurrence` module).

## Components

### Shared recurrence core (`backend/src/recurrence/`)

Extract from `forecast/recurring.rs` per DEC-0013:

```
recurrence::detect(transactions, config) -> Vec<RecurrenceGroup>
  ├─ normalize::payee_key(description)     // strip trailing codes
  ├─ group::by_payee(expense_txs)            // amount < 0; exclude transfers
  ├─ cadence::median_interval + is_stable
  ├─ amount::median + within_tolerance
  └─ score confidence tier (60|80|95)        // DEC-0014
```

`forecast/recurring.rs` wraps to `Vec<RecurringPattern>` for projection (behavior-preserving refactor).

### Subscription Engine (`backend/src/subscriptions/`)

```
SubscriptionService::run_detection(sync_run_id)
  ├─ repository::load_transactions(window=365d)
  ├─ recurrence::detect → candidate groups
  ├─ classify::kind(group) → subscription | standing_order   // DEC-0016
  ├─ repository::upsert_patterns (skip rejected fingerprints) // DEC-0015
  ├─ for confirmed patterns:
  │    ├─ price_change::check_and_record
  │    └─ repository::insert_alert if material change         // DEC-0017
  ├─ repository::mark_inactive (gap > 2× interval)
  └─ return DetectionResult { confirmed_recurring, rejected_fingerprints }
```

| Submodule | Responsibility |
|-----------|----------------|
| `classify` | Dauerauftrag rules + config category/payee patterns |
| `detection` | Orchestration |
| `price_change` | Dual threshold; interval change; append billing events |
| `repository` | SQLx CRUD |
| `service` | Public API for sync + handlers |
| `types` | DTOs, `ConfirmedRecurring`, `DetectionResult` |

### Database migration `003_subscriptions.sql`

Per DEC-0015 / R-0012 (see architecture.md for full DDL):

- Enums: `subscription_status`, `subscription_kind`, `subscription_alert_type`
- Tables: `subscription_patterns`, `subscription_pattern_transactions`, `subscription_price_events`, `subscription_rejections`, `subscription_alerts`
- Indexes on status/kind, last_seen_at, price events, alerts

### Sync integration

Modify `SyncService::execute_run` (`backend/src/sync/mod.rs`):

```rust
// After successful Firefly sync:
*self.phase.lock().await = Some("subscriptions".into());
let detection = self.subscriptions.run_detection(run_id).await;
let detection_result = detection.ok(); // warn on failure, continue

*self.phase.lock().await = Some("forecast".into());
if let Err(e) = self.forecast.recompute(run_id, detection_result.as_ref()).await {
    warn!(?e, "forecast recompute failed; serving stale snapshot");
}
```

`AppState` adds `subscriptions: SubscriptionService`.

### Forecast override

Modify `ForecastService::recompute` and `project_account`:

```rust
// In project_account:
let mut recurring = detect_patterns(&non_transfer, config.tolerance);
if let Some(ctx) = subscription_context {
    recurring = apply_subscription_override(recurring, &ctx.confirmed_recurring);
    recurring = exclude_rejected(recurring, &ctx.rejected_fingerprints);
}
```

Confirmed patterns replace heuristic match on same `payee_key`. Rejected fingerprints excluded entirely.

### Backend API routes

| Method | Path | Handler |
|--------|------|---------|
| GET | `/api/v1/subscriptions` | List patterns (`status`, `kind` query) |
| GET | `/api/v1/subscriptions/{id}` | Pattern detail |
| POST | `/api/v1/subscriptions/{id}/confirm` | Confirm (+ optional kind) |
| POST | `/api/v1/subscriptions/{id}/reject` | Reject + store fingerprint |
| GET | `/api/v1/subscriptions/{id}/price-history` | Price events for ECharts |
| GET | `/api/v1/subscriptions/alerts` | Alerts (`unread=true`) |
| PATCH | `/api/v1/subscriptions/alerts/{id}/read` | Mark read |

All routes: JWT middleware (existing).

### Frontend (`frontend/src/pages/SubscriptionsPage.tsx`)

| Element | Detail |
|---------|--------|
| Route | `/subscriptions` — enable nav in `AppLayout.tsx` |
| Tabs | All \| Pending review \| Standing orders |
| Pending | Card + Confirm/Reject; confidence Badge |
| Confirmed | Table with interval, amount, kind |
| Detail | Sheet + ECharts price history line |
| Alerts | Banner (unread) + toast on sync poll |
| Data | TanStack Query → subscription API |

### Grafana provisioning

Add `grafana/provisioning/dashboards/analytics/subscriptions.json`:

- uid: `subscriptions`, folder: `Analytics`
- Stat: confirmed count, monthly spend (interval-normalized SQL), pending count
- Table: price changes (90d, before/after columns)
- Time series: daily new detections by `created_at`
- Datasource uid: `FlowFinancePostgreSQL` (DEC-0012)

### Config (TOML)

```toml
[subscriptions]
detection_window_days = 365
full_rescan_interval_days = 7
price_change_min_eur = 1.0
price_change_min_pct = 5.0
inactive_grace_days = 5
standing_order_category_patterns = ["rent", "miete", "insurance", "versicherung", "utilities", "nebenkosten", "loan", "darlehen"]
standing_order_payee_patterns = []

[subscriptions.confidence_tolerance_pct]
high = 5
medium = 10
low = 15
```

## Non-functional

| Concern | Target |
|---------|--------|
| Detection latency | Inline in sync mutex; monitor combined sync+detection+forecast; queue if > ~30s |
| False positives | Confirm/reject before forecast override; 60% minimum emit |
| Firefly guarantee | No Firefly writes; subscription state in Flow DB only |
| Test coverage | Unit tests for recurrence extraction, confidence scoring, classify, price_change; integration test with fixture patterns |

## Traceability

| AC | Component |
|----|-----------|
| Confidence tiers | `recurrence` scoring + DEC-0014 |
| Confirm/reject notification | `subscription_alerts` + React pending cards |
| Confirmed list | GET `/subscriptions?status=confirmed` |
| Standing orders | `classify` + tab filter `kind=standing_order` |
| Price change | `price_change` + price-history API + drawer chart |
| Alerts | `subscription_alerts` + banner/toast |
| Grafana Dashboard 2 | `subscriptions.json` |
| Rejected forecast exclusion | forecast override hook + `subscription_rejections` |

## References

- Architecture: `docs/engineering/architecture.md` — US-0003
- Research: R-0009 … R-0014
- Decisions: DEC-0013 … DEC-0018
