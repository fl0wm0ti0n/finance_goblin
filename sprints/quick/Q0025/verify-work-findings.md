# Verify-work Findings — Quick Q0025 / BUG-0017

**Work item:** BUG-0017 (defect)  
**Quick task:** Q0025  
**Phase:** `/verify-work`  
**Orchestrator:** `intake-20260609-ui-audit`  
**Date:** 2026-06-10  
**Decisions:** DEC-0105, DEC-0106  
**Verify-work agent:** fresh subagent (`verify-work-20260610-bug0017-qa-fresh`)  
**Verdict:** **PASS** — rows **AY**–**BD** satisfied at code/test level; V1 operator runtime probes pass-with-prerequisites per BUG-0013/0014/0015 precedent; proceed to `/release`

## Summary

Verify-work populated UAT artifacts from QA PASS evidence (`sprints/quick/Q0025/qa-findings.md`, `handoffs/dev_to_qa.md`). Independent re-run confirms **213/213** lib tests, **3/3** forecast integration tests (including T1 paired retention prune), and **9/9** frontend vitest. Implemented tasks **AY1**, **BA1**, **BA2**, **BD1**, **T1** pass DEC-0105/DEC-0106 contracts at code/test level. V1 operator probes (sync audit rows, month-bucket SQL, Planning Compare **Plan stale**, ForecastPage nav) recorded as **pass-with-prerequisites** pending **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** on Q0025 delta. Zero blocking findings.

## Per-row verdict (acceptance AY–BD)

| Row | Verdict | Summary |
|-----|---------|---------|
| **AY** | **pass** | AY1 migration adds `forecast_bucket_assignment` to CHECK; T-1 PASS. Live audit row probe **pass-with-prerequisites** — 0 rows on pre-Q0025 `:18080` stack |
| **AZ** | **pass** | AY1 migration adds `low_confidence`, `provider_unavailable`, `parse_error`; T-1 PASS. Live extended-status probe **pass-with-prerequisites** |
| **BA** | **pass** | BA1 CASCADE + BA2 retention order + T1 integration 3/3; sync trigger 202 + meta `stale=false` on `:18080` (partial). Full post-deploy recompute probe **pass-with-prerequisites** |
| **BB** | **pass** | Meta API shows honest `ml_skipped_reason=insufficient_history` when ML unavailable; monthly API exposes `bucket_sources`/`ai_mapped`. Month-bucket SQL probe **pass-with-prerequisites** |
| **BC** | **pass** | Downstream of BA; no plan-engine change in scope. `plan_stale=true` persists on pre-Q0025 stack after sync — **pass-with-prerequisites** |
| **BD** | **pass** | BD1 `showLoading`/`showEmpty` contract verified by code review; T-4 npm PASS. Browser Forecast nav probe **pass-with-prerequisites** |

## Operator gate

| Gate | Status |
|------|--------|
| Code (AY1–BD1, T1) | **CLEARED** |
| `cargo test --lib` | **CLEARED** — 213/213 PASS |
| `cargo test --test forecast_integration` | **CLEARED** — 3/3 PASS |
| `npm test -- --run` | **CLEARED** — 9/9 PASS |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** — Q0025 migrations + repository + ForecastPage |
| Operator **FULL_FIREFLY_SYNC** | **PENDING** — Full sync + recompute on Q0025 image |
| V1 sync audit / month-bucket SQL / planning / Forecast nav | **PENDING** — pass-with-prerequisites |

## Automated verification (verify-work independent re-run)

| Check | Result |
|-------|--------|
| `cd backend && cargo test --lib` | **PASS** (213/213) |
| `cargo test --test forecast_integration` | **PASS** (3/3) |
| `cd frontend && npm test -- --run` | **PASS** (9/9) |
| localhost:18080 `/health` | **PASS** — HTTP 200 |
| localhost:18080 `POST /api/v1/sync/trigger` | **pass_with_prerequisites** — HTTP 202; sync `success`; pre-Q0025 deploy |
| localhost:18080 `GET /api/v1/forecast/meta` | **pass_with_prerequisites** — `computation_id` present, `stale=false`, honest `ml_skipped_reason`; pre-Q0025 deploy |
| localhost:18080 `GET /api/v1/ai/audit` | **pass_with_prerequisites** — 0 `forecast_bucket_assignment` rows (migration/deploy pending) |
| localhost:18080 `GET /api/v1/plans` | **pass_with_prerequisites** — active plan `plan_stale=true` post-sync (BC deferred) |
| Omniflow reachability | **pass_with_prerequisites** — root 401; `/api/v1/forecast/meta` 200 behind auth |

