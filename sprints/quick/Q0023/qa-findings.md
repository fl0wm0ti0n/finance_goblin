# QA Findings — Quick Q0023 / BUG-0015

**Work item:** BUG-0015 (defect)  
**Quick task:** Q0023  
**QA phase:** `/qa`  
**Date:** 2026-06-07  
**Orchestrator:** `auto-20260607-bug0015-001`  
**Decisions:** DEC-0084, DEC-0085, DEC-0086  
**QA agent:** fresh subagent (`qa-20260607-q0023-bug0015`)  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — Implemented tasks **AU1**, **AU2**, **AU3**, **AU4** satisfy DEC-0084/0085/0086 at code and test level. Zero blocking findings. **V1** correctly deferred (operator gates). Hand off to **`/verify-work`** for omniflow runtime probes (AU–AW live steps).

## Scope

BUG-0015 confirm persistence after rebuild: card billing `payee_key` normalization (**AU1** / DEC-0084), payee+interval confirm inheritance maps and merge upsert (**AU2** / DEC-0085/0086), detection skip+merge path (**AU3**), stale inactive by payee+interval (**AU4**). Runtime **V1** rebuild smoke deferred per operator gates.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0023/progress.md`, `sprints/quick/Q0023/uat.md`, `sprints/quick/Q0023/task.json`, `backend/src/recurrence/normalize.rs`, `backend/src/subscriptions/repository.rs`, `backend/src/subscriptions/detection.rs`, `backend/src/subscriptions/service.rs`, `backend/src/subscriptions/types.rs`, `backend/migrations/012_subscription_patterns_payee_status.sql`, `decisions/DEC-0084.md`, `decisions/DEC-0085.md`, `decisions/DEC-0086.md`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | AU1 card billing normalization | `cargo test --lib card_billing` | **PASS** (4/4) |
| T-2 | AU2 interval tolerance + payee lookup | `cargo test --lib interval_matches` | **PASS** (2/2) |
| T-3 | AU4 active payee+interval set | `cargo test --lib build_active_payee` | **PASS** (1/1) |
| T-4 | Frontend vitest regression | `npm test -- --run` | **PASS** (6/6) |
| T-5 | Full lib regression | `cargo test --lib` | **PASS** (187/187) |
| T-6 | **AU1** — DEC-0084 rules in `payee_key()` | Code review + T-1 | **PASS** |
| T-7 | **AU2** — maps, merge upsert, migration index | Code review + T-2 | **PASS** |
| T-8 | **AU3** — skip+merge before pending; no alert on merge | Code review `detection.rs` | **PASS** |
| T-9 | **AU4** — stale inactive by payee+interval; wired in `run_detection` | Code review + T-3 | **PASS** |
| T-10 | **AW-CODE** — merge path suppresses `new_detection` | Code review AU3 `continue` after merge | **PASS** |
| T-11 | V1 omniflow runtime smoke | `sprints/quick/Q0023/uat.md` | **DEFERRED** — verify-work |
| T-12 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |

### Test output (T-5)

```
test result: ok. 187 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Task verdict matrix

| Task | Status (execute) | QA verdict | Notes |
|------|------------------|------------|-------|
| AU1 | done | **PASS** | DEC-0084 asterisk, comma memo, Apple roots, domain tail; 4 unit tests |
| AU2 | done | **PASS** | load maps, `merge_confirmed_pattern`, ±3d tolerance, migration 012 |
| AU3 | done | **PASS** | merge before pending upsert; rejected skip; no `new_detection` on merge |
| AU4 | done | **PASS** | `mark_stale_inactive` payee+interval; gap > 2× interval; wired post-candidates |
| V1 | open | **DEFERRED** | Operator gates: BACKEND_FRONTEND_DEPLOY, POSTGRES_PERSISTENCE_PROBE, FULL_FIREFLY_SYNC |

## Acceptance criteria matrix (BUG-0015)

| Row | Criterion | QA verdict | Evidence |
|-----|-----------|------------|----------|
| **AU** | Confirmed Cursor/Apple remain confirmed after rebuild + Full sync | **PASS** (code) / **DEFERRED** (runtime) | AU1 normalization + AU2/AU3 inheritance path; T-1/T-6/T-7/T-8 PASS. Live AU-1/AU-2/H2-1 **DEFERRED** |
| **AV** | No duplicate pending; confirmed skip/merge on payee+interval | **PASS** (code) / **DEFERRED** (runtime) | AU3 merge+skip, AU4 stale map; DEC-0086 tolerance. T-8/T-9 PASS. Live AV-1 **DEFERRED** |
| **AW** | Unread alerts reconcile; no spurious `new_detection` on merge | **PASS** (code) / **DEFERRED** (runtime) | AU3 `continue` after successful merge bypasses `upsert_alert`. T-10 PASS. Live AW-1 **DEFERRED** |

