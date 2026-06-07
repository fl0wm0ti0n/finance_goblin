# Sprint S0015

**ID:** S0015  
**Story:** US-0014 — Planning mode intuitive UX completion  
**Status:** PLANNED  
**Created:** 2026-06-08  
**Orchestrator:** `auto-20260608-us0014-001`

## Goal

Deliver **DEC-0077** planning mutation feedback on `PlanningPage.tsx`: page-local `planningFeedback` helper with success/error card variants; mandatory `onError` on all seven planning mutations; success confirmations on create/template/add/activate; immediate `plan-vs-actual` invalidation on adjustment CRUD; set-active banner extended for Grafana Dashboard 3. Verify-first AC-1/AC-3/AC-4 (Q0019 shipped); finalize `docs/user-guides/US-0014.md`; OIDC smoke template in UAT.

## Scope

| Slice | Tasks | Primary files |
|-------|-------|---------------|
| **US-0014-S1** — Verify + banner + confirmations | T-0155 … T-0157 | `frontend/src/pages/PlanningPage.tsx` |
| **US-0014-S2** — Mutation feedback + invalidation | T-0158 … T-0160 | `planningFeedback.ts`, `PlanningPage.tsx` |
| **US-0014-S3** — Verify + user guide + OIDC UAT | T-0161, T-0162 | `PlanningPage.tsx`, `docs/user-guides/US-0014.md` |

**Out of scope:** Compare formula changes (DEC-0073); PVA API shape (DEC-0074); auto-activate first plan; global toast library / MutationCache refactor; backend plans API changes.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Toast flood on rapid retry | Single active feedback slot — new message replaces prior | DEC-0077, T-0158 |
| Duplicate Q0019 rewrite | AC-1/AC-3/AC-4 verify-only — regression checklist, no greenfield | T-0155, T-0161 |
| DEC-0073/0074 drift | Frozen contracts — frontend-only execute | architecture § US-0014 |
| AC-8 operator gate | OIDC smoke pass-with-prerequisites **BACKEND_FRONTEND_DEPLOY** | UAT template T-0162 |
| applyTemplate toast gap | Extend beyond Custom-only per DEC-0077 §3 in T-0157 | R-0073 |

## Definition of Done

- All 8 sprint tasks complete (`T-0155` … `T-0162`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` § US-0014 (8 open AC + prerequisite checked)
- All seven planning mutations wire `onError` → red error card (AC-7)
- `plan-vs-actual` invalidated immediately on adjustment CRUD + activate + createPlan (AC-2)
- Set-active banner mentions Plan vs Actual + Grafana Dashboard 3 (`budgets`) (AC-6)
- `docs/user-guides/US-0014.md` validated against shipped UI (`USER_GUIDE_MODE=1`)
- Operator gate **BACKEND_FRONTEND_DEPLOY** documented before omniflow OIDC smoke (AC-8)

## Architecture references

- `docs/engineering/architecture.md` § US-0014
- `decisions/DEC-0077.md`
- Research: R-0072, R-0073; frozen DEC-0073, DEC-0074, DEC-0024
- Spec-pack: `docs/engineering/spec-pack/US-0014-{design-concept,crs,technical-specification}.md`
- User guide: `docs/user-guides/US-0014.md`
- Discovery: `handoffs/po_to_tl.md#discovery-20260608-us0014`
- Acceptance: `docs/product/acceptance.md` § US-0014
- Handoff: `handoffs/tl_to_dev.md` (`sprint-plan-20260608-s0015-us0014`)

## Sequencing (frozen)

```text
S2 foundation: T-0158 → T-0159 → T-0160
S1 (after T-0158 for toasts): T-0155 ∥ T-0156; T-0157 after T-0158
S3 (after S2): T-0161 → T-0162
Operator: BACKEND_FRONTEND_DEPLOY → verify-work omniflow /planning smoke (UAT)
```

## Split decision

- **Why 8 tasks:** Maps architecture slices US-0014-S1(3) + S2(3) + S3(2); within `SPRINT_MAX_TASKS=12`.
- **Why not split S0015a/b:** Single epic; S2 helper is prerequisite for S1 toasts and S3 error-path smoke — incomplete UX if split.
- **S2-weighted:** T-0158..T-0160 are primary execute gap (AC-7 + AC-2); S1 mostly verify + banner/copy.
- **OIDC smoke:** Captured in UAT placeholder (T-0162 template) — reduces task count while preserving AC-8 gate.

## Next phase

`/plan-verify` in fresh subagent/chat
