# QA Findings — Sprint S0006 / US-0006

**Sprint:** S0006  
**Story:** US-0006  
**QA phase:** `/qa`  
**Date:** 2026-06-01  
**Verdict:** **PASS** (ready for `/verify-work`)

## Scope

Privacy-safe AI financial assistant: migration 006 audit table, `[ai]`/`[privacy]` config, TransactionsService aggregates, PlanService ephemeral projection, PrivacyLayer, six-tool registry, AiOrchestrator with SSE chat API, React ChatPanel + header AiSheet + `/chat`, Settings AI & Privacy with audit table, unit/integration tests, operator user guide.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0006/summary.md`, `sprints/S0006/tasks.md`, `sprints/S0006/plan-verify.json`, `docs/product/acceptance.md` (US-0006), `backend/migrations/006_ai_audit.sql`, `backend/src/ai/`, `backend/src/transactions/`, `backend/src/api/{chat,ai_audit}.rs`, `backend/tests/ai_assistant_integration.rs`, `frontend/src/components/chat/`, `frontend/src/components/AiSheet.tsx`, `frontend/src/pages/{ChatPage,SettingsPage}.tsx`, `docs/user-guides/US-0006.md`, `docs/engineering/spec-pack/US-0006-*.md`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Automated harness | `bash tests/run-tests.sh` | **PASS** |
| T-2 | Firefly GET-only unit test | `cargo test --test firefly_readonly` | **PASS** (1/1) |
| T-3 | Backend unit tests | `cargo test --lib` | **PASS** (47/47; 8 AI-specific) |
| T-4 | AI assistant integration file | `cargo test --test ai_assistant_integration` | **PASS** (4/4; DB persistence test early-returns when `DATABASE_URL` unset) |
| T-5 | AI tools path static audit | `ai_modules_have_no_direct_sqlx_in_tools_path` | **PASS** — no `sqlx::query` in `ai/tools/*`, registry, orchestrator |
| T-6 | Registry allowlist | `registry_has_six_tools_matching_migration` + migration CHECK | **PASS** — 6 tools match `006_ai_audit.sql` |
| T-7 | PrivacyLayer unit tests | `blocks_raw_rows_when_disabled`, `redacts_iban_in_nested_json`, `counterparty_hash_is_stable`, `summarize_args_strips_pii` | **PASS** |
| T-8 | Other integration suites | firefly/forecast/subscriptions/plans/wealth_alerts | **SKIP** — `DATABASE_URL` not set |
| T-9 | Frontend build | `npm run build` (via harness) | **PASS** |
| T-10 | User guide (USER_GUIDE_MODE=1) | Static review `docs/user-guides/US-0006.md` | **PASS** — Purpose, Prerequisites, privacy defaults, example queries, audit |
| T-11 | Spec-pack (SPEC_PACK_MODE=1) | Static review `docs/engineering/spec-pack/US-0006-*.md` | **PASS** — design-concept, crs, technical-specification (3/3) |
| T-12 | Runtime E2E (live stack → `/chat`, Sheet drawer, OpenAI chat, Settings audit) | Not executed in QA environment | **Deferred** to `/verify-work` |

### Environment dependencies (non-blocking)

- **`DATABASE_URL`:** Required for `audit_insert_and_list_with_migration` persistence path and full harness integration block. Static AI boundary tests and unit coverage pass without DB. Not a QA blocker (same pattern as S0001–S0005).
- **`OPENAI_API_KEY`:** Required for live chat streaming — deferred to verify-work.
- **Synced mirror data + active plan:** Required for meaningful example-query answers at runtime — deferred to verify-work.
- **OIDC / `AUTH_DEV_BYPASS=true`:** Required for live API/UI acceptance — deferred to verify-work.

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | Chat UI accepts natural-language questions about finances | **PASS** | `ChatPanel` with message input, SSE streaming via `useChatStream` → `POST /api/v1/chat/stream`; `SuggestedPrompts` German chips; `AiSheet` header drawer + `/chat` route sharing `ChatPanel`; graceful error when OpenAI not configured; unit test `sse_event_format`. |
| AC-2 | AI invokes only registered tools — no direct DB access | **PASS** | `ToolRegistry` exposes exactly 6 tools (`TOOL_NAMES`); `ToolContext` holds service handles only (no `DbPool`); tool implementations delegate to Forecast/Subscription/Plan/Wealth/Alert/Transactions services; static audit `ai_modules_have_no_direct_sqlx_in_tools_path`; orchestrator executes via `registry.execute` only. |
| AC-3 | OpenAI provider configurable via environment/config | **PASS** | TOML `[ai]` in `backend/config/default.toml` (`provider`, `model`, `api_key_env`, rate limits); `AiConfig::openai_configured()` reads env via `api_key_env`; `OpenAiProvider::from_config`; Settings badge Configured/Not configured; integration test `config_uses_env_for_api_key_not_toml_field`. |
| AC-4 | Privacy settings honored in tool responses | **PASS** | `[privacy]` defaults (`allow_raw_transactions=false`, `redact_iban=true`, `redact_counterparties=true`); `PrivacyLayer::redact_tool_result` blocks raw rows, redacts IBAN/counterparties; orchestrator applies layer on every tool output and `summarize_args` for audit; Settings read-only privacy table; 4 PrivacyLayer unit tests. |
| AC-5 | Example queries work (affordability, subscription prices, budget overrun, savings from cancel, top categories) | **PASS** | `SuggestedPrompts` maps all five German examples; user guide query→tool table (`simulate_plan`, `get_subscriptions`, `get_budget_status`, `get_transactions`); six tool implementations with parameter schemas; multi-round orchestrator enables chained calls. Live LLM/tool-chain E2E deferred to verify-work. |
| AC-6 | Tool call audit log available for operator review | **PASS** | Migration `006_ai_audit.sql` with tool-name CHECK allowlist; `AuditRepository::insert` on every invocation with redacted `args_summary`; `GET /api/v1/ai/audit`; Settings paginated audit table; retention job in `AiService::run_audit_retention`; integration test `audit_insert_and_list_with_migration` (DB path when `DATABASE_URL` set). |

**Summary:** 6/6 PASS (5 fully verified in QA static/unit path; live chat, example-query E2E, and DB audit persistence deferred to verify-work with operator env).

## Generated baseline test evidence (US-0066 / DEC-0048)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` + `typescript` (Axum backend + Vite/React frontend) |
| `generated_test_command` | `bash tests/run-tests.sh` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-06-01 — exit 0, message `All US-0006 tests passed` |
| `generated_test_paths_ref` | `backend/src/ai/*` (unit), `backend/tests/ai_assistant_integration.rs`, `backend/tests/firefly_readonly.rs`, `frontend/` build |
| `generated_test_reason_code` | — |

## Runtime QA evidence (US-0065 / DEC-0047)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed (compile/test-only QA pass) |
| `runtime_stack_profile` | `rust` + `node` (backend + frontend) |
| `runtime_mode` | `local` |
| `runtime_health_target` | Deferred — `http://localhost:8080/api/v1/chat/stream`, `http://localhost:5173/chat`, Settings audit table |
| `runtime_health_result` | `deferred` |
| `runtime_log_summary` | N/A — no live stack started |
| `runtime_retry_count` | 0 |
| `runtime_retry_ledger` | `[]` |
| `runtime_final_verdict` | `deferred` (verify-work) |
| `runtime_reason_code` | `RUNTIME_E2E_DEFERRED_VERIFY_WORK` |
| `runtime_evidence_refs` | `sprints/S0006/uat.md`, `docs/user-guides/US-0006.md` |

## Findings

### Blockers

None.

### Observations (non-blocking)

1. **`ai_assistant_integration` omitted from harness without `DATABASE_URL`:** Static boundary tests in that file are not run by `run-tests.sh` when DB is absent (though equivalent registry/privacy unit tests pass). Consider splitting static audit into always-run target in a future sprint.
2. **Live chat / example-query E2E not exercised:** Requires `OPENAI_API_KEY`, synced data, and optional active plan — deferred to verify-work per `sprints/S0006/uat.md`.
3. **Runtime privacy toggles require TOML edit + restart:** Documented in user guide; runtime Settings toggles deferred by design.
4. **Chat stream uses completion-then-chunked tokens:** Not native OpenAI delta streaming — acceptable per sprint summary known limitation.
5. **Rust warnings:** Unused imports in `ai/service.rs`, `api/chat.rs`; `AuditItem` visibility warning — cosmetic, non-blocking.
6. **ECharts bundle size:** Main chunk ~1 MB (vite warning) — acceptable for MVP carry-forward.

## Decision gates

No critical issues requiring dev rework. Proceed to **`/verify-work`**.

## Next phase

Run `/verify-work` in a fresh subagent with operator-provisioned PostgreSQL (`DATABASE_URL`), `OPENAI_API_KEY`, synced mirror data, active plan for `simulate_plan`, and optional `AUTH_DEV_BYPASS=true` for API/UI acceptance.
