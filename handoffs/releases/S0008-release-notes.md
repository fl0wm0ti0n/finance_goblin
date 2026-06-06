# Sprint Release Notes — S0008

**Sprint:** S0008  
**Date:** 2026-05-31  
**Stories:** US-0008  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — `bash tests/run-tests.sh` @ 2026-05-31T23:13:47Z
2. **QA completion gate:** PASS — `sprints/S0008/qa-findings.md`, no blockers
3. **UAT completeness gate:** PASS — `sprints/S0008/uat.json`, `sprints/S0008/uat.md` (5/5 AC; live provider E2E PASS-with-prerequisites)
4. **Isolation compliance gate:** PASS — execute/qa/verify-work/release checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose --profile minimal up --build` (cloud OpenAI path); `docker compose --profile full up -d` when `provider = "ollama"`
- `runtime_mode`: local
- `runtime_context_ref`: `docs/engineering/runbook.md` (Project run steps — Local AI provider); external TimescaleDB required; migration `008_ai_audit_provider.sql` at backend startup

## Connect

- `service_url`: `http://localhost:8080` (Flow Finance AI UI/API)
- `service_port`: `8080` (`FLOW_PORT` env override)
- `health_endpoint`: `http://localhost:8080/health` (liveness); `http://localhost:8080/health/ready` (readiness + DB ping)

Additional services:

| Service | URL | Notes |
|---------|-----|-------|
| Firefly III | `http://localhost:8081` | `FIREFLY_PORT` |
| Grafana | `http://localhost:3000` | `GRAFANA_PORT` |
| Ollama (full profile) | `http://ollama:11434` (in-network) | `docker compose --profile full`; model pull operator-managed |

AI surfaces: Settings **AI & Privacy** (provider table + Test AI provider); chat header **Local · …** / **Cloud · …** badge; `POST /api/v1/ai/test`

## Verify

- `verification_steps`:
  1. Copy `.env.example` to `.env`; set `DATABASE_*`, `FIREFLY_*`; optional `AUTH_DEV_BYPASS=true` for API-only dev.
  2. Provision external PostgreSQL with TimescaleDB; confirm migration `008_ai_audit_provider.sql` applies at backend startup.
  3. Configure `[ai]` in `backend/config/default.toml` (`provider`, `model`, optional `base_url`, `temperature`, `local_tool_nudge_retry`); restart backend after any change.
  4. **Ollama:** `docker compose --profile full up -d`; `docker compose --profile full exec ollama ollama pull qwen2.5:14b`; set `provider = "ollama"`.
  5. **Compatible URL:** LM Studio on host `:1234` — `provider = "openai_compatible"`, `base_url = "http://host.docker.internal:1234/v1"` (Linux: `extra_hosts: host.docker.internal:host-gateway` on backend).
  6. Settings → **Test AI provider** — expect success with latency and sample text (no audit row).
  7. Chat — provider badge beside privacy badge; submit a question; confirm SSE stream and tool transparency when tools fire.
  8. Unset `OPENAI_API_KEY` with local provider — confirm no outbound `api.openai.com` (operator packet capture or logs).
  9. Settings audit log — **Provider** column on tool invocations after chat.
  10. Run `bash tests/run-tests.sh`; `cargo test --test ai_local_provider_isolation` and `cargo test --test ai_frozen_modules` for AC4/AC5 proof without GPU.
  11. Optional: `DATABASE_URL=... cargo test --test ai_assistant_integration` for migration-008 persistence.
- `expected_health_signal`: HTTP 200 from `/health`; Test AI returns `ok: true`; local provider chat works without OpenAI key; `ai_frozen_modules` and `ai_local_provider_isolation` tests pass

## Credentials

- `credential_source_refs` (env names only):
  - `DATABASE_PASSWORD`, `DATABASE_USER`, `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`
  - `FIREFLY_PERSONAL_ACCESS_TOKEN`, `FIREFLY_APP_KEY`, `FIREFLY_DB_PASSWORD`
  - `OPENAI_API_KEY` (cloud `provider = "openai"` only)
  - Custom `api_key_env` from TOML when set (names only in settings API)
  - `AI_PROVIDER`, `AI_BASE_URL`, `AI_MODEL` (optional env overrides)
  - `OIDC_ISSUER_URL`, `OIDC_AUDIENCE`, `VITE_OIDC_AUTHORITY`, `VITE_OIDC_CLIENT_ID`
  - `GRAFANA_ADMIN_PASSWORD`
  - `AUTH_DEV_BYPASS` (dev-only)
- `expected_value_source`: operator `.env` at repo root (from `.env.example`); see `docs/user-guides/US-0008.md`

## Known Issues

- External TimescaleDB and migration 008 are operator prerequisites; integration tests skip without `DATABASE_URL`.
- Live Ollama/LM Studio/vLLM E2E not executed in CI/agent environment — operator steps above required for full runtime proof.
- Provider switch requires backend restart — no in-app hot-swap or model pull UI.
- Local models may omit tools — graceful SSE warning and optional `local_tool_nudge_retry`; no OpenAI fallback when local selected.
- Smaller local context windows vs 8 KB tool payloads (DEC-0035) — prefer capable models (`qwen2.5:14b`, `llama3.1:8b`).
- vLLM requires explicit tool-calling flags; endpoint variance for `openai_compatible` (R-0039).

## Deliverables (US-0008)

- `AiProvider` trait + `build_provider` factory — `openai`, `ollama`, `openai_compatible`
- Unified `OpenAiCompatibleProvider`; `AiService` holds `Arc<dyn AiProvider>` at startup
- `[ai]` config: `base_url`, `temperature`, `local_tool_nudge_retry`; env `AI_*` overrides
- Migration `008_ai_audit_provider.sql` — audit `provider` column
- Orchestrator: omit `tool_choice` for local; temperature defaults; local no-tool SSE warning + optional nudge retry
- `POST /api/v1/ai/test`; `AiPublicSettings` (label, base_url, is_local, provider_configured)
- React: Settings provider table + Test AI provider; chat `ProviderBadge`; SSE `warning` handler
- Docker Compose `full` profile Ollama service; runbook + `.env.example` AI vars
- Tests: `ai_local_provider_isolation` (AC5), `ai_frozen_modules` (AC4); 61 lib tests
- Operator guide: `docs/user-guides/US-0008.md`

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0008 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.8.0-us0008`
