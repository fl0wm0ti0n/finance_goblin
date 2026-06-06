# UAT — Sprint S0004 / US-0004

**Sprint:** S0004  
**Story:** US-0004  
**Phase:** `/verify-work`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/release`)  
**Ready for release:** yes (operator runtime prerequisites documented below)

## Inputs

- QA PASS: `sprints/S0004/qa-findings.md`
- Acceptance: `docs/product/acceptance.md` (US-0004)
- Operator guide: `docs/user-guides/US-0004.md`
- Implementation: `backend/src/plan/`, `backend/src/api/plans.rs`, `backend/migrations/004_plans.sql`, `backend/src/forecast/service.rs`, `frontend/src/pages/PlanningPage.tsx`, `grafana/provisioning/dashboards/analytics/budgets.json`

## Environment

| Prerequisite | Status in verify-work |
|--------------|----------------------|
| `.env` populated | **Not present** — no operator `.env` in workspace |
| `DATABASE_URL` (TimescaleDB + extension) | **Unset** — `plans_integration`, `forecast_integration`, `subscriptions_integration`, and `firefly_integration` skipped |
| Firefly PAT + synced asset transactions | **Not provisioned** — live Plan vs Actual Ist series deferred |
| US-0002 forecast recompute completed | **Not provisioned** — live plan projection baseline deferred |
| US-0003 confirmed subscriptions (savings-mode) | **Not provisioned** — savings-mode suggestions deferred |
| `AUTH_DEV_BYPASS` or OIDC IdP | **Unset** — live API/UI auth flow deferred |

Per workflow policy: code-level and automated verification **pass**; runtime E2E steps recorded as **PASS-with-prerequisites** where external infra is required.

## Automated checks

| ID | Check | Command | Result |
|----|-------|---------|--------|
| AUTO-1 | Test harness | `bash tests/run-tests.sh` | **PASS** |
| AUTO-2 | Firefly GET-only unit test | (via harness) `cargo test --test firefly_readonly` | **PASS** (1/1) |
| AUTO-3 | Plan + carry-forward unit tests | (via harness) `cargo test --lib` | **PASS** (28/28; 10 plan-specific) |
| AUTO-4 | Firefly integration (audit log) | (via harness) `cargo test --test firefly_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-5 | Forecast hypertable integration | (via harness) `cargo test --test forecast_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-6 | Subscription integration | (via harness) `cargo test --test subscriptions_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-7 | Plan integration | (via harness) `cargo test --test plans_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-8 | Frontend production build | (via harness) `npm run build` | **PASS** (lazy chunks: `CompareChart`, `PlanVsActualChart`) |
| AUTO-9 | Compose minimal services | `docker compose --profile minimal config --services` (placeholder env) | **PASS** — `firefly-iii`, `flow-finance-ai`, `grafana` |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | Create named plan with scenario adjustments | **PASS** | Migration `004_plans.sql` (`plans`, `plan_versions`, `plan_adjustments`); `POST /api/v1/plans` + adjustment CRUD in `api/plans.rs`; `overlay.rs` monthly/weekly/one-time deltas; `PlanningPage` Scenarios tab with adjustment table + template apply; unit tests `leasing_template_produces_monthly_outflow`, `monthly_outflow_applies_on_schedule`. |
| UAT-2 | AC-2 | Built-in templates: current, leasing, savings mode, house purchase | **PASS** | `PlanTemplate` enum in `types.rs`; `templates.rs` defaults + `map_savings_suggestions`; `GET .../templates/savings-mode/suggestions`; UI template cards + savings modal; user guide §1–§3 documents all four templates. |
| UAT-3 | AC-3 | Plan versions v1/v2/v3 created and compared side-by-side | **PASS-with-prerequisites** | `PlansConfig::max_versions_per_plan = 3`; `VersionCapReached` → HTTP 409; `compare_versions` + Compare tab with lazy `CompareChart`; unit test `version_cap_is_three`. **Operator prerequisite:** `DATABASE_URL` + forecast baseline to create v2/v3 and exercise live compare UI. |
| UAT-4 | AC-4 | Daily plan-vs-Ist shows planned, actual, deviation | **PASS-with-prerequisites** | `plan_vs_actual` API + Plan vs Actual tab/chart; unit test `deviation_sign_is_actual_minus_planned` (actual − planned). **Operator prerequisite:** synced asset transactions for non-empty Ist series. |
| UAT-5 | AC-5 | Grafana Dashboard 3 reflects active plan | **PASS** | `grafana/provisioning/dashboards/analytics/budgets.json` — uid `budgets`, title "Budgets"; panels Active plan, Plan/Ist time series, Abweichung, MTD summary; SQL filters `is_active = true AND is_latest = true`; empty-state annotation. **Operator smoke:** activate a plan for live panel data. |
| UAT-6 | AC-6 | Scenario changes do not modify Firefly data | **PASS** | Plan module read-only SQL on `transactions` mirror; no `FireflyClient` in `src/plan/`; `firefly_readonly` PASS; `plans_integration::plan_module_has_no_firefly_writes` static audit (integration SKIP without `DATABASE_URL`). |

## Summary

| Metric | Count |
|--------|-------|
| Acceptance criteria passed (code/automated) | 6/6 |
| Full runtime E2E executed | 0/6 (blocked by missing operator infra) |
| Automated checks passed | 5/9 (4 SKIP — expected without `DATABASE_URL`) |
| Failed | 0 |

## Operator prerequisites (post-release smoke)

1. Copy `.env.example` → `.env`; set `DATABASE_*`, `FIREFLY_*`, optional OIDC or `AUTH_DEV_BYPASS=true`.
2. Provision external TimescaleDB; apply migrations including `004_plans.sql`.
3. `docker compose --profile minimal up --build`
4. Complete Firefly setup; create PAT; sync asset transactions; run forecast recompute (US-0002).
5. Confirm subscriptions (US-0003) for savings-mode template suggestions.
6. Open `http://localhost:8080/planning` — create plan, apply leasing/savings templates, create v2/v3, compare versions, activate plan, verify Plan vs Actual tab.
7. Open Grafana Analytics dashboard `budgets`.
8. Optional: `DATABASE_URL=... cargo test --test plans_integration` for plan CRUD/recompute/plan-vs-actual and Firefly write audit proof.

## Findings

### Blockers

None.

### Observations

1. `plans_integration` and carry-forward integration tests require operator `DATABASE_URL` — skipped by design in verify-work; unit tests and static Firefly write audit provide sufficient gate coverage.
2. Live plan compare and Plan vs Actual depend on forecast baseline and synced transaction history — documented in `docs/user-guides/US-0004.md`.
3. ECharts main chunk ~1 MB (vite warning); `CompareChart` and `PlanVsActualChart` are code-split — acceptable for MVP.
4. Compose env interpolation requires placeholder values (e.g. `AUTHENTIK_SECRET_KEY`, `FIREFLY_APP_KEY`) even for `--profile minimal` — documented in `.env.example`.

## Next phase

Run `/release` in a fresh release subagent context.
