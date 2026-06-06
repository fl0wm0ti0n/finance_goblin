# Technical Specification — US-0008

## Overview

US-0008 refactors the AI **HTTP provider layer** only: `OpenAiCompatibleProvider`, `build_provider` factory, extended `[ai]` config, settings provider status, `POST /api/v1/ai/test`, orchestrator `&dyn AiProvider` injection, local tool-calling fallback, migration 008 audit `provider` column, React provider badge + Settings test button, and wiremock AC5 isolation test.

**Dependencies:** US-0006 AiOrchestrator, six tools, PrivacyLayer, SSE chat, audit log, Settings AI & Privacy section.

**Frozen boundary (AC4):** `ai::{registry,privacy,tools}` — no functional changes.

## Components

### Provider factory (`backend/src/ai/provider.rs`)

```
build_provider(&AiConfig) -> Result<Arc<dyn AiProvider>, ProviderError>
  ├─ openai → base https://api.openai.com/v1, api_key required
  ├─ ollama → base http://ollama:11434/v1, api_key optional, omit_tool_choice=true
  └─ openai_compatible → base_url from TOML required, omit_tool_choice=true

OpenAiCompatibleProvider::chat_completion(req)
  POST {base_url}/chat/completions
  Authorization: Bearer {api_key}  // only if api_key Some
```

Per DEC-0043 / R-0040.

### Orchestrator changes (`backend/src/ai/orchestrator.rs`)

```
AiOrchestrator::complete(provider: &dyn AiProvider, ctx, body)
  ├─ build request with tools from registry (unchanged)
  ├─ if !provider.omit_tool_choice() → tool_choice = "auto"
  ├─ temperature = provider.default_temperature()
  ├─ tool rounds: stream=false (unchanged)
  ├─ on local + text without tool_calls → SSE warning + optional nudge retry (DEC-0046)
  └─ final pass: stream=true SSE (unchanged DEC-0033)
```

Tool execution, PrivacyLayer, audit insert — unchanged except `provider` field on audit row.

### AiService startup

```
AiService::new(config, registry, ...) {
  provider = build_provider(&config.ai)?;
  orchestrator = Orchestrator::new(registry, privacy, audit, config.ai);
}
AiService::provider(&self) -> &dyn AiProvider
```

Chat handlers use `state.ai.provider()` — not `OpenAiProvider::from_config` per request.

### Config

```toml
[ai]
provider = "openai"              # openai | ollama | openai_compatible
base_url = ""
model = "gpt-4o-mini"
api_key_env = "OPENAI_API_KEY"
temperature = 0.7                # default 0.3 when is_local if unset
local_tool_nudge_retry = true
# max_tool_rounds, max_completion_tokens, max_tool_result_bytes,
# request_timeout_secs, rate_limit_per_min, audit_* unchanged
```

Validation at startup per DEC-0044.

### REST API

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/settings` | Extended `ai` object: `provider`, `provider_label`, `base_url`, `model`, `is_local`, `provider_configured` |
| POST | `/api/v1/ai/test` | Minimal completion test (JWT); no tools; no audit |
| POST | `/api/v1/chat/stream` | Unchanged path; uses injected provider |
| GET | `/api/v1/ai/audit` | Unchanged; rows include `provider` after migration 008 |

Per DEC-0047 / R-0042.

### Database migration `008_ai_audit_provider.sql`

```sql
ALTER TABLE ai_tool_audit ADD COLUMN IF NOT EXISTS provider TEXT;
CREATE INDEX IF NOT EXISTS ai_tool_audit_provider
  ON ai_tool_audit (provider, created_at DESC);
```

Per DEC-0048.

### React frontend

| Component | Change |
|-----------|--------|
| `SettingsPage` | AI & Privacy: provider table + Test AI provider button |
| `ChatPanel` | Provider badge from settings; disable when `!provider_configured` |
| `useChatStream` | Handle optional SSE `warning` event |

Privacy badge, tool transparency, suggested chips — unchanged US-0006.

### Compose operator flow

```bash
docker compose --profile full up -d
docker compose --profile full exec ollama ollama pull qwen2.5:14b
# config: provider=ollama, model=qwen2.5:14b
```

Per R-0038. Document LM Studio host URL pattern in user guide.

### Tests

| Test | Purpose |
|------|---------|
| `local_provider_no_openai_calls` | Wiremock: zero `api.openai.com` when `provider=ollama` (AC5) |
| `provider_factory_validation` | Config error paths |
| `ai_test_endpoint` | Test handler success/failure |
| `omit_tool_choice_local` | Request body lacks `tool_choice` for ollama preset |

Per DEC-0048 / R-0042.

## Non-functional

- Restart required to change provider (same as privacy/exchange secrets)
- Local E2E: no outbound OpenAI when local selected (AC5)
- Request timeout: existing `[ai] request_timeout_secs` (60s default)
- SSE auth: unchanged Bearer JWT (DEC-0006, R-0029)

## Acceptance mapping

| AC | Deliverable |
|----|-------------|
| AC1 Provider selector OpenAI + local compat | Factory + three modes + config |
| AC2 Ollama in Compose full | Document + default URL preset |
| AC3 LM Studio / LocalAI / vLLM via base_url | `openai_compatible` mode |
| AC4 Tool layer unchanged | Frozen registry/privacy/tools |
| AC5 Local E2E no external API | Wiremock CI + operator UAT guide |
