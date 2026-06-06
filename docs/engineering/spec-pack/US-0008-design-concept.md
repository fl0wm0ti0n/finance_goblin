# Design Concept — US-0008

## Summary

US-0008 extends the US-0006 **AI assistant** with **local & self-hosted provider support**: provider factory behind the existing `AiProvider` trait, three modes (`openai`, `ollama`, `openai_compatible`), unified `OpenAiCompatibleProvider` HTTP client, Settings **AI & Privacy** provider display + **Test AI provider**, chat **Local vs Cloud** badge, Compose `full` profile Ollama wiring, and AC5 wiremock verification — **without changing** the six-tool layer, PrivacyLayer, or orchestrator tool semantics.

Builds on released US-0006 chat UI, settings API, orchestrator, and audit log.

## Goals

- Provider factory: `build_provider()` → `Arc<dyn AiProvider>` with enum presets (DEC-0043, R-0040)
- Three modes: OpenAI cloud, Ollama in-compose, OpenAI-compatible host endpoints (DEC-0044, R-0038, R-0039)
- Local HTTP quirks: omit `tool_choice`, temperature 0.3 default (DEC-0045, R-0038, R-0041)
- Local fallback: graceful text + SSE warning; optional single nudge; no OpenAI fallback (DEC-0046, R-0041)
- Settings: provider status fields + `POST /api/v1/ai/test` (DEC-0047, R-0042)
- Audit: migration 008 `provider` column (DEC-0048, R-0042)
- React: provider badge on ChatPanel; Settings test button (US-0007 exchange pattern, R-0035)
- Compose `full` profile: document Ollama + model pull operator flow (R-0038)
- AC5: wiremock asserts zero `api.openai.com` when local configured (DEC-0048)

## Non-goals

- Model fine-tuning; GPU orchestration beyond Compose profiles
- In-app model catalog / pull UI
- Runtime provider switching without restart
- Changes to six tools, PrivacyLayer, tool audit retention, or chat SSE core events
- Token vault / NER rehydration (DEC-0032 deferral)
- ML forecasts (US-0009)

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0043 | Unified `OpenAiCompatibleProvider` + factory | Single HTTP impl; trait-object orchestrator (R-0040) |
| DEC-0044 | Three provider modes + `base_url` | Preset URLs + labels; restart to switch (R-0038, R-0039) |
| DEC-0045 | Omit `tool_choice` for local | Ollama unsupported; safe default for mixed endpoints (R-0038) |
| DEC-0046 | Graceful local fallback | No OpenAI fallback; preserves AC5 privacy (R-0041) |
| DEC-0047 | Settings status + test endpoint | Mirrors exchange test UX; no secrets in API (R-0042, R-0035) |
| DEC-0048 | Audit provider column + wiremock AC5 | Operator traceability; CI isolation proof (R-0042) |

**UX references:** Ollama Compose full profile; LM Studio/LocalAI/vLLM OpenAI-compat; US-0007 exchange Settings test pattern; chat Local vs Cloud badge — see `docs/product/vision.md` US-0008 discovery notes.

**Spec-pack traceability:** `docs/engineering/spec-pack/US-0008-crs.md`, `docs/engineering/spec-pack/US-0008-technical-specification.md`
