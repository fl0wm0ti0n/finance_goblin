# Q0032 Summary — BUG-0026 Forecast monthly Income card mismatch

**Sprint:** Q0032 (`/quick`)  
**Bug:** BUG-0026  
**Orchestrator:** `auto-20260613-bug0026`  
**Phase:** execute **COMPLETE**  
**Date:** 2026-06-13

## Goal

Close BUG-0026 on `/forecast` **Monthly**: summary cards currently bind to unlabeled `series[0]` (partial June with **0.00** Income) while **MonthlyChart** plots the full series including July salary (~**3266**). Fix is frontend-only — resolve a labeled reference month for cards and show shared subtitle above the card grid.

## Tasks

| ID | Title | Status | Acceptance | Priority |
|----|-------|--------|------------|----------|
| H1 | Pure helper `forecastSummaryMonth.ts` | **done** | **BZ**, **CA** | P0 |
| F1 | ForecastPage wire + subtitle | **done** | **BZ**, **CA** | P0 |
| T1 | Vitest partial-month fixture | **done** | **BZ**, **CA** | P0 |
| G1 | Automated gate (`npm test`, `npm run build`) | **done** | **BZ**, **CA** | P0 |
| V1 | verify-work `/forecast` BZ/CA + OIDC smoke | open | **BZ**, **CA** | P0 |

**Task count:** 5 mandatory (5/12 under `SPRINT_MAX_TASKS=12`; no split).

## Implementation summary

- **H1:** `resolveForecastSummaryPoint`, `formatForecastMonthLabel`, `formatForecastSummarySubtitle` in `forecastSummaryMonth.ts` per **GATE-MONTH-1** / **GATE-LABEL-1**
- **F1:** `ForecastPage.tsx` wires resolved point; subtitle above card grid; **DEC-0089** — category filter unchanged on cards
- **T1:** 7 vitest cases including partial-month trap (June 0 → July 3266.16)
- **G1:** npm **24/24** PASS; build PASS

## Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **BZ** | H1, F1, T1, G1, V1 | Skip partial June → July point; Income card **3266.16** matches chart July bar; not **0.00** with unexplained chart bars |
| **CA** | H1, F1, T1, G1, V1 | Subtitle **"Forecast for July 2026"** above four cards; not unlabeled `series[0]` |

## Frozen boundaries

- **GATE-SCOPE-1:** frontend-only — no `project.rs` / API contract change
- **DEC-0089:** category filter unchanged on cards
- **GATE-DEC-1:** no new DEC
- **Out of scope:** `MonthlyChart.tsx`, chart hover sync, backend `summary_month` field

## Operator gate

**FRONTEND_DEPLOY** — frontend rebuild only (no migration) before V1 runtime smoke on account **114**.

## Next phase

**`/qa`** — fresh subagent (role: qa).
