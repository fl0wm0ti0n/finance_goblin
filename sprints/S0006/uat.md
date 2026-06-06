# UAT — Sprint S0006 / US-0006

**Sprint:** S0006  
**Story:** US-0006  
**Phase:** `/verify-work`  
**Date:** 2026-06-01  
**Verdict:** **PASS** (ready for `/release`)  
**Ready for release:** yes (operator runtime prerequisites documented below)

## Inputs

- QA PASS: `sprints/S0006/qa-findings.md`
- Acceptance: `docs/product/acceptance.md` (US-0006)
- Operator guide: `docs/user-guides/US-0006.md`
- Implementation: `backend/migrations/006_ai_audit.sql`, `backend/src/ai/`, `backend/src/transactions/`, `backend/src/api/{chat,ai_audit}.rs`, `frontend/src/components/chat/`, `frontend/src/components/AiSheet.tsx`, `frontend/src/pages/{ChatPage,SettingsPage}.tsx`

## Environment

| Prerequisite | Status in verify-work |
|--------------|----------------------|
| `.env` with `OPENAI_API_KEY` | **Unset** — live chat streaming deferred |
| `DATABASE_URL` (TimescaleDB) | **Unset** — integration tests skipped by design |
| US-0001–US-0005 services operational | **Not provisioned** — meaningful example-query answers deferred |
| `AUTH_DEV_BYPASS` or OIDC IdP | **Unset** — live API/UI auth flow deferred |
| Synced mirror data + active plan | **Not provisioned** — `simulate_plan` / category tools deferred |

Per workflow policy: code-level and automated verification **pass**; runtime E2E steps recorded as **PASS-with-prerequisites** where external infra or API keys are required.

## Automated checks

