# Design Concept — US-0004

## Summary

US-0004 delivers **financial planning** for household decision-making: a Plan Engine that applies explicit scenario deltas on top of US-0002 forecast baselines, versioned plans (v1/v2/v3 compare), daily plan-vs-Ist (actual) deviation tracking, a React `/planning` page, and Grafana Dashboard 3 (Budgets).

Plans live entirely in Flow Finance AI DB; Firefly remains the read-only source for actuals. One **active plan** drives plan-vs-Ist and Grafana.

## Goals

- Delta overlay Plan Engine on latest forecast snapshot — no forecast fork (DEC-0019, R-0015)
- Built-in templates: Current (Ist), Leasing, Savings mode, House purchase, Custom (DEC-0019)
- Versioning: hybrid editable latest + freeze on new version; max 3 versions (DEC-0020, R-0016)
- Migration 004 persistence + `plan_daily_cashflow` hypertable (DEC-0022, R-0018)
- Household daily net cashflow plan-vs-Ist metric (DEC-0021, R-0017)
- Recompute on plan save + post-forecast hook; no sync planning phase (DEC-0023, R-0019)
- React `/planning`: Scenarios | Compare | Plan vs Actual tabs; ECharts compare + dual line
- Grafana Dashboard 3 (`uid: budgets`) household aggregate (DEC-0024, R-0020)
- Single global active plan selector (DEC-0024)

## Non-goals

- AI `simulate_plan` chat (US-0006)
- Crypto allocation scenarios (US-0007)
- Plan viability / budget-drift Alert Engine (US-0005)
- Active plan overlay on `/forecast`
- Per-category Grafana panels (React drill-down only)
- Multi-currency conversion
- Any write to Firefly III

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0019 | Delta overlay on forecast | Simplest correct model; no engine drift (R-0015) |
| DEC-0020 | Hybrid versions, cap 3 | Matches acceptance + compare UX (R-0016) |
| DEC-0021 | Daily net cashflow | Aligns with forecast vocabulary (R-0017) |
| DEC-0022 | Migration 004 relational + hypertable | API + Grafana queryability (R-0018) |
| DEC-0023 | Async plan save + post-forecast hook | Avoid mutex bloat (R-0019) |
| DEC-0024 | Single active plan; uid `budgets` | Dashboard 3 + Ist join discipline (R-0020) |

**UX references:** Finanzguru scenario templates and v1/v2/v3 compare, Firefly vocabulary for Ist, shadcn Tabs/Cards/Select, ECharts grouped bar + dual line, Grafana Budgets — see `docs/product/vision.md`.

**Spec-pack traceability:** `docs/engineering/spec-pack/US-0004-crs.md`, `docs/engineering/spec-pack/US-0004-technical-specification.md`
