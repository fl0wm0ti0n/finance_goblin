# Tasks — Sprint S0003

**Story:** US-0003  
**Task count:** 12 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0025 | Extract shared recurrence core from forecast | open | AC-1 |
| T-0026 | SQLx migration 003 subscriptions schema | open | AC-3, AC-4, AC-6 |
| T-0027 | Subscription repository and config | open | AC-3, AC-5, AC-6 |
| T-0028 | Subscription engine classify, detection, price_change | open | AC-1, AC-4, AC-5, AC-6 |
| T-0029 | SubscriptionService run_detection orchestration | open | AC-1, AC-2, AC-6 |
| T-0030 | Sync pipeline subscriptions phase hook | open | AC-2, AC-6, AC-8 |
| T-0031 | Forecast override hook with DetectionResult | open | AC-8 |
| T-0032 | Subscription REST API endpoints | open | AC-2, AC-3, AC-5, AC-6 |
| T-0033 | React subscriptions page shell and routing | open | AC-2, AC-3, AC-4 |
| T-0034 | React detail drawer, ECharts price history, alerts UX | open | AC-5, AC-6 |
| T-0035 | Grafana Dashboard 2 Subscriptions | open | AC-7 |
| T-0036 | Subscription tests and operator user guide | open | AC-1–AC-8 |

---

## T-0025 — Extract shared recurrence core from forecast

**Status:** open  
**Depends on:** US-0002 (`forecast/recurring.rs`)  
**Decisions:** DEC-0013, DEC-0014

### Description

Extract grouping, cadence stability, amount tolerance, and descriptor normalization from `forecast/recurring.rs` into new `backend/src/recurrence/` module per architecture § shared recurrence core:

- `normalize` — lowercase trim, collapse whitespace, strip trailing reference tokens (card suffixes, alphanum codes)
- `group` — group expense transactions (`amount < 0`) by normalized payee key; exclude internal transfers (reuse `is_transfer`)
- `cadence` — median inter-tx days; classify weekly/biweekly/monthly/quarterly/annual; `is_stable_cadence`
- `amount` — median amount; tolerance bands per confidence tier
- `detect` — `detect_recurrence_groups(transactions, config) -> Vec<RecurrenceGroup>` with linked transaction ids and confidence scoring (95/80/60 tiers per DEC-0014; emit only ≥60%, min 3 txs)

Refactor `forecast/recurring.rs` to thin wrapper calling `recurrence::detect`. Preserve existing forecast unit test behavior.

### Done when

- [ ] `recurrence/` module exists with normalize, group, cadence, amount, detect submodules
- [ ] Confidence tiers 95/80/60 implemented per DEC-0014 criteria
- [ ] `forecast/recurring.rs` delegates to shared core without behavior regression
- [ ] Existing forecast recurring unit tests pass unchanged

---

## T-0026 — SQLx migration 003 subscriptions schema

**Status:** open  
**Depends on:** US-0001 (migration 001), US-0002 (migration 002)  
**Decisions:** DEC-0015

### Description

Add SQLx migration `003_subscriptions.sql` per architecture § migration 003:

| Object | Purpose |
|--------|---------|
| `subscription_status` enum | `pending`, `confirmed`, `rejected`, `inactive` |
| `subscription_kind` enum | `subscription`, `standing_order` |
| `subscription_alert_type` enum | `new_detection`, `price_change`, `interval_change` |
| `subscription_patterns` | Lifecycle entity: fingerprint, status, kind, payee_key, display_name, interval_days, current_amount, confidence_pct, timestamps |
| `subscription_pattern_transactions` | M:N link pattern ↔ mirror `transactions` |
| `subscription_price_events` | Append-only billing + change events |
| `subscription_rejections` | Fingerprint PK; permanent exclusion |
| `subscription_alerts` | Page-scoped alerts with nullable `read_at` |

Indexes: `(status, kind)`, `(last_seen_at DESC)`, `(pattern_id, occurred_at DESC)`, `(read_at, created_at DESC)`.

### Done when

- [ ] Migration applies cleanly against external PostgreSQL (relational only — no hypertables)
- [ ] All enums and tables match architecture schema
- [ ] Unique constraint on `subscription_patterns.fingerprint`
- [ ] Foreign keys link pattern transactions to mirror `transactions` table

