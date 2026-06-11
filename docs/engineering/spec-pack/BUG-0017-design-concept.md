# Design Concept — BUG-0017

## Summary

BUG-0017 fixes post-sync forecast recompute failures caused by two confirmed backend defects: `ai_tool_audit` CHECK gaps (**AY**/**AZ**) and `paired_baseline_id` FK retention order (**BA**). A frontend loading-contract fix (**BD**) eliminates false empty flash. **BB** (ML gate) and **BC** (plan stale) resolve via verify-work after backend fixes — no threshold lowering.

## Goals

- **AY/AZ:** `forecast_bucket_assignment` and extended `result_status` values persist in `ai_tool_audit` without CHECK violations
- **BA:** Recompute delete/insert completes; `GET /api/v1/forecast/meta` reflects fresh computation after successful sync
- **BB:** ML-enhanced selectable when gate passes; honest `ml_skipped_reason` when history genuinely sparse
- **BC:** Planning Compare loses **Plan stale** badge after successful recompute
- **BD:** Forecast page does not show **No forecast data yet** when meta already has `computation_id`
- OIDC-enabled deploy regression checks pass

## Non-goals

- Lowering `min_monthly_points` to mask sparse history
- Sync phase fail-on-recompute (deferred — keep R-0050 warn-and-serve)
- Separate `forecast_bucket_audit` table or PostgreSQL ENUM migration
- Plan-engine changes for **BC** (downstream of **BA**)

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0105 | DROP+ADD CHECK migration | Preserves audit fidelity; R-0087 §2 |
| DEC-0106 | CASCADE + ml-before-baseline order | Robust FK + defense in depth; R-0087 §3 |
| BD contract | `isFetched` empty guard | Minimal TanStack Query fix; R-0087 §6 |
| BB gate | Month-bucket SQL probe post-deploy | Verify-after-fix; do not change threshold |
| Sprint shape | `/quick` ≤6 tasks | Backend migrations + repository + ForecastPage |

**Spec-pack traceability:** `docs/engineering/spec-pack/BUG-0017-crs.md`, `docs/engineering/spec-pack/BUG-0017-technical-specification.md`
