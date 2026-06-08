# Design Concept — US-0019

## Summary

US-0019 delivers **goal-driven planning** — target balance + target date plans with **per-plan statistics**, **category-scoped spend overlays** that affect recompute, and **deterministic savings suggestions** the operator explicitly applies — building on US-0018 category APIs without altering DEC-0007 forecast projection or DEC-0089 compare actuals preview.

## Goals

- **Goal balance** template with target fields persisted on plan (AC-1)
- Per-plan goal-stats strip: monthly delta, yearly rollup, projected balance at target date (AC-2)
- Category `remove_outflow` adjustments capped by mirror history; affects compare after recompute (AC-3)
- Category savings modal with evidence; checkbox apply — no auto-apply (AC-4)
- Aggregate-only AI/REST path; audit on adjustment create (AC-5)
- US-0014 onboarding/templates + OIDC regression (AC-6)

## Non-goals

- Per-plan Plan vs Actual endpoint (DEC-0096)
- PMT / interest-rate feasibility math
- LLM-only savings ranking
- Category-scoped forecast bucket re-projection
- Auto-insert required monthly savings as plan lines
- Grafana dashboard changes

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0091 | Plan-level goal columns + enum | AC-1 typed schema; backward compatible NULLs |
| DEC-0092 | Dedicated goal-stats API | AC-2 scope guard; beyond_horizon explicit |
| DEC-0093 | 3-mo avg cap on category remove | Prevents over-removal; uses DEC-0087 series |
| DEC-0094 | Deterministic aggregate ranking | AC-4/AC-5 privacy + reproducibility |
| DEC-0095 | Optional goal account + default | Resolves acct 114/116 ambiguity |
| DEC-0096 | PVA household unchanged | Preserves DEC-0024 active plan contract |
| DEC-0097 | REST primary; tool optional P2 | Modal needs REST; chat wrapper deferred OK |
| DEC-0089 | Compare actuals chart unchanged | Regression guard AC-6 |

## User experience

Scenarios grid gains a **Goal balance** card. After create, a stats strip shows progress toward the target date — gap, required monthly savings (informational), and on-track indicator. Compare tab shows the same strip for the selected plan. Add-line form accepts category cuts; overlay engine caps reductions to recent spending. **Suggest savings** opens a modal ranked by discretionary category spend; operator checks rows and applies as plan adjustments. Plan vs Actual tab behavior is unchanged (active plan household view).
