# QA Findings — Quick Q0025 / BUG-0017

**Work item:** BUG-0017 (defect)  
**Quick task:** Q0025  
**QA phase:** `/qa`  
**Date:** 2026-06-10  
**Orchestrator:** `intake-20260609-ui-audit`  
**Decisions:** DEC-0105, DEC-0106  
**QA agent:** fresh subagent (`qa-20260610-bug0017-qa-fresh`)  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — Implemented tasks **AY1**, **BA1**, **BA2**, **BD1**, and **T1** satisfy **DEC-0105** and **DEC-0106** at code and test level. Zero blocking findings. **V1** correctly deferred (operator **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** gates). Hand off to **`/verify-work`** for operator runtime probes (AY–BD live matrix).

## Scope

BUG-0017 post-sync forecast recompute cluster: audit CHECK migration (**AY1** / DEC-0105), `paired_baseline_id` FK CASCADE (**BA1** / DEC-0106), retention loop order (**BA2**), ForecastPage `isFetched` loading guard (**BD1**), paired retention integration test (**T1**). Runtime **V1** sync/audit/meta/planning/ML probes deferred per operator deploy gates.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0025/summary.md`, `sprints/quick/Q0025/tasks.md`, `backend/migrations/015_bug0017_ai_audit_forecast.sql`, `backend/migrations/015_bug0017_forecast_fk_cascade.sql`, `backend/src/forecast/repository.rs`, `frontend/src/pages/ForecastPage.tsx`, `backend/tests/forecast_integration.rs`, `decisions/DEC-0105.md`, `decisions/DEC-0106.md`, `docs/product/acceptance.md` BUG-0017 rows **AY**–**BD**. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | Full lib regression | `cargo test --lib` | **PASS** (213/213) |
| T-2 | Forecast integration suite | `cargo test --test forecast_integration` | **PASS** (3/3) |
| T-3 | T1 retention paired prune | `forecast_retention_prunes_paired_ml_without_fk_violation` in T-2 | **PASS** |
| T-4 | Frontend vitest regression | `cd frontend && npm test -- --run` | **PASS** (9/9) |
| T-5 | **AY1** — DEC-0105 audit CHECK migration | Code review migration SQL | **PASS** |
| T-6 | **BA1** — DEC-0106 FK CASCADE migration | Code review migration SQL | **PASS** |
| T-7 | **BA2** — retention loop order | Code review `repository.rs` | **PASS** |
| T-8 | **BD1** — ForecastPage `isFetched` guard | Code review `ForecastPage.tsx` | **PASS** |
| T-9 | **T1** — paired baseline+ML prune | Code review + T-3 | **PASS** |
| T-10 | Frozen boundaries — no sync fail-on-recompute / plan-engine change | Code review scope | **PASS** |
| T-11 | V1 operator smoke AY–BD | `sprints/quick/Q0025/uat.md` | **DEFERRED** — verify-work |
| T-12 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |
| T-13 | LINT / TYPECHECK | runbook keys blank | **SKIP** |

### Test output (T-1)

```
test result: ok. 213 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Test output (T-2)

```
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  forecast_retention_prunes_paired_ml_without_fk_violation ... ok
  forecast_recompute_persists_hypertable_rows ... ok
  forecast_meta_stale_when_latest_failed ... ok
```

## Task verdict matrix

| Task | Status (execute) | QA verdict | Notes |
|------|------------------|------------|-------|
| AY1 | done | **PASS** | DEC-0105 DROP+ADD+NOT VALID+VALIDATE for both CHECK constraints |
| BA1 | done | **PASS** | DEC-0106 `ON DELETE CASCADE` on `paired_baseline_id` FK |
| BA2 | done | **PASS** | `enforce_retention` loop `["ml_enhanced", "baseline"]` |
| BD1 | done | **PASS** | `showLoading` / `showEmpty` per architecture contract |
| T1 | done | **PASS** | Seeds 4 paired baseline+ML rows; asserts retention to 2+2 without FK error |
| V1 | open | **DEFERRED** | Operator gates: **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** |

## Acceptance criteria matrix (BUG-0017)

| Row | Criterion | QA verdict | Evidence |
|-----|-----------|------------|----------|
| **AY** | `forecast_bucket_assignment` audit rows persist without `tool_name` CHECK violation | **PASS** (code) / **DEFERRED** (runtime) | AY1 migration adds `forecast_bucket_assignment` to CHECK; T-1 PASS. Live sync log probe **DEFERRED** to verify-work |
| **AZ** | `low_confidence` / extended statuses persist without `result_status` CHECK violation | **PASS** (code) / **DEFERRED** (runtime) | AY1 migration adds `low_confidence`, `provider_unavailable`, `parse_error`; T-1 PASS. Live audit row probe **DEFERRED** to verify-work |
| **BA** | Recompute retention succeeds; meta fresh after sync | **PASS** (code+test) / **DEFERRED** (runtime) | BA1 CASCADE + BA2 order + T1 integration 3/3; T-3 asserts prune without FK violation. `GET /forecast/meta` live probe **DEFERRED** to verify-work |
| **BB** | ML-enhanced selectable when gate passes; honest `ml_skipped_reason` | **DEFERRED** | V1-only per sprint contract; no code change in Q0025 execute scope |
| **BC** | Planning Compare loses **Plan stale** after successful recompute | **DEFERRED** | Downstream of BA fix; V1-only per DEC-0106; no plan-engine change |
| **BD** | No false empty flash when meta has `computation_id` | **PASS** (code) / **DEFERRED** (runtime) | BD1 `showLoading` on pending; `showEmpty` only when `isFetched && !isError && !computation_id`; T-4 PASS. Browser nav probe **DEFERRED** to verify-work |

