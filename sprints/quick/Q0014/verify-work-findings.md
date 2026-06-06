# Verify-work Findings — Quick Q0014+Q0015 / BUG-0012

**Work item:** BUG-0012  
**Quick tasks:** Q0014 (DEC-0067), Q0015 (follow-up)  
**Phase:** `/verify-work` (re-run)  
**Date:** 2026-06-06  
**Verdict:** **PASS** — omniflow runtime rows **(AG)** / **(AH)** satisfied; release unblocked

## Summary

Re-ran verify-work against `https://financegnome.omniflow.cc` after Q0014+Q0015 deploy and Full Firefly sync. Monthly forecast for funded account **114** (Raiffeisenbank Giro) shows **non-zero Fixed** (Jun **2073.85**) and **non-zero Income** from Jul onward (**3266.16**, projected salary from revenue account 33). Pre-fix pattern (all Variable, Income/Fixed permanently 0) **resolved**. Code **142/142** unit tests PASS.

## Per-row verdict (acceptance AG / AH)

| Row | Verdict | Evidence |
|-----|---------|----------|
| **(AG)** | **PASS** | `income > 0` from **2026-07** (`3266.16`); Jun `0.00` expected — no June salary due in projection window (last mirror pay 2026-04-30); operator confirmed recurring projection acceptable |
| **(AH)** | **PASS** | Jun `fixed_costs: 2073.85` (strom/subscriptions); Jul+ `fixed_costs > 0` |
| Regression | **DEFERRED** | OIDC browser smoke deferred (external dev-bypass); API health/sync PASS |

## Live curl evidence (2026-06-06)

| Probe | HTTP | Result |
|-------|------|--------|
| `GET /health` | 200 | OK |
| `GET /api/v1/forecast/meta` | 200 | `computed_at: 2026-06-05T21:39:42Z`, sync `1cc59f24` |
| `GET /api/v1/forecast/monthly?account_id=114` | 200 | **PASS** — see sample rows |

### Sample rows (account 114)

```json
{"month":"2026-06-01","income":"0.00","fixed_costs":"2073.85","variable_costs":"3944.85","free_cashflow":"-6018.70"}
{"month":"2026-07-01","income":"3266.16","fixed_costs":"652.09","variable_costs":"4752.53","free_cashflow":"-2138.46"}
```

## Code verification

| Check | Result |
|-------|--------|
| `cargo test --lib` | **142/142 PASS** |
| DEC-0067 + Q0015 payee/income/standing-order fixes | Deployed |

## Acceptance impact

| Row | Verify-work | `acceptance.md` |
|-----|-------------|-----------------|
| **(AG)** | **PASS** | Check on release |
| **(AH)** | **PASS** | Check on release |

## Next phase

**`/release`** — Q0014+Q0015 release notes, backlog BUG-0012 → DONE.
