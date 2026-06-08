# Technical Specification — US-0019

## Overview

Implement **DEC-0091** through **DEC-0097**: goal balance plan type; per-plan goal-stats API; category overlay cap; deterministic savings suggestions; goal account scope; PVA unchanged; optional AI tool.

## Components

| Layer | Change |
|-------|--------|
| `backend/migrations/` | **New** — `goal_balance` enum + plan columns |
| `backend/src/plan/types.rs` | **Extend** — goal fields on `PlanRow` |
| `backend/src/plan/templates.rs` | **Extend** — goal_balance preset |
| `backend/src/plan/overlay.rs` | **Extend** — category remove cap (DEC-0093) |
| `backend/src/plan/project.rs` | **Extend** — goal_account_id baseline (DEC-0095) |
| `backend/src/plan/service.rs` | **Extend** — goal-stats computation |
| `backend/src/api/plans.rs` | **Extend** — goal-stats + category-savings routes |
| `frontend/src/pages/PlanningPage.tsx` | **Extend** — template, strip, modal |
| `frontend/src/components/plan/GoalStatsStrip.tsx` | **New** |
| `frontend/src/lib/api.ts` | **Extend** — goal + savings types |
| `backend/src/ai/tools/` | **Optional** — `get_category_savings` (P2) |
| `docs/user-guides/US-0019.md` | **New** — at execute |

## Interfaces

### `POST /api/v1/plans` (goal_balance)

**Body:** `template: "goal_balance"`, `target_balance_eur`, `target_date`, optional `goal_account_id`

**Errors:** 422 when template is goal_balance but target fields missing; 422 when `target_date` < today

### `GET /api/v1/plans/{plan_id}/goal-stats`

**Query:** `version_id` optional

**Response:** See DEC-0092 — includes `yearly_rollup`, `projected_balance_at_target`, `gap_eur`, `required_monthly_savings_eur`, `beyond_horizon`

### `GET /api/v1/plans/{plan_id}/category-savings-suggestions`

**Query:** `months` default 6; `limit` default 10 max 20

**Response:**

```json
{
  "suggestions": [
    {
      "category_id": "42",
      "category_name": "Entertainment",
      "avg_monthly_outflow_eur": "85.50",
      "transaction_count": 24,
      "suggested_reduction_eur": "42.75",
      "evidence_summary": "6-month avg €85.50/mo; 24 transactions"
    }
  ]
}
```

### Category overlay (internal)

For `target_type=category` + `direction=remove_outflow`:

```
effective_amount = min(adjustment.amount, avg_outflow_last_3_months)
```

## Sequencing (sprint-plan input)

1. G1–G2 schema + create flow
2. O1–O2 overlay + account scope (parallel after G1)
3. S1–S2 goal-stats API + UI
4. A1–A2 savings service + modal
5. D1 + R1 docs/regression
6. V1 smoke; T1 optional

## Verification

- Unit: goal create validation; overlay cap math; beyond_horizon; fixed-bucket exclusion
- Integration: goal-stats 404 for non-goal template; savings excludes existing adjustments
- UI: stats strip on Scenarios/Compare; modal apply triggers recompute
- Regression: US-0014 empty plan flow; DEC-0089 compare chart actuals-only
- Privacy: savings + goal-stats responses aggregate-only

## Non-functional

- Single sprint ≤12 tasks (S0018)
- Read-only Firefly preserved
- 730d projection horizon enforced
- Audit log per adjustment batch item
