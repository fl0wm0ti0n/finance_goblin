# CRS — BUG-0026 Forecast monthly Income card vs chart mismatch

## Purpose

Restore operator trust on `/forecast` **Monthly** by selecting a meaningful reference month for summary cards and labeling it explicitly. Partial current month with zero projected salary misleads when chart shows non-zero Income bars from the next month.

## Scope

### In scope

- **H1:** `forecastSummaryMonth.ts` — `resolveForecastSummaryPoint`, `formatForecastMonthLabel`, `formatForecastSummarySubtitle`
- **F1:** `ForecastPage.tsx` — wire helper into `monthlySummary` useMemo; subtitle above card grid
- **T1:** Vitest `forecastSummaryMonth.test.ts` with partial-month fixture
- **G1:** `npm test` + `npm run build` gate
- **V1:** verify-work BZ/CA on `/forecast` Monthly; OIDC smoke

### Out of scope

- `project.rs`, `backend/src/api/forecast.rs`
- `MonthlyChart.tsx` changes
- Category filter wiring to cards (**DEC-0089**)
- Chart hover/selection sync
- Optional P2 partial-month footnote

## Acceptance criteria ref

`docs/product/acceptance.md` — BUG-0026:

- **(BZ)** Summary **Income** card consistent with monthly bar chart for the same labeled reference month — not **0.00** vs ~€3000 chart bars
- **(CA)** Summary cards show which month they represent — not unlabeled `series[0]` when misleading; OIDC regression pass

## Dependencies

- **US-0002** (forecast monthly view)
- **DEC-0089** (category filter scopes trend chart only — cards unchanged)
- [R-0098](docs/engineering/research.md#r-0098--bug-0026-forecast-monthly-income-card-vs-chart-mismatch)
