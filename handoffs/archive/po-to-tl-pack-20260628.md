# PO to TL archive pack (2026-06-28)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=650, PO_TO_TL_HOT_MAX_SECTIONS=60`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 18
- Retained units in hot file: 60
- First archived heading: `## Closure summary`
- Last archived heading: `## discovery-20260613-bug0024 — BUG-0024 Plan delete selector regression (2nd pass)`
- Verification tuple (mandatory):
  - archived_body_lines=216
  - retained_body_lines=506

---

## Closure summary

BUG-0027 lifecycle complete. Operator PAT regeneration + deploy → sync operational.

**Operator verification:**
- Date: 2026-06-25T19:03:26Z
- Sync run ID: `run_44f0f6ca-f344-4f79-be3d-d5ce76df0b58`
- Status: `success`
- Trigger: `scheduled`
- error_message: `null`
- Container: `financegoblin-flow-finance-ai-1` (build `0.22.1-bug0027`)

**Acceptance rows:** CB ✅, CC ✅, CD ✅ — ALL DONE

**Backlog status:** DONE (was READY_FOR_OPERATOR)

**Operator feedback:** "sync now works"

**Artifacts updated:**
- `docs/product/acceptance.md` — BUG-0027 CB/CC/CD → DONE
- `docs/product/backlog.md` — BUG-0027 → DONE
- `sprints/quick/Q0035/progress.md` — V1 → DONE, closure summary appended
- `handoffs/releases/Q0035-release-notes.md` — live verification paragraph added
- `docs/engineering/state.md` — LIFECYCLE COMPLETE checkpoint appended
- `handoffs/resume_brief.md` — refreshed for post-closure (active bug: none)
- `handoffs/po_to_tl.md` — this handoff prepended

**Post-closure status:** No OPEN bugs remain. Backlog drained. Orchestrator idle.

---

# verify-work-20260622-bug0027 — Q0035 READY_FOR_RELEASE (CC PASS; CB/CD PENDING_OPERATOR)

**From:** QA (verify-work phase, fresh isolated context)  
**To:** Release (`/release`)  
**Date:** 2026-06-22  
**Work item:** BUG-0027  
**Sprint:** Q0035 (`/quick`)  
**Verdict:** **READY_FOR_RELEASE**  
**Report:** `sprints/quick/Q0035/uat.json`, `sprints/quick/Q0035/release-plan.md`

## Verify-work summary

BUG-0027 Q0035 verify-work pass — code-verifiable acceptance (CC) PASS independently verified; operator-gated acceptance (CB, CD) PENDING_OPERATOR with V1 runbook ready.

## Acceptance verification

| Row | Status | Detail |
|-----|--------|--------|
| **CC** | ✅ PASS | Display message EXACT-MATCH architecture § BUG-0027 at `firefly/mod.rs` L37-40; 401 arm at L156 precedes `UnexpectedStatus` L166 (no shadowing); integration test asserts `Err(FireflyError::Unauthorized)` + message `"firefly_personal_access_token invalid or expired"` |
| **CB** | ⏸ PENDING_OPERATOR | PAT regen + .env + container recreate + manual sync; `sprints/quick/Q0035/operator-v1-runbook.md` Steps 2-5 |
| **CD** | ⏸ PENDING_OPERATOR | ≥3 scheduled syncs post-PAT regen; `sprints/quick/Q0035/operator-v1-runbook.md` Step 6 |

## Regression gates (verify-work re-run)

| Gate | Result |
|------|--------|
| `cargo test --test firefly_integration` | ✅ 2/2 PASS (`test_firefly_401_returns_unauthorized_variant` + `sync_issues_only_get_requests_to_firefly`) |

## Release-prep

