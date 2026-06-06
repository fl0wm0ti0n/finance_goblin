# PO to TL archive pack (2026-06-01-b)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 4
- Retained units in hot file: 5
- First archived heading: `## discovery-20260602-us0008 — US-0008 local AI provider UX discovery`
- Last archived heading: `## sprint-plan-20260602-s0008 — US-0008 local AI provider sprint decomposition`
- Verification tuple (mandatory):
  - archived_body_lines=240
  - retained_body_lines=337

---

## discovery-20260602-us0008 — US-0008 local AI provider UX discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-02  
**Story:** US-0008  
**Next phase:** `/research`

### Summary

Discovery captured design/UX references for local & self-hosted AI provider support (Phase 6). US-0008 extends the US-0006 **AI Tool Layer** with a provider factory behind the existing stub `AiProvider` trait — **no changes** to six tools, PrivacyLayer, orchestrator loop, or chat SSE contract. Delivers three provider modes (`openai`, `ollama`, `openai_compatible`), TOML config extension, Settings **AI & Privacy** provider display + test connection, chat **Local vs Cloud** badge, Compose `full` profile Ollama wiring, and local E2E verification (no outbound OpenAI when local selected). Builds on released US-0006 chat UI, settings API, and R-0027/R-0029 OpenAI-compatible path assumptions.

### UX references captured (see `docs/product/vision.md`)

| Reference | US-0008 application |
|-----------|---------------------|
| **Ollama** | Default Compose `full` service; OpenAI-compatible `/v1/chat/completions` + tools; CLI model pull — no in-app catalog |
| **LM Studio** | Host-run OpenAI-compatible server → `openai_compatible` + configurable `base_url` |
| **LocalAI / vLLM** | Same OpenAI-compatible contract; optional bearer when gateway requires auth |
| **US-0006 chat** | Shared `ChatPanel`; provider swap backend-only; privacy badge + new provider badge |
| **US-0007 exchange Settings** | Test-connection button pattern; read-only TOML display; status badges (R-0035) |
| **Projectplan Phase 6** | Privacy-first Q&A never leaves infrastructure when local provider active |

### Scope refinements (backlog updated)

- **Provider modes:** `openai` (existing path), `ollama` (default internal `http://ollama:11434/v1`), `openai_compatible` (LM Studio / LocalAI / vLLM via `base_url`)
- **Tool layer frozen:** Six tools + PrivacyLayer + audit log semantics unchanged — HTTP client layer only
- **Config:** Extend `[ai]` with `provider`, `base_url`, optional `api_key_env`; restart required to switch
- **Settings:** Provider table + status badge + **Test AI provider** POST endpoint; no secrets rendered
- **Chat UX:** Provider badge (`Local · Ollama` vs `Cloud · OpenAI`); disabled input + Alert when misconfigured
- **Compose:** Document `full` profile Ollama + backend dependency when `provider = "ollama"`
- **Out of scope unchanged:** model fine-tuning, GPU orchestration beyond Compose, in-app model pull UI, runtime config edit, token vault (DEC-0032), ML forecasts (US-0009)

### Discovery decomposition evidence

- Feature/workflow count: provider factory + 3 paths + config + settings test + chat badge + compose docs + E2E local verification (moderate — single story retained)
- Cross-cutting impact: provider module, config/settings API, orchestrator factory, React Settings + ChatPanel, compose operator docs
- Acceptance breadth: unchanged (5 criteria in `docs/product/acceptance.md#US-0008`)
- Risk surface: local tool-calling reliability (R-0027), endpoint variance (`tool_choice`, parallel tools, stream deltas), context window vs 8 KB payloads, SSE auth (R-0029), model-management scope creep

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#us-0008` | Discovery refinements + decomposition evidence present | pass |
| `docs/product/acceptance.md` US-0008 | 5 criteria still valid; no AC change required at discovery | pass |
| `docs/product/vision.md` | US-0008 UX refs + discovery notes appended | pass |
| US-0006 released boundary | Tool layer + privacy unchanged across providers (AC4) | pass |
| `backend/src/ai/provider.rs` | Stub trait exists; extension point confirmed | pass |

