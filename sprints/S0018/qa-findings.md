# QA findings — S0018 / US-0019

**Story:** US-0019 — Goal-driven planning with per-plan stats & AI savings suggestions  
**Sprint:** S0018  
**Phase:** `/qa`  
**Orchestrator:** `auto-20260608-us0019-001`  
**Decisions:** DEC-0091, DEC-0092, DEC-0093, DEC-0094, DEC-0095, DEC-0096, DEC-0097  
**QA agent:** fresh subagent (`qa-20260609-us0019-qa-fresh`)  
**Date:** 2026-06-09  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — All acceptance criteria AC-1 through AC-6 satisfied via code review and automated tests. **0 blocking findings.** Operator omniflow OIDC smoke (AC-6 live) deferred to `/verify-work` pending **BACKEND_FRONTEND_DEPLOY** and **FULL_FIREFLY_SYNC**. Hand off to `/verify-work`.

## Test plan

| # | Check | Method | Result |
|---|-------|--------|--------|
| 1 | AC-1 goal plan type | Code review migration, create API, PlanningPage template card | **PASS** |
| 2 | AC-2 per-plan statistics | Code review `goal_stats` service/route, `GoalStatsStrip`; vitest | **PASS** |
| 3 | AC-3 category adjustments | Code review `overlay.rs` cap, `goal_baseline_series`; lib tests | **PASS** |
| 4 | AC-4 savings suggestions + operator select | Code review savings API, modal checkbox apply; no auto-apply | **PASS** |
| 5 | AC-5 privacy + audit | Code review aggregate-only savings; AI tool registry; orchestrator audit path | **PASS** |
| 6 | AC-6 US-0014 regression | Code review 7-template grid, PVA guided card, DEC-0089 compare copy | **PASS** (code) |
| 7 | DEC-0091 goal schema | Migration `013_goal_balance.sql`, `PlanTemplate::GoalBalance` | **PASS** |
| 8 | DEC-0092 goal-stats API | `GET …/goal-stats`, yearly rollup, `beyond_horizon`, feasibility copy | **PASS** |
| 9 | DEC-0093 category overlay cap | 3-mo avg cap via `build_category_remove_caps`; lib tests | **PASS** |
| 10 | DEC-0094 savings ranking | Deterministic aggregates, fixed-bucket exclusion, €20/mo min | **PASS** |
| 11 | DEC-0095 goal account | Default max-balance asset on create; per-account projection fork | **PASS** |
| 12 | DEC-0096 PVA scope | PVA endpoint/tab unchanged; goal stats per-plan only | **PASS** |
| 13 | DEC-0097 AI tool | `get_category_savings` seventh tool; wraps same service as REST | **PASS** |
| 14 | Frozen boundaries | No `forecast/project.rs` bucket-inference edits; compare API unchanged | **PASS** |
| 15 | `cargo test --lib` | QA re-run (`backend/`) | **PASS** (204/204) |
| 16 | `npm test -- --run` | QA re-run (`frontend/`) | **PASS** (9/9) |
| 17 | User guide | Code review `docs/user-guides/US-0019.md` | **PASS** |
| 18 | Operator OIDC smoke | `sprints/S0018/uat.md` template | **DEFERRED** → verify-work |

## Automated test output

```
$ cd backend && cargo test --lib
running 204 tests
test result: ok. 204 passed; 0 failed; 0 ignored
EXIT_CODE=0

$ cd frontend && npm test -- --run
Test Files  4 passed (4)
Tests  9 passed (9)
EXIT_CODE=0
```

**US-0019-targeted tests (representative):** `validate_goal_fields_*`, `goal_validation_returns_422`, `not_goal_plan_returns_404`, `beyond_horizon_when_target_beyond_730_days`, `category_remove_outflow_clamped_to_cap`, `category_remove_zero_cap_produces_no_overlay`, `map_suggestion_ranks_entertainment`, `map_suggestion_skips_below_min_spend`, `excludes_existing_category_removals`, `registry_has_seven_tools_including_category_savings`; frontend `GoalStatsStrip` off-track + beyond-horizon copy.

