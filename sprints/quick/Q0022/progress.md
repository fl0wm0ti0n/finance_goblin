# Q0022 progress

**Sprint:** Q0022 (BUG-0014)  
**Status:** QA PASS → verify-work  
**Last updated:** 2026-06-07  
**Orchestrator:** `auto-20260607-bug0014-001`

## Task status

| ID | Status | Priority | Notes |
|----|--------|----------|-------|
| AO1 | **done** | P0 | Panel 13 dual-scenario ML copy (DEC-0066/DEC-0076) |
| AQ1 | **done** | P0 | `holdings_all` cap 50 + PnL `unpriced_assets` / `fx_incomplete` (DEC-0081) |
| AQ2 | **done** | P0 | WealthPage native qty + EUR table; unified FX banner |
| AS1 | **done** | P0 | Delete plan UI + 409 active guard (DEC-0082) |
| AS2 | **done** | P1 optional | target_type select + help (DEC-0083) |
| AP2 | **skipped** | P0 conditional | Gate not met in dev — AP1_SQL_PROBE requires omniflow deploy + priced futures + subtotal 0 |
| AR1 | **skipped** | P2 conditional | Gate not met in dev — requires V1 AR verify (API≠Grafana on acct 114) |
| V1 | open | P0 | QA / operator verify-work after deploy + gates |

## Execute order

`AO1 ✓ → AQ1 ✓ → AQ2 ✓ → AS1 ✓ → AS2 ✓ → [AP2 skip] → [AR1 skip] → deploy → operator gates → V1`

## Test results (dev)

| Suite | Result |
|-------|--------|
| `cargo test --lib wealth::service::tests` | 4/4 PASS |
| `cargo test --lib plan_delete_api_tests` | 1/1 PASS |
| `cargo test --test grafana_provisioning_bug0009` | 6/6 PASS |
| `npm test -- --run` (frontend) | 6/6 PASS |
| Full `cargo test` (integration) | pre-existing `AppConfig` fixture gaps in `firefly_*` tests — unrelated to Q0022 |

## Operator gates

| Gate | Status |
|------|--------|
| **BACKEND_FRONTEND_DEPLOY** | pending (required before V1 / AP2 probe) |
| **THREE_SERVICE_COMPOSE** | pending |
| **FULL_FIREFLY_SYNC** | pending |
| **GRAFANA_PROVISIONING_RELOAD** | pending (after AO1 deploy) |
| **AP1_SQL_PROBE** | pending |

## Files changed

- `grafana/provisioning/dashboards/analytics/forecast-horizons.json` — AO1
- `backend/src/wealth/types.rs`, `backend/src/wealth/service.rs` — AQ1
- `backend/src/portfolio/service.rs`, `backend/src/portfolio/repository.rs` — AQ1 PnL payload wire
- `frontend/src/lib/api.ts`, `frontend/src/pages/WealthPage.tsx` — AQ2
- `backend/src/plan/service.rs`, `backend/src/api/plans.rs` — AS1
- `frontend/src/pages/PlanningPage.tsx`, `frontend/src/pages/planningFeedback.tsx` — AS1/AS2
- `backend/tests/grafana_provisioning_bug0009.rs` — AO1 test update
- `frontend/src/pages/planningFeedback.test.ts` — AS1 error parse test

## QA results (2026-06-07)

| Suite | Result |
|-------|--------|
| `cargo test --lib wealth::service::tests` | 4/4 PASS |
| `cargo test --lib plan_delete_api_tests` | 1/1 PASS |
| `cargo test --test grafana_provisioning_bug0009` | 6/6 PASS |
| `npm test -- --run` (frontend) | 6/6 PASS |
| `cargo test --lib` (full) | 177/177 PASS |

**QA verdict:** PASS — 0 blockers; evidence `sprints/quick/Q0022/qa-findings.md`

## Next phase

`/verify-work` — handoff `handoffs/qa_to_verify_work.md`