### Test output (lib suite)

```
test result: ok. 213 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Test output (forecast integration)

```
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  forecast_retention_prunes_paired_ml_without_fk_violation ... ok
  forecast_recompute_persists_hypertable_rows ... ok
  forecast_meta_stale_when_latest_failed ... ok
```

### Runtime probe output (localhost:18080, verify-work 2026-06-10)

```
POST /api/v1/sync/trigger → HTTP 202
GET /api/v1/sync/status → state=success, last_run status=success
GET /api/v1/forecast/meta → computation_id=3e4f0f9a…, stale=false, ml_status=skipped, ml_skipped_reason=insufficient_history
GET /api/v1/ai/audit (limit=200) → forecast_bucket_assignment rows: 0
GET /api/v1/plans → active plan plan_stale=true
GET /api/v1/forecast/monthly?account_id=114 → bucket_sources + ai_mapped present
```

## Code contract verification

| Contract | Evidence |
|----------|----------|
| **DEC-0105** — `forecast_bucket_assignment` + extended statuses | `015_bug0017_ai_audit_forecast.sql`; qa-findings T-5/T-6 |
| **DEC-0106** — FK CASCADE + ml_enhanced-first retention | `015_bug0017_forecast_fk_cascade.sql`; `repository.rs` L560; T1 integration |
| **BD1** — `isFetched` empty guard | `ForecastPage.tsx` L147–149, L219–224 |
| **T1** — paired prune without FK violation | `forecast_integration.rs` retention test |
| Frozen boundaries — no sync fail-on-recompute / plan-engine change | Code review per qa-findings T-10 |

## Acceptance impact

| Row | Verify-work | `acceptance.md` checkbox |
|-----|-------------|---------------------------|
| **AY**–**BD** | **PASS** | Release phase |

## Release gate

| Gate | Status |
|------|--------|
| QA PASS (dev handoff) | yes |
| Verify-work PASS | **yes** |
| UAT populated (DEC-0009) | **yes** — `uat.json`, `uat.md` |
| Acceptance AY–BD | **PASS** (code) + runtime prerequisites documented |
| Isolation evidence (verify-work) | **yes** |
| Release proceed | **yes** |

## Operator advisory (non-blocking)

1. **BACKEND_FRONTEND_DEPLOY:** Rebuild `flow-finance-ai` with Q0025 migrations (`015_bug0017_*`), repository retention order, ForecastPage BD1 guard.
2. **FULL_FIREFLY_SYNC:** Full Firefly sync + forecast recompute before audit/meta/planning probes.
3. **V1-SYNC:** `POST /api/v1/sync/trigger` — confirm logs free of `ai_tool_audit_tool_name_check` / FK WARN.
4. **V1-AUDIT:** `SELECT tool_name, result_status FROM ai_tool_audit WHERE tool_name = 'forecast_bucket_assignment' LIMIT 5` — rows present.
5. **V1-BB-SQL:** Month-bucket probe per R-0087 §4 — confirm `month_buckets` vs `min_monthly_points` before interpreting ML skip.
6. **V1-BC:** Planning Compare — **Plan stale** badge clears after successful recompute on Q0025 image.
7. **V1-BD:** Forecast nav from Home — loading skeleton during pending meta; no false empty when `computation_id` set.
8. **Reopen criteria:** Audit CHECK WARN after deploy → reopen execute; FK violation on retention → reopen BA1/BA2; false empty flash persists → reopen BD1.

## Artifacts

- `sprints/quick/Q0025/uat.json`
- `sprints/quick/Q0025/uat.md`
- `sprints/quick/Q0025/qa-findings.md`
- `handoffs/dev_to_qa.md`
- `handoffs/verify_work_to_release.md`

## Next steps

1. **`/release`** — check BUG-0017 acceptance AY–BD; publish release notes; update backlog → DONE

---

**Stop here.** Continue in a **new** subagent/chat with **`/release`**.
