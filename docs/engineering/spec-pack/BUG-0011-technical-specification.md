# Technical Specification — BUG-0011

## Overview

Backend-first metric fix (DEC-0073) then API contract fix (DEC-0074), then frontend AD wiring. Compare endpoint semantics align with R-0016; PVA empty state mirrors risk-score pattern.

**Sequencing:** AE1–AE2 before AF1; AD after AF1 API frozen.

## Components

| Component | Change | Decision |
|-----------|--------|----------|
| `plan/overlay.rs` (or `project.rs`) | `monthly_overlay_delta_sum()` helper | DEC-0073 |
| `plan/repository.rs` | `version_metrics` uses overlay sum | DEC-0073 |
| `plan/service.rs` | In-memory compare + `plan_vs_actual` gate | DEC-0073, DEC-0074 |
| `api/plans.rs` | `PlanVsActualApiResponse` tagged enum; 200 no active | DEC-0074 |
| `PlanningPage.tsx` | PVA guided state, empty create, add form | AD, DEC-0074 |
| `budgets.json` (Grafana) | **No change** | R-0020 |

## Interfaces

### Compare metrics (DEC-0073)

**Existing route:** `GET /api/v1/plans/{id}/compare`

`monthly_delta_sum` per version:

```text
SUM(build_overlay_deltas(adjustments, confirmed_subs, month_start, month_end)[d])
for d in month_start .. min(today, month_end)
```

`projected_month_end_balance`: unchanged — last `planned_balance` in month horizon.

**Empty guard:** `adjustments.is_empty()` → `monthly_delta_sum` = `"0.00"`.

### Plan vs actual (DEC-0074)

**Route:** `GET /api/v1/plans/active/plan-vs-actual?month=YYYY-MM`

**No active plan (200):**

```json
{
  "status": "no_active_plan",
  "reason": "no_active_plan"
}
```

**Active plan (200, unchanged):**

```json
{
  "status": "ok",
  "month": "2026-06",
  "reporting_currency": "EUR",
  "plan_stale": false,
  "actuals_stale": false,
  "rows": [ … ]
}
```

### Add adjustment (AD — existing routes)

`POST /api/v1/plans/{id}/versions/{vid}/adjustments` — wire from inline form.

`PATCH /api/v1/plans/{id}/versions/{vid}/adjustments/{aid}` — edit rows.

**Create empty plan:** `POST /api/v1/plans` `{ "name": "…", "template": "custom" }`.

## Non-functional

- **Regression:** Grafana Dashboard 3 unchanged; OIDC `/planning` three-tab smoke
- **Breaking change:** PVA 404 → 200 documented in release notes
- **Release note:** Compare numbers shift for non-empty plans (overlay-only)
- **Privacy:** No host secrets; Firefly read-only unchanged

## User guide

`docs/user-guides/BUG-0011.md` — operator-facing planning fixes and Set active guidance.
