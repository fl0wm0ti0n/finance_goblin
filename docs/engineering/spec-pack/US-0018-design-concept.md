# Design Concept — US-0018

## Summary

US-0018 delivers a **shared category filter contract** and **monthly expense trend analytics** across forecast, planning, wealth, and two Grafana dashboards — building on BUG-0006 mirror `category_id` ingest without altering DEC-0007 forecast projection or US-0015 bucket mapping.

## Goals

- Shared `CategoryFilter` on forecast monthly, planning compare, wealth breakdown (AC-1)
- `GET /api/v1/categories` catalog + `GET /api/v1/categories/expense-series` monthly spine API (AC-2, AC-5)
- Bar trend chart with month EUR labels and MoM/best/worst insight (AC-3, AC-4)
- Grafana `$category` on cashflow + budgets (AC-1)
- OIDC external profile regression (AC-6)

## Non-goals

- Multi-category chart overlay (deferred per DEC-0088)
- Category-scoped forecast re-projection (US-0019)
- Grafana↔SPA bidirectional filter sync (DEC-0089)
- Firefly category editing; ML auto-labeling
- Unconditional `category_id` index (DEC-0090)

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0087 | Month spine SQL + `__uncategorized__` | AC-3 labels; AC-5 explicit bucket (R-0083) |
| DEC-0088 | Single-select + bar chart | Finanzguru-like discrete months; reuse ECharts |
| DEC-0089 | Forecast actuals-only; planning widget | Avoid forecast/plan engine fork |
| DEC-0090 | Defer index unless EXPLAIN >50 ms | ~1k row mirror acceptable at MVP |
| DEC-0007 | Household forecast unchanged | AC-6 regression; display filter only |

## User experience

Forecast Monthly tab gains a category picker and bar chart showing **actual spending** by month for the selected category. Household Income/Fixed/Variable cards and bucket chart stay unchanged. Planning compare adds the same trend widget beside version deltas. Wealth shows a category spending summary. Grafana dashboards get an independent category dropdown.
