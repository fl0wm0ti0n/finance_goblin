# Release Findings — Q0035 (BUG-0027)

**Sprint:** Q0035  
**Bug:** BUG-0027 — Firefly sync fails with 401 Unauthorized (PAT invalid/expired after deploy)  
**Release version:** `0.22.1-bug0027`  
**Release date:** 2026-06-22  
**Orchestrator run:** auto-20260622-bug0027  

## Release verdict

**READY_FOR_OPERATOR_VERIFICATION** — All automated gates passed; operator V1 verification (CB/CD acceptance) pending post-deploy.

## Gate results

1. **Check-in test gate:** ✅ PASS
   - `cargo test --test firefly_integration`: 2/2 PASS (test_firefly_401_returns_unauthorized_variant + sync_issues_only_get_requests_to_firefly)
   - `cargo test sync --lib`: 24/24 PASS
   - `cargo test sync --test bug0025_sync_transaction_window`: 3/3 PASS

2. **QA completion gate:** ✅ PASS
   - No blocking findings in qa-verdict.json
   - Code review PASS (all architecture gates satisfied)
   - Test review PASS (wiremock 401 test correct)
   - Regression gates PASS

3. **UAT / verify-work gate:** ✅ PASS-WITH-PREREQUISITES
   - CC (code verification): ✅ PASS — Display message EXACT-match, 401 arm precedes UnexpectedStatus
   - CB (PAT regen + sync): ⏳ PENDING_OPERATOR — requires Firefly UI access
   - CD (≥3 scheduled syncs): ⏳ PENDING_OPERATOR — requires 3h monitoring window
   - `ready_for_release: true`

4. **Isolation compliance gate:** ✅ PASS
   - execute: `execute-20260622-bug0027-dev-fresh`
   - qa: `qa-20260622-bug0027-qa-fresh`
   - verify-work: `verify-work-20260622-bug0027-qa-fresh`

5. **Strict runtime proof gate:** ✅ PASS
   - `runtime-proof-verify-work-20260622-bug0027-001`
   - Hash: `verify-work-bug0027-20260622-qa-fresh-001`
   - Phase boundary: verify-work → release (DEC-0038 enforced)

6. **Release finalization gate:** ✅ PASS

## Release scope

**Backend-only** — no frontend, no migration, no DEC changes.

| Task | Status | Artifacts |
|------|--------|-----------|
| E1 | DONE | `FireflyError::Unauthorized` variant + Display (L37-40) |
| E2 | DONE | 401 match arm in `request()` (L156-158) |
| T1 | DONE | wiremock 401 integration test (L155-192) |
| G1 | DONE | cargo lib, firefly integration, sync tests all PASS |
| V1 | ⏳ PENDING_OPERATOR | Runbook written, execution deferred |

## Acceptance status

| Row | Status | Detail |
|-----|--------|--------|
| **CC** | ✅ DONE | Code verification complete — error taxonomy frozen per architecture § BUG-0027 |
| **CB** | ⏳ PENDING_OPERATOR | PAT regen + `.env` update + container recreate + manual sync verification |
| **CD** | ⏳ PENDING_OPERATOR | ≥3 scheduled sync_runs monitoring (3h window) |

## Operator V1 runbook

**Location:** `sprints/quick/Q0035/operator-v1-runbook.md` (8-step guide)

**Required actions:**
1. Deploy `0.22.1-bug0027` via `deploy.sh`
2. Regenerate Firefly PAT (Firefly profile → OAuth → Personal Access Tokens)
3. Update `FIREFLY_PERSONAL_ACCESS_TOKEN` in `/workdir/financegoblin/.env`
4. Recreate container: `docker compose up -d --force-recreate flow-finance-ai`
5. Trigger manual sync: `POST /api/v1/sync/trigger`
6. Verify sync status: `GET /api/v1/sync/status` → `state: completed`
7. Monitor ≥3 scheduled syncs (hourly cron, 3h minimum)
8. Confirm no 401 errors in logs

**Estimated duration:** ~15 min setup + 3h monitoring

## Deploy target

- **Version:** `0.22.1-bug0027`
- **Build command:** `RELEASE_TAG=0.22.1-bug0027 bash /workdir/financegoblin/deploy.sh`
- **Environment:** omniflow-external (financegnome.omniflow.cc)
- **Scope:** backend only (no frontend rebuild, no migration)

## Rollback strategy

Source revert + redeploy `0.22.0-us0022`:
```bash
cd /workdir/dev_git/finance_goblin && git revert <commit-hash>
RELEASE_TAG=0.22.0-us0022 bash /workdir/financegoblin/deploy.sh
```

Scope LOW — backend-only bugfix touches `firefly/mod.rs` + test; cannot destabilize other layers.

## Release artifacts

- `handoffs/releases/Q0035-release-notes.md` — per-sprint release notes
- `sprints/quick/Q0035/release-findings.md` — this file
- `sprints/quick/Q0035/release-plan.md` — build/deploy/rollback strategy
- `sprints/quick/Q0035/release-verification-checklist.md` — operator V1 checklist
- `sprints/quick/Q0035/release-report.md` — release summary for traceability
- `sprints/quick/Q0035/operator-v1-runbook.md` — 8-step operator guide

## Traceability

- **Architecture:** `docs/engineering/architecture.md` § BUG-0027 (error taxonomy)
- **Acceptance:** `docs/product/acceptance.md` BUG-0027 (CC/CB/CD)
- **Backlog:** `docs/product/backlog.md` BUG-0027 (READY_FOR_OPERATOR)
- **State:** `docs/engineering/state.md` release checkpoint + isolation evidence
- **Queue:** `handoffs/release_queue.md` Q0035 row (unreleased → awaiting operator V1)

## Next phase

**refresh-context** — refresh handoff documents after operator V1 completion (or confirm pending status if operator has not yet executed V1).

**Phase boundary (DEC-0038):** release → refresh-context (stop after release; do not proceed to refresh-context in this phase)

## Isolation evidence

- `phase_id`: release
- `role`: release
- `fresh_context_marker`: release-20260622-bug0027-release-fresh
- `timestamp`: 2026-06-22T22:58:00Z
- `evidence_ref`: handoffs/releases/Q0035-release-notes.md
- `inputs_read`: sprint.md, qa-verdict.json, qa-report.md, uat.json, uat.md, release-plan.md, operator-v1-runbook.md, acceptance.md, backlog.md, state.md
- `isolation_scope`: artifact writes only; no code edits; no deploy execution; no .env modification; no Firefly UI access
