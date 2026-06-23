# Progress — Q0035 (BUG-0027)

**Sprint status:** VERIFY-WORK COMPLETE (CC PASS; CB, CD PENDING_OPERATOR)

## Task log

| Task | Status | Notes |
|------|--------|-------|
| E1 | ✅ DONE | `FireflyError::Unauthorized` variant (dev, execute phase) |
| E2 | ✅ DONE | 401 match arm in `request()` (dev, execute phase) |
| T1 | ✅ DONE | wiremock 401 test `test_firefly_401_returns_unauthorized_variant` (dev, execute phase) |
| G1 | ✅ DONE | Regression gates: cargo lib, firefly integration, sync tests all PASS (dev, execute phase) |
| V1 | ⏸ PENDING_OPERATOR | Runbook ready (`operator-v1-runbook.md`). Needs operator PAT regen + .env update + container recreate + deploy + ≥3 scheduled syncs monitor. |

## Regression gate results (QA re-run, verify-work-20260622-bug0027-qa-fresh)

| Gate | Result | Detail |
|------|--------|--------|
| `cargo test --test firefly_integration` | ✅ 2/2 PASS | `test_firefly_401_returns_unauthorized_variant` + `sync_issues_only_get_requests_to_firefly` |
| `cargo build --lib` | ✅ exit 0 (in qa phase) | 15 warnings (all pre-existing, not in BUG-0027 blast radius) |

## Acceptance status (verify-work)

| Row | Status | Detail |
|-----|--------|--------|
| **CB** | ⏸ PENDING_OPERATOR | Runbook `operator-v1-runbook.md` Step 5. Needs PAT regen + .env + container recreate + manual Sync. |
| **CC** | ✅ PASS | Code verified independently: Display message EXACT-MATCH spec at `firefly/mod.rs` L37-40; 401 match arm at L156 precedes UnexpectedStatus fallthrough at L166; integration test asserts `Err(FireflyError::Unauthorized)` + message substring `"firefly_personal_access_token invalid or expired"` |
| **CD** | ⏸ PENDING_OPERATOR | Runbook `operator-v1-runbook.md` Step 6. Needs ≥3 scheduled syncs post-PAT regen. |

## Verify-work summary (2026-06-22T22:58:00Z)

- **Fresh context:** `verify-work-20260622-bug0027-qa-fresh`
- **Code verification (CC) PASS**: independently re-read `firefly/mod.rs` + `firefly_integration.rs` — matches architecture § BUG-0027 frozen contract exactly
- **Regression gates PASS**: 2/2 firefly_integration (the two tests are `sync_issues_only_get_requests_to_firefly` + `test_firefly_401_returns_unauthorized_variant`)
- **Release plan written**: `sprints/quick/Q0035/release-plan.md` (version `0.22.1-bug0027`, build/deploy/rollback)
- **Operator V1 runbook written**: `sprints/quick/Q0035/operator-v1-runbook.md` (8-step guide)
- **UAT artifacts populated**: `uat.json` + `uat.md` transitioned from PLANNED placeholder → POPULATED state (DEC-0009)
- **Next phase:** `/release` (release agent finalizes release notes, traceability index, closure of BUG-0027; but CB/CD closure itself is blocked on operator V1 in live environment)

## Verify-work verdict

**READY_FOR_RELEASE** — all code-verifiable acceptance PASS; operator-gated acceptance (CB, CD) documented in V1 runbook; release plan prepared for `omniflow-external` deploy.

## Notes (carry-forward)

- Sprint-plan created 2026-06-22 in fresh isolated context
- Orchestrator run: auto-20260622-bug0027
- Plan-verify PASS 2026-06-22T22:45:00Z (qa fresh context)
- Execute PASS 2026-06-22 (dev fresh context; see `handoffs/dev_to_qa.md`)
- QA PASS 2026-06-22T22:55:00Z (qa fresh context; see `sprints/quick/Q0035/qa-verdict.json`)
- Verify-work PASS (CC) + READY_FOR_RELEASE 2026-06-22T22:58:00Z (qa fresh context)

## Isolation evidence (verify-work)

- `phase_id`: verify-work
- `role`: qa
- `fresh_context_marker`: verify-work-20260622-bug0027-qa-fresh
- `timestamp`: 2026-06-22T22:58:00Z
- `evidence_ref`: sprints/quick/Q0035/progress.md, sprints/quick/Q0035/uat.json, sprints/quick/Q0035/release-plan.md, sprints/quick/Q0035/operator-v1-runbook.md
- `inputs_read`: backend/src/firefly/mod.rs, backend/tests/firefly_integration.rs, docs/engineering/release-targets.json, scripts/deploy-omniflow.sh, /workdir/financegoblin/deploy.sh, sprint.md, qa-verdict.json, plan-verify-report.json, state.md, resume_brief.md, po_to_tl.md
- `isolation_scope`: artifact writes only; read-only code inspection + cargo test run; no code edits; no .env modification; no deploy execution
