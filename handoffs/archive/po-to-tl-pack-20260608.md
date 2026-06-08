# PO to TL archive pack (2026-06-08)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## discovery-20260608-us0018 — US-0018 category filters & expense trend analytics discovery`
- Last archived heading: `## discovery-20260608-us0018 — US-0018 category filters & expense trend analytics discovery`
- Verification tuple (mandatory):
  - archived_body_lines=93
  - retained_body_lines=496

---

## discovery-20260608-us0018 — US-0018 category filters & expense trend analytics discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-08  
**Story:** US-0018  
**Orchestrator run:** `auto-20260608-us0018-001`  
**Intake bundle:** `intake-20260607-category-planning-subscriptions`  
**Next phase:** `/research`

### Summary

Discovery refined **US-0018** as the **category analytics foundation** for the intake bundle: a shared **category filter contract** (React + API) on forecast monthly, planning compare, wealth breakdown, and Grafana **cashflow** + **budgets**, plus a **monthly per-category expense series** API and **trend chart** with MoM performance insight. Builds on intake **[R-0080](docs/engineering/research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake)** and existing mirror ingest (BUG-0006). **Single-category MVP** recommended; multi-overlay deferred.

### Surface → filter map (canonical)

| Surface | Route / uid | Filter UX | MVP behavior |
|---------|-------------|-----------|--------------|
| Forecast monthly | `/forecast` | `CategoryFilter` above monthly chart | Scopes monthly breakdown display; forecast re-projection by category → architecture |
| Planning compare | `/planning` | Filter in compare toolbar | Compare/PVA respects `category_id`; none = household |
| Wealth breakdown | `/wealth` | Filter above new category subsection | Period totals + link to trend |
| Grafana cashflow | `cashflow` | `$category` variable | New monthly category outflow panel |
| Grafana budgets | `budgets` | `$category` variable | Category-scoped Ist/deviation when set |

### API contract (discovery draft — research validates)

| Endpoint | Purpose |
|----------|---------|
| `GET /api/v1/categories` | Mirror catalog (`firefly_id`, `name`); sorted; search `?q=` optional |
| `GET /api/v1/categories/expense-series` | Params: `category_id`(s), `months` (default 12, max 24), `start`/`end` optional; returns per-month EUR outflow/inflow, uncategorized bucket explicit |
| Shared query param | `category_id` on forecast monthly, planning compare, wealth breakdown reads |

### Partial implementation review

| Area | Status |
|------|--------|
| `transactions.category_id` + `categories` mirror | **Done** (BUG-0006) |
| `aggregates_by_category` (period, not monthly) | **Done** — AI tool path only |
| Public REST transactions/category routes | **Missing** (`api/mod.rs` has no category routes) |
| `CategoryFilter` + `CategoryTrendChart` | **Missing** |
| Planning `category` target enum | **Present** — compare filter missing |
| Grafana `$category` on cashflow/budgets | **Missing** |
| US-0011 `/analytics/:slug` embed | **Done** — category panels added at execute |
| US-0015 bucket mapping | **Done** — AC-6 regression guard |

### Discovery decomposition evidence

- Feature/workflow count: filter component + 2 APIs + 3 SPA surfaces + 2 Grafana dashboards + trend chart (moderate — **single story retained**)
- Cross-cutting impact: `backend/transactions`, `backend/api`, `frontend` forecast/planning/wealth, `grafana/provisioning/dashboards/analytics/{cashflow,budgets}.json`
- Acceptance breadth: 6 AC unchanged (`docs/product/acceptance.md#US-0018`)
- Risk surface: monthly per-category SQL performance; DEC-0007 forecast bucket ↔ `category_id` join; uncategorized semantics; Grafana variable empty state; OIDC regression (AC-6)

### Decision gates (PO recommendation)

| Topic | Recommendation | Alternative |
|-------|--------------|-------------|
| Multi-category chart | **Defer** — single series satisfies AC-3 | ≤3 overlay series (stretch) |
| Trend chart primary home | **Forecast monthly tab** | Dedicated `/analytics/category-trends` only |
| Grafana ↔ SPA sync | **Independent filters MVP** | iframe `category_id` URL sync |
| Forecast filter depth | Display/breakdown first; engine category projection in architecture | Block on forecast fork |

### Open questions (carry to research)

| Topic | Question |
|-------|----------|
| **Monthly series SQL** | `date_trunc('month')` + `category_id` GROUP BY vs materialized view; uncategorized as `NULL` bucket row |
| **Category catalog** | Paginate vs full list; min search length (reuse `MIN_CATEGORY_SEARCH_LEN`?) |
| **Forecast join** | How DEC-0007 buckets map to `category_id` for filtered monthly forecast display |
| **Planning compare** | Filter actuals only vs planned deltas by category target_type |
| **Wealth subsection** | New table vs extend existing firefly breakdown card |
| **Grafana panels** | New panel on cashflow vs extend monthly decomposition; budgets Ist filter SQL |
| **Chart default** | Bar vs line for MoM readability (Finanzguru-like) |
| **Performance** | 24-month × single category acceptable on 900+ tx mirror without index change? |

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#US-0018` | Discovery refinements + surface map + API draft | pass |
| `docs/product/acceptance.md` US-0018 | 6 criteria still valid; no AC rewrite | pass |
| `docs/product/vision.md` US-0018 discovery | UX references + principles + partial impl | pass |
| `backend/src/transactions/repository.rs` | Period aggregates exist; monthly per-category gap documented | pass |
| `grafana/.../cashflow.json`, `budgets.json` | No `$category` yet; target dashboards identified | pass |
| R-0080 | Intake research linked; discovery extends with surface map | pass |

### Recommended next steps

1. `/research` — Monthly per-category SQL spike; category catalog endpoint; forecast bucket join; Grafana `$category` variable pattern (extends R-0080, R-0008, R-0016 deferred per-category note)
2. `/architecture` — DEC for shared filter contract, API shapes, uncategorized bucket, forecast filter depth, Grafana panel scope
3. `/sprint-plan` — Decompose 6 AC after architecture

---

