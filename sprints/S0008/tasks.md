# Tasks — Sprint S0008

**Story:** US-0008  
**Task count:** 12 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0085 | AiProvider factory and trait extension | open | AC-1, AC-4 |
| T-0086 | OpenAiCompatibleProvider HTTP client refactor | open | AC-1, AC-2, AC-3 |
| T-0087 | Ollama and openai_compatible config plus settings API | open | AC-1, AC-2, AC-3 |
| T-0088 | Migration 008 audit provider column | open | AC-4 |
| T-0089 | Orchestrator dyn AiProvider refactor and local fallback | open | AC-1, AC-4, AC-5 |
| T-0090 | POST /api/v1/ai/test endpoint | open | AC-2, AC-3 |
| T-0091 | Settings AI provider UI and test connection button | open | AC-1, AC-2, AC-3 |
| T-0092 | ChatPanel Local vs Cloud provider badge | open | AC-1, AC-2 |
| T-0093 | Compose full profile Ollama operator documentation | open | AC-2 |
| T-0094 | Wiremock AC5 network isolation integration test | open | AC-5 |
| T-0095 | Provider factory and orchestrator unit tests | open | AC-1, AC-4, AC-5 |
| T-0096 | Operator user guide US-0008 | open | AC-1–AC-5 |

---

## T-0085 — AiProvider factory and trait extension

**Status:** open  
**Depends on:** US-0006 `ai::provider` stub, `AiService` shell  
**Decisions:** DEC-0043, DEC-0044, R-0040

### Description

Extend stub `AiProvider` trait and add factory in `backend/src/ai/provider.rs`:

**Types:**

| Type | Purpose |
|------|---------|
| `AiProviderKind` | Parse TOML `provider`: `OpenAi`, `Ollama`, `OpenAiCompatible` |
| `ProviderError` | Config validation, HTTP, JSON parse errors |
| `build_provider(&AiConfig) -> Result<Arc<dyn AiProvider>, ProviderError>` | Startup factory |

**Trait methods (DEC-0043):**

```rust
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_configured(&self) -> bool;
    fn is_local(&self) -> bool;
    fn display_label(&self) -> &str;
    fn omit_tool_choice(&self) -> bool;
    fn default_temperature(&self) -> f32;
    async fn chat_completion(&self, req: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProviderError>;
    async fn chat_completion_stream(&self, req: ChatCompletionRequest) -> Result<reqwest::Response, ProviderError>;
}
```

**Factory preset resolution (DEC-0044):**

| Mode | `base_url` | API key | `omit_tool_choice` | `is_local` | `is_configured` |
|------|------------|---------|-------------------|------------|-----------------|
| `openai` | `https://api.openai.com/v1` | env **required** | `false` | `false` | key present |
| `ollama` | default `http://ollama:11434/v1` | optional | `true` | `true` | always true |
| `openai_compatible` | TOML **required** | optional | `true` | `true` | `base_url` non-empty |

Wire `AiService` to hold `Arc<dyn AiProvider>` built at startup via `build_provider`. Replace direct `OpenAiProvider` construction in `AppState` / `main.rs`.

**Alternative considered:** separate `OllamaProvider` type — rejected (duplicate HTTP; R-0040).

**Frozen:** do not modify `ai::registry`, `ai::privacy`, `ai::tools`.

### Done when

- [ ] `AiProvider` trait extended with factory contract methods
- [ ] `AiProviderKind` parses three valid modes; invalid value fails startup
- [ ] `build_provider` returns `Arc<dyn AiProvider>` with correct preset flags
- [ ] `AiService` holds injected provider at startup
- [ ] `display_label` returns `Cloud · OpenAI` / `Local · Ollama` / `Local · Compatible`

---

## T-0086 — OpenAiCompatibleProvider HTTP client refactor

**Status:** open  
**Depends on:** T-0085  
**Decisions:** DEC-0043, DEC-0045, R-0038, R-0039

### Description

Implement unified HTTP client replacing US-0006 `OpenAiProvider`:

```rust
pub struct OpenAiCompatibleProvider {
    base_url: String,           // no trailing slash; append /chat/completions
    api_key: Option<String>,
    model: String,
    timeout: Duration,
    kind: AiProviderKind,
}
```

**HTTP behavior:**

