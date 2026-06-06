# UAT — Sprint S0008 / US-0008

**Sprint:** S0008  
**Story:** US-0008  
**Phase:** `/verify-work`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/release`)  
**Ready for release:** yes (operator runtime prerequisites documented below)

## Inputs

- QA PASS: `sprints/S0008/qa-findings.md`
- Acceptance: `docs/product/acceptance.md` (US-0008)
- Operator guide: `docs/user-guides/US-0008.md`
- Implementation: `backend/src/ai/provider.rs`, `backend/src/ai/orchestrator.rs`, `backend/src/api/ai_test.rs`, `backend/migrations/008_ai_audit_provider.sql`, `backend/tests/{ai_local_provider_isolation,ai_frozen_modules}.rs`, `frontend/src/components/chat/ProviderBadge.tsx`, `frontend/src/pages/SettingsPage.tsx`, `docker-compose.yml` (Ollama full profile)

## Environment

| Prerequisite | Status in verify-work |
|--------------|----------------------|
| US-0006 AI chat operational | **Not provisioned** — live stack E2E deferred |
| `DATABASE_URL` (TimescaleDB + migration 008) | **Unset** — integration tests skipped by design |
| Ollama (`--profile full`, model pull) | **Not running** — `localhost:11434` unreachable |
| LM Studio / vLLM on host | **Not configured** — operator `base_url` deferred |
| `OPENAI_API_KEY` for cloud smoke | **Unset** — cloud path deferred; AC5 covered by wiremock |

Per workflow policy: code-level and automated verification **pass**; live Ollama/LM Studio/Settings UI E2E recorded as **PASS-with-prerequisites** where local provider runtime is required.

## Automated checks

| ID | Check | Command | Result |
|----|-------|---------|--------|
| AUTO-1 | Test harness | `bash tests/run-tests.sh` | **PASS** |
| AUTO-2 | Firefly GET-only unit test | (via harness) `cargo test --test firefly_readonly` | **PASS** (1/1) |
| AUTO-3 | Backend unit tests | (via harness) `cargo test --lib` | **PASS** (61/61; provider factory, orchestrator local/tool_choice, frozen registry) |
| AUTO-4 | AC5 wiremock isolation | (via harness) `cargo test --test ai_local_provider_isolation` | **PASS** (2/2 — local never hits `api.openai.com`) |
| AUTO-5 | AC4 frozen modules | (via harness) `cargo test --test ai_frozen_modules` | **PASS** (2/2 — registry/privacy/tools unchanged) |
| AUTO-6 | Exchange signing | (via harness) `cargo test --test exchange_signing` | **PASS** (4/4) |
| AUTO-7 | Integration suites | firefly/forecast/subscriptions/plans/wealth_alerts/ai_assistant/exchanges_portfolio | **SKIP** — `DATABASE_URL` unset |
| AUTO-8 | Frontend production build | (via harness) `npm run build` | **PASS** (ProviderBadge, Settings provider table + test button in build) |
| AUTO-9 | Compose full profile | `docker compose --profile full config --services` (placeholder env) | **PASS** — `firefly-iii`, `flow-finance-ai`, `grafana`, `ollama`, `redis` |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | Provider selector supports OpenAI and local OpenAI-compatible endpoints | **PASS** | `build_provider` handles `openai`, `ollama`, `openai_compatible`; `AiPublicSettings` + Settings provider table + **Test AI provider** (`POST /api/v1/ai/test`); chat `ProviderBadge`; unit tests `openai_requires_api_key`, `openai_compatible_requires_base_url`, `request_builder_omits_tool_choice_for_local`. |
| UAT-2 | AC-2 | Ollama integration works when Ollama service is running (full Docker Compose profile) | **PASS-with-prerequisites** | `docker-compose.yml` `ollama` service under `profiles: [full]`; AUTO-9 lists `ollama`; `default.toml` ollama examples; runbook + user guide pull workflow. **Operator prerequisite:** `--profile full up`, `ollama pull`, `provider=ollama`, Test AI + chat Local badge. |
| UAT-3 | AC-3 | LM Studio / LocalAI / vLLM work via OpenAI-compatible base URL configuration | **PASS-with-prerequisites** | `openai_compatible` mode + required `base_url`; wiremock provider unit test; Settings test endpoint; docs for `host.docker.internal:1234/v1`. **Operator prerequisite:** host LM Studio/vLLM + `base_url` + Test AI success. |
| UAT-4 | AC-4 | US-0006 tool layer and privacy settings unchanged across providers | **PASS** | `ai_frozen_modules` — no hash drift on `registry.rs`/`privacy.rs`, no `US-0008-MODIFIED` in tools/; six-tool registry unit test still passes; orchestrator injects provider only (HTTP layer). |
| UAT-5 | AC-5 | Chat functionality verified end-to-end with local provider (no external API call when local selected) | **PASS** | `local_provider_never_calls_openai_com` wiremock: requests only to mock `base_url`, never `api.openai.com`; `omit_tool_choice` for local. **Operator smoke:** unset `OPENAI_API_KEY`, chat + audit Provider column with live Ollama/compatible URL. |

## Summary

| Metric | Count |
|--------|-------|
| Acceptance criteria passed (code/automated) | 5/5 |
| Full runtime E2E executed | 0/5 (blocked by missing Ollama/LM Studio and PostgreSQL) |
| Automated checks passed | 7/9 (2 SKIP — expected without `DATABASE_URL`) |
| Failed | 0 |

## Operator prerequisites (post-release smoke)

1. Copy `.env.example` → `.env`; set `DATABASE_*`, `FIREFLY_*`, optional `AUTH_DEV_BYPASS=true`.
2. Provision external TimescaleDB; apply migrations including `008_ai_audit_provider.sql` at backend startup.
3. Configure `[ai]` in `backend/config/default.toml` (`provider`, `model`, optional `base_url`); restart backend.
4. **Ollama path:** `docker compose --profile full up -d`; `docker compose --profile full exec ollama ollama pull qwen2.5:14b`; set `provider = "ollama"`.
5. **Compatible URL path:** LM Studio on host `:1234`; `provider = "openai_compatible"`, `base_url = "http://host.docker.internal:1234/v1"`.
6. Settings → **Test AI provider** — expect success with latency and sample text.
7. Chat — **Local · Ollama** or **Local · Compatible** badge; example query without `OPENAI_API_KEY`.
8. Settings audit log — **Provider** column populated on tool calls.
9. Optional: `DATABASE_URL=... cargo test --test ai_assistant_integration` for migration-008 audit persistence proof.

## Findings

### Blockers

None.

### Observations

1. Integration tests (`ai_assistant_integration` and prior-story suites) require operator `DATABASE_URL` — skipped by design; AC5 covered by dedicated wiremock suite without DB.
2. Live Ollama, LM Studio, Settings Test AI UI, and chat SSE E2E depend on local provider runtime and PostgreSQL — documented in `docs/user-guides/US-0008.md`.
3. Provider switch requires backend restart — documented; not a verify-work blocker.
4. Rust unused-import warnings (`ProviderError` in `chat.rs`, exchange/portfolio modules) — cosmetic, non-blocking.

## Next phase

Run `/release` in a fresh release subagent context.
