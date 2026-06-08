# Technical Specification — US-0018

## Overview

Implement **DEC-0087** through **DEC-0090**: monthly per-category expense API with uncategorized sentinel; shared React filter and bar trend chart; surface-scoped filter semantics; Grafana `$category` variable; optional index gate.

## Components

| Layer | Change |
|-------|--------|
| `backend/src/transactions/repository.rs` | **Extend** — `expense_series_by_month` |
| `backend/src/api/categories.rs` | **New** — catalog + expense-series routes |
| `backend/src/api/mod.rs` | **Extend** — register category routes |
| `frontend/src/components/category/CategoryFilter.tsx` | **New** |
| `frontend/src/components/category/CategoryTrendChart.tsx` | **New** |
| `frontend/src/pages/ForecastPage.tsx` | **Extend** — filter + trend chart |
| `frontend/src/pages/PlanningPage.tsx` | **Extend** — compare toolbar widget |
| `frontend/src/pages/WealthPage.tsx` | **Extend** — category subsection |
| `frontend/src/lib/api.ts` | **Extend** — category types + fetchers |
| `grafana/.../cashflow.json` | **Extend** — `$category` + panel |
| `grafana/.../budgets.json` | **Extend** — `$category` on Ist leg |
| `docs/user-guides/US-0018.md` | **New** — at execute |

## Interfaces

### `GET /api/v1/categories/expense-series`

**Query:** `category_id` (required), `months` (default 12, max 24), `end` (optional ISO date)

**Response:**

```json
{
  "category_id": "3",
  "category_name": "Groceries",
  "uncategorized": false,
  "months": [
    { "month": "2025-07", "outflow_eur": 300.0, "inflow_eur": 0.0, "transaction_count": 12 }
  ],
  "summary": {
    "mom_delta_pct": -16.7,
    "best_month": "2025-08",
    "worst_month": "2025-07"
  },
  "meta": { "period_start": "2025-07-01", "period_end": "2026-06-30" }
}
```

**Uncategorized:** `category_id=__uncategorized__` → `uncategorized: true`, `category_label: "Uncategorized"`.

### CategoryFilter props

```typescript
interface CategoryFilterProps {
  value: string;  // "" | firefly_id | "__uncategorized__"
  onChange: (categoryId: string) => void;
  allowAll?: boolean;
  includeUncategorized?: boolean;
}
```

## Sequencing (sprint-plan input)

1. C1–C2 backend API + tests
2. C3–C4 frontend filter/chart + forecast integration
3. C5–C6 planning + wealth (parallel after C3)
4. G1–G2 Grafana provisioning
5. D1 user guide + V1 smoke

## Verification

- Unit: spine zeros, uncategorized sentinel, 24-month cap, summary math
- Integration: category routes behind JWT
- UI: bar chart labels, empty state, MoM callouts
- Grafana: `POST /analytics/grafana/api/ds/query` with `$category` set and empty
- Regression: US-0015 monthly forecast unchanged when category selected
- Optional: EXPLAIN probe per DEC-0090

## Non-functional

- Aggregate-only public REST (no raw rows)
- Single sprint ≤12 tasks (S0017)
- Read-only Firefly preserved
