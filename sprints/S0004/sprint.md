# Sprint S0004

**ID:** S0004  
**Story:** US-0004 — Financial planning, scenarios & plan-vs-actual  
**Status:** PLANNED  
**Created:** 2026-05-31

## Goal

Deliver financial planning on top of US-0001 sync, US-0002 forecast snapshots, and US-0003 confirmed subscriptions: Plan Engine delta overlay, migration 004 persistence, built-in scenario templates, v1/v2/v3 versioning with compare, household daily plan-vs-Ist, post-forecast recompute hook, JWT-protected plan REST API, React `/planning` page, Grafana Dashboard 3, tests, and operator user guide.

## Scope

- Migration `004_plans.sql` — plans, versions, adjustments, computations, `plan_daily_cashflow` hypertable (DEC-0022)
- Plan Engine: `overlay`, `project`, `templates`, `repository`, `service` — delta overlay on latest forecast baseline (DEC-0019)
- Versioning: latest editable; freeze on new version; max 3 versions per plan; 409 on v4 (DEC-0020)
- Plan-vs-Ist: household daily net cashflow; deviation = actual − planned; stale metadata (DEC-0021)
- Active plan: exactly one global active; transactional activate (DEC-0024)
- Recompute: async on plan save; post-forecast hook for active plan only; no sync `"planning"` phase (DEC-0023)
- REST API: plan CRUD, versioning, templates, compare, plan-vs-actual, activate
- React: enable Planning nav; tabs Scenarios | Compare | Plan vs Actual; ECharts compare bar + dual-line plan-vs-Ist
- Grafana: `budgets.json` — uid `budgets`, household aggregate MVP (R-0020, DEC-0024)
- Tests and operator user guide (`docs/user-guides/US-0004.md`)

**Out of scope:** AI `simulate_plan` (US-0006), crypto scenarios (US-0007), plan viability Alert Engine (US-0005), active plan overlay on `/forecast`, per-category Grafana panels, any Firefly writes, multi-currency conversion.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Baseline staleness | Bind `forecast_computation_id`; expose `plan_stale` / `actuals_stale` | R-0015, R-0019 |
| Savings mode Ist lag | Deviation surfaces plan vs reality mismatch when sub still charging | R-0017 |
| Version cap surprise | Clear 409 on v4; UX copy on Compare tab | R-0016, DEC-0020 |
| Active plan race | Transactional activate; guard `is_latest` edits | R-0018, DEC-0024 |
| Grafana empty active plan | Annotation + friendly stat when no active plan | R-0020 |
| Multi-currency | MVP single reporting currency in config | R-0017 |
| Overlay order conflicts | Deterministic `sort_order` when multiple deltas target same payee | R-0015 |
| Plan/forecast recompute race | Version-level guard; `is_latest` check before write | R-0019 |

## Definition of Done

- All 12 sprint tasks complete (`T-0037` … `T-0048`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0004
- Plan mutations trigger async recompute; post-forecast refreshes active plan
- `/planning` renders Scenarios, Compare (v1/v2/v3), and Plan vs Actual for active plan
- Grafana Dashboard 3 loads with uid `budgets`
- User guide published at `docs/user-guides/US-0004.md`
- No Firefly write operations introduced

## Architecture references

- `docs/engineering/architecture.md` — US-0004
- Decisions: DEC-0019 … DEC-0024
- Research: R-0015 … R-0020
- Depends on: US-0001 mirror tables + sync; US-0002 forecast snapshots; US-0003 confirmed subscriptions (savings-mode template)
