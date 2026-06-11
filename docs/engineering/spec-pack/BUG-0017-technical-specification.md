# Technical Specification — BUG-0017

## Overview

Implement **DEC-0105** (audit CHECK migration) and **DEC-0106** (FK CASCADE + retention order) plus **BD** ForecastPage loading contract. **BB** and **BC** verified in verify-work after backend deploy — no additional product code unless probe reveals separate defect.

## Components

| Layer | Change | Decision |
|-------|--------|----------|
| `backend/migrations/0XX_bug0017_*.sql` | CHECK extension + FK CASCADE | DEC-0105, DEC-0106 |
| `backend/src/forecast/repository.rs` | `enforce_retention` kind order: `ml_enhanced` → `baseline` | DEC-0106 |
| `frontend/src/pages/ForecastPage.tsx` | `showEmpty = isFetched && !isError && !computation_id` | BD contract |
| `backend/tests/` or `forecast` module tests | Retention with paired rows | DEC-0106 |
| `forecast/service.rs`, `bucket_inference.rs` | **No change** (insert values already correct) | DEC-0105 |

## Interfaces

### Audit CHECK (DEC-0105)

| Column | Allowed values (post-migration) |
|--------|--------------------------------|
| `tool_name` | Six chat tools + `forecast_bucket_assignment` |
| `result_status` | `ok`, `error`, `low_confidence`, `provider_unavailable`, `parse_error` |

**Verify:** `SELECT tool_name, result_status FROM ai_tool_audit WHERE tool_name = 'forecast_bucket_assignment' LIMIT 5` — rows present post-sync.

### FK retention (DEC-0106)

- **FK:** `forecast_computations.paired_baseline_id` → `forecast_computations(id)` `ON DELETE CASCADE`
- **Loop order:** `["ml_enhanced", "baseline"]` in `enforce_retention()`
- **Expected:** `POST /api/v1/sync/trigger` logs free of FK WARN; `GET /api/v1/forecast/meta` → `stale=false`, fresh `computation_id`

### ForecastPage loading (BD)

```tsx
const showLoading = metaQuery.isPending;
const showEmpty =
  metaQuery.isFetched && !metaQuery.isError && !metaQuery.data?.computation_id;
```

- Show skeleton/loading when `showLoading`
- Show empty card only when `showEmpty`
- Preserve content when `computation_id` present

### BB verification probe (operator gate — verify-work)

```sql
SELECT account_id,
       COUNT(DISTINCT date_trunc('month', date)) AS month_buckets
FROM transactions
WHERE date IS NOT NULL
GROUP BY account_id
ORDER BY month_buckets;
```

| Outcome | Action |
|---------|--------|
| All asset accounts `month_buckets >= 12` | ML gate should pass after **BA**; investigate if still `insufficient_history` |
| Any account `< 12` | BB satisfied with honest skip — do **not** lower `min_monthly_points` |

## Non-functional

- **Deploy:** Migration before backend image; rebuild `flow-finance-ai` + frontend for **BD**
- **Compatibility:** localhost `:18080`, omniflow external profile; OIDC regression unchanged
- **Testing:** `cargo test` retention integration; `npm test` ForecastPage loading guard if unit-tested
- **Regression:** R-0050 sync warn-and-serve semantics preserved (gate 7 deferred)

## Traceability

- DEC-0105, DEC-0106, R-0087, `docs/engineering/architecture.md` § **BUG-0017**
