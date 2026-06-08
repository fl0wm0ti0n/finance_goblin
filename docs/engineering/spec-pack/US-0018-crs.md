# CRS — US-0018

## Purpose

Give household budgeters **category-scoped visibility** — filter by Firefly category across product views and see **month-over-month expense trends** with explicit uncategorized handling — as the analytics foundation for US-0019 goal planning what-ifs.

## Scope

**In scope**

- `TransactionsRepository::expense_series_by_month` (DEC-0087)
- `GET /api/v1/categories` and `GET /api/v1/categories/expense-series`
- `CategoryFilter` + `CategoryTrendChart` shared components (DEC-0088)
- Forecast monthly, planning compare, wealth integrations (DEC-0089)
- Grafana `$category` on cashflow + budgets (DEC-0089)
- `docs/user-guides/US-0018.md`
- OIDC US-0010 external profile smoke (AC-6)

**Out of scope**

- Multi-category overlay charts
- Category-scoped forecast recompute
- Plan compare API category fork
- Grafana↔SPA filter sync
- Firefly write-back
- US-0015 bucket inference changes

## Acceptance criteria ref

See `docs/product/acceptance.md` § US-0018 — AC-1 through AC-6.

## Dependencies

- BUG-0006 DONE (`category_id` ingest)
- US-0011 DONE (analytics embed routes)
- US-0015 DONE (bucket mapping — regression guard AC-6)
- DEC-0007 (forecast engine — unchanged in MVP)