- POST `{base_url}/chat/completions` with JSON body
- Bearer header only when `api_key` is `Some(non_empty)` (R-0038 dummy-key pattern)
- Shared reqwest client with `[ai] request_timeout_secs`
- Parse non-stream and stream responses into internal types consumed by orchestrator
- Map HTTP 4xx/5xx to `ProviderError` with response body snippet for test endpoint

**Refactor scope:** migrate existing OpenAI path to this type with `kind = OpenAi` — preserve US-0006 chat behavior for `provider = "openai"`.

Do **not** embed tool loop logic in provider — orchestrator owns rounds.

### Done when

- [ ] Single `OpenAiCompatibleProvider` handles all three factory presets
- [ ] OpenAI cloud path behavior unchanged for existing US-0006 config
- [ ] Bearer omitted when API key env unset/empty (local)
- [ ] `chat_completion` and `chat_completion_stream` POST to configured base URL
- [ ] Provider unit tests with wiremock/local mock server for 200/401/connection refused

---

## T-0087 — Ollama and openai_compatible config plus settings API

**Status:** open  
**Depends on:** T-0085  
**Decisions:** DEC-0044, DEC-0047, R-0038, R-0039

### Description

Extend `AiConfig` and TOML loading in `backend/src/config/`:

```toml
[ai]
provider = "openai"                    # openai | ollama | openai_compatible
base_url = ""                          # required when openai_compatible
model = "gpt-4o-mini"
api_key_env = "OPENAI_API_KEY"
temperature = 0.7                      # default 0.7 openai / 0.3 local if unset
local_tool_nudge_retry = true
# unchanged: max_tool_rounds, max_completion_tokens, max_tool_result_bytes,
#            request_timeout_secs, rate_limit_per_min, audit_retention_*
```

**Env overrides (optional):** `AI_PROVIDER`, `AI_BASE_URL`, `AI_MODEL`.

**Startup validation:**

- `openai_compatible` without `base_url` → startup error
- Log resolved provider label and effective base URL (no secrets)

**Extend `GET /api/v1/settings` `ai` object (secrets excluded):**

```json
{
  "provider": "ollama",
  "provider_label": "Local · Ollama",
  "base_url": "http://ollama:11434/v1",
  "model": "qwen2.5:14b",
  "is_local": true,
  "provider_configured": true
}
```

Replace or alias legacy `openai_configured: bool` → `provider_configured` (frontend migration in T-0091).

**Operator examples in `backend/config/default.toml` comments:**

- Ollama Compose full: `provider = "ollama"`, `model = "qwen2.5:14b"`
- LM Studio host: `provider = "openai_compatible"`, `base_url = "http://host.docker.internal:1234/v1"`

Update `.env.example` with optional `OLLAMA_API_KEY`, `AI_PROVIDER`, `AI_BASE_URL`.

### Done when

- [ ] Config loads new `[ai]` fields with validation
- [ ] Env overrides applied after TOML load
- [ ] Settings API returns provider status fields without secrets
- [ ] `provider_configured` false when OpenAI mode missing API key
- [ ] `provider_configured` true for ollama; requires base_url for openai_compatible
- [ ] default.toml documents ollama and LM Studio examples

---

## T-0088 — Migration 008 audit provider column

**Status:** open  
**Depends on:** US-0006 migration 006  
**Decisions:** DEC-0048, R-0042

### Description

Add SQLx migration `008_ai_audit_provider.sql`:

```sql
ALTER TABLE ai_tool_audit ADD COLUMN IF NOT EXISTS provider TEXT;
CREATE INDEX IF NOT EXISTS ai_tool_audit_provider
  ON ai_tool_audit (provider, created_at DESC);
```

Update `ai::audit::repository`:

- `insert` accepts `provider: &str` from `AiProvider::name()`
- `list` returns `provider` column for Settings audit table

Orchestrator populates `provider` on every tool audit row (wired in T-0089).

No changes to retention job, redaction rules, or tool_name CHECK.

### Done when

- [ ] Migration applies cleanly on external PostgreSQL
- [ ] Index created on `(provider, created_at DESC)`
- [ ] Audit insert includes provider string
- [ ] Audit list API exposes provider field
- [ ] Existing rows nullable provider acceptable (migration backfill not required)

---

## T-0089 — Orchestrator dyn AiProvider refactor and local fallback

