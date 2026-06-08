# PO to TL archive pack (2026-06-08)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 10
- First archived heading: `## sprint-plan-20260608-us0018 — US-0018 category filters & trend analytics (hot pointer)`
- Last archived heading: `## architecture-20260608-us0018 — US-0018 category filters & trend analytics (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=60
  - retained_body_lines=498

---

## sprint-plan-20260608-us0018 — US-0018 category filters & trend analytics (hot pointer)

**From:** Tech Lead  
**To:** Plan-verify  
**Date:** 2026-06-08  
**Story:** US-0018  
**Orchestrator run:** `auto-20260608-us0018-001`  
**Next phase:** `/plan-verify` (role: qa)

### Summary

Sprint **S0017** materialized — **11 tasks** (T-0175..T-0185) mapped to AC-1..AC-6 and architecture slices S1–S5; 11/12 under `SPRINT_MAX_TASKS`; no split.

| Slice | Tasks | AC |
|-------|-------|-----|
| S1 API | T-0175, T-0176 | AC-2, AC-4, AC-5 |
| S2 Filter+chart | T-0177, T-0178 | AC-1, AC-3, AC-4 |
| S3 Surfaces | T-0179, T-0180 | AC-1 |
| S4 Grafana | T-0181, T-0182 | AC-1 |
| S5 Docs/regression | T-0183, T-0184, T-0185 (optional) | AC-6, user guide |

**Decisions:** DEC-0087, DEC-0088, DEC-0089, DEC-0090  
**Evidence:** `sprints/S0017/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260608-s0017-us0018`)

`triad_hot_surface`: sprint-plan prepended; --rollover units=2,1 + --check PASS (2026-06-08T21:00:00Z)

---

## architecture-20260608-us0018 — US-0018 category filters & trend analytics (hot pointer)

**From:** Tech Lead  
**To:** Sprint-plan  
**Date:** 2026-06-08  
**Story:** US-0018  
**Orchestrator run:** `auto-20260608-us0018-001`  
**Next phase:** `/sprint-plan`

### Summary

Architecture resolves all **7 R-0083 gates**: single-category bar trend chart; independent Grafana `$category`; forecast **actuals-only** side panel (household DEC-0007 unchanged); planning compare **UI widget**; `__uncategorized__` sentinel; index **defer** unless EXPLAIN >50 ms.

### Decisions

| ID | Contract |
|----|----------|
| **DEC-0087** | Month spine SQL; `GET /categories` + `/categories/expense-series`; `__uncategorized__`; server `summary` |
| **DEC-0088** | `CategoryFilter` single-select; `CategoryTrendChart` bar default |
| **DEC-0089** | Forecast/planning/wealth semantics; Grafana independence |
| **DEC-0090** | Optional index migration on EXPLAIN gate |

### Sprint-plan input

10–11 tasks (C1–C4, C5–C6, G1–G2, D1, V1, optional P1) → **S0017** ≤ `SPRINT_MAX_TASKS` 12.

**Evidence:** `docs/engineering/architecture.md` § **US-0018**, [R-0083](docs/engineering/research.md#r-0083--us-0018-category-filters-expense-series-api--trend-analytics), spec-pack `US-0018-*`, archive `handoffs/archive/po-to-tl-pack-20260608-b.md`

`triad_hot_surface`: architecture --rollover units=2,1,1 + --check PASS + heading policy baseline_h2=14 PASS (2026-06-08T20:00:00Z)

---

