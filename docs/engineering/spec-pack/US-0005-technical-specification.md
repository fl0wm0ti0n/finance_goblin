# Technical Specification — US-0005

## Overview

US-0005 adds **WealthService** and **AlertService** to the Rust/Axum backend, migration `005_alerts_wealth.sql`, wealth and alert REST endpoints, sync pipeline `"alerts"` phase, React `/wealth` and `/alerts` pages with header bell, Grafana Dashboard 4 (`uid: portfolio`), and Dashboard 1 scarcity threshold variable migration.

**Dependencies:** US-0001 mirror `accounts`; US-0002 `forecast_balance_daily` + `forecast_computations`; US-0003 `subscription_alerts` (boundary only); US-0004 active plan + category-targeted `plan_adjustments`.

## Components

### WealthService (`backend/src/wealth/`)

```
WealthService::upsert_daily_snapshot(sync_run_id)
  ├─ repository::load_asset_accounts()
  ├─ compute total + per-account breakdown + mixed_currency flag
  └─ repository::upsert_snapshot(date, payload)

WealthService::breakdown() -> NetWorthBreakdown
WealthService::history(days) -> Vec<WealthHistoryPoint>
```

Per DEC-0025 / R-0021: filter asset accounts with `include_net_worth=true`, exclude crypto, upsert daily snapshot.

### AlertService (`backend/src/alerts/`)

```
AlertService::run_post_sync(run_id, EvalContext)
  ├─ wealth.upsert_daily_snapshot(run_id)
  ├─ evaluate_scarcity(forecast_computation_id, config)
  ├─ evaluate_budget_drift(active_plan, transactions MTD, config)
  ├─ evaluate_plan_viability(active_plan_computation, config)
  └─ repository::upsert_or_resolve(fingerprint rules)
```

Per DEC-0026 / R-0022 evaluation rules. Fingerprint dedup per DEC-0027.

### Database migration `005_alerts_wealth.sql`

Per architecture.md §4:

- Enums: `alert_type`, `alert_severity`, `alert_status`
- Tables: `alert_config` (singleton), `alerts`, `net_worth_snapshots`
- Partial unique index: `alerts_active_fingerprint`
- Seed: `INSERT INTO alert_config DEFAULT VALUES`

### Sync integration

Extend `SyncService::execute_run` (DEC-0028):

```rust
// After forecast success (plan hook already awaited per DEC-0023):
self.set_phase("alerts").await;
if let Err(e) = self.alerts.run_post_sync(run_id, eval_context).await {
    tracing::warn!("alert evaluation failed: {e}");
}
// Clear mutex
```

Phase reporting: Sync Status UI shows "Evaluating alerts…"

### Config & threshold mirror

```toml
[alerts]
scarcity_threshold_eur = 200.0
budget_drift_pct = 20.0
```

Startup: load → `AlertsConfig` in memory + UPSERT `alert_config` (DEC-0029).

### REST API

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/wealth` | Breakdown + metadata |
| GET | `/api/v1/wealth/history?days=90` | Snapshot trend |
| GET | `/api/v1/alerts?status=active` | Inbox |
| GET | `/api/v1/alerts/unread-count` | Bell badge |
| PATCH | `/api/v1/alerts/{id}/acknowledge` | Mark read |
| PATCH | `/api/v1/alerts/{id}/dismiss` | Hide until clear/re-trigger |

### React UI

**`/wealth`** (DEC-0030):
- Enable Wealth nav
- Stat card, account table, mixed-currency banner, crypto placeholder
- Optional ECharts bar/line; Grafana Dashboard 4 link

**Header bell + `/alerts`** (DEC-0030):
- Bell with unread badge; Popover preview (5 latest)
- Full inbox page with Acknowledge/Dismiss
- Optional subscription alerts cross-link

### Grafana

**Dashboard 4:** `grafana/provisioning/dashboards/analytics/portfolio.json`
- uid `portfolio`, folder Analytics
- Panels: total wealth, account breakdown, wealth-over-time, mixed-currency warning, crypto placeholder

**Dashboard 1 update:** `$scarcity_threshold` variable from `alert_config`; replace hardcoded `200` in refId B (DEC-0029).

## Interfaces

| Consumer | Interface |
|----------|-----------|
| React | JWT Bearer → `/api/v1/wealth/*`, `/api/v1/alerts/*` |
| Grafana | PostgreSQL → `net_worth_snapshots`, `alert_config`, `accounts` |
| US-0006 (future) | `get_budget_status`, `get_portfolio` tools read wealth/alert API |

## Non-functional

- **Read-only Firefly:** No Firefly writes (DEC-0004)
- **Latency:** Alert pass ~100–500ms inline; monitor combined sync pipeline
- **Failure:** Alert failure non-blocking for sync success
- **Retention:** `net_worth_snapshots` optional 365-day prune
- **Auth:** JWT on all routes (DEC-0006)

## Verification

- Unit tests: scarcity/budget-drift/plan-viability evaluators; fingerprint dedup; dismiss/resolve lifecycle
- Integration test: sync trigger → snapshot upsert → alert row created when threshold breached
- Grafana: provision `portfolio.json`; verify Dashboard 1 variable
- User guide: `docs/user-guides/US-0005.md` (execute phase, USER_GUIDE_MODE=1)
