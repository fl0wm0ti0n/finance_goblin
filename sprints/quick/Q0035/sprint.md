# Q0035 — BUG-0027 Firefly sync 401 Unauthorized (PAT invalid/expired)

| Field | Value |
|-------|-------|
| **ID** | Q0035 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0027 |
| **Created** | 2026-06-22 |
| **Architecture** | `docs/engineering/architecture.md` § BUG-0027 (error taxonomy; **GATE-DEC-1** no new DEC) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260622-bug0027-q0035`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0027 rows **CB**, **CC**, **CD** |
| **Task count** | 5 mandatory (5/12 under `SPRINT_MAX_TASKS=12`; no split) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0027: Firefly sync fails with `unexpected status 401 Unauthorized` because PAT is present but revoked/expired. App currently surfaces generic `UnexpectedStatus(401)` — operator cannot distinguish "PAT invalid" from "Firefly unreachable" or "PAT missing". Fix: add `FireflyError::Unauthorized` variant, match HTTP 401, write wiremock test, run regression gates, then operator PAT-rename smoke.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| **CC** — 401 error taxonomy code | E1, E2, T1, G1 | `backend/src/firefly/mod.rs` |
| **CB** — operator PAT regeneration (ops) | V1 | ops-only |
| **CD** — ≥3 scheduled syncs regression (ops) | V1 | ops-only |
| Regression + gates | G1, V1 | cargo lib, firefly integration |

**Backend-only.** No frontend changes. No runbook change. No new DEC.

**Out of scope:** Startup health probe (GATE-PREFLIGHT-1 deferred to future US); 302 redirect handling (content negotiation ensures 401 — GATE-302-HANDLING); sync/mod.rs L249-256 changes (e.to_string() pass-through already captures new variant Display); config/mod.rs pat_configured check (emptiness only, by design).

## Task summary

| ID | Title | Est. | Depends | Acceptance | Gate |
|----|-------|------|---------|------------|------|
| E1 | `FireflyError::Unauthorized` variant | 0.5h | — | **CC** | GATE-ERROR-1, GATE-MESSAGE-1 |
| E2 | Match 401 → `Unauthorized` in request() | 0.5h | E1 | **CC** | GATE-ERROR-1 |
| T1 | wiremock 401 integration test | 1.5h | E2 | **CC**, **CD** | GATE-TEST-1 |
| G1 | Regression gates | 0.5h | E1, E2, T1 | all | — |
| V1 | Operator smoke: PAT regen + ≥3 syncs | 1h | G1 + ops gates | **CB**, **CC**, **CD** | — |

**Total estimate:** ~4h dev + ~1h operator V1.

## Deploy order

```
E1 (firefly/mod.rs — FireflyError::Unauthorized variant + Display)
  → E2 (firefly/mod.rs — 401 match arm in request())
  → T1 (tests/firefly_integration.rs — wiremock 401 test)
  → G1 (cargo lib + firefly integration + sync tests)
  → operator: BACKEND_DEPLOY
  → V1: PAT regen + container recreate + ≥3 scheduled syncs
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **CB** | V1 | After PAT regen + env update + container recreate, sync succeeds; no 401 in error_message |
| **CC** | E1, E2, T1, G1 | App surfaces `FireflyError::Unauthorized` display message on 401; wiremock test asserts variant |
| **CD** | V1 (with T1 coverage) | ≥3 scheduled syncs succeed post-PAT regen (no silent 401 recurrence) |

## Architecture → sprint mapping

| Architecture ID | Disposition | Gate |
|-----------------|-------------|------|
| E1 | Task **E1** | GATE-ERROR-1, GATE-MESSAGE-1 |
| E2 | Task **E2** | GATE-ERROR-1 |
| T1 | Task **T1** | GATE-TEST-1 |
| G1 | Task **G1** | — |
| V1 | Task **V1** | — |
| Preflight probe | **Out of scope** (GATE-PREFLIGHT-1 deferred) | — |

## Frozen boundaries

See `task.json` `frozen_boundaries`.

## Gate checklist

| Gate | Status | Detail |
|------|--------|--------|
| GATE-ERROR-1 | ✅ frozen | `Unauthorized` variant on `FireflyError` (unit variant, no fields) |
| GATE-MESSAGE-1 | ✅ frozen | Display: "firefly_personal_access_token invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN" |
| GATE-302-HANDLING | ✅ closed | No handling — content negotiation ensures 401 (`Accept: application/json` always sent) |
| GATE-PREFLIGHT-1 | ❌ deferred | Startup health probe deferred to future US |
| GATE-TEST-1 | ✅ frozen | wiremock 401 mock → assert `FireflyError::Unauthorized`; preserve existing `PersonalAccessTokenMissing` test |
| GATE-DEC-1 | ✅ closed | No new DEC (implementation detail per architecture) |

## User guide (USER_GUIDE_MODE=1)

**Waived** — operator remediation is standard Firefly PAT regen; app surfaces self-explanatory display message. No new `docs/user-guides/US-xxxx.md` workflow.

## Isolation evidence

- **phase_id:** sprint-plan
- **role:** tech-lead
- **fresh_context_marker:** sprint-plan-20260622-bug0027-tl-fresh
- **timestamp:** 2026-06-22T22:26:00Z
- **inputs_read:** architecture.md § BUG-0027, acceptance.md L57, scratchpad.md (SPRINT_MAX_TASKS=12), po_to_tl.md, state.md, Q0034 sprint format reference
- **isolation_scope:** artifact writes only; no code edits; no host `.env` / secrets read
