# Technical Specification — BUG-0026

## Overview

Frontend-only fix: extract pure month-selection helpers and wire `ForecastPage` summary cards to a resolved reference month with shared subtitle. Extends **US-0002** monthly view semantics — no backend or API shape change.

## Components

| Layer | Change | Gate |
|-------|--------|------|
| `frontend/src/pages/forecastSummaryMonth.ts` | Pure helpers `resolveForecastSummaryPoint`, `formatForecastMonthLabel`, `formatForecastSummarySubtitle` (new) | H1 |
| `frontend/src/pages/ForecastPage.tsx` | Replace `series[0]` useMemo; subtitle above card grid | F1 |
| `frontend/src/pages/forecastSummaryMonth.test.ts` | Vitest table-driven cases + partial-month fixture | T1 |

## Summary month contract (frozen)

```typescript
function resolveForecastSummaryPoint(series: ForecastMonthlyPoint[]): ForecastMonthlyPoint | null {
  if (series.length === 0) return null;
  if (parseFloat(series[0].income) === 0 && series.length > 1) {
    return series.find((p) => parseFloat(p.income) > 0) ?? series[1];
  }
  return series[0];
}

function formatForecastSummarySubtitle(monthIso: string): string {
  return `Forecast for ${formatForecastMonthLabel(monthIso)}`;
}
```

## UI behavior

| Operator view | Expected |
|---------------|----------|
| Account **114** partial June + July salary | Subtitle **Forecast for July 2026**; Income card **3266.16** |
| `series[0].income > 0` | Subtitle names `series[0]` month; values from `series[0]` |
| All-zero income series | Subtitle names `series[0]`; cards show zero |
| Category filter selected | Cards unchanged; filter still scopes trend chart only (**DEC-0089**) |
| Empty series | Card grid hidden (unchanged) |

## API (unchanged)

- `GET /api/v1/forecast/monthly?account_id={id}` — ordered `series[]` with `month`, `income`, `fixed_costs`, `variable_costs`, `free_cashflow`

## Non-functional

- **Compatibility:** localhost `:18080`, omniflow external profile; OIDC smoke unchanged
- **Testing:** Vitest helper cases; `npm test` frontend; no backend test changes
- **Deploy:** Frontend rebuild only — no migration

## Traceability

- [R-0098](docs/engineering/research.md#r-0098--bug-0026-forecast-monthly-income-card-vs-chart-mismatch)
- `docs/engineering/architecture.md` § **BUG-0026**
- **US-0002**, **DEC-0089** (no new DEC)
