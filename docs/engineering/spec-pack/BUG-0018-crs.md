# CRS — BUG-0018 Alert evaluation SQL failure (balance ambiguous)

## Purpose

Close ui-audit **UI-003**: post-sync alert evaluation logs PostgreSQL **42702** (`column reference "balance" is ambiguous`) in `evaluate_scarcity`, aborting the wealth alert pass. Operators see permanent **No active alerts** despite overdrawn accounts and matching scarcity rules.

## Scope

### In scope

- **DEC-0107:** Qualify `fbd.balance` and `fbd.ts` in `evaluate_scarcity` daily aggregate SQL
- **T1:** `wealth_alerts_integration` regression gate (existing test; PASS when `DATABASE_URL` set)
- **V1:** Operator smoke — sync logs clean; wealth alerts API + header bell; subscription dedup regression per BUG-0008

### Out of scope

- Sync fail-on-alert-error product change (preserve R-0024)
- Frontend AlertBell/AlertsPage error-state UX
- Subscription alert evaluation SQL (separate phase)
- Migration or schema change
- `cargo sqlx prepare` / static grep lint (defer per R-0088)
- CI DB service container (optional stretch)

## Acceptance criteria ref

`docs/product/acceptance.md` — BUG-0018:

- **(BE)** Post-sync alert evaluation completes without SQL error (`balance` ambiguous / **42702**); logs show no `alert evaluation failed` for normal sync runs
- **(BF)** Header Alerts panel and `GET /api/v1/subscriptions/alerts` surface matching alerts when overdraft or subscription rules apply—not permanent **No active alerts** due to evaluation skip. OIDC-enabled deploy regression checks pass.

**BF proof split (architecture):** wealth inbox (`GET /api/v1/alerts`) is **primary** validation; subscription path is **regression gate** only (not blocked by this defect per R-0068).

## Dependencies

- US-0005 / **DEC-0026** (scarcity evaluation rules — R-0022)
- **DEC-0028** (sync alerts phase — R-0024)
- BUG-0008 / **DEC-0071** (subscription dedup — regression gate)
- R-0088 research complete
- Q0025 / BUG-0017 released (forecast recompute baseline)
