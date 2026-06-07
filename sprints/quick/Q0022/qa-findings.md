# QA Findings — Quick Q0022 / BUG-0014

**Work item:** BUG-0014 (defect)  
**Quick task:** Q0022  
**QA phase:** `/qa`  
**Date:** 2026-06-07  
**Orchestrator:** `auto-20260607-bug0014-001`  
**Decisions:** DEC-0081, DEC-0082, DEC-0083  
**QA agent:** fresh subagent (`qa-20260607-q0022-bug0014`)  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — Implemented tasks **AO1**, **AQ1**, **AQ2**, **AS1**, **AS2** satisfy DEC-0081/0082/0083 at code and test level. Zero blocking findings. **AP2** and **AR1** correctly skipped (operator gates). Hand off to **`/verify-work`** for omniflow runtime probes (AO/AP/AQ/AR/AS/AT live steps).

## Scope

BUG-0014 post-rebuild omniflow cluster: Grafana dual-scenario ML copy (**AO**), wealth holdings + unified FX incomplete (**AQ**), plan delete guard + target_type UI (**AS**). Conditional **AP2** (crypto subtotal) and **AR1** (cashflow acct 114) deferred per architecture gates. Ops-only **AT** documented for verify-work.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0022/progress.md`, `sprints/quick/Q0022/uat.md`, `sprints/quick/Q0022/tasks.md`, `grafana/provisioning/dashboards/analytics/forecast-horizons.json`, `backend/src/wealth/service.rs`, `backend/src/wealth/types.rs`, `backend/src/portfolio/service.rs`, `backend/src/api/plans.rs`, `backend/src/plan/service.rs`, `frontend/src/pages/WealthPage.tsx`, `frontend/src/pages/PlanningPage.tsx`, `frontend/src/lib/api.ts`, `backend/tests/grafana_provisioning_bug0009.rs`, `decisions/DEC-0081.md`, `decisions/DEC-0082.md`, `decisions/DEC-0083.md`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | Wealth service unit tests | `cargo test --lib wealth::service::tests` | **PASS** (4/4) |
| T-2 | Plan delete API serialization | `cargo test --lib plan_delete_api_tests` | **PASS** (1/1) |
| T-3 | Grafana provisioning (AO1) | `cargo test --test grafana_provisioning_bug0009` | **PASS** (6/6) |
| T-4 | Frontend vitest | `npm test -- --run` | **PASS** (6/6) |
| T-5 | Full lib regression | `cargo test --lib` | **PASS** (177/177) |
| T-6 | **AO1** — Panel 13 dual-scenario copy | Code review + T-3 `forecast_horizons_ml_banner_and_no_value` | **PASS** |
| T-7 | **AQ1** — `holdings_all` cap 50 + unified `fx_incomplete` | Code review `wealth/service.rs` + T-1 | **PASS** |
| T-8 | **AQ2** — WealthPage native qty + EUR table + banner | Code review `WealthPage.tsx` | **PASS** |
| T-9 | **AS1** — DELETE active 409 + confirm modal + disabled active | Code review `plan/service.rs`, `api/plans.rs`, `PlanningPage.tsx` + T-2 | **PASS** |
| T-10 | **AS2** — Five target_type options + help; no `account` | Code review `PlanningPage.tsx`, `api.ts` grep | **PASS** |
| T-11 | **AP2** — Conditional wealth subtotal fix | Skipped — AP1_SQL_PROBE gate not met | **SKIP** (expected) |
| T-12 | **AR1** — Conditional cashflow Grafana fix | Skipped — V1 AR verify prerequisite | **SKIP** (expected) |
| T-13 | V1 omniflow runtime smoke | `sprints/quick/Q0022/uat.md` | **DEFERRED** — verify-work |
| T-14 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |

### Test output (T-1)

```
running 4 tests
test wealth::service::tests::fx_incomplete_when_unpriced_assets_present ... ok
test wealth::service::tests::holdings_all_sorts_priced_before_unpriced ... ok
test wealth::service::tests::overdrawn_flag_when_balance_negative ... ok
test wealth::service::tests::mixed_currency_when_multiple_currencies ... ok
test result: ok. 4 passed; 0 failed
```

### Test output (T-5)

```
test result: ok. 177 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Task verdict matrix

| Task | Status (execute) | QA verdict | Notes |
|------|------------------|------------|-------|
| AO1 | done | **PASS** | Dual-scenario static markdown; US-0013 + stats-forecast named; ML panels `noValue` |
| AQ1 | done | **PASS** | `holdings_all` cap 50; priced-first sort; unified `fx_incomplete` |
| AQ2 | done | **PASS** | Table from `holdings_all`; native qty + EUR; unified FX banner |
| AS1 | done | **PASS** | 409 `active_plan_delete_forbidden`; modal; active delete disabled + tooltip |
| AS2 | done | **PASS** | 5 valid target_types; inline help; `account` removed |
| AP2 | skipped | **N/A** | AP1 gate — evaluate at verify-work post-deploy |
| AR1 | skipped | **N/A** | V1 AR gate — partial probe at verify-work |
| V1 | open | **DEFERRED** | Operator omniflow smoke |

## Acceptance criteria matrix (BUG-0014)

