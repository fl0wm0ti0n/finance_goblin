# Tasks — Sprint S0004

**Story:** US-0004  
**Task count:** 12 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0037 | SQLx migration 004 plans schema | done | AC-1, AC-3, AC-4 |
| T-0038 | Plan repository and config | done | AC-1, AC-3 |
| T-0039 | Plan Engine overlay and project modules | done | AC-1, AC-4, AC-6 |
| T-0040 | Plan templates and versioning logic | done | AC-2, AC-3 |
| T-0041 | PlanService orchestration and plan-vs-Ist | done | AC-1, AC-3, AC-4 |
| T-0042 | Forecast post-recompute active plan hook | done | AC-4, AC-6 |
| T-0043 | Plan REST API endpoints | done | AC-1–AC-5 |
| T-0044 | React planning page shell and Scenarios tab | done | AC-1, AC-2, AC-3 |
| T-0045 | React Compare and Plan vs Actual tabs | done | AC-3, AC-4 |
| T-0046 | Grafana Dashboard 3 Budgets | done | AC-5 |
| T-0047 | Plan tests | done | AC-1–AC-6 |
| T-0048 | Operator user guide | done | AC-1–AC-6 |

---

## T-0037 — SQLx migration 004 plans schema

**Status:** open  
**Depends on:** US-0001 (migration 001), US-0002 (migration 002), US-0003 (migration 003)  
**Decisions:** DEC-0022

### Description

Add SQLx migration `004_plans.sql` per architecture § migration 004:

| Object | Purpose |
|--------|---------|
| `plan_template` enum | `current`, `leasing`, `savings_mode`, `house_purchase`, `custom` |
| `plan_adjustment_direction` enum | `add_outflow`, `remove_outflow`, `add_inflow`, `remove_inflow` |
| `plan_adjustment_frequency` enum | `monthly`, `weekly`, `quarterly`, `one_time` |
| `plan_adjustment_target` enum | `household`, `subscription`, `category`, `custom_label` |
| `plans` | Named plan container: `name`, `template`, `is_active`, timestamps |
| `plan_versions` | v1–v3 per plan; `version_number`, `is_latest`, `frozen_at`, `baseline_computation_id` |
| `plan_adjustments` | Delta lines per version: direction, amount, frequency, effective dates, target |
| `plan_computations` | Recompute metadata; links `forecast_computation_id`, `version_id`, status |
| `plan_daily_cashflow` | Hypertable: `ts`, `planned_net`, optional `planned_balance` |

Indexes and constraints:

- `plans_one_active` — partial unique index where `is_active = true`
- `plan_versions_one_latest` — partial unique per plan where `is_latest = true`
- `(version_id, computation_id, ts DESC)` for latest series lookup
- CHECK `version_number BETWEEN 1 AND 3`

Retention policy enforced in service layer: last 3 successful `plan_computations` per version.

### Done when

- [ ] Migration applies cleanly against external PostgreSQL with TimescaleDB extension
- [ ] All enums and tables match architecture schema
- [ ] Partial unique indexes enforce single active plan and single latest version per plan
- [ ] `plan_daily_cashflow` created as hypertable with appropriate chunk interval

---

## T-0038 — Plan repository and config

**Status:** open  
**Depends on:** T-0037  
**Decisions:** DEC-0020, DEC-0022, DEC-0024

### Description

Implement `plan::repository` and config additions: `[plans]` TOML section per architecture:

```toml
[plans]
leasing_default_monthly_eur = 300.0
house_purchase_default_savings_eur = 500.0
savings_mode_discretionary_cut_eur = 100.0
max_versions_per_plan = 3
computation_retention_per_version = 3
reporting_currency = "EUR"
```

Repository methods:

- Plan CRUD: create, rename, delete (cascade), list with latest version summary
- Version CRUD: create version (copy adjustments), freeze prior, fetch by id/number, enforce cap
- Adjustment CRUD: add/update/delete lines on latest version only
- Computation: insert metadata, bulk insert `plan_daily_cashflow`, fetch latest successful series
- Active plan: `get_active`, `set_active` (transactional deactivate-all → activate target)
- Compare: load all versions with adjustment summaries for metrics

