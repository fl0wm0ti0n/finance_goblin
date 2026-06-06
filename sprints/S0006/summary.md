# Sprint S0006 Summary — US-0006 AI Assistant

**Story:** US-0006  
**Sprint:** S0006  
**Date:** 2026-06-01

## Delivered

| Layer | Deliverable |
|-------|-------------|
| Database | `006_ai_audit.sql` — `ai_tool_audit` + indexes + startup retention |
| Config | TOML `[ai]` + `[privacy]`; settings API with `openai_configured` |
| TransactionsService | Read-only aggregates + capped raw rows |
| PlanService | `project_readonly`, `project_ephemeral`, active plan projection |
| AI core | PrivacyLayer, ToolRegistry (6 tools), OpenAiProvider, AiOrchestrator |
| API | `POST /chat/stream`, `POST /chat/completions`, `GET /ai/audit` |
| Frontend | ChatPanel, SSE client, AiSheet drawer, `/chat`, Settings AI & audit |
| Tests | 47+ unit tests; `ai_assistant_integration` (SKIP without DATABASE_URL) |
| Docs | `docs/user-guides/US-0006.md` |

## Task completion

T-0061 … T-0072 — all complete (12/12).

## Test results

```
bash tests/run-tests.sh PASS
- cargo test --lib: 47 passed
- ai_assistant_integration: SKIP (DATABASE_URL unset); static audit tests pass
- firefly_readonly: PASS
- frontend build: PASS
```

## Key decisions applied

DEC-0031 (tools-only), DEC-0032 (PrivacyLayer), DEC-0033 (SSE + JWT), DEC-0034 (audit), DEC-0035 (8 KB cap), DEC-0036 (Sheet + /chat)

## Known limitations

- Integration tests require operator `DATABASE_URL`
- Chat stream uses completion-then-chunked tokens (not native OpenAI delta streaming yet)
- Runtime privacy toggles require TOML edit + restart
- OpenAI API required for live chat; graceful error when key missing

## Release

- **Version:** `0.6.0-us0006` (2026-06-01)
- **Notes:** `handoffs/releases/S0006-release-notes.md`
- **Queue:** `handoffs/release_queue.md` — S0006 `released`

## Context refresh

- Curator `/refresh-context` 2026-06-01 — S0006 checkpoints archived to `docs/engineering/state-archive/state-pack-20260601-s0006.md`; hot state compacted for US-0007 discovery.

## Next phase

`/discovery` for US-0007 (fresh PO subagent under backlog drain).
