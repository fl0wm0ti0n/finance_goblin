# PO to TL archive pack (2026-06-08)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 10
- First archived heading: `## discovery-20260609-us0019 — US-0019 goal-driven planning & AI savings (hot pointer)`
- Last archived heading: `## discovery-20260609-us0019 — US-0019 goal-driven planning, per-plan stats & AI savings suggestions`
- Verification tuple (mandatory):
  - archived_body_lines=102
  - retained_body_lines=498

---

## discovery-20260609-us0019 — US-0019 goal-driven planning & AI savings (hot pointer)
**From:** PO **To:** Tech Lead **Story:** US-0019 **Run:** `auto-20260608-us0019-001` **Next:** `/research` · [R-0080](docs/engineering/research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake) · US-0018 **DEC-0087**..**DEC-0089** (category dependency)

## discovery-20260609-us0019 — US-0019 goal-driven planning, per-plan stats & AI savings suggestions

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-09  
**Story:** US-0019  
**Orchestrator run:** `auto-20260608-us0019-001`  
**Intake bundle:** `intake-20260607-category-planning-subscriptions`  
**Prior release:** US-0018 `0.18.0-us0018` (S0017) — category filter contract **DEC-0087**..**DEC-0090**  
**Next phase:** `/research`

### Summary

Discovery refined **US-0019** as the **goal-planning vertical** for the intake bundle: **target balance + target date** plans with **per-plan statistics** (monthly delta vs baseline, yearly rollup, projected balance at target date), **category-scoped adjustments** in the plan builder, and **AI-assisted category savings** the operator selects before applying — building on released US-0018 category APIs and US-0014 planning UX. **Single story retained.**

### US-0018 dependency (frozen contracts)

| DEC | Relevance to US-0019 |
|-----|----------------------|
| **DEC-0087** | `GET /api/v1/categories` + expense-series — category picker + AI evidence totals |
| **DEC-0088** | `CategoryFilter` combobox pattern — reuse in adjustment form |
| **DEC-0089** | Planning compare: `CategoryTrendChart` = **actuals only**; compare API **unchanged** — US-0019 owns category **overlay** in plan engine + goal stats strip |

### Surface → feature map (canonical)

| Surface | Tab | Discovery target |
|---------|-----|------------------|
| Goal template | Scenarios | New **`goal_balance`** card: name, `target_balance_eur`, `target_date`, optional `account_id` |
| Goal metadata | Scenarios | Editable on latest unfrozen version; read-only when frozen |
| Per-plan stats | Compare + Scenarios summary | Scoped to `plan_id`: monthly delta vs baseline, yearly rollup, balance at `target_date` |
| Category adjustments | Scenarios add-line form | `CategoryFilter` + `target_type=category`; overlay must affect recompute (AC-3) |
| AI savings | Scenarios action | Modal (mirror savings-mode UX): ranked categories + evidence; checkbox → POST adjustments |
| Regression | All tabs | US-0014 templates, toasts, PVA guided empty state, OIDC smoke (AC-6) |

### Partial implementation review

| Area | Status |
|------|--------|
| Plan templates + versioning | **Done** (US-0004) |
| US-0014 onboarding / mutation toasts | **Done** |
| `AdjustmentTarget::Category` enum + form | **Done** — overlay ignores category scoping |
| `CategoryFilter` + categories API | **Done** (US-0018) |
| Compare version metrics per plan | **Done** — no target-date / yearly rollup |
| Subscription savings modal + API | **Done** — payee-based only |
| Goal metadata schema/API/UI | **Missing** |
| Per-plan goal stats at target date | **Missing** |
| AI category savings (REST + tool) | **Missing** |
| Category overlay in `build_overlay_deltas` | **Missing** |

### Discovery decomposition evidence

- Feature/workflow count: goal template + metadata + stats strip + category overlay + AI modal (moderate-high — **single story retained**)
- Cross-cutting impact: `backend/plan/`, `backend/api/plans.rs`, `backend/ai/tools`, `frontend/PlanningPage.tsx`, possible migration
- Acceptance breadth: 6 AC unchanged (`docs/product/acceptance.md#US-0019`)
- Risk surface: category overlay join to mirror txs; goal feasibility math; account selection for starting balance (acct 114 vs 116); AI privacy audit; DEC-0089 actuals vs overlay semantics

### Decision gates (PO recommendation)

| Topic | Recommendation | Alternative |
|-------|--------------|-------------|
| Goal shape | Template **`goal_balance`** + DB columns | JSON metadata blob |
| Stats placement | Compare tab strip + Scenarios card | Fourth **Goals** tab |
| Category overlay | Engine maps `category_id` to spend deltas | Display-only until forecast fork |
| AI API | `GET .../savings-suggestions` aggregates + optional chat tool | Chat-only |
| Balance account | Selectable asset account; default primary funded | Household total |

### Open questions (carry to research)

| Topic | Question |
|-------|----------|
| **Goal schema** | Columns on `plans` vs `plan_versions`; migration shape |
| **Target-date projection** | SQL/API for balance at arbitrary `target_date` from `plan_computations` |
| **Yearly rollup** | Calendar year vs rolling 12m from target horizon |
| **Category overlay** | Join mirror `category_id` to size `remove_outflow` caps — interaction with DEC-0007 buckets |
| **Feasibility** | Back-solve required monthly savings; show gap copy only vs auto-lines |
| **AI ranking** | Top-N categories from `aggregates_by_category`; min spend threshold |
| **Account default** | Which asset account seeds starting balance for goal progress |
| **PVA scope** | AC-2 stats on Compare/Scenarios only — confirm PVA stays active-plan household |

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#US-0019` | Discovery refinements + dependency note | pass |
| `docs/product/acceptance.md` US-0019 | AC-1..AC-6 still valid; no rewrite | pass |
| `docs/product/vision.md` US-0019 discovery | UX references + surface map + partial impl | pass |
| `frontend/src/pages/PlanningPage.tsx` | Gaps documented (goal, overlay, AI) | pass |
| `backend/src/plan/overlay.rs` | Category target not scoped — documented | pass |
| US-0018 DEC-0087..0089 | Dependency contracts referenced | pass |
| R-0080 | Intake research linked; discovery extends | pass |

### Recommended next steps

1. `/research` — Goal schema spike; target-date balance SQL; category overlay join; AI category ranking API; account selection (extends R-0080)
2. `/architecture` — DEC for goal template, stats API, overlay semantics, AI suggestions contract
3. `/sprint-plan` — Decompose 6 AC after architecture

---

