# PO to TL archive pack (2026-06-08)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## discovery-20260608-us0018 — US-0018 category filters & expense trend analytics discovery (hot pointer)`
- Last archived heading: `## discovery-20260608-us0018 — US-0018 category filters & expense trend analytics discovery (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=50
  - retained_body_lines=496

---

## discovery-20260608-us0018 — US-0018 category filters & expense trend analytics discovery (hot pointer)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-08  
**Story:** US-0018  
**Orchestrator run:** `auto-20260608-us0018-001`  
**Intake bundle:** `intake-20260607-category-planning-subscriptions`  
**Next phase:** `/research`

### Summary

**US-0018** delivers shared **category filter contract** (React + API) on forecast monthly, planning compare, wealth breakdown, and Grafana **cashflow** + **budgets**, plus **monthly per-category expense series** API and **trend chart** with MoM insight. **Single-category MVP**; multi-overlay and Grafana↔SPA sync deferred. Builds on **[R-0080](docs/engineering/research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake)** + mirror ingest (BUG-0006).

### Surface map (canonical)

| Surface | Filter placement |
|---------|------------------|
| `/forecast` monthly | `CategoryFilter` above monthly chart |
| `/planning` compare | Filter in compare toolbar |
| `/wealth` overview | Filter above category subsection |
| Grafana `cashflow`, `budgets` | `$category` variable + scoped panels |

### API draft (research validates)

- `GET /api/v1/categories` — mirror catalog
- `GET /api/v1/categories/expense-series` — monthly EUR outflow/inflow (12–24 mo); uncategorized explicit

### Decision gates (PO recommendation)

| Topic | Recommendation |
|-------|----------------|
| Multi-category chart | Defer — single series (AC-3) |
| Trend chart home | Forecast monthly tab |
| Forecast filter depth | Display/breakdown MVP; engine projection → architecture |

### Research carry (extends R-0080)

Monthly per-category SQL; DEC-0007 bucket ↔ `category_id` join; Grafana `$category` pattern; category catalog pagination; bar vs line default.

### Artifacts updated

- `docs/product/vision.md` US-0018 discovery, `docs/product/backlog.md#US-0018`, `docs/engineering/state.md` isolation + runtime proof

**Full handoff:** `handoffs/archive/po-to-tl-pack-20260608.md`

`triad_hot_surface`: rollover archived full discovery → `handoffs/archive/po-to-tl-pack-20260608.md`; hot pointer prepended; retained_body_lines=496; --check PASS (2026-06-08T18:52:53Z)

---

