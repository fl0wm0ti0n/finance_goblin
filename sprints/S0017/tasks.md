# Tasks — Sprint S0017

**Story:** US-0018  
**Task count:** 11 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Slice | Est. | Acceptance refs |
|----|-------|--------|-------|------|-----------------|
| T-0175 | `expense_series_by_month` repository + spine tests | open | S1 | 4h | AC-2, AC-5 |
| T-0176 | Categories REST routes + service + summary | open | S1 | 4h | AC-1, AC-2, AC-4, AC-5 |
| T-0177 | `CategoryFilter` + `CategoryTrendChart` components | open | S2 | 4h | AC-1, AC-3, AC-4 |
| T-0178 | Forecast monthly tab integration | open | S2 | 3h | AC-1, AC-3, AC-4 |
| T-0179 | Planning compare toolbar + actuals widget | open | S3 | 3h | AC-1 |
| T-0180 | Wealth category spending subsection | open | S3 | 2h | AC-1 |
| T-0181 | cashflow `$category` variable + panel | open | S4 | 3h | AC-1 |
| T-0182 | budgets `$category` + Ist actual filter | open | S4 | 3h | AC-1 |
| T-0183 | User guide US-0018 | open | S5 | 2h | — |
| T-0184 | UAT OIDC smoke + AC-1..AC-6 template | open | S5 | 2h | AC-6 |
| T-0185 | EXPLAIN probe (conditional index) | open | S5 | 1h | — |

---

## T-0175 — `expense_series_by_month` repository + spine tests

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0087, DEC-0090  
**Architecture slice:** US-0018-S1  
**Research:** R-0083

### Description

Add `TransactionsRepository::expense_series_by_month` in `backend/src/transactions/repository.rs`:

| Aspect | Contract |
|--------|----------|
| Spine | `generate_series` month spine + `LEFT JOIN transactions` on `date_trunc('month', t.date)` |
| Outflow | `ABS(amount)` where `amount < 0` |
| Inflow | `amount > 0` where scoped |
| Window | `$end = end param or today`; `$start = date_trunc('month', $end) - (months-1) * 1 month` |
| Zeros | Explicit €0 months in window (not data min/max) |
| `category_id` | Firefly mirror id **or** `__uncategorized__` → `t.category_id IS NULL` |
| Cap | `months` default 12, max 24 |

Unit/integration tests: spine zeros, `__uncategorized__` bucket metadata, 24-month cap rejection, invalid window.

### Done when

- [ ] Month spine returns all calendar months in window with zero fill
- [ ] `__uncategorized__` filter uses `IS NULL` and returns bucket metadata
- [ ] 24-month cap enforced; months >24 rejected
- [ ] `cargo test` repository spine tests green

---

## T-0176 — Categories REST routes + service + summary

**Status:** open  
**Depends on:** T-0175  
**Decisions:** DEC-0087  
**Architecture slice:** US-0018-S1  
**Research:** R-0083

### Description

Add `backend/src/api/categories.rs` and wire in `api/mod.rs`:

| Method | Path | Contract |
|--------|------|----------|
| `GET` | `/api/v1/categories` | `{ categories: [{id, name}], truncated?: bool }`; optional `?q=` (`MIN_CATEGORY_SEARCH_LEN=2`); cap 200 |
| `GET` | `/api/v1/categories/expense-series` | Query: `category_id` (required), `months` (12 default, 24 max), optional `end` |

Response includes `months[]`, `category_name`, `uncategorized` flag, `transaction_count`, and server-side `summary`:

| Summary field | Rule |
|---------------|------|
| `mom_delta_pct` | Last two calendar months in window |
| `best_month` | Max `outflow_eur` |
| `worst_month` | Min outflow among active months |

Unknown `category_id` → **404**. Never return uncategorized series without bucket metadata (AC-5).

### Done when

- [ ] Both endpoints registered and return documented JSON shapes
- [ ] Catalog sorted by name; search + truncation behave per contract
- [ ] `summary` computed server-side on full spine
- [ ] `cargo test` categories API tests green

---

## T-0177 — `CategoryFilter` + `CategoryTrendChart` components

**Status:** open  
**Depends on:** T-0176  
**Decisions:** DEC-0088  
**Architecture slice:** US-0018-S2  
**Research:** R-0083

### Description

Create `frontend/src/components/category/`:

