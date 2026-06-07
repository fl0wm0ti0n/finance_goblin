# QA Findings — Quick Q0020 / BUG-0013

**Work item:** BUG-0013 (defect)  
**Quick task:** Q0020  
**QA phase:** `/qa`  
**Date:** 2026-06-08  
**Orchestrator:** `auto-20260608-bug0013-001`  
**Decisions:** DEC-0079, DEC-0080  
**QA agent:** fresh subagent (`qa-20260608-q0020-bug0013`)  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — Acceptance rows **AL**, **AN**, and **AK** satisfied at code/test level per DEC-0079 and DEC-0080. Zero blocking findings. Hand off to **`/verify-work`** (V1 omniflow runtime probes deferred until operator deploy + Full sync).

## Scope

BUG-0013 omniflow analytics regression cluster: budgets MTD upper date bound (**AL**), Bitunix wallet array parse + linear unrealized USDT→EUR (**AN/AK**), optional Grafana copy (**AJ**, **AK2**), UAT smoke template (**V1**) per `handoffs/dev_to_qa.md` and `sprints/quick/Q0020/summary.md`.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0020/summary.md`, `sprints/quick/Q0020/tasks.md`, `sprints/quick/Q0020/uat.md`, `grafana/provisioning/dashboards/analytics/budgets.json`, `grafana/provisioning/dashboards/analytics/subscriptions.json`, `grafana/provisioning/dashboards/analytics/portfolio.json`, `backend/src/exchanges/bitunix.rs`, `backend/src/portfolio/pnl.rs`, `backend/src/exchanges/repository.rs`, `decisions/DEC-0079.md`, `decisions/DEC-0080.md`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Full lib regression | `cd backend && cargo test --lib` | **PASS** (174/174) |
| T-2 | AN1 Bitunix unit tests | `cargo test --lib exchanges::bitunix::tests::resolve_futures_account_array_shape sync_balances_futures_wallet_array_shape sync_positions_linear_unrealized_pnl_casing` | **PASS** (3/3) |
| T-3 | AN1 PnL parser tests | `cargo test --lib portfolio::pnl::tests::parse_unrealized_pnl_usdt` | **PASS** (2/2) |
| T-4 | **AL** — DEC-0079 MTD SQL cap | Code review panel id 5 `budgets.json` | **PASS** |
| T-5 | **AN** — DEC-0080 wallet array ingest | Code review `resolve_futures_account`, `parse_futures_wallet` | **PASS** |
| T-6 | **AN/AK** — DEC-0080 linear unrealized EUR | Code review `compute_hybrid_pnl` linear branch | **PASS** |
| T-7 | **AJ** — Subscriptions empty-state copy | Code review `subscriptions.json` panel description | **PASS** |
| T-8 | **AK2** — Performance % min-snapshot footnote | Code review `portfolio.json` `noValue` + description | **PASS** |
| T-9 | User-visible metadata guard | `scripts/check-user-visible-metadata.py` | **SKIP** — entrypoint absent (repo precedent S0013/Q0018) |
| T-10 | V1 omniflow runtime smoke | `sprints/quick/Q0020/uat.md` probes | **DEFERRED** — verify-work after operator gates |
| T-11 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |

### Test output (T-1)

```
test result: ok. 174 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Note (T-1):** First parallel run reported 173/174 (flake: `config::tests::effective_enabled_futures_auto_when_creds_present` — env-var pollution under parallel tests, pre-existing, out of Q0020 scope). Re-run parallel and single-threaded both **174/174 PASS**.

## Generated baseline test evidence (US-0066 / DEC-0048)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | rust |
| `generated_test_command` | `cd backend && cargo test --lib` |
| `generated_test_result` | pass |
| `generated_test_output_ref` | T-1 above (174/174) |
| `generated_test_paths_ref` | `backend/src/exchanges/bitunix.rs`, `backend/src/portfolio/pnl.rs` |
| `generated_test_reason_code` | _(none — pass)_ |

## Runtime QA evidence (US-0065 / DEC-0047)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | _(not run — operator gates pending)_ |
| `runtime_stack_profile` | rust + grafana-provisioned postgres dashboards |
| `runtime_mode` | local (unit tests only) |
| `runtime_health_target` | `financegnome.omniflow.cc` — deferred to verify-work |
| `runtime_health_result` | deferred |
| `runtime_log_summary` | n/a |
| `runtime_retry_count` | 0 |
| `runtime_retry_ledger` | [] |
| `runtime_final_verdict` | deferred |
| `runtime_reason_code` | `OPERATOR_GATES_PENDING` |
| `runtime_evidence_refs` | `sprints/quick/Q0020/uat.md`, `handoffs/dev_to_qa.md` |

