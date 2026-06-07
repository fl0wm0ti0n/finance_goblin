# Q0023 progress

**Sprint:** Q0023 (BUG-0015)  
**Status:** EXECUTE COMPLETE (code) — V1 pending operator gates  
**Last updated:** 2026-06-07  
**Orchestrator:** `auto-20260607-bug0015-001`

## Task status

| ID | Status | Priority | Notes |
|----|--------|----------|-------|
| AU1 | done | P0 | DEC-0084 card `payee_key` normalization + unit tests |
| AU2 | done | P0 | DEC-0085/0086 maps + merge upsert + migration 012 |
| AU3 | done | P0 | Detection skip + merge path wired in service |
| AU4 | done | P0 | Stale inactive by payee+interval; wired post-candidates |
| V1 | open | P0 | verify-work after deploy + operator gates |

## Execute order

`AU1 → AU2 → (AU3 ∥ AU4) → deploy → operator gates → V1`

## Test evidence

| Command | Result |
|---------|--------|
| `cargo test --lib` | 187/187 PASS |
| `npm test -- --run` | 6/6 PASS |

## Operator gates

| Gate | Status |
|------|--------|
| **BACKEND_FRONTEND_DEPLOY** | pending |
| **POSTGRES_PERSISTENCE_PROBE** | pending |
| **FULL_FIREFLY_SYNC** | pending |

## Next phase

`/qa` — handoff `handoffs/dev_to_qa.md`