---

## T-0027 — Subscription repository and config

**Status:** open  
**Depends on:** T-0026  
**Decisions:** DEC-0015, DEC-0017

### Description

Implement `subscriptions::repository` and config additions: `[subscriptions]` TOML section per architecture:

```toml
[subscriptions]
detection_window_days = 365
full_rescan_interval_days = 7
price_change_min_eur = 1.0
price_change_min_pct = 5.0
inactive_grace_days = 5
standing_order_category_patterns = [...]
standing_order_payee_patterns = []

[subscriptions.confidence_tolerance_pct]
high = 5
medium = 10
low = 15
```

Repository methods: upsert pattern, link transactions, append price events, insert rejection fingerprint, CRUD alerts, fetch by status/kind, load confirmed + rejection set for forecast hook.

### Done when

- [ ] Config loads subscription settings from TOML + env overlay
- [ ] Repository writes and reads against migration 003 schema
- [ ] Rejection fingerprint lookup prevents re-detection of rejected patterns
- [ ] Alert CRUD supports unread filter and mark-read

---

## T-0028 — Subscription engine classify, detection, price_change

**Status:** open  
**Depends on:** T-0025, T-0027  
**Decisions:** DEC-0014, DEC-0016, DEC-0017

### Description

Implement pure logic modules under `subscriptions/`:

- **`classify`** — Dauerauftrag vs subscription rules (DEC-0016): exact-amount CV < 0.02, near-exact ±1% monthly/quarterly, category boost from config list, large fixed ≤ −€200; optional TOML payee patterns; default `subscription`
- **`detection`** — orchestrate `recurrence::detect` → classify kind → fingerprint dedup (skip if in `subscription_rejections` or matches confirmed); upsert `pending` patterns; mark `inactive` when gap > 2× interval
- **`price_change`** — dual threshold on confirmed only: `|delta| ≥ €1.00` AND `|delta|/|previous| × 100 ≥ 5%` (DEC-0017); append `subscription_price_events` on every billing; detect interval changes; emit alert types `new_detection`, `price_change`, `interval_change`

### Done when

- [ ] Unit tests cover confidence scoring, Dauerauftrag classification rules, and price-change dual threshold
- [ ] Fingerprint dedup skips rejected and existing confirmed patterns
- [ ] Inactive detection when gap exceeds 2× interval
- [ ] Price events append on every confirmed billing occurrence

---

## T-0029 — SubscriptionService run_detection orchestration

**Status:** open  
**Depends on:** T-0028  
**Decisions:** DEC-0014, DEC-0015

### Description

Implement `subscriptions::service::SubscriptionService` with:

```rust
pub struct DetectionResult {
    pub confirmed_recurring: Vec<ConfirmedRecurring>,
    pub rejected_fingerprints: HashSet<String>,
}

impl SubscriptionService {
    pub async fn run_detection(&self, sync_run_id: Uuid) -> Result<DetectionResult, SubscriptionError>;
}
```

`run_detection` steps: load expense transactions (default 12-month window); run detection pipeline; upsert patterns; check price changes on confirmed; insert alerts; return confirmed list + rejection set. Wire `SubscriptionService` into `AppState`.

Failure semantics: log warning, return prior confirmed state; do not fail sync run if ingest succeeded.

### Done when

- [ ] `run_detection` produces `DetectionResult` with confirmed patterns and rejection fingerprints
- [ ] New/changed candidates upserted as `pending` with confidence badge metadata
- [ ] Alerts inserted for new detection and price/interval changes
- [ ] `SubscriptionService` available on shared application state

---

## T-0030 — Sync pipeline subscriptions phase hook

**Status:** open  
**Depends on:** T-0029  
**Decisions:** DEC-0018

### Description

Extend `SyncService::execute_run` lifecycle per DEC-0018:

```text
1. Firefly entity + transaction sync          (phase: "sync")
2. finish_sync_run(success) on ingest OK
3. SubscriptionService::run_detection(run_id) (phase: "subscriptions")  ← NEW
4. ForecastService::recompute(run_id, detection_result)                 (phase: "forecast")
5. active_run = None; phase = None
```

Mutex covers sync + detection + forecast; `POST /api/v1/sync/trigger` returns 409 during any phase. Sync Status UI displays phase `"sync"` | `"subscriptions"` | `"forecast"`. Log detection duration separately from sync and forecast.

