# Tasks — Q0032 (BUG-0026)

**Bug:** BUG-0026  
**Task count:** 5 mandatory (5/12 under `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260613-bug0026-q0032`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **H1** | Task **H1** | `resolveForecastSummaryPoint`, label helpers |
| **F1** | Task **F1** | ForecastPage useMemo + subtitle above card grid |
| **T1** | Task **T1** | Vitest partial-month trap fixture |
| **G1** | Task **G1** | `npm test`, `npm run build` |
| **BZ/CA runtime** | Task **V1** | verify-work after FRONTEND_DEPLOY |

## Execute order

```text
H1 → F1 → T1 → G1
  → operator: FRONTEND_DEPLOY
  → V1 verify-work
```

**Parallelism:** F1 and T1 both depend on H1; G1 blocked on F1 + T1; V1 blocked on G1 + deploy.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **BZ** | H1, F1, T1, G1, V1 | Income card matches chart for labeled reference month — not 0.00 with unexplained ~€3000 chart bars |
| **CA** | H1, F1, T1, G1, V1 | Subtitle names reference month; not unlabeled `series[0]` |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| H1 | Pure helper forecastSummaryMonth.ts | 1h | **done** | **BZ**, **CA** | P0 |
| F1 | ForecastPage wire + subtitle | 1h | **done** | **BZ**, **CA** | P0 |
| T1 | Vitest partial-month fixture | 1.5h | **done** | **BZ**, **CA** | P0 |
| G1 | Automated gate | 0.5h | **done** | **BZ**, **CA** | P0 |
| V1 | verify-work `/forecast` + OIDC | 1.5h | open | **BZ**, **CA** | P0 |

---

## H1 — Pure helper forecastSummaryMonth.ts

**Status:** **done**  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0026 **BZ**, **CA** — **GATE-MONTH-1**

### Description

Create `frontend/src/pages/forecastSummaryMonth.ts` with frozen contract:

- `resolveForecastSummaryPoint(series)` — skip `series[0]` when `parseFloat(income) === 0` and `series.length > 1`; else `series.find(p => parseFloat(p.income) > 0) ?? series[1]`; empty → `null`
- `formatForecastMonthLabel(monthIso)` — locale month-year from API `month` ISO date slice
- `formatForecastSummarySubtitle(monthIso)` — `Forecast for {Month YYYY}`

Mirror **BUG-0022** `planSelector.ts` colocated-helper pattern.

**Files:** `frontend/src/pages/forecastSummaryMonth.ts` (new)

### Done when

- [ ] All three exports implemented per frozen algorithm
- [ ] `parseFloat` on income strings (same as `MonthlyChart.tsx`)
- [ ] Month label derived from API `month` string, not client clock

### Verification

Typecheck + importable from ForecastPage.

---

## F1 — ForecastPage wire + subtitle

**Status:** **done**  
**Depends on:** H1  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0026 **BZ**, **CA** — **GATE-LABEL-1**, **GATE-SCOPE-1**

### Description

In `frontend/src/pages/ForecastPage.tsx`:

1. Replace `monthlySummary = series[0]` useMemo with `resolveForecastSummaryPoint(monthlyQuery.data?.series ?? [])`
2. Render `formatForecastSummarySubtitle(monthlySummary.month)` immediately **above** `.grid` card block (L312–330)
3. Card values (Income, Fixed, Variable, Free cashflow) from **same** resolved point
4. **Do not** add `categoryId` to `monthlyQuery` key or card data path (**DEC-0089**)
5. Leave `MonthlyChart` unchanged — still plots full `series`

**Files:** `frontend/src/pages/ForecastPage.tsx` (L148–152 useMemo; L312–330 card grid + subtitle)

### Done when

- [ ] Cards use resolved point not raw `series[0]`
- [ ] Shared subtitle visible above four cards
- [ ] Category filter unchanged on cards
- [ ] No backend or MonthlyChart changes

### Verification

Visual on `/forecast` Monthly after deploy; vitest covers helper contract.

---

## T1 — Vitest partial-month fixture

**Status:** **done**  
**Depends on:** H1  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0026 **BZ**, **CA** — **GATE-TEST-1**, **R-0098**

### Description

Add `frontend/src/pages/forecastSummaryMonth.test.ts` with frozen fixture:

```typescript
const partialMonthTrap = [
  { month: "2026-06-01", income: "0.00", fixed_costs: "86.02", variable_costs: "2866.57", free_cashflow: "-2952.59" },
  { month: "2026-07-01", income: "3266.16", fixed_costs: "86.02", variable_costs: "2866.57", free_cashflow: "313.57" },
];
```

| Case | Expected |
|------|----------|
| Partial-month trap | Resolve index **1**; income **"3266.16"** |
| `series[0].income > 0` | Resolve index **0** |
| All-zero income (multi-month) | Resolve index **0** |
| Single-month series | Resolve index **0** |
| Empty series | `null` |
| `formatForecastMonthLabel("2026-07-01")` | Contains **July** and **2026** |
| `formatForecastSummarySubtitle("2026-07-01")` | **`Forecast for July 2026`** |

**Files:** `frontend/src/pages/forecastSummaryMonth.test.ts` (new)

### Done when

- [ ] All vitest cases PASS
- [ ] No backend test changes
- [ ] Regression suite `npm test` green

### Verification

`npm test forecastSummaryMonth` → all PASS.

---

## G1 — Automated gate

**Status:** **done**  
**Depends on:** F1, T1  
**Estimate:** 0.5h  
**Acceptance hook:** BUG-0026 **BZ**, **CA** — automated verification

### Description

Run and record automated checks in `sprints/quick/Q0032/progress.md`:

1. `npm test` → PASS (includes forecastSummaryMonth + existing suites).
2. `npm run build` → PASS.
3. `git diff --stat` blast radius matches frozen file list (frontend only).

**Files:** `sprints/quick/Q0032/progress.md`

### Done when

- [ ] All automated checks PASS, recorded in progress.md
- [ ] No forbidden paths touched (backend forecast API, MonthlyChart, project.rs)

### Verification

Test output pasted in progress.md; diff stat confirms scope.

---

## V1 — verify-work `/forecast` + OIDC smoke

**Status:** open  
**Depends on:** G1 + operator FRONTEND_DEPLOY  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0026 **BZ**, **CA**

### Description

Populate `sprints/quick/Q0032/uat.md` and `uat.json` after deploy on
localhost:18080 (and optional omniflow OIDC):

1. **BZ-UI** — `/forecast` **Monthly** account **114**: Income card ~**3266.16** matches July chart Income bar; not **0.00**.
2. **BZ-API** — `GET /api/v1/forecast/monthly?account_id=114` — `series[1]` income **3266.16**.
3. **CA-UI** — Subtitle **"Forecast for July 2026"** above four summary cards.
4. **DEC-0089** — Category filter **Wohnen - Stromkosten** does not change card values.
5. **OIDC-1** — `/forecast`, `/api/v1/forecast/monthly` smoke on omniflow profile.

**Files:** `sprints/quick/Q0032/uat.md`, `sprints/quick/Q0032/uat.json`

### Done when

- [ ] Rows **BZ**, **CA** probed per acceptance.md matrix
- [ ] Regression gates documented
- [ ] `uat.md` and `uat.json` populated with results

**Operator gate:** **FRONTEND_DEPLOY** — frontend rebuild only (no migration).
