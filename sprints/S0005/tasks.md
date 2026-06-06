# Tasks — Sprint S0005

**Story:** US-0005  
**Task count:** 12 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0049 | SQLx migration 005 alerts wealth schema | done | AC-1, AC-2, AC-3, AC-4, AC-5 |
| T-0050 | Alert config and wealth/alert repositories | done | AC-2, AC-3, AC-4 |
| T-0051 | WealthService net worth aggregation and snapshots | done | AC-1, AC-6 |
| T-0052 | Alert Engine evaluate modules | done | AC-2, AC-3, AC-4 |
| T-0053 | AlertService orchestration and lifecycle | done | AC-2, AC-3, AC-4, AC-5 |
| T-0054 | Sync pipeline alerts phase | done | AC-2, AC-3, AC-4 |
| T-0055 | Wealth and alert REST API endpoints | done | AC-1, AC-2, AC-3, AC-4, AC-5 |
| T-0056 | React /wealth page | done | AC-1, AC-6 |
| T-0057 | React header bell and /alerts inbox | done | AC-5 |
| T-0058 | Grafana Dashboard 4 and Dashboard 1 threshold | done | AC-2, AC-6 |
| T-0059 | Alert and wealth tests | done | AC-1–AC-5 |
| T-0060 | Operator user guide | done | AC-1–AC-6 |

---

## T-0049 — SQLx migration 005 alerts wealth schema

**Status:** open  
**Depends on:** US-0001 (migration 001), US-0002 (migration 002), US-0003 (migration 003), US-0004 (migration 004)  
**Decisions:** DEC-0027, DEC-0029

### Description

Add SQLx migration `005_alerts_wealth.sql` per architecture § migration 005:

| Object | Purpose |
|--------|---------|
| `alert_type` enum | `scarcity`, `budget_drift`, `plan_viability` |
| `alert_severity` enum | `info`, `warning`, `critical` |
| `alert_status` enum | `active`, `acknowledged`, `dismissed`, `resolved` |
| `alert_config` | Singleton threshold mirror: `scarcity_threshold_eur`, `budget_drift_pct`, `updated_at` |
| `alerts` | Unified inbox: type, severity, status, fingerprint, title, message, entity refs, `context` JSONB, lifecycle timestamps |
| `net_worth_snapshots` | Daily wealth trend: `snapshot_date`, `total_eur`, `mixed_currency`, `account_count`, `payload` JSONB |

Indexes and constraints:

- `alerts_active_fingerprint` — partial unique on `fingerprint` WHERE `status IN ('active', 'acknowledged')`
- `alerts_status_triggered` — `(status, triggered_at DESC)`
- `alerts_unread` — partial on `(status, acknowledged_at)` WHERE `status = 'active' AND acknowledged_at IS NULL`
- `net_worth_snapshots_snapshot_date` — `UNIQUE(snapshot_date)` + index on `(snapshot_date DESC)`

Seed: `INSERT INTO alert_config DEFAULT VALUES;` (defaults: scarcity 200 EUR, budget_drift 20%).

### Done when

- [ ] Migration applies cleanly against external PostgreSQL with TimescaleDB extension
- [ ] All enums and tables match architecture schema
- [ ] Partial unique index enforces fingerprint dedup for active/acknowledged alerts
- [ ] `alert_config` singleton seeded with Projectplan defaults

---

## T-0050 — Alert config and wealth/alert repositories

**Status:** open  
**Depends on:** T-0049  
**Decisions:** DEC-0025, DEC-0027, DEC-0029

### Description

Implement config additions and repository layers:

**TOML `[alerts]` section:**

```toml
[alerts]
scarcity_threshold_eur = 200.0
budget_drift_pct = 20.0
reporting_currency = "EUR"

[wealth]
snapshot_retention_days = 365
```

**Startup (DEC-0029):** load TOML → in-memory `AlertsConfig` + UPSERT `alert_config` singleton row.

**`wealth::repository`:**

- `load_asset_accounts()` — filter `type = asset`, active, `include_net_worth = true`, `balance >= 0`
- `upsert_snapshot(date, total, mixed_currency, payload)` — daily upsert
- `fetch_history(days)` — trend series from `net_worth_snapshots`
- Optional retention prune on upsert per `[wealth].snapshot_retention_days`

**`alerts::repository`:**

- `upsert_or_resolve(fingerprint, alert row)` — dedup per partial unique index
- `resolve_by_fingerprint(fingerprint)` — mark resolved when condition clears
- `list(status filter)`, `unread_count()`, `acknowledge(id)`, `dismiss(id)`
- `fetch_config()` — read singleton for evaluators

