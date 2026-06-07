# CRS — US-0014

## Purpose

Make **planning mode intuitive from first visit** by closing UX gaps left after BUG-0011 functional fixes: visible mutation feedback, template create confirmations, set-active guidance, and operator verification — without backend contract changes.

## Scope

**In scope**

- `PlanningPage.tsx` mutation `onError` / success toasts per DEC-0077
- Set-active banner extension (Grafana Dashboard 3 cue)
- Query invalidation: `plan-vs-actual` on adjustment CRUD + activate
- Regression verify: AC-1 onboarding, AC-3 Compare help, AC-4 PVA guided card
- User guide `docs/user-guides/US-0014.md`
- OIDC `/planning` three-tab smoke (AC-8)

**Out of scope**

- Plans API / compare metric / PVA response shape changes
- AI plan simulation (US-0006)
- Crypto allocation scenarios (US-0007)

## Acceptance criteria ref

See `docs/product/acceptance.md` § US-0014 — 9 rows (1 prerequisite checked + AC-1–AC-8).

## Dependencies

- BUG-0011 DONE (Q0019, DEC-0073, DEC-0074)
- US-0004 DONE (plan engine)
- US-0010 external profile (AC-8 smoke target)
