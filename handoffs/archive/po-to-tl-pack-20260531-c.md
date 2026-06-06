# PO to TL archive pack (2026-05-31)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## architecture-20260531-us0004 — US-0004 financial planning architecture`
- Last archived heading: `## architecture-20260531-us0004 — US-0004 financial planning architecture`
- Verification tuple (mandatory):
  - archived_body_lines=65
  - retained_body_lines=477

---

## architecture-20260531-us0004 — US-0004 financial planning architecture

**From:** Tech Lead  
**To:** Sprint planning (`/sprint-plan`)  
**Date:** 2026-05-31  
**Story:** US-0004  
**Next phase:** `/sprint-plan`

### Summary

Architecture complete for US-0004 financial planning. Defines **Plan Engine** delta overlay on forecast baseline, migration **004**, plan REST API, plan-vs-Ist (household daily net cashflow), React **`/planning`**, and **Grafana Dashboard 3** (`uid: budgets`). Six decisions (DEC-0019–DEC-0024). Spec-pack complete (3/3). Research R-0015–R-0020 incorporated.

### Architecture highlights

| Area | Decision |
|------|----------|
| **Plan Engine** | Delta overlay on latest forecast snapshot — store adjustments only (DEC-0019, R-0015) |
| **Templates** | Current, Leasing (+€300/mo default), Savings mode (confirmed sub pick-list), House purchase, Custom |
| **Versioning** | Hybrid: latest editable; freeze on new version; max 3 versions (DEC-0020, R-0016) |
| **Migration 004** | `plans`, `plan_versions`, `plan_adjustments`, `plan_computations`, `plan_daily_cashflow` hypertable (DEC-0022, R-0018) |
| **Plan-vs-Ist** | Household daily net cashflow; deviation = actual − planned (DEC-0021, R-0017) |
| **Recompute** | Plan save (async) + post-forecast hook for active plan only; no sync `"planning"` phase (DEC-0023, R-0019) |
| **Active plan** | Single global `is_active`; drives API + Grafana `$active_plan_version` (DEC-0024) |
| **React** | `/planning` — Scenarios \| Compare \| Plan vs Actual; ECharts grouped bar + dual line |
| **Grafana** | Dashboard 3 `budgets` — Plan/Ist/Abweichung household aggregate MVP (DEC-0024, R-0020) |
| **Forecast page** | No active-plan overlay in US-0004 — planning-only surface |

### Decisions created

- **DEC-0019** — Plan projection: delta overlay on forecast baseline
- **DEC-0020** — Version semantics: hybrid editable latest; cap 3
- **DEC-0021** — Plan-vs-Ist: household daily net cashflow
- **DEC-0022** — Migration 004 persistence schema
- **DEC-0023** — Recompute: plan save + post-forecast hook
- **DEC-0024** — Active plan + Grafana Dashboard 3 scope

### Spec-pack status

| Artifact | Status |
|----------|--------|
| `US-0004-design-concept.md` | Complete |
| `US-0004-crs.md` | Complete |
| `US-0004-technical-specification.md` | Complete |

### Discovery questions resolved (research → architecture)

| Question | Resolution |
|----------|------------|
| Fork forecast vs Plan Engine? | Delta overlay (DEC-0019) |
| Ist aggregation grain? | Household daily net cashflow (DEC-0021) |
| Savings mode subs? | Auto-suggest confirmed; user confirms before save |
| Version semantics? | Hybrid latest editable; max 3 versions (DEC-0020) |
| Compare UX? | Table-first + ECharts grouped bar |
| Daily plan-vs-Ist metric? | Net cashflow per day (DEC-0021) |
| Active plan on `/forecast`? | Deferred — `/planning` only (DEC-0023) |
| Dashboard 3 MVP? | Household aggregate (DEC-0024) |
| Recompute trigger? | Plan save + post-forecast; no sync phase (DEC-0023) |

### Recommended next steps

1. `/sprint-plan` — S0004 decomposition (6 acceptance criteria; expect ~12 tasks)
2. `/plan-verify` — AC coverage after sprint-plan

---