**Environment label:** local (tests ran locally; no omniflow runtime probes in QA).

## Acceptance criteria matrix (BUG-0013)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **AL** | Budgets MTD plan/actual/deviation plausible — not −€150K artifact | **PASS** (code) / **DEFERRED** (runtime) | Panel id 5 `planned` CTE: `pdc.ts >= date_trunc('month', CURRENT_DATE) AND pdc.ts::date <= CURRENT_DATE`; deviation uses capped planned; optional mid-month footnote in panel description. Tests T-4 PASS. Live AL-1 **DEFERRED** |
| **AN** | Exchange crypto balances in wealth/portfolio when sync succeeds | **PASS** (code) / **DEFERRED** (runtime) | `resolve_futures_account` handles `data[]` array; `unrealizedPNL` in wallet/position keys; futures USDT equity via stablecoin path; 5 new unit tests PASS. Live AN-1 **DEFERRED** |
| **AK** | Crypto value reflects holdings; FX warning only with gaps; performance % when snapshots exist | **PASS** (code) / **DEFERRED** (runtime) | Linear: unrealized USDT→EUR, excluded from `crypto_value_eur`; futures priced via `fx.to_eur(qty, asset)`; AK2 `noValue` "Needs ≥2 snapshots" + description. Tests T-6, T-8 PASS. Live AK-1/AK-2 **DEFERRED** |
| **AI** | Cashflow + forecast-horizons baseline non-empty | **DEFERRED** | No Q0020 code change; V1 smoke after Full sync |
| **AJ** | Price changes empty-state or event rows | **PASS** (code) / **DEFERRED** (runtime) | Panel description documents 90d empty table expectation. Test T-7 PASS. Live AJ-1 **DEFERRED** |
| **AM** | ds/query + annotations 200 | **WAIVED** | Per R-0077 unless HAR shows failure |

## Code review vs decisions

### DEC-0079 (AL1)

| Contract | Status | Evidence |
|----------|--------|----------|
| `planned` CTE upper bound `pdc.ts::date <= CURRENT_DATE` | **PASS** | `budgets.json` panel 5 `rawSql` |
| `actual` CTE unchanged | **PASS** | No upper bound on transactions |
| Deviation MTD = actual − capped planned | **PASS** | `(SELECT total FROM actual) - (SELECT total FROM planned)` |
| Panel id 5 only | **PASS** | Plan time-series / deviation chart panels unchanged |
| Optional mid-month footnote | **PASS** | Panel `description` documents elapsed-days semantics |

### DEC-0080 (AN1)

| Contract | Status | Evidence |
|----------|--------|----------|
| Array wallet shape (`data[]` → first marginCoin/available element) | **PASS** | `resolve_futures_account` |
| Object shape preserved (`data.account`) | **PASS** | `data.get("account").unwrap_or(data)` fallback |
| `unrealizedPNL` in wallet + position key lists | **PASS** | `parse_futures_wallet`, `parse_futures_positions`, `parse_unrealized_pnl_usdt` |
| Futures `market_value_eur` via USDT/USDC stable path | **PASS** | `holding_value_eur` → `fx.to_eur(qty, asset)` |
| Linear excluded from `crypto_value_eur`; unrealized USDT→EUR | **PASS** | `compute_hybrid_pnl` linear branch with `continue` before crypto sum |
| Linear does not add symbol to `unpriced_assets` | **PASS** | Linear handled before `holding_value_eur` Unpriced path |
| Unit tests for array wallet + linear unrealized | **PASS** | 5 tests listed in dev handoff — all green |

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

**Informational (non-blocking):**

1. **Test flake:** `effective_enabled_futures_auto_when_creds_present` may fail under parallel env-var contention — pre-existing; not introduced by Q0020.
2. **Runtime deferred:** Operator must complete **BACKEND_FRONTEND_DEPLOY**, **GRAFANA_PROVISIONING_RELOAD**, **FULL_FIREFLY_SYNC** before V1 omniflow smoke.

## Verify-work readiness

| Gate | Status |
|------|--------|
| Code QA (AL1, AN1, AJ1, AK2) | **READY** |
| `cargo test --lib` | **READY** — 174/174 PASS |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** |
| Operator **GRAFANA_PROVISIONING_RELOAD** | **PENDING** |
| Operator **FULL_FIREFLY_SYNC** | **PENDING** |
| V1 omniflow smoke (AI–AN rows) | **PENDING** — blocked on deploy |

## Next phase

**`/verify-work`** — operator deploy + Grafana reload + Full sync, then V1 probes per `sprints/quick/Q0020/uat.md`.

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
