# Tasks — Sprint S0006

**Story:** US-0006  
**Task count:** 12 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0061 | SQLx migration 006 ai_tool_audit schema and retention | open | AC-6 |
| T-0062 | AI and privacy config plus TransactionsService | open | AC-3, AC-4, AC-5 |
| T-0063 | PlanService project_ephemeral and project_readonly | open | AC-2, AC-5 |
| T-0064 | PrivacyLayer central redaction middleware | open | AC-4 |
| T-0065 | AiTool trait registry and OpenAI provider | open | AC-2, AC-3 |
| T-0066 | Six AI tool implementations | open | AC-2, AC-5 |
| T-0067 | AiOrchestrator SSE chat API and audit endpoints | open | AC-1, AC-2, AC-3, AC-6 |
| T-0068 | React ChatPanel and SSE streaming client | open | AC-1, AC-5 |
| T-0069 | React header Sheet drawer and /chat route | open | AC-1, AC-5 |
| T-0070 | Settings AI and Privacy audit UI | open | AC-3, AC-4, AC-6 |
| T-0071 | AI privacy orchestrator and tool tests | open | AC-2, AC-4, AC-5, AC-6 |
| T-0072 | Operator user guide | open | AC-1–AC-6 |

---

## T-0061 — SQLx migration 006 ai_tool_audit schema and retention

**Status:** open  
**Depends on:** US-0001 (migration 001) through US-0005 (migration 005)  
**Decisions:** DEC-0034, R-0030

### Description

Add SQLx migration `006_ai_audit.sql` per architecture § migration 006:

| Object | Purpose |
|--------|---------|
| `ai_tool_audit` | Operator audit log: session, user, tool name, redacted args, status, duration, model |

Columns and constraints:

- `tool_name` CHECK IN six registered tools only
- `result_status` CHECK IN (`ok`, `error`)
- `args_summary` JSONB — redacted only; never raw transaction rows or API keys
- Indexes: `created_at DESC`, `(tool_name, created_at DESC)`, `(session_id, created_at DESC)`

**Retention startup job (DEC-0034):**

- Purge rows older than `[ai] audit_retention_days` (default 90)
- Enforce `[ai] audit_max_rows` cap (default 500) — delete oldest beyond cap
- Run once at backend startup after migrations

Implement `ai::audit::repository`:

- `insert(row)` — called by orchestrator after each tool invocation
- `list(limit, offset)` — for Settings audit table and `GET /ai/audit`
- `purge_expired()` — retention job

**What NOT to store:** full prompts/responses, unredacted payloads, OpenAI API keys, raw transaction rows.

### Done when

- [ ] Migration applies cleanly against external PostgreSQL
- [ ] Table schema and indexes match architecture
- [ ] `tool_name` and `result_status` CHECK constraints enforce allowlist
- [ ] Retention job purges by age and enforces row cap at startup
- [ ] Audit repository insert/list methods wired for orchestrator use

---

## T-0062 — AI and privacy config plus TransactionsService

**Status:** open  
**Depends on:** T-0061, US-0001 mirror schema  
**Decisions:** DEC-0032, DEC-0035, R-0031

### Description

**TOML config additions:**

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

Load into `AiConfig` and `PrivacyConfig` at startup. OpenAI key resolved from env via `api_key_env` — never from TOML plaintext.

**Extend `GET /api/v1/settings`:** include `ai` and `privacy` sections; exclude secrets; expose `openai_configured: bool` derived from env presence.

**New `transactions` module** (`backend/src/transactions/`):

```rust
TransactionsService::aggregates(AggregateFilter {
    period_start, period_end,
    category_id: Option<i64>,
    group_by: Category | Month,
}) -> TransactionAggregates
```

- Read-only queries against mirror `transactions` + `categories`
- When `allow_raw_transactions=false`: category/month totals, counts, inflow/outflow — no row arrays
- When `true`: capped list (max 20 rows, default last 30 days) — still passes through `PrivacyLayer` later
- No dedicated REST endpoint required for MVP (internal to AI tool path)

