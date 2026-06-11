# Technical Specification — BUG-0018

## Change surface

| Component | Change |
|-----------|--------|
| `backend/src/alerts/evaluate.rs` | `evaluate_scarcity` SQL string — qualify `fbd.balance`, `fbd.ts` |
| `backend/tests/wealth_alerts_integration.rs` | No code change expected — existing test validates fix |
| Sync / frontend | **No change** |

## SQL contract (DEC-0107)

**Before (defective):**

```sql
SELECT ts::date AS day, SUM(balance::float8) AS balance
FROM forecast_balance_daily fbd
JOIN accounts a ON a.firefly_id = fbd.account_id
WHERE fbd.computation_id = $1
  AND a.type = 'asset'
  AND COALESCE((a.payload->>'include_net_worth')::boolean, true) = true
  AND ts::date >= $2 AND ts::date <= $3
GROUP BY ts::date
ORDER BY day
```

**After (frozen):**

```sql
SELECT fbd.ts::date AS day, SUM(fbd.balance::float8) AS balance
FROM forecast_balance_daily fbd
JOIN accounts a ON a.firefly_id = fbd.account_id
WHERE fbd.computation_id = $1
  AND a.type = 'asset'
  AND COALESCE((a.payload->>'include_net_worth')::boolean, true) = true
  AND fbd.ts::date >= $2 AND fbd.ts::date <= $3
GROUP BY fbd.ts::date
ORDER BY day
```

## Pipeline behavior (unchanged)

```text
sync/mod.rs alerts phase
  → WealthService::upsert_snapshot
  → AlertService::run_post_sync
      → evaluate_scarcity  ← fix target (runs FIRST)
      → evaluate_budget_drift
      → evaluate_plan_viability
      → upsert / resolve candidates
```

On SQL error: `sync/mod.rs` L413–414 logs **`warn!`**; sync status remains **`success`**.

## Test plan

| Task | Command / probe | Expected |
|------|-----------------|----------|
| **BE1** | Code review + `cargo test --lib alerts::evaluate` | Qualified columns only |
| **T1** | `DATABASE_URL=… cargo test --test wealth_alerts_integration` | `wealth_snapshot_and_scarcity_alert_on_post_sync` PASS |
| **V1** | Full sync + API probes | No 42702; wealth alerts non-empty when rules match |

## Deploy

1. Merge BE1 backend change
2. Rebuild `flow-finance-ai` image (**BACKEND_FRONTEND_DEPLOY**)
3. **FULL_FIREFLY_SYNC**
4. V1 operator smoke matrix (architecture § BUG-0018)

## Regression boundaries

- Do **not** change scarcity threshold, fingerprint format, or severity logic
- Do **not** modify `evaluate_budget_drift` / `evaluate_plan_viability`
- Do **not** change subscription alert sync phase or dedup (BUG-0008)
