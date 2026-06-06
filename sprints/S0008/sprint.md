# Sprint S0008

**ID:** S0008  
**Story:** US-0008 — Local & self-hosted AI provider support  
**Status:** PLANNED  
**Created:** 2026-06-02

## Goal

Extend the US-0006 AI assistant with a provider factory and unified `OpenAiCompatibleProvider` so operators can run chat against OpenAI, Ollama (Compose `full` profile), or any OpenAI-compatible local endpoint (LM Studio, LocalAI, vLLM). Deliver config/schema extension, migration 008 audit `provider` column, Settings provider status + test connection, chat Local vs Cloud badge, Compose operator docs, wiremock AC5 isolation test, and operator user guide — **without changing the six-tool registry, PrivacyLayer, or tool loop semantics (AC4)**.

## Scope

- `AiProvider` trait extension + `build_provider()` factory → `Arc<dyn AiProvider>` (DEC-0043, R-0040)
- Unified `OpenAiCompatibleProvider` HTTP client posting to `{base_url}/chat/completions` (DEC-0043, DEC-0045)
- Three modes: `openai`, `ollama`, `openai_compatible` + TOML `[ai]` extension and settings API fields (DEC-0044, DEC-0047)
- Migration `008_ai_audit_provider.sql` — `provider` column on `ai_tool_audit` (DEC-0048)
- `AiOrchestrator` refactor to `&dyn AiProvider`; omit `tool_choice` for local; local fallback + SSE `warning` (DEC-0043, DEC-0045, DEC-0046)
- `POST /api/v1/ai/test` minimal completion probe (DEC-0047, R-0042)
- React Settings AI & Privacy: provider table, status badge, Test AI provider button (DEC-0047, R-0035)
- React ChatPanel: Local vs Cloud provider badge; misconfigured disable state (DEC-0047)
- Compose `full` profile Ollama operator documentation (R-0038)
- Wiremock AC5 network isolation integration test — zero `api.openai.com` when local configured (DEC-0048, R-0042)
- Provider factory + orchestrator unit tests (omit_tool_choice, local nudge, configured flags)
- Operator user guide (`docs/user-guides/US-0008.md`)

**Out of scope:** Model fine-tuning; GPU orchestration beyond Compose profiles; in-app model catalog/pull UI; runtime provider switching without restart; changes to `ai::registry`, `ai::privacy`, `ai::tools::*`; OpenAI fallback when local fails (AC5); token vault / NER rehydration (DEC-0032 deferral).

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Ollama rejects `tool_choice` | Omit when `omit_tool_choice=true` | R-0038, DEC-0045 |
| Local model skips tool calls | Graceful text + SSE warning; optional single nudge | R-0041, DEC-0046 |
| vLLM misconfiguration | Test endpoint surfaces error; user guide for server flags | R-0039 |
| LM Studio host unreachable from container | Document `host.docker.internal:host-gateway` | R-0039, R-0005 |
| Orchestrator refactor scope creep | Trait-object injection only; freeze tool/privacy modules | R-0040, AC4 |
| AC5 false confidence | Wiremock guard on `api.openai.com` in CI | R-0042, DEC-0048 |
| Context window vs 8 KB payloads | Unchanged US-0006 summarization; six tools within budget | DEC-0035, R-0041 |
| Model not pulled in Ollama | Test endpoint + user guide `ollama pull` | R-0038 |
| Settings `openai_configured` breakage | Alias or migrate to `provider_configured` in execute | DEC-0047 |

## Definition of Done

- All 12 sprint tasks complete (`T-0085` … `T-0096`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0008
- Three provider modes selectable via TOML + restart
- Ollama works with Compose `--profile full` when configured
- LM Studio / LocalAI / vLLM work via `openai_compatible` + `base_url`
- Six tools, PrivacyLayer, audit redaction unchanged (AC4 static audit)
- Wiremock proves no outbound OpenAI when local provider active (AC5)
- User guide published at `docs/user-guides/US-0008.md`
- No Firefly write operations introduced

## Architecture references

- `docs/engineering/architecture.md` — US-0008
- Decisions: DEC-0043 … DEC-0048
- Research: R-0038 … R-0042
- Depends on: US-0006 AI Tool Layer (released `0.6.0-us0006`)