Wire `PlanRepository` into `AppState` as part of `PlanService` construction prep.

### Done when

- [ ] Config loads plan settings from TOML + env overlay
- [ ] Repository writes and reads against migration 004 schema
- [ ] `set_active` uses transaction with partial unique index enforcement
- [ ] Version cap returns structured error when attempting v4
- [ ] Frozen version rejects adjustment mutations at repository layer

---

## T-0039 — Plan Engine overlay and project modules

**Status:** open  
**Depends on:** T-0038, US-0002 forecast snapshots  
**Decisions:** DEC-0019, DEC-0021

### Description

Implement pure logic modules under `plan/`:

- **`types`** — `PlanAdjustment`, direction/frequency/target enums, API DTOs, computation metadata
- **`overlay`** — map adjustments → daily net-cashflow deltas:
  - Resolve `baseline_computation_id` from latest successful `forecast_computations`
  - Load baseline household daily net cashflow from forecast output
  - Project recurring impacts onto future calendar days (reuse cadence math from forecast recurring layer)
  - One-time deltas apply on `effective_from` only
  - `target_type = subscription` + `remove_outflow` → zero matching confirmed subscription recurring amount by `payee_key` (US-0003)
  - Deterministic `sort_order` when multiple deltas target same payee
- **`project`** — merge baseline with overlay; emit `plan_daily_cashflow` rows; optional `planned_balance` path from forecast balance layer

Do **not** fork or duplicate `ForecastService` projection logic (DEC-0019). Read-only toward Firefly mirrors for Ist path prep (DEC-0004).

### Done when

- [ ] Unit tests cover overlay for monthly/weekly/quarterly/one-time adjustments
- [ ] Subscription removal overlay matches confirmed `payee_key` from US-0003
- [ ] Project merges baseline + overlay without mutating forecast hypertables
- [ ] `forecast_computation_id` binding recorded on each computation

---

## T-0040 — Plan templates and versioning logic

**Status:** open  
**Depends on:** T-0039  
**Decisions:** DEC-0019, DEC-0020

### Description

Implement `plan::templates` and versioning orchestration:

**Built-in templates (defaults, user-editable on apply — TOML overridable):**

| Template | Default deltas |
|----------|----------------|
| **Current (Ist)** | none (baseline only) |
| **Leasing** | `+€300/month` household outflow, label "Leasing" |
| **Savings mode** | suggest confirmed subscriptions; user selects IDs; optional `−€100/month` discretionary cut |
| **House purchase** | `+€500/month` savings transfer (category or custom label) |
| **Custom** | empty; user adds lines |

**Version semantics (DEC-0020):**

- Latest version (`is_latest=true`) editable in-place
- `create_version`: freeze prior (`frozen_at` set); copy adjustments to new row; increment version number
- Hard cap 3 versions per plan; v4 attempt → structured error for HTTP 409
- Compare metrics helpers: monthly delta sum, projected month-end balance (household aggregate)

**Savings mode suggestions:** query confirmed `subscription_patterns` for template apply flow.

### Done when

- [ ] Each built-in template produces correct default adjustment bundle
- [ ] `apply_template` merges template deltas with optional user overrides
- [ ] Savings mode suggestions return confirmed subscriptions only
- [ ] Version create copies adjustments and freezes prior version
- [ ] Unit tests cover version cap and freeze semantics

---

## T-0041 — PlanService orchestration and plan-vs-Ist

**Status:** open  
**Depends on:** T-0039, T-0040  
**Decisions:** DEC-0021, DEC-0023, DEC-0024

### Description

Implement `plan::service::PlanService`:

```rust
impl PlanService {
    pub async fn recompute_version(&self, version_id: Uuid, forecast_computation_id: Uuid) -> Result<Uuid, PlanError>;
    pub async fn plan_vs_actual(&self, month: Option<NaiveDate>) -> Result<PlanVsActualResponse, PlanError>;
    pub async fn compare_versions(&self, plan_id: Uuid) -> Result<CompareResponse, PlanError>;
    // ... CRUD wrappers delegating to repository + spawn recompute
}
```