**CategoryFilter** — single-select MVP; sentinel `""` = All categories; `__uncategorized__` option; data from `GET /api/v1/categories`; combobox when >20 categories.

**CategoryTrendChart** — ECharts bar default; data from `expense-series`; month label + `outflow_eur`; empty state when `transaction_count=0`; disabled prompt when no category selected; MoM/best/worst callouts from API `summary` (AC-4).

Extend `frontend/src/lib/api.ts` with `fetchCategories` + `fetchCategoryExpenseSeries`.

### Done when

- [ ] Components render in isolation with mocked API data
- [ ] Bar chart shows month labels with EUR amounts (AC-3)
- [ ] Empty-state and disabled-state copy match architecture
- [ ] Summary callouts display MoM + best/worst months
- [ ] `npm test` component smoke passes if applicable

---

## T-0178 — Forecast monthly tab integration

**Status:** open  
**Depends on:** T-0177  
**Decisions:** DEC-0088, DEC-0089  
**Architecture slice:** US-0018-S2

### Description

Integrate into `ForecastPage.tsx` **Monthly** tab:

- `CategoryFilter` above stat cards
- `CategoryTrendChart` below `MonthlyChart` when category selected
- Household stat cards + `MonthlyChart` remain **unchanged** (DEC-0007 / DEC-0089)
- Tooltip/copy: filter scopes **actual spending trend** only — not forecast buckets

### Done when

- [ ] Filter + chart visible on Monthly tab
- [ ] Selecting category loads expense-series and renders trend
- [ ] Household forecast cards/charts unchanged when filter applied
- [ ] No regression to US-0015 AI-mapped badge behavior

---

## T-0179 — Planning compare toolbar + actuals widget

**Status:** open  
**Depends on:** T-0177  
**Decisions:** DEC-0089  
**Architecture slice:** US-0018-S3

### Description

Add to `PlanningPage.tsx` compare context:

- Toolbar `CategoryFilter`
- Adjacent `CategoryTrendChart` labeled **Actual spending trend**
- Compare metrics API / version table **unchanged** — no `category_id` query param

### Done when

- [ ] Filter + chart widget visible on planning compare view
- [ ] Compare table/metrics unaffected by category selection
- [ ] Widget copy distinguishes actuals from plan compare lines

---

## T-0180 — Wealth category spending subsection

**Status:** open  
**Depends on:** T-0177  
**Decisions:** DEC-0089  
**Architecture slice:** US-0018-S3

### Description

Add "Category spending" subsection to `WealthPage.tsx`:

- `CategoryFilter` + period total from expense-series
- Link or embedded `CategoryTrendChart` for selected category
- Net worth / crypto totals remain household-level

### Done when

- [ ] Subsection renders with filter + trend for selected category
- [ ] Household wealth totals unchanged
- [ ] Empty-state when no categorized rows in period

---

## T-0181 — cashflow `$category` variable + panel

**Status:** open  
**Depends on:** T-0176  
**Decisions:** DEC-0089  
**Architecture slice:** US-0018-S4  
**Research:** R-0083

### Description

Update `grafana/provisioning/dashboards/analytics/cashflow.json`:

- Add `$category` query variable (All + categories from mirror)
- New panel: monthly category outflow (`date_trunc` + sum abs negative)
- Panel filter: `AND ('${category}' = '' OR t.category_id = '${category}')`
- Default `category=''` preserves pre-US-0018 behavior

### Done when

- [ ] `$category` variable appears in dashboard
- [ ] Category panel filters correctly when variable set
- [ ] Empty variable = all categories (no regression)

---

## T-0182 — budgets `$category` + Ist actual filter

**Status:** open  
**Depends on:** T-0176  
**Decisions:** DEC-0089  
**Architecture slice:** US-0018-S4

### Description

Update `grafana/provisioning/dashboards/analytics/budgets.json`:

- Add `$category` variable (same contract as cashflow)
- Extend Ist/deviation **actual** CTE with category filter
- Planned leg remains household-only

### Done when

- [ ] `$category` variable on budgets dashboard
- [ ] Actual/deviation panels respect category filter
- [ ] Planned budget leg unchanged when category selected

---

## T-0183 — User guide US-0018

**Status:** open  
**Depends on:** T-0178, T-0179, T-0180, T-0181, T-0182  
**Decisions:** DEC-0088, DEC-0089, DEC-0059  
**Architecture slice:** US-0018-S5  
**Research:** R-0083

### Description

