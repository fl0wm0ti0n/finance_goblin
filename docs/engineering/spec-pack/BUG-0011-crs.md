# CRS — BUG-0011 Planning mode broken (AD/AE/AF)

## Purpose

Close operator-reported planning defects **AD** (empty plan no-op), **AE** (compare illogical sums), and **AF** (plan-vs-actual 404) without Grafana Dashboard 3 changes or US-0014 epic scope.

## Scope

### In scope

- **AE (DEC-0073):** shared overlay delta helper; `version_metrics` + in-memory compare path; zero-overlay → 0.00 guard
- **AF (DEC-0074):** tagged `PlanVsActualApiResponse`; route 200 `no_active_plan`; frontend guided PVA tab
- **AD (execute):** first-run Create empty plan; inline add/edit adjustments; Custom Apply toast; Set active banner
- Backend integration tests for compare + PVA contracts
- Operator OIDC `/planning` smoke (acceptance footer)

### Out of scope

- Grafana `budgets` panel SQL (R-0020 — correct as-is)
- Auto-activate on plan create
- US-0014 holistic UX (tooltips, wizard)
- Plan hypertable schema migration

## Acceptance criteria ref

`docs/product/acceptance.md` — BUG-0011:

- **(AD)** "Start empty and add lines" creates editable empty plan — not silent no-op
- **(AE)** Compare with empty/minimal plan shows zero/neutral deltas — not illogical aggregates
- **(AF)** `GET /api/v1/plans/active/plan-vs-actual` returns 200 JSON (active payload or documented empty-state) — not raw 404

OIDC-enabled deploy regression checks pass. Intuitive UX epic tracked in **US-0014**.

## Dependencies

- US-0004 plan engine (baseline)
- DEC-0073 before or with DEC-0074 execute slices
- R-0015 overlay model; R-0016 compare labels; R-0020 Grafana boundary
