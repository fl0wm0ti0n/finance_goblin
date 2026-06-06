# CRS — US-0006

## Purpose

Deliver a privacy-safe AI financial assistant that answers natural-language questions via six registered tools (no direct database access), with OpenAI provider integration, Privacy Layer redaction, streaming chat UI, and operator tool audit log.

## Scope

**In scope:** AI Tool Layer (6 tools), OpenAI provider (TOML + env), Privacy Layer, SSE chat API, header Sheet drawer + `/chat`, suggested prompts, tool transparency, Settings AI & Privacy, migration 006 audit, `TransactionsService`, `PlanService::project_ephemeral`.

**Out of scope:** Local providers (US-0008), ML forecasts (US-0009), Grafana AI dashboard, DB chat threads, runtime privacy toggles, Firefly writes.

Canonical boundaries: `docs/product/backlog.md` — US-0006.

## Acceptance criteria ref

`docs/product/acceptance.md` — section **US-0006**:

1. Chat UI accepts natural-language questions about finances
2. AI invokes only registered tools — no direct DB access
3. OpenAI provider configurable via environment/config
4. Privacy settings honored in tool responses
5. Example queries work (affordability, subs, budget, savings, top categories)
6. Tool call audit log available for operator review

## Architecture traceability

- Architecture: `docs/engineering/architecture.md` — US-0006
- Decisions: DEC-0031 … DEC-0036
- Research: R-0027 … R-0031
