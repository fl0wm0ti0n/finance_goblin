# Technical Specification — US-0014

## Overview

Implement **DEC-0077**: page-local planning mutation feedback helper; wire `onError` on seven mutations; extend success toasts and `plan-vs-actual` invalidation; extend set-active banner for Grafana Dashboard 3; verify shipped AC-1/AC-3/AC-4; publish user guide; OIDC smoke.

## Components

| Layer | Change |
|-------|--------|
| `frontend/src/pages/planningFeedback.ts` | **New** — `showPlanningFeedback`, `formatPlanningError` (or co-located in PlanningPage) |
| `frontend/src/pages/PlanningPage.tsx` | Mutation handlers, banner copy, toast UI variants |
| `docs/user-guides/US-0014.md` | First-run flow, Set active, Compare semantics |
| `backend/src/api/plans.rs` | **Verify only** — no contract change |
| `backend/tests/plans_integration.rs` | **Retain** — regression guard |

## Interfaces

### Feedback helper (DEC-0077)

```typescript
type PlanningFeedbackKind = "success" | "error";

function showPlanningFeedback(opts: {
  kind: PlanningFeedbackKind;
  message: string;
}): void;

function formatPlanningError(err: unknown, label: string): string;
```

| Kind | Background | Auto-dismiss |
|------|------------|--------------|
| success | `#ecfdf5` | 4 seconds |
| error | `#fef2f2` | Manual Dismiss |

### Mutations requiring `onError`

1. `createPlanMutation` — `POST /api/v1/plans`
2. `activateMutation` — `POST /api/v1/plans/{id}/activate`
3. `applyTemplateMutation` — `POST .../apply-template`
4. `createVersionMutation` — `POST .../versions`
5. `addAdjustmentMutation` — `POST .../adjustments`
6. `updateAdjustmentMutation` — `PATCH .../adjustments/{id}`
7. `deleteAdjustmentMutation` — `DELETE .../adjustments/{id}`

### Invalidation keys (add to existing success handlers)

```typescript
queryClient.invalidateQueries({ queryKey: ["plan-vs-actual"] });
```

Apply on: add/update/delete adjustment, activate, createPlan.

### Set-active banner copy (AC-6)

After plan create when `is_active=false`:

> Plan created. Click **Set active** so Plan vs Actual and Grafana Dashboard 3 (Budgets) use this scenario.

## Verification

| Test | Command / action |
|------|------------------|
| Backend regression | `cargo test --test plans_integration` |
| AC-7 manual | Submit invalid adjustment → red error card |
| AC-8 operator | OIDC omniflow `/planning` Scenarios + Compare + Plan vs Actual |

**Gate:** BACKEND_FRONTEND_DEPLOY before AC-8 smoke.