Wire `WealthRepository` and `AlertRepository` into `AppState` construction prep.

### Done when

- [ ] Config loads alert/wealth settings from TOML + env overlay
- [ ] Startup mirrors TOML thresholds into `alert_config` row
- [ ] Wealth repository reads asset accounts and upserts/fetches snapshots
- [ ] Alert repository enforces fingerprint dedup and lifecycle transitions
- [ ] Dismiss sets status without deleting row; resolve clears active fingerprint slot

---

## T-0051 — WealthService net worth aggregation and snapshots

**Status:** open  
**Depends on:** T-0050  
**Decisions:** DEC-0025

### Description

Implement `wealth::service::WealthService`:

```rust
impl WealthService {
    pub async fn compute_breakdown(&self) -> Result<NetWorthBreakdown, WealthError>;
    pub async fn upsert_daily_snapshot(&self, sync_run_id: Uuid) -> Result<(), WealthError>;
    pub async fn history(&self, days: u32) -> Result<Vec<WealthHistoryPoint>, WealthError>;
}
```

**Breakdown logic (DEC-0025, R-0021):**

- Sum asset account balances for household headline in reporting currency (EUR default)
- If `COUNT(DISTINCT currency) > 1` → `mixed_currency = true` (no FX conversion)
- Per-account rows: name, role label, currency, balance, % of total (same-currency group when mixed)
- Crypto excluded from total; expose `crypto_placeholder: true` for React static row
- Metadata: `reporting_currency`, `last_successful_sync_at`, `computed_at`

**Daily snapshot:** upsert one row per calendar day; `payload` JSONB stores per-account breakdown array for trend chart + Grafana.

Read-only toward Firefly mirrors (DEC-0004).

### Done when

- [ ] `compute_breakdown` returns correct total and per-account rows from mirror
- [ ] `mixed_currency` flag set when multiple currencies present
- [ ] `upsert_daily_snapshot` persists snapshot with breakdown payload
- [ ] `history` returns ordered series for configurable lookback
- [ ] Unit tests cover mixed-currency flag and asset account filter

---

## T-0052 — Alert Engine evaluate modules

**Status:** open  
**Depends on:** T-0050, US-0002 forecast snapshots, US-0004 active plan  
**Decisions:** DEC-0026

### Description

Implement pure evaluation logic under `alerts::evaluate`:

**`evaluate_scarcity(forecast_computation_id, config)`:**

- Sum `forecast_balance_daily.balance` across asset accounts per projected day
- Breach when any day in `[today, today+45d]` OR current-month month-end < `scarcity_threshold_eur`
- Severity: `critical` if breach tomorrow or current balance already below; else `warning`
- Fingerprint: `scarcity:household:{YYYY-MM}`
- Emit title/message with threshold and earliest breach date

**`evaluate_budget_drift(active_plan, transactions MTD, config)`:**

- Load active plan latest version adjustments where `target_type = category` only
- MTD actual = sum expense amounts (abs) for category in current month from mirror
- MTD target = prorated monthly plan delta (`monthly × days_elapsed / days_in_month`)
- Fire when `actual > target × (1 + budget_drift_pct/100)`
- Fingerprint: `budget_drift:category:{firefly_id}:{YYYY-MM}`

**`evaluate_plan_viability(active_plan_computation, config)`:**

- Primary: `planned_balance` at current month-end < 0 (household from plan overlay)
- Secondary: current + next month-end both < 0
- Skip when no active plan
- Fingerprint: `plan_viability:{plan_id}:{version_id}`

Return `Vec<AlertCandidate>` with type, severity, fingerprint, title, message, entity refs, `context` JSONB (computation IDs for stale detection).

### Done when

- [ ] Unit tests cover scarcity breach at threshold boundary and severity tiers
- [ ] Budget drift skips non-category-targeted adjustments
- [ ] Budget drift proration math tested for mid-month scenarios
- [ ] Plan viability skips when no active plan; fires on negative month-end balance
- [ ] Each evaluator returns stable fingerprint strings per architecture

---

## T-0053 — AlertService orchestration and lifecycle

**Status:** open  
**Depends on:** T-0051, T-0052  
**Decisions:** DEC-0027, DEC-0026

### Description

Implement `alerts::service::AlertService`:

```rust
impl AlertService {
    pub async fn run_post_sync(&self, run_id: Uuid, ctx: EvalContext) -> Result<AlertEvalResult, AlertError>;
    pub async fn list(&self, filter: AlertListFilter) -> Result<Vec<AlertRow>, AlertError>;
    pub async fn unread_count(&self) -> Result<u32, AlertError>;
    pub async fn acknowledge(&self, id: Uuid) -> Result<(), AlertError>;
    pub async fn dismiss(&self, id: Uuid) -> Result<(), AlertError>;
}
```

