# Sprint S0017

**ID:** S0017  
**Story:** US-0018 — Category filters & expense trend analytics  
**Status:** PLANNED  
**Created:** 2026-06-08  
**Orchestrator:** `auto-20260608-us0018-001`

## Goal

Deliver **DEC-0087** expense-series API with month spine SQL and `__uncategorized__` sentinel; **DEC-0088** shared `CategoryFilter` + bar-default `CategoryTrendChart`; **DEC-0089** cross-surface semantics (forecast actuals-only side panel, planning widget, independent Grafana `$category`); **DEC-0090** index deferral unless EXPLAIN >50 ms; publish `docs/user-guides/US-0018.md`; OIDC smoke template in UAT.

## Scope

| Slice | Tasks | Primary files |
|-------|-------|---------------|
| **US-0018-S1** — Backend category APIs | T-0175 … T-0176 | `transactions/repository.rs`, `api/categories.rs`, `api/mod.rs` |
| **US-0018-S2** — Shared filter + trend chart | T-0177 … T-0178 | `components/category/*`, `ForecastPage.tsx`, `api.ts` |
| **US-0018-S3** — Planning + wealth surfaces | T-0179 … T-0180 | `PlanningPage.tsx`, `WealthPage.tsx` |
| **US-0018-S4** — Grafana `$category` | T-0181 … T-0182 | `cashflow.json`, `budgets.json` |
| **US-0018-S5** — Docs + regression + perf gate | T-0183 … T-0185 | `docs/user-guides/US-0018.md`, `uat.md`, optional migration |

**Out of scope:** Multi-category chart overlay; Grafana↔SPA bidirectional sync; category-scoped forecast re-projection (US-0019); materialized views; Firefly category editing; US-0015 `bucket_sources` / AI inference changes.

## Task table

| ID | Title | Slice | Est. | Acceptance |
|----|-------|-------|------|------------|
| T-0175 | `expense_series_by_month` repository + spine tests | S1 | 4h | AC-2, AC-5 |
| T-0176 | Categories REST routes + service + summary | S1 | 4h | AC-1, AC-2, AC-4, AC-5 |
| T-0177 | `CategoryFilter` + `CategoryTrendChart` components | S2 | 4h | AC-1, AC-3, AC-4 |
| T-0178 | Forecast monthly tab integration | S2 | 3h | AC-1, AC-3, AC-4 |
| T-0179 | Planning compare toolbar + actuals widget | S3 | 3h | AC-1 |
| T-0180 | Wealth category spending subsection | S3 | 2h | AC-1 |
| T-0181 | cashflow `$category` variable + panel | S4 | 3h | AC-1 |
| T-0182 | budgets `$category` + Ist actual filter | S4 | 3h | AC-1 |
| T-0183 | User guide US-0018 | S5 | 2h | — |
| T-0184 | UAT OIDC smoke + AC-1..AC-6 template | S5 | 2h | AC-6 |
| T-0185 | EXPLAIN probe (conditional index) | S5 | 1h | — |

**Total estimate:** ~31h across 11 tasks (T-0185 P2 optional).

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Operators expect category filter to change forecast buckets | Copy/tooltip: household forecast unchanged; trend shows actuals | DEC-0089, T-0178, T-0183 |
| Planning compare confusion (filter vs plan lines) | Widget labeled "Actual spending trend"; compare API unchanged | T-0179, T-0183 |
| Grafana vs SPA category mismatch | Document independent filters in user guide | T-0183 |
| Stale category id post-Firefly delete | Empty series + 404 on unknown id | T-0176 |
| 24-month query slow on large mirrors | DEC-0090 EXPLAIN gate; optional index task T-0185 | T-0185 |
| US-0015 regression | No `project.rs` / bucket_inference changes | T-0184 regression checklist |
| AC-6 operator gate | OIDC smoke pass-with-prerequisites **BACKEND_FRONTEND_DEPLOY** | T-0184 |

## Definition of Done

- All 10 mandatory sprint tasks complete (`T-0175` … `T-0184`; T-0185 conditional)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` § US-0018 (AC-1..AC-6)
- `GET /api/v1/categories` + `/categories/expense-series` return spine months with `__uncategorized__` sentinel (AC-2, AC-5)
- `CategoryTrendChart` renders bar chart with MoM/best/worst callouts from API `summary` (AC-3, AC-4)
- Category filter on forecast, planning, wealth + Grafana cashflow/budgets (AC-1)
- `docs/user-guides/US-0018.md` published (`USER_GUIDE_MODE=1`)
- Operator gate **BACKEND_FRONTEND_DEPLOY** documented before omniflow OIDC smoke (AC-6)

## Architecture references

- `docs/engineering/architecture.md` § US-0018
- `decisions/DEC-0087.md`, `DEC-0088.md`, `DEC-0089.md`, `DEC-0090.md`
- Research: R-0080, R-0083; frozen DEC-0007, DEC-0032
- Spec-pack: `docs/engineering/spec-pack/US-0018-{design-concept,crs,technical-specification}.md`
- User guide: `docs/user-guides/US-0018.md`
- Discovery: `handoffs/po_to_tl.md#architecture-20260608-us0018`
- Acceptance: `docs/product/acceptance.md` § US-0018
- Handoff: `handoffs/tl_to_dev.md` (`sprint-plan-20260608-s0017-us0018`)

## Sequencing (frozen)

```text
S1: T-0175 → T-0176
S2: T-0177 → T-0178 (after S1 — API required)
S3: T-0179, T-0180 (after S2 — shared components)
S4: T-0181, T-0182 (after T-0176 — catalog query for Grafana variable; ∥ S2/S3)
S5: T-0183 → T-0184 (after S2–S4); T-0185 optional after T-0175
Operator: BACKEND_FRONTEND_DEPLOY → verify-work omniflow smoke (UAT)
```

## Acceptance coverage map

| Row | Tasks | Notes |
|-----|-------|-------|
| AC-1 | T-0176, T-0177, T-0178, T-0179, T-0180, T-0181, T-0182, T-0184 | Filter on 4 SPA/Grafana surfaces (forecast, planning, wealth, 2 Grafana) |
| AC-2 | T-0175, T-0176 | Monthly series API; 12 default / 24 max months |
| AC-3 | T-0177, T-0178 | Bar trend chart; empty-state; month labels |
| AC-4 | T-0176, T-0177, T-0178 | Server `summary` MoM + best/worst; chart callouts |
| AC-5 | T-0175, T-0176 | Mirror `category_id`; `__uncategorized__` explicit bucket |
| AC-6 | T-0184 | OIDC smoke; US-0015 bucket mapping unchanged; read-only Firefly |

## Split decision

- **Why 11 tasks:** Architecture C1–C4 + G1–G2 + D1 + V1 + optional P1 = 11 ≤ `SPRINT_MAX_TASKS` 12.
- **Why not split S0017a/b:** Single vertical slice with S1→S2 dependency; Grafana (S4) parallelizes after catalog API.
- **P1 optional:** T-0185 gated on EXPLAIN >50 ms — does not block MVP if probe passes.
- **User guide in D1:** `USER_GUIDE_MODE=1` — separate task avoids bundling with V1 (mirrors S0016 pattern).

## Next phase

`/plan-verify` in fresh subagent/chat (role: qa)
