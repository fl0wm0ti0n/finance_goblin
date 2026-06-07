# Q0020 progress

**Sprint:** Q0020 (BUG-0013)  
**Status:** EXECUTE COMPLETE → QA  
**Last updated:** 2026-06-08

## Task status

| ID | Status | Priority |
|----|--------|----------|
| AL1 | DONE | P0 |
| AN1 | DONE | P0 |
| AJ1 | DONE | P2 optional |
| AK2 | DONE | P2 optional |
| V1 | DONE (template) | P0 |

## Execute order

`AL1 ∥ AN1 → AJ1, AK2 → V1 template` — runtime smoke pending operator gates

## Operator gates

| Gate | Status |
|------|--------|
| **BACKEND_FRONTEND_DEPLOY** | pending |
| **GRAFANA_PROVISIONING_RELOAD** | pending |
| **FULL_FIREFLY_SYNC** | pending |

## Notes

- AL1: panel 5 planned CTE capped at `CURRENT_DATE` per DEC-0079
- AN1: array wallet parse, `unrealizedPNL` keys, linear USDT→EUR in `pnl.rs`
- AJ1/AK2: Grafana empty-state copy on subscriptions + portfolio performance %
- V1: `uat.md` smoke checklist ready; omniflow probes blocked on deploy + Full sync
- Tests: `cargo test --lib` 174 passed
