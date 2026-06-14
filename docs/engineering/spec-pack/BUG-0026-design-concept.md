# Design Concept — BUG-0026

## Summary

BUG-0026 fixes a trust gap on `/forecast` **Monthly**: summary cards bind to unlabeled `series[0]` (partial current month with **0.00** Income) while **MonthlyChart** plots the full series including salary from the next month. Fix is frontend-only — resolve a labeled reference month for cards and show a shared subtitle above the card grid.

## Goals

- **BZ:** Income card consistent with chart for the same labeled reference month — not **0.00** card vs ~€3000 chart bars
- **CA:** Summary cards show which month they represent via shared subtitle **"Forecast for {Month YYYY}"**
- Vitest coverage for month-selection helper and partial-month fixture
- OIDC-enabled deploy regression pass

## Non-goals

- Backend `project.rs` or forecast API contract changes
- Chart hover/selection sync with cards
- Category filter changes (**DEC-0089** cards remain independent)
- New DEC record
- Playwright E2E (defer to verify-work / operator smoke)

## Key decisions

| Gate | Choice | Rationale |
|------|--------|-----------|
| GATE-MONTH-1 | Skip partial head when `series[0].income === 0`; else first income>0 or `series[1]` | Aligns cards with first meaningful Income bar |
| GATE-LABEL-1 | Shared subtitle above card grid | One period label for all four metrics |
| GATE-SCOPE-1 | Frontend-only | API already returns correct ordered series |
| GATE-TEST-1 | Vitest pure helper | `planSelector.test.ts` precedent |
| GATE-DEC-1 | No new DEC | Architecture documents summary month contract |

**Spec-pack traceability:** `docs/engineering/spec-pack/BUG-0026-crs.md`, `docs/engineering/spec-pack/BUG-0026-technical-specification.md`
