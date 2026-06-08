# CRS — US-0019

## Purpose

Enable operators to define **savings goals with deadlines**, see **plan-scoped progress statistics**, model **category-level spending changes** in what-if scenarios, and adopt **evidence-backed savings ideas** they choose — extending US-0004/US-0014 planning with US-0018 category infrastructure.

## Scope

**In scope**

- Migration + `goal_balance` template (DEC-0091)
- `GET /api/v1/plans/{id}/goal-stats` (DEC-0092)
- Category overlay cap in `build_overlay_deltas` (DEC-0093)
- `GET /api/v1/plans/{id}/category-savings-suggestions` + apply modal (DEC-0094)
- `goal_account_id` scope + default (DEC-0095)
- GoalStatsStrip on Scenarios + Compare (DEC-0092)
- PVA unchanged (DEC-0096)
- Optional `get_category_savings` tool (DEC-0097 P2)
- `docs/user-guides/US-0019.md`
- OIDC US-0010 external profile smoke (AC-6)

**Out of scope**

- Per-plan PVA API
- PMT feasibility with interest rate
- LLM savings ranking
- Forecast Income/Fixed/Variable re-projection per category
- DEC-0089 compare API category fork
- Firefly write-back
- Grafana changes

## Acceptance criteria ref

See `docs/product/acceptance.md` § US-0019 — AC-1 through AC-6.

## Dependencies

- US-0018 DONE (DEC-0087..0089 — category catalog, expense-series, CategoryFilter)
- US-0014 DONE (templates, onboarding, mutation toasts)
- US-0006 DONE (AI audit patterns)
- DEC-0073 (overlay monthly delta)
- DEC-0007 (forecast baseline — unchanged)
- DEC-0024 (single active plan — PVA)
