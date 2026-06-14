# Q0030 Summary — BUG-0023 Crypto Wealth EUR values missing

**Sprint:** Q0030 (`/quick`)  
**Bug:** BUG-0023  
**Orchestrator:** `auto-20260612-bug0023`  
**Phase:** execute **COMPLETE**  
**Date:** 2026-06-12

## Outcome

Restored Bitunix crypto Wealth EUR display path under existing DEC-0064 / DEC-0080 / DEC-0081 / DEC-0038 contracts:

- **BO:** Futures wallet ingest hardened — equity fallback includes `crossUnrealizedPNL` + `isolationUnrealizedPNL`; `code!=0` rejected; parse-skip emits structured `warn!`; OpenAPI wiremock regression.
- **BP:** Nullable `exposure_eur` column (migration 017); linear `entryValue` → EUR at recompute; `holdings_all.value_eur` maps `market_value_eur.or(exposure_eur)`; subtotal remains wallet-only.
- **BQ:** Baseline captured before `total_return_pct` in same recompute run so first priced sync yields non-null return %.

## Tasks

| ID | Status | Deliverable |
|----|--------|-------------|
| BO1 | done | `bitunix.rs` equity + unrealized key parse |
| BO2 | done | `code==0` validation + parse-skip logging |
| BO3 | done | OpenAPI wiremock + unit tests |
| BP1 | done | Migration 017 + `pnl.rs` `entryValue` persist |
| BP2 | done | `wealth/service.rs` `value_eur` map |
| BQ1 | done | `portfolio/service.rs` baseline order fix |
| T1 | done | `bug0023_crypto_wealth_eur.rs` |
| G1 | done | cargo test + npm build PASS |
| V1 | deferred | Operator BACKEND_DEPLOY + EXCHANGE_SYNC + PNL_RECOMPUTE |

## Test evidence

- `cargo test --lib` → **218/218**
- `cargo test --test bug0023_crypto_wealth_eur` → **4/4**
- `npm run build` → **PASS**

## Next phase

**`/qa`** — fresh subagent; V1 deferred to operator deploy.
