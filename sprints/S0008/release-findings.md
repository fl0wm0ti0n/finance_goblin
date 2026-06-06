# Release Findings — Sprint S0008 / US-0008

**Sprint:** S0008  
**Story:** US-0008  
**Phase:** `/release`  
**Date:** 2026-05-31  
**Gate status:** PASS

## Gate audit (US-0039 / DEC-0019)

| gate | verdict | reason_code | remediation | evidence_refs |
|------|---------|-------------|-------------|---------------|
| check-in_test | pass | — | — | `bash tests/run-tests.sh` @ 2026-05-31T23:13:47Z (exit 0, All tests passed) |
| qa | pass | — | — | `sprints/S0008/qa-findings.md` (no blockers) |
| uat | pass | — | — | `sprints/S0008/uat.md`, `sprints/S0008/uat.json` (5/5 AC; 2 PASS-with-prerequisites for live Ollama/LM Studio) |
| isolation | pass | — | — | `docs/engineering/state.md` checkpoints execute/qa/verify-work; release checkpoint appended this run |
| finalization | pass | — | — | `handoffs/releases/S0008-release-notes.md`, `handoffs/release_queue.md`, backlog/acceptance reconciled |

**Optional checks:** `LINT_COMMAND` and `TYPECHECK_COMMAND` blank — reported as skipped per US-0039 AC-10.

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

## Blocking findings

None.

## Non-blocking observations

1. Live Ollama, LM Studio, Settings Test AI UI, and chat SSE E2E require operator runtime (documented in `docs/user-guides/US-0008.md` and runbook § Local AI provider).
2. Integration suites skip without `DATABASE_URL` — AC5 covered by `ai_local_provider_isolation` wiremock.
3. Provider switch requires backend restart — documented limitation.
4. `USER_GUIDE_MODE=1`: guide uses story-specific sections; canonical `# Usage steps`, `# Example`, `# Limitations` headings added at release for schema compliance.

## Backlog reconciliation (US-0043)

- Target story US-0008: backlog `DONE`, acceptance checkboxes checked from release evidence.
- `release_version`: `0.8.0-us0008`

## Next phase

None for S0008. Backlog drain may schedule US-0009 via `/discovery` or `/intake` in a fresh PO context.
