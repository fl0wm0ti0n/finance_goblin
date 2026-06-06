# UAT — Sprint S0005 / US-0005

**Sprint:** S0005  
**Story:** US-0005  
**Phase:** `/verify-work`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/release`)  
**Ready for release:** yes (operator runtime prerequisites documented below)

## Inputs

- QA PASS: `sprints/S0005/qa-findings.md`
- Acceptance: `docs/product/acceptance.md` (US-0005)
- Operator guide: `docs/user-guides/US-0005.md`
- Implementation: `backend/src/{wealth,alerts}/`, `backend/src/api/{wealth,alerts}.rs`, `backend/migrations/005_alerts_wealth.sql`, `frontend/src/pages/{WealthPage,AlertsPage}.tsx`, `frontend/src/components/AlertBell.tsx`, `grafana/provisioning/dashboards/analytics/{portfolio,cashflow}.json`

## Environment

| Prerequisite | Status in verify-work |
|--------------|----------------------|
| `.env` populated | **Not present** — no operator `.env` in workspace |
| `DATABASE_URL` (TimescaleDB + extension) | **Unset** — integration tests skipped by design |
| Firefly PAT + synced asset accounts | **Not provisioned** — live net worth breakdown deferred |
| US-0002 forecast recompute completed | **Not provisioned** — live scarcity alert firing deferred |
| US-0004 active plan with category adjustments | **Not provisioned** — live budget drift / plan viability firing deferred |
| `AUTH_DEV_BYPASS` or OIDC IdP | **Unset** — live API/UI auth flow deferred |

Per workflow policy: code-level and automated verification **pass**; runtime E2E steps recorded as **PASS-with-prerequisites** where external infra is required.

## Automated checks

| ID | Check | Command | Result |
|----|-------|---------|--------|
| AUTO-1 | Test harness | `bash tests/run-tests.sh` | **PASS** |
| AUTO-2 | Firefly GET-only unit test | (via harness) `cargo test --test firefly_readonly` | **PASS** (1/1) |
| AUTO-3 | Wealth + alert unit tests | (via harness) `cargo test --lib` | **PASS** (36/36; 8 wealth/alert-specific) |
| AUTO-4 | Firefly integration (audit log) | (via harness) `cargo test --test firefly_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-5 | Forecast hypertable integration | (via harness) `cargo test --test forecast_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-6 | Subscription integration | (via harness) `cargo test --test subscriptions_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-7 | Plan integration | (via harness) `cargo test --test plans_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-8 | Wealth/alerts integration (DB path) | (via harness) `cargo test --test wealth_alerts_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-9 | Static Firefly write audit | `cargo test --test wealth_alerts_integration wealth_alerts_modules_have_no_firefly_writes` | **PASS** |
| AUTO-10 | Dashboard 1 threshold variable | `cargo test --test wealth_alerts_integration cashflow_dashboard_uses_scarcity_threshold_variable` | **PASS** |
| AUTO-11 | Frontend production build | (via harness) `npm run build` | **PASS** (lazy chunk `WealthChart`; routes `/wealth`, `/alerts`) |
| AUTO-12 | Compose minimal services | `docker compose --profile minimal config --services` (placeholder env) | **PASS** — `firefly-iii`, `flow-finance-ai`, `grafana` |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | Net worth view aggregates Firefly-linked accounts | **PASS-with-prerequisites** | Migration `005_alerts_wealth.sql` (`net_worth_snapshots`); `WealthService::compute_breakdown` filters asset accounts with `include_net_worth`; `GET /api/v1/wealth` + `/wealth/history`; React `/wealth` stat card, breakdown table, mixed-currency banner, crypto placeholder; unit tests `asset_filter_excludes_negative_balance`, `mixed_currency_when_multiple_currencies`. **Operator prerequisite:** synced Firefly asset accounts for live breakdown data. |
| UAT-2 | AC-2 | Scarcity alert fires when projected balance falls below configurable threshold | **PASS-with-prerequisites** | `alert_config.scarcity_threshold_eur` + TOML `[alerts]` mirror; `evaluate_scarcity` household aggregate over 45-day horizon; sync `"alerts"` phase; Dashboard 1 `$scarcity_threshold` from `alert_config`; unit tests `scarcity_severity_critical_when_current_below`, `scarcity_fingerprint_format`. **Operator prerequisite:** successful forecast with breaching balances + post-sync alerts phase. |
| UAT-3 | AC-3 | Budget drift alert fires when category spending exceeds plan by configurable % | **PASS-with-prerequisites** | `alert_config.budget_drift_pct`; `evaluate_budget_drift` MTD proration vs category-targeted plan adjustments; fingerprint `budget_drift:category:{id}:{month}`; surfaced via alert REST + `/alerts` UI; unit tests `budget_drift_proration_mid_month`, `budget_drift_skips_non_category`. **Operator prerequisite:** active plan with category adjustments + synced MTD transactions. |
| UAT-4 | AC-4 | Plan viability alert fires when active scenario becomes infeasible | **PASS-with-prerequisites** | `evaluate_plan_viability` on `plan_daily_cashflow`; severity escalates on consecutive negative month-ends; `AlertService` binds `plan_computation_id`; alert type `plan_viability` in schema + UI. **Operator prerequisite:** active plan computation with negative projected month-end. |
| UAT-5 | AC-5 | Alert inbox lists active alerts with acknowledge/dismiss | **PASS** | `alerts` lifecycle (`active` → `acknowledged` / `dismissed` / `resolved`); `PATCH /api/v1/alerts/{id}/acknowledge` + `/dismiss`; `GET /api/v1/alerts` + `/unread-count`; `AlertsPage` with Acknowledge/Dismiss; `AlertBell` unread badge + preview popover; unit test `unread_requires_active_unacknowledged`. **Operator smoke:** open `/alerts` and bell after sync to see live alerts. |
| UAT-6 | AC-6 | Grafana Dashboard 4 shows total wealth (non-crypto until US-0007) | **PASS** | `grafana/provisioning/dashboards/analytics/portfolio.json` — uid `portfolio`; panels Total wealth stat, Wealth over time, Account breakdown, Mixed-currency warning, Crypto placeholder ("US-0007"); `/wealth` links to Dashboard 4; snapshots fed by `WealthService::upsert_daily_snapshot`. **Operator smoke:** post-sync snapshot for live panel data. |

