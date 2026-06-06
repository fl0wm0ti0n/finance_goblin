# PO to TL archive pack (2026-05-31)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 8
- First archived heading: `## discovery-20260531-us0006 — US-0006 AI assistant UX discovery`
- Last archived heading: `## discovery-20260531-us0006 — US-0006 AI assistant UX discovery`
- Verification tuple (mandatory):
  - archived_body_lines=126
  - retained_body_lines=430

---

## discovery-20260531-us0006 — US-0006 AI assistant UX discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-05-31  
**Story:** US-0006  
**Next phase:** `/research`

### Summary

Discovery captured design/UX references for the privacy-safe AI financial assistant (Phase 4). US-0006 delivers the **AI Tool Layer** (six registered tools), **OpenAI provider** integration, **Privacy Layer** (Projectplan TOML defaults), **React chat UI** (header `Sheet` drawer + `/chat` full page), **suggested prompt chips**, **tool transparency** row, **Settings AI & Privacy** section, and **operator tool audit log**. Builds on US-0001 OIDC shell and Settings, US-0002 forecast API, US-0003 subscriptions API, US-0004 plan API (`simulate_plan`), US-0005 wealth/budget-status API (`get_portfolio`, `get_budget_status`), and read-only Firefly guarantee (DEC-0004).

### UX references captured (see `docs/product/vision.md`)

| Reference | US-0006 application |
|-----------|---------------------|
| **Finanzguru** | Conversational Q&A for affordability, subscription changes, budget overruns, cancel savings, top categories; suggested prompt chips; grounded answers without UI clone |
| **Firefly III** | Tool-only access via Flow APIs; Firefly-native labels in summaries; no Firefly mutation |
| **shadcn/ui** | Enable AI nav at `/chat`; header AI button + `Sheet` drawer; shared `ChatPanel`; Settings AI & Privacy; tool audit table |
| **Projectplan** | Six tools (`get_transactions`, `get_subscriptions`, `get_forecast`, `get_budget_status`, `get_portfolio`, `simulate_plan`); privacy TOML options |
| **US-0002 forecast** | `get_forecast` tool wraps forecast REST API |
| **US-0003 subscriptions** | `get_subscriptions` tool wraps subscription list/detail API |
| **US-0004 planning** | `simulate_plan` read-only plan API invocation |
| **US-0005 wealth** | `get_portfolio` and `get_budget_status` wrap wealth/alert surfaces |

### Scope refinements (backlog updated)

- AI Tool Layer: 6 tools via OpenAI function calling; each wraps existing services — no direct DB from AI path
- OpenAI: TOML `[ai]` + env; API key secrets-only; configurable model
- Chat: header `Sheet` drawer + `/chat` full page sharing `ChatPanel`; suggested prompt chips on empty thread
- Privacy: Projectplan defaults (`allow_raw_transactions=false`, `redact_iban=true`, `redact_counterparties=true`); chat privacy badge; Settings section
- Tool transparency: collapsible "Tools used" under assistant messages
- Audit log: operator-visible recent tool invocations
- Out of scope unchanged: local providers (US-0008), ML forecasts (US-0009), Grafana AI dashboard

### Discovery decomposition evidence

- Feature/workflow count: 6 tools + OpenAI + privacy + chat drawer + `/chat` + audit + Settings + example queries (moderate-high — single story retained)
- Cross-cutting impact: backend AI orchestration, tool registry, privacy middleware, React chat UI, audit persistence, upstream API integration
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md`)
- Risk surface: redaction bypass, API key handling, payload size vs context window, prompt injection, streaming auth, US-0008 scope creep

### Open questions (carry to research/architecture)

- Tool registry: Rust trait + OpenAI JSON schema generation vs static schema files per tool?
- Streaming: SSE token stream for chat MVP vs buffered full response?
- Redaction placement: privacy middleware on tool output vs per-tool redaction helpers?
- `allow_raw_transactions=false`: aggregate-only responses (category totals, counts) vs empty tool payload with error message?
- Audit persistence: dedicated DB table (`ai_tool_audit`) vs structured log file only?
- Audit retention: fixed row cap vs time-based purge?
- `simulate_plan`: which plan API endpoints — active plan overlay vs named scenario parameter?
- Tool payload limits: truncate/summarize large transaction lists before model context?
- Rate limiting: per-user session token budget or request throttle?
- Settings privacy toggles: runtime-editable in UI vs read-only display of TOML (restart required)?
- Chat history: persist threads in DB vs ephemeral session-only MVP?
- BFF reconsideration gate (DEC-0006): does SSE streaming require cookie-based auth?

### Recommended next steps

1. `/research` — OpenAI function-calling patterns in Axum, privacy redaction strategies, tool schema design, SSE streaming with JWT auth, audit log persistence (extends R-0001–R-0005 foundation)
2. `/architecture` — AI orchestration contract, tool registry trait, privacy middleware, chat REST/SSE API, migration 006 schema, DEC-xxxx for redaction defaults and audit retention
3. `/sprint-plan` — S0006 decomposition for US-0006 acceptance criteria

---


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

