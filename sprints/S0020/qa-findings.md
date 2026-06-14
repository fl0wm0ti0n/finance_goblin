# QA findings — S0020 / US-0021

**Story:** US-0021 — Subscription transaction explorer with rich filters  
**Sprint:** S0020  
**Phase:** `/qa`  
**Orchestrator:** `auto-20260613-us0021`  
**Decisions:** DEC-0112, DEC-0113, DEC-0114 (extends DEC-0098, DEC-0099, DEC-0111)  
**QA agent:** fresh subagent (`qa-20260613-us0021-qa-fresh`)  
**Date:** 2026-06-13  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — Independent re-run and code review confirm AC-1..AC-5 implementation under **DEC-0112** / **DEC-0113** / **DEC-0114**. Automated gates: `cargo test --lib` **221/221**, `cargo test --test us0021_transaction_search` **6/6**, `npm test` **17/17**, `npm run build` **PASS**. **0 blocking findings.** **V1** runtime smoke (AC-1..AC-6 OIDC + account **114** SEPA-Lastschrift fixture) deferred to `/verify-work` pending operator **BACKEND_FRONTEND_DEPLOY**.

**Blockers:** 0

## Scope

**Inputs reviewed:** `handoffs/dev_to_qa.md` (US-0021 top section), `sprints/S0020/{summary,progress,tasks,uat}.md`, `docs/product/acceptance.md` § US-0021, `decisions/DEC-0112.md`, `DEC-0113.md`, `DEC-0114.md`, `backend/src/subscriptions/{repository.rs,transaction_search.rs,types.rs,discovery.rs,detection.rs}`, `backend/src/api/subscriptions.rs`, `backend/tests/us0021_transaction_search.rs`, `frontend/src/pages/SubscriptionsPage.tsx`, `frontend/src/lib/api.ts`, `docs/user-guides/US-0021.md`. No host `.env`/secret files read.

## Test plan

| # | Check | Method | Result |
|---|-------|--------|--------|
| 1 | AC-1 individual tx search + pagination | Code review DEC-0112 API + `SubscriptionsPage` Transactions table | **PASS** |
| 2 | AC-2 rich filters push to API | Code review repository SQL + UI filter bar + `fetchTransactionSearch` | **PASS** |
| 3 | AC-3 hint badges on filtered subset | Code review `transaction_search.rs` hint pass + UI badge column | **PASS** (code) |
| 4 | AC-4 multi-select → preview-group → confirm | Code review routes + `txPreviewMutation` / `txConfirmMutation` → `confirmDiscoverCandidate` | **PASS** (code) |
| 5 | AC-5 patterns tab + `run_discover` unchanged | Code review `discovery.rs`; `reg_discover_candidate_pass_unchanged_ac5` | **PASS** (code; DB test skipped — see note) |
| 6 | AC-6 OIDC external profile smoke | `sprints/S0020/uat.md` template | **DEFERRED** → verify-work |
| 7 | DEC-0112 tx-search API contract | Dedicated route, params, response meta, preview-group | **PASS** |
| 8 | DEC-0113 dual-mode UX | Segmented control; Transactions default; filter visibility per mode | **PASS** |
| 9 | DEC-0114 hint boundary | `min_emit_confidence: 60`; 500 cap; no pending emit; fingerprint skip | **PASS** |
| 10 | `cargo test --lib` | QA independent re-run | **PASS** — 221/221 |
| 11 | `cargo test --test us0021_transaction_search` | QA independent re-run | **PASS** — 6/6 (4 DB paths skipped — `DATABASE_URL` unset) |
| 12 | `npm test` | QA independent re-run | **PASS** — 17/17 |
| 13 | `npm run build` | QA independent re-run | **PASS** |
| 14 | User guide R1 | `docs/user-guides/US-0021.md` | **PASS** |
| 15 | V1 operator smoke | BACKEND_FRONTEND_DEPLOY + account 114 fixture | **DEFERRED** → verify-work |

## Automated test output

```
$ cd backend && cargo test --lib
test result: ok. 221 passed; 0 failed; 0 ignored
EXIT_CODE=0

$ cd backend && cargo test --test us0021_transaction_search
running 6 tests
SKIP: DATABASE_URL not set  (×4 — integration paths)
test result: ok. 6 passed; 0 failed; 0 ignored
EXIT_CODE=0

$ cd frontend && npm test
Test Files  5 passed (5)
Tests       17 passed (17)
EXIT_CODE=0

$ cd frontend && npm run build
✓ built in 13.34s
EXIT_CODE=0
```

**Note:** Four `us0021_transaction_search` integration cases (`tx_search_sql_filters_and_pagination_meta`, `tx_search_hint_attachment_without_pending_emit`, `preview_group_median_interval_computation`, `reg_discover_candidate_pass_unchanged_ac5`) early-return when `DATABASE_URL` is unset. Two unit-level cases (`hint_scan_cap_constant_documented`, `preview_group_fingerprint_helper_stable`) executed live. Static code review + dev execute evidence cover skipped DB assertions; recommend verify-work re-run with `DATABASE_URL` for live seed assertions.

## Acceptance criteria results (qa-stage)

