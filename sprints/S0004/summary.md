# Sprint S0004 Summary — US-0004 Financial Planning

## Context pack (refresh 2026-05-31)

- **Status:** CLOSED / released (`0.4.0-us0004`)
- **Story:** US-0004 DONE — 6/6 acceptance, queue `released`
- **Evidence:** handoffs/releases/S0004-release-notes.md, sprints/S0004/release-findings.md
- **Next story:** US-0005 (wealth analysis, budget drift & scarcity alerts) — discovery phase

**Story:** US-0004  
**Sprint:** S0004  
**Date:** 2026-05-31

## Delivered

| Layer | Deliverable |
|-------|-------------|
| Database | `004_plans.sql` — plans, versions, adjustments, plan_daily hypertable (DEC-0022) |
| Plan Engine | Delta overlay on forecast baseline, templates, project (DEC-0019) |
| PlanService | Plan-vs-Ist, compare, versioning cap v1/v2/v3 (DEC-0020, DEC-0021) |
| Recompute | Async on plan save + post-forecast hook (DEC-0023) |
| API | 17 JWT routes under `/api/v1/plans/*` |
| Frontend | `/planning` — Scenarios, Compare, Plan vs Actual; lazy ECharts |
| Grafana | Dashboard `budgets` in Analytics folder (DEC-0024) |
| Tests | 10 plan unit tests; `plans_integration` (SKIP without DATABASE_URL) |
| Docs | `docs/user-guides/US-0004.md` |

## Task completion

T-0037 … T-0048 — all complete (12/12).

## Test results

```
bash tests/run-tests.sh PASS
- cargo test --lib: 28 passed
- plans_integration: SKIP (DATABASE_URL unset)
- firefly_readonly: PASS
- frontend build: PASS
```

## Key decisions applied

DEC-0019 (delta overlay), DEC-0020 (version cap), DEC-0021 (plan-vs-Ist metric), DEC-0022 (migration 004), DEC-0023 (recompute triggers), DEC-0024 (active plan + Dashboard 3)

## Known limitations

- Integration tests require operator TimescaleDB with `DATABASE_URL`
- Live plan-vs-Ist and compare need synced asset transactions + forecast recompute
- Savings mode template needs US-0003 confirmed subscriptions
- Plan recompute async — stale badge until snapshot completes
- ECharts main bundle ~1 MB (code-split compare/plan-vs-actual charts)

## Release closure

- Released 2026-05-31; publish skipped (`RELEASE_PUBLISH_MODE=disabled`)
- Carry-forward: OIDC E2E needs IdP or `AUTH_DEV_BYPASS=true`; integration tests need `DATABASE_URL`; scarcity threshold still static on Dashboard 1 until US-0005
