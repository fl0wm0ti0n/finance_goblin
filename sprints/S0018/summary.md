# Summary — Sprint S0018 / US-0019

**Story:** US-0019 — Goal-driven planning with per-plan stats & category savings  
**Sprint:** S0018  
**Execute date:** 2026-06-09  
**Orchestrator:** `auto-20260608-us0019-001`

## Delivered

- **DEC-0091:** Migration `013_goal_balance.sql`; `goal_balance` template; plan columns `target_balance_eur`, `target_date`, `goal_account_id`
- **DEC-0092:** `GET /api/v1/plans/{id}/goal-stats`; yearly rollup; feasibility copy; `beyond_horizon`
- **DEC-0093:** Category `remove_outflow` cap via 3-month `expense_series_by_month` average
- **DEC-0095:** Goal account projection fork; default max-balance asset on create
- **DEC-0094 / DEC-0097:** `GET …/category-savings-suggestions`; CategorySavingsModal; optional `get_category_savings` AI tool (7-tool registry)
- **DEC-0096:** PVA tab unchanged (household active plan)
- **Docs:** `docs/user-guides/US-0019.md`; UAT regression checklist in `uat.md`

## Tasks

| ID | Status |
|----|--------|
| T-0186..T-0197 | **done** (12/12) |

## Tests

- `cargo test --lib`: **204/204 PASS**
- `npm test -- --run`: **9/9 PASS**

## Operator gates (verify-work)

- **BACKEND_FRONTEND_DEPLOY**
- **FULL_FIREFLY_SYNC**

## Release

- **Version:** `0.19.0-us0019` (2026-06-09)
- **Acceptance:** AC-1..AC-6 checked
- **Operator smoke:** AC-6 pass-with-prerequisites (BACKEND_FRONTEND_DEPLOY, FULL_FIREFLY_SYNC)
- **Evidence:** `handoffs/releases/S0018-release-notes.md`, `sprints/S0018/release-findings.md`

## Next phase

Backlog drain continues — **US-0020** discovery (last story in bundle)