**`run_post_sync` orchestration:**

1. `WealthService::upsert_daily_snapshot(run_id)`
2. Run scarcity, budget drift, plan viability evaluators with `EvalContext { forecast_computation_id, plan_computation_id, config }`
3. Upsert active/acknowledged rows per fingerprint; resolve when condition clears
4. Re-breach after resolve creates new row (DEC-0027 dismiss semantics)

**Lifecycle rules:**

| Status | Behavior |
|--------|----------|
| `active` | Surfaces in bell + `/alerts`; unread if `acknowledged_at IS NULL` |
| `acknowledged` | Muted styling; excluded from unread count |
| `dismissed` | Hidden from bell + active list while condition persists |
| `resolved` | Condition cleared; historical only |

**Dismiss (DEC-0027):** hide until condition clears or re-triggers — not permanent suppress.

Bind `forecast_computation_id` and `plan_computation_id` in alert `context` for stale metadata.

### Done when

- [ ] `run_post_sync` upserts snapshot then evaluates all three alert types
- [ ] Fingerprint dedup prevents duplicate active rows for same condition
- [ ] Condition clear resolves existing row; re-breach creates new alert
- [ ] `acknowledge` and `dismiss` transition status correctly
- [ ] `unread_count` counts only `active AND acknowledged_at IS NULL`
- [ ] Unit tests cover dedup, dismiss-until-clear, and resolve lifecycle

---

## T-0054 — Sync pipeline alerts phase

**Status:** open  
**Depends on:** T-0053  
**Decisions:** DEC-0028, DEC-0010, DEC-0023

### Description

Extend `SyncService::execute_run` per DEC-0028:

```rust
// After forecast success (plan hook already awaited per DEC-0023):
self.set_phase("alerts").await;
let eval_context = EvalContext {
    forecast_computation_id: latest_forecast_id,
    plan_computation_id: active_plan_latest_computation,
    config: self.alerts.config(),
};
if let Err(e) = self.alerts.run_post_sync(run_id, eval_context).await {
    tracing::warn!("alert evaluation failed: {e}");
}
// Clear mutex
```

**Pipeline order:**

```
sync → subscriptions → forecast (+ plan hook) → alerts → done
```

Rules:

- Inline in same Tokio task (DEC-0010 precedent)
- Alert failure **non-blocking** — sync run still succeeds; preserve last alert/snapshot state
- Sync Status UI shows phase `"Evaluating alerts…"`
- Log alerts phase duration separately for mutex monitoring

Do **not** migrate subscription alerts into unified inbox (US-0003 boundary).

### Done when

- [ ] Successful sync runs alerts phase after forecast+plan hook
- [ ] Alert evaluation failure logged as warning; sync run still succeeds
- [ ] Sync Status API reports `"alerts"` phase during evaluation
- [ ] `EvalContext` receives latest forecast and active plan computation IDs
- [ ] Alerts phase timing logged separately from forecast duration

---

## T-0055 — Wealth and alert REST API endpoints

**Status:** open  
**Depends on:** T-0053  
**Decisions:** DEC-0006, DEC-0030, DEC-0004

### Description

