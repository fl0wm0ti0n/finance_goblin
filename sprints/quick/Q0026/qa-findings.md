# QA Findings — Quick Q0026 / BUG-0018

**Work item:** BUG-0018 (defect)  
**Quick task:** Q0026  
**QA phase:** `/qa`  
**Date:** 2026-06-09  
**Orchestrator:** `intake-20260609-ui-audit`  
**Decisions:** DEC-0107  
**QA agent:** fresh subagent (`qa-20260609-bug0018-qa-fresh`)  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — Implemented tasks **BE1** and **T1** satisfy **DEC-0107** at code and test level. Zero blocking findings. **V1** correctly deferred (operator **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** gates). Hand off to **`/verify-work`** for operator runtime probes (BE/BF live matrix).

## Scope

BUG-0018 alert evaluation SQL failure: qualify `fbd.balance` and `fbd.ts` in `evaluate_scarcity` daily aggregate query (**BE1** / DEC-0107), regression gate via `wealth_alerts_integration` (**T1**). Runtime **V1** sync log + alerts inbox + subscription dedup probes deferred per operator deploy gates.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0026/summary.md`, `sprints/quick/Q0026/tasks.md`, `backend/src/alerts/evaluate.rs`, `backend/tests/wealth_alerts_integration.rs`, `decisions/DEC-0107.md`, `docs/product/acceptance.md` BUG-0018 rows **BE**–**BF**. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | Full lib regression | `cargo test --lib` | **PASS** (213/213) |
| T-2 | Wealth/alerts integration suite | `cargo test --test wealth_alerts_integration` | **PASS** (3/3) |
| T-3 | Scarcity post-sync integration | `wealth_snapshot_and_scarcity_alert_on_post_sync` in T-2 | **PASS** (`DATABASE_URL` set — live DB path executed) |
| T-4 | Frontend vitest regression | `cd frontend && npm test -- --run` | **PASS** (9/9) |
| T-5 | **BE1** — DEC-0107 SQL qualification | Code review `evaluate_scarcity` query | **PASS** |
| T-6 | Forbidden column patterns absent | Grep `evaluate_scarcity` JOIN query | **PASS** |
| T-7 | Frozen boundaries — no migration / sibling evaluators / sync semantics | Code review scope + `sync/mod.rs` L414 warn-only | **PASS** |
| T-8 | **T1** — static module guards | `wealth_alerts_modules_have_no_firefly_writes` | **PASS** |
| T-9 | V1 operator smoke BE/BF | `sprints/quick/Q0026/uat.md` | **DEFERRED** — verify-work |
| T-10 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |
| T-11 | LINT / TYPECHECK | runbook keys blank | **SKIP** |

### Test output (T-1)

```
test result: ok. 213 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Test output (T-2)

```
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  wealth_snapshot_and_scarcity_alert_on_post_sync ... ok
  wealth_alerts_modules_have_no_firefly_writes ... ok
  cashflow_dashboard_uses_scarcity_threshold_variable ... ok
```

## Task verdict matrix

| Task | Status (execute) | QA verdict | Notes |
|------|------------------|------------|-------|
| BE1 | done | **PASS** | DEC-0107 `fbd.balance` + `fbd.ts` in SELECT, WHERE, GROUP BY |
| T1 | done | **PASS** | 3/3 integration; scarcity DB path PASS with `DATABASE_URL` |
| V1 | open | **DEFERRED** | Operator gates: **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** |

## Acceptance criteria matrix (BUG-0018)

| Row | Criterion | QA verdict | Evidence |
|-----|-----------|------------|----------|
| **BE** | Post-sync alert evaluation completes without SQL 42702; logs free of `alert evaluation failed` | **PASS** (code+test) / **DEFERRED** (runtime) | BE1 SQL matches DEC-0107 sketch; T-3 `run_post_sync` creates scarcity alert without error; live sync log probe **DEFERRED** to verify-work |
| **BF** | `GET /api/v1/alerts?status=active` and header bell surface alerts when rules match; subscription dedup regression | **DEFERRED** | V1-only per sprint contract; downstream of BE fix; no frontend change in Q0026 execute scope |

