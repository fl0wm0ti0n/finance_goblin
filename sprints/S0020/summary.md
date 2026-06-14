# Summary — Sprint S0020 / US-0021

**Story:** US-0021 — Subscription transaction explorer with rich filters  
**Sprint:** S0020  
**Status:** EXECUTE COMPLETE (2026-06-13)  
**Orchestrator:** `auto-20260613-us0021`

## Delivered

- **DEC-0112:** `GET /api/v1/subscriptions/transactions/search` + `POST /transactions/preview-group`
- **DEC-0113:** Dual-mode Discover — Transactions (default) | Suggested patterns
- **DEC-0114:** Hint pass on filtered subset — row metadata only; no auto-emit
- **DEC-0098:** Patterns sub-tab — existing discover candidates unchanged
- **DEC-0099:** Multi-select tx confirm via preview-group → existing confirm body
- **Docs:** `docs/user-guides/US-0021.md`
- **Tests:** `backend/tests/us0021_transaction_search.rs` — search, hints, preview-group, AC-5 regression

## Tasks

| ID | Status |
|----|--------|
| TX1–PT1, T1–T2, R1 | **done** (11/12) |
| V1 | **deferred** — operator BACKEND_FRONTEND_DEPLOY |

## Test counts

- `cargo test --lib`: 221/221
- `cargo test --test us0021_transaction_search`: 6/6
- `npm test`: 17/17
- `npm run build`: PASS

## Operator gates (verify-work)

- **BACKEND_FRONTEND_DEPLOY**

## Next phase

`/qa` (role: qa)
