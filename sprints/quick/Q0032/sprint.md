# Q0032 — BUG-0026 Forecast monthly Income card mismatch

| Field | Value |
|-------|-------|
| **ID** | Q0032 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0026 |
| **Created** | 2026-06-13 |
| **Architecture** | `docs/engineering/architecture.md` § BUG-0026 (extends **DEC-0089**, **US-0002**; **GATE-DEC-1** no new DEC) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260613-bug0026-q0032`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0026 rows **BZ**, **CA** |
| **Task count** | 5 mandatory (5/12 under `SPRINT_MAX_TASKS=12`) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0026 on `/forecast` **Monthly**: summary **Income** card shows **0.00** while chart displays non-zero Income bars (~€3000) from July onward. Root cause: cards bind unlabeled `series[0]` (partial current month) while chart plots full series. Fix: pure helper resolves reference month (skip partial zero-income head per **GATE-MONTH-1**), wire **ForecastPage**, shared subtitle per **GATE-LABEL-1**, vitest coverage (**GATE-TEST-1**), operator verify-work (**V1**).

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| BZ — summary month + Income parity (P0) | H1, F1 | `forecastSummaryMonth.ts`, `ForecastPage.tsx` |
| CA — month label (P0) | H1, F1 | shared subtitle above card grid |
| T — Vitest coverage (P0) | T1 | `forecastSummaryMonth.test.ts` |
| Regression + gates | G1, V1 | `npm test`, `npm run build`, uat |

**Ops-only (not execute tasks):** Operator **FRONTEND_DEPLOY** (no migration).

**Out of scope:** `project.rs`, `backend/src/api/forecast.rs`, `MonthlyChart.tsx`, category filter wiring, chart hover sync, backend `summary_month` field, new DEC.

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| H1 | Pure helper `forecastSummaryMonth.ts` | 1h | — | **BZ**, **CA** | P0 |
| F1 | ForecastPage wire + subtitle | 1h | H1 | **BZ**, **CA** | P0 |
| T1 | Vitest partial-month fixture | 1.5h | H1 | **BZ**, **CA** | P0 |
| G1 | Automated gate | 0.5h | F1, T1 | **BZ**, **CA** | P0 |
| V1 | verify-work `/forecast` + OIDC | 1.5h | G1 + deploy | **BZ**, **CA** | P0 |

**Total estimate:** ~4.5h dev + ~1.5h operator V1.

## Deploy order

```text
H1 (forecastSummaryMonth.ts)
  → F1 (ForecastPage useMemo + subtitle)
  → T1 (vitest partial-month fixture)
  → G1 (npm test + build)
  → operator: FRONTEND_DEPLOY (frontend rebuild only)
  → V1 verify-work
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **BZ** | H1, F1, T1, G1, V1 | Skip partial June → July; Income card **3266.16** matches chart July bar on account **114** |
| **CA** | H1, F1, T1, G1, V1 | Subtitle **"Forecast for July 2026"** above four cards; not unlabeled `series[0]` |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| H1 | Task **H1** |
| F1 | Task **F1** |
| T1 | Task **T1** |
| G1 | Task **G1** |
| BZ/CA runtime gates | Task **V1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