- **Version:** `0.22.1-bug0027` (patch — bugfix-only, no US)
- **Build command:** `RELEASE_TAG=0.22.1-bug0027 bash /workdir/financegoblin/deploy.sh`
- **Deploy target:** omniflow-external (https://financegnome.omniflow.cc)
- **Rollback:** git revert + `RELEASE_TAG=0.22.0-us0022 bash /workdir/financegoblin/deploy.sh`

## UAT artifacts

- `sprints/quick/Q0035/uat.json`: PLANNED → POPULATED (DEC-0009 lifecycle)
  - CC: PASS (code verified)
  - CB: PENDING_OPERATOR (V1 runbook ready)
  - CD: PENDING_OPERATOR (V1 runbook ready)
- `sprints/quick/Q0035/uat.md`: PLANNED → POPULATED

## Files created/updated by verify-work

- `sprints/quick/Q0035/release-plan.md` (new)
- `sprints/quick/Q0035/operator-v1-runbook.md` (new)
- `sprints/quick/Q0035/progress.md` (updated: V1 PENDING_OPERATOR)
- `sprints/quick/Q0035/uat.json` (populated)
- `sprints/quick/Q0035/uat.md` (populated)
- `docs/engineering/state.md` (verify-work checkpoint + isolation evidence + DEC-0038 proof)
- `handoffs/resume_brief.md` (refreshed for release phase)
- `handoffs/po_to_tl.md` (this handoff prepended)

## Next phase

**`/release`** in fresh subagent/chat (role: release). Finalize release notes, traceability index, BUG-0027 DONE/released status. Note: V1 operator smoke (CB/CD live verification) is post-release activity — same pattern as US-0022 S0021 AC-5/AC-6, BUG-0025 Q0034 V1.

## Isolation evidence

- phase_id: verify-work
- role: qa
- fresh_context_marker: verify-work-20260622-bug0027-qa-fresh
- timestamp: 2026-06-22T22:58:00Z
- evidence_ref: sprints/quick/Q0035/uat.json
- runtime_proof_id: runtime-proof-verify-work-20260622-bug0027-001

---

# plan-verify-20260622-bug0027 — Q0035 PASS (ready for /execute)

**From:** QA (plan-verify phase, fresh isolated context)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-22  
**Work item:** BUG-0027  
**Sprint:** Q0035 (`/quick`)  
**Verdict:** **PASS**  
**Report:** `sprints/quick/Q0035/plan-verify-report.json`

## Completeness: PASS
All 5 tasks (E1, E2, T1, G1, V1) present with description, file_targets, tests, status. 5/12 under SPRINT_MAX_TASKS.

## Acceptance traceability: PASS
| Row | Tasks | Coverage |
|-----|-------|----------|
| CB | V1 | PAT regen + env + container + sync succeeds |
| CC | E1+E2+T1+G1 | Unauthorized variant + 401 match + wiremock + regression |
| CD | V1+T1 | ≥3 scheduled syncs regression |

No orphan tasks.

## Gate integrity: PASS
| Gate | Status |
|------|--------|
| GATE-ERROR-1 | ✅ E1+E2 |
| GATE-MESSAGE-1 | ✅ E1 frozen Display |
| GATE-TEST-1 | ✅ T1 wiremock |
| GATE-DEC-1 | ✅ closed |
| GATE-PREFLIGHT-1 | ✅ explicitly deferred |
| GATE-302-HANDLING | ✅ closed |

## Code map accuracy: PASS (verified against actual source)
- `firefly/mod.rs` L19-37 enum — Unauthorized fits after PersonalAccessTokenMissing (L36)
- `firefly/mod.rs` L128-158 — 401 arm goes before `UnexpectedStatus` fallthrough at L158
- `tests/firefly_integration.rs` — wiremock infra ready for reuse
- `sync/mod.rs` L249-255 — `e.to_string()` propagates new variant (no change needed)

## MINOR remediation note for execute
T1 wiremock task description says mock `/api/v1/about` — **NOT in ALLOWED_PATHS** (L10-17). T1 implementation MUST use `/api/v1/accounts` (or another ALLOWED_PATHS entry) to reach the HTTP 401 flow. Otherwise hits `PathNotAllowed` before the request is made.

## Isolation evidence
- phase_id: plan-verify
- role: qa
- fresh_context_marker: plan-verify-20260622-bug0027-qa-fresh
- timestamp: 2026-06-22T22:45:00Z
- evidence_ref: sprints/quick/Q0035/plan-verify-report.json

---

# sprint-plan-20260622-bug0027 — BUG-0027 Firefly sync 401 Unauthorized (PAT invalid/expired)

**From:** Tech Lead (sprint-plan phase, fresh isolated context)  
**To:** QA (`/plan-verify`) → Dev (`/execute`)  
**Date:** 2026-06-22  
**Work item:** BUG-0027 (bug)  
**Sprint:** `/quick` **Q0035**  
**Orchestrator run:** `auto-20260622-bug0027`  
**Phase:** sprint-plan → plan-verify (next)  
**Previous phase:** architecture (same run, completed 2026-06-22T22:18:45Z)

## Summary

Sprint **Q0035** materialized from architecture § BUG-0027 in fresh isolated context: Firefly sync fails with 401 Unauthorized because PAT is present but revoked/expired. App currently surfaces generic `UnexpectedStatus(401)` — operator cannot distinguish "PAT invalid" from "Firefly unreachable" or "PAT missing".

**Five-task /quick sprint (5/12 under SPRINT_MAX_TASKS, no split):**
- **E1:** Add `FireflyError::Unauthorized` variant to `backend/src/firefly/mod.rs` L20-37 (unit variant + frozen Display message)
- **E2:** Add 401 match arm in `request()` method L128-158 to return `FireflyError::Unauthorized` instead of `UnexpectedStatus`
- **T1:** wiremock integration test mocking 401 response, asserting `Unauthorized` variant (preserving existing `PersonalAccessTokenMissing` test)
- **G1:** Regression gates (cargo check lib, firefly integration, sync tests, blast radius)
- **V1:** Operator smoke test (PAT regen, .env update, container recreate, verify sync succeeds, monitor ≥3 scheduled syncs)

**Acceptance traceability:**
- **CB** (ops): V1 — operator PAT regen + container recreate + sync succeeds
- **CC** (code): E1, E2, T1, G1 — `FireflyError::Unauthorized` variant + 401 match + wiremock test
- **CD** (ops): V1 (with T1 coverage) — monitor ≥3 scheduled syncs succeed post-regen

**Architecture gates (frozen in architecture phase):**
- **GATE-ERROR-1** ✅ — `FireflyError::Unauthorized` variant (unit variant, no fields)
- **GATE-MESSAGE-1** ✅ — Display message frozen: "firefly_personal_access_token invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN"
- **GATE-302-HANDLING** ✅ (closed) — No 302 handling needed (content negotiation ensures 401; app sends `Accept: application/json`)
- **GATE-PREFLIGHT-1** ❌ deferred — Startup health probe deferred to future US
- **GATE-TEST-1** ✅ — wiremock 401 mock → assert Unauthorized; preserve existing PersonalAccessTokenMissing test
- **GATE-DEC-1** ✅ (closed) — No new DEC (implementation detail per architecture)

**Research reference:** [R-0099 §10](docs/engineering/research.md#10-research-phase-findings-tech-lead-2026-06-22-isolated-fresh-context) (research phase findings, critical correction: Firefly returns 401 JSON not 302 redirect)

**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0027** (error taxonomy, sync error propagation path, operator remediation flow)

**Sprint artifacts:** `sprints/quick/Q0035/{sprint.md, tasks.md, task.json, progress.md}` + per-task JSON in `tasks/`

**Isolation evidence:**
- **phase_id:** sprint-plan
- **role:** tech-lead
- **fresh_context_marker:** sprint-plan-20260622-bug0027-tl-fresh
- **timestamp:** 2026-06-22T22:26:00Z
- **inputs_read:** architecture.md § BUG-0027, acceptance.md L57, scratchpad.md, phase-context.md, state.md, Q0034/Q0033 sprint format references
- **isolation_scope:** artifact writes only in /quick/Q0035 + handoffs/po_to_tl.md prepend + state.md append; no code edits; no host `.env` / secrets read

**Operator next:** After E1+E2+T1+G1+execute: BACKEND_DEPLOY; then V1 operator PAT regen + container recreate + ≥3 scheduled syncs monitor.

## Next phase

`/plan-verify` (role: qa) — validate acceptance coverage, frozen gates, task traceability; then `/execute`.

---

## discovery-20260613-bug0024 — BUG-0024 Plan delete selector regression (2nd pass)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-13  
**Work item:** BUG-0024 (bug)  
**Orchestrator run:** `auto-20260613-bug0024`  
**Phase:** discovery → architecture (next)