## Acceptance criteria results

| AC | Requirement | Result | Evidence |
|----|-------------|--------|----------|
| AC-1 | Goal plan with target balance + target date; persists in Scenarios list | **PASS** | `013_goal_balance.sql`; `create_plan` 422 guards (`validate_goal_fields`); `PlanningPage.tsx` Goal balance card + `createGoalPlan`; `goal_create_api_tests` |
| AC-2 | Per-plan stats — monthly delta, yearly rollup, projected balance at target — not household on detail | **PASS** | `GET /api/v1/plans/{id}/goal-stats`; `goal_stats` service; `GoalStatsStrip` on Scenarios + Compare when `goal_balance`; PVA tab has no goal strip (DEC-0096) |
| AC-3 | Category-scoped spend adjustments affect compare/PVA after recompute | **PASS** | `overlay.rs` `remove_outflow` cap; `goal_baseline_series` account fork; category add-line form + recompute on apply |
| AC-4 | AI savings suggestions; operator selects to apply — no auto-apply | **PASS** | `category-savings-suggestions` route; modal checkbox + disabled Apply until selection; batch POST adjustments on confirm only |
| AC-5 | Privacy aggregates only; audit log per US-0006 | **PASS** | `savings_service` uses `aggregates_by_category` only; `get_category_savings` tool aggregate-only; chat tool calls audited by orchestrator (DEC-0034). See ADV-1 for modal-apply audit nuance. |
| AC-6 | US-0014 onboarding/templates; OIDC external smoke | **PASS** (code) | 7-template grid incl. Goal balance + Custom; PVA `no_active_plan` guided card; compare CategoryTrendChart actuals-only copy; operator live smoke **deferred** |

## Architecture decision alignment

| DEC | Contract | Result | Notes |
|-----|----------|--------|-------|
| DEC-0091 | `goal_balance` enum + plan columns | **PASS** | Migration + `PlanRow` fields |
| DEC-0092 | goal-stats API, calendar yearly, gap copy, 730d guard | **PASS** | `beyond_horizon` tested; feasibility 0% interest copy |
| DEC-0093 | Category remove cap 3-mo avg; add household-labeled | **PASS** | `build_category_remove_caps` + overlay clamp tests |
| DEC-0094 | Deterministic ranking; fixed exclusion; modal apply | **PASS** | 50% reduction hint; evidence summary; no LLM ranking |
| DEC-0095 | Optional `goal_account_id`; default max-balance asset | **PASS** | `default_goal_account_id` SQL; projection fork in `goal_baseline_series` |
| DEC-0096 | PVA household active plan unchanged | **PASS** | No PVA route/UI scope change |
| DEC-0097 | REST primary; optional seventh AI tool | **PASS** | T-0196 shipped; registry test confirms 7 tools |

## Findings summary

| ID | Severity | Finding | Blocking |
|----|----------|---------|----------|
| ADV-1 | Advisory | DEC-0094 §4 / T-0193 mention audit on category-savings modal apply (`POST` adjustments); no dedicated audit row beyond standard plan adjustment persist — consistent with all other plan mutations (no plan-module audit). AI tool invocations use orchestrator audit. | No |

**Blocking findings:** 0  
**Critical findings:** 0  
**Advisory (non-blocking):** 1 (ADV-1)

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
| `fresh_context_marker` | `qa-20260609-us0019-qa-fresh` |
| `runtime_proof_id` | `runtime-proof-qa-20260609-us0019-001` |
| `phase_boundary` | qa → verify-work |
| `isolation_scope` | QA fresh subagent; artifact/handoff reads only; no prior chat history; no host `.env` / operator secrets read; verify-work not started |

## Handoff

**Next phase:** `/verify-work` in fresh subagent/chat (role: qa)  
**Stop reason:** QA_PASS — no `handoffs/qa_to_dev.md` required
