# Design Concept — US-0006

## Summary

US-0006 delivers a **privacy-safe AI financial assistant**: six registered tools exposed via OpenAI function calling, central Privacy Layer (Projectplan TOML defaults), SSE streaming chat UI (header `Sheet` drawer + `/chat` full page), suggested prompt chips, tool transparency, Settings AI & Privacy section, and operator tool audit log.

Builds on US-0002 forecast, US-0003 subscriptions, US-0004 plan simulation, US-0005 wealth/budget-status APIs, and US-0001 OIDC + Settings shell.

## Goals

- AI Tool Layer: six tools via OpenAI function calling; each wraps existing Flow services — no SQL from AI path (DEC-0031, R-0027)
- Privacy Layer: `allow_raw_transactions=false`, `redact_iban=true`, `redact_counterparties=true` defaults; central middleware (DEC-0032, R-0028)
- Chat: SSE `POST /api/v1/chat/stream` with Bearer JWT; ephemeral client threads (DEC-0033, R-0029)
- Audit: migration 006 `ai_tool_audit`; 500 row cap + 90-day purge; Settings table (DEC-0034, R-0030)
- Tools: in-process service mapping including new `TransactionsService` and `PlanService::project_ephemeral` (DEC-0035, R-0031)
- React: header AI Sheet drawer + `/chat` + shared `ChatPanel`; Settings AI & Privacy (DEC-0036)
- Example queries: affordability, subscription price changes, budget overrun, cancel savings, top categories

## Non-goals

- Local/self-hosted AI providers (US-0008) — provider stub only
- ML-enhanced forecasts (US-0009)
- Grafana AI dashboard
- DB-persisted chat threads
- Runtime-editable privacy toggles in UI
- Inline ECharts in chat
- Any write to Firefly III or direct AI PostgreSQL access

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0031 | `AiTool` trait + async-openai loop | Testable registry; services-only context (R-0027) |
| DEC-0032 | Central privacy middleware | Single enforcement; Projectplan defaults (R-0028) |
| DEC-0033 | SSE POST + JWT | DEC-0006 gate closed; responsive UX (R-0029) |
| DEC-0034 | PostgreSQL audit + dual retention | Operator Settings query; storage minimization (R-0030) |
| DEC-0035 | Service mapping + ephemeral simulate | No REST self-calls; read-only what-ifs (R-0031) |
| DEC-0036 | Sheet drawer + `/chat` shared panel | Discovery UX; Finanzguru-style prompts (R-0029) |

**UX references:** Finanzguru conversational Q&A + suggested chips; Firefly-native labels in summaries; shadcn Sheet/Chat; tool transparency row — see `docs/product/vision.md`.

**Spec-pack traceability:** `docs/engineering/spec-pack/US-0006-crs.md`, `docs/engineering/spec-pack/US-0006-technical-specification.md`
