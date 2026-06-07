# Q0023 Summary — BUG-0015

**Sprint:** Q0023  
**Bug:** BUG-0015 — Confirmed subscriptions reappear as pending after rebuild  
**Orchestrator:** `auto-20260607-bug0015-001`  
**Status:** RELEASED (`bug0015-q0023`)  
**Last updated:** 2026-06-07

## Outcome

Subscription confirm persistence after rebuild: card billing `payee_key` normalization (AU1), payee+interval confirm inheritance skip+merge (AU2–AU4), ±3d interval tolerance and in-place fingerprint rotation. V1 operator rebuild smoke deferred.

## Tasks

| ID | Status | Notes |
|----|--------|-------|
| AU1 | DONE | DEC-0084 card descriptor rules in `payee_key()` |
| AU2 | DONE | `load_confirmed_payee_intervals` + merge upsert |
| AU3 | DONE | Detection skip+merge path |
| AU4 | DONE | `mark_stale_inactive` by payee+interval |
| V1 | OPERATOR | 10-step rebuild smoke post-deploy |

## Validation

| Command | Result |
|---------|--------|
| `cargo test --lib` | 187/187 PASS |
| `cargo test card_billing` | 4/4 PASS |
| `cargo test interval_matches` | 2/2 PASS |
| `npm test -- --run` (frontend) | 6/6 PASS |

## Decisions

- DEC-0084 — card billing payee_key normalization
- DEC-0085 — payee+interval confirm inheritance
- DEC-0086 — ±3d interval tolerance + fingerprint rotation

## Research

- R-0081 — confirm inheritance bundle (fulfilled)
- R-0082 — card descriptor normalization (fulfilled)

## Operator follow-up

BACKEND_FRONTEND_DEPLOY + POSTGRES_PERSISTENCE_PROBE + FULL_FIREFLY_SYNC; then smoke per `sprints/quick/Q0023/uat.json`.
