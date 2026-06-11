# Q0026 summary — BUG-0018

**Sprint:** Q0026 (quick)  
**Bug:** BUG-0018 — Alert evaluation SQL failure (ambiguous balance)  
**Orchestrator:** `intake-20260609-ui-audit`  
**Execute completed:** 2026-06-10

## Goal

Close BUG-0018 by qualifying `fbd.balance` and `fbd.ts` in `evaluate_scarcity` per **DEC-0107**, restoring post-sync wealth alert evaluation without PostgreSQL 42702.

## Tasks completed

| ID | Title | Status | Evidence |
|----|-------|--------|----------|
| BE1 | Qualify `fbd.balance` + `fbd.ts` in `evaluate_scarcity` | **done** | `backend/src/alerts/evaluate.rs` |
| T1 | `wealth_alerts_integration` regression gate | **done** | `backend/tests/wealth_alerts_integration.rs` — 3/3 PASS (DB test skipped — `DATABASE_URL` unset) |
| V1 | verify-work sync + alerts smoke | **open** | Deferred to `/verify-work` after **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** |

## Implementation notes

- **DEC-0107:** `SELECT fbd.ts::date`, `SUM(fbd.balance::float8)`, `WHERE fbd.ts::date`, `GROUP BY fbd.ts::date` — no unqualified `balance`/`ts`; no `a.balance`.
- **Frozen boundaries:** No migration; no sibling evaluator changes; R-0024 warn-only sync semantics preserved.

## Test results

| Check | Result |
|-------|--------|
| `cargo test --lib` | **213/213 PASS** |
| `cargo test --test wealth_alerts_integration` | **3/3 PASS** (scarcity integration skipped — `DATABASE_URL` unset) |

## Operator gates (before V1)

| Gate | Status |
|------|--------|
| **BACKEND_FRONTEND_DEPLOY** | pending — rebuild `flow-finance-ai` with BE1 |
| **FULL_FIREFLY_SYNC** | pending — Full sync; alerts phase must complete without 42702 |

## Next phase

**`/qa`** — code review + test verification; V1 runtime smoke deferred to verify-work.