`triad_hot_surface`: check pass (rollover 4 sections → `handoffs/archive/po-to-tl-pack-20260531-h.md` then 2 sections → `handoffs/archive/po-to-tl-pack-20260531-i.md`; US-0008 discovery restored to hot surface at tail; retained_body_lines=453, pack_ref=handoffs/archive/po-to-tl-pack-20260531-i.md)

### Open questions (carry to research/architecture)

- Recommended Ollama models for reliable six-tool function calling?
- Orchestrator behavior when local model returns no `tool_calls` or malformed arguments?
- Unified `OpenAiCompatibleProvider` vs separate Ollama type?
- Optional `api_key_env` for local (dummy `ollama` key)?
- Settings API response fields for provider status?
- Compose `depends_on: ollama` conditional vs always in `full` profile?
- Audit row provider metadata for traceability?
- AC5 network isolation verification approach?

### Recommended next steps

1. `/research` — Ollama/LM Studio OpenAI-compatible tool-calling compatibility, recommended local models, streaming delta variance, config schema, compose wiring (extends R-0027, R-0029)
2. `/architecture` — Provider factory trait methods, `[ai]` config extension, settings + test endpoint, orchestrator injection, DEC-xxxx for provider enum defaults
3. `/sprint-plan` — S0008 decomposition for US-0008 acceptance criteria

---

## research-20260602-us0008 — US-0008 local AI provider technical research

**From:** Tech Lead  
**To:** Architecture (`/architecture`)  
**Date:** 2026-06-02  
**Story:** US-0008  
**Next phase:** `/architecture`

### Summary

