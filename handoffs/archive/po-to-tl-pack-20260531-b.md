# PO to TL archive pack (2026-05-31)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## research-20260531-us0004 — US-0004 financial planning technical research`
- Last archived heading: `## research-20260531-us0004 — US-0004 financial planning technical research`
- Verification tuple (mandatory):
  - archived_body_lines=54
  - retained_body_lines=477

---

## research-20260531-us0004 — US-0004 financial planning technical research

**From:** Tech Lead  
**To:** Dev (via `/architecture` handoff)  
**Date:** 2026-05-31  
**Story:** US-0004  
**Next phase:** `/architecture`

### Summary

Web research completed for US-0004 financial planning. Six new entries (R-0015–R-0020) extend R-0006/R-0007 forecast baseline, R-0008 Grafana provisioning, and DEC-0010 sync pipeline with Plan Engine delta overlay, version semantics, plan-vs-Ist aggregation, persistence schema, recompute triggers, and Dashboard 3 as-code.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|-------------------------------|
| **Plan Engine patterns** | [R-0015](docs/engineering/research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline) | Delta overlay on latest forecast snapshot — do not fork forecast engine; explicit adjustment lines with template presets |
| **Scenario versioning** | [R-0016](docs/engineering/research.md#r-0016--plan-scenario-versioning-immutable-snapshots-vs-editable-drafts) | Hybrid: latest version editable; "create new version" freezes prior (v1/v2/v3 cap); table-first compare + grouped bar |
| **Plan-vs-Ist computation** | [R-0017](docs/engineering/research.md#r-0017--plan-vs-ist-daily-computation--aggregation-grain) | Primary metric = household daily net cashflow; deviation = actual − planned; category drill-down in React only |
| **Persistence model** | [R-0018](docs/engineering/research.md#r-0018--plan-persistence-schema-plans-versions-adjustments-daily-snapshots) | `plans` + `plan_versions` + `plan_adjustments` + `plan_computations` + `plan_daily_cashflow` hypertable; migration 004 |
| **Recompute trigger** | [R-0019](docs/engineering/research.md#r-0019--plan-recompute-trigger--forecast-baseline-hook) | Recompute on plan save + post-forecast hook for active plan only; no new sync mutex phase |
| **Grafana Dashboard 3** | [R-0020](docs/engineering/research.md#r-0020--grafana-dashboard-3-budgets-planistdeviation-provisioning) | uid `budgets`; Plan/Ist/Abweichung time series + MTD table; household aggregate MVP |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Fork forecast vs independent Plan Engine? | **Delta overlay** on latest forecast computation — store adjustments only, apply at projection time (R-0015) |
| Ist aggregation grain? | **Household daily net cashflow** from mirrored transactions; category drill-down secondary in React (R-0017) |
| Savings mode: auto-populate subs vs manual? | **Auto-suggest confirmed subscriptions** on template apply; user confirms selection before save (R-0015, R-0018) |
| Version semantics: immutable vs editable? | **Hybrid** — latest editable; prior versions frozen on "create new version"; max 3 versions (R-0016) |
| Compare UX: table vs chart? | **Table-first metrics** with ECharts grouped bar secondary (R-0016) |
| Daily plan-vs-Ist primary number? | **Net cashflow per day** (signed); not balance path (R-0017) |
| Active plan overlay on `/forecast`? | **Defer** — planning-only surface in US-0004; link from forecast optional (R-0019) |
| Dashboard 3 MVP scope? | **Household aggregate** Plan/Ist/Abweichung; per-category deferred to US-0005 (R-0020) |
| Recompute: plan save vs sync mutex? | **Plan save + post-forecast hook**; no `"planning"` sync phase (R-0019) |

### Risks surfaced (carry to architecture)

1. **Baseline staleness** — plan projection tied to forecast computation; surface `stale` metadata when either recompute fails (R-0015, R-0019)
2. **Savings mode Ist lag** — plan assumes cancelled sub but actuals still show charges until real cancellation (R-0017)
3. **Version cap UX** — v1/v2/v3 limit may surprise users; clear messaging on v4 attempt (R-0016)
4. **Active plan mutex** — partial unique index requires transactional switch; race with concurrent edits (R-0018)
5. **Grafana empty state** — no active plan breaks `$active_plan_version` variable; annotation fallback required (R-0020)
6. **Multi-currency** — MVP assumes single reporting currency for household aggregate (R-0017)

### Recommended next steps

1. `/architecture` — Plan Engine trait/contract, REST API, migration 004 schema (R-0018), delta overlay hook on forecast baseline (R-0015), post-forecast active-plan recompute (R-0019), DEC-xxxx for template defaults, version cap, and active-plan semantics
2. `/sprint-plan` — S0004 task decomposition against 6 acceptance criteria
3. Spec-pack expansion for US-0004 (SPEC_PACK_MODE=1)

---

