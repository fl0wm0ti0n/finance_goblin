# Q0029 — BUG-0021 Frontend UX polish (category filter delay, wealth role column)

| Field | Value |
|-------|-------|
| **ID** | Q0029 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0021 |
| **Created** | 2026-06-11 |
| **Architecture** | `docs/engineering/architecture.md` § BUG-0021 (DEC-0110, DEC-0111) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260611-q0029-bug0021`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0021 rows **BK**, **BL** |
| **Task count** | 7 mandatory + 1 optional P2 (8/12 under `SPRINT_MAX_TASKS=12`) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0021 via **DEC-0110** + **DEC-0111**: static CategoryFilter import on Forecast
Monthly and Wealth Overview (**EA1**, **EA2**, gate **BK**); SQL
`COALESCE(attributes, root)` `account_role` extract + frontend `formatAccountRole`
label map (**EB1**, **EB2**, gate **BL**); integration tests + operator verify-work.
Optional **EA3** PlanningPage parity (out of BK).

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| EA1 — ForecastPage static import | EA1 | `frontend/src/pages/ForecastPage.tsx` |
| EA2 — WealthPage static import | EA2 | `frontend/src/pages/WealthPage.tsx` |
| EB1 — SQL attributes path | EB1 | `backend/src/wealth/repository.rs` |
| EB2 — Role display labels | EB2 | `frontend/src/pages/WealthPage.tsx` (+ optional `frontend/src/lib/accountRole.ts`) |
| EA3 — PlanningPage parity (optional P2) | EA3 | `frontend/src/pages/PlanningPage.tsx` |
| T1 — integration tests | T1 | `backend/tests/bug0021_wealth_account_role.rs` |
| Automated gate | G1 | `cargo test` + frontend build + blast-radius check |
| Verify | V1 | `uat.md` + operator localhost/omniflow smoke |

**Ops-only (not execute tasks):** Operator **BACKEND_FRONTEND_DEPLOY** (includes
ForecastPage TS6133 fix prerequisite from BUG-0020 release notes); optional manual
sync or wait for daily snapshot upsert before BL snapshot/Grafana gate.

**Out of scope (DEC-0110/0111 forbidden):** CategoryFilter component logic change;
CategoryTrendChart lazy boundary; Firefly sync payload rewrite; Grafana
`portfolio.json` SQL edit; migration; root `active`/`include_net_worth` path fix.

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| EA1 | ForecastPage static CategoryFilter import | 1h | — | **BK** | P0 |
| EA2 | WealthPage static CategoryFilter import | 1h | — | **BK** | P0 |
| EB1 | wealth/repository.rs account_role SQL path | 1.5h | — | **BL** | P0 |
| EB2 | WealthPage formatAccountRole label map | 1h | EB1 | **BL** | P0 |
| EA3 | PlanningPage CategoryFilter parity (optional) | 0.5h | — | — | P2 |
| T1 | Integration tests BK/BL + regression | 2h | EA1–EB2 | **BK**, **BL** | P0 |
| G1 | Automated gate (cargo test + build) | 0.5h | T1 | **BK**, **BL** static | P0 |
| V1 | verify-work operator smoke | 2h | G1 + deploy | **BK**, **BL** | P0 |

**Total estimate:** ~9.5h mandatory + 0.5h optional (~10h dev + ~2h operator V1).

## Deploy order

```text
EA1 ∥ EA2 ∥ EB1 (disjoint files — parallel ok)
  → EB2 (WealthPage Role column; same file as EA2 — sequence after EA2 or same commit)
  → EA3 (optional PlanningPage parity)
  → T1 integration tests
  → G1 automated gate
  → operator: BACKEND_FRONTEND_DEPLOY (fix ForecastPage TS6133 if still present)
  → operator: optional sync or wait for daily snapshot (BL snapshot/Grafana gate)
  → V1 verify-work
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **BK** | EA1, EA2, T1, G1, V1 | Forecast → Monthly and Wealth → Overview: no multi-second **Loading category filter…**; combobox interactive ≤1 s |
| **BL** | EB1, EB2, T1, G1, V1 | `GET /api/v1/wealth` — Giro/savings/cash wallet `account_role` non-null; Wealth Role column human labels; snapshot/Grafana heal post-upsert; OIDC smoke |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| EA1 | Task **EA1** |
| EA2 | Task **EA2** |
| EA3 | Task **EA3** (optional P2) |
| EB1 | Task **EB1** |
| EB2 | Task **EB2** |
| T1 | Task **T1** |
| BK/BL runtime gates | Task **V1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