## Code review vs DEC-0107

### BE1 — `evaluate_scarcity` SQL qualification

| Contract element | Expected (DEC-0107) | Observed | Verdict |
|------------------|---------------------|----------|---------|
| SELECT | `fbd.ts::date AS day`, `SUM(fbd.balance::float8) AS balance` | L23 — exact match | **PASS** |
| WHERE date bounds | `fbd.ts::date >= $2 AND fbd.ts::date <= $3` | L29 — qualified | **PASS** |
| GROUP BY | `fbd.ts::date` | L30 — qualified | **PASS** |
| JOIN purpose | Filter asset accounts; aggregate reads `fbd.balance` only | L24–28 — `accounts` filter only | **PASS** |
| Forbidden: unqualified `balance`/`ts` in JOIN query | None | None in L21–31 block | **PASS** |
| Forbidden: `a.balance` / `SUM(a.balance::float8)` | None | None | **PASS** |
| Migration | None | No migration files touched | **PASS** |
| Sibling evaluators | Unchanged | `evaluate_budget_drift`, `evaluate_plan_viability` untouched | **PASS** |
| Sync semantics (R-0024) | warn-only on alert eval error | `sync/mod.rs` L414 `warn!(?e, "alert evaluation failed")` unchanged | **PASS** |

**Note:** Lines 42–46 (`SELECT COALESCE(SUM(balance::float8), 0) FROM accounts`) use unqualified `balance` on a **single-table** query — not ambiguous; outside DEC-0107 JOIN scope. Correct per decision.

### T1 — regression gate

- `wealth_snapshot_and_scarcity_alert_on_post_sync` seeds `forecast_balance_daily` + `accounts` JOIN fixture and exercises `AlertService::run_post_sync`.
- Asserts `alert_type = 'scarcity'` AND `status = 'active'` row created — would catch unqualified `balance` regression (42702).
- Static tests confirm evaluate module has no Firefly write paths.

## Runtime proof summary (DEC-0038 advisory)

| Field | Value |
|-------|-------|
| `runtime_log_summary` | n/a |
| `runtime_retry_count` | 0 |
| `runtime_retry_ledger` | [] |
| `runtime_final_verdict` | deferred |
| `runtime_reason_code` | V1_DEFERRED_BACKEND_FRONTEND_DEPLOY_FULL_FIREFLY_SYNC |
| `runtime_evidence_refs` | `sprints/quick/Q0026/uat.md`, `sprints/quick/Q0026/uat.json` |

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

**Informational (non-blocking):**

1. **V1 runtime** — sync trigger log audit (**BE**), `GET /api/v1/alerts` + header bell (**BF**), subscription dedup regression pending deploy + Full sync.
2. Dev handoff noted DB path skipped (`DATABASE_URL` unset); QA environment had `DATABASE_URL` set — scarcity integration executed live (**T-3 PASS**).
3. Pre-existing lib compile warnings (unused imports) — unchanged by Q0026; not blocking.
4. No frontend change in scope — npm 9/9 baseline regression only.

## Verify-work readiness

| Gate | Status |
|------|--------|
| Code QA (BE1, T1) | **READY** |
| `cargo test --lib` | **READY** — 213/213 PASS |
| `cargo test --test wealth_alerts_integration` | **READY** — 3/3 PASS |
| `npm test` | **READY** — 9/9 PASS |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** |
| Operator **FULL_FIREFLY_SYNC** | **PENDING** |
| V1 operator smoke (BE, BF) | **PENDING** |

## Next phase

**`/verify-work`** — rebuild `flow-finance-ai`, Full sync, then V1 probes per `sprints/quick/Q0026/uat.md`.

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
