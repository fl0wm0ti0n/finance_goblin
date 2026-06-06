# Technical Specification — US-0006

## Overview

US-0006 adds **AiOrchestrator** with six registered tools, **PrivacyLayer**, **TransactionsService**, migration `006_ai_audit.sql`, SSE chat API, and React chat UI (header Sheet drawer + `/chat` + Settings AI & Privacy).

**Dependencies:** US-0001 JWT auth + Settings; US-0002 `ForecastService`; US-0003 `SubscriptionService`; US-0004 `PlanService` + overlay; US-0005 `WealthService` + `AlertService`.

## Components

### AiOrchestrator (`backend/src/ai/`)

```
AiOrchestrator::stream_chat(user, messages, session_id)
  ├─ rate_limit_check(user.sub)
  ├─ openai.create_stream(tools=registry.schemas(), messages)
  ├─ loop (max 5 rounds):
  │    ├─ on ToolCalls → validate args → registry.execute(tool, ctx, args)
  │    ├─ PrivacyLayer::redact_tool_result
  │    ├─ truncate to max_tool_result_bytes
  │    ├─ audit_repo.insert(redacted args_summary)
  │    └─ append tool messages → continue stream
  └─ emit SSE: token | tool_start | tool_end | done | error
```

Per DEC-0031 / R-0027. `ToolContext` holds services only — no `DbPool`.

### PrivacyLayer (`backend/src/ai/privacy.rs`)

```
PrivacyLayer::redact_tool_result(tool_name, json) -> json
PrivacyLayer::summarize_args(args) -> json   // for audit
```

Per DEC-0032 / R-0028. Defaults from TOML `[privacy]`.

### Six tools (`backend/src/ai/tools/`)

| Tool | Service call |
|------|--------------|
| `get_transactions` | `TransactionsService::aggregates(filter)` |
| `get_subscriptions` | `SubscriptionService::list_patterns` + price summary |
| `get_forecast` | `ForecastService` latest series (downsampled) |
| `get_budget_status` | Active plan MTD vs actual + drift alerts compose |
| `get_portfolio` | `WealthService::compute_breakdown` |
| `simulate_plan` | `PlanService::project_ephemeral(draft)` |

Per DEC-0035 / R-0031. Max 8 KB per result.

### TransactionsService (`backend/src/transactions/`)

```
TransactionsService::aggregates(AggregateFilter) -> TransactionAggregates
  ├─ repository::sum_by_category(period)
  └─ repository::totals(period)
```

Read-only mirror queries. Aggregates when `allow_raw_transactions=false`.

### PlanService extensions

```rust
PlanService::project_ephemeral(draft: PlanDraft) -> PlanProjectionSummary
PlanService::project_readonly(version_id: Uuid) -> PlanProjectionSummary
```

Shares `plan::overlay` with persisted recompute (DEC-0035).

### Database migration `006_ai_audit.sql`

Per architecture.md §6 / DEC-0034:

- Table: `ai_tool_audit` with tool name CHECK, redacted `args_summary` JSONB
- Indexes: `created_at DESC`, `(tool_name, created_at)`, `(session_id, created_at)`
- Retention job: 500 row cap + 90-day purge on startup

### REST / SSE API

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/v1/chat/stream` | Primary SSE chat (JWT) |
| POST | `/api/v1/chat/completions` | Non-streaming fallback |
| GET | `/api/v1/ai/audit?limit=&offset=` | Operator audit log |
| GET | `/api/v1/settings` | Extended with `[ai]` + `[privacy]` (no secrets) |

Per DEC-0033 / R-0029. Rate limit: in-memory token bucket per JWT `sub`.

### Config

```toml
[ai]
provider = "openai"
model = "gpt-4o-mini"
api_key_env = "OPENAI_API_KEY"
max_tool_rounds = 5
max_completion_tokens = 1024
max_tool_result_bytes = 8192
request_timeout_secs = 60
rate_limit_per_min = 20
audit_retention_days = 90
audit_max_rows = 500

[privacy]
allow_raw_transactions = false
redact_iban = true
redact_counterparties = true
```

API key from env only — never in TOML or API responses.

### React frontend

| Component | Responsibility |
|-----------|----------------|
| `ChatPanel` | Messages, input, SSE stream hook, tool transparency, abort |
| `AiSheet` | Header Sheet drawer wrapping `ChatPanel` |
| `ChatPage` | Full `/chat` route with same `ChatPanel` |
| `SettingsPage` | AI & Privacy section + audit table |
| `useChatStream` | fetch + ReadableStream SSE parser |

Per DEC-0036. Suggested prompt chips on empty thread. Privacy badge when aggregates-only.

### AppState additions

```rust
ai: AiService,              // orchestrator + registry + rate limiter
transactions: TransactionsService,
```

OpenAI client constructed at startup when `OPENAI_API_KEY` present; graceful error when missing (Settings shows "not configured").

## Non-functional

- **Security:** tools-only boundary; no AI SQL; env-only API key; redacted audit
- **Privacy:** central redaction before any OpenAI or audit persistence
- **Latency:** SSE token streaming; abort on disconnect saves tokens
- **Observability:** audit log per tool invocation; structured tracing on orchestrator rounds

## Verification targets

- Unit: privacy redaction walker; tool arg validation; truncate helper
- Integration: chat stream with mock OpenAI; audit insert with redacted args
- E2E: suggested prompt → tool invocation → assistant response with "Tools used" row
- Operator: Settings audit table shows recent invocations
