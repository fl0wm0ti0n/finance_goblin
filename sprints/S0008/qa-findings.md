# QA Findings — Sprint S0008 / US-0008

**Sprint:** S0008  
**Story:** US-0008  
**QA phase:** `/qa`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/verify-work`)

## Scope

Local & self-hosted AI provider support: provider factory (`openai` / `ollama` / `openai_compatible`), unified `OpenAiCompatibleProvider`, extended `[ai]` config and Settings API, `POST /api/v1/ai/test`, orchestrator `dyn AiProvider` with local tool fallback, migration 008 audit `provider` column, React provider badge + Settings test button, wiremock AC5 isolation test, frozen-module guard (AC4), operator guide.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0008/summary.md`, `sprints/S0008/uat.md`, `sprints/S0008/plan-verify.json`, `docs/product/acceptance.md` (US-0008), `backend/src/ai/provider.rs`, `backend/src/ai/orchestrator.rs`, `backend/src/api/ai_test.rs`, `backend/migrations/008_ai_audit_provider.sql`, `backend/tests/{ai_local_provider_isolation,ai_frozen_modules}.rs`, `frontend/src/components/chat/ProviderBadge.tsx`, `frontend/src/pages/SettingsPage.tsx`, `docker-compose.yml`, `backend/config/default.toml`, `docs/user-guides/US-0008.md`, `docs/engineering/runbook.md`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Automated harness | `bash tests/run-tests.sh` | **PASS** (exit 0, `All tests passed`) |
| T-2 | Firefly GET-only unit test | `cargo test --test firefly_readonly` | **PASS** (1/1) |
| T-3 | Backend unit tests | `cargo test --lib` | **PASS** (61/61; provider factory, orchestrator local/tool_choice, privacy/registry unchanged) |
| T-4 | AC5 wiremock isolation | `cargo test --test ai_local_provider_isolation` | **PASS** (2/2 — local never hits `api.openai.com`; cloud includes tool_choice) |
| T-5 | AC4 frozen modules | `cargo test --test ai_frozen_modules` | **PASS** (2/2 — registry/privacy/tools no US-0008 marker) |
| T-6 | Exchange signing | `cargo test --test exchange_signing` | **PASS** (4/4) |
| T-7 | Integration suites | firefly/forecast/subscriptions/plans/wealth_alerts/ai_assistant/exchanges_portfolio | **SKIP** — `DATABASE_URL` not set (harness skips; S0001–S0007 pattern) |
| T-8 | Frontend build | `npm run build` (via harness) | **PASS** |
| T-9 | Operator guide | Static review `docs/user-guides/US-0008.md` | **PASS** — three modes, Compose full + pull, LM Studio host URL, test endpoint, audit Provider column |
| T-10 | Runtime E2E (live Ollama, LM Studio, Settings Test AI, chat badges, audit Provider column) | Not executed in QA environment | **Deferred** to `/verify-work` |

### Environment dependencies (non-blocking)

- **`DATABASE_URL`:** Required for migration 008 persistence and `ai_assistant_integration`; harness skips without DB.
- **Ollama / LM Studio / vLLM:** Live provider reachability and model pull — deferred to verify-work (`--profile full`, operator `base_url`).
- **`OPENAI_API_KEY`:** Cloud path smoke — deferred to verify-work; AC5 local path covered by wiremock.

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | Provider selector supports OpenAI and local OpenAI-compatible endpoints | **PASS** | `build_provider` handles `openai`, `ollama`, `openai_compatible`; `AiPublicSettings` + Settings provider table + **Test AI provider** (`POST /api/v1/ai/test`); chat `ProviderBadge` (`is_local` / `provider_label`); unit tests `openai_requires_api_key`, `openai_compatible_requires_base_url`, cloud vs local display labels. |
| AC-2 | Ollama integration works when Ollama service is running (full Docker Compose profile) | **PASS** (static) | `docker-compose.yml` `ollama` service under `profiles: [full]`; `default.toml` ollama examples; runbook § Local AI provider; user guide pull/`provider=ollama` workflow. Live Compose + chat deferred to verify-work. |
| AC-3 | LM Studio / LocalAI / vLLM work via OpenAI-compatible base URL configuration | **PASS** (static) | `openai_compatible` mode + required `base_url`; provider unit test against wiremock; Settings test endpoint; docs for `host.docker.internal:1234/v1`. Live LM Studio deferred to verify-work. |
| AC-4 | US-0006 tool layer and privacy settings unchanged across providers | **PASS** | `ai_frozen_modules` — no hash drift on `registry.rs`/`privacy.rs`, no `US-0008-MODIFIED` in tools/; six-tool registry unit test still passes; orchestrator injects provider only (HTTP layer). |
| AC-5 | Chat functionality verified end-to-end with local provider (no external API call when local selected) | **PASS** (automated) | `local_provider_never_calls_openai_com` wiremock: requests only to mock `base_url`, never `api.openai.com`; `omit_tool_choice` for local; operator UAT steps in `docs/user-guides/US-0008.md`. Full UI SSE chat deferred to verify-work. |

**Summary:** 5/5 PASS on static/unit/harness path; live Ollama/LM Studio/Settings UI E2E deferred to verify-work with operator env.

## Generated baseline test evidence

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` + `typescript` |
| `generated_test_command` | `bash tests/run-tests.sh` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-05-31 — exit 0, message `All tests passed` |
| `generated_test_paths_ref` | `backend/src/ai/provider.rs`, `backend/tests/ai_local_provider_isolation.rs`, `backend/tests/ai_frozen_modules.rs`, `frontend/` build |
| `generated_test_reason_code` | — |

## Runtime QA evidence

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed |
| `runtime_stack_profile` | `rust` + `node` + `ollama` (optional full profile) |
| `runtime_mode` | `local` |
| `runtime_health_target` | Deferred — Settings Test AI, chat Local/Cloud badge, Ollama full profile, audit Provider column |
| `runtime_health_result` | `deferred` |
| `runtime_final_verdict` | `deferred` (verify-work) |
| `runtime_reason_code` | `RUNTIME_E2E_DEFERRED_VERIFY_WORK` |
| `runtime_evidence_refs` | `docs/user-guides/US-0008.md`, `handoffs/dev_to_qa.md` |

## Findings

### Blockers

None.

### Observations (non-blocking)

1. **Integration tests skipped without `DATABASE_URL`:** Same harness pattern as S0001–S0007; AC5 covered by dedicated wiremock suite without DB.
2. **Live Ollama / compatible URL E2E not exercised:** Requires `--profile full`, model pull, and optional host gateway — deferred to verify-work.
3. **Provider switch requires backend restart:** Documented in summary and user guide — not a QA blocker.
4. **Rust warnings:** Unused `ProviderError` import in `chat.rs` and other pre-existing warnings — cosmetic, non-blocking.

## Decision gates

No critical issues requiring dev rework. Proceed to **`/verify-work`**.

## Next phase

Run `/verify-work` in a fresh subagent with operator PostgreSQL (`DATABASE_URL`), optional Ollama full profile or LM Studio `base_url`, and stack per `docs/user-guides/US-0008.md`.
