# Release Report — Q0035 (BUG-0027)

**Orchestrator run:** `auto-20260622-bug0027`  
**Sprint:** Q0035 (`/quick`)  
**Bug:** BUG-0027 — Firefly sync fails with 401 Unauthorized (PAT invalid/expired after deploy)  
**Release date:** 2026-06-22  
**Release version:** `0.22.1-bug0027`  
**Verdict:** READY_FOR_OPERATOR_VERIFICATION

---

## Release summary

BUG-0027 adds structured error classification for Firefly PAT authentication failures (401 Unauthorized → clear operator-facing message). Backend-only change; no frontend, migration, or DEC modifications. All automated gates passed; operator V1 verification (CB/CD acceptance) documented and pending post-deploy execution.

---

## Build target

| Field | Value |
|-------|-------|
| **Version** | `0.22.1-bug0027` |
| **Build command** | `RELEASE_TAG=0.22.1-bug0027 bash /workdir/financegoblin/deploy.sh` |
| **Deploy target** | omniflow-external (`https://financegnome.omniflow.cc`) |
| **Container** | `financegoblin-flow-finance-ai-1` |
| **Profile** | `external` |
| **Rollback** | `git revert <commit-hash>` + redeploy `0.22.0-us0022` |

---

## Code changes (3 tasks)

| Task | Status | Files |
|------|--------|-------|
| **E1** — `FireflyError::Unauthorized` variant + Display | ✅ DONE | `backend/src/firefly/mod.rs` L37-40 |
| **E2** — Match 401 → `Unauthorized` in `request()` | ✅ DONE | `backend/src/firefly/mod.rs` L156-158 |
| **T1** — Wiremock 401 integration test | ✅ DONE | `backend/tests/firefly_integration.rs` L155-192 |
| **G1** — Regression gates | ✅ DONE | cargo lib, firefly integration, sync tests |
| **V1** — Operator smoke (PAT regen + ≥3 syncs) | ⏳ PENDING_OPERATOR | runbook written; execution pending |

**Display message (frozen):** "firefly_personal_access_token invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN"

---

## QA verification

| Artifact | Result |
|----------|--------|
| `sprints/quick/Q0035/qa-verdict.json` | READY_FOR_VERIFY |
| `sprints/quick/Q0035/qa-report.md` | PASS (0 blockers) |
| Code review | PASS (all architecture gates satisfied) |
| Test review | PASS (wiremock test correct) |
| Regression gates | PASS (firefly 2/2, sync lib 24/24, bug0025 3/3) |

---

## Acceptance status

| Row | Status | Detail |
|-----|--------|--------|
| **CC** | ✅ DONE | Code verified — Display message EXACT-match, 401 arm precedes UnexpectedStatus, integration test asserts Unauthorized variant |
| **CB** | ⏳ PENDING_OPERATOR | Requires PAT regen + `.env` update + container recreate + manual sync. Documented in `sprints/quick/Q0035/release-verification-checklist.md` Step 5 |
| **CD** | ⏳ PENDING_OPERATOR | Requires ≥3 scheduled sync Runs monitoring (3h window). Documented in `release-verification-checklist.md` Step 6 |

---

## Release artifacts (all created)

| Artifact | Path |
|----------|------|
| Sprint release notes | `handoffs/releases/Q0035-release-notes.md` |
| Release findings | `sprints/quick/Q0035/release-findings.md` |
| Release verification checklist | `sprints/quick/Q0035/release-verification-checklist.md` (operator V1) |
| Release report | this file (`sprints/quick/Q0035/release-report.md`) |
| Release plan | `sprints/quick/Q0035/release-plan.md` |
| Operator V1 runbook | `sprints/quick/Q0035/operator-v1-runbook.md` |

---

## Operator actions (documented, NOT executed)

1. Deploy `0.22.1-bug0027` via `deploy.sh`
2. Regenerate PAT in Firefly web UI (Profile → OAuth → Personal Access Tokens)
3. Update `FIREFLY_PERSONAL_ACCESS_TOKEN` in `/workdir/financegoblin/.env`
4. Recreate container: `docker compose up -d --force-recreate flow-finance-ai`
5. Verify sync status: `GET /api/v1/sync/status` → expected: `state: completed`
6. Monitor ≥3 scheduled syncs (hourly) — confirm no 401 errors in logs
7. Mark CB ✅ DONE
8. Mark CD ✅ DONE
9. Update backlog: BUG-0027 status → TODO: DONE

