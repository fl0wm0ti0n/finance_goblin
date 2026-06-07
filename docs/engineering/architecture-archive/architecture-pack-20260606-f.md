# Architecture archive pack (2026-06-06)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 13
- First archived heading: `## US-0008 — Local & self-hosted AI provider support`
- Last archived heading: `## US-0008 — Local & self-hosted AI provider support`
- Verification tuple (mandatory):
  - archived_body_lines=292
  - preamble_lines=10
  - retained_body_lines=2819

---

## US-0008 — Local & self-hosted AI provider support

**Status:** architecture complete (2026-06-02)  
**Research:** R-0038, R-0039, R-0040, R-0041, R-0042 (extends R-0027, R-0029, R-0030, R-0035, DEC-0031–DEC-0036)  
**Decisions:** DEC-0043, DEC-0044, DEC-0045, DEC-0046, DEC-0047, DEC-0048  
**Spec-pack:** `docs/engineering/spec-pack/US-0008-{design-concept,crs,technical-specification}.md`  
**Depends on:** US-0006 AI Tool Layer (six tools, PrivacyLayer, orchestrator loop, SSE chat, audit log) — **frozen at tool layer (AC4)**

### System context

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Browser — ChatPanel (header Sheet + /chat)                                  │
│            Settings AI & Privacy: provider table + Test AI provider button   │
│            Badges: Privacy (US-0006) + Local vs Cloud provider (US-0008)     │
└───────────────────────────────┬──────────────────────────────────────────────┘
                                │ JWT Bearer (SSE unchanged)
                                ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum)                                                       │
│                                                                               │
│  AiService (startup)                                                          │
│    build_provider(&AiConfig) ──▶ Arc<dyn AiProvider>                         │
│         │                                                                     │
│  POST /api/v1/chat/stream ──▶ AiOrchestrator (&dyn AiProvider)             │
│         │                         │                                           │
│         │                         ├─▶ AiTool registry (6 tools) — FROZEN   │
│         │                         ├─▶ PrivacyLayer — FROZEN                   │
│         │                         ├─▶ truncate 8 KB — FROZEN                  │
│         │                         └─▶ ai_tool_audit (+ provider col)         │
│         │                                                                     │
│  GET /api/v1/settings ──▶ ai.provider_label, is_local, provider_configured   │
│  POST /api/v1/ai/test ──▶ minimal chat/completions (no tools)                │
└───────────────────────────────┬──────────────────────────────────────────────┘
                                │ HTTPS to configured base_url only
                ┌───────────────┼───────────────┐
                ▼               ▼               ▼
         api.openai.com   ollama:11434/v1   host.docker.internal:1234/v1
         (provider=       (Compose full      (LM Studio /
          openai)          profile)           openai_compatible)
```

**AC4 boundary:** Provider swap is **HTTP client layer only**. No changes to tool registry, PrivacyLayer, orchestrator tool loop semantics, six tool implementations, or chat SSE event contract (except additive `warning` event per DEC-0046).

### Components

#### 1. Provider factory (`backend/src/ai/provider.rs`)

Extend stub trait into full HTTP abstraction (**DEC-0043**, **R-0040**).

| Type | Responsibility |
|------|----------------|
| `AiProviderKind` | Enum: `OpenAi`, `Ollama`, `OpenAiCompatible` parsed from TOML `provider` |
| `OpenAiCompatibleProvider` | Single reqwest implementor; `{base_url}/chat/completions`; optional bearer |
| `build_provider` | Factory: resolve URL, auth, flags; return `Arc<dyn AiProvider>` |
| `ProviderError` | Config validation, HTTP, parse errors |

**Trait contract:**

```rust
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;              // "openai" | "ollama" | "openai_compatible"
    fn is_configured(&self) -> bool;
    fn is_local(&self) -> bool;
    fn display_label(&self) -> &str;     // "Cloud · OpenAI" | "Local · Ollama" | "Local · Compatible"
    fn omit_tool_choice(&self) -> bool;
    fn default_temperature(&self) -> f32;
    async fn chat_completion(&self, req: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProviderError>;
    async fn chat_completion_stream(&self, req: ChatCompletionRequest) -> Result<reqwest::Response, ProviderError>;
}
```

**Factory preset resolution (DEC-0044):**

| Mode | `base_url` | API key | `omit_tool_choice` | `is_local` |
|------|------------|---------|-------------------|------------|
| `openai` | `https://api.openai.com/v1` | **required** | `false` | `false` |
| `ollama` | default `http://ollama:11434/v1` | optional | `true` | `true` |
| `openai_compatible` | TOML **required** | optional | `true` | `true` |