Add JWT-protected Axum handlers under `api::wealth` and `api::alerts`:

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/wealth` | Net worth breakdown + metadata (`mixed_currency`, `reporting_currency`, crypto placeholder) |
| GET | `/api/v1/wealth/history?days=90` | Trend series from `net_worth_snapshots` |
| GET | `/api/v1/alerts?status=active` | Inbox list (excludes dismissed unless `?include_dismissed=true`) |
| GET | `/api/v1/alerts/unread-count` | Header bell badge |
| PATCH | `/api/v1/alerts/{id}/acknowledge` | Mark read |
| PATCH | `/api/v1/alerts/{id}/dismiss` | Suppress until clear/re-trigger |

Optional stretch: `POST /api/v1/alerts/evaluate` for operator manual re-eval.

Subscription alerts remain at `/api/v1/subscriptions/alerts/*` (US-0003). Read-only toward Firefly (DEC-0004).

Response DTOs include stale metadata when bound computation IDs lag latest forecast/plan.

### Done when

- [ ] All six core endpoints registered and JWT-protected
- [ ] Wealth breakdown returns per-account rows and mixed-currency flag
- [ ] Alert list excludes dismissed by default; unread-count matches repository logic
- [ ] Acknowledge and dismiss return updated alert row
- [ ] No Firefly client usage in wealth/alerts handlers

---

## T-0056 — React /wealth page

**Status:** open  
**Depends on:** T-0055  
**Decisions:** DEC-0030, DEC-0025

### Description

Enable Wealth nav item (replace US-0001 disabled placeholder). Add `/wealth` route with shadcn layout:

| UI element | Implementation |
|------------|----------------|
| Overview tab | Net worth stat card (EUR headline); mixed-currency warning `Alert` banner |
| Account breakdown | Table: name, role label, currency, balance, % of total |
| Crypto placeholder | Static row "Connect exchanges — US-0007" (`included_in_total=false`) |
| Optional chart | ECharts stacked bar by account type OR wealth-over-time line from `/wealth/history` |
| Grafana link | External link to `{GRAFANA_URL}/d/portfolio` |
| Stale badge | When `last_successful_sync_at` lags |

Wire TanStack Query hooks for wealth breakdown and history with bearer token. Lazy-load ECharts if chart included.

### Done when

- [ ] Wealth nav enabled and `/wealth` route reachable when authenticated
- [ ] Stat card shows household total from API
- [ ] Mixed-currency banner renders when `mixed_currency=true`
- [ ] Account breakdown table populated; crypto placeholder row visible
- [ ] Grafana Dashboard 4 link present
- [ ] Stale badge shown when sync metadata indicates lag

---

## T-0057 — React header bell and /alerts inbox

**Status:** open  
**Depends on:** T-0055, T-0056  
**Decisions:** DEC-0030, DEC-0027

### Description

Extend app shell and add `/alerts` route:

| UI element | Implementation |
|------------|----------------|
| Header bell | `Bell` icon + unread badge from `/api/v1/alerts/unread-count` |
| Popover preview | Latest 5 active alerts; link to full `/alerts` |
| `/alerts` page | Table: type icon, severity badge, title, message, `triggered_at`; Acknowledge + Dismiss buttons |
| Subscription cross-link | Optional "View subscription alerts (N)" in popover when subscription unread > 0 (read-only) |
| Empty state | "No active alerts" with last sync timestamp |
| Polling | TanStack Query on unread-count; refresh after sync completes |

**Unread:** `status=active AND acknowledged_at IS NULL`. Dismissed excluded from bell.

Acknowledged-but-active alerts: muted styling with copy "Acknowledged — condition still active".

Do **not** migrate US-0003 subscription alerts into this inbox.

### Done when

- [ ] Header bell shows unread badge count from API
- [ ] Popover preview lists latest 5 alerts with link to `/alerts`
- [ ] `/alerts` page lists active alerts with Acknowledge and Dismiss actions
- [ ] Dismissed alerts hidden from bell and default list
- [ ] Optional subscription alerts cross-link when unread subscription alerts exist
- [ ] Empty state renders when no active alerts

---

## T-0058 — Grafana Dashboard 4 and Dashboard 1 threshold

**Status:** open  
**Depends on:** T-0049, T-0051  
**Decisions:** DEC-0029, DEC-0030, DEC-0012, R-0026

### Description

**Dashboard 4 — `grafana/provisioning/dashboards/analytics/portfolio.json`:**

| Panel | Type | Query notes |
|-------|------|-------------|
| Total wealth stat | stat | Latest `net_worth_snapshots.total_eur` |
| Account count | stat | Latest snapshot `account_count` |
| Mixed-currency warning | text | Annotation when latest snapshot `mixed_currency=true` |
| Account breakdown | table | Per-account rows from latest snapshot `payload` JSONB |
| Wealth over time | time series | `net_worth_snapshots.total_eur` filtered by `$__timeFilter` |
| Crypto placeholder | text | Static "Connect exchanges — US-0007" |
| Optional pie | piechart | Breakdown by `account_role` from payload |

uid `portfolio`, folder Analytics, `"id": null`. Reuse datasource uid `FlowFinancePostgreSQL`.

**Dashboard 1 update — `cashflow.json` (DEC-0029):**

- Add template variable `$scarcity_threshold` query:
  ```sql
  SELECT scarcity_threshold_eur AS __value, 'Scarcity threshold (€)' AS __text
  FROM alert_config WHERE id = 1;
  ```
- Replace hardcoded `200` in refId B scarcity reference line with `$scarcity_threshold::numeric`
- Supersedes DEC-0012 static hardcode; stable dashboard uid unchanged

Platform Health, Dashboards 2, 3, 5 unchanged except Dashboard 1 variable.

### Done when

- [ ] Dashboard 4 loads with uid `portfolio` in Analytics folder
- [ ] Total wealth and wealth-over-time panels query `net_worth_snapshots` correctly
- [ ] Account breakdown table renders from snapshot payload
- [ ] Mixed-currency and crypto placeholder panels present
- [ ] Dashboard 1 scarcity line uses `$scarcity_threshold` from `alert_config`
- [ ] Changing TOML threshold and restarting backend updates Dashboard 1 line after config mirror

---

## T-0059 — Alert and wealth tests

**Status:** open  
**Depends on:** T-0052, T-0053, T-0054, T-0055, T-0057, T-0058  
**Decisions:** DEC-0026, DEC-0027, DEC-0028, DEC-0004

### Description

Add Rust unit tests for:

- Scarcity: threshold boundary, severity tiers, household aggregate path
- Budget drift: category-targeted only, MTD proration, skip untargeted categories
- Plan viability: negative month-end, skip without active plan
- Fingerprint dedup: upsert same fingerprint, resolve on clear, re-breach new row
- Dismiss/acknowledge lifecycle and unread count logic
- WealthService: mixed-currency flag, asset filter, snapshot upsert

Add integration test with fixture mirror + forecast + plan data:

- Trigger sync (or call `run_post_sync`) → snapshot upsert → alert row created when threshold breached
- Extend `tests/run-tests.sh` to include wealth/alert test targets
- Integration test skips without `DATABASE_URL` (same pattern as US-0001–US-0004)

Add read-only audit assertion: no Firefly write paths in wealth/alerts modules (DEC-0004).

Optional: verify Dashboard 1 JSON contains `$scarcity_threshold` variable (static file check).

### Done when

- [ ] Unit tests pass for all three evaluators and lifecycle logic
- [ ] WealthService unit tests cover mixed-currency and asset filter
- [ ] Integration test validates sync → snapshot → alert path (or SKIP without DATABASE_URL)
- [ ] Sync alerts phase covered by unit or integration test
- [ ] `bash tests/run-tests.sh` includes wealth/alert tests and passes
- [ ] No Firefly write operations in wealth/alerts code paths verified

---

## T-0060 — Operator user guide

**Status:** open  
**Depends on:** T-0056, T-0057, T-0058, T-0059  
**Decisions:** —

### Description

Create `docs/user-guides/US-0005.md` per USER_GUIDE_MODE=1:

- Prerequisites: sync + forecast recompute; active plan with category adjustments for budget drift
- Net worth view: account breakdown, mixed-currency banner, crypto placeholder (US-0007)
- Alert types: scarcity, budget drift, plan viability — what triggers each
- Threshold config: TOML `[alerts]` `scarcity_threshold_eur`, `budget_drift_pct`; Dashboard 1 line follows same config
- Header bell and `/alerts` inbox: acknowledge vs dismiss semantics
- US-0003 boundary: subscription alerts remain on `/subscriptions`; optional cross-link
- Grafana Dashboard 4 (`portfolio`) and wealth-over-time panel
- Read-only guarantee: wealth from Firefly mirror; alerts stored in Flow DB

### Done when

- [ ] User guide covers all six acceptance criteria from operator perspective
- [ ] Acknowledge vs dismiss semantics documented clearly
- [ ] Threshold config and Dashboard 1 centralization documented
- [ ] US-0003 subscription alert boundary documented
- [ ] Grafana Dashboard 4 access documented

---

## Execution order (recommended)

1. **Database:** T-0049 → T-0050
2. **Engine:** T-0051 → T-0052 → T-0053
3. **Integration:** T-0054 → T-0055
4. **Frontend (after T-0055):** T-0056 → T-0057
5. **Grafana (parallel after T-0051):** T-0058
6. **Verification:** T-0059 → T-0060

## Acceptance coverage map

| AC | Tasks |
|----|-------|
| Net worth aggregation | T-0049, T-0050, T-0051, T-0055, T-0056, T-0058, T-0059, T-0060 |
| Scarcity alert | T-0049, T-0050, T-0052, T-0053, T-0054, T-0055, T-0058, T-0059, T-0060 |
| Budget drift alert | T-0049, T-0050, T-0052, T-0053, T-0054, T-0055, T-0059, T-0060 |
| Plan viability alert | T-0049, T-0050, T-0052, T-0053, T-0054, T-0055, T-0059, T-0060 |
| Alert inbox + acknowledge/dismiss | T-0049, T-0050, T-0053, T-0055, T-0057, T-0059, T-0060 |
| Grafana Dashboard 4 total wealth | T-0049, T-0051, T-0056, T-0058, T-0060 |
