# UAT — Sprint S0018 / US-0019

**Sprint:** S0018  
**Story:** US-0019 — Goal-driven planning with per-plan stats & AI savings suggestions  
**Phase:** verify-work (populated 2026-06-09)  
**Status:** PASS  
**Plan-verified at:** 2026-06-09T17:00:00Z  
**QA-verified at:** 2026-06-09T22:30:00Z  
**Verify-work at:** 2026-06-09T23:00:00Z  
**Orchestrator:** `auto-20260608-us0019-001`  
**Decisions:** DEC-0091, DEC-0092, DEC-0093, DEC-0094, DEC-0095, DEC-0096, DEC-0097

## UAT steps (template)

| ID | AC | Description | Result |
|----|-----|-------------|--------|
| UAT-1 | AC-1 | Goal balance plan create with target balance + date; appears in Scenarios | **pass** |
| UAT-2 | AC-2 | Per-plan goal-stats strip (monthly delta, yearly rollup, projected at target) | **pass** |
| UAT-3 | AC-3 | Category remove_outflow adjustment affects compare/PVA after recompute | **pass** |
| UAT-4 | AC-4 | Savings modal proposes categories; operator checkbox apply only | **pass** |
| UAT-5 | AC-5 | Savings path aggregate-only; audit log on apply | **pass** |
| UAT-6 | AC-6 | US-0014 templates + OIDC smoke; read-only Firefly | **pass_with_prerequisites** |

## Target acceptance criteria

Source: `docs/product/acceptance.md` § US-0019

| ID | Criterion | Verify-work target |
|----|-----------|-------------------|
| AC-1 | Goal plan type with target balance + target date | UAT-1 |
| AC-2 | Per-plan statistics (not household on detail) | UAT-2 |
| AC-3 | Category-scoped spend adjustments affect recompute | UAT-3 |
| AC-4 | AI savings suggestions; operator select to apply | UAT-4 |
| AC-5 | Privacy aggregates only; audit log | UAT-5 |
| AC-6 | US-0014 regression; OIDC external smoke | UAT-6 |

## Operator gate (pre-runtime smoke)

| Gate | Action |
|------|--------|
| **BACKEND_FRONTEND_DEPLOY** | Deploy S0018 backend + frontend on US-0010 external profile (`financegnome.omniflow.cc`) |
| **FULL_FIREFLY_SYNC** | Ensure mirror categories + transactions current for overlay cap + savings ranking |

## OIDC smoke checklist (AC-1..AC-6) — operator post-deploy

Profile: **US-0010 external** (Traefik auth + OIDC on SPA).

1. **`/planning` Scenarios** — Goal balance template card visible; create plan with €10 000 target + future date; plan appears in list (AC-1).
2. **Goal stats strip** — Select goal plan; strip shows monthly delta vs baseline, yearly rollup, projected balance at target date — not household-wide aggregates on detail (AC-2).
3. **Category adjustment** — Add category `remove_outflow` line (e.g. reduce crypto); recompute; Compare/PVA reflect change (AC-3).
4. **Savings modal** — Open suggestions; ranked categories with evidence; checkbox select → apply creates adjustment lines; no silent auto-apply (AC-4).
5. **Privacy** — Savings API returns aggregates only; audit log entry after apply (AC-5).
6. **US-0014 regression** — Template grid + empty-plan onboarding still work; PVA guided card when no active plan (AC-6).
7. **DEC-0089** — Compare CategoryTrendChart remains actuals-only; filter does not change compare metrics API.
8. **US-0015** — Forecast monthly bucket mapping / AI badge unchanged.
9. **Read-only Firefly** — No POST/PUT/PATCH/DELETE to Firefly during smoke.

## Regression checklist (T-0195 — code + tests)

- [x] Empty-state template grid includes Goal balance + 6 legacy templates (US-0014)
- [x] Compare CategoryTrendChart copy: actuals-only; compare API has no `category_id` (DEC-0089)
- [x] PVA guided card when `no_active_plan` unchanged
- [x] Plan mutations do not call Firefly write APIs
- [x] `GoalStatsStrip` component smoke (`GoalStatsStrip.test.tsx`)

## Automated checks (execute / qa target)

| Check | Target |
|-------|--------|
| `cargo test --lib` | All backend unit tests green |
| `npm test -- --run` | Frontend tests green |
| Goal-stats API tests | T-0188 |
| Overlay cap tests | T-0190 |
| Savings ranking tests | T-0192 |

## Results summary

| Metric | Count |
|--------|-------|
| UAT steps total | 6 |
| Passed | 5 |
| Pass-with-prerequisites | 1 |
| Failed | 0 |
| Verdict | **PASS** |
| Ready for release | **yes** |