| ID | Check | Command | Result |
|----|-------|---------|--------|
| AUTO-1 | Test harness | `bash tests/run-tests.sh` | **PASS** |
| AUTO-2 | Firefly GET-only unit test | (via harness) `cargo test --test firefly_readonly` | **PASS** (1/1) |
| AUTO-3 | Backend unit tests | (via harness) `cargo test --lib` | **PASS** (47/47; 8 AI-specific) |
| AUTO-4 | Firefly integration | (via harness) `cargo test --test firefly_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-5 | Forecast integration | (via harness) `cargo test --test forecast_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-6 | Subscription integration | (via harness) `cargo test --test subscriptions_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-7 | Plan integration | (via harness) `cargo test --test plans_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-8 | Wealth/alerts integration | (via harness) `cargo test --test wealth_alerts_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-9 | AI assistant integration (static + early-skip DB) | `cargo test --test ai_assistant_integration` | **PASS** (4/4) |
| AUTO-10 | AI tools path static audit | `ai_modules_have_no_direct_sqlx_in_tools_path` | **PASS** |
| AUTO-11 | Registry allowlist vs migration 006 | `registry_names_match_migration_allowlist` | **PASS** |
| AUTO-12 | OpenAI key from env not TOML | `config_uses_env_for_api_key_not_toml_field` | **PASS** |
| AUTO-13 | Frontend production build | (via harness) `npm run build` | **PASS** (ChatPanel, AiSheet, `/chat`, Settings audit in build) |
| AUTO-14 | Compose minimal services | `docker compose --profile minimal config --services` (placeholder env) | **PASS** — `firefly-iii`, `flow-finance-ai`, `grafana` |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | Chat UI accepts natural-language questions about finances | **PASS-with-prerequisites** | `ChatPanel` + `useChatStream` → `POST /api/v1/chat/stream`; `SuggestedPrompts` German chips; `AiSheet` header drawer + `/chat` route; graceful error when OpenAI not configured; unit test `sse_event_format`. **Operator prerequisite:** `OPENAI_API_KEY` + auth for live SSE chat. |
| UAT-2 | AC-2 | AI invokes only registered tools — no direct DB access | **PASS** | `ToolRegistry` with 6 tools (`TOOL_NAMES`); `ToolContext` has service handles only (no `DbPool`); static audit `ai_modules_have_no_direct_sqlx_in_tools_path`; orchestrator `registry.execute` only; migration 006 tool-name CHECK allowlist. |
| UAT-3 | AC-3 | OpenAI provider configurable via environment/config | **PASS** | TOML `[ai]` (`provider`, `model`, `api_key_env`, rate limits); `AiConfig::openai_configured()`; `OpenAiProvider::from_config`; Settings badge Configured/Not configured; integration test `config_uses_env_for_api_key_not_toml_field`. **Operator prerequisite:** set `OPENAI_API_KEY` (or env named in `api_key_env`) for live provider. |
| UAT-4 | AC-4 | Privacy settings honored in tool responses | **PASS** | `[privacy]` defaults in `default.toml`; `PrivacyLayer::redact_tool_result` blocks raw rows, redacts IBAN/counterparties; orchestrator applies on every tool output + `summarize_args` for audit; Settings read-only privacy table; 4 PrivacyLayer unit tests. |
| UAT-5 | AC-5 | Example queries work (affordability, subscriptions, budget, savings, top categories) | **PASS-with-prerequisites** | `SuggestedPrompts` maps five German examples; user guide query→tool table; six tool implementations with schemas; multi-round orchestrator. **Operator prerequisite:** synced data, active plan for `simulate_plan`, `OPENAI_API_KEY` for live LLM tool-chain E2E. |
| UAT-6 | AC-6 | Tool call audit log available for operator review | **PASS-with-prerequisites** | Migration `006_ai_audit.sql`; `AuditRepository::insert` per invocation; `GET /api/v1/ai/audit`; Settings paginated audit table; retention in `AiService::run_audit_retention`. **Operator prerequisite:** `DATABASE_URL` for `audit_insert_and_list_with_migration` persistence proof (static path passes without DB). |

## Summary

| Metric | Count |
|--------|-------|
| Acceptance criteria passed (code/automated) | 6/6 |
| Full runtime E2E executed | 0/6 (blocked by missing `DATABASE_URL`, `OPENAI_API_KEY`, synced data) |
| Automated checks passed | 9/14 (5 SKIP — expected without `DATABASE_URL`) |
| Failed | 0 |

## Operator prerequisites (post-release smoke)

1. Copy `.env.example` → `.env`; set `DATABASE_*`, `FIREFLY_*`, `OPENAI_API_KEY`, optional OIDC or `AUTH_DEV_BYPASS=true`.
2. Provision external TimescaleDB; apply migrations including `006_ai_audit.sql` at backend startup.
3. `docker compose --profile minimal up --build`
4. Complete Firefly setup; sync mirror data (US-0001); ensure forecast/plan/wealth data from US-0002–US-0005 for meaningful tool answers.
5. Set `OPENAI_API_KEY` (or env per `[ai].api_key_env`); confirm Settings shows OpenAI **Configured**.
6. Open header **AI** sheet or `http://localhost:8080/chat` — send suggested prompt or natural-language question; verify SSE stream and tool transparency.
7. Open **Settings → AI & Privacy** — verify privacy defaults table and paginated tool audit log after chat.
8. Optional: `DATABASE_URL=... cargo test --test ai_assistant_integration` for full audit persistence path.
9. Privacy toggles: edit TOML `[privacy]` + restart backend (runtime toggles deferred by design).

## Findings

### Blockers

None.

### Observations

1. `ai_assistant_integration` full DB path requires operator `DATABASE_URL` — harness skips file; direct run passes 4/4 (static + early-skip DB).
2. Live chat and example-query E2E require `OPENAI_API_KEY` and synced mirror — documented in `docs/user-guides/US-0006.md`.
3. Runtime privacy toggles require TOML edit + restart — by design (DEC-0032).
4. Chat stream uses completion-then-chunked tokens — acceptable per sprint summary.
5. Rust cosmetic warnings (unused imports, `AuditItem` visibility) — non-blocking.
6. ECharts main chunk ~1 MB (vite warning) — acceptable MVP carry-forward.

## Next phase

Run `/release` in a fresh release subagent context.
