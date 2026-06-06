# Verify-work Findings — Quick Q0016 / BUG-0009

**Work item:** BUG-0009  
**Quick task:** Q0016  
**Phase:** `/verify-work`  
**Date:** 2026-06-06  
**Orchestrator:** `auto-20260606-bug0009-001`  
**Verdict:** **PASS** — omniflow runtime rows **(Y)** / **(Z)** satisfied; release unblocked

## Summary

Ran V1 omniflow smoke after Grafana deploy + **GRAFANA_PROVISIONING_RELOAD** on `https://financegnome.omniflow.cc`. Provisioning fixes (DEC-0068) are live: ABS(balance) default account selection, portfolio LATERAL 3-row overview, ML status banner. Pre-fix pattern (default acct 116 flat zeros; portfolio breakdown 1 row) **resolved**.

## Per-row verdict (acceptance Y / Z)

| Row | Verdict | Evidence |
|-----|---------|----------|
| **(Y)** | **PASS** | Default `$account_id` **114**; cashflow series non-flat (731 distinct balance values); ML banner panel 13 in live dashboard; `ml_enhanced` count 0 with `noValue` on ML panels |
| **(Z)** | **PASS** | Overview table **3** rows + `total_eur` **-3395.75** stat; panel title **All accounts (latest snapshot)** at y=4 |
| Regression | **PASS** | Six embed routes **200**; ds/query **200** for representative panels including UNION pie |

## Live probe evidence (2026-06-06)

| Probe | HTTP | Result |
|-------|------|--------|
| `GET /health` | 200 | OK |
| `GET /analytics/grafana/api/health` | 200 | database ok |
| Account variable ABS sort | 200 | 114, 116, 115 ordered by ABS(balance) DESC |
| Cashflow at default 114 | 200 | min **-132348.57**, max **-3395.75**, cnt **731** |
| Portfolio overview LATERAL | 200 | 3 rows |
| Portfolio `total_eur` | 200 | **-3395.75** |
| Portfolio UNION pie | 200 | Firefly + Crypto slices |
| Forecast baseline path | 200 | Non-empty at 114 |
| Subscriptions confirmed count | 200 | **3** |
| Budgets active plan | 200 | test v1 |
| Platform-health panels | 200 | 3/3 panel SQL execute |
| Live dashboard API | 200 | LATERAL SQL, ML banner, no `current` on account_id |

## Code verification

| Check | Result |
|-------|--------|
| `cargo test --test grafana_provisioning_bug0009` | **6/6 PASS** |

## Acceptance impact

| Row | Verify-work | `acceptance.md` |
|-----|-------------|-----------------|
| **(Y)** | **PASS** | Checked |
| **(Z)** | **PASS** | Checked |

## Next phase

**`/release`** — Q0016 release notes, backlog BUG-0009 → DONE.
