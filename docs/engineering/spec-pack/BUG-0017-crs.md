# CRS — BUG-0017 Post-sync forecast recompute cluster

## Purpose

Close UI-audit cluster **UI-002/006/009/010**: post-sync forecast recompute logs WARN on audit CHECK and FK delete failures while sync status stays success. Operators see disabled ML controls, **Plan stale**, and transient Forecast empty state.

## Scope

### In scope

- **DEC-0105:** New migration extending `ai_tool_audit` `tool_name` and `result_status` CHECK constraints
- **DEC-0106:** `ON DELETE CASCADE` on `paired_baseline_id`; `enforce_retention` processes `ml_enhanced` before `baseline`
- **BD:** `ForecastPage.tsx` — empty state gated on `metaQuery.isFetched && !computation_id`
- Integration/retention test for paired baseline+ML delete path
- Verify-work: month-bucket SQL probe for **BB**; Planning Compare re-smoke for **BC**; operator sync smoke matrix (R-0087 §8)

### Out of scope

- Sync fail-on-recompute product change (deferred per R-0050)
- `min_monthly_points` threshold change
- TanStack `placeholderData` / SSR `initialData` for **BD**
- Plan-engine or `forecast_ml` algorithm changes beyond retention unblock

## Acceptance criteria ref

`docs/product/acceptance.md` — BUG-0017:

- **(AY)** Post-sync recompute inserts `ai_tool_audit` rows for `forecast_bucket_assignment` without `ai_tool_audit_tool_name_check` violation
- **(AZ)** Low-confidence bucket assignments persist without `ai_tool_audit_result_status_check` violation
- **(BA)** Recompute delete/insert does not fail on `forecast_computations_paired_baseline_id_fkey`; fresh meta after successful sync
- **(BB)** ML-enhanced selectable when gate passes; accurate `ml_skipped_reason` otherwise
- **(BC)** Planning Compare loses **Plan stale** after successful recompute/sync
- **(BD)** Forecast page does not show false empty when meta has `computation_id`
- OIDC-enabled deploy regression checks pass

## Dependencies

- US-0015 / **DEC-0078** (bucket audit insert path)
- **DEC-0034** (audit schema)
- **DEC-0050**, **DEC-0052** (ML overlay + sync phase order)
- **DEC-0011** (retention count)
- R-0087 research complete
- Q0024 / BUG-0016 released (SPA fallback baseline image)
