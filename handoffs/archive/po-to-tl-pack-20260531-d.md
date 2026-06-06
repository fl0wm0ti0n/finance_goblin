# PO to TL archive pack (2026-05-31)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## discovery-20260531-us0005 — US-0005 wealth & alerts UX discovery`
- Last archived heading: `## discovery-20260531-us0005 — US-0005 wealth & alerts UX discovery`
- Verification tuple (mandatory):
  - archived_body_lines=62
  - retained_body_lines=477

---

## discovery-20260531-us0005 — US-0005 wealth & alerts UX discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-05-31  
**Story:** US-0005  
**Next phase:** `/research`

### Summary

Discovery captured design/UX references for wealth analysis and the unified Alert Engine (Phase 4). US-0005 delivers **net worth aggregation** (Firefly asset accounts; crypto placeholder until US-0007), **Alert Engine** (scarcity threshold, budget drift %, plan viability warnings), **React `/wealth` page** + **`/alerts` inbox** with header notification bell, **threshold config centralization** (wire Dashboard 1 scarcity line to TOML), and **Grafana Dashboard 4 partial** (total non-crypto wealth). Builds on US-0001 synced accounts, US-0002 forecast snapshots, US-0003 subscription-scoped alerts (unchanged boundary), US-0004 active plan and plan-vs-Ist series, and Grafana provisioning (DEC-0012).

### UX references captured (see `docs/product/vision.md`)

| Reference | US-0005 application |
|-----------|---------------------|
| **Finanzguru** | Gesamtvermögen headline + account breakdown; proactive scarcity/budget-drift/plan-viability warnings; alert inbox with acknowledge/dismiss; header bell unread count |
| **Firefly III** | Asset account balances from mirror; native account type labels; category actuals for budget drift vs active-plan targets |
| **shadcn/ui** | Enable Wealth nav at `/wealth`; header bell + Popover preview; `/alerts` inbox table; Overview stat card + account breakdown table |
| **Apache ECharts** | Optional account breakdown bar/pie on wealth page; wealth-over-time line if snapshots stored |
| **Grafana Dashboard 4** | uid `portfolio`; total wealth stat + account breakdown; crypto/performance deferred US-0007 |
| **US-0002 forecast** | Scarcity alert evaluates projected balance path; centralize €200 threshold from Dashboard 1 static line |
| **US-0004 planning** | Budget drift vs category-targeted plan adjustments; plan viability on active scenario infeasibility |
| **US-0003 subscriptions** | Page-scoped subscription alerts unchanged — no migration to unified inbox |

### Scope refinements (backlog updated)

- Net worth: sum Firefly asset accounts; reporting currency EUR default; crypto placeholder row
- Alert Engine: post-sync evaluation; scarcity / budget drift / plan viability types; acknowledge + dismiss lifecycle
- TOML `[alerts]` config: `scarcity_threshold_eur` (default 200), `budget_drift_pct` (default 20)
- React: `/wealth` route; `/alerts` inbox; header bell with unread badge
- Grafana Dashboard 4 partial provisioned; Dashboard 1 scarcity line wired to shared config
- Out of scope unchanged: crypto PnL (US-0007), Grafana Alertmanager rules, AI tools (US-0006), subscription alert migration

### Discovery decomposition evidence

- Feature/workflow count: net worth + 3 alert types + config centralization + bell + inbox + wealth page + Grafana 4 partial (moderate-high — single story retained)
- Cross-cutting impact: backend Alert Engine, DB migration, React UI (wealth + alerts + header), forecast/plan hooks, Grafana 4 + Dashboard 1 wiring
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md`)
- Risk surface: dedup vs US-0003 alerts, budget-drift grain, plan-viability rule, threshold migration, multi-currency MVP, post-sync latency

### Open questions (carry to research/architecture)

- Alert evaluation trigger: inline at end of post-sync mutex (after forecast + plan hook) vs async job if latency exceeds threshold?
- Scarcity scope: per primary asset account vs household aggregate minimum balance path?
- Budget drift grain: per Firefly category vs per active-plan category-targeted adjustment only vs Firefly budget entity?
- Plan viability rule: negative projected month-end balance once vs N consecutive days below zero vs free-cashflow deficit?
- Alert dedup/cooldown: suppress repeat same-type+entity until condition clears or fixed cooldown window?
- Dismiss semantics: hide until re-trigger vs permanent suppress for entity+type pair?
- Unified inbox: include read-only link to subscription alerts or strictly separate surfaces?
- Net worth snapshots: store daily balance history for trend chart or compute on-demand from mirror?
- Dashboard 1 migration: Grafana variable from DB config query vs API-provisioned threshold refresh?
- Multi-currency net worth: sum native balances with warning banner vs single reporting currency conversion (deferred)?

### Recommended next steps

1. `/research` — Alert Engine evaluation rules, persistence schema, budget-drift metric, plan-viability heuristics, Dashboard 4 as-code (extends R-0008), scarcity config centralization (extends DEC-0012)
2. `/architecture` — Alert Engine contract, REST API, migration 005 schema, post-sync evaluation hook, DEC-xxxx for thresholds and dismiss semantics
3. `/sprint-plan` — S0005 decomposition for US-0005 acceptance criteria

---