**Status:** open  
**Depends on:** T-0085, T-0086, T-0088  
**Decisions:** DEC-0043, DEC-0045, DEC-0046, R-0041

### Description

Refactor `backend/src/ai/orchestrator.rs`:

- Change from concrete `OpenAiProvider` to `&dyn AiProvider` (or `Arc<dyn AiProvider>`)
- Chat handlers use `state.ai.provider()` — no per-request provider construction

**Request building changes only:**

```rust
let mut req = build_request(/* messages, tools, stream, max_tokens */);
if !provider.omit_tool_choice() {
    req.tool_choice = Some("auto".into());
}
req.temperature = Some(
    config.temperature.unwrap_or_else(|| provider.default_temperature())
);
```

**Tool rounds:** non-streaming (`stream: false`) — unchanged US-0006 semantics.

**Local fallback (DEC-0046):** when `provider.is_local()` and model returns assistant text without `tool_calls`:

1. Optional single nudge retry when `[ai] local_tool_nudge_retry = true`
2. Return assistant text to client
3. Emit SSE `event: warning` with `{ "code": "local_no_tools", "message": "..." }`
4. **Never** route to OpenAI (AC5)

**Frozen (verify no edits):** `registry.execute`, `PrivacyLayer::redact_tool_result`, tool schema generation, max 5 rounds, parallel tool execution, 8 KB cap, audit redaction.

Populate audit `provider` column on insert (T-0088).

### Done when

- [ ] Orchestrator compiles against `dyn AiProvider`
- [ ] `tool_choice` omitted for ollama and openai_compatible
- [ ] Temperature defaults: 0.7 openai / 0.3 local unless TOML override
- [ ] Local no-tool response emits SSE warning event
- [ ] Optional nudge retry fires at most once per user message
- [ ] Audit rows include provider name
- [ ] OpenAI cloud path tool loop unchanged (regression sanity)

---

## T-0090 — POST /api/v1/ai/test endpoint

**Status:** open  
**Depends on:** T-0086, T-0087, T-0089  
**Decisions:** DEC-0047, R-0042, R-0035

### Description

Add `backend/src/api/ai.rs` (or extend existing AI routes):

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/v1/ai/test` | Minimal chat completion — **no tools**, no audit row |

**Request/response:**

```json
// POST /api/v1/ai/test  (optional body)
{ "prompt": "Reply OK." }

// 200 success
{ "ok": true, "latency_ms": 842, "model": "qwen2.5:14b", "provider": "ollama", "sample": "OK" }