Detection failure must not fail sync run; continue to forecast with prior confirmed state.

### Done when

- [ ] Successful sync triggers `run_detection` before forecast recompute
- [ ] Mutex covers sync, subscriptions, and forecast phases; second trigger returns 409
- [ ] Sync Status reports `"subscriptions"` phase during detection
- [ ] Sync run marked success even if detection fails (forecast uses fallback)

---

## T-0031 — Forecast override hook with DetectionResult

**Status:** open  
**Depends on:** T-0029, T-0030  
**Decisions:** DEC-0013, DEC-0015

### Description

Extend `ForecastService::recompute` to accept optional `DetectionResult`:

```rust
pub async fn recompute(
    &self,
    sync_run_id: Uuid,
    subscription_context: Option<&DetectionResult>,
) -> Result<Uuid, ForecastError>;
```

Override rules in `project_account`:

1. Load `confirmed_recurring` from detection result (or repository fallback if detection skipped)
2. For each confirmed pattern: **replace** heuristic `RecurringPattern` with same `payee_key` (confirmed amount + interval take precedence)
3. Exclude transaction groups whose fingerprint is in `rejected_fingerprints`
4. Heuristic-only patterns for payees without confirmed/rejected state unchanged (DEC-0007 baseline)

Update T-0030 wiring to pass `DetectionResult` from detection to recompute.

### Done when

- [ ] Confirmed subscription amount/interval overrides heuristic recurring projection
- [ ] Rejected fingerprints excluded from forecast projection
- [ ] Heuristic patterns unchanged for payees without user decision
- [ ] Unit or integration test verifies override and exclusion behavior

---

## T-0032 — Subscription REST API endpoints

**Status:** open  
**Depends on:** T-0029  
**Decisions:** DEC-0006, DEC-0015

### Description

