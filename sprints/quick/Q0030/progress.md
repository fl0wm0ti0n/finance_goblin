# Q0030 progress



**Sprint:** Q0030 (BUG-0023)  

**Status:** QA PASS — ready for `/verify-work`  

**Last updated:** 2026-06-12 (qa, `auto-20260612-bug0023`)  

**Orchestrator:** `auto-20260612-bug0023`



## Task status



| ID | Status | Priority | Notes |

|----|--------|----------|-------|

| BO1 | done | P0 | Equity fallback + unrealized key aliases |

| BO2 | done | P0 | `code==0` validation + parse-skip `warn!` |

| BO3 | done | P0 | OpenAPI wiremock + unit tests (5 new cases) |

| BP1 | done | P1 | Migration 017 + `entryValue` → `exposure_eur` |

| BP2 | done | P1 | `holdings_all.value_eur = market_value_eur.or(exposure_eur)` |

| BQ1 | done | P1 | Baseline capture before `total_return_pct` (order fix) |

| T1 | done | P0 | `bug0023_crypto_wealth_eur.rs` — 4 integration cases |

| G1 | done | P0 | Automated gate PASS |

| V1 | open | P0 | verify-work — blocked on BACKEND_DEPLOY (qa PASS at code+test gate) |



## G1 automated gate (2026-06-12)



```

cargo test --lib

  → 218 passed / 0 failed (+5 vs plan-verify baseline 213)



cargo test --test bug0023_crypto_wealth_eur

  → 4 passed / 0 failed (SKIP when DATABASE_URL unset — same harness as bug0021)



npm run build

  → PASS (tsc --noEmit && vite build)

```



### git diff --stat (implementation blast radius)



```

backend/migrations/017_bug0023_exposure_eur.sql          |  4 ++

backend/src/exchanges/bitunix.rs                        | 219 ++++-

backend/src/exchanges/repository.rs                     |   7 +-

backend/src/portfolio/pnl.rs                            |  35 +

backend/src/portfolio/service.rs                        |  14 +-

backend/src/wealth/service.rs                          |   2 +-

backend/tests/bug0023_crypto_wealth_eur.rs             | 362 +++++++++

backend/tests/exchanges_portfolio_integration.rs         |   2 +-

```



No forbidden paths touched (subtotal merge, mark-price tier-2, Grafana).



## Operator gates (before V1)



1. **BACKEND_DEPLOY** — ship BO+BP; apply migration `017_bug0023_exposure_eur.sql`

2. **EXCHANGE_SYNC** — Bitunix full/exchange sync success

3. **PNL_RECOMPUTE** — post-sync recompute for `exposure_eur` + wallet pricing

4. **AP1_SQL_PROBE** (optional) — SQL probe per architecture § BUG-0023



## QA gate (2026-06-12)

```
cargo test --lib
  → 218 passed / 0 failed

cargo test --test bug0023_crypto_wealth_eur
  → 4 passed / 0 failed (DATABASE_URL present — all live seed paths executed)

npm test
  → 9 passed / 0 failed

npm run build
  → PASS
```

**Verdict:** PASS — 0 blockers. Code review aligned with DEC-0064, DEC-0080, DEC-0081, DEC-0038. See `sprints/quick/Q0030/qa-findings.md`.

## V1 verify-work

_Pending operator deploy (BACKEND_DEPLOY → EXCHANGE_SYNC → PNL_RECOMPUTE)._