**Recompute (DEC-0023):**

- On plan mutation: `tokio::spawn` recompute affected version **outside** sync mutex
- Retain last 3 successful computations per version
- Failure: serve last successful snapshot with `plan_stale=true`

**Plan-vs-Ist (DEC-0021):**

- **Planned:** `plan_daily_cashflow.planned_net` for active latest version's latest computation
- **Actual (Ist):** household daily net cashflow from mirror transactions (asset accounts, exclude transfers)
- **Deviation:** `actual_net - planned_net` (positive = better than plan)
- Metadata: `actuals_stale`, `plan_stale`, `reporting_currency`
- Default month: current calendar month

Secondary category drill-down deferred to React tab only (not Grafana MVP).

### Done when

- [ ] `recompute_version` persists computation + daily series
- [ ] `plan_vs_actual` returns daily planned/actual/deviation with stale flags
- [ ] `compare_versions` returns side-by-side v1/v2/v3 metrics
- [ ] Plan mutations spawn async recompute without blocking API response
- [ ] Unit or integration test validates deviation sign convention

---

## T-0042 — Forecast post-recompute active plan hook

**Status:** open  
**Depends on:** T-0041  
**Decisions:** DEC-0023, DEC-0010

### Description

Extend `ForecastService::recompute` success path per DEC-0023:

```rust
// End of ForecastService::recompute success:
if let Some(active) = plan_service.active_plan().await? {
    plan_service.recompute_version(active.latest_version_id, computation_id).await?;
}
```

Rules:

- Post-forecast refresh applies to **active plan latest version only**
- Plan recompute failure does **not** fail sync/forecast run
- **No** new `"planning"` phase in sync mutex (DEC-0010 latency precedent)
- Log plan recompute duration separately from forecast

Do **not** add active plan overlay on `/forecast` page (deferred per R-0019).

### Done when

- [ ] Successful forecast recompute triggers active plan refresh when active plan exists
- [ ] Plan recompute failure logged as warning; sync run still succeeds
- [ ] Sync mutex phases unchanged (`sync` | `subscriptions` | `forecast` only)
- [ ] Active plan uses new `forecast_computation_id` after post-sync refresh

---

## T-0043 — Plan REST API endpoints

**Status:** open  
**Depends on:** T-0041  
**Decisions:** DEC-0006, DEC-0020, DEC-0024

### Description

