# Q0025 summary — BUG-0017

**Sprint:** Q0025 (quick)  
**Bug:** BUG-0017 — Post-sync forecast recompute cluster  
**Orchestrator:** `intake-20260609-ui-audit`  
**Execute completed:** 2026-06-10

## Goal

Close BUG-0017 by extending `ai_tool_audit` CHECK constraints (**DEC-0105**), adding `paired_baseline_id` ON DELETE CASCADE (**DEC-0106**), reordering retention prune loop, fixing ForecastPage loading flash (**BD**), and adding retention integration test.

## Tasks completed

| ID | Title | Status | Evidence |
|----|-------|--------|----------|
| AY1 | `ai_tool_audit` CHECK migration | **done** | `backend/migrations/015_bug0017_ai_audit_forecast.sql` |
| BA1 | `paired_baseline_id` CASCADE migration | **done** | `backend/migrations/015_bug0017_forecast_fk_cascade.sql` |
| BA2 | Retention loop order | **done** | `backend/src/forecast/repository.rs` — `ml_enhanced` before `baseline` |
| BD1 | ForecastPage `isFetched` empty guard | **done** | `frontend/src/pages/ForecastPage.tsx` — `showLoading` / `showEmpty` |
| T1 | Retention integration test | **done** | `backend/tests/forecast_integration.rs` — paired prune test |
| V1 | verify-work operator smoke | **open** | Deferred to `/verify-work` after **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** |

## Implementation notes

- **DEC-0105:** DROP+ADD+NOT VALID+VALIDATE for `tool_name` (`forecast_bucket_assignment`) and `result_status` (`low_confidence`, `provider_unavailable`, `parse_error`).
- **DEC-0106:** FK `ON DELETE CASCADE` on `paired_baseline_id`; `enforce_retention` processes `ml_enhanced` before `baseline`.
- **BD:** Pending meta query shows loading card; empty card only when `isFetched && !isError && !computation_id`.
- **Frozen boundaries:** No sync fail-on-recompute change; no plan-engine edits; BB/BC ops-only in V1.

## Test results

| Check | Result |
|-------|--------|
| `cargo test --lib` | **213/213 PASS** |
| `cargo test --test forecast_integration forecast_retention_prunes_paired_ml_without_fk_violation` | **1/1 PASS** |
| `npm test -- --run` | **9/9 PASS** |

## Operator gates (before V1)

| Gate | Status |
|------|--------|
| **BACKEND_FRONTEND_DEPLOY** | pending — rebuild `flow-finance-ai` with Q0025 migrations + repository + ForecastPage |
| **FULL_FIREFLY_SYNC** | pending — Full sync + forecast recompute before meta/planning/ML probes |

## Next phase

**`/qa`** — code review + test verification; V1 runtime smoke deferred to verify-work.