## Code review vs decisions

### DEC-0105 (AY1)

| Contract | Status | Evidence |
|----------|--------|----------|
| Migration file `015_bug0017_ai_audit_forecast.sql` | **PASS** | Sequential after 014 |
| `tool_name` adds `forecast_bucket_assignment` | **PASS** | Lines 4–9 |
| `result_status` adds `low_confidence`, `provider_unavailable`, `parse_error` | **PASS** | Lines 13–16 |
| DROP → ADD NOT VALID → VALIDATE pattern | **PASS** | Both constraints |
| No Rust insert-path changes | **PASS** | Scope limited to migration |

### DEC-0106 (BA1 + BA2 + T1)

| Contract | Status | Evidence |
|----------|--------|----------|
| FK `ON DELETE CASCADE` on `paired_baseline_id` | **PASS** | `015_bug0017_forecast_fk_cascade.sql` lines 3–8 |
| NOT VALID + VALIDATE pattern | **PASS** | Lines 7–8 |
| Retention loop `ml_enhanced` before `baseline` | **PASS** | `repository.rs` line 560 |
| Integration test for paired prune | **PASS** | `forecast_integration.rs` lines 174–247 |
| Sync fail-on-recompute unchanged | **PASS** | No sync module edits in Q0025 scope |

### BD1 (ForecastPage loading guard)

| Contract | Status | Evidence |
|----------|--------|----------|
| `showLoading = metaQuery.isPending` | **PASS** | `ForecastPage.tsx` line 147 |
| `showEmpty = isFetched && !isError && !computation_id` | **PASS** | Lines 148–149 |
| Loading card before empty card | **PASS** | Lines 219–224 |
| Chart content preserved when `computation_id` present | **PASS** | `hasForecast` still drives downstream queries |

## Generated baseline test evidence (US-0066)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | rust + node (vitest) |
| `generated_test_command` | `cargo test --lib`; `cargo test --test forecast_integration`; `cd frontend && npm test -- --run` |
| `generated_test_result` | **pass** |
| `generated_test_output_ref` | T-1/T-2/T-4 output above |
| `generated_test_paths_ref` | `backend/tests/forecast_integration.rs`, `backend/migrations/015_bug0017_*.sql`, `backend/src/forecast/repository.rs`, `frontend/src/pages/ForecastPage.tsx` |
| `generated_test_reason_code` | — |

## Runtime QA evidence (US-0065)

Runtime autopilot probes **not executed in QA phase** — V1 deferred per sprint contract and operator gates.

| Field | Value |
|-------|-------|
| `runtime_startup_command` | deferred |
| `runtime_stack_profile` | rust axum + react vite + postgres |
| `runtime_mode` | deferred (verify-work) |
| `runtime_health_target` | `:18080` + `financegnome.omniflow.cc` |
| `runtime_health_result` | deferred |
| `runtime_log_summary` | n/a |
| `runtime_retry_count` | 0 |
| `runtime_retry_ledger` | [] |
| `runtime_final_verdict` | deferred |
| `runtime_reason_code` | V1_DEFERRED_BACKEND_FRONTEND_DEPLOY_FULL_FIREFLY_SYNC |
| `runtime_evidence_refs` | `sprints/quick/Q0025/uat.md`, `sprints/quick/Q0025/uat.json` |

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

**Informational (non-blocking):**

1. **V1 runtime** — sync trigger log audit, meta freshness, month-bucket SQL (**BB**), Planning Compare (**BC**), Forecast nav (**BD**) pending deploy + Full sync.
2. No dedicated ForecastPage unit test for BD1 guard — acceptable; contract verified by code review + npm regression baseline.
3. Pre-existing lib compile warnings (unused imports) — unchanged by Q0025; not blocking.
4. Two migration files share prefix `015_` — SQLx lexicographic order runs audit CHECK before FK CASCADE; acceptable ordering.

## Verify-work readiness

| Gate | Status |
|------|--------|
| Code QA (AY1–BD1, T1) | **READY** |
| `cargo test --lib` | **READY** — 213/213 PASS |
| `cargo test --test forecast_integration` | **READY** — 3/3 PASS |
| `npm test` | **READY** — 9/9 PASS |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** |
| Operator **FULL_FIREFLY_SYNC** | **PENDING** |
| V1 operator smoke (AY–BD) | **PENDING** |

## Next phase

**`/verify-work`** — rebuild `flow-finance-ai`, Full sync + recompute, then V1 probes per `sprints/quick/Q0025/uat.md`.

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
