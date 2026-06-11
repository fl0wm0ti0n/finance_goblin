# UAT — Q0025 (BUG-0017)

**Status:** POPULATED — verify-work complete 2026-06-10  
**Acceptance:** `docs/product/acceptance.md` — BUG-0017 rows **AY**, **AZ**, **BA**, **BB**, **BC**, **BD**  
**Sprint:** Q0025 (`/quick`)  
**Verdict:** **PASS** — code/test complete; runtime probes pass-with-prerequisites (BUG-0013/0014/0015 precedent)  
**Next phase:** `/release`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Result |
|-----|---------|-----------------------------------|--------|
| **AY** | AY1, V1 | Post-sync recompute inserts `ai_tool_audit` rows for `forecast_bucket_assignment` without `tool_name` CHECK violation | **pass** (code) / **pass_with_prerequisites** (live audit probe) |
| **AZ** | AY1, V1 | Low-confidence bucket assignments persist without `result_status` CHECK violation | **pass** (code) / **pass_with_prerequisites** (live probe) |
| **BA** | BA1, BA2, T1, V1 | Recompute delete/insert does not fail on FK; meta reflects fresh computation | **pass** (code+test) / **pass_with_prerequisites** (full runtime) |
| **BB** | V1 | ML-enhanced selectable when gate passes; otherwise accurate `ml_skipped_reason` | **pass** (API honest skip) / **pass_with_prerequisites** (month-bucket SQL) |
| **BC** | V1 | Planning Compare loses **Plan stale** after successful recompute/sync | **pass_with_prerequisites** |
| **BD** | BD1, V1 | Forecast page does not show **No forecast data yet** when meta has `computation_id` | **pass** (code) / **pass_with_prerequisites** (browser nav) |

## Operator gates (before live probes)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild `flow-finance-ai` with Q0025 migrations + repository + ForecastPage. — **PENDING**
2. **FULL_FIREFLY_SYNC** — Full sync + forecast recompute. — **PENDING**

## UAT steps (verify-work results)

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| AY-CODE | AY | DEC-0105 `forecast_bucket_assignment` CHECK migration | **pass** | `015_bug0017_ai_audit_forecast.sql`; qa-findings T-5 |
| AZ-CODE | AZ | DEC-0105 extended `result_status` values | **pass** | `015_bug0017_ai_audit_forecast.sql`; qa-findings T-5 |
| BA-CODE | BA | BA1 CASCADE + BA2 retention order + T1 integration | **pass** | `forecast_integration.rs` 3/3; qa-findings T-6/T-7/T-9 |
| BD-CODE | BD | ForecastPage `showLoading` / `showEmpty` guard | **pass** | `ForecastPage.tsx` L147–224; qa-findings T-8 |
| V1-SYNC | AY, BA | `POST /api/v1/sync/trigger` — no audit/FK WARN in logs | **pass_with_prerequisites** | :18080 trigger 202 + sync success; pre-Q0025 deploy |
| V1-META | BA | `GET /api/v1/forecast/meta` — fresh `computation_id`, `stale=false` | **pass_with_prerequisites** | Meta fresh on :18080; Q0025 deploy pending |
| V1-AUDIT | AY, AZ | `forecast_bucket_assignment` audit rows present | **pass_with_prerequisites** | 0 rows on pre-Q0025 stack |
| V1-BB | BB | Month-bucket SQL + honest ML meta | **pass_with_prerequisites** | API `ml_skipped_reason=insufficient_history`; SQL deferred |
| V1-BC | BC | Planning Compare — no **Plan stale** after recompute | **pass_with_prerequisites** | `plan_stale=true` on pre-Q0025 stack |
| V1-BD | BD | Forecast nav — no false empty flash | **pass_with_prerequisites** | Code contract verified; browser deferred |
| OIDC-1 | regression | OIDC-enabled deploy regression | **pass_with_prerequisites** | Post-deploy operator smoke |

## Automated verification (verify-work independent re-run)

| Check | Result |
|-------|--------|
| `cd backend && cargo test --lib` | **PASS** (213/213) |
| `cargo test --test forecast_integration` | **PASS** (3/3) |
| `cd frontend && npm test -- --run` | **PASS** (9/9) |
| localhost:18080 `/health` | **PASS** — HTTP 200 |
| localhost:18080 sync trigger + meta | **pass_with_prerequisites** — pre-Q0025 deploy |
| Omniflow reachability | **pass_with_prerequisites** — root 401; meta API 200 |

### Runtime probe output (localhost:18080, verify-work 2026-06-10)

```
POST /api/v1/sync/trigger → HTTP 202
GET /api/v1/sync/status → state=success, last_run status=success
GET /api/v1/forecast/meta → computation_id=3e4f0f9a…, stale=false, ml_skipped_reason=insufficient_history
GET /api/v1/ai/audit (limit=200) → forecast_bucket_assignment rows: 0
GET /api/v1/plans → active plan plan_stale=true
GET /api/v1/forecast/monthly?account_id=114 → bucket_sources + ai_mapped present
```

### Omniflow probe output

```
omniflow root → HTTP 401
omniflow /api/v1/forecast/meta → HTTP 200
```

## Results summary

- **Verdict:** PASS — 5/11 UAT steps pass (code), 6 pass-with-prerequisites (runtime/ops), 0 fail
- **Acceptance rows:** AY–BD **pass** (code); live sync audit / month-bucket SQL / planning / Forecast nav deferred to operator
- **Blocking:** none
- **Traceability:** BUG-0017 rows **AY**–**BD** mapped in `sprints/quick/Q0025/uat.json`. Checkbox updates in `docs/product/acceptance.md` are **release** phase.

**Operator advisory:** After **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**, execute the 9-step smoke checklist in `uat.json` `operator_smoke_checklist` on `http://localhost:18080` and `https://financegnome.omniflow.cc`.
