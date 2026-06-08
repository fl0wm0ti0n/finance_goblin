# Sprint S0017 — Summary

**Story:** US-0018  
**Status:** CLOSED / released (`0.18.0-us0018`, 2026-06-09)  
**Date:** 2026-06-09

## Context pack (refresh 2026-06-09)

- **Release:** `0.18.0-us0018` — category filters & expense trend analytics
- **Acceptance:** AC-1..AC-6 checked; operator category-filter smoke pass-with-prerequisites
- **Decisions:** DEC-0087, DEC-0088, DEC-0089, DEC-0090
- **Evidence:** handoffs/releases/S0017-release-notes.md, sprints/S0017/release-findings.md, sprints/S0017/uat.json
- **Next story:** US-0019 (goal-driven planning) — discovery phase

## Delivered

- Backend: month-spine `expense_series_by_month`, catalog + expense-series REST with server `summary` and `__uncategorized__` sentinel (DEC-0087)
- Frontend: shared `CategoryFilter` + `CategoryTrendChart` on Forecast Monthly, Planning Compare, Wealth Overview (DEC-0088, DEC-0089)
- Grafana: `$category` on cashflow and budgets dashboards (DEC-0089)
- Docs: `docs/user-guides/US-0018.md`; UAT OIDC smoke template

## Tests

- `cargo test --lib`: 193/193 PASS
- `npm test -- --run`: 7/7 PASS

## Deferred

- **T-0185** EXPLAIN probe / conditional index (DEC-0090) — no operator mirror
- Operator omniflow smoke (AC-6 live) — **BACKEND_FRONTEND_DEPLOY**, **FULL_FIREFLY_SYNC**, **GRAFANA_PROVISIONING_RELOAD**

## Tasks

T-0175..T-0184: **done** · T-0185: **deferred**
