# QA Report — Q0035 (BUG-0027)

**Date:** 2026-06-22  
**Phase:** qa (post-execute)  
**Orchestrator run:** auto-20260622-bug0027  
**Verdict:** **READY_FOR_VERIFY**

## Summary

BUG-0027 Firefly sync 401 Unauthorized fix reviewed. Implementation is correct and complete for the code-change scope (E1, E2, T1, G1). Two acceptance rows (CB, CD) remain operator-gated pending BACKEND_DEPLOY + PAT regen + live sync verification.

## 1. Code Review: PASS

### FireflyError::Unauthorized variant (E1)
- **Location:** `backend/src/firefly/mod.rs` L37-40
- Variant placed after `PersonalAccessTokenMissing` (L33-36) — correct ordering
- **Display message EXACT MATCH** with architecture spec:
  ```
  firefly_personal_access_token invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN
  ```

### 401 match arm (E2)
- **Location:** `backend/src/firefly/mod.rs` L156-158, inside `request()` method
- `status == StatusCode::UNAUTHORIZED` checked at L156
- Returns `Err(FireflyError::Unauthorized)` immediately (no retry)
- **Precedence correct:** Unauthorized check (L156) before server-error/429 retry block (L160) and before `UnexpectedStatus` catch-all (L166). 401 cannot leak to generic UnexpectedStatus(401).

### No regressions in surrounding code
- `PersonalAccessTokenMissing` variant and Display message unchanged (L33-36)
- `sync/mod.rs` error propagation (`e.to_string()`) inherits new variant automatically

## 2. Test Review: PASS

### Wiremock 401 test (T1)
- **File:** `backend/tests/firefly_integration.rs` L155-192
- Test name: `test_firefly_401_returns_unauthorized_variant`
- Mock path: `/api/v1/accounts` ✅ (plan-verify remediation applied — NOT `/api/v1/about`)
- Mock configures: 401 response via `ResponseTemplate::new(401)`
- Asserts: `Err(FireflyError::Unauthorized)` via pattern match
- Also asserts: error message contains `"firefly_personal_access_token invalid or expired"`

### Existing test preserved
- `sync_issues_only_get_requests_to_firefly` (L86-153) unmodified
- Note: No explicit `PersonalAccessTokenMissing` integration test exists in this file; that was not in BUG-0027 scope

## 3. Regression Gates: PASS

| Gate | Result | Detail |
|------|--------|--------|
| `cargo build --lib` | ✅ exit 0 | 15 warnings (all pre-existing, unchanged) |
| `cargo test --test firefly_integration` | ✅ 2/2 PASS | sync_issues_only_get + 401_unauthorized |
| `cargo test sync --lib` | ✅ 24/24 PASS | All sync unit tests |
| `cargo test sync --test bug0025_sync_transaction_window` | ✅ 3/3 PASS | BUG-0025 regression |

**Note:** Broad `cargo test sync` hits pre-existing compile errors in `firefly_readonly_test.rs` (missing AppConfig fields) and `exchanges_portfolio_integration.rs` (missing Clone impl). These are **NOT** in BUG-0027 blast radius — they existed before this sprint.

## 4. Acceptance Verification

| Row | Status | Verification |
|-----|--------|--------------|
| **CB** | PENDING_OPERATOR | PAT regen + .env update + container recreate + live sync. Cannot verify in QA. |
| **CC** | PASS | Display message matches architecture spec. Wiremock test confirms Unauthorized variant + error message. Manual /sync UX probe after V1 deploy will confirm end-to-end. |
| **CD** | PENDING_OPERATOR | Requires ≥3 scheduled syncs post-PAT regen. Cannot verify in QA. |

## 5. UAT Status

- **Code tests:** PASS
- **Operator tests:** PENDING (BACKEND_DEPLOY + PAT regen + live sync required)

## 6. Operator Actions Required (verify-work)

1. BACKEND_DEPLOY — rebuild with BUG-0027 changes
2. PAT regen — Firefly profile → API tokens → regenerate
3. Update `FIREFLY_PERSONAL_ACCESS_TOKEN` in `/workdir/financegoblin/.env`
4. Recreate container
5. Smoke: `GET /api/v1/sync/status` → `state: completed`
6. Smoke: manual "Sync now" → `status: completed`, entity counts > 0
7. Monitor ≥3 scheduled syncs (CD regression — no silent 401 recurrence)

## 7. Isolation Evidence

- `phase_id`: qa
- `role`: qa
- `fresh_context_marker`: qa-20260622-bug0027-qa-fresh
- `timestamp`: 2026-06-22T22:55:00Z
- `evidence_ref`: sprints/quick/Q0035/qa-verdict.json
- `inputs_read`: firefly/mod.rs, firefly_integration.rs, progress.md, acceptance.md, plan-verify-report.json, state.md, resume_brief.md
- `isolation_scope`: read-only code review + test re-run + artifact writes; no code edits; no operator action
- `dec_0038_proof`: qa phase verifies execute correctness; does NOT run verify-work or later phases; stop after qa
- `next_scheduled_phase`: verify-work
- `next_scheduled_role`: qa