Wire `TransactionsService` into `AppState`.

### Done when

- [ ] Config loads `[ai]` and `[privacy]` from TOML + env overlay
- [ ] Settings API returns ai/privacy sections without secrets
- [ ] `openai_configured` flag reflects env key presence
- [ ] `TransactionsService::aggregates` returns category/month aggregates
- [ ] Raw mode returns capped row list when enabled in config
- [ ] Unit tests cover aggregate grouping and raw cap

---

## T-0063 — PlanService project_ephemeral and project_readonly

**Status:** open  
**Depends on:** US-0004 `PlanService`, US-0002 forecast baseline  
**Decisions:** DEC-0035, R-0031

### Description

Extend `plan::service::PlanService` with read-only ephemeral projection for `simulate_plan` tool:

```rust
impl PlanService {
    pub async fn project_readonly(
        &self, plan_id: Uuid, version_number: Option<u32>,
    ) -> Result<PlanProjection, PlanError>;

    pub async fn project_ephemeral(
        &self, draft: EphemeralPlanDraft,
    ) -> Result<PlanProjection, PlanError>;
}
```

**Resolution order for `simulate_plan` (no persistence):**

1. `plan_id` (+ optional `version_number`) → load version adjustments → `project_readonly`
2. Else `template` → template defaults as draft adjustments → `project_ephemeral`
3. Else active plan latest version
4. Else structured `{ "error": "no_plan_context" }`

**Critical:** share `plan::overlay` math with persisted recompute — no duplicate projection logic (R-0031).

Templates: `current`, `leasing`, `savings_mode`, `house_purchase` per US-0004.

Return projection summary suitable for 8 KB cap: monthly delta sum, projected month-end balance, key metrics — not full daily series unless downsampled.

### Done when

- [ ] `project_readonly` loads persisted plan version and projects without writes
- [ ] `project_ephemeral` applies draft/template adjustments without persistence
- [ ] Template resolution matches US-0004 built-in scenarios
- [ ] Overlay math reuses shared `plan::overlay` module
- [ ] Unit tests cover resolution order and template defaults
- [ ] No Firefly or plan DB writes on ephemeral path

---

## T-0064 — PrivacyLayer central redaction middleware

**Status:** open  
**Depends on:** T-0062 (PrivacyConfig)  
**Decisions:** DEC-0032, R-0028

### Description

Implement `backend/src/ai/privacy.rs`:

```rust
pub struct PrivacyLayer { config: PrivacyConfig, pepper: String }
impl PrivacyLayer {
    pub fn redact_tool_result(&self, tool_name: &str, value: serde_json::Value)
        -> serde_json::Value;
    pub fn summarize_args(&self, args: &serde_json::Value) -> serde_json::Value;
}
```

**Redaction rules (Projectplan defaults):**

| Setting | Default | Behavior |
|---------|---------|----------|
| `allow_raw_transactions` | `false` | `get_transactions` output is aggregates only |
| `redact_iban` | `true` | ISO IBAN → `[IBAN_REDACTED]` |
| `redact_counterparties` | `true` | payee/description → `Counterparty-{hash8}` (SHA-256 + app pepper) |

- Recursive JSON walker on known field names (`iban`, `payee`, `description`, `counterparty`, `destination_name`) + regex on string leaves
- When raw disabled and payload contains row arrays → replace with structured `{ "error": "raw_transactions_disabled", "hint": "use category aggregates" }`
- `summarize_args` produces redacted `args_summary` for audit rows — no PII

Pepper from env `PRIVACY_PEPPER` or derived app secret — document in user guide.

### Done when

- [ ] IBAN strings redacted in nested JSON structures
- [ ] Counterparty fields hashed with stable hash8 per value
- [ ] Raw transaction rows blocked when `allow_raw_transactions=false`
- [ ] `summarize_args` safe for audit persistence
- [ ] Unit tests cover nested payloads, IBAN regex, counterparty hash stability
- [ ] Integration test: tool output with mirror-like nested JSON fully scrubbed

