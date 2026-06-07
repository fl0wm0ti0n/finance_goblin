# QA findings — S0015 / US-0014

**Story:** US-0014 — Planning mode intuitive UX completion  
**Sprint:** S0015  
**Phase:** `/qa`  
**Orchestrator:** `auto-20260608-us0014-001`  
**Decision:** DEC-0077  
**QA agent:** fresh subagent (`qa-20260608-s0015-us0014`)  
**Date:** 2026-06-08  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — All blocking acceptance criteria (prerequisite + AC-1 through AC-7) satisfied via code review and automated guards. AC-8 OIDC three-tab smoke is **pass_with_prerequisites** pending operator **BACKEND_FRONTEND_DEPLOY** (consistent with S0010/S0011/Q0019 precedent). Hand off to `/verify-work`.

## Test plan

| # | Check | Method | Result |
|---|-------|--------|--------|
| 1 | Prerequisite BUG-0011 Q0019 | Intake + release evidence | **PASS** (pre-verified) |
| 2 | AC-1 empty-state onboarding | Code audit `PlanningPage.tsx` L336–464 | **PASS** |
| 3 | AC-2 add-lines + invalidation | Code audit mutations + invalidation matrix | **PASS** |
| 4 | AC-3 Compare 0.00 delta + footnote | Code audit L668–671; `plans_integration` zero-adjustment test | **PASS** |
| 5 | AC-4 PVA guided card | Code audit L681–701 | **PASS** |
| 6 | AC-5 success confirmations | Code audit create/template/apply paths | **PASS** |
| 7 | AC-6 set-active banner | Code audit L468–472 | **PASS** |
| 8 | AC-7 error surfaces (7 mutations) | Code audit `planningFeedback.tsx` + all `onError` handlers | **PASS** |
| 9 | AC-8 OIDC three-tab smoke | UAT template + operator gate | **pass_with_prerequisites** |
| 10 | Frontend unit tests | Run `npm test` | **PASS** (5/5) |
| 11 | Plans integration guard | Run `cargo test --test plans_integration` | **PASS** (5/5) |
| 12 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |

## Automated test output

```
$ cd frontend && npm test
 ✓ src/pages/planningFeedback.test.ts (3 tests)
 ✓ src/components/chat/ChatPanel.test.tsx (2 tests)
 Test Files  2 passed (2)
      Tests  5 passed (5)
EXIT_CODE=0

$ cd backend && cargo test --test plans_integration
running 5 tests
test compare_zero_adjustments_overlay_delta_is_zero ... ok
test plan_create_apply_recompute_plan_vs_actual ... ok
test compare_leasing_template_overlay_delta_approx_minus_300 ... ok
test plan_vs_actual_without_active_plan_returns_error ... ok
test plan_module_has_no_firefly_writes ... ok
test result: ok. 5 passed; 0 failed
EXIT_CODE=0
```

## Acceptance criteria results

| AC | Requirement | Result | Evidence |
|----|-------------|--------|----------|
| Prerequisite | BUG-0011 AD/AE/AF released (Q0019, DEC-0073, DEC-0074) | **PASS** | `docs/product/acceptance.md` L166 checked; Q0019 release notes |
| AC-1 | Empty state: template grid + **Create empty plan** → add-lines flow | **PASS** | `PlanningPage.tsx` L336–464: `empty` branch with 6-card `TEMPLATES` grid, name input, primary **Create empty plan** CTA (`template: "custom"`); post-create selects plan and shows inline add form L557–568 |
| AC-2 | Add adjustment success toast; Compare/PVA invalidate immediately | **PASS** | `addAdjustmentMutation` L273–278: success toast `"Adjustment added"`; invalidates `plan-version`, `plan-compare`, `plan-vs-actual`; `onError` L280–284 surfaces red card |
| AC-3 | Compare zero-adjustment: **0.00** delta + overlay footnote | **PASS** | Footnote L668–671 explains overlay-only delta vs projected balance; `plans_integration.rs` `compare_zero_adjustments_overlay_delta_is_zero` asserts `monthly_delta_sum == "0.00"` |
| AC-4 | PVA without active plan: guided card + navigation | **PASS** | L681–701: `status === "no_active_plan"` renders guided card with **Go to Scenarios** and **Set active now** buttons |
| AC-5 | Create empty / template / apply → visible success confirmation | **PASS** | `createPlanMutation` L167–177 success messages; `applyTemplateMutation` L223–232 template-specific success; `#ecfdf5` card via `PlanningFeedbackCard` L72 |
| AC-6 | Yellow banner mentions PVA + Grafana Dashboard 3 (Budgets) | **PASS** | L468–472: `#fffbeb` banner copy includes "Plan vs Actual and Grafana Dashboard 3 (**Budgets**)" |
| AC-7 | All 7 mutations show operator-visible errors on failure | **PASS** | `planningFeedback.tsx`: error card `#fef2f2` + **Dismiss** button L77–80; all 7 mutations wire `onError` → `formatPlanningError` (L179–183, 195–199, 235–239, 250–254, 280–284, 310–314, 329–333) |
| AC-8 | OIDC `/planning` three-tab smoke on external profile | **pass_with_prerequisites** | `sprints/S0015/uat.md` UAT-9 template; runtime deferred pending **BACKEND_FRONTEND_DEPLOY** |

## DEC-0077 contract verification

| Element | Contract | Result |
|---------|----------|--------|
| Helper module | `planningFeedback.tsx` co-located | **PASS** |
| Success styling | `#ecfdf5`, 4s auto-dismiss | **PASS** (`SUCCESS_DISMISS_MS = 4000`) |
| Error styling | `#fef2f2`, persist until Dismiss | **PASS** |
| Message truncation | 240 chars max | **PASS** (unit test + helper) |
| Single feedback slot | New message replaces prior | **PASS** (`showPlanningFeedback` clears timer) |
| Invalidation matrix | PVA on adjustment CRUD + activate + createPlan | **PASS** |

## Findings summary

| ID | Severity | Finding | Blocking US-0014 |
|----|----------|---------|------------------|
| — | — | No findings | — |

**Blocking findings:** 0  
**Critical findings:** 0

## Operator gate (non-blocking for QA)

Runtime omniflow planning smoke (UAT-2 … UAT-9 in `sprints/S0015/uat.md`) requires operator **BACKEND_FRONTEND_DEPLOY**: deploy S1–S2 frontend on US-0010 external profile, smoke `/planning` Scenarios + Compare + Plan vs Actual, force mutation failure for UAT-8. Deferred to `/verify-work`.

## Handoff

**Next phase:** `/verify-work` in fresh subagent/chat  
**Stop reason:** QA_PASS — US-0014 prerequisite + AC-1 through AC-7 verified; AC-8 pass-with-prerequisites; no `handoffs/qa_to_dev.md` required
