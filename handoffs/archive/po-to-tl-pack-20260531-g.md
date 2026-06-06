# PO to TL archive pack (2026-05-31)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 4
- Retained units in hot file: 9
- First archived heading: `## discovery-20260601-us0007 — US-0007 crypto exchange portfolio UX discovery`
- Last archived heading: `## research-20260531-us0006 — US-0006 AI assistant technical research`
- Verification tuple (mandatory):
  - archived_body_lines=224
  - retained_body_lines=495

---

## discovery-20260601-us0007 — US-0007 crypto exchange portfolio UX discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-01  
**Story:** US-0007  
**Next phase:** `/research`

### Summary

Discovery captured design/UX references for crypto exchange portfolio integration (Projectplan Phase 5). US-0007 delivers **read-only exchange connectors** (Binance, Bybit, Bitunix), **Portfolio Engine** (realized/unrealized/total-return PnL), **net worth extension** replacing US-0005 crypto placeholder, React **`/wealth` Crypto tab**, Settings **Crypto exchanges** credentials, **Sync Status** exchange rows, **allocation target** planning template, and **Grafana Dashboard 4** completion. Builds on US-0001 platform/sync shell, US-0005 wealth snapshots and Alert Engine pipeline, US-0004 Plan Engine for allocation scenarios, and US-0006 `get_portfolio` tool surface.

### UX references captured (see `docs/product/vision.md`)

| Reference | US-0007 application |
|-----------|---------------------|
| **Finanzguru** | Complete net worth with crypto; per-exchange holdings and PnL; allocation vs target (50/50 ETF/crypto); proactive portfolio drift without UI clone |
| **Exchange connectors** | Read-only API keys; Binance/Bybit/Bitunix start set; test connection in Settings; sync status parity with Firefly |
| **Firefly III** | Firefly remains transaction ledger (DEC-0004); exchange data in Flow DB only; combined headline with labeled subtotals |
| **shadcn/ui** | `/wealth` Tabs Overview \| Crypto; Settings Crypto exchanges section; Sync Status exchange table; holdings Table + PnL stat cards |
| **Apache ECharts** | Allocation pie (Firefly vs crypto); portfolio performance line from extended snapshots |
| **Grafana Dashboard 4** | Crypto value stat, allocation pie, performance time series; replace placeholder panel; uid `portfolio` |
| **US-0005 wealth** | Replace placeholder row; include crypto in `total`; extend `net_worth_snapshots` payload |
| **US-0004 planning** | Allocation target template (50/50, 70/20/10); current vs target gap display |
| **US-0006 AI** | `get_portfolio` includes crypto narrative once connected (extends R-0031) |

### Scope refinements (backlog updated)

- Exchange connectors: Binance, Bybit, Bitunix — balances, positions, trades, transfers, funding, PnL import
- Portfolio Engine: realized/unrealized gains, total return; holdings + trade persistence in Flow DB
- Sync: exchange phase in post-sync pipeline; Sync Status per-exchange rows; manual trigger
- Wealth: combined net worth headline; Overview \| Crypto tabs; PnL summary on Crypto tab
- Settings: Crypto exchanges credentials (masked, TOML/env secrets)
- Planning: allocation target template with current vs target comparison
- Grafana Dashboard 4: crypto value, allocation, performance panels (complete partial from US-0005)
- Out of scope unchanged: Kraken/Coinbase/Bitpanda/OKX, on-chain wallets, tax reporting, trade execution

### Discovery decomposition evidence

