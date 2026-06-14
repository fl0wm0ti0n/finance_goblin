# Progress — Sprint S0020

**Story:** US-0021  
**Sprint:** S0020  
**Phase:** execute complete — awaiting `/qa`

| ID | Status | Title |
|----|--------|-------|
| TX1 | done | Repository SQL search + COUNT + role JOIN |
| TX2 | done | Search service + hint pass |
| TX3 | done | GET search + POST preview-group routes |
| UI1 | done | Dual-mode tab shell (DEC-0113) |
| UI2 | done | Rich filter bar |
| UI3 | done | Tx table + pagination + hints |
| UI4 | done | Multi-select confirm flow |
| PT1 | done | Patterns sub-tab extraction |
| T1 | done | Search + hint integration tests |
| T2 | done | AC-5 regression tests |
| R1 | done | User guide US-0021 |
| V1 | deferred | UAT OIDC smoke + AC-1..AC-6 template |

## Milestones

- **2026-06-13:** Sprint planned — 12 tasks; see `handoffs/tl_to_dev.md`
- **2026-06-13:** Execute complete — 11/12 tasks done; V1 deferred (operator BACKEND_FRONTEND_DEPLOY)

## Test counts (execute)

| Suite | Result |
|-------|--------|
| `cargo test --lib` | **221 passed** / 0 failed (+3 vs plan-verify baseline 218) |
| `cargo test --test us0021_transaction_search` | **6 passed** / 0 failed |
| `npm test` | **17 passed** / 0 failed |
| `npm run build` | **PASS** |

## Operator gates (verify-work)

- **BACKEND_FRONTEND_DEPLOY** — required before V1

## Next

- `/qa` in fresh subagent/chat (role: qa)