---

## Backlog status

| Field | Value |
|-------|-------|
| BUG-0027 status (this release) | `READY_FOR_OPERATOR` |
| Rationale | CB/CD live verification pending; operator must execute V1 smoke AFTER deploy to confirm end-to-end behavior. Cannot mark DONE without operator confirmation. |
| After V1 completion | Update to `DONE` in refresh-context phase |

---

## Isolation evidence (release phase)

| Field | Value |
|-------|-------|
| `phase_id` | release |
| `role` | release |
| `fresh_context_marker` | release-20260622-bug0027-release-fresh |
| `timestamp` | 2026-06-22T22:58:00Z |
| `evidence_ref` | `sprints/quick/Q0035/release-report.md`, `handoffs/releases/Q0035-release-notes.md` |
| `isolation_scope` | artifact writes only; no code edits; no deploy execution; no `.env` modification; no PAT regeneration |

---

## DEC-0038 strict runtime proof

| Field | Value |
|-------|-------|
| `orchestrator_run_id` | auto-20260622-bug0027 |
| `runtime_proof_id` | runtime-proof-release-20260622-bug0027-001 |
| `phase_id` | release |
| `role` | release |
| `active_bug_id` | BUG-0027 |
| `sprint_id` | Q0035 |
| `proof_hash` | release-bug0027-20260622-release-fresh-001 |
| `proof_basis` | BUG-0027 release READY_FOR_OPERATOR_VERIFICATION — all automated gates PASS; CB/CD documented as PENDING_OPERATOR; release notes written; checklist written; backlog status READY_FOR_OPERATOR; DEC-0038 phase boundary enforced (release only; no refresh-context) |

---

## Phase boundary

| Field | Value |
|-------|-------|
| `next_scheduled_phase` | refresh-context |
| `next_scheduled_role` | curator |
| `stop_reason` | completed (phase boundary; DEC-0038 isolation; V1 PENDING_OPERATOR; hand off to /refresh-context in new subagent/chat) |

---

## Files created/updated

- **CREATED:** `handoffs/releases/Q0035-release-notes.md`
- **CREATED:** `sprints/quick/Q0035/release-findings.md`
- **CREATED:** `sprints/quick/Q0035/release-verification-checklist.md`
- **CREATED:** `sprints/quick/Q0035/release-report.md`
- **UPDATED:** `handoffs/release_queue.md` — Q0035 row added (status=unreleased)
- **UPDATED:** `handoffs/release_notes.md` — latest pointer + operator summary
- **UPDATED:** `docs/product/backlog.md` — BUG-0027 status → READY_FOR_OPERATOR
- **UPDATED:** `docs/product/acceptance.md` — BUG-0027 CC ✅ DONE, CB/CD ⏳ PENDING_OPERATOR
- **APPENDED:** `docs/engineering/state.md` — release checkpoint + isolation evidence + proof tuple
- **REFRESHED:** `handoffs/resume_brief.md` — refresh-context carry-forward
- **PREPENDED:** `handoffs/po_to_tl.md` — release handoff section
- **DELETED:** `handoffs/archive/state-append-release-BUG-0027-20260622.md` (erroneous, replaced by state.md append)

---

## Known limitations

- CB/CD closure requires **operator action AFTER release deploy** — cannot be executed by release agent
- PAT regeneration requires Firefly III web UI access (not possible in release context)
- `.env` modification requires shell access to deploy host (not possible in release context)
- Monitor ≥3 syncs requires ~3h wall-clock time after deploy
- **V1 PENDING_OPERATOR is the norm** — same pattern as US-0022, BUG-0025 (pass-with-prerequisites)

---

## Return to orchestrator

**Release verdict:** READY_FOR_OPERATOR_VERIFICATION  
**Status:** release artifacts complete; backlog updated; acceptance updated; state evidence appended; isolation enforced; phase boundary respected  
**Next phase:** refresh-context (curator; V1 PENDING_OPERATOR; handoff documents prepared)  
**Blocker:** operator must execute V1 smoke AFTER deploy to mark CB/CD DONE and BUG-0027 DONE