Add JWT-protected Axum handlers under `api::subscriptions`:

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/subscriptions` | List patterns; query `status`, `kind` |
| GET | `/api/v1/subscriptions/{id}` | Pattern detail + linked transaction count |
| POST | `/api/v1/subscriptions/{id}/confirm` | Confirm with optional `{ "kind" }` override |
| POST | `/api/v1/subscriptions/{id}/reject` | Reject + store fingerprint in `subscription_rejections` |
| GET | `/api/v1/subscriptions/{id}/price-history` | Append-only price events for ECharts |
| GET | `/api/v1/subscriptions/alerts` | Alert list; query `unread=true` |
| PATCH | `/api/v1/subscriptions/alerts/{id}/read` | Mark alert read |

Confirm transitions `pending` → `confirmed`; reject transitions to `rejected` + permanent fingerprint exclusion. Read-only toward Firefly (DEC-0004).

### Done when

- [ ] All seven endpoints registered and JWT-protected
- [ ] Confirm/reject state transitions persist correctly with fingerprint on reject
- [ ] List endpoint filters by status and kind (for tabs)
- [ ] Price-history returns events suitable for ECharts line chart

---

## T-0033 — React subscriptions page shell and routing

**Status:** open  
**Depends on:** T-0032  
**Decisions:** —

### Description

Enable Subscriptions nav item (replace US-0001 disabled placeholder). Add `/subscriptions` route with shadcn layout:

- Tabs: **All** | **Pending review** | **Standing orders**
- **Pending:** Card per candidate — payee, interval, amount, confidence Badge (95/80/60); Confirm / Reject actions
- **Confirm dialog:** optional kind override dropdown (subscription vs standing order)
- **Confirmed / All:** Table — display name, interval, amount, kind badge, last seen
- **Standing orders tab:** filter `kind=standing_order`
- Empty state: no patterns detected; link to Sync Status

Wire TanStack Query hooks for subscription list and confirm/reject mutations with bearer token.

### Done when

- [ ] Subscriptions nav enabled and route reachable when authenticated
- [ ] Tab switching filters list by status/kind via API query params
- [ ] Pending cards show confidence badge and confirm/reject actions
- [ ] Confirm dialog supports optional kind override
- [ ] Empty state renders when no patterns exist

---

## T-0034 — React detail drawer, ECharts price history, alerts UX

**Status:** open  
**Depends on:** T-0033  
**Decisions:** DEC-0017

### Description

Extend `/subscriptions` page:

- **Detail drawer (Sheet):** linked transactions summary + ECharts price history line from `/price-history`
- **Alerts banner:** visible when unread alerts exist (`GET /subscriptions/alerts?unread=true`)
- **Toast:** surface new detection or price-change alerts after sync poll completes
- Poll alerts endpoint after sync completion (reuse Sync Status poll pattern from US-0002)

Lazy-load ECharts chart component in drawer to limit bundle impact.

### Done when

- [ ] Row click opens detail drawer with price history line chart
- [ ] Banner displays when unread alerts exist; dismiss/mark-read works
- [ ] Toast fires for new alerts after sync-triggered detection
- [ ] Price history chart renders increase/decrease events correctly

---

## T-0035 — Grafana Dashboard 2 Subscriptions

**Status:** open  
**Depends on:** T-0026, T-0029  
**Decisions:** DEC-0012, R-0014

### Description

Add `grafana/provisioning/dashboards/analytics/subscriptions.json` with uid `subscriptions`, folder Analytics. Panels per R-0014:

| Panel | Type | Query notes |
|-------|------|-------------|
| Confirmed count | stat | `status = confirmed` |
| Monthly spend | stat | interval-normalized (weekly ×4.33, annual ÷12) |
| Pending count | stat | `status = pending` |
| Price changes (90d) | table | before/after columns |
| New detections | time series | daily count by `created_at` |

MVP scope: **global** (no per-account variable). Reuse datasource uid `FlowFinancePostgreSQL`. Platform Health, Dashboard 1, and Dashboard 5 unchanged.

### Done when

- [ ] Dashboard loads with uid `subscriptions` in Analytics folder
- [ ] Stat panels show confirmed count, normalized monthly spend, pending count
- [ ] Price-change table shows before/after for last 90 days
- [ ] New detections time series panel queries daily counts
- [ ] Empty-state panels render gracefully with no subscription data

---

## T-0036 — Subscription tests and operator user guide

**Status:** open  
**Depends on:** T-0025, T-0030, T-0031, T-0032, T-0034, T-0035  
**Decisions:** DEC-0013, DEC-0018

### Description

Add Rust unit tests for recurrence confidence scoring, Dauerauftrag classification, price-change dual threshold, and forecast override/exclusion. Add integration test with fixture mirror data: run detection → assert pattern rows and API responses. Extend `tests/run-tests.sh` to include subscription test targets. Integration test skips without `DATABASE_URL` (same pattern as US-0001/US-0002).

Create `docs/user-guides/US-0003.md`: sync prerequisite, using `/subscriptions` page (pending review, confirm/reject, standing orders tab), interpreting confidence tiers, price-change alerts, Grafana Dashboard 2, forecast override behavior for confirmed/rejected patterns.

### Done when

- [ ] Unit tests pass for recurrence, classify, price_change, and forecast override logic
- [ ] Integration test validates detection → DB → API read path (or SKIP without DATABASE_URL)
- [ ] `bash tests/run-tests.sh` includes subscription tests and passes
- [ ] User guide covers prerequisites, subscription UI, alerts, Grafana Dashboard 2, and forecast integration

---

## Execution order (recommended)

1. **Refactor + database:** T-0025 → T-0026 → T-0027
2. **Engine:** T-0028 → T-0029
3. **Integration:** T-0030 → T-0031 → T-0032
4. **Frontend (after T-0032):** T-0033 → T-0034
5. **Grafana (parallel after T-0029):** T-0035
6. **Verification:** T-0036

## Acceptance coverage map

| AC | Tasks |
|----|-------|
| Confidence tiers 95/80/60% | T-0025, T-0028, T-0029, T-0036 |
| New detection + confirm/reject notification | T-0029, T-0030, T-0032, T-0033, T-0034 |
| Confirmed list with interval and amount | T-0027, T-0032, T-0033 |
| Standing-order separation | T-0028, T-0032, T-0033 |
| Price increase/decrease on confirmed | T-0028, T-0032, T-0034 |
| Alerts on new detection and price change | T-0028, T-0029, T-0032, T-0034 |
| Grafana Dashboard 2 | T-0035 |
| Rejected excluded from forecast and alerts | T-0027, T-0031, T-0032, T-0036 |