- Feature/workflow count: 3 connectors + portfolio engine + PnL + wealth/settings/sync UI + allocation template + Grafana (moderate-high — single story retained per `crypto-portfolio` plan area)
- Cross-cutting impact: backend connectors, portfolio engine, DB migration, React UI, plan template, wealth/AI extension, Grafana, sync pipeline
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md`)
- Risk surface: API key security, rate limits, FX crypto→EUR, PnL methodology, spot/futures scope, mutex latency, mixed-currency accuracy

### Open questions (carry to research/architecture)

- Exchange API auth: HMAC signing patterns per exchange; read-only key permissions required?
- Sync frequency vs rate limits: shared interval with Firefly or independent exchange cron?
- FX conversion: exchange ticker prices vs external rate API vs manual reporting-currency map?
- PnL methodology: FIFO/average-cost locally computed vs trust exchange-reported PnL?
- Product scope MVP: spot balances only vs futures/funding/margin positions on Bybit/Binance?
- Secret storage: TOML `[exchanges]` env override vs encrypted DB vault vs Docker secrets file?
- Pipeline order: new `exchanges` phase before alerts snapshot (extend DEC-0028) vs parallel async job?
- Allocation template: extend Plan Engine adjustments vs dedicated `allocation_targets` table?
- Snapshot schema: extend `net_worth_snapshots.total_eur` vs separate `portfolio_snapshots` hypertable?
- `get_portfolio` payload: include top-N holdings and PnL summary within 8 KB cap (DEC-0035)?
- Settings UX: runtime credential edit vs restart-required TOML display (mirror US-0006 privacy pattern)?
- Grafana allocation pie: Firefly account_role grouping vs flat asset list?

### Recommended next steps

1. `/research` — exchange REST API patterns (Binance/Bybit/Bitunix), read-only key setup, FX/rate sources, PnL calculation approaches, secret storage (extends R-0021/R-0026 deferrals)
2. `/architecture` — connector trait, Portfolio Engine contract, migration 007 schema, post-sync pipeline extension, wealth API changes, DEC-xxxx for FX defaults and PnL method
3. `/sprint-plan` — S0007 decomposition for US-0007 acceptance criteria

---

## sprint-plan-20260531-s0006 — US-0006 sprint decomposition

**From:** Tech Lead  
**To:** QA (`/plan-verify`) → Dev (`/execute`)  
**Date:** 2026-05-31  
**Story:** US-0006  
**Sprint:** S0006  
**Next phase:** `/plan-verify`

### Summary

Created sprint **S0006** with **12 tasks** (`T-0061` … `T-0072`) covering all US-0006 acceptance criteria. No sprint split (SPRINT_MAX_TASKS=12). Decomposition follows architecture layers: migration 006, PrivacyLayer, tool registry, six tools, AiOrchestrator SSE chat API, React `/chat` + Sheet drawer, Settings AI privacy audit UI, tests, user guide.

### Task summary

| ID | Title |
|----|-------|
| T-0061 | SQLx migration 006 ai_tool_audit schema and retention |
| T-0062 | AI and privacy config plus TransactionsService |
| T-0063 | PlanService project_ephemeral and project_readonly |
| T-0064 | PrivacyLayer central redaction middleware |
| T-0065 | AiTool trait registry and OpenAI provider |
| T-0066 | Six AI tool implementations |
| T-0067 | AiOrchestrator SSE chat API and audit endpoints |
| T-0068 | React ChatPanel and SSE streaming client |
| T-0069 | React header Sheet drawer and /chat route |
| T-0070 | Settings AI and Privacy audit UI |
| T-0071 | AI privacy orchestrator and tool tests |
| T-0072 | Operator user guide |

### Split decision

- Exactly at SPRINT_MAX_TASKS=12; single story retained
- Six tools grouped under T-0066 (shared registry contract)
- Frontend split: ChatPanel/SSE (T-0068), shell routing (T-0069), Settings audit (T-0070)

### Recommended next steps

1. `/plan-verify` — confirm task coverage against 6 AC
2. `/execute` — T-0061 through T-0072 in dependency order

---

## architecture-20260531-us0006 — US-0006 AI assistant architecture

**From:** Tech Lead  
**To:** Sprint planning (`/sprint-plan`)  
**Date:** 2026-05-31  
**Story:** US-0006  
**Next phase:** `/sprint-plan`

### Summary

Architecture complete for US-0006 privacy-safe AI assistant. Defines **AiOrchestrator** (async-openai tool-calling loop), **`AiTool` trait registry** (six tools), central **PrivacyLayer**, new **TransactionsService**, **`PlanService::project_ephemeral`**, migration **006** `ai_tool_audit`, SSE **`POST /api/v1/chat/stream`**, and React **header Sheet drawer + `/chat`** sharing `ChatPanel`. Six decisions (DEC-0031–DEC-0036). Spec-pack complete (3/3). Research R-0027–R-0031 incorporated.

### Architecture highlights

| Area | Decision |
|------|----------|
| **Orchestration** | `AiTool` trait + static registry; max 5 tool rounds; services-only `ToolContext` (DEC-0031, R-0027) |
| **Privacy** | Central middleware; Projectplan defaults; aggregates when raw disabled (DEC-0032, R-0028) |
| **Chat API** | SSE POST stream; Bearer JWT; ephemeral client threads (DEC-0033, R-0029) |
| **Audit** | Migration 006; 500 cap + 90-day purge; redacted args only (DEC-0034, R-0030) |
| **Tools** | Six in-process service tools; `project_ephemeral`; 8 KB cap (DEC-0035, R-0031) |
| **React UI** | Sheet drawer + `/chat` + Settings AI & Privacy + audit table (DEC-0036) |

### Decisions created

- **DEC-0031** — AI orchestration & tool registry
- **DEC-0032** — Privacy layer defaults & central redaction
- **DEC-0033** — Chat SSE API & ephemeral sessions
- **DEC-0034** — Audit persistence & retention
- **DEC-0035** — Six-tool service mapping & payload limits
- **DEC-0036** — React chat UX (Sheet + /chat)

### Spec-pack status

| Artifact | Status |
|----------|--------|
| `US-0006-design-concept.md` | Complete |
| `US-0006-crs.md` | Complete |
| `US-0006-technical-specification.md` | Complete |

### Risks carried to sprint-plan

1. New `TransactionsService` scope within US-0006
2. Privacy bypass via nested JSON — central walker + tests required
3. OpenAI key env-only — graceful degrade when unset
4. `get_budget_status` must align with Alert Engine rules (R-0022)
5. Ephemeral plan projection must share `plan::overlay` module
6. SSE proxy buffering — `X-Accel-Buffering: no`
7. US-0008 scope creep — provider stub only

### Recommended next steps

1. `/sprint-plan` — S0006 decomposition for US-0006 acceptance criteria
2. `/plan-verify` — confirm task coverage against 6 AC

---

## research-20260531-us0006 — US-0006 AI assistant technical research

**From:** Tech Lead  
**To:** Dev (via `/architecture` handoff)  
**Date:** 2026-05-31  
**Story:** US-0006  
**Next phase:** `/architecture`

### Summary

Web research completed for US-0006 privacy-safe AI assistant. Five new entries (R-0027–R-0031) cover OpenAI tool-calling orchestration in Axum, central privacy redaction middleware, SSE chat streaming with JWT auth (DEC-0006 gate closed), PostgreSQL audit persistence, and six-tool service mapping including read-only `simulate_plan`.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **OpenAI tool calling** | [R-0027](docs/engineering/research.md#r-0027--openai-tool-calling-orchestration-in-axum-async-openai) | `async-openai` Chat Completions + `AiTool` trait registry; 6 tools; max 5 tool rounds; services-only `ToolContext` (no DbPool) |
| **Privacy redaction** | [R-0028](docs/engineering/research.md#r-0028--privacy-redaction-middleware-for-ai-tool-outputs) | Central `PrivacyLayer` on tool output; aggregates when `allow_raw_transactions=false`; IBAN regex + counterparty hash8; Settings read-only TOML display |
| **Chat streaming** | [R-0029](docs/engineering/research.md#r-0029--chat-streaming-sse-with-jwt-auth-dec-0006-gate) | SSE `POST /api/v1/chat/stream` + fetch ReadableStream; Bearer JWT (no BFF); abort on disconnect; ephemeral client thread |
| **Audit log** | [R-0030](docs/engineering/research.md#r-0030--ai-tool-audit-log-persistence-migration-006) | Migration 006 `ai_tool_audit`; redacted args only; cap 500 rows + 90-day purge; `GET /api/v1/ai/audit` |
| **Tool mapping / simulate_plan** | [R-0031](docs/engineering/research.md#r-0031--six-tool-registry-mapping--simulate_plan-read-only-contract) | In-process service calls; new `TransactionsService` aggregates; `PlanService::project_ephemeral` for template what-ifs; 8 KB tool result cap |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Tool registry: trait vs static schema? | **Trait + JSON schema from registry** — static `serde_json` per tool; skip `openai-func-enums` for MVP (R-0027) |
| Streaming: SSE vs buffered? | **SSE MVP** via `POST /api/v1/chat/stream` (R-0029) |
| Redaction placement? | **Central middleware on tool output** before OpenAI + audit (R-0028) |
| `allow_raw_transactions=false` semantics? | **Category/month aggregates** — not empty payload; structured error if model needs raw rows (R-0028) |
| Audit: DB vs log file? | **PostgreSQL `ai_tool_audit`** migration 006 (R-0030) |
| Audit retention? | **500 row cap + 90-day purge** — configurable `[ai] audit_retention_days` (R-0030) |
| `simulate_plan` API mapping? | **Ephemeral `project_ephemeral`** — template/plan_id/active-plan resolution order; no persistence (R-0031) |
| Tool payload limits? | **8 KB max per tool result** with summarize/truncate (R-0031) |
| Rate limiting? | **In-memory per-user token bucket** on JWT sub (R-0029) |
| Settings privacy toggles? | **Read-only TOML display** for MVP — runtime edit deferred (R-0028) |
| Chat history persistence? | **Ephemeral client-side** — optional sessionStorage; no DB threads MVP (R-0029) |
| DEC-0006 BFF gate for SSE? | **Closed — Bearer JWT sufficient** on POST SSE (R-0029) |

### Risks surfaced (carry to architecture)

1. **Missing transactions API** — `get_transactions` needs new read-only `TransactionsService` (R-0031)
2. **Privacy bypass via field paths** — central JSON walker must cover nested mirror payloads (R-0028)
3. **Prompt injection / tool arg abuse** — strict JSON Schema + serde validation on args (R-0027)
4. **OpenAI key exposure** — env-only; never in browser or audit rows (R-0027, R-0030)
5. **Ephemeral vs persisted plan projection drift** — share `plan::overlay` module (R-0031)
6. **SSE proxy buffering** — require `X-Accel-Buffering: no` + keep-alive (R-0029)
7. **US-0008 scope creep** — provider abstraction stub only; OpenAI path first (R-0027)

### Recommended next steps

1. `/architecture` — AI orchestration contract, `AiTool` registry, `PrivacyLayer`, chat SSE API, migration 006 schema, `TransactionsService`, `PlanService::project_ephemeral`, DEC-xxxx for privacy defaults and audit retention
2. `/sprint-plan` — S0006 decomposition for US-0006 acceptance criteria
3. Spec-pack expansion for US-0006 (SPEC_PACK_MODE=1)

---