## Code review vs decisions

### DEC-0084 (AU1)

| Contract | Status | Evidence |
|----------|--------|----------|
| Asterisk: substring before first `*` | **PASS** | `apply_card_billing_rules` lines 14–16; test `card_billing_asterisk_split` |
| Comma memo: leftmost segment | **PASS** | lines 17–19; test `card_billing_comma_memo_left_segment` |
| Billing root alias: Apple roots → `apple` | **PASS** | `collapse_billing_root_alias`; test `card_billing_apple_roots_collapse` |
| Domain tail: strip `.com`, `/bill` | **PASS** | `strip_domain_tail`; test `card_billing_domain_tail_strips_com_and_bill` |
| Rules after DEC-0072 SEPA passes | **PASS** | `payee_key()` calls SEPA passes before `apply_card_billing_rules` |
| SEPA paths unchanged | **PASS** | test `sepa_rules_unchanged_for_non_card_paths` |

### DEC-0085 (AU2–AU4)

| Contract | Status | Evidence |
|----------|--------|----------|
| `load_confirmed_payee_intervals()` keyed by payee+interval | **PASS** | `repository.rs` lines 112–124; `ConfirmedPayeeInterval` type |
| `load_rejected_payee_intervals()` | **PASS** | lines 126–137 |
| `merge_confirmed_pattern()` in-place refresh | **PASS** | lines 139–224; UPDATE by `id` WHERE `status = confirmed` |
| Detection: rejected payee+interval → skip | **PASS** | `detection.rs` lines 54–57 |
| Detection: confirmed match → merge → skip pending + no alert | **PASS** | lines 59–75 `continue` before `upsert_pending_pattern` |
| Retain `confirmed_fps` fast path | **PASS** | lines 49–51 |
| `mark_stale_inactive` uses active payee+interval set | **PASS** | lines 261–279; `build_active_payee_intervals` |
| Wired into `run_detection` post-candidates | **PASS** | `service.rs` lines 55–57 |
| Index `idx_subscription_patterns_payee_status` | **PASS** | migration `012_subscription_patterns_payee_status.sql` |

### DEC-0086 (AU2–AU4)

| Contract | Status | Evidence |
|----------|--------|----------|
| `interval_matches` ±3 day tolerance | **PASS** | `repository.rs` lines 17–20; tests in repository + detection |
| Tolerance on confirmed/rejected/stale paths | **PASS** | `find_confirmed_payee_interval`, `is_rejected_payee_interval`, `mark_stale_inactive` |
| Fingerprint rotation on merge | **PASS** | `merge_confirmed_pattern` SET `fingerprint = $2` |
| Preserve `status=confirmed`, `confirmed_at`, `id` | **PASS** | UPDATE omits `confirmed_at`/`status`; WHERE guards confirmed |
| UNIQUE conflict fail-safe to pending path | **PASS** | `is_unique_violation` → `Ok(false)` → falls through to `upsert_pending_pattern` |
| Transaction boundary for merge + relink | **PASS** | `begin`/`commit` with rollback on conflict |

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

**Informational (non-blocking):**

1. **No DB integration test for `merge_confirmed_pattern`** — logic verified by code review; runtime V1 will exercise live merge path.
2. **AW-CODE** — merge→no-alert enforced by control flow (`continue`); no dedicated mock-DB test asserting zero `upsert_alert` calls.
3. **Runtime deferred** — Operator gates BACKEND_FRONTEND_DEPLOY, POSTGRES_PERSISTENCE_PROBE, FULL_FIREFLY_SYNC pending.
4. **Pre-fix orphan pending cleanup** — deferred per frozen boundaries; not blocking V1.

## Verify-work readiness

| Gate | Status |
|------|--------|
| Code QA (AU1–AU4) | **READY** |
| Targeted test suites (T-1–T-4) | **READY** — all PASS |
| `cargo test --lib` | **READY** — 187/187 PASS |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** |
| Operator **POSTGRES_PERSISTENCE_PROBE** (H2 SQL) | **PENDING** |
| Operator **FULL_FIREFLY_SYNC** | **PENDING** |
| V1 omniflow smoke (AU–AW) | **PENDING** |

## Next phase

**`/verify-work`** — deploy bundle, operator gates, then V1 probes per `sprints/quick/Q0023/uat.md`.

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