Publish `docs/user-guides/US-0018.md` (`USER_GUIDE_MODE=1`):

| Section | Content |
|---------|---------|
| Purpose | Category-scoped spending visibility across surfaces |
| Forecast | Filter shows actuals trend; household forecast unchanged |
| Planning | Actuals widget vs compare table semantics |
| Wealth | Category spending subsection |
| Grafana | Independent `$category` on cashflow/budgets |
| Uncategorized | `__uncategorized__` bucket behavior |
| Troubleshooting | Stale category id → empty series; 404 guidance |

Cross-link forecast user guide (`docs/user-guides/US-0002.md`).

### Done when

- [ ] User guide file created with operator-facing content
- [ ] DEC-0089 semantics documented (forecast/planning/Grafana independence)
- [ ] `__uncategorized__` behavior explained

---

## T-0184 — UAT OIDC smoke + AC-1..AC-6 template

**Status:** open  
**Depends on:** T-0183  
**Decisions:** DEC-0087, DEC-0089  
**Architecture slice:** US-0018-S5

### Description

Populate `sprints/S0017/uat.md` and extend `uat.json` with OIDC smoke checklist:

| Step | Contract |
|------|----------|
| Profile | US-0010 external |
| Routes | `/forecast` Monthly, `/planning` compare, `/wealth`, Grafana cashflow + budgets |
| Gate | **BACKEND_FRONTEND_DEPLOY** |
| Regression | US-0015 `bucket_sources` / AI badge unchanged; read-only Firefly; no `project.rs` changes |

### Done when

- [ ] UAT template lists all 6 acceptance criteria with task refs
- [ ] OIDC smoke steps documented per surface (AC-1, AC-6)
- [ ] US-0015 regression scope explicit in checklist

---

## T-0185 — EXPLAIN probe (conditional index)

**Status:** open  
**Depends on:** T-0175  
**Decisions:** DEC-0090  
**Architecture slice:** US-0018-S5  
**Priority:** P2 optional

### Description

During execute on operator mirror (or integration test DB with representative row count):

- Run `EXPLAIN ANALYZE` on expense-series query for single `category_id`, 24-month window
- If **>50 ms**, add migration: `CREATE INDEX idx_transactions_category_date ON transactions (category_id, date)`
- If ≤50 ms, document probe result in task notes — no migration

### Done when

- [ ] EXPLAIN result recorded in sprint notes or task completion evidence
- [ ] Index migration added only if gate triggered
- [ ] No index shipped if probe passes ≤50 ms threshold

---

## Execution order (recommended)

1. **S1 API:** T-0175 → T-0176
2. **S2 components:** T-0177 → T-0178
3. **S3 surfaces:** T-0179, T-0180 (parallel after T-0177)
4. **S4 Grafana:** T-0181, T-0182 (parallel after T-0176)
5. **S5 docs/regression:** T-0183 → T-0184; T-0185 optional after T-0175
6. **Operator:** BACKEND_FRONTEND_DEPLOY → UAT omniflow smoke

```text
T-0175 → T-0176
  ↓         ↓
T-0177   T-0181, T-0182
  ↓
T-0178, T-0179, T-0180
  ↓
T-0183 → T-0184
T-0185 (optional, after T-0175)
```

## Acceptance coverage map

| AC | Tasks | Notes |
|----|-------|-------|
| AC-1 | T-0176, T-0177, T-0178, T-0179, T-0180, T-0181, T-0182, T-0184 | Four SPA/Grafana surfaces |
| AC-2 | T-0175, T-0176 | Monthly series API |
| AC-3 | T-0177, T-0178 | Bar trend chart + empty-state |
| AC-4 | T-0176, T-0177, T-0178 | Server summary + UI callouts |
| AC-5 | T-0175, T-0176 | `__uncategorized__` sentinel |
| AC-6 | T-0184 | OIDC smoke; US-0015 unchanged |

## Operator gate

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_FRONTEND_DEPLOY** | UAT AC-6 omniflow smoke | Deploy S1–S4 backend + frontend on US-0010 external profile |

## Split decision

- **Why 11 tasks:** Architecture task table C1–C4 + G1–G2 + D1 + V1 + optional P1; under `SPRINT_MAX_TASKS` 12.
- **P1 optional:** T-0185 does not block MVP if EXPLAIN ≤50 ms.
- **S1 before S2:** API required for chart data.
- **Grafana parallel:** After catalog API (T-0176) for variable query.