Bearer sent only when `api_key_env` resolves to non-empty string (R-0038, R-0039 dummy-key pattern for LM Studio).

**Alternative considered:** Separate `OllamaProvider` type — rejected (duplicate HTTP; R-0040).

#### 2. Orchestrator injection (minimal refactor)

`AiOrchestrator` methods change from `&OpenAiProvider` to `&dyn AiProvider` (**DEC-0043**). Chat handlers use `state.ai.provider()` built at startup — not per-request construction.

**Request building changes only:**

```rust
let mut req = ChatCompletionRequest { /* messages, tools, stream, max_tokens */ };
if !provider.omit_tool_choice() {
    req.tool_choice = Some("auto".into());
}
req.temperature = Some(provider.default_temperature());
```

**Tool rounds:** non-streaming (`stream: false`) — unchanged US-0006. Final assistant pass: streaming SSE — unchanged R-0029.

**Local fallback (DEC-0046, R-0041):** when `is_local` and response has text but no `tool_calls` → return text + SSE `warning`; optional single nudge retry when `[ai] local_tool_nudge_retry = true`. **Never** fallback to OpenAI (AC5).

**Frozen (no edits):** `registry.execute`, `PrivacyLayer::redact_tool_result`, tool schema generation, max 5 rounds, parallel tool execution, audit redaction rules.

#### 3. Config extension (`[ai]` TOML)

```toml
[ai]
provider = "openai"                    # openai | ollama | openai_compatible
base_url = ""                          # required for openai_compatible
model = "gpt-4o-mini"                  # or qwen2.5:14b
api_key_env = "OPENAI_API_KEY"         # optional for local
temperature = 0.7                      # default 0.7 openai / 0.3 local (override)
local_tool_nudge_retry = true          # local-only soft retry (DEC-0046)
# unchanged: max_tool_rounds, max_completion_tokens, max_tool_result_bytes,
#            request_timeout_secs, rate_limit_per_min, audit_retention_*
```

**Env overrides (optional):** `AI_PROVIDER`, `AI_BASE_URL`, `AI_MODEL`.

**Startup validation:** fail fast on invalid combo; log resolved provider label.

**Operator examples:**

```toml
# Ollama in Compose full profile
provider = "ollama"
model = "qwen2.5:14b"
api_key_env = "OLLAMA_API_KEY"   # optional; set OLLAMA_API_KEY=ollama if client requires bearer

# LM Studio on host
provider = "openai_compatible"
base_url = "http://host.docker.internal:1234/v1"
model = "local-model-id"
api_key_env = "LMSTUDIO_API_KEY"  # dummy value ok
```

#### 4. Settings API + test endpoint (**DEC-0047**, **R-0042**)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/settings` | Extended `ai` status object (see below) |
| POST | `/api/v1/ai/test` | Minimal completion test — no tools, no audit row |

**Settings `ai` view (secrets excluded):**

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

Replace legacy `openai_configured: bool` with `provider_configured` (execute may alias during migration).

**Test request/response:**

```json
// POST /api/v1/ai/test  (optional body)
{ "prompt": "Reply OK." }

// 200
{ "ok": true, "latency_ms": 842, "model": "qwen2.5:14b", "provider": "ollama", "sample": "OK" }

// 200 (failure)
{ "ok": false, "error": "connection refused" }
```

Mirrors US-0007 exchange test-connection UX (R-0035).

#### 5. React Settings + Chat UI

| Surface | Change |
|---------|--------|
| **Settings AI & Privacy** | Provider table: Provider, Model, Base URL, Status badge; **Test AI provider** button (TanStack Query mutation) |
| **ChatPanel header** | Provider badge: `Local · Ollama` / `Cloud · OpenAI` / `Local · Compatible` from settings query |
| **Misconfigured state** | When `provider_configured=false`: disable input + Alert (same pattern as missing OpenAI key) |
| **Privacy badge** | Unchanged US-0006 |
| **Tool transparency** | Unchanged; optional subtle warning when SSE `warning` + empty tools row |

Read-only TOML display retained — restart required to switch provider (DEC-0044).

#### 6. Docker Compose `full` profile (**R-0038**)

Existing `ollama` service on profile `[full]` — no conditional YAML branching (Compose cannot env-branch `depends_on`).

**Operator wiring (document in user guide + runbook):**

```bash
docker compose --profile full up -d
docker compose --profile full exec ollama ollama pull qwen2.5:14b
```

TOML:

```toml
[ai]
provider = "ollama"
model = "qwen2.5:14b"
```

**Document:** when `provider = "ollama"`, operator must use `--profile full`. Optional manual `depends_on: ollama` snippet for startup ordering — not injected automatically.

**Recommended models:**

| Tag | Use | VRAM (Q4 approx) |
|-----|-----|------------------|
| `llama3.1:8b` | Dev / fast iteration | ~5.5 GB |
| `qwen2.5:14b` | Prod default | ~9.5 GB |
| `qwen2.5:7b` | Minimum GPU | ~5 GB |

**Host-run servers (R-0039):** LM Studio `http://host.docker.internal:1234/v1`; LocalAI `http://localai:8080/v1`; vLLM requires `--enable-auto-tool-choice --tool-call-parser <family>` — document in user guide.

#### 7. Migration `008_ai_audit_provider.sql` (**DEC-0048**, **R-0042**)

```sql
ALTER TABLE ai_tool_audit ADD COLUMN IF NOT EXISTS provider TEXT;
CREATE INDEX IF NOT EXISTS ai_tool_audit_provider
  ON ai_tool_audit (provider, created_at DESC);
```

Populate from `AiProvider::name()` on each tool audit insert. Enables operator filter by provider in Settings audit table.

#### 8. AC5 network isolation verification (**DEC-0048**, **R-0042**)

**CI (required):** wiremock guard on `https://api.openai.com` — zero matches when `provider=ollama` with mocked local base URL. Orchestrator integration test with mocked `tool_calls` response.

**Operator UAT:** Compose full profile + example US-0006 query; optional tcpdump — not CI gate.

### Backend module layout

| Module | Change |
|--------|--------|
| `ai::provider` | Trait extension, `OpenAiCompatibleProvider`, `build_provider` factory |
| `ai::orchestrator` | `&dyn AiProvider`; omit `tool_choice`; local fallback; temperature |
| `ai::{registry,privacy,tools}` | **No changes** |
| `ai::service` | Hold `Arc<dyn AiProvider>` at startup |
| `api::chat` | Use injected provider |
| `api::ai` (new) | `POST /api/v1/ai/test` |
| `api::mod` | Extend settings AI view |
| `config` | `AiConfig` + validation |
| `migrations/008_*` | Audit `provider` column |

### Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Ollama `tool_choice` unsupported | Omit field when `omit_tool_choice=true` | R-0038, DEC-0045 |
| vLLM misconfiguration | Test endpoint + user guide for server flags | R-0039 |
| Local model skips tools | Graceful text + SSE warning + optional nudge | R-0041, DEC-0046 |
| LM Studio host unreachable | `host.docker.internal:host-gateway` docs | R-0039, R-0005 |
| Orchestrator refactor scope | Trait-object only; freeze tool/privacy modules | R-0040, AC4 |
| Hallucinated numbers without tools | System prompt + tool transparency + badge | R-0041 |
| AC5 false confidence | Wiremock OpenAI guard in CI | R-0042, DEC-0048 |
| Context window vs 8 KB payloads | Unchanged summarization; six tools within budget | DEC-0035, R-0041 |
| Model not pulled | Test endpoint surfaces connection error | R-0038 |
| Scope creep (model pull UI) | Explicitly out of backlog | discovery |

### Decisions (US-0008)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0043 | Provider factory | Unified `OpenAiCompatibleProvider` + `build_provider`; trait-object orchestrator |
| DEC-0044 | Provider modes | `openai \| ollama \| openai_compatible` + `base_url`; restart to switch |
| DEC-0045 | Local HTTP quirks | Omit `tool_choice` for local; temperature 0.3 default |
| DEC-0046 | Local tool fallback | Graceful text + SSE warning; optional single nudge; no OpenAI fallback |
| DEC-0047 | Settings + test API | Provider status fields; `POST /api/v1/ai/test`; chat badge UX |
| DEC-0048 | Audit + AC5 | Migration 008 `provider` column; wiremock isolation test |

Full records: `decisions/DEC-0043.md` … `decisions/DEC-0048.md`

### Out of scope (US-0008)

- Model fine-tuning; GPU orchestration beyond Compose profiles
- In-app model catalog / pull UI
- Runtime provider switching without restart
- User message pre-redaction (DEC-0032 deferral)
- Token vault / NER rehydration (DEC-0032)
- Changes to six tools, PrivacyLayer defaults, or audit retention semantics
- ML forecasts (US-0009)
- Any write to Firefly III

### Next phase

`/sprint-plan` — S0008 task decomposition against 5 acceptance criteria.

---