// 200 failure (connection/model error)
{ "ok": false, "error": "connection refused", "provider": "ollama" }
```

**Behavior:**

- JWT Bearer auth (`require_auth`) — same as chat
- Uses injected `AiProvider` from `AiService`
- Single non-streaming completion; max_tokens small (e.g. 16)
- No tool definitions; no audit insert
- Surfaces vLLM misconfiguration / model-not-found clearly in `error` string

Register route in `api/mod.rs`.

### Done when

- [ ] Endpoint reachable with JWT auth
- [ ] Success path returns latency_ms and sample text
- [ ] Failure path returns `ok: false` with actionable error (not 500 for provider down)
- [ ] No audit row created on test
- [ ] No tools sent in request body
- [ ] Works for openai, ollama, openai_compatible configs (mock or wiremock)

---

## T-0091 — Settings AI provider UI and test connection button

**Status:** open  
**Depends on:** T-0087, T-0090  
**Decisions:** DEC-0047, R-0035

### Description

Extend Settings **AI & Privacy** section (`frontend/src/pages/SettingsPage.tsx`):

| UI element | Implementation |
|------------|----------------|
| Provider table | Columns: Provider label, Model, Base URL, Status badge |
| Status badge | `Configured` / `Not configured` from `provider_configured` |
| Local indicator | Badge variant when `is_local=true` |
| Test AI provider | Button → TanStack Query mutation `POST /api/v1/ai/test` |
| Test result | Toast or inline Alert: success latency + sample / error message |
| Edit note | Retain "Edit config.toml and restart to change" |

Mirror US-0007 exchange test-connection UX (R-0035): loading state, disabled when not configured.

Migrate from `openai_configured` to `provider_configured` if needed.

Extend audit table optional **Provider** column when API returns field (T-0088).

No secrets rendered.

### Done when

- [ ] Provider table shows label, model, base_url, status
- [ ] Test button calls `/api/v1/ai/test` with Bearer JWT
- [ ] Success/failure feedback visible to operator
- [ ] Misconfigured state shows Not configured badge
- [ ] Privacy section unchanged from US-0006
- [ ] `npm run build` succeeds

---

## T-0092 — ChatPanel Local vs Cloud provider badge

**Status:** open  
**Depends on:** T-0087, T-0089  
**Decisions:** DEC-0047

### Description

Extend `frontend/src/components/chat/`:

| Component | Change |
|-----------|--------|
| `ProviderBadge` (new) | Shows `provider_label` from settings query: `Local · Ollama`, `Cloud · OpenAI`, `Local · Compatible` |
| `ChatPanel` header | Place `ProviderBadge` beside existing `PrivacyBadge` |
| Misconfigured state | When `provider_configured=false`: disable input + Alert (extend US-0006 OpenAI-missing pattern) |
| SSE warning | When `event: warning` with `local_no_tools`: subtle Alert under assistant message |

Fetch provider status via existing settings TanStack Query — no new endpoint.

Sheet drawer (`AiSheet`) and `/chat` route share `ChatPanel` — both show badge.

### Done when

- [ ] Local vs Cloud badge visible in chat header
- [ ] Badge text matches settings `provider_label`
- [ ] Input disabled + Alert when provider not configured
- [ ] SSE warning event renders user-visible notice
- [ ] Privacy badge unchanged
- [ ] Tool transparency row unchanged

---

## T-0093 — Compose full profile Ollama operator documentation

**Status:** open  
**Depends on:** T-0087  
**Decisions:** R-0038, R-0039

### Description

Document Compose `full` profile Ollama wiring — **no conditional YAML branching** (Compose cannot env-branch `depends_on`).

**Update:**

- `docker-compose.yml` comments on `ollama` service (profile `[full]`, port 11434, volume)
- `docs/engineering/runbook.md` — section "Local AI provider (US-0008)"
- `README.md` or operator quick-start snippet if existing pattern

**Content:**

```bash
docker compose --profile full up -d
docker compose --profile full exec ollama ollama pull qwen2.5:14b
```

TOML snippet for `provider = "ollama"`.

**Recommended models table (R-0038):**

| Tag | Use | VRAM approx |
|-----|-----|-------------|
| `llama3.1:8b` | Dev / fast | ~5.5 GB |
| `qwen2.5:14b` | Prod default | ~9.5 GB |
| `qwen2.5:7b` | Minimum GPU | ~5 GB |

**Host-run servers (R-0039):**

- LM Studio: `http://host.docker.internal:1234/v1` + `extra_hosts` note
- LocalAI: `http://localai:8080/v1` if sidecar added
- vLLM: document `--enable-auto-tool-choice --tool-call-parser <family>` flags

Note: operator must use `--profile full` when `provider = "ollama"`; optional manual `depends_on: ollama` snippet for startup ordering.

### Done when

- [ ] Runbook section covers full profile startup + model pull
- [ ] docker-compose.yml comments reference US-0008
- [ ] LM Studio / vLLM host URL patterns documented
- [ ] No conditional Compose YAML added (document-only per R-0038)

---

## T-0094 — Wiremock AC5 network isolation integration test

**Status:** open  
**Depends on:** T-0089, T-0090  
**Decisions:** DEC-0048, R-0042, AC5

### Description

Add integration test `backend/tests/ai_local_provider_isolation.rs` (or extend `ai_assistant_integration`):

**Setup:**

- Configure `provider = "ollama"` (or `openai_compatible`) with `base_url` pointing to wiremock/mock server
- Register wiremock guard on `https://api.openai.com` — **zero requests allowed**
- Mock local server returns valid `tool_calls` JSON for one tool round

**Assertions:**

- Orchestrator completes chat/tool path using local base URL only
- Wiremock OpenAI guard: 0 matched requests
- Audit row has `provider = "ollama"` (requires `DATABASE_URL` or in-memory audit mock)

**Skip pattern:** same as US-0006 — skip without `DATABASE_URL` if DB-dependent portions; unit-level wiremock must still run in CI.

Extend `tests/run-tests.sh` to include new test target.