| Row | Criterion | QA verdict | Evidence |
|-----|-----------|------------|----------|
| **AO** | ML available after Full sync or accurate sidecar-down copy | **PASS** (code) / **DEFERRED** (runtime) | Panel 13 markdown scenarios 1+2; US-0013 guide; stats-forecast service named; T-3/T-6 PASS. Live AO-1 **DEFERRED** |
| **AP** | crypto.subtotal_eur > 0 when Bitunix wallet priced | **DEFERRED** | AP2 skipped; AP1 SQL + AP-1 probes at verify-work after Q0020 deploy |
| **AQ** | Native quantities + EUR equivalents; FX incomplete only with documented unpriced_assets | **PASS** (code) / **DEFERRED** (runtime) | DEC-0081 wired; `holdings_all`; `fx_incomplete = pnl \|\| unpriced`; WealthPage table + banner. T-7–T-8 PASS. Live AQ-1 **DEFERRED** |
| **AR** | Cashflow panels non-zero acct 114 after Full sync | **DEFERRED** | AR1 skipped; AR-API/AR-GRAF at verify-work |
| **AS** | Delete non-active plan; 409 on active; valid target_type + help | **PASS** (code) / **DEFERRED** (runtime) | AS1 guard + UI; AS2 five options + help. T-9–T-10 PASS. Live AS-1 **DEFERRED** |
| **AT** | Three-service compose includes stats-forecast when ML enabled | **DEFERRED** (ops) | AT-1 `docker ps` at verify-work; documented in UAT |

## Code review vs decisions

### DEC-0081 (AQ1/AQ2)

| Contract | Status | Evidence |
|----------|--------|----------|
| `holdings_all` cap 50, priced first by `value_eur` desc | **PASS** | `wealth/service.rs` `truncate(50)` + `compare_holdings_all` |
| Row fields: asset, quantity, product_type, value_eur, unrealized_pnl_eur, native_unit | **PASS** | `wealth/types.rs` `HoldingsAllRow` |
| `holdings_top` unchanged (priced top-5) | **PASS** | Existing filter + `truncate(5)` preserved |
| `fx_incomplete = pnl.fx_incomplete \|\| !unpriced_assets.is_empty()` | **PASS** | `wealth/service.rs` line 212 |
| `unpriced_assets` from PnL breakdown | **PASS** | Wired from `portfolio.latest()` |
| WealthPage uses same flag + asset list | **PASS** | `WealthPage.tsx` banner + `holdings_all` table |

### DEC-0082 (AS1)

| Contract | Status | Evidence |
|----------|--------|----------|
| DELETE active plan → 409 Conflict | **PASS** | `plan/service.rs` `ActivePlanDeleteForbidden`; `api/plans.rs` handler |
| Body `{ error: "active_plan_delete_forbidden", message: "…" }` | **PASS** | `plan_error_status` + unit test T-2 |
| Confirm modal with plan name | **PASS** | `PlanningPage.tsx` `deleteConfirmPlan` modal |
| Active plan delete disabled + tooltip | **PASS** | `disabled={activePlanIsSelected}` + title |
| 409 surfaced via mutation feedback | **PASS** | `planningFeedback.test.ts` parses `active_plan_delete_forbidden` |

### DEC-0083 (AS2)

| Contract | Status | Evidence |
|----------|--------|----------|
| Five UI options: household, subscription, category, custom_label, allocation_target | **PASS** | `PlanningPage.tsx` select options |
| `account` removed from select | **PASS** | No `account` in frontend grep |
| Inline help copy | **PASS** | Paragraph below target_type select |
| Types align with backend enum | **PASS** | `api.ts` — no invalid `account` target_type |

### AO1 (DEC-0066 / DEC-0076 extension)

| Contract | Status | Evidence |
|----------|--------|----------|
| Dual-scenario static markdown (not runtime Postgres var) | **PASS** | `forecast-horizons.json` panel content |
| Scenario 1: ML not configured + US-0013 guide | **PASS** | T-3 asserts "ML not configured", "US-0013" |
| Scenario 2: sidecar unreachable + stats-forecast | **PASS** | T-3 asserts "sidecar unreachable", "stats-forecast" |
| ML chart panels `noValue`: "ML unavailable" | **PASS** | T-3 loop over `ml_panel_ids()` |

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

**Informational (non-blocking):**

1. **AP2/AR1 skipped** — per architecture; verify-work must run AP1 SQL probe and partial AR compare before conditional reopen.
2. **Runtime deferred** — Operator gates BACKEND_FRONTEND_DEPLOY, THREE_SERVICE_COMPOSE, FULL_FIREFLY_SYNC, GRAFANA_PROVISIONING_RELOAD pending.
3. **Pre-existing integration drift** — full `cargo test` integration (`firefly_*`) has AppConfig fixture gaps unrelated to Q0022 (per dev handoff).

## Verify-work readiness

| Gate | Status |
|------|--------|
| Code QA (AO1, AQ1, AQ2, AS1, AS2) | **READY** |
| Targeted test suites (T-1–T-4) | **READY** — all PASS |
| `cargo test --lib` | **READY** — 177/177 PASS |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** |
| Operator **THREE_SERVICE_COMPOSE** | **PENDING** |
| Operator **FULL_FIREFLY_SYNC** + acct 114 recompute | **PENDING** |
| Operator **GRAFANA_PROVISIONING_RELOAD** | **PENDING** |
| **AP1_SQL_PROBE** → AP2 decision | **PENDING** |
| V1 omniflow smoke (AO–AT) | **PENDING** |

## Next phase

**`/verify-work`** — deploy bundle, operator gates, then V1 probes per `sprints/quick/Q0022/uat.md`. Evaluate AP1 → AP2 and AR partial probe → AR1.

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