---

## T-0065 — AiTool trait registry and OpenAI provider

**Status:** open  
**Depends on:** T-0062, T-0064  
**Decisions:** DEC-0031, R-0027

### Description

Implement `backend/src/ai/` core scaffolding:

**Trait contract:**

```rust
pub trait AiTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, ctx: &ToolContext, args: serde_json::Value)
        -> Result<serde_json::Value, ToolError>;
}

pub struct ToolContext {
    pub transactions: TransactionsService,
    pub subscriptions: SubscriptionService,
    pub forecast: ForecastService,
    pub plans: PlanService,
    pub wealth: WealthService,
    pub alerts: AlertService,
    pub privacy: PrivacyConfig,
    pub user_subject: String,
    pub session_id: Uuid,
    // NO DbPool — tools call services only
}
```

**Registry (`ai/registry.rs`):**

- Static `Vec<Arc<dyn AiTool>>` populated at startup
- `build_openai_tools()` → OpenAI Chat Completions `tools` array from JSON schemas
- Registry allowlist — reject unknown tool names at execution

**Provider (`ai/provider.rs`):**

- OpenAI client factory from `AiConfig` + env key
- Stub `AiProvider` trait for US-0008 — OpenAI implementation only in this sprint
- Graceful error when key missing: `ProviderNotConfigured`

Wire `AiService` shell into `AppState` (orchestrator filled in T-0067).

### Done when

- [ ] `AiTool` trait and `ToolContext` defined without `DbPool`
- [ ] Registry builds OpenAI tools array from six tool slots (implementations in T-0066)
- [ ] OpenAI client created when env key present; clear error when absent
- [ ] `AiProvider` stub trait present for US-0008 extension
- [ ] Unit test: registry contains exactly six tool names matching migration CHECK

---

## T-0066 — Six AI tool implementations

**Status:** open  
**Depends on:** T-0062, T-0063, T-0065  
**Decisions:** DEC-0035, R-0031, R-0022

### Description

Implement six tools under `backend/src/ai/tools/` — each delegates to in-process services only:

| Tool | Service method | Key args |
|------|----------------|----------|
| `get_transactions` | `TransactionsService::aggregates` | `period`, `category_id?`, `group_by` |
| `get_subscriptions` | `SubscriptionService::list_patterns` + price summary | `status`, `kind`, `include_price_events?` |
| `get_forecast` | `ForecastService` latest computation | `horizon`, `account_id?` |
| `get_budget_status` | Compose from `AlertService` + active plan MTD | `category_id?` — mirrors R-0022 budget drift inputs |
| `get_portfolio` | `WealthService::compute_breakdown` + optional `history(90)` | `include_history?` |
| `simulate_plan` | `PlanService::project_ephemeral` / `project_readonly` | `template?`, `plan_id?`, `version_number?`, `adjustments?` |

**Per-tool requirements:**

- Strict JSON Schema in `parameters_schema`; serde deserialize + validate args before execute
- Return JSON serializable to ≤ `[ai] max_tool_result_bytes` (8192) — downsample series to 30 points + `{ min, max, latest }`; set `{ "truncated": true }` on overflow
- No `sqlx::query` or Firefly HTTP from `ai/tools/*`

**Example query mapping (acceptance AC-5):**

| Query | Primary tool(s) |
|-------|-----------------|
| Leasing affordability | `simulate_plan` (template=leasing) |
| Subscription price increases | `get_subscriptions` (price events) |
| Budget overrun this month | `get_budget_status` + `get_transactions` |
| Savings if cancel subscription | `simulate_plan` (savings_mode) + `get_subscriptions` |
| Top spending categories | `get_transactions` (aggregates) |

Register all six in registry at startup.

### Done when

