# Q0031 — BUG-0022 Plan delete selector regression

| Field | Value |
|-------|-------|
| **ID** | Q0031 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0022 |
| **Created** | 2026-06-13 |
| **Architecture** | `docs/engineering/architecture.md` § BUG-0022 (extends **DEC-0082**, **DEC-0024**, **DEC-0074**) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260613-bug0022-q0031`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0022 rows **BM**, **BN** |
| **Task count** | 4 mandatory + 1 optional P2 (4/12 under `SPRINT_MAX_TASKS=12`) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0022 post-**Q0022** frontend regression on `/planning`: invert `activePlanId`
useMemo so operator dropdown selection wins over global `is_active`, restoring **BM**
(delete non-active plans) while preserving **BN** (**DEC-0082** UI disabled + API **409**).
Vitest selector coverage (**T1**), automated gate (**G1**), operator verify-work (**V1**).

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| BM — selector priority (P0) | BM1 | `frontend/src/pages/PlanningPage.tsx` |
| T — Vitest coverage (P0) | T1 | `planSelector.ts`, `planSelector.test.ts` |
| Regression + gates | G1, V1 | `npm test`, `npm run build`, uat |

**Ops-only (not execute tasks):** Operator **FRONTEND_DEPLOY** (no migration).

**Out of scope:** Backend `plans.rs` / `plan/service.rs`; PVA active endpoint; sole-plan
delete policy (**DEC-0082** §Risks acceptable); Grafana; new DEC; optional **L1** label
rename (P2 — skip if capacity tight).

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| BM1 | Invert selector useMemo priority | 1h | — | **BM** | P0 |
| T1 | Vitest `resolveDisplayedPlanId` + delete enablement | 2h | BM1 | **BM**, **BN** | P0 |
| G1 | Automated gate (`npm test`, `npm run build`) | 0.5h | T1 | **BM**, **BN** | P0 |
| V1 | verify-work `/planning` BM/BN + OIDC smoke | 1.5h | G1 + deploy | **BM**, **BN** | P0 |
| L1 | Dropdown label "Active plan" → "Plan" | 0.5h | V1 | UX | P2 optional |

**Total estimate:** ~5h dev + ~1.5h operator V1.

## Deploy order

```text
BM1 (PlanningPage.tsx useMemo invert)
  → T1 (planSelector.ts + vitest)
  → G1 (npm test + build)
  → operator: FRONTEND_DEPLOY (frontend rebuild only)
  → V1 verify-work
  → optional L1 (label rename)
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **BM** | BM1, T1, G1, V1 | 2+ plans with global active; select non-active → Delete enabled → confirm → plan removed; list refreshes |
| **BN** | BM1, T1, G1, V1 | Select active → delete disabled + tooltip; `DELETE` active → **409** `active_plan_delete_forbidden`; OIDC smoke |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| BM1 | Task **BM1** |
| T1 | Task **T1** |
| G1 | Task **G1** |
| BM/BN runtime gates | Task **V1** |
| L1 | Task **L1** (P2 optional) |

## Frozen boundaries

See `task.json` `frozen_boundaries`.

## User guide (USER_GUIDE_MODE=1)

**Waived** — bug regression fix under existing Planning UX; no new operator workflow.