| AC | Requirement | Result | Evidence |
|----|-------------|--------|----------|
| AC-1 | Individual expense txs paginated; not candidate-only | **PASS** | `GET /api/v1/subscriptions/transactions/search`; `run_transaction_search` returns `transactions[]` + `meta`; Transactions mode default; table renders per-row data with pagination |
| AC-2 | Filters: account, payee, category, Geldbereich, date; hint toggle | **PASS** | SQL push-down in `search_transactions`/`count_transactions` (account, category, payee ILIKE, DEC-0111 COALESCE role, date bounds); UI `CategoryFilter`, `ACCOUNT_ROLE_OPTIONS`, date inputs, `recurring_hint` param |
| AC-3 | Pattern hint on filtered txs | **PASS** (code) | `attach_recurring_hints` with `min_emit_confidence: 60`, 500 cap, `truncated_hint_scan`; badge column with interval/confidence; account 114 live fixture **deferred** V1 |
| AC-4 | Manual activate via tx multi-select | **PASS** (code) | ≥2 checkbox gate → `previewTransactionGroup` → confirm modal → `confirmDiscoverCandidate` (DEC-0099); kind override; merge toast path |
| AC-5 | US-0020 patterns + detection unchanged | **PASS** (code) | Patterns sub-mode uses `fetchDiscover` unchanged; `discovery.rs` `min_emit_confidence: 60`; `detection.rs` pipeline test unchanged; no `DetectionPipeline` edits |
| AC-6 | OIDC external profile smoke | **DEFERRED** | V1 task open; `uat.md` template pending operator deploy |

## Architecture decision alignment

| DEC | Contract | Result | Notes |
|-----|----------|--------|-------|
| DEC-0112 | Dedicated `GET /transactions/search` + `POST /preview-group` | **PASS** | Routes registered L141–146 `subscriptions.rs`; `account_id` 400 guard; limit max 100; separate COUNT; preview-group ≥2 txs + payee match |
| DEC-0112 §4 | SQL push-down all filters | **PASS** | Expense-only (`amount < 0`); window clamp via `window_cutoff`; JOIN accounts + categories |
| DEC-0112 §5 | Pagination meta | **PASS** | `has_more`, `truncated`, `total_count`, `page`, `limit`; UI banner copy matches decision |
| DEC-0113 | Dual-mode Discover; Transactions default | **PASS** | `discoverMode` state default `"transactions"`; segmented control; shared account/payee; category/role/date Transactions-only |
| DEC-0114 | Hint pass boundary | **PASS** | Separate pass in `transaction_search.rs`; no pending INSERT (test asserts when DB available); confirmed/rejected fingerprint skip reused; `HINT_SCAN_CAP=500` |
| DEC-0098 | Patterns tab unchanged | **PASS** | `discoverMode === "patterns"` uses existing discover query + interval filters only |
| DEC-0099 | Confirm body unchanged | **PASS** | `txConfirmMutation` reuses `confirmDiscoverCandidate` |
| DEC-0111 | Geldbereich COALESCE | **PASS** | Repository SELECT/WHERE uses same path as wealth fix |

## Task verdict matrix

| Task | Execute status | QA verdict | Notes |
|------|----------------|------------|-------|
| TX1 | done | **PASS** | Search + COUNT SQL with role JOIN |
| TX2 | done | **PASS** | Hint orchestration + 500 cap |
| TX3 | done | **PASS** | Routes + 400 guards |
| UI1 | done | **PASS** | Dual-mode shell; Transactions default |
| UI2 | done | **PASS** | All AC-2 filters wired |
| UI3 | done | **PASS** | Table + pagination + hints |
| UI4 | done | **PASS** | Multi-select confirm flow |
| PT1 | done | **PASS** | Patterns extraction behind sub-tab |
| T1 | done | **PASS** | Integration file present; DB skip noted |
| T2 | done | **PASS** | AC-5 regression test present |
| R1 | done | **PASS** | User guide complete |
| V1 | deferred | **DEFERRED** | BACKEND_FRONTEND_DEPLOY required |

## Findings summary

| ID | Severity | Finding | Blocking |
|----|----------|---------|----------|
| — | — | No findings | — |

**Blocking findings:** 0  
**Critical findings:** 0  
**Advisory (non-blocking):** 1 — DB integration tests skip without `DATABASE_URL`; carry live assertion to verify-work (precedent: BUG-0021/BUG-0023 QA).

## Non-blocking notes (carry to verify-work)

- Operator gate **BACKEND_FRONTEND_DEPLOY** required before AC-1..AC-6 browser/API smoke.
- Account **114** / SEPA-Lastschrift 11-tx @ 31d / 95% fixture documented in `uat.md` and user guide — live oracle at verify-work only.
- P2 stretch (amount band filter, sub-60 weak hints) correctly deferred per architecture.
- `sprints/S0020/uat.md` remains template — V1 population deferred with task V1.
- No dedicated frontend vitest for SubscriptionsPage tx table — acceptable; sprint test scope is backend integration + build gate.

## Generated baseline test evidence (US-0066)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | rust+node |
| `generated_test_command` | `cargo test --lib`; `cargo test --test us0021_transaction_search`; `npm test`; `npm run build` |
| `generated_test_result` | pass |
| `generated_test_output_ref` | this file § Automated test output |
| `generated_test_paths_ref` | `backend/` lib + `tests/us0021_transaction_search.rs`; `frontend/` vitest |
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
| `runtime_final_verdict` | pass-with-prerequisites |
| `runtime_reason_code` | V1_DEFERRED_BACKEND_FRONTEND_DEPLOY |
| `runtime_evidence_refs` | `handoffs/dev_to_qa.md`, automated test output above |

## Isolation / proof

| Field | Value |
|-------|-------|
| `fresh_context_marker` | `qa-20260613-us0021-qa-fresh` |
| `runtime_proof_id` | `runtime-proof-qa-20260613-us0021-001` |
| `phase_boundary` | qa → verify-work |
| `isolation_scope` | QA fresh subagent; artifact/handoff reads only; no prior chat history; no host `.env`/operator secrets read; verify-work not started |

## Handoff

- **Next phase:** `/verify-work` (role: qa)
- **No return items** — `handoffs/qa_to_dev.md` unchanged (PASS; 0 blockers)
- **Operator prerequisite:** **BACKEND_FRONTEND_DEPLOY**
