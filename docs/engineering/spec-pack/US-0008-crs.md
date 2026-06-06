# CRS — US-0008

## Purpose

Canonical requirements snapshot for **US-0008 — Local & self-hosted AI provider support**. Enables privacy-first operators to run the AI financial assistant on Ollama or OpenAI-compatible local endpoints without changing the US-0006 tool layer.

See `docs/product/backlog.md#us-0008` and `docs/product/acceptance.md#US-0008`.

## Scope

**In:** Provider factory (`build_provider`), unified `OpenAiCompatibleProvider`, three provider modes (`openai`, `ollama`, `openai_compatible`), `[ai]` TOML extension (`provider`, `base_url`, optional `api_key_env`, `temperature`, `local_tool_nudge_retry`), Settings provider status + `POST /api/v1/ai/test`, chat Local vs Cloud badge, migration 008 audit `provider` column, Compose `full` profile Ollama operator docs, wiremock AC5 test, operator user guide.

**Out:** Model fine-tuning; GPU orchestration; in-app model pull UI; runtime config edit; token vault; changes to six tools, PrivacyLayer, or audit retention semantics.

## Acceptance criteria ref

`docs/product/acceptance.md` — section **US-0008**:

1. Provider selector supports OpenAI and local OpenAI-compatible endpoints
2. Ollama integration works when Ollama service is running (full Docker Compose profile)
3. LM Studio / LocalAI / vLLM work via OpenAI-compatible base URL configuration
4. US-0006 tool layer and privacy settings unchanged across providers
5. Chat functionality verified end-to-end with local provider (no external API call when local selected)

## Architecture ref

`docs/engineering/architecture.md` — section **US-0008**  
**Decisions:** DEC-0043–DEC-0048  
**Research:** R-0038–R-0042
