# QA findings — S0017 / US-0018

**Story:** US-0018 — Category filters & expense trend analytics  
**Sprint:** S0017  
**Phase:** `/qa`  
**Orchestrator:** `auto-20260608-us0018-001`  
**Decisions:** DEC-0087, DEC-0088, DEC-0089, DEC-0090  
**QA agent:** fresh subagent (`qa-20260608-us0018-qa-fresh`)  
**Date:** 2026-06-08  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — All acceptance criteria AC-1 through AC-6 satisfied via code review and automated tests. **0 blocking findings.** T-0185 (EXPLAIN probe / conditional index) deferred per DEC-0090 — gate not triggered. Operator omniflow OIDC smoke (AC-6 live) deferred to `/verify-work` pending **BACKEND_FRONTEND_DEPLOY**, **FULL_FIREFLY_SYNC**, **GRAFANA_PROVISIONING_RELOAD**. Hand off to `/verify-work`.

## Test plan

| # | Check | Method | Result |
|---|-------|--------|--------|
| 1 | AC-1 shared filter surfaces | Code review Forecast/Planning/Wealth + Grafana `$category` | **PASS** |
| 2 | AC-2 expense-series API | Code review `categories.rs`, `service.rs`, `repository.rs`; lib tests | **PASS** |
| 3 | AC-3 trend chart UI | Code review `CategoryTrendChart.tsx`; vitest | **PASS** |
| 4 | AC-4 MoM / best / worst | Code review `compute_expense_series_summary`, chart callouts | **PASS** |
| 5 | AC-5 mirror fidelity + uncategorized | Code review sentinel `__uncategorized__`; lib tests | **PASS** |
| 6 | AC-6 regression / US-0015 | Grep: no `project.rs` edits; forecast monthly API unchanged; AI badge path intact | **PASS** (code) |
| 7 | DEC-0087 API contract | Code review catalog + expense-series endpoints | **PASS** |
| 8 | DEC-0088 filter + bar chart | Code review `CategoryFilter`, `CategoryTrendChart` | **PASS** |
| 9 | DEC-0089 cross-surface semantics | Code review DEC-0089 copy on Forecast/Planning; Grafana independence | **PASS** |
| 10 | DEC-0090 index deferral | T-0185 deferred; no migration shipped | **PASS** (gate-documented) |
| 11 | Baseline `cargo test --lib` | QA re-run | **PASS** (193/193) |
| 12 | Baseline `npm test -- --run` | QA re-run | **PASS** (7/7) |
| 13 | User guide (USER_GUIDE_MODE=1) | Code review `docs/user-guides/US-0018.md` | **PASS** |
| 14 | User-visible metadata guard | `scripts/check-user-visible-metadata.py` | **SKIP** (entrypoint absent; repo precedent S0013/Q0020) |
| 15 | Integration `us0018_categories.rs` | Requires `DATABASE_URL` | **SKIP** (sandbox; lib tests cover spine/summary/sentinel) |
| 16 | Operator OIDC smoke | `sprints/S0017/uat.json` | **DEFERRED** → verify-work |

## Automated test output

```
$ cd backend && cargo test --lib
running 193 tests
test result: ok. 193 passed; 0 failed; 0 ignored
EXIT_CODE=0

$ cd frontend && npm test -- --run
Test Files  3 passed (3)
Tests  7 passed (7)
EXIT_CODE=0
```

**US-0018-targeted lib tests (representative):** `expense_series_window_twelve_months`, `expense_series_window_rejects_over_cap_via_types`, `compute_summary_mom_best_worst`, `compute_summary_zero_spine_months`, `uncategorized_sentinel_constant`, `validate_months_cap`; frontend `CategoryTrendChart` empty-selection prompt.

## Acceptance criteria results

