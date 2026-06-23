# Quick Release Notes — Q0035 / BUG-0027

**Quick task:** Q0035  
**Bug:** BUG-0027 — Firefly sync fails with 401 Unauthorized (PAT invalid/expired after deploy)  
**Date:** 2026-06-22  
**Backlog status:** READY_FOR_OPERATOR  
**Acceptance:** CC ✅ DONE, CB ⏳ PENDING_OPERATOR, CD ⏳ PENDING_OPERATOR  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --test firefly_integration` (2/2); `cargo test sync --lib` (24/24); `cargo test sync --test bug0025_sync_transaction_window` (3/3)
2. **QA completion gate:** PASS — `sprints/quick/Q0035/qa-verdict.json`, `sprints/quick/Q0035/qa-report.md` (0 blockers)
3. **UAT / verify-work gate:** PASS-WITH-PREREQUISITES — `sprints/quick/Q0035/uat.json`, `sprints/quick/Q0035/uat.md`; 3 acceptance rows — 1 pass (CC), 0 pass_with_prerequisites, 2 pending_operator (CB, CD); `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260622-bug0027-001`
6. **Release finalization gate:** PASS

---

## Summary

Firefly sync 401 Unauthorized error classification — adds structured error variant so operator can distinguish "PAT invalid/expired" from "PAT missing" or "Firefly unreachable". Backend-only change, no frontend/migration/DEC.

| Scope | Fix |
|-------|-----|
| **E1** | `backend/src/firefly/mod.rs` L37-40 — Added `FireflyError::Unauthorized` variant |
| **E2** | `backend/src/firefly/mod.rs` L156-158 — Match 401 status → `Err(FireflyError::Unauthorized)` before retry + `UnexpectedStatus` fallthrough |
| **T1** | `backend/tests/firefly_integration.rs` L155-192 — Added `test_firefly_401_returns_unauthorized_variant` (wiremock 401 mock → assert variant + error message substring) |
| **G1** | Regression gates: cargo lib, firefly integration, sync tests all PASS |
| **V1** | Operator smoke: PAT regen + deploy + ≥3 scheduled syncs (runbook written, execution pending) |

**Display message (frozen):** "firefly_personal_access_token invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN"

**Code proof:** firefly_integration 2/2 (test_firefly_401_returns_unauthorized_variant + sync_issues_only_get_requests_to_firefly); cargo lib sync 24/24; bug0025 regression 3/3.

**Operator post-release:** Deploy via `RELEASE_TAG=0.22.1-bug0027 bash /workdir/financegoblin/deploy.sh` → Regenerate PAT in Firefly UI → Update `.env` → Recreate container → Verify sync status → Monitor ≥3 scheduled syncs (see `sprints/quick/Q0035/release-verification-checklist.md`).

---

## Run

**Target service:** `flow-finance-ai` (backend only — Firefly sync error classification).

**Deploy (backend rebuild — no migration, no frontend):**

```bash
RELEASE_TAG=0.22.1-bug0027 bash /workdir/financegoblin/deploy.sh
```

Or via wrapper:

```bash
RELEASE_TAG=0.22.1-bug0027 bash /workdir/finance_goblin/scripts/deploy-omniflow.sh
```

- `start_command`: docker compose commands above
- `runtime_mode`: local (`:18080`) and remote (omniflow external US-0010)
- `runtime_context_ref`: `sprints/quick/Q0035/operator-v1-runbook.md`, `docs/engineering/runbook.md`

---

## Connect

- `service_url`: `http://localhost:18080` (local) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 18080 (local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health` → JSON 200
- Build metadata: `GET /api/v1/meta/build-info` → `release_tag: "0.22.1-bug0027"`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `FIREFLY_BASE_URL` | Firefly III API |
| `FIREFLY_PERSONAL_ACCESS_TOKEN` | Firefly PAT — regenerate post-deploy |
| `DATABASE_URL` | External PostgreSQL |
| `AUTHENTIK_SECRET_KEY` | External compose profile build gate (dummy value) |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(CC)** | Code: `firefly/mod.rs` L37-40, L156-158 | Display EXACT-match; 401 arm precedes `UnexpectedStatus` |
| **(CC)** | `cargo test --test firefly_integration` | 2/2 PASS |
| **(CB)** | PAT regen → `.env` update → recreate → sync | `state: completed`, `error_message: null`, entity_counts > 0 |
| **(CD)** | ≥3 scheduled sync_runs | All `completed`, no 401 in logs |
| Regression | `cargo test sync --lib` | 24/24 PASS |
| Regression | `cargo test sync --test bug0025_sync_transaction_window` | 3/3 PASS |

**Live (operator):** `sprints/quick/Q0035/release-verification-checklist.md`

---

## Credentials

- `FIREFLY_PERSONAL_ACCESS_TOKEN` — regenerate in Firefly UI post-deploy (no inline secrets)
- `DATABASE_URL` — external PostgreSQL (no inline secrets)

---

## Changes

| Area | Summary |
|------|---------|
| `backend/src/firefly/mod.rs` | E1 — `FireflyError::Unauthorized` variant + Display; E2 — 401 match arm |
| `backend/tests/firefly_integration.rs` | T1 — wiremock 401 test |

**Deferred:** GATE-PREFLIGHT-1 (startup health probe) to future US

---

## Known Issues

- CB/CD PENDING_OPERATOR — requires Firefly UI PAT regen
- Running container predates Q0035 — new variant absent until deploy
- ≥3h monitoring window for CD regression

---

## Rollback

```bash
cd /workdir/dev_git/finance_goblin && git revert <commit-hash>
RELEASE_TAG=0.22.0-us0022 bash /workdir/financegoblin/deploy.sh
```

Scope LOW — backend-only bugfix, no migration, no frontend.

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0035 `status=unreleased`
- `release_notes_ref`: this file
- `release_version`: `0.22.1-bug0027`
- `release_status`: READY_FOR_OPERATOR_VERIFICATION
