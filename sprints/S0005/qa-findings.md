# QA Findings — Sprint S0005 / US-0005

**Sprint:** S0005  
**Story:** US-0005  
**QA phase:** `/qa`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/verify-work`)

## Scope

Wealth analysis and unified Alert Engine: migration 005, WealthService (net worth + daily snapshots), Alert Engine (scarcity/budget-drift/plan-viability), AlertService lifecycle, sync `"alerts"` phase, 6 wealth/alert REST endpoints, React `/wealth` + `/alerts` + header bell, Grafana Dashboard 4 (`portfolio`) + Dashboard 1 `$scarcity_threshold`, unit/integration tests, operator user guide.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0005/summary.md`, `sprints/S0005/tasks.md`, `sprints/S0005/plan-verify.json`, `docs/product/acceptance.md` (US-0005), `backend/src/{wealth,alerts}/`, `backend/src/api/{wealth,alerts}.rs`, `backend/migrations/005_alerts_wealth.sql`, `backend/tests/wealth_alerts_integration.rs`, `frontend/src/pages/{WealthPage,AlertsPage}.tsx`, `frontend/src/components/AlertBell.tsx`, `grafana/provisioning/dashboards/analytics/{portfolio,cashflow}.json`, `docs/user-guides/US-0005.md`, `docs/engineering/spec-pack/US-0005-*.md`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Automated harness | `bash tests/run-tests.sh` | **PASS** |
| T-2 | Firefly GET-only unit test | `cargo test --test firefly_readonly` | **PASS** (1/1) |
| T-3 | Wealth + alert unit tests | `cargo test --lib` | **PASS** (36/36; 8 wealth/alert-specific) |
| T-4 | Wealth/alerts integration | `cargo test --test wealth_alerts_integration` | **SKIP** — `DATABASE_URL` not set |
| T-5 | Static Firefly write audit | `wealth_alerts_modules_have_no_firefly_writes` | **PASS** |
| T-6 | Dashboard 1 threshold variable | `cashflow_dashboard_uses_scarcity_threshold_variable` | **PASS** |
| T-7 | Forecast integration (US-0002 carry-forward) | `cargo test --test forecast_integration` | **SKIP** — `DATABASE_URL` not set |
| T-8 | Plan integration (US-0004 carry-forward) | `cargo test --test plans_integration` | **SKIP** — `DATABASE_URL` not set |
| T-9 | Subscription integration (US-0003 carry-forward) | `cargo test --test subscriptions_integration` | **SKIP** — `DATABASE_URL` not set |
| T-10 | Firefly integration (US-0001 carry-forward) | `cargo test --test firefly_integration` | **SKIP** — `DATABASE_URL` not set |
| T-11 | Frontend build | `npm run build` (via harness) | **PASS** |
| T-12 | Grafana Dashboard 4 provisioning | Static review `portfolio.json` | **PASS** — uid `portfolio`, datasource `FlowFinancePostgreSQL` |
| T-13 | User guide (USER_GUIDE_MODE=1) | Static review `docs/user-guides/US-0005.md` | **PASS** — Purpose, Prerequisites, Usage, threshold config, Grafana |
| T-14 | Spec-pack (SPEC_PACK_MODE=1) | Static review `docs/engineering/spec-pack/US-0005-*.md` | **PASS** — design-concept, crs, technical-specification (3/3) |
| T-15 | Runtime E2E (live stack → `/wealth`, `/alerts`, bell, Grafana 4) | Not executed in QA environment | **Deferred** to `/verify-work` |

### Environment dependencies (non-blocking)

