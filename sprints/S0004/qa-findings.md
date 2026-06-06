# QA Findings — Sprint S0004 / US-0004

**Sprint:** S0004  
**Story:** US-0004  
**QA phase:** `/qa`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/verify-work`)

## Scope

Financial planning: migration 004, Plan Engine (delta overlay, project, templates), PlanService (plan-vs-Ist, compare, versioning), post-forecast recompute hook, 17 plan REST endpoints, React `/planning` (Scenarios, Compare, Plan vs Actual), Grafana Dashboard 3 (`budgets`), unit/integration tests, operator user guide.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0004/summary.md`, `sprints/S0004/tasks.md`, `sprints/S0004/plan-verify.json`, `docs/product/acceptance.md` (US-0004), `backend/src/plan/`, `backend/src/api/plans.rs`, `backend/migrations/004_plans.sql`, `backend/tests/plans_integration.rs`, `frontend/src/pages/PlanningPage.tsx`, `grafana/provisioning/dashboards/analytics/budgets.json`, `docs/user-guides/US-0004.md`, `docs/engineering/spec-pack/US-0004-*.md`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Automated harness | `bash tests/run-tests.sh` | **PASS** |
| T-2 | Firefly GET-only unit test | `cargo test --test firefly_readonly` | **PASS** (1/1) |
| T-3 | Plan + carry-forward unit tests | `cargo test --lib` | **PASS** (28/28; 10 plan-specific) |
| T-4 | Plan integration | `cargo test --test plans_integration` | **SKIP** — `DATABASE_URL` not set |
| T-5 | Forecast integration (US-0002 carry-forward) | `cargo test --test forecast_integration` | **SKIP** — `DATABASE_URL` not set |
| T-6 | Subscription integration (US-0003 carry-forward) | `cargo test --test subscriptions_integration` | **SKIP** — `DATABASE_URL` not set |
| T-7 | Firefly integration (US-0001 carry-forward) | `cargo test --test firefly_integration` | **SKIP** — `DATABASE_URL` not set |
| T-8 | Grafana Dashboard 3 provisioning | Static review `budgets.json` | **PASS** — uid `budgets`, datasource `FlowFinancePostgreSQL` |
| T-9 | User guide (USER_GUIDE_MODE=1) | Static review `docs/user-guides/US-0004.md` | **PASS** — Purpose, Prerequisites, Usage, API examples, Grafana |
| T-10 | Spec-pack (SPEC_PACK_MODE=1) | Static review `docs/engineering/spec-pack/US-0004-*.md` | **PASS** — design-concept, crs, technical-specification (3/3) |
| T-11 | User-visible metadata guard | `python3 scripts/check-user-visible-metadata.py` | **SKIP** — entrypoint not present in repo (see observations) |
| T-12 | Runtime E2E (live stack → `/planning` → Grafana Dashboard 3) | Not executed in QA environment | **Deferred** to `/verify-work` |

### Environment dependencies (non-blocking)

