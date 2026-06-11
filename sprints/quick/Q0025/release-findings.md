# Release Findings — Quick Q0025 / BUG-0017

**Quick task:** Q0025  
**Bug:** BUG-0017  
**Phase:** `/release`  
**Date:** 2026-06-10  
**Orchestrator:** `intake-20260609-ui-audit`  
**Gate status:** **PASS**

## Gate audit (US-0039)

| gate | verdict | reason_code | remediation | evidence_refs |
|------|---------|-------------|-------------|---------------|
| check-in_test | pass | — | — | `cargo test --lib` 213/213; `forecast_integration` 3/3; `npm test` 9/9 @ 2026-06-10; `sprints/quick/Q0025/qa-findings.md` |
| qa | pass | — | — | `sprints/quick/Q0025/qa-findings.md` (0 blockers) |
| uat | pass-with-prerequisites | — | — | `sprints/quick/Q0025/uat.json`, `handoffs/verify_work_to_release.md` — 5 pass, 6 pass_with_prerequisites, 0 fail |
| isolation | pass | — | — | `docs/engineering/state.md` (execute→verify-work checkpoints) |
| runtime_proof | pass | — | — | `runtime-proof-verify-work-20260610-bug0017-001`; release tuple at finalization |
| finalization | pass | — | — | `handoffs/releases/Q0025-release-notes.md`, backlog BUG-0017 DONE |
| publish | skipped | — | — | `RELEASE_PUBLISH_MODE=disabled` |

**Doc gates:** README feature coverage `skipped` (`README_FEATURE_COVERAGE_ENFORCE=0`)

## Release verdict

**PASS** — BUG-0017 finalized; acceptance AY–BD checked; operator smoke advisory documented; V1 runtime gates deferred.

## Blocking findings

None.

## Non-blocking findings

- `:18080` pre-Q0025 deploy — 0 audit rows, `plan_stale=true` — pass-with-prerequisites until **BACKEND_FRONTEND_DEPLOY**
- Omniflow browser/OIDC smoke deferred — auth barrier + deploy pending per BUG-0013/0014/0015/0016 precedent
- V1 task remains open at operator gate — 9-step smoke in `sprints/quick/Q0025/uat.json`
- BB month-bucket SQL probe deferred to operator per R-0087

## Operator smoke checklist (post-deploy)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild `flow-finance-ai` with Q0025 migrations, repository retention order, ForecastPage BD1 guard
2. **FULL_FIREFLY_SYNC** — Full sync + forecast recompute
3. **V1-SYNC** — `POST /api/v1/sync/trigger`; confirm logs free of audit CHECK WARN and FK WARN
4. **V1-META** — `GET /api/v1/forecast/meta` — fresh `computation_id`, `stale=false`
5. **V1-AUDIT** — `SELECT FROM ai_tool_audit WHERE tool_name='forecast_bucket_assignment' LIMIT 5`
6. **V1-BB** — Month-bucket SQL probe per R-0087; confirm honest `ml_skipped_reason` when gate fails
7. **V1-BC** — Planning Compare — **Plan stale** badge clears after successful recompute
8. **V1-BD** — Forecast nav from Home — loading skeleton during pending; no false empty when meta has data
9. **OIDC-1** — OIDC regression smoke per acceptance footnote

## Rerun criteria

N/A — release finalization PASS.
