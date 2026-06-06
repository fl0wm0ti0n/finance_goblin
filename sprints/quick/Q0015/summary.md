# Q0015 — BUG-0012 follow-up (forecast monthly buckets completion)

**Parent:** BUG-0012 / Q0014  
**Status:** executed

## Fixes

1. **AH — payee-key matching:** `is_recurring_due` matches via normalized `payee_key`, not raw description (strom/Miete due-days fire).
2. **AH — standing-order rejections:** Forecast excludes only **subscription** rejections; rejected standing orders (e.g. strom) still project.
3. **AH — amount stability:** Recurrence scoring uses recent 6-tx window (ignores old amount outliers).
4. **AG — household income:** Revenue-account recurring inflows (salary) merge into asset-account **monthly** Income bucket without changing asset daily balance.
5. **Config:** German `category_buckets` + standing-order patterns (`strom`, `versorgung`, `wohnen`).
6. **AG — payroll payee collapse:** `Lohn/Gehalt …/YYYYMM` normalizes to `lohn/gehalt` for inflow recurrence grouping.

## Production smoke (account 114, post-deploy)

| Month | Income | Fixed |
|-------|--------|-------|
| 2026-06 | 0.00* | **2073.85** |
| 2026-07 | **3266.16** | **652.09** |

\*June income 0 — salary due projects from July onward (last mirror pay date 2026-04-30).

## Tests

`cargo test --lib` — 142/142 PASS.
