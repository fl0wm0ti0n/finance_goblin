# PO to TL archive pack (2026-05-31)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 9
- First archived heading: `## discovery-20260531-us0004 — US-0004 financial planning UX discovery`
- Last archived heading: `## sprint-plan-20260531-s0003 — US-0003 sprint decomposition`
- Verification tuple (mandatory):
  - archived_body_lines=104
  - retained_body_lines=471

---

## discovery-20260531-us0004 — US-0004 financial planning UX discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-05-31  
**Story:** US-0004  
**Next phase:** `/research`

### Summary

Discovery captured design/UX references for financial planning (Phase 3). US-0004 delivers the **Plan Engine** with scenario templates and custom deltas, **plan versioning** (v1/v2/v3 compare), **daily plan-vs-Ist** comparison, React **`/planning`** page, and **Grafana Dashboard 3** (Budgets: plan/ist/deviation). Builds on US-0001 synced Firefly actuals, US-0002 forecast baseline, and US-0003 confirmed subscriptions for savings-mode template picks.

### UX references captured (see `docs/product/vision.md`)

| Reference | US-0004 application |
|-----------|---------------------|
| **Finanzguru** | Scenario templates (leasing, savings mode, house purchase); v1/v2/v3 side-by-side compare; daily plan/ist/deviation; life-decision framing without UI clone |
| **Firefly III** | Ist from synced transactions (read-only); Firefly budget/category vocabulary; no Firefly mutation |
| **shadcn/ui** | Enable Planning nav at `/planning`; Tabs Scenarios / Compare / Plan vs Actual; active plan selector; template cards + adjustment table |
| **Apache ECharts** | Daily planned vs actual dual line; version compare grouped bar; deviation trend with zero line |
| **Grafana Dashboard 3** | Plan, Ist, Abweichung panels for active plan; uid `budgets` |
| **US-0002 forecast** | Current (Ist) scenario aligns with latest forecast computation; deltas adjust projected layer only |
| **US-0003 subscriptions** | Savings-mode template suggests confirmed subscriptions to remove |

### Scope refinements (backlog updated)

- Plan Engine: explicit user deltas on forecast baseline; one active plan drives plan-vs-Ist + Dashboard 3
- Templates: Current (Ist), Leasing (+€/month), Savings mode (remove subs / cut spend), House purchase (raise savings rate)
- Custom adjustments: amount, frequency, target (subscription / category / custom label)
- Versioning: v1/v2/v3 per named plan with Compare tab
- React: `/planning` route; daily plan-vs-Ist table/chart for active plan
- Grafana Dashboard 3 provisioned alongside existing analytics dashboards
- Out of scope unchanged: AI `simulate_plan` (US-0006), crypto allocation scenarios (US-0007), plan viability Alert Engine (US-0005)

### Discovery decomposition evidence

- Feature/workflow count: Plan Engine + 4 templates + custom deltas + version compare + daily plan-vs-Ist + React page + Grafana (moderate-high — single story retained)
- Cross-cutting impact: backend engine, DB migration, React UI, forecast/subscription integration, Grafana
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md`)
- Risk surface: delta modeling vs forecast baseline, Ist lag vs sync, v1/v2/v3 compare UX breadth, Dashboard 3 SQL joining plan + actual tables, read-only actuals constraint

### Open questions (carry to research/architecture)

- Plan projection: fork US-0002 forecast computation with delta overlay vs independent Plan Engine projection layer?
- Ist aggregation grain: household free cashflow vs category budget spend vs account balance path?
- Savings mode: auto-populate from confirmed subscriptions vs manual line items only?
- Version semantics: immutable snapshots on "new version" vs editable in-place with history?
- Compare UX: table-first metrics vs chart-first for v1/v2/v3?
- Daily plan-vs-Ist metric: which primary number — free cashflow, total outflow, or category-specific?
- Active plan overlay on `/forecast` in US-0004 or planning-only surface?
- Dashboard 3 MVP: household aggregate vs per-category breakdown panels?
- Recompute trigger: on plan save only vs extend sync mutex after forecast recompute?

### Recommended next steps

1. `/research` — Plan Engine delta model, persistence schema, Ist aggregation rules, Grafana Dashboard 3 as-code (extends R-0008)
2. `/architecture` — Plan Engine contract, REST API, migration 004 schema, active plan selection, forecast baseline hook, DEC-xxxx for template defaults and version limits
3. `/sprint-plan` — S0004 decomposition for US-0004 acceptance criteria

---

## sprint-plan-20260531-s0003 — US-0003 sprint decomposition

**From:** Tech Lead  
**To:** QA (`/plan-verify`) → Dev (`/execute`)  
**Date:** 2026-05-31  
**Story:** US-0003  
**Sprint:** S0003  
**Next phase:** `/plan-verify`

### Summary

Created sprint **S0003** with **12 tasks** covering all US-0003 acceptance criteria. No sprint split required (SPRINT_MAX_TASKS=12, SPRINT_AUTO_SPLIT=1). Tasks T-0025–T-0036 map to recurrence core, migration 003, subscription engine, sync hook, forecast override, API, React `/subscriptions`, Grafana Dashboard 2, tests, and user guide.

### Task summary

| ID | Title |
|----|-------|
| T-0025 | Extract shared recurrence core from forecast |
| T-0026 | SQLx migration 003 subscriptions schema |
| T-0027 | Subscription repository and config |
| T-0028 | Subscription engine classify, detection, price_change |
| T-0029 | SubscriptionService run_detection orchestration |
| T-0030 | Sync pipeline subscriptions phase hook |
| T-0031 | Forecast override hook with DetectionResult |
| T-0032 | Subscription REST API endpoints |
| T-0033 | React subscriptions page shell and routing |
| T-0034 | React detail drawer, ECharts price history, alerts UX |
| T-0035 | Grafana Dashboard 2 Subscriptions |
| T-0036 | Subscription tests and operator user guide |

### Split decision

- **Why 12 tasks:** Each maps to one deployable increment aligned with architecture layers; combined tasks would obscure acceptance boundaries and block parallel Grafana/frontend work.
- **Why not split into S0003/S0004:** Exactly at threshold; dependency chain is linear enough for single sprint; all 8 ACs belong to one story.
- **USER_GUIDE_MODE=1:** T-0036 includes `docs/user-guides/US-0003.md`.

### Recommended next steps

1. `/plan-verify` — confirm AC coverage in `sprints/S0003/plan-verify.json`
2. `/execute` — implement T-0025 through T-0036 in dependency order

---

