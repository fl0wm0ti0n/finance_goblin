# Q0025 — BUG-0017 post-sync forecast recompute cluster

| Field | Value |
|-------|-------|
| **ID** | Q0025 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0017 |
| **Created** | 2026-06-10 |
| **Architecture** | `architecture-20260609-bug0017` (`docs/engineering/architecture-archive/architecture-pack-20260609.md` § BUG-0017) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260610-q0025-bug0017`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0017 rows **AY**–**BD** |
| **Task count** | 6 (all P0 mandatory) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0017 post-sync forecast recompute cluster: **DEC-0105** audit CHECK migration (**AY1**), **DEC-0106** FK CASCADE + retention prune order (**BA1**, **BA2**), ForecastPage `isFetched` loading guard (**BD1**), retention integration test (**T1**), operator verify-work smoke (**V1**) covering **BB** month-bucket probe, **BC** planning re-smoke, and **BD** nav UX after deploy + Full sync.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| Audit CHECK (DEC-0105) | AY1 | `backend/migrations/015_bug0017_ai_audit_forecast.sql` |
| FK CASCADE (DEC-0106) | BA1 | `backend/migrations/015_bug0017_forecast_fk_cascade.sql` (or combined with AY1) |
| Retention order (DEC-0106) | BA2 | `backend/src/forecast/repository.rs` |
| ForecastPage UX | BD1 | `frontend/src/pages/ForecastPage.tsx` |
| Integration test | T1 | `backend/tests/` or `forecast/repository.rs` tests |
| Verify | V1 | `uat.md` + operator sync/meta/planning smoke |

**Ops-only (not execute tasks):** Operator **BACKEND_FRONTEND_DEPLOY** + Full sync + recompute before V1 runtime probes.

**Out of scope:** Sync fail-on-recompute semantics (deferred product gate); BC separate plan-engine fix; lowering `min_monthly_points`; `placeholderData` / SSR `initialData` for BD.

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| AY1 | `ai_tool_audit` CHECK migration | 2h | — | **AY**, **AZ** | P0 |
| BA1 | `paired_baseline_id` ON DELETE CASCADE migration | 1.5h | — | **BA** | P0 |
| BA2 | Retention loop `ml_enhanced` before `baseline` | 1h | BA1 | **BA** | P0 |
| BD1 | ForecastPage `isFetched` empty guard | 1h | — | **BD** | P0 |
| T1 | Retention integration test — paired prune | 2h | BA1, BA2 | **BA** | P0 |
| V1 | verify-work sync + meta + planning smoke | 2h | AY1, BA1, BA2, BD1, T1 + deploy | **AY**–**BD** | P0 |

**Total estimate:** ~9.5h (7.5h dev + ~2h operator V1).

## Deploy order

```text
AY1 + BA1 (migrations) → BA2 + BD1 + T1
  → single backend+frontend release
  → operator: BACKEND_FRONTEND_DEPLOY
  → Full Firefly sync + forecast recompute
  → V1 verify-work (logs, meta, month-bucket SQL, planning, forecast nav)
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **AY** | AY1, V1 | Post-sync `ai_tool_audit` rows for `forecast_bucket_assignment`; no CHECK WARN |
| **AZ** | AY1, V1 | `low_confidence` (and extended statuses) persist without CHECK violation |
| **BA** | BA1, BA2, T1, V1 | Recompute completes; `GET /api/v1/forecast/meta` fresh `computation_id` |
| **BB** | V1 | Month-bucket SQL probe; ML selectable or honest `ml_skipped_reason` |
| **BC** | V1 | Planning Compare no **Plan stale** after successful recompute |
| **BD** | BD1, V1 | No false **No forecast data yet** when meta has `computation_id` |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| AY1 | Task **AY1** |
| BA1 | Task **BA1** |
| BA2 | Task **BA2** |
| BD1 | Task **BD1** |
| T1 | Task **T1** |
| V1 | Task **V1** |
| BB month-bucket probe | **V1** operator gate only |
| BC plan stale | **V1** re-smoke only (downstream of BA) |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