| AC | Requirement | Result | Evidence |
|----|-------------|--------|----------|
| AC-1 | Shared category filter on `/forecast` monthly, `/planning` compare, `/wealth` breakdown, and ≥2 Grafana dashboards | **PASS** | `ForecastPage.tsx` Monthly tab `CategoryFilter` + `CategoryTrendChart` (lines 268–340); `PlanningPage.tsx` compare toolbar filter + widget (696–719); `WealthPage.tsx` "Category spending" subsection (173–188); Grafana `cashflow.json` + `budgets.json` `$category` variable and filtered SQL |
| AC-2 | `GET` category expense API: per-month EUR outflow/inflow, default 12 months, max 24 | **PASS** | `GET /api/v1/categories/expense-series` in `api/categories.rs`; `EXPENSE_SERIES_DEFAULT_MONTHS=12`, `EXPENSE_SERIES_MAX_MONTHS=24` in `types.rs`; month-spine SQL in `repository.rs::expense_series_by_month` |
| AC-3 | React trend chart: month labels + EUR amounts; single category; empty-state | **PASS** | `CategoryTrendChart.tsx` ECharts bar chart with `monthLabels` / `formatEur`; disabled prompt when `categoryId` empty; empty copy "No categorized spending in this period"; vitest confirms prompt |
| AC-4 | MoM change and best/worst month indicator | **PASS** | Server `compute_expense_series_summary` (`mom_delta_pct`, `best_month`, `worst_month`); UI stat cards MoM / Highest / Lowest month in `CategoryTrendChart.tsx` |
| AC-5 | Mirror `category_id`; explicit uncategorized bucket | **PASS** | SQL filters `t.category_id` / `IS NULL`; sentinel `__uncategorized__` → `uncategorized: true`, `category_label: "Uncategorized"` in `service.rs`; full spine with €0 months |
| AC-6 | OIDC regression; read-only Firefly; US-0015 bucket mapping unchanged | **PASS** (code) | No `forecast/project.rs` changes; compare API has no `category_id` param; US-0015 AI-mapped badge block preserved in `ForecastPage.tsx`; operator live smoke **deferred** |

## Architecture decision alignment

| DEC | Contract | Result | Notes |
|-----|----------|--------|-------|
| DEC-0087 | Catalog + expense-series API, `__uncategorized__`, server summary | **PASS** | `api/categories.rs`, `transactions/service.rs`, `types.rs` |
| DEC-0088 | Single-select `CategoryFilter`, bar `CategoryTrendChart` | **PASS** | Shared components; combobox when >20 categories |
| DEC-0089 | Forecast actuals-only filter; planning widget; Grafana independent | **PASS** | Household forecast cards/chart unchanged; DEC-0089 copy on pages; no iframe sync |
| DEC-0090 | No index unless EXPLAIN >50 ms | **PASS** (deferred) | T-0185 skipped — no operator mirror; MVP sequential scan accepted |

## Findings summary

| ID | Severity | Finding | Blocking |
|----|----------|---------|----------|
| — | — | No findings | — |

**Blocking findings:** 0  
**Critical findings:** 0  
**Advisory (non-blocking):** Operator omniflow smoke for AC-1/AC-6 live paths deferred per sprint UAT template and prior story precedent.

## Generated baseline test evidence (US-0066)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | rust+node |
| `generated_test_command` | `cargo test --lib`; `npm test -- --run` |
| `generated_test_result` | pass |
| `generated_test_output_ref` | this file § Automated test output |
| `generated_test_paths_ref` | `backend/` lib tests; `frontend/src/**/*.test.ts(x)` |
| `generated_test_reason_code` | — |

## Runtime QA evidence (US-0065)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | n/a (code-review + unit-test QA scope) |
| `runtime_stack_profile` | rust axum + react vitest |
| `runtime_mode` | local |
| `runtime_health_target` | n/a |
| `runtime_health_result` | n/a |
| `runtime_log_summary` | n/a |
| `runtime_retry_count` | 0 |
| `runtime_retry_ledger` | [] |
| `runtime_final_verdict` | pass |
| `runtime_reason_code` | — |
| `runtime_evidence_refs` | `handoffs/dev_to_qa.md`, automated test output above |

## Isolation / proof

| Field | Value |
|-------|-------|
| `fresh_context_marker` | `qa-20260608-us0018-qa-fresh` |
| `runtime_proof_id` | `runtime-proof-qa-20260608-us0018-001` |
| `phase_boundary` | qa → verify-work |
| `isolation_scope` | QA fresh subagent; artifact/handoff reads only; no prior chat history; no host `.env` / operator secrets read; verify-work not started |

## Handoff

**Next phase:** `/verify-work` in fresh subagent/chat (role: qa)  
**Stop reason:** QA_PASS — no `handoffs/qa_to_dev.md` required