Web research completed for US-0008 local & self-hosted AI providers. Five new entries (R-0038–R-0042) extend R-0027/R-0029 OpenAI-compatible assumptions with Ollama/LM Studio/LocalAI/vLLM compatibility, provider factory design, local tool-calling reliability, and AC5 isolation strategy.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Ollama + Compose** | [R-0038](docs/engineering/research.md#r-0038--ollama-openai-compatible-api--compose-full-profile) | OpenAI-compat at `http://ollama:11434/v1`; omit `tool_choice`; default prod model `qwen2.5:14b`; document `--profile full` + model pull; no conditional Compose `depends_on` |
| **LM Studio / LocalAI / vLLM** | [R-0039](docs/engineering/research.md#r-0039--lm-studio-localai--vllm-openai-compatible-endpoint-variance) | Single `openai_compatible` mode + configurable `base_url`; vLLM needs server tool-parser flags (operator doc); optional `api_key_env` for local |
| **Provider factory** | [R-0040](docs/engineering/research.md#r-0040--ai-provider-factory-pattern-aiprovider-http-client-swap) | Unified `OpenAiCompatibleProvider` + enum presets; extend stub `AiProvider` trait with HTTP methods; orchestrator takes `&dyn AiProvider` |
| **Local tool calling** | [R-0041](docs/engineering/research.md#r-0041--local-model-tool-calling-reliability--orchestrator-fallback) | Graceful text fallback when no `tool_calls`; one optional nudge retry; tool rounds non-streaming; temperature 0.3 local default |
| **Settings + AC5** | [R-0042](docs/engineering/research.md#r-0042--settings-provider-status-test-endpoint--ac5-network-isolation) | Settings fields `provider_configured`, `provider_label`, `base_url`, `is_local`; `POST /api/v1/ai/test`; wiremock guard for no `api.openai.com`; audit `provider` column |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Recommended Ollama models for six-tool calling? | **Dev:** `llama3.1:8b`; **prod default:** `qwen2.5:14b` (R-0038, R-0041) |
| Orchestrator fallback when no/malformed `tool_calls`? | Return assistant text + SSE warning; invalid args → tool error message to model; optional single nudge retry for local (R-0041) |
| Unified vs separate Ollama type? | **Unified `OpenAiCompatibleProvider`** with `ollama` preset URL + `omit_tool_choice` flag (R-0040) |
| Optional vs required `api_key_env` for local? | **Optional** — bearer sent only when env non-empty; dummy key pattern documented (R-0038, R-0039) |
| Settings API shape? | `provider`, `provider_label`, `base_url`, `model`, `is_local`, `provider_configured` (R-0042) |
| Compose conditional `depends_on: ollama`? | **Document only** — no env-conditional Compose YAML; operator uses `--profile full` (R-0038) |
| Audit provider metadata? | Add `provider` column migration 008; populate from `AiProvider::name()` (R-0042) |
| AC5 network isolation test? | **Wiremock** asserts zero OpenAI requests when local configured; Compose full profile for operator UAT (R-0042) |

### Risks surfaced (carry to architecture)

1. **Ollama `tool_choice` unsupported** — orchestrator must omit field for local targets (R-0038)
2. **vLLM misconfiguration** — tool calls 400 without `--enable-auto-tool-choice`; test endpoint must surface (R-0039)
3. **Local model hallucination without tools** — graceful degradation + UI warning; no OpenAI fallback (R-0041, AC5)
4. **Host-run LM Studio reachability** — `host.docker.internal:host-gateway` required from backend container (R-0039, R-0005)
5. **Orchestrator refactor scope** — trait-object provider injection touches chat handlers; keep tool/privacy layers frozen (R-0040, AC4)

### Recommended next steps

1. `/architecture` — Provider factory, `[ai]` config extension (`base_url`, `provider` enum), settings + test endpoint, orchestrator `dyn AiProvider`, migration 008 audit `provider`, DEC-xxxx for provider defaults and local fallback policy
2. `/sprint-plan` — S0008 decomposition for US-0008 acceptance criteria (5 AC)

---

## architecture-20260602-us0008 — US-0008 local AI provider architecture

**From:** Tech Lead  
**To:** Sprint planning (`/sprint-plan`)  
**Date:** 2026-06-02  
**Story:** US-0008  
**Next phase:** `/sprint-plan`

### Summary

Architecture defined for local & self-hosted AI providers over frozen US-0006 tool layer. Six decisions (DEC-0043–DEC-0048). Spec-pack 3/3. Provider swap limited to HTTP client layer — six tools, PrivacyLayer, orchestrator tool loop, and SSE contract unchanged (AC4).

### Architecture highlights

| Area | Decision |
|------|----------|
| **Provider factory** | `build_provider()` → `Arc<dyn AiProvider>`; startup wiring in `AiService` (DEC-0043, R-0040) |
| **HTTP client** | Unified `OpenAiCompatibleProvider` posting to `{base_url}/chat/completions` (DEC-0043) |
| **Provider modes** | `openai`, `ollama` (default `http://ollama:11434/v1`), `openai_compatible` (TOML `base_url`) (DEC-0044, R-0038, R-0039) |
| **Local quirks** | Omit `tool_choice` when local; temperature 0.3 default (DEC-0045, R-0038) |
| **Local fallback** | Graceful text + SSE `warning`; optional single nudge; **no OpenAI fallback** (DEC-0046, R-0041) |
| **Settings API** | `provider_label`, `is_local`, `provider_configured`, `base_url`; `POST /api/v1/ai/test` (DEC-0047, R-0042) |
| **Chat UX** | Local vs Cloud badge; disable input when misconfigured (DEC-0047) |
| **Audit** | Migration 008 `provider` column on `ai_tool_audit` (DEC-0048) |
| **AC5 isolation** | Wiremock zero `api.openai.com` when local configured (DEC-0048, R-0042) |
| **Compose full** | Document `--profile full` + `ollama pull qwen2.5:14b`; no conditional YAML (R-0038) |
| **Tool layer** | **Frozen** — registry, privacy, six tools, 8 KB cap unchanged (AC4) |

### Decisions created

- **DEC-0043** — Unified `OpenAiCompatibleProvider` + factory; trait-object orchestrator
- **DEC-0044** — Three provider modes + `[ai]` schema extension
- **DEC-0045** — Omit `tool_choice` for local; temperature defaults
- **DEC-0046** — Local tool-calling fallback; no OpenAI fallback
- **DEC-0047** — Settings provider status + test endpoint + chat badge
- **DEC-0048** — Migration 008 audit `provider`; wiremock AC5 verification

### Spec-pack status

| Artifact | Status |
|----------|--------|
| `US-0008-design-concept.md` | Complete |
| `US-0008-crs.md` | Complete |
| `US-0008-technical-specification.md` | Complete |

### Triad check (architecture phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#us-0008` | Architecture aligns with discovery scope; tool layer frozen | pass |
| `docs/product/acceptance.md` US-0008 | 5 AC mapped in architecture + spec-pack | pass |
| `docs/engineering/architecture.md` US-0008 | Provider factory, config, settings, compose, frozen tool layer | pass |
| `docs/engineering/decisions.md` | DEC-0043–DEC-0048 indexed + canonical records | pass |
| US-0006 released boundary | AC4 explicit — no registry/privacy/tool changes | pass |

`triad_hot_surface`: check pass (architecture section at po_to_tl tail; spec-pack 3/3; state checkpoint updated)

### Risks carried to sprint-plan

1. Orchestrator trait-object refactor — keep diff scoped to provider/orchestrator/chat wiring
2. Ollama `tool_choice` — omit defensively (DEC-0045)
3. Local model tool reliability — graceful fallback + operator model docs (DEC-0046)
4. vLLM/LM Studio endpoint variance — test endpoint + user guide (R-0039)
5. AC5 wiremock test must guard `api.openai.com` explicitly (DEC-0048)

### Recommended next steps

1. `/sprint-plan` — S0008 decomposition for US-0008 (5 AC)
2. `/plan-verify` — confirm task coverage against acceptance

---

## sprint-plan-20260602-s0008 — US-0008 local AI provider sprint decomposition

**From:** Tech Lead  
**To:** QA (`/plan-verify`) → Dev (`/execute`)  
**Date:** 2026-06-02  
**Story:** US-0008  
**Sprint:** S0008  
**Next phase:** `/plan-verify`

### Summary

Created sprint **S0008** with **12 tasks** (T-0085–T-0096) covering all five US-0008 acceptance criteria. No sprint split — exactly at SPRINT_MAX_TASKS=12; discovery retained single story because AC5 requires backend wiremock isolation and operator UX together.

### Task summary

| ID | Title |
|----|-------|
| T-0085 | AiProvider factory and trait extension |
| T-0086 | OpenAiCompatibleProvider HTTP client refactor |
| T-0087 | Ollama and openai_compatible config plus settings API |
| T-0088 | Migration 008 audit provider column |
| T-0089 | Orchestrator dyn AiProvider refactor and local fallback |
| T-0090 | POST /api/v1/ai/test endpoint |
| T-0091 | Settings AI provider UI and test connection button |
| T-0092 | ChatPanel Local vs Cloud provider badge |
| T-0093 | Compose full profile Ollama operator documentation |
| T-0094 | Wiremock AC5 network isolation integration test |
| T-0095 | Provider factory and orchestrator unit tests |
| T-0096 | Operator user guide US-0008 |

### Split decision

- **Why 12 tasks:** Maps PO discovery decomposition + orchestrator trait-object refactor (DEC-0043) and unit/static AC4 tests — one deployable increment per task.
- **Why not S0008a/b:** Neither "provider backend only" nor "operator UX only" independently satisfies AC5.
- **USER_GUIDE_MODE=1:** T-0096 → `docs/user-guides/US-0008.md`.

### Artifacts created

- `sprints/S0008/sprint.md`, `sprint.json`, `tasks.md`, `progress.md`
- `sprints/S0008/uat.md`, `uat.json`, `plan-verify.json`
- `handoffs/tl_to_dev.md`, `docs/engineering/state.md` checkpoint

### Recommended next steps

1. `/plan-verify` — validate AC coverage in `sprints/S0008/plan-verify.json`
2. `/execute` — implement T-0085 through T-0096 in dependency order

---
