# Design Concept — US-0014

## Summary

US-0014 completes **planning mode first-visit UX** deferred from BUG-0011: operator-visible mutation feedback (success + error), set-active guidance including Grafana Dashboard 3, and confirmation toasts on plan create — without changing compare metrics or PVA API contracts (DEC-0073/0074 frozen).

## Goals

- All planning mutations show operator-visible errors on failure (AC-7 / DEC-0077)
- Add-line flow gives success feedback and refreshes Compare/PVA (AC-2)
- Create-from-template paths show confirmation (AC-5)
- Set-active banner explains Plan vs Actual + Dashboard 3 requirement (AC-6)
- Shipped AC-1/AC-3/AC-4 verified; OIDC smoke on omniflow (AC-8)

## Non-goals

- Compare metric formula changes (DEC-0073)
- PVA API contract changes (DEC-0074)
- Auto-activate first plan
- Global toast library or MutationCache refactor
- Backend plans API changes

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0077 | Page-local `planningFeedback` helper | Matches existing inline card pattern; avoids new deps (R-0073) |
| DEC-0073 | Overlay-only compare delta | Frozen — help text only for negative projected balance |
| DEC-0074 | PVA 200 `no_active_plan` | Frozen — polish copy only |
| DEC-0024 | Single active plan + Dashboard 3 | Banner copy references `budgets` uid |

## User experience

First visit: template grid + **Create empty plan** → add adjustments with visible success/error → **Set active** banner cues Plan vs Actual and Grafana → Compare tab footnote explains overlay delta vs projected balance → PVA guided card when no active plan.