- [ ] All six tools implement `AiTool` and register in static registry
- [ ] Each tool calls service layer only — no direct DB or Firefly access
- [ ] Args validated via serde + schema before execution
- [ ] Results respect 8 KB cap with downsample/truncate behavior
- [ ] `get_budget_status` composes same MTD/category logic as Alert Engine (R-0022)
- [ ] `simulate_plan` follows resolution order from T-0063
- [ ] Unit tests per tool with mocked services

---

## T-0067 — AiOrchestrator SSE chat API and audit endpoints

**Status:** open  
**Depends on:** T-0061, T-0064, T-0065, T-0066  
**Decisions:** DEC-0031, DEC-0033, DEC-0034, R-0029

### Description

**AiOrchestrator (`ai/orchestrator.rs`):**

1. Append user message from request body to in-memory thread
2. `client.chat().create_stream(request)` with `tools` + `tool_choice: auto`
3. On `FinishReason::ToolCalls` → validate args → execute tools (parallel `tokio::join!` when multiple)
4. `PrivacyLayer::redact_tool_result` → truncate to max bytes → append `role: tool` messages
5. Insert `ai_tool_audit` row per invocation via repository (redacted `args_summary`)
6. Repeat until `FinishReason::Stop` or **max_tool_rounds** (default 5)
7. Forward SSE events to client

Minimal system prompt: use tools for factual data; never invent balances; respect privacy aggregates.