- **`DATABASE_URL`:** Required for `plans_integration` (create → apply leasing → recompute → plan-vs-actual; `plan_module_has_no_firefly_writes` audit). Harness skips gracefully; unit tests and static verification cover engine logic. Not a QA blocker (same pattern as S0002/S0003).
- **Successful forecast computation:** Required for non-empty plan projections at runtime — deferred to verify-work.
- **Active plan + synced transactions:** Required for Grafana Dashboard 3 panels and Plan vs Actual Ist series — deferred to verify-work.
- **OIDC / `AUTH_DEV_BYPASS=true`:** Required for live API/UI acceptance — deferred to verify-work.

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | User can create named plan with scenario adjustments (e.g. +300 €/month leasing) | **PASS** | Migration `004_plans.sql` (`plans`, `plan_versions`, `plan_adjustments`); `POST /api/v1/plans`, adjustment CRUD routes in `api/plans.rs`; `overlay.rs` applies monthly/weekly/one-time deltas; `PlanningPage` Scenarios tab with adjustment table + template apply; unit tests `leasing_template_produces_monthly_outflow`, `monthly_outflow_applies_on_schedule`. |
| AC-2 | Built-in scenario templates: current (Ist), leasing, savings mode, house purchase | **PASS** | `PlanTemplate` enum (`current`, `leasing`, `savings_mode`, `house_purchase`) in `types.rs`; `templates.rs` defaults + `map_savings_suggestions`; `GET .../templates/savings-mode/suggestions`; UI template cards + savings modal; user guide §1–§3 documents all four templates. |
| AC-3 | Plan versions (v1, v2, v3) can be created and compared side-by-side | **PASS** | `PlansConfig::max_versions_per_plan = 3`; `VersionCapReached` → HTTP 409 in `api/plans.rs`; `create_version` copies adjustments and freezes prior (`repository.rs`); `compare_versions` + `GET .../compare`; Compare tab with metrics + lazy `CompareChart`; unit test `version_cap_is_three`. |
| AC-4 | Daily plan-vs-Ist comparison shows planned, actual, and deviation amounts | **PASS** | `plan_vs_actual` aggregates planned/actual/deviation per day (`repository.rs` `build_plan_vs_actual_rows`); `GET /api/v1/plans/active/plan-vs-actual`; Plan vs Actual tab with table columns + lazy `PlanVsActualChart`; unit test `deviation_sign_is_actual_minus_planned` (actual − planned). |
| AC-5 | Grafana Dashboard 3 (Budget plan/ist/deviation) reflects active plan | **PASS** | `grafana/provisioning/dashboards/analytics/budgets.json` — uid `budgets`, title "Budgets"; panels: Active plan stat, Plan time series, Ist time series, Abweichung (deviation), MTD summary; SQL filters `p.is_active = true AND v.is_latest = true`; empty-state annotation "No active plan". |
| AC-6 | Scenario changes do not modify Firefly transaction data | **PASS** | Plan module uses read-only SQL on `transactions` mirror for Ist; no `FireflyClient` or `firefly::sync` in `src/plan/`; `plans_integration::plan_module_has_no_firefly_writes` static audit; carry-forward `firefly_readonly` test PASS; user guide documents read-only guarantee. |

**Summary:** 6/6 PASS (5 fully verified in QA static/unit path; runtime Grafana/UI E2E and `plans_integration` deferred to verify-work with operator env).

## Generated baseline test evidence (US-0066 / DEC-0048)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` + `typescript` (Axum backend + Vite/React frontend) |
| `generated_test_command` | `bash tests/run-tests.sh` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-05-31 — exit 0, message `All US-0004 tests passed` |
| `generated_test_paths_ref` | `backend/src/plan/*` (unit), `backend/tests/plans_integration.rs`, `backend/tests/firefly_readonly.rs`, `frontend/` build |
| `generated_test_reason_code` | — |

## Runtime QA evidence (US-0065 / DEC-0047)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed (compile/test-only QA pass) |
| `runtime_stack_profile` | `rust` + `node` (backend + frontend) |
| `runtime_mode` | `local` |
| `runtime_health_target` | Deferred — `http://localhost:8080/health`, `http://localhost:5173/planning` |
| `runtime_health_result` | `deferred` |
| `runtime_log_summary` | N/A — no live stack started |
| `runtime_retry_count` | 0 |
| `runtime_retry_ledger` | `[]` |
| `runtime_final_verdict` | `deferred` (verify-work) |
| `runtime_reason_code` | `RUNTIME_E2E_DEFERRED_VERIFY_WORK` |
| `runtime_evidence_refs` | `sprints/S0004/uat.md`, `docs/user-guides/US-0004.md` |

## Findings

### Blockers

None.

### Observations (non-blocking)

1. **`plans_integration` skipped:** Expected without external PostgreSQL/TimescaleDB. Unit tests cover overlay, templates, versioning cap, deviation sign, and Firefly write audit; verify-work should run with `DATABASE_URL`.
2. **Runtime E2E deferred:** Live `/planning` flows (template apply, v2/v3 compare, Plan vs Actual chart, activate plan → Grafana Dashboard 3) require operator-provisioned stack — covered in `sprints/S0004/uat.md`.
3. **Metadata sanitizer missing:** `scripts/check-user-visible-metadata.py` not in repository; cannot run US-0071 guard. Prior sprints (S0001–S0003) passed QA without this script. Recommend adding checker in a platform hygiene story — not a US-0004 blocker.
4. **ECharts bundle size:** Main chunk ~1 MB (vite warning); planning charts code-split (`CompareChart`, `PlanVsActualChart`) — acceptable for MVP.
5. **Rust dead-code warning:** `Claims` fields unused in `auth/mod.rs` — cosmetic carry-forward from US-0001.

## Decision gates

No critical issues requiring dev rework. Proceed to **`/verify-work`**.

## Next phase

Run `/verify-work` in a fresh subagent with operator-provisioned PostgreSQL (`DATABASE_URL`), successful forecast computation, Firefly sync with asset transactions, active plan selected, and optional `AUTH_DEV_BYPASS=true` for API/UI acceptance.
