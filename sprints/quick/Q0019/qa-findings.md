# QA Findings — Quick Q0019 / BUG-0011

**Work item:** BUG-0011 (defect)  
**Quick task:** Q0019  
**QA phase:** `/qa`  
**Date:** 2026-06-08  
**Orchestrator:** `auto-20260608-bug0011-001`  
**Decisions:** DEC-0073, DEC-0074  
**QA agent:** fresh subagent (`qa-20260608-q0019-bug0011`)  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — Acceptance rows **AD**, **AE**, and **AF** satisfied at code/test level. Zero blocking findings. Hand off to `/verify-work` (V1 omniflow runtime probes deferred until **BACKEND_FRONTEND_DEPLOY**).

## Scope

BUG-0011 planning mode fixes: first-run/add-line UX (**AD**), overlay-only compare deltas (**AE**), and plan-vs-actual 200 `no_active_plan` guided UX (**AF**) per `handoffs/dev_to_qa.md` and `sprints/quick/Q0019/summary.md`.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0019/summary.md`, `sprints/quick/Q0019/tasks.md`, `sprints/quick/Q0019/uat.md`, `backend/src/plan/overlay.rs`, `backend/src/plan/repository.rs`, `backend/src/plan/service.rs`, `backend/src/plan/types.rs`, `backend/src/api/plans.rs`, `backend/tests/plans_integration.rs`, `frontend/src/pages/PlanningPage.tsx`, `frontend/src/lib/api.ts`, `decisions/DEC-0073.md`, `decisions/DEC-0074.md`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Full lib regression | `cd backend && cargo test --lib` | **PASS** (160/160) |
| T-2 | Plans integration suite | `cd backend && cargo test --test plans_integration` | **PASS** (5/5; vacuous when `DATABASE_URL` unset — see note) |
| T-3 | AE3 overlay unit tests | `cargo test --lib plan::overlay::tests::monthly_overlay_delta_sum` | **PASS** (2/2) |
| T-4 | AF1 API serialization | `cargo test --lib api::plans::plan_vs_actual_api_tests` | **PASS** (2/2) |
| T-5 | **AD** — Create empty plan + first-run grid | Code review `PlanningPage.tsx` AD1 | **PASS** |
| T-6 | **AD** — Inline add/edit adjustment (POST/PATCH) | Code review AD2 + mutation wiring | **PASS** |
| T-7 | **AD** — Custom Apply toast + invalidation | Code review AD3 | **PASS** |
| T-8 | **AD** — Compare footnote + Set active banner | Code review AD4 | **PASS** |
| T-9 | **AE** — `monthly_overlay_delta_sum` helper | Code review AE1 + overlay.rs | **PASS** |
| T-10 | **AE** — Repository + service compare wiring | Code review AE2 (`version_metrics`, `project_adjustments_in_memory`) | **PASS** |
| T-11 | **AE** — Zero-overlay → 0.00 | Unit test `monthly_overlay_delta_sum_zero_when_no_adjustments` | **PASS** |
| T-12 | **AE** — Leasing ~-300 overlay delta | Unit test `monthly_overlay_delta_sum_leasing_template_approx_minus_300` | **PASS** |
| T-13 | **AF** — Route 200 `no_active_plan` tagged JSON | Code review `plan_vs_actual` handler + `PlanVsActualApiResponse` | **PASS** |
| T-14 | **AF** — PVA guided empty state (`retry: false`) | Code review `PlanningPage.tsx` pvaQuery branch | **PASS** |
| T-15 | Regression OIDC `/planning` three-tab smoke | `sprints/quick/Q0019/uat.md` runtime probes | **DEFERRED** — verify-work after deploy |
| T-16 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |

### Test output (T-1)

```
test result: ok. 160 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Test output (T-2)

```
running 5 tests
test compare_zero_adjustments_overlay_delta_is_zero ... ok
test plan_vs_actual_without_active_plan_returns_error ... ok
test plan_create_apply_recompute_plan_vs_actual ... ok
test compare_leasing_template_overlay_delta_approx_minus_300 ... ok
test plan_module_has_no_firefly_writes ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Note (T-2):** `DATABASE_URL` unset in QA environment — four DB-backed integration tests return early (vacuous PASS). AE compare/PVA DB paths are additionally covered by AE3 unit tests and AF1 serialization tests; live DB integration deferred to verify-work with operator `DATABASE_URL`.

## Acceptance criteria matrix (BUG-0011)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **AD** | "Start empty and add lines" creates editable plan with add-line UX — not silent no-op; Custom toast; Set active banner | **PASS** (code) / **DEFERRED** (runtime) | First-run card with template grid + **Create empty plan** (`template: "custom"`); inline Add adjustment form with POST/PATCH/DELETE; Custom Apply toast `"Custom plan ready — add lines below"` + query invalidation; post-create Set active banner. Tests T-5–T-8 PASS. Live AD-1–AD-3 **DEFERRED** |
| **AE** | Compare with empty/minimal plan shows zero or neutral deltas — not illogical baseline aggregates | **PASS** (code) / **DEFERRED** (runtime) | `monthly_overlay_delta_sum()` in `overlay.rs` with empty-adjustment guard → 0.0; wired in `repository.rs` `version_metrics` and `service.rs` `project_adjustments_in_memory`; compare footnote explains overlay-only delta vs projected balance. Unit tests T-11–T-12 PASS. Live AE-1/AE-2 **DEFERRED** |
| **AF** | Plan-vs-actual returns 200 JSON or guided UX when no active plan — not 404 breaking tab | **PASS** (code) / **DEFERRED** (runtime) | Route checks `active_plan()` before service call; returns `PlanVsActualApiResponse::NoActivePlan` at 200; frontend `pvaQuery` with `retry: false` renders guided card + Set active CTA. Tests T-13–T-14 PASS. Live AF-1/AF-2 **DEFERRED** |
| Regression | OIDC-enabled deploy `/planning` three-tab regression | **DEFERRED** | V1 checklist in `uat.md`; REG-1/REG-2 pending verify-work |

## Code review highlights

| Area | Finding | Severity |
|------|---------|----------|
| AE1–AE3 | Shared `monthly_overlay_delta_sum`; empty adjustments → 0.0; Leasing ~-300 tests | **PASS** |
| AE2 | Both compare call sites use overlay helper (not `planned_net` sum) | **PASS** |
| AD4 | Compare footnote present; Set active banner after create | **PASS** |
| AF1 | Tagged enum mirrors risk-score pattern; route returns 200 before service `NoActivePlan` error | **PASS** |
| AF2 | Guided PVA card with Scenarios + Set active CTAs; no blank tab on 404 | **PASS** |
| Sequencing | AE-before-AF honored; no auto-activate on create | **PASS** |

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

**Informational:** `plan_vs_actual_without_active_plan_returns_error` integration test asserts service layer still returns `PlanError::NoActivePlan` — correct; HTTP 200 handling is route-layer only per DEC-0074.

## Verify-work readiness

| Gate | Status |
|------|--------|
| Code QA (AE1–AD4, T1) | **READY** |
| `cargo test --lib` | **READY** — 160/160 PASS |
| `cargo test --test plans_integration` | **READY** — 5/5 PASS (vacuous without `DATABASE_URL`) |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** |
| V1 omniflow smoke (AD/AE/AF rows) | **PENDING** — blocked on deploy |

## Next phase

**`/verify-work`** — operator **BACKEND_FRONTEND_DEPLOY** then V1 omniflow probes per `sprints/quick/Q0019/uat.md`.

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
