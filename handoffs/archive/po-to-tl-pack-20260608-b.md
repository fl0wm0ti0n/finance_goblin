# PO to TL archive pack (2026-06-08)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 10
- First archived heading: `## architecture-20260608-us0018 — US-0018 category filters & trend analytics (hot pointer)`
- Last archived heading: `## architecture-20260608-us0018 — US-0018 category filters & trend analytics (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=32
  - retained_body_lines=498

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

**Evidence:** `docs/engineering/architecture.md` § **US-0018**, [R-0083](docs/engineering/research.md#r-0083--us-0018-category-filters-expense-series-api--trend-analytics), spec-pack `US-0018-*`

`triad_hot_surface`: architecture --rollover + --check (pending sprint-plan run)

---