- **`DATABASE_URL`:** Required for `wealth_alerts_integration` (snapshot upsert + scarcity alert on post-sync; static Firefly write audit runs without DB). Harness skips gracefully; unit tests cover evaluator logic, lifecycle, and wealth aggregation. Not a QA blocker (same pattern as S0002–S0004).
- **Successful forecast computation:** Required for scarcity alerts at runtime — deferred to verify-work.
- **Active plan + category adjustments + synced MTD transactions:** Required for budget drift alerts at runtime — deferred to verify-work.
- **Active plan with negative projected month-end:** Required for plan viability alerts at runtime — deferred to verify-work.
- **OIDC / `AUTH_DEV_BYPASS=true`:** Required for live API/UI acceptance — deferred to verify-work.

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | Net worth view aggregates Firefly-linked accounts (giro, savings, etc.) | **PASS** | Migration `005_alerts_wealth.sql` (`net_worth_snapshots`); `WealthRepository::load_asset_accounts` filters `type = 'asset'`, active, `include_net_worth`, non-negative balance; `WealthService::compute_breakdown` + `upsert_daily_snapshot`; `GET /api/v1/wealth` + `/wealth/history`; React `/wealth` with stat card, breakdown table, mixed-currency banner, crypto placeholder; unit tests `asset_filter_excludes_negative_balance`, `mixed_currency_when_multiple_currencies`. |
| AC-2 | Scarcity alert fires when projected balance falls below configurable threshold (e.g. 200 €) | **PASS** | `alert_config.scarcity_threshold_eur` + TOML `[alerts]` mirror; `evaluate_scarcity` household aggregate over 45-day horizon; `AlertService::run_post_sync` upsert/resolve; sync `"alerts"` phase after forecast; `GET /api/v1/alerts`; Dashboard 1 `$scarcity_threshold` from `alert_config`; unit tests `scarcity_severity_critical_when_current_below`, `scarcity_fingerprint_format`; integration test `wealth_snapshot_and_scarcity_alert_on_post_sync` (SKIP without DB). |
| AC-3 | Budget drift alert fires when category spending exceeds plan by configurable % (e.g. +20%) | **PASS** | `alert_config.budget_drift_pct`; `evaluate_budget_drift` MTD proration vs category-targeted plan adjustments; fingerprint `budget_drift:category:{id}:{month}`; surfaced via alert REST + `/alerts` UI; unit tests `budget_drift_proration_mid_month`, `budget_drift_skips_non_category`; user guide §2 documents trigger. Runtime firing deferred to verify-work (needs active plan + synced transactions). |
| AC-4 | Plan viability alert fires when active scenario becomes infeasible per forecast | **PASS** | `evaluate_plan_viability` checks negative month-end balances on `plan_daily_cashflow`; severity escalates on consecutive negative month-ends; `AlertService` binds `plan_computation_id` in sync eval context; alert type `plan_viability` in schema + UI; user guide §2 documents trigger. Runtime firing deferred to verify-work (needs active plan computation). |
| AC-5 | Alert inbox in React UI lists active alerts with acknowledge/dismiss | **PASS** | `alerts` table lifecycle (`active` → `acknowledged` / `dismissed` / `resolved`); `PATCH /api/v1/alerts/{id}/acknowledge` + `/dismiss`; `GET /api/v1/alerts` + `/unread-count`; `AlertsPage` with Acknowledge/Dismiss actions; `AlertBell` with unread badge + preview popover + link to `/alerts`; unit test `unread_requires_active_unacknowledged`; DEC-0027 lifecycle applied. |
| AC-6 | Grafana Dashboard 4 shows total wealth (non-crypto until US-0007) | **PASS** | `grafana/provisioning/dashboards/analytics/portfolio.json` — uid `portfolio`, title "Portfolio"; panels: Total wealth stat (`net_worth_snapshots.total_eur`), Wealth over time time series, Account breakdown table, Mixed-currency warning, Crypto placeholder ("US-0007"); `/wealth` page links to Grafana Dashboard 4; snapshots fed by `WealthService::upsert_daily_snapshot`. |

**Summary:** 6/6 PASS (5 fully verified in QA static/unit path; runtime alert firing, integration test, and Grafana/UI E2E deferred to verify-work with operator env).

## Generated baseline test evidence (US-0066 / DEC-0048)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` + `typescript` (Axum backend + Vite/React frontend) |
| `generated_test_command` | `bash tests/run-tests.sh` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-05-31 — exit 0, message `All US-0005 tests passed` |
| `generated_test_paths_ref` | `backend/src/{wealth,alerts}/*` (unit), `backend/tests/wealth_alerts_integration.rs`, `backend/tests/firefly_readonly.rs`, `frontend/` build |
| `generated_test_reason_code` | — |

## Runtime QA evidence (US-0065 / DEC-0047)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed (compile/test-only QA pass) |
| `runtime_stack_profile` | `rust` + `node` (backend + frontend) |
| `runtime_mode` | `local` |
| `runtime_health_target` | Deferred — `http://localhost:8080/api/v1/wealth`, `http://localhost:5173/wealth`, `http://localhost:5173/alerts`, Grafana `uid=portfolio` |
| `runtime_health_result` | `deferred` |
| `runtime_log_summary` | N/A — no live stack started |
| `runtime_retry_count` | 0 |
| `runtime_retry_ledger` | `[]` |
| `runtime_final_verdict` | `deferred` (verify-work) |
| `runtime_reason_code` | `RUNTIME_E2E_DEFERRED_VERIFY_WORK` |
| `runtime_evidence_refs` | `sprints/S0005/uat.md`, `docs/user-guides/US-0005.md` |

## Findings

### Blockers

None.

### Observations (non-blocking)

1. **`wealth_alerts_integration` skipped:** Expected without external PostgreSQL/TimescaleDB. Static audit `wealth_alerts_modules_have_no_firefly_writes` passes; scarcity integration path documented in test. Verify-work should run with `DATABASE_URL`.
2. **Budget drift / plan viability runtime firing not exercised:** Evaluator logic covered by unit tests; end-to-end alert creation for these types requires operator-provisioned plan + transaction fixtures — deferred to verify-work per `sprints/S0005/uat.md`.
3. **Runtime E2E deferred:** Live `/wealth`, `/alerts`, header bell badge, and Grafana Dashboard 4 panels require operator-provisioned stack with synced asset accounts and at least one post-sync snapshot.
4. **ECharts bundle size:** Main chunk ~1 MB (vite warning); `WealthChart` code-split — acceptable for MVP.
5. **Rust dead-code warning:** `Claims` fields unused in `auth/mod.rs` — cosmetic carry-forward from US-0001.
6. **Mixed-currency headline total:** Sums native balances without FX — mandatory warning banner present in React and Grafana (R-0021); by design until US-0007.

## Decision gates

No critical issues requiring dev rework. Proceed to **`/verify-work`**.

## Next phase

Run `/verify-work` in a fresh subagent with operator-provisioned PostgreSQL (`DATABASE_URL`), successful forecast computation, synced asset accounts, active plan with category adjustments, and optional `AUTH_DEV_BYPASS=true` for API/UI acceptance.