**Operator UAT (document in T-0096, not CI gate):** Compose full + chat without `OPENAI_API_KEY`.

### Done when

- [ ] Wiremock asserts zero `api.openai.com` requests when local configured
- [ ] Mock local server receives chat/completions POST
- [ ] Test passes in CI without real Ollama GPU
- [ ] `bash tests/run-tests.sh` includes isolation test
- [ ] No OpenAI fallback code path triggered on local failure

---

## T-0095 — Provider factory and orchestrator unit tests

**Status:** open  
**Depends on:** T-0085, T-0086, T-0089  
**Decisions:** DEC-0043, DEC-0045, DEC-0046, AC4

### Description

Rust unit tests (no GPU required):

**Factory (`ai::provider`):**

- `build_provider` presets: openai requires key; ollama always configured; openai_compatible requires base_url
- `omit_tool_choice` true for local modes only
- `display_label` strings per kind
- Invalid `provider` enum fails

**Orchestrator (mock `AiProvider`):**

- Request builder omits `tool_choice` when `omit_tool_choice=true`
- Temperature uses provider default when config unset
- Local no-tool path emits warning (mock response without tool_calls)
- Nudge retry fires at most once when enabled

**AC4 static audit:**

- Grep/test guard: `ai/registry.rs`, `ai/privacy.rs`, `ai/tools/` file hashes or explicit allowlist — no modifications in US-0008 PR (document expected freeze)

**Regression:**

- OpenAI mode request includes `tool_choice: auto` in mock capture

### Done when

- [ ] Factory unit tests cover three modes + validation errors
- [ ] Orchestrator unit tests cover omit_tool_choice and local fallback
- [ ] Static AC4 check documents frozen modules unchanged
- [ ] All unit tests pass via `cargo test --lib`
- [ ] No new dependencies on `api.openai.com` in unit test defaults

---

## T-0096 — Operator user guide US-0008

**Status:** open  
**Depends on:** T-0091, T-0092, T-0093, T-0094, T-0095  
**Decisions:** —

### Description

Create `docs/user-guides/US-0008.md` per USER_GUIDE_MODE=1:

- Prerequisites: US-0006 operational; privacy defaults unchanged
- Provider modes overview: openai vs ollama vs openai_compatible
- **Ollama setup:** Compose `--profile full`, model pull commands, TOML snippet
- **LM Studio / LocalAI / vLLM:** base_url examples, host.docker.internal, vLLM server flags
- Settings: provider table, Test AI provider button interpretation
- Chat: Local vs Cloud badge meaning; misconfigured troubleshooting
- Local tool reliability: models recommended; graceful degradation when model skips tools
- AC5 verification: no OpenAI key needed for local path; wiremock CI note
- Audit log: new Provider column
- Troubleshooting: connection refused, model not found, vLLM 400 tool errors
- Boundary: restart required to switch provider; no in-app model pull

### Done when

- [ ] User guide covers all five acceptance criteria from operator perspective
- [ ] Ollama Compose full workflow documented step-by-step
- [ ] LM Studio / vLLM configuration examples included
- [ ] Test connection and chat badge documented
- [ ] AC5 local-only verification steps for operator UAT

---

## Execution order (recommended)

1. **Provider core:** T-0085 → T-0086
2. **Config + DB:** T-0087 ∥ T-0088 (after T-0085)
3. **Orchestrator:** T-0089 (after T-0086, T-0088)
4. **API:** T-0090
5. **Frontend:** T-0091 → T-0092
6. **Docs:** T-0093 (parallel with frontend after T-0087)
7. **Verification:** T-0095 → T-0094 → T-0096

## Acceptance coverage map

| AC | Tasks |
|----|-------|
| AC1 OpenAI + local OpenAI-compatible selector | T-0085, T-0086, T-0087, T-0089, T-0091, T-0092, T-0095, T-0096 |
| AC2 Ollama Compose full profile | T-0086, T-0087, T-0090, T-0091, T-0092, T-0093, T-0096 |
| AC3 LM Studio / LocalAI / vLLM via base_url | T-0086, T-0087, T-0090, T-0091, T-0093, T-0096 |
| AC4 Tool layer unchanged | T-0085, T-0088, T-0089, T-0095 (freeze verified) |
| AC5 Local E2E no external API | T-0089, T-0094, T-0095, T-0096 |