## Summary

| Metric | Count |
|--------|-------|
| Acceptance criteria passed (code/automated) | 6/6 |
| Full runtime E2E executed | 0/6 (blocked by missing operator infra) |
| Automated checks passed | 7/12 (5 SKIP — expected without `DATABASE_URL`) |
| Failed | 0 |

## Operator prerequisites (post-release smoke)

1. Copy `.env.example` → `.env`; set `DATABASE_*`, `FIREFLY_*`, optional OIDC or `AUTH_DEV_BYPASS=true`.
2. Provision external TimescaleDB; apply migrations including `005_alerts_wealth.sql`.
3. `docker compose --profile minimal up --build`
4. Complete Firefly setup; create PAT; sync asset accounts with `include_net_worth=true`; run forecast recompute (US-0002).
5. Create active plan with category-targeted adjustments (US-0004) for budget drift alerts.
6. Run **Sync now** — verify post-sync `"alerts"` phase creates/updates alerts.
7. Open `http://localhost:8080/wealth` — net worth stat, breakdown, mixed-currency banner, wealth chart.
8. Open `http://localhost:8080/alerts` and header bell — acknowledge/dismiss active alerts.
9. Open Grafana Analytics dashboard `portfolio` (Dashboard 4) and verify Dashboard 1 `$scarcity_threshold`.
10. Optional: `DATABASE_URL=... cargo test --test wealth_alerts_integration` for snapshot upsert + scarcity alert on post-sync proof.

## Findings

### Blockers

None.

### Observations

1. `wealth_alerts_integration` DB path requires operator `DATABASE_URL` — skipped by design; static Firefly write audit and Dashboard 1 threshold tests pass without DB.
2. Live scarcity, budget drift, and plan viability alert firing depend on forecast baseline, active plan, and synced transactions — documented in `docs/user-guides/US-0005.md`.
3. Mixed-currency headline total sums native balances without FX — mandatory warning banner in React and Grafana (R-0021); by design until US-0007.
4. ECharts main chunk ~1 MB (vite warning); `WealthChart` code-split — acceptable for MVP.
5. Subscription alerts remain on separate surface (US-0003) with cross-link in bell popover — by design (R-0023).

## Next phase

Run `/release` in a fresh release subagent context.
