# UAT — Q0035 (BUG-0027)

**Sprint:** Q0035 (`/quick`)
**Bug:** BUG-0027 — Firefly sync 401 Unauthorized (PAT invalid/expired)
**Status:** POPULATED (verify-work, 2026-06-22T22:58:00Z; was PLANNED placeholder pre-verify-work)
**User guide mode:** waived (operator remediation is standard Firefly PAT regen; app surfaces self-explanatory Display message)

## Target acceptance rows

- **CB** — After regenerating Firefly PAT and updating `.env` + recreating container, sync succeeds; no `401` in `error_message`
- **CC** — App surfaces clear user-facing diagnosis on `/sync` when PAT auth fails (distinguishing "PAT invalid/expired" from "PAT missing" and "Firefly unreachable")
- **CD** — ≥3 scheduled syncs succeed post-PAT regen (no silent 401 recurrence)

## Test plan

| Step | Row | Test | Expected | Actual | Result |
|------|-----|------|----------|--------|--------|
| CC-code | CC | Read `backend/src/firefly/mod.rs` L37-40 — Display message for `Unauthorized` variant | `"firefly_personal_access_token invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN"` | EXACT MATCH (verified 2026-06-22T22:58:00Z) | ✅ PASS |
| CC-arm-order | CC | Read `backend/src/firefly/mod.rs` L156-166 — 401 match arm precedes `UnexpectedStatus` fallthrough | `status == StatusCode::UNAUTHORIZED` at L156 BEFORE `UnexpectedStatus` at L166 (no shadowing) | Verified 2026-06-22T22:58:00Z | ✅ PASS |
| CC-wiremock | CC | `cargo test --test firefly_integration` — wiremock 401 → `Err(FireflyError::Unauthorized)` | 2/2 PASS | `test result: ok. 2 passed; 0 failed; 0 ignored` | ✅ PASS |
| CB-PAT-regen | CB | Operator regenerates PAT (Firefly profile → OAuth → regenerate), updates `.env`, recreates container, `POST /api/v1/sync/trigger` (manual) + `GET /api/v1/sync/status` | `state: completed`, `error_message: null`, entity_counts > 0 | — | ⏸ PENDING_OPERATOR |
| CD-scheduled-syncs | CD | Monitor `sync_runs` table ≥3h; check ≥3 `trigger=scheduled` rows with `state=completed` + `error_message IS NULL` | ≥3 clean scheduled sync_runs, no 401 in logs | — | ⏸ PENDING_OPERATOR |

## UAT counts (verify-work)

- **pass:** 1 acceptance row (CC)
- **pass_with_prerequisites:** 0
- **pending_operator:** 2 acceptance rows (CB, CD)
- **fail:** 0
- **total:** 3

## Operator gates (V1)

- `sprints/quick/Q0035/operator-v1-runbook.md` — 8-step guide for operator-side verification
- `sprints/quick/Q0035/release-plan.md` — build/deploy/rollback strategy for `0.22.1-bug0027`

## Results summary

- **CC: PASS (code-level)** — Display message frozen per architecture § BUG-0027; 401 match arm precedes catch-all; wiremock test asserts correct variant + error message
- **CB: PENDING_OPERATOR** — requires Firefly UI access for PAT regen + `.env` update + container recreate + manual sync trigger
- **CD: PENDING_OPERATOR** — requires ≥3 scheduled sync cycles (hourly cron, ~3h minimum)

## Regression verification (verify-work re-run)

| Suite | Result | Detail |
|-------|--------|--------|
| `cargo test --test firefly_integration` | ✅ 2/2 PASS | `test_firefly_401_returns_unauthorized_variant` + `sync_issues_only_get_requests_to_firefly` |

## Traceability to acceptance criteria

| AC row | Verify step | Verdict |
|--------|-------------|---------|
| CB | CB-PAT-regen (operator runbook Step 5) | ⏸ PENDING_OPERATOR |
| CC | CC-code, CC-arm-order, CC-wiremock | ✅ PASS |
| CD | CD-scheduled-syncs (operator runbook Step 6) | ⏸ PENDING_OPERATOR |

## Verify-work → release transition

- `uat.json`: status transitioned PLANNED → POPULATED (DEC-0009 lifecycle)
- CB/CD will transition POPULATED → VERIFIED in release phase after operator V1
- `/release` agent will: finalize release notes, update traceability index, mark BUG-0027 DONE/released once V1 completes

## Isolation evidence (verify-work)

- `phase_id`: verify-work
- `role`: qa
- `fresh_context_marker`: verify-work-20260622-bug0027-qa-fresh
- `timestamp`: 2026-06-22T22:58:00Z
- `evidence_ref`: sprints/quick/Q0035/uat.json, sprints/quick/Q0035/uat.md
- `inputs_read`: backend/src/firefly/mod.rs, backend/tests/firefly_integration.rs, sprint.md, qa-verdict.json, plan-verify-report.json, acceptance.md
- `isolation_scope`: artifact writes only; read-only code inspection + cargo test run; no code edits; no host secrets read
