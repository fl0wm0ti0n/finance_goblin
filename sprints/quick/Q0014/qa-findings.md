# QA Findings — Quick Q0014 / BUG-0012

**Work item:** BUG-0012 (defect)  
**Quick task:** Q0014  
**QA phase:** `/qa`  
**Date:** 2026-06-06  
**Orchestrator:** `auto-20260605-bug0012-001`  
**Verdict:** **PASS** (ready for `/verify-work`; V1 operator deploy + Full Firefly sync deferred)

## Scope

Forecast monthly Income/Fixed buckets always zero per `architecture-20260605-bug0012` (`handoffs/dev_to_qa.md`, `docs/engineering/architecture.md` § BUG-0012, **DEC-0067**):

- **AH1** — `RecurringPattern.category_id` from group mode; subscription override inherit/lookup
- **AG1** — Per-component `monthly_map` attribution (rolling → Variable; recurring → `resolve_bucket`)
- **T1** — Unit tests for salary→Income, rent→Fixed, mixed same-day, coffee→Variable, fixed-vs-unmapped regression
- **D1** — Retire net-delta `categorize_delta` from monthly bucket path; wire `category_names`
- **V1** — Omniflow probes + runbook §16 TOML checklist — **DEFERRED**

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0014/summary.md`, `sprints/quick/Q0014/plan-verify.json`, `sprints/quick/Q0014/plan-verify.md`, `docs/product/acceptance.md` (BUG-0012 rows AG/AH), `docs/engineering/architecture.md` (§ BUG-0012), `decisions/DEC-0067.md`, `backend/src/forecast/project.rs`, `backend/src/forecast/recurring.rs`, `backend/src/forecast/categories.rs`, `backend/src/forecast/types.rs`, `backend/src/forecast/service.rs`, `docs/engineering/runbook.md` (§16), `sprints/quick/Q0014/uat.md`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Backend unit tests | `cd backend && cargo test --lib` | **PASS** (139/139) |
| T-2 | Frontend unit tests | N/A — backend-only fix per frozen scope | **SKIP** |
| T-3 | Frontend build | N/A — no frontend changes | **SKIP** |
| T-4 | AH1 category_id on RecurringPattern | Static review `types.rs` + `recurring.rs` | **PASS** |
| T-5 | AH1 mode carry from group | `carries_mode_category_id_from_group` | **PASS** |
| T-6 | AH1 subscription override inherit | `subscription_override_inherits_category_id` | **PASS** |
| T-7 | AG1 rolling → Variable only | `project.rs` L78 `accumulate_bucket(..., Variable, rolling.daily_rate)` | **PASS** |
| T-8 | AG1 recurring → resolve_bucket | `project.rs` L79–81 per-component loop | **PASS** |
| T-9 | AG1 salary → Income | `salary_recurring_maps_to_income_bucket` | **PASS** |
| T-10 | AH1 rent → Fixed | `rent_recurring_maps_to_fixed_bucket` | **PASS** |
| T-11 | AG/AH same-day mixed | `same_day_salary_and_rent_both_buckets_nonzero` | **PASS** |
| T-12 | Unmapped → Variable regression | `discretionary_coffee_recurring_stays_variable` | **PASS** |
| T-13 | Fixed vs unmapped regression | `rent_moves_from_variable_to_fixed_bucket` | **PASS** |
| T-14 | D1 categorize_delta removed | `grep` — no `categorize_delta` in `backend/src/` | **PASS** |
| T-15 | D1 category_names wired | `service.rs` L95–117 passes map to `project_account` | **PASS** |
| T-16 | Daily balance path unchanged | `balance += delta` preserved; transfer/rejected/milestone tests pass | **PASS** |
| T-17 | Frozen boundaries | No positive rolling→Income; US-0015/US-0013 out of scope; no frontend | **PASS** |
| T-18 | Rows AG/AH live smoke | Omniflow deploy + Full Firefly sync + recompute | **DEFERRED** — verify-work (V1) |
| T-19 | Regression footer (OIDC + bundled-firefly) | Operator smoke per acceptance | **DEFERRED** — verify-work (plan-verify ADV-1) |

### Environment dependencies (non-blocking)

- **Operator deploy:** Q0014 backend image to omniflow before live acceptance rows AG/AH.
- **FULL_FIREFLY_SYNC_RECOMPUTE gate:** Manual Full Firefly sync after deploy; forecast recompute before V1 probes.
- **TOML_CATEGORY_BUCKETS (conditional):** If AG/AH fail on German/custom labels, extend `[forecast.category_buckets]` per runbook §16.

## Acceptance criteria matrix (BUG-0012)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **(AG)** | Non-zero Income when mirror has income-category inflows in forecast month | **PASS** (code) / **DEFERRED** (runtime) | AG1: per-component `monthly_map`; salary recurring + confirmed override → `income > 0` in unit tests. Live `GET /api/v1/forecast/monthly` + Monthly tab **DEFERRED** until deploy + Full sync |
| **(AH)** | Non-zero Fixed when mirror has fixed-cost category outflows | **PASS** (code) / **DEFERRED** (runtime) | AH1: `category_id` carry + inherit; AG1: rent recurring → `fixed_costs > 0`. Live probes **DEFERRED** until deploy + Full sync |
| Regression | OIDC + bundled-firefly deploy checks | **PASS** (unit) / **DEFERRED** (live) | No API shape change; frozen boundaries intact; full OIDC smoke at verify-work |

**Summary:** AH1–D1 **PASS** on static/automated path; full acceptance runtime deferred with `OPERATOR_DEPLOY_PENDING` + `OPERATOR_FULL_FIREFLY_SYNC_PENDING`.

## Architecture compliance (DEC-0067)

### AH1 — RecurringPattern category carry

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Field on struct | `category_id: Option<String>` | `types.rs` L21 | PASS |
| Mode from group | Mode of `RecurrenceGroup.category_ids`; tie-break latest date | `mode_category_id` in `recurring.rs` L29–47 | PASS |
| Subscription override | Inherit from replaced pattern or mirror lookup | `apply_subscription_override` + `lookup_category_id` in `project.rs` L192–226 | PASS |

### AG1 — Component-level monthly attribution

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Rolling component | Always Variable | `accumulate_bucket(entry, Bucket::Variable, rolling.daily_rate)` L78 | PASS |
| Recurring component | `resolve_bucket(category_id, category_names, config)` | L79–81 | PASS |
| Daily balance | `balance += delta` unchanged | L56–67 | PASS |
| Positive rolling → Income | **Rejected** — stays Variable | No income path for rolling | PASS |
| Unmapped categories | Default Variable | `map_category` fallback + `rent_moves_from_variable_to_fixed_bucket` | PASS |

### D1 — categorize_delta retirement

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| No categorize_delta in monthly path | Function removed/unused | Absent from `backend/src/` | PASS |
| category_names active | Passed through projection | `service.rs` → `project_account` param used in `resolve_bucket` | PASS |
| map_category tests | Preserved | `maps_known_categories`, `resolve_bucket_uses_category_name_map` | PASS |

### Frozen boundaries

| Boundary | Verdict |
|----------|---------|
| No positive rolling → Income | PASS |
| Transfer exclusion unchanged | PASS (`excludes_transfers_from_projection`) |
| Rejected fingerprint exclusion unchanged | PASS |
| US-0015 AI buckets out of scope | PASS |
| US-0013 ML overlay unchanged | PASS |
| No frontend changes | PASS |
| No `default.toml` code expansion | PASS |

## Plan-verify alignment

Plan-verify PASS (2026-06-06): 2/2 acceptance rows AG/AH covered; 5/5 tasks mapped; 0 gaps; 7 advisories. QA confirms code tasks AH1, AG1, T1, D1 implemented per DEC-0067. V1 correctly deferred to verify-work per operator gate.

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

## Advisories (carry-forward from plan-verify)

| ID | Note |
|----|------|
| ADV-1 | OIDC + bundled-firefly regression footer — operator verify-work smoke post-V1 |
| ADV-2 | Rows AG/AH blocked until operator manual Full Firefly sync + forecast recompute |
| ADV-3 | German/custom category labels may need operator `[forecast.category_buckets]` TOML extend |
| ADV-4 | Income via categorized recurring only — positive rolling stays Variable (DEC-0067) |
| ADV-5 | Variable shrink when fixed moves out is intended DEC-0007 behavior |
| ADV-6 | US-0015 AI buckets and US-0013 ML overlay out of scope |

## Next phase

**`/verify-work`** — after operator deploys AH1–D1 and runs manual Full Firefly sync + recompute:

1. `GET /api/v1/forecast/monthly?account_id=<funded>` — `income > 0` when mirror has salary/income categories (AG)
2. `/forecast` Monthly tab — Income stat card non-zero (AG-2)
3. `GET /api/v1/forecast/monthly?account_id=<funded>` — `fixed_costs > 0` when mirror has rent/utilities (AH)
4. `/forecast` Monthly tab — Fixed stat card non-zero (AH-2)
5. If AG/AH still zero: runbook §16 TOML checklist → extend buckets → recompute → re-probe
6. Regression footer: OIDC + bundled-firefly

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
