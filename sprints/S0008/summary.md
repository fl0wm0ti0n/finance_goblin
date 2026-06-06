# Sprint S0008 Summary — US-0008 local AI providers

**Status:** Released (`0.8.0-us0008`, 2026-05-31)  
**Tasks completed:** 12/12 (T-0085 … T-0096)

## Deliverables

### Provider core (T-0085, T-0086)
- Extended `AiProvider` trait with factory `build_provider`
- Unified `OpenAiCompatibleProvider` for openai / ollama / openai_compatible
- `AiService` holds `Arc<dyn AiProvider>` at startup

### Config & API (T-0087, T-0090)
- `[ai]` fields: `base_url`, `temperature`, `local_tool_nudge_retry`; env `AI_*` overrides
- Settings `AiPublicSettings` (label, base_url, is_local, provider_configured)
- `POST /api/v1/ai/test` — minimal completion, no tools/audit

### Orchestrator & audit (T-0088, T-0089)
- Migration `008_ai_audit_provider.sql`
- Omit `tool_choice` for local; temperature 0.7/0.3 defaults
- Local no-tool SSE `warning` + optional nudge retry
- Audit rows include `provider`

### Frontend (T-0091, T-0092)
- Settings provider table + Test AI provider button
- Chat `ProviderBadge`; misconfigured disables input
- SSE `warning` handler for `local_no_tools`

### Docs & tests (T-0093–T-0096)
- Runbook + Compose Ollama comments; `.env.example` AI vars
- `ai_local_provider_isolation` wiremock AC5 test
- `ai_frozen_modules` AC4 guard (registry/privacy/tools untouched)
- Operator guide `docs/user-guides/US-0008.md`

## Test results

- `cargo test --lib`: 61 passed
- `cargo test --test ai_local_provider_isolation`: 2 passed
- `cargo test --test ai_frozen_modules`: 2 passed
- `npm run build`: passed
- Integration tests skip without `DATABASE_URL`

## Frozen boundary (AC4)

No edits to `ai::registry`, `ai::privacy`, `ai::tools/*`.

## Known limitations

- Provider switch requires backend restart
- Local models may skip tools — graceful warning, no OpenAI fallback
- Ollama model pull is operator-managed (not in-app)

## Post-release refresh

- Context compacted 2026-05-31T23:14:41Z (`/refresh-context`); checkpoints archived to `docs/engineering/state-archive/state-pack-20260531-s0008.md`
- Next story: US-0009 (advanced ML forecasting & risk); backlog drain active (`AUTO_BACKLOG_DRAIN=1`)
