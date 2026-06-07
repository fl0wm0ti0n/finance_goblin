# Dev → QA Handoff

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-07  
**Bug:** BUG-0015  
**Sprint:** Q0023 (`/quick`)  
**Orchestrator:** `auto-20260607-bug0015-001`

### Summary

Implemented AU1–AU4 for BUG-0015 confirm persistence after rebuild: **DEC-0084** card billing `payee_key` normalization, **DEC-0085/0086** payee+interval confirm inheritance (load maps, merge upsert, ±3d tolerance), detection skip+merge path, and stale inactive by payee+interval wired into `run_detection`. **V1** open — blocked on operator deploy + gates.

### Tasks completed

| ID | Status | Acceptance row |
|----|--------|----------------|
| AU1 | done | AU, AV |
| AU2 | done | AU, AV |
| AU3 | done | AU, AV, AW |
| AU4 | done | AV |
| V1 | open | AU–AW — operator smoke after deploy |

### Acceptance mapping (verify)

| Row | Tasks | Verify focus |
|-----|-------|--------------|
| **AU** | AU1, AU2, AU3, V1 | Confirmed Cursor/Apple after rebuild + Full sync — not pending |
| **AV** | AU1–AU4, V1 | No duplicate pending; merge/skip on payee+interval |
| **AW** | AU3, V1 | Unread alerts reconcile; no spurious new_detection on merge |

### Test evidence

| Command | Result |
|---------|--------|
| `cargo test --lib` | 187/187 PASS |
| `npm test -- --run` | 6/6 PASS |

### Key contracts

- **DEC-0084:** `payee_key()` asterisk split, comma left-segment, Apple billing roots → `apple`, domain `.com`/`/bill` tail strip
- **DEC-0085:** `load_confirmed_payee_intervals`, `load_rejected_payee_intervals`, `merge_confirmed_pattern` in-place refresh; index `idx_subscription_patterns_payee_status`
- **DEC-0086:** `interval_matches` ±3d; fingerprint rotation on merge; UNIQUE conflict fail-safe to pending path
- **AU3:** merge before pending upsert; no `new_detection` on confirmed merge
- **AU4:** `mark_stale_inactive` uses `build_active_payee_intervals`; gap > 2× `interval_days`

### Files changed

- `backend/src/recurrence/normalize.rs` — AU1
- `backend/src/subscriptions/repository.rs` — AU2
- `backend/src/subscriptions/types.rs` — `ConfirmedPayeeInterval`
- `backend/src/subscriptions/detection.rs` — AU3, AU4
- `backend/src/subscriptions/service.rs` — wire maps + stale pass
- `backend/migrations/012_subscription_patterns_payee_status.sql` — AU2 index

### Operator prerequisites for V1

1. **BACKEND_FRONTEND_DEPLOY** — Q0023 backend bundle on financegnome.omniflow.cc
2. **POSTGRES_PERSISTENCE_PROBE** — H2 SQL before Full sync
3. **FULL_FIREFLY_SYNC** — Full sync + detection phase

### QA instructions

Run `/qa` per `sprints/quick/Q0023/uat.md`. Code review AU1–AU4 contracts; V1 runtime probes deferred to verify-work after operator gates.

### Gaps / advisories

- V1 omniflow smoke not run in dev — operator gates required
- Pre-fix orphan pending cleanup deferred per frozen boundaries

### Next phase

`/qa` in fresh subagent context.
