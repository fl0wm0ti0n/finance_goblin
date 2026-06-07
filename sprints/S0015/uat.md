# UAT — Sprint S0015 / US-0014

**Sprint:** S0015  
**Story:** US-0014  
**Phase:** verify-work complete — runtime smoke pending deploy  
**Status:** PASS (code/test); AC-8 pass-with-prerequisites  
**Plan-verified at:** 2026-06-08T12:45:00Z  
**QA verified at:** 2026-06-08T13:10:00Z  
**Verify-work at:** 2026-06-08T13:20:00Z  
**Orchestrator:** `auto-20260608-us0014-001`  
**Decision:** DEC-0077

## Inputs

- Acceptance: `docs/product/acceptance.md` § US-0014
- Architecture: `docs/engineering/architecture.md` § US-0014, `decisions/DEC-0077.md`
- Research: R-0072, R-0073; frozen DEC-0073, DEC-0074
- User guide: `docs/user-guides/US-0014.md`
- Sprint tasks: `sprints/S0015/tasks.md`

## Operator gate (pre-runtime smoke)

| Gate | Action |
|------|--------|
| **BACKEND_FRONTEND_DEPLOY** | Deploy S1–S2 frontend on US-0010 external profile (`financegnome.omniflow.cc`) |

## Regression checklists (code audit — verify-work)

### AC-1 empty-state onboarding (T-0155)

| Check | Contract | Audit |
|-------|----------|-------|
| Empty branch | `plans.length === 0` → template `card-grid` | PASS — `PlanningPage.tsx` L359–396 |
| Templates | Current, Leasing, Savings mode, House purchase, Allocation target, Custom (6 cards) | PASS — `TEMPLATES` constant |
| Primary CTA | **Create empty plan** visible and wired | PASS — L369–377 |
| Name field | Plan name input present | PASS — L364–368 |
| Reach add-lines | Create empty plan → editable plan with inline add form | PASS — POST custom template + add form L489–500 |

### Compare / PVA verify-only (T-0161)

| Surface | Contract | Audit |
|---------|----------|-------|
| Compare footnote | Overlay-only delta vs projected balance | PASS — L600–603 |
| Zero-adjustment delta | `monthly_delta_sum` = 0.00 | PASS — backend AE3 + footnote; `compare_zero_adjustments_overlay_delta_is_zero` |
| PVA no active | Guided card + Scenarios / Set active buttons | PASS — L613–632 |
| PVA with active | Month selector + plan vs actual rows | PASS — L636–677 |
| API/formula | No Compare or PVA API changes | PASS — verify-only |

### DEC-0077 mutation feedback (T-0158..T-0160)

| Surface | Contract | Audit |
|---------|----------|-------|
| Success card | Green `#ecfdf5`, 4s auto-dismiss | PASS — `planningFeedback.tsx` |
| Error card | Red `#fef2f2`, Dismiss button | PASS — `PlanningFeedbackCard` |
| onError coverage | All 7 mutations wired | PASS — createPlan, activate, version, add/update/delete, apply |
| PVA invalidation | Adjustment CRUD + activate + createPlan | PASS — `queryClient.invalidateQueries` |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | Prerequisite | BUG-0011 AD/AE/AF released (Q0019) | **pass** | Q0019 release; DEC-0073/DEC-0074 |
| UAT-2 | AC-1 | Empty state: template grid + Create empty plan → add-lines flow | **pass** | T-0155; PlanningPage code audit |
| UAT-3 | AC-2 | Add adjustment → success toast; Compare/PVA refresh without manual reload | **pass** | T-0160; invalidation matrix |
| UAT-4 | AC-3 | Compare zero-adjustment plan: 0.00 delta + overlay footnote | **pass** | T-0161; plans_integration |
| UAT-5 | AC-4 | PVA without active plan: guided card with Set active / Scenarios links | **pass** | T-0161; plans_integration |
| UAT-6 | AC-5 | Create empty plan / Create from template / Apply template → success confirmation | **pass** | T-0157; planningFeedback |
| UAT-7 | AC-6 | Yellow Set-active banner mentions PVA + Grafana Dashboard 3 (Budgets) | **pass** | T-0156; banner copy |
| UAT-8 | AC-7 | Force mutation failure → red error card visible with Dismiss | **pass** | T-0158, T-0159; 7× onError |
| UAT-9 | AC-8 | OIDC `/planning` three-tab smoke on external profile | **pass_with_prerequisites** | T-0162; pending **BACKEND_FRONTEND_DEPLOY** |

## Results summary

| Metric | Count |
|--------|-------|
| Acceptance criteria (AC-1..AC-8) | 8 |
| Pass (code/test) | 7 (AC-1..AC-7) |
| Pass-with-prerequisites | 1 (AC-8) |
| Fail | 0 |
| Prerequisite checked | 1 (BUG-0011 Q0019) |

**Verdict:** **PASS** — AC-1 through AC-7 satisfied at code/test level; AC-8 runtime deferred to operator post-deploy smoke per US-0010 precedent.

## Omniflow smoke steps (post-deploy — operator)

**Profile:** US-0010 external — `https://financegnome.omniflow.cc` (Traefik `auth` + OIDC)

### Planning OIDC (AC-8)

1. Authenticate at `https://financegnome.omniflow.cc`
2. Open `/planning` → **Scenarios** tab — empty or existing plans load
3. **Compare** tab — footnote visible; zero-overlay shows 0.00 delta
4. **Plan vs Actual** tab — guided card when no active plan; data when active
5. Create plan or add line — green success card (4s auto-dismiss)
6. Force mutation failure (e.g. stop backend or invalid payload) — red error card with **Dismiss**
7. After create (non-active plan) — yellow banner mentions Plan vs Actual + Grafana Dashboard 3 (Budgets, `uid=budgets`)

## Notes

- AC-1/AC-3/AC-4 largely verify-only (Q0019 shipped) — verify-work code audit PASS
- Compare formula and PVA API frozen per DEC-0073 / DEC-0074
- Error-path smoke (step 6) requires S2 deploy — force 4xx/5xx after **BACKEND_FRONTEND_DEPLOY**