**REST/SSE handlers (`api/chat.rs`, `api/ai_audit.rs`):**

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/v1/chat/stream` | Primary SSE stream for ChatPanel |
| POST | `/api/v1/chat/completions` | Non-streaming fallback (tests/admin) |
| GET | `/api/v1/ai/audit` | Operator audit log (`limit`, `offset`) |

**SSE event types:**

```text
event: token       data: {"delta":"..."}
event: tool_start  data: {"tool":"get_forecast","call_id":"..."}
event: tool_end    data: {"tool":"get_forecast","duration_ms":42,"status":"ok"}
event: done        data: {"message_id":"...","tools_used":[...]}
event: error       data: {"code":"provider_error","message":"..."}
```

**Handler requirements (DEC-0033):**

- JWT Bearer auth (`require_auth`) — DEC-0006 gate closed
- Per-user in-memory token bucket rate limit on JWT `sub` (`[ai] rate_limit_per_min`)
- `mpsc` channel → `Sse::new(ReceiverStream)` + 15s keep-alive
- `CancellationToken` aborts OpenAI stream on client disconnect
- Headers: `Content-Type: text/event-stream`, `Cache-Control: no-cache`, `X-Accel-Buffering: no`

Request body: `{ "messages": [...], "session_id": "uuid-optional" }`.

### Done when

- [ ] Orchestrator completes tool-calling loop with max 5 rounds
- [ ] Every tool output passes through PrivacyLayer before OpenAI and audit
- [ ] Audit row inserted per tool invocation with redacted args
- [ ] SSE stream emits token, tool_start, tool_end, done, error events
- [ ] Rate limit returns 429 when exceeded
- [ ] Client disconnect cancels OpenAI stream
- [ ] Non-streaming fallback endpoint works for tests
- [ ] `GET /api/v1/ai/audit` returns paginated audit rows
- [ ] Missing OpenAI key returns structured SSE error

---

## T-0068 — React ChatPanel and SSE streaming client

**Status:** open  
**Depends on:** T-0067  
**Decisions:** DEC-0033, DEC-0036, R-0029

### Description

Implement shared chat components under `frontend/src/components/chat/`:

| Component | Responsibility |
|-----------|----------------|
| `ChatPanel` | Message list, input, streaming token render, abort on unmount |
| `useChatStream` hook | Authenticated `fetch()` + `ReadableStream` SSE parser |
| `MessageBubble` | User/assistant styling; collapsible "Tools used" row |
| `SuggestedPrompts` | Empty-state chips (Projectplan examples) |
| `PrivacyBadge` | "Privacy: aggregates only" when raw disabled |

**SSE client (not EventSource — lacks POST + Authorization):**

- POST `/api/v1/chat/stream` with Bearer JWT
- Parse partial chunks; handle `token`, `tool_start`, `tool_end`, `done`, `error` events
- AbortController on unmount/navigation
- Accumulate assistant message from token deltas
- Tool transparency: show tool name + timestamp under assistant messages — no raw JSON

**Suggested prompt chips (empty thread):**

- "Kann ich mir ein Leasing Auto leisten?"
- "Welche Abos sind teurer geworden?"
- "Warum bin ich diesen Monat über Budget?"
- "Wie viel spare ich wenn ich Netflix kündige?"
- "Top Ausgabenkategorien diesen Monat"

Ephemeral thread state in React; optional `sessionStorage` restore — no server persistence.

### Done when

- [ ] `useChatStream` parses SSE events from fetch ReadableStream
- [ ] Streaming tokens render incrementally in assistant bubble
- [ ] Abort cancels in-flight stream on unmount
- [ ] Tool transparency row shows tools used with timestamps
- [ ] Suggested prompts populate input and send on click
- [ ] Privacy badge reflects settings `allow_raw_transactions`
- [ ] Error state when OpenAI not configured

---

## T-0069 — React header Sheet drawer and /chat route

**Status:** open  
**Depends on:** T-0068  
**Decisions:** DEC-0036

### Description

Wire chat UX into app shell:

| UI element | Implementation |
|------------|----------------|
| Header AI button | Opens shadcn `Sheet` side drawer (~400px) with `ChatPanel` |
| `/chat` route | Full-page chat; enable sidebar AI nav (replace US-0001 disabled placeholder) |
| Shared state | Same `ChatPanel` component in Sheet and full page — independent thread state per surface acceptable for MVP |

**Nav changes:**

- Enable AI nav item at `/chat`
- Remove "Coming soon" badge from AI nav entry

Add shadcn `Sheet` primitive if not present (Radix dialog-based).

Header layout: place AI button near existing header controls (bell, sync badge).

### Done when

- [ ] Header AI button opens Sheet drawer with functional ChatPanel
- [ ] `/chat` route reachable when authenticated
- [ ] Sidebar AI nav enabled without "Coming soon" badge
- [ ] Sheet and full page both stream chat responses
- [ ] Drawer closes without breaking abort semantics
- [ ] `npm run build` succeeds with new routes/components

---

## T-0070 — Settings AI and Privacy audit UI

**Status:** open  
**Depends on:** T-0067, T-0068  
**Decisions:** DEC-0032, DEC-0034, DEC-0036, R-0028

### Description

Extend Settings page with **AI & Privacy** section:

| UI element | Implementation |
|------------|----------------|
| AI config display | Read-only table from settings API `[ai]` section (model, max rounds, rate limit — no key) |
| OpenAI status | Badge: configured / not configured from `openai_configured` |
| Privacy config display | Read-only `[privacy]` toggles (allow_raw_transactions, redact_iban, redact_counterparties) |
| Edit note | Copy: "Edit config.toml and restart to change" — runtime toggle deferred |
| Audit log table | TanStack Query fetch from `GET /api/v1/ai/audit`; columns: timestamp, tool, user, duration, status |
| Pagination | limit/offset for audit rows |

No secrets rendered. Audit table shows redacted args summary only.

### Done when

- [ ] Settings page includes AI & Privacy section
- [ ] AI and privacy TOML values displayed read-only
- [ ] OpenAI configured/not-configured badge accurate
- [ ] Audit table lists recent tool invocations with pagination
- [ ] No API keys or raw transaction data in UI
- [ ] Restart-required note present for config changes

---

## T-0071 — AI privacy orchestrator and tool tests

**Status:** open  
**Depends on:** T-0064, T-0066, T-0067, T-0069, T-0070  
**Decisions:** DEC-0031, DEC-0032, DEC-0034, DEC-0004

### Description

Add Rust unit and integration tests:

**PrivacyLayer:**

- Nested JSON with IBAN, payee, description fields
- Raw transactions disabled → aggregate-only enforcement
- Counterparty hash stability

**Registry / tools:**

- Six tool names match migration CHECK constraint
- Args validation rejects malformed JSON
- 8 KB truncation/downsample behavior
- `get_budget_status` MTD logic alignment with alert evaluator

**Orchestrator (mock OpenAI):**

- Single tool round completes and emits audit row
- Max rounds respected
- PrivacyLayer invoked on every tool output

**Integration test (`ai_assistant_integration`):**

- With `DATABASE_URL`: migration 006 + audit insert + list endpoint
- Chat stream with mock provider or recorded fixture
- Skip without `DATABASE_URL` (same pattern as US-0001–US-0005)

**Static audits:**

- `ai/` modules have no `sqlx::query` or Firefly client usage
- No OpenAI key strings in audit insert paths

Extend `tests/run-tests.sh` to include AI test targets.

Optional: example query smoke tests mapping AC-5 tool chains (unit level with mocks).

### Done when

- [ ] PrivacyLayer unit tests pass for nested redaction cases
- [ ] Tool unit tests pass with mocked services
- [ ] Orchestrator mock test completes tool round + audit insert
- [ ] Integration test validates audit persistence (or SKIP without DATABASE_URL)
- [ ] Static audit: no DbPool/Firefly in ai path
- [ ] `bash tests/run-tests.sh` includes AI tests and passes

---

## T-0072 — Operator user guide

**Status:** open  
**Depends on:** T-0068, T-0069, T-0070, T-0071  
**Decisions:** —

### Description

Create `docs/user-guides/US-0006.md` per USER_GUIDE_MODE=1:

- Prerequisites: US-0001–US-0005 operational; `OPENAI_API_KEY` in env; synced Firefly data
- Enabling AI: TOML `[ai]` section + env key; model selection
- Privacy defaults: `allow_raw_transactions=false`, IBAN/counterparty redaction; what aggregates mean
- Using chat: header Sheet drawer vs `/chat` full page; suggested prompts
- Tool transparency: "Tools used" row meaning
- Example queries and expected tool behavior (AC-5 mapping)
- Settings AI & Privacy: read-only display; how to change TOML + restart
- Audit log: what is recorded; retention (500 rows / 90 days)
- Troubleshooting: missing API key, rate limit, provider errors
- Read-only guarantee: tools wrap services only; no Firefly mutation
- US-0008 boundary: OpenAI only in this release; local providers coming

### Done when

- [ ] User guide covers all six acceptance criteria from operator perspective
- [ ] Privacy defaults and redaction behavior documented
- [ ] Example queries mapped to tools
- [ ] OpenAI key setup and Settings audit log documented
- [ ] TOML edit + restart workflow documented for privacy changes

---

## Execution order (recommended)

1. **Database + config:** T-0061 → T-0062
2. **Services:** T-0063 (parallel with T-0062 after T-0061)
3. **AI core:** T-0064 → T-0065 → T-0066
4. **Orchestration + API:** T-0067
5. **Frontend:** T-0068 → T-0069 → T-0070
6. **Verification:** T-0071 → T-0072

## Acceptance coverage map

| AC | Tasks |
|----|-------|
| Chat UI natural-language questions | T-0067, T-0068, T-0069, T-0072 |
| Registered tools only — no direct DB | T-0065, T-0066, T-0067, T-0071 |
| OpenAI configurable via env/config | T-0062, T-0065, T-0067, T-0070, T-0072 |
| Privacy settings honored | T-0062, T-0064, T-0066, T-0067, T-0070, T-0071, T-0072 |
| Example queries work | T-0063, T-0066, T-0067, T-0068, T-0071, T-0072 |
| Tool call audit log for operator | T-0061, T-0067, T-0070, T-0071, T-0072 |