Add JWT-protected Axum handlers under `api::plans`:

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/plans` | List plans with latest version summary |
| POST | `/api/v1/plans` | Create plan `{ name, template? }` |
| GET | `/api/v1/plans/{id}` | Plan detail + versions |
| PATCH | `/api/v1/plans/{id}` | Rename plan |
| DELETE | `/api/v1/plans/{id}` | Delete plan (cascade) |
| POST | `/api/v1/plans/{id}/activate` | Set global active plan |
| GET | `/api/v1/plans/{id}/versions` | List v1–v3 |
| POST | `/api/v1/plans/{id}/versions` | Create new version (409 if at cap) |
| GET | `/api/v1/plans/{id}/versions/{vid}` | Version + adjustments |
| PATCH | `/api/v1/plans/{id}/versions/{vid}` | Edit adjustments (latest only; 409 if frozen) |
| POST | `/api/v1/plans/{id}/versions/{vid}/adjustments` | Add adjustment line |
| PATCH | `/api/v1/plans/{id}/versions/{vid}/adjustments/{aid}` | Update line |
| DELETE | `/api/v1/plans/{id}/versions/{vid}/adjustments/{aid}` | Remove line |
| POST | `/api/v1/plans/{id}/versions/{vid}/apply-template` | Apply template `{ template, overrides? }` |
| GET | `/api/v1/plans/templates/savings-mode/suggestions` | Confirmed subscriptions |
| GET | `/api/v1/plans/{id}/compare` | Side-by-side v1/v2/v3 metrics |
| GET | `/api/v1/plans/active/plan-vs-actual` | Daily series `{ month?: YYYY-MM }` |
| POST | `/api/v1/plans/{id}/versions/{vid}/recompute` | Manual recompute (operator/debug) |

Read-only toward Firefly (DEC-0004). Return 409 with clear message on v4 attempt and frozen version edits.

### Done when

- [ ] All endpoints registered and JWT-protected
- [ ] Activate enforces single global active plan transactionally
- [ ] Version cap and frozen-version guards return HTTP 409
- [ ] Plan-vs-actual and compare endpoints return data suitable for React/ECharts
- [ ] Mutation endpoints spawn async recompute (non-blocking response)

---

## T-0044 — React planning page shell and Scenarios tab

**Status:** open  
**Depends on:** T-0043  
**Decisions:** —

### Description

Enable Planning nav item (replace US-0001 disabled placeholder). Add `/planning` route with shadcn layout:

- **Header:** Active plan selector (`Select`); "Set active" action
- **Tabs:** Scenarios | Compare | Plan vs Actual (Compare and Plan vs Actual wired in T-0045)
- **Scenarios tab:**
  - Template cards: Current, Leasing, Savings, House, Custom
  - Adjustment table: amount, frequency, target, label; add/edit/delete on latest version
  - "Create new version" when viewing frozen history
  - Savings mode modal: suggested confirmed subscriptions with checkboxes before apply
- **Empty state:** No plans yet; CTA to create from template
- **Stale badges:** Show when `plan_stale` or `actuals_stale` from API

Wire TanStack Query hooks for plan list, detail, mutations, and activate with bearer token.

### Done when

- [ ] Planning nav enabled and route reachable when authenticated
- [ ] Template cards apply presets via apply-template API
- [ ] Adjustment table CRUD works on latest version only
- [ ] Savings mode modal loads suggestions and applies selected subscription removals
- [ ] Active plan selector and set-active action work
- [ ] Empty state and stale badges render correctly

---

## T-0045 — React Compare and Plan vs Actual tabs

**Status:** open  
**Depends on:** T-0044  
**Decisions:** DEC-0020, DEC-0021

### Description

Extend `/planning` page:

- **Compare tab:**
  - Metrics table: v1/v2/v3 side-by-side (monthly delta sum, projected month-end balance)
  - ECharts grouped bar chart for month-end balance and monthly delta sum
  - UX copy when at version cap (v3 exists; explain archive/new plan for v4)
- **Plan vs Actual tab:**
  - Dual-line ECharts: planned vs actual daily net cashflow
  - Daily table: planned, Ist (actual), deviation columns
  - Default view: current calendar month; month picker optional
  - Deviation zero reference line on chart
  - Document that planned extends beyond today; Ist stops at last sync date

Lazy-load ECharts chart components to limit bundle impact.

### Done when

- [ ] Compare tab renders metrics table and grouped bar for up to 3 versions
- [ ] Plan vs Actual dual-line chart and daily table populate from active plan API
- [ ] Deviation column uses actual − planned sign convention
- [ ] Version cap messaging shown when v3 exists
- [ ] Charts lazy-loaded; page builds without regression

---

## T-0046 — Grafana Dashboard 3 Budgets

**Status:** open  
**Depends on:** T-0037, T-0041  
**Decisions:** DEC-0012, DEC-0024, R-0020

### Description

Add `grafana/provisioning/dashboards/analytics/budgets.json` with uid `budgets`, folder Analytics. Panels per R-0020:

| Panel | Type | Query notes |
|-------|------|-------------|
| Active plan stat | stat | Name of active plan + version |
| Plan time series | time series | `plan_daily_cashflow.planned_net` for active latest version |
| Ist time series | time series | household daily net from mirror transactions |
| Abweichung (deviation) | time series | FULL OUTER JOIN actuals and planned; `actual - planned` |
| MTD summary table | table | month-to-date planned, actual, deviation totals |

**Template variable:** `$active_plan_version` — latest version of `is_active=true` plan.

**MVP scope:** household aggregate only — no per-category panels.

**Empty state:** dashboard annotation when no active plan — "Select active plan in Flow Finance Planning UI".

Reuse datasource uid `FlowFinancePostgreSQL`. Platform Health, Dashboards 1, 2, 5 unchanged.

### Done when

- [ ] Dashboard loads with uid `budgets` in Analytics folder
- [ ] Plan, Ist, and Abweichung panels query active plan series correctly
- [ ] Deviation SQL uses FULL OUTER JOIN with timestamptz cast for Grafana time axis
- [ ] Empty-state annotation renders when no active plan
- [ ] MTD summary table shows planned/actual/deviation totals

---

## T-0047 — Plan tests

**Status:** open  
**Depends on:** T-0039, T-0041, T-0042, T-0043, T-0045, T-0046  
**Decisions:** DEC-0019, DEC-0020, DEC-0023

### Description

Add Rust unit tests for:

- Overlay: monthly/weekly adjustments, subscription removal by payee_key, one-time deltas
- Versioning: create version freezes prior, v4 cap rejection, compare metrics
- Plan-vs-Ist: deviation sign, stale metadata flags
- Forecast hook: active plan refresh invoked after recompute (mock or integration)

Add integration test with fixture mirror + forecast + plan data: create plan → apply template → recompute → assert plan-vs-actual API response. Extend `tests/run-tests.sh` to include plan test targets. Integration test skips without `DATABASE_URL` (same pattern as US-0001–US-0003).

Add read-only audit assertion: no Firefly write paths in plan module (AC-6).

### Done when

- [ ] Unit tests pass for overlay, templates, versioning, and plan-vs-Ist logic
- [ ] Integration test validates plan create → recompute → plan-vs-actual path (or SKIP without DATABASE_URL)
- [ ] Forecast hook covered by unit or integration test
- [ ] `bash tests/run-tests.sh` includes plan tests and passes
- [ ] No Firefly write operations in plan code paths verified

---

## T-0048 — Operator user guide

**Status:** open  
**Depends on:** T-0044, T-0045, T-0046, T-0047  
**Decisions:** —

### Description

Create `docs/user-guides/US-0004.md` per USER_GUIDE_MODE=1:

- Prerequisites: sync + forecast recompute; confirmed subscriptions for savings mode
- Creating plans from templates (Current, Leasing, Savings, House, Custom)
- Editing adjustments and creating v2/v3 versions
- Compare tab interpretation (monthly delta, month-end balance)
- Plan vs Actual tab: planned vs Ist vs deviation; stale badge meaning
- Setting active plan and Grafana Dashboard 3 (`budgets`)
- Savings mode: selecting subscriptions to remove
- Version cap (max 3) and creating a new named plan
- Read-only guarantee: plans stored in Flow DB; Firefly unchanged

### Done when

- [ ] User guide covers all six acceptance criteria from operator perspective
- [ ] Prerequisites and stale-badge semantics documented
- [ ] Grafana Dashboard 3 access and active-plan requirement documented
- [ ] Savings mode workflow documented with subscription selection step

---

## Execution order (recommended)

1. **Database:** T-0037 → T-0038
2. **Engine:** T-0039 → T-0040 → T-0041
3. **Integration:** T-0042 → T-0043
4. **Frontend (after T-0043):** T-0044 → T-0045
5. **Grafana (parallel after T-0041):** T-0046
6. **Verification:** T-0047 → T-0048

## Acceptance coverage map

| AC | Tasks |
|----|-------|
| Named plan + scenario adjustments | T-0037, T-0038, T-0039, T-0041, T-0043, T-0044, T-0047 |
| Built-in templates (current, leasing, savings, house) | T-0040, T-0043, T-0044, T-0048 |
| v1/v2/v3 create and compare | T-0038, T-0040, T-0041, T-0043, T-0045, T-0047 |
| Daily plan-vs-Ist (planned, actual, deviation) | T-0039, T-0041, T-0043, T-0045, T-0047 |
| Grafana Dashboard 3 active plan | T-0046, T-0048 |
| No Firefly mutation | T-0039, T-0042, T-0043, T-0047 |
