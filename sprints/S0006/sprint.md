# Sprint S0006

**ID:** S0006  
**Story:** US-0006 — AI financial assistant with privacy-safe tool layer  
**Status:** PLANNED  
**Created:** 2026-05-31

## Goal

Deliver the privacy-safe AI assistant on top of US-0001–US-0005 services: migration 006 audit persistence, `TransactionsService` + `PlanService::project_ephemeral`, central `PrivacyLayer`, `AiTool` registry with six in-process tools, `AiOrchestrator` with OpenAI tool-calling loop, SSE `POST /api/v1/chat/stream`, React header `Sheet` drawer + `/chat` sharing `ChatPanel`, Settings AI & Privacy + audit table, tests, and operator user guide.

## Scope

- Migration `006_ai_audit.sql` — `ai_tool_audit` table, indexes, startup retention (500 cap + 90-day purge) (DEC-0034)
- Config: TOML `[ai]` + `[privacy]`; extend `GET /api/v1/settings` (secrets excluded) (DEC-0032, DEC-0033)
- `TransactionsService`: read-only aggregates for `get_transactions` (DEC-0035, R-0031)
- `PlanService::project_ephemeral` / `project_readonly` sharing `plan::overlay` (DEC-0035, R-0031)
- `PrivacyLayer`: central JSON walker + IBAN/counterparty redaction; aggregates when raw disabled (DEC-0032)
- `AiTool` trait registry + OpenAI provider factory; stub `AiProvider` for US-0008 (DEC-0031)
- Six tools: `get_transactions`, `get_subscriptions`, `get_forecast`, `get_budget_status`, `get_portfolio`, `simulate_plan` (DEC-0035)
- `AiOrchestrator`: async-openai loop, max 5 rounds, 8 KB tool cap, audit insert (DEC-0031, DEC-0034)
- REST/SSE API: `POST /api/v1/chat/stream`, optional non-stream fallback, `GET /api/v1/ai/audit` (DEC-0033)
- React: `ChatPanel`, header `AiSheet`, `/chat` route, suggested prompts, tool transparency, privacy badge (DEC-0036)
- Settings: read-only AI & Privacy TOML display + audit log table (DEC-0032, DEC-0034, DEC-0036)
- Tests and operator user guide (`docs/user-guides/US-0006.md`)

**Out of scope:** Local/self-hosted AI providers (US-0008 — stub only), ML forecasts (US-0009), Grafana AI dashboard, DB-persisted chat threads, runtime-editable privacy toggles, inline ECharts in chat, user message pre-redaction, any Firefly write or direct AI SQL access.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Privacy bypass via nested JSON | Central walker + field allowlist + unit/integration tests | R-0028, DEC-0032 |
| Prompt injection / tool arg abuse | JSON Schema + serde validation; registry allowlist | R-0027, DEC-0031 |
| OpenAI key exposure | Env-only via `[ai] api_key_env`; never in TOML/browser/audit | R-0027, R-0030 |
| Missing OpenAI key at runtime | Graceful SSE error + Settings "not configured" state | R-0027 |
| Ephemeral vs persisted plan drift | Share `plan::overlay` for `project_ephemeral` | R-0031, DEC-0035 |
| SSE proxy buffering | `X-Accel-Buffering: no` + 15s keep-alive | R-0029, DEC-0033 |
| `get_budget_status` drift from Alert Engine | Compose from same R-0022 rules via `AlertService` | R-0031, DEC-0035 |
| Context window overflow | 8 KB tool cap + series downsample to 30 points | R-0031, DEC-0035 |
| US-0008 scope creep | Provider stub trait only; OpenAI path first | R-0027 |
| Settings read-only confusion | User guide documents TOML edit + restart | R-0028 |

## Definition of Done

- All 12 sprint tasks complete (`T-0061` … `T-0072`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0006
- Chat accepts natural-language questions via SSE stream with Bearer JWT
- AI invokes only six registered tools through services — no `DbPool` in AI path
- Privacy defaults honored; audit log queryable in Settings
- Example query tool mapping verified (mock or integration)
- User guide published at `docs/user-guides/US-0006.md`
- No Firefly write operations introduced

## Architecture references

- `docs/engineering/architecture.md` — US-0006
- Decisions: DEC-0031 … DEC-0036
- Research: R-0027 … R-0031
- Depends on: US-0001 OIDC + Settings; US-0002 `ForecastService`; US-0003 `SubscriptionService`; US-0004 `PlanService`; US-0005 `WealthService` + `AlertService`
