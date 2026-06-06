# Progress — Sprint S0008

**Story:** US-0008  
**Sprint status:** EXECUTE complete → QA  
**Tasks:** 12/12 complete

| Task | Status | Notes |
|------|--------|-------|
| T-0085 | done | AiProvider factory, trait extension, AiService injection |
| T-0086 | done | OpenAiCompatibleProvider unified HTTP client |
| T-0087 | done | AiConfig base_url/temperature; settings AiPublicSettings; env overrides |
| T-0088 | done | Migration 008 provider column; audit insert/list |
| T-0089 | done | Orchestrator dyn AiProvider; local nudge + SSE warning |
| T-0090 | done | POST /api/v1/ai/test |
| T-0091 | done | Settings provider table + Test AI provider |
| T-0092 | done | ChatPanel ProviderBadge; provider_configured gate |
| T-0093 | done | docker-compose + runbook US-0008 section |
| T-0094 | done | ai_local_provider_isolation wiremock test |
| T-0095 | done | Provider/orchestrator unit tests; ai_frozen_modules AC4 guard |
| T-0096 | done | docs/user-guides/US-0008.md |

**Tests:** `bash tests/run-tests.sh` — pass (lib 61, isolation + frozen, frontend build)
