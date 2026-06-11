# Q0026 — BUG-0018 alert evaluation SQL failure

| Field | Value |
|-------|-------|
| **ID** | Q0026 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0018 |
| **Created** | 2026-06-10 |
| **Architecture** | `architecture-20260610-bug0018` (`docs/engineering/architecture-archive/architecture-pack-20260609-a.md` § BUG-0018) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260610-q0026-bug0018`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0018 rows **BE**, **BF** |
| **Task count** | 3 (all P0 mandatory) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0018 alert evaluation SQL failure: **DEC-0107** qualify `fbd.balance` and `fbd.ts` in `evaluate_scarcity` daily aggregate (**BE1**), `wealth_alerts_integration` regression gate (**T1**), operator verify-work smoke (**V1**) covering **BE** log-clean sync and **BF** wealth inbox + header bell after deploy + Full sync. Subscription dedup regression gate in V1 only.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| Scarcity SQL fix (DEC-0107) | BE1 | `backend/src/alerts/evaluate.rs` |
| Integration regression | T1 | `backend/tests/wealth_alerts_integration.rs` |
| Verify | V1 | `uat.md` + operator sync/alerts smoke |

**Ops-only (not execute tasks):** Operator **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** before V1 runtime probes.

**Out of scope:** Migration; frontend change; sync fail-on-alert-error (R-0024 warn-only preserved); subscription alert SQL (BUG-0008 separate path); CI TimescaleDB service container (defer).

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| BE1 | Qualify `fbd.balance` + `fbd.ts` in `evaluate_scarcity` | 1h | — | **BE** | P0 |
| T1 | `wealth_alerts_integration` regression gate | 1h | BE1 | **BE** | P0 |
| V1 | verify-work sync + alerts smoke | 1.5h | BE1, T1 + deploy | **BE**, **BF** | P0 |

**Total estimate:** ~3.5h (2h dev + ~1.5h operator V1).

## Deploy order

```text
BE1 (evaluate.rs SQL qualification)
  → T1 (integration test PASS)
  → single backend release
  → operator: BACKEND_FRONTEND_DEPLOY
  → FULL_FIREFLY_SYNC
  → V1 verify-work (logs, GET /api/v1/alerts, header bell, subscription dedup regression)
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **BE** | BE1, T1, V1 | Post-sync eval completes; no 42702 / `alert evaluation failed` in logs |
| **BF** | BE1, V1 | Wealth `GET /api/v1/alerts` + header bell non-empty when rules match; subscription dedup regression |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| BE1 | Task **BE1** |
| T1 | Task **T1** |
| V1 | Task **V1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
