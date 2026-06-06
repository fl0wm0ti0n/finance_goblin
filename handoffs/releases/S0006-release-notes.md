# Sprint Release Notes — S0006

**Sprint:** S0006  
**Date:** 2026-06-01  
**Stories:** US-0006  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — `bash tests/run-tests.sh` @ 2026-06-01T14:45:00Z (release run + QA/UAT evidence)
2. **QA completion gate:** PASS — `sprints/S0006/qa-findings.md`, no blockers
3. **UAT completeness gate:** PASS — `sprints/S0006/uat.json`, `sprints/S0006/uat.md` (6/6 AC)
4. **Isolation compliance gate:** PASS — execute/qa/verify-work/release checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose --profile minimal up --build`
- `runtime_mode`: local
- `runtime_context_ref`: `docs/engineering/runbook.md` (Project run steps); external TimescaleDB required for audit persistence and tool data surfaces

## Connect

- `service_url`: `http://localhost:8080` (Flow Finance AI UI/API)
- `service_port`: `8080` (`FLOW_PORT` env override)
- `health_endpoint`: `http://localhost:8080/health` (liveness); `http://localhost:8080/health/ready` (readiness + DB ping)

Additional services (minimal profile):

| Service | URL | Port env |
|---------|-----|----------|
| Firefly III | `http://localhost:8081` | `FIREFLY_PORT` (default 8081) |
| Grafana | `http://localhost:3000` | `GRAFANA_PORT` (default 3000) |

AI surfaces: header **AI** Sheet drawer; full-page chat `http://localhost:8080/chat`; Settings **AI & Privacy** at `http://localhost:8080/settings` (after auth or `AUTH_DEV_BYPASS=true`)

## Verify

- `verification_steps`:
  1. Copy `.env.example` to `.env`; set `DATABASE_*`, `FIREFLY_*`, and `OPENAI_API_KEY` (or env name from `[ai] api_key_env` in TOML).
  2. Provision external PostgreSQL **with TimescaleDB extension**; confirm migration `backend/migrations/006_ai_audit.sql` applies at backend startup.
  3. `docker compose --profile minimal up --build`.
  4. `curl -sf http://localhost:8080/health` and `curl -sf http://localhost:8080/health/ready`.
  5. Complete Firefly sync and US-0002–US-0005 surfaces (forecast, subscriptions, active plan, wealth) so tool answers are meaningful.
  6. Confirm Settings shows **OpenAI: Configured** when `OPENAI_API_KEY` is set.
  7. Open header **AI** drawer or `/chat` — submit a natural-language question; confirm SSE stream and **Tools used** row.
  8. Trigger a tool-backed answer (e.g. suggested prompt for top categories); open Settings → **Tool audit log** and confirm redacted invocation rows.
  9. With `allow_raw_transactions=false`, confirm privacy badge and aggregate-only `get_transactions` behavior.
  10. Run `bash tests/run-tests.sh`; optional `DATABASE_URL=... cargo test --test ai_assistant_integration` for full audit persistence proof.
- `expected_health_signal`: HTTP 200 from `/health`; `POST /api/v1/chat/stream` returns SSE events when OpenAI configured; `GET /api/v1/ai/audit` lists recent tool invocations after chat; static audit confirms no `sqlx::query` in `ai/tools/*`.

## Credentials

- `credential_source_refs` (env names only):
  - `DATABASE_PASSWORD`, `DATABASE_USER`, `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`
  - `FIREFLY_PERSONAL_ACCESS_TOKEN`, `FIREFLY_APP_KEY`, `FIREFLY_DB_PASSWORD`
  - `OPENAI_API_KEY` (or value of `api_key_env` from `[ai]` — never in TOML or browser)
  - `PRIVACY_PEPPER` (optional; stable counterparty hashing)
  - `OIDC_ISSUER_URL`, `OIDC_AUDIENCE`, `VITE_OIDC_AUTHORITY`, `VITE_OIDC_CLIENT_ID`
  - `GRAFANA_ADMIN_PASSWORD`
  - `AUTH_DEV_BYPASS` (dev-only; never production)
- `expected_value_source`: operator `.env` at repo root (from `.env.example`); OpenAI key from operator secrets store; AI/privacy defaults in `backend/config/default.toml` `[ai]` and `[privacy]` — see `docs/user-guides/US-0006.md`.

## Known Issues

- External TimescaleDB is an operator prerequisite for `ai_tool_audit` persistence and integration tests.
- `ai_assistant_integration`, `wealth_alerts_integration`, and other integration suites skip without `DATABASE_URL`.
- Live chat requires `OPENAI_API_KEY`; backend returns graceful error when unset.
- Chat stream uses completion-then-chunked tokens (not native OpenAI delta streaming yet).
- Privacy and model settings require TOML edit + backend restart (no runtime toggles in MVP).
- Example-query E2E requires synced mirror data, active plan for `simulate_plan`, and OpenAI connectivity.
- OIDC live session requires IdP or `AUTH_DEV_BYPASS=true` for API/UI dev access.
- Local/self-hosted AI providers deferred to US-0008.

## Deliverables (US-0006)

- Migration `006_ai_audit.sql` — `ai_tool_audit` + retention (DEC-0034)
- PrivacyLayer — raw-transaction gate, IBAN/counterparty redaction (DEC-0032)
- ToolRegistry — six tools: `get_transactions`, `get_subscriptions`, `get_forecast`, `get_budget_status`, `get_portfolio`, `simulate_plan` (DEC-0031)
- AiOrchestrator + OpenAiProvider — multi-round tool loop, 8 KB payload cap (DEC-0035)
- API — `POST /api/v1/chat/stream`, `POST /api/v1/chat/completions`, `GET /api/v1/ai/audit` (DEC-0033 SSE + JWT)
- React — `ChatPanel`, `AiSheet` header drawer, `/chat`, Settings AI & audit (DEC-0036)
- Operator guide: `docs/user-guides/US-0006.md`
- Tests — 47+ unit tests; `ai_assistant_integration` (optional with `DATABASE_URL`)

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0006 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.6.0-us0006`
