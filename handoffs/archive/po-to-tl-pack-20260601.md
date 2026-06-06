# PO to TL archive pack (2026-06-01)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 4
- Retained units in hot file: 8
- First archived heading: `## intake-20260531-us0010 — External Firefly/Postgres + Traefik (omniflow)`
- Last archived heading: `## discovery-20260531-us0006 — US-0006 AI assistant UX discovery`
- Verification tuple (mandatory):
  - archived_body_lines=212
  - retained_body_lines=493

---

## intake-20260531-us0010 — External Firefly/Postgres + Traefik (omniflow)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-05-31  
**Story:** US-0010  
**Next phase:** `/discovery` → `/architecture` → `/execute`

### Summary

Bounded infra story: reuse host `firefly` + `postgres` + `traefik` instead of finance_goblin bundled services; publish Flow Finance AI at **`https://financegnome.omniflow.cc`** with existing **`auth`** basic-auth middleware (same pattern as `finance.omniflow.cc`).

### Split decision

- **Single story** — deploy wiring only; no app feature scope

### Key assumptions

- Operator creates `flow_finance_ai` database (and user grants) on shared `postgres` if not present
- Firefly PAT provided via `.env` (`FIREFLY_PERSONAL_ACCESS_TOKEN`)
- OIDC redirect URIs updated manually if SPA auth used with public URL

### Risks

- Port 8090 clash with `firefly_product_manager` when `full` profile enabled
- Missing TimescaleDB extensions on shared Postgres may block hypertable migrations (verify extension)
- Traefik router name collision if labels reuse `firefly` router id

### Intake evidence

- `selected_pack`: `small-intake-pack`
- `intake_run_id`: `intake-20260531-external-compose`
- Bundle: `handoffs/intake_evidence/intake-20260531-external-compose.json`
- Research: **R-0052**

### Recommended next steps

1. `/discovery` — confirm DB extension + PAT test from traefik network
2. `/execute` — `external` Compose profile, `.env.example`, smoke test on host

---

## sprint-plan-20260531-s0001 — US-0001 sprint decomposition

**From:** Tech Lead  
**To:** QA (`/plan-verify`) → Dev (`/execute`)  
**Date:** 2026-05-31  
**Story:** US-0001  
**Sprint:** S0001  
**Next phase:** `/plan-verify`

### Summary

Created first sprint **S0001** with **12 tasks** covering all US-0001 acceptance criteria. No sprint split required (SPRINT_MAX_TASKS=12, SPRINT_AUTO_SPLIT=1). US-0002 not included — would exceed threshold.

### Task summary

| ID | Title |
|----|-------|
| T-0001 | Docker Compose profiles and minimal stack |
| T-0002 | External PostgreSQL configuration layer |
| T-0003 | Rust/Axum backend project skeleton |
| T-0004 | Health endpoints and DB startup retry |
| T-0005 | SQLx migrations and mirror schema |
| T-0006 | Firefly GET-only connector |
| T-0007 | Sync scheduler and sync API endpoints |
| T-0008 | JWT auth middleware and protected API skeleton |
| T-0009 | React UI shell and OIDC integration |
| T-0010 | Home and Sync Status UI pages |
| T-0011 | Grafana datasource and Platform Health dashboard |
| T-0012 | Read-only integration test and operator user guide |

### Split decision

- **Why 12 tasks:** Each maps to one deployable increment; combined would obscure acceptance boundaries and block parallel work (Compose vs backend vs UI).
- **Why not split into S0001/S0002:** Exactly at threshold; logical dependency chain is linear enough for single sprint; US-0002 is a separate story.
- **USER_GUIDE_MODE=1:** T-0012 includes `docs/user-guides/US-0001.md`.

### Artifacts created

- `sprints/S0001/sprint.md`, `sprint.json`, `tasks.md`, `progress.md`
- `sprints/S0001/uat.md`, `uat.json` (placeholders)
- `handoffs/tl_to_dev.md`
- Traceability row in `docs/engineering/state.md`

### Recommended next steps

1. `/plan-verify` — validate AC coverage in `sprints/S0001/plan-verify.json`
2. `/execute` — implement T-0001 through T-0012 in dependency order

---

## discovery-20260531-us0002 — US-0002 cashflow forecasting UX discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-05-31  
**Story:** US-0002  
**Next phase:** `/research`

### Summary

Discovery captured design/UX references for cashflow forecasting MVP. US-0002 delivers the Forecast Engine, TimescaleDB hypertables, React `/forecast` page with ECharts, and Grafana Dashboards 1 (Cashflow) and 5 (Forecast horizons). Builds on US-0001 synced Firefly mirrors and Grafana datasource provisioning.

### UX references captured (see `docs/product/vision.md`)

| Reference | US-0002 application |
|-----------|---------------------|
| **Finanzguru** | Account balance projections with horizon pickers (daily / monthly / long-term); stat cards for near-term milestones; proactive "where will I be?" density without UI clone |
| **Firefly III** | Account selector uses synced Firefly asset accounts; labels follow Firefly account vocabulary |
| **shadcn/ui** | Enable Forecast nav at `/forecast`; Card, Select, Tabs/ToggleGroup layout; TanStack Query for forecast API |
| **Apache ECharts** | Daily balance line with milestone markers; monthly income/fixed/variable/free-cashflow bars; long-term area/line with 3/6/12/24-month pills |
| **Grafana Dashboard 1** | Cashflow: balance time series, forecast overlay, scarcity threshold reference lines (visual only until US-0005 Alert Engine) |
| **Grafana Dashboard 5** | Forecast horizons: 1/3/6/12-month panels (Projectplan); optional 24-month panel aligned with React selector |

### Scope refinements (backlog updated)

- Forecast Engine: daily (tomorrow, next week, month-end), monthly (income/fixed/variable/free cashflow), long-term (3/6/12/24 months)
- React: `/forecast` route replaces disabled nav placeholder; account selector + horizon tabs + ECharts
- TimescaleDB hypertables for forecast time series (per R-0004 prep)
- Recompute forecast after successful Firefly sync
- Grafana Dashboard 1 + 5 provisioned as JSON alongside Platform Health
- Out of scope unchanged: ML (US-0009), subscription adjustments (US-0003), plan overlays (US-0004), Dashboards 2–4

### Discovery decomposition evidence

- Feature/workflow count: 3 forecast granularities + React page + 2 Grafana dashboards + DB hypertables (moderate breadth — single story retained)
- Cross-cutting impact: backend engine, DB migration, React UI, Grafana provisioning
- Acceptance breadth: unchanged (8 criteria)
- Risk surface: sparse-transaction forecast accuracy, external TimescaleDB hypertable migration, sync-triggered recompute latency

### Open questions (carry to research/architecture)

- Baseline forecast algorithm: rolling averages from transaction history vs simple recurring-pattern inference (before US-0003 subscription engine)?
- Per-account vs household aggregate default in account selector?
- Hypertable schema: store daily snapshot points vs on-demand compute with cache?
- Dashboard 5 horizon set: strict Projectplan 1/3/6/12 or extend to 24 months to match React?
- Scarcity markers on Dashboard 1: static configurable threshold vs placeholder until US-0005?

### Recommended next steps

1. `/research` — forecast algorithm patterns, TimescaleDB hypertable schema for forecast series, Grafana dashboard-as-code (extends R-0004, R-0005)
2. `/architecture` — Forecast Engine contract, API endpoints, recompute trigger on sync completion, hypertable migration plan
3. `/sprint-plan` — S0002 decomposition for US-0002 acceptance criteria

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

