# QA Findings — Quick Q0030 / BUG-0023

**Work item:** BUG-0023 (defect)  
**Quick task:** Q0030  
**QA phase:** `/qa`  
**Date:** 2026-06-12  
**Orchestrator:** `auto-20260612-bug0023`  
**Decisions:** DEC-0064, DEC-0080, DEC-0081, DEC-0038  
**QA agent:** fresh subagent (`qa-20260612-bug0023-qa-fresh`)

## Verdict

**PASS** — Independent re-run confirms BO/BP/BQ implementation under frozen architecture gates: Bitunix wallet ingest hardening (equity fallback + `code==0` reject + parse-skip warn + wiremock); migration 017 `exposure_eur` + linear `entryValue` persist; `holdings_all.value_eur = market_value_eur.or(exposure_eur)` with wallet-only subtotal (DEC-0064); baseline captured before `total_return_pct` in same recompute (DEC-0038). Automated gates **218/218** lib, **4/4** bug0023 integration (with `DATABASE_URL`), **9/9** npm, **PASS** build. **V1** live smoke (BACKEND_DEPLOY → EXCHANGE_SYNC → PNL_RECOMPUTE → `:18080` oracle) deferred to verify-work — pass-with-prerequisites.

**Blockers:** 0

## Scope

**Inputs reviewed:** `handoffs/dev_to_qa.md` (Q0030 top section), `sprints/quick/Q0030/{summary,progress}.md`, `docs/product/acceptance.md` BUG-0023 row (BO/BP/BQ), `docs/engineering/architecture.md` § BUG-0023, `backend/migrations/017_bug0023_exposure_eur.sql`, `backend/src/exchanges/bitunix.rs`, `backend/src/exchanges/repository.rs`, `backend/src/portfolio/pnl.rs`, `backend/src/portfolio/service.rs`, `backend/src/wealth/service.rs`, `backend/tests/bug0023_crypto_wealth_eur.rs`. No host `.env`/secret files read.

## Test plan

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | Lib unit suite | `cargo test --lib` | **PASS** — 218/218 (1.73s) |
| T-2 | BUG-0023 integration suite | `cargo test --test bug0023_crypto_wealth_eur` | **PASS** — 4/4 (0 skipped; `DATABASE_URL` present) |
| T-3 | Frontend unit suite | `npm test` | **PASS** — 9/9 |
| T-4 | Frontend build | `npm run build` | **PASS** — tsc + vite build |
| T-5 | BO equity fallback | `bitunix.rs` L124–141 + unit `parse_futures_wallet_openapi_equity_fallback` | **PASS** — sum includes `crossUnrealizedPNL` + `isolationUnrealizedPNL` |
| T-6 | BO code validation | `bitunix_response_ok` + `sync_balances` L397–407 + test `sync_balances_futures_wallet_code_nonzero_no_row` | **PASS** — `code!=0` rejects wallet row |
| T-7 | BO parse-skip warn | `warn_futures_wallet_parse_skip` L282–308 | **PASS** — structured `warn!` with margin_coin, components, derived_sum |
| T-8 | BO OpenAPI wiremock | `sync_balances_futures_wallet_openapi_sample` | **PASS** — futures row qty=2000, product_type=futures |
| T-9 | BP migration + persist | `017_bug0023_exposure_eur.sql` + `pnl.rs` L42–60 + integration `bp_linear_exposure_eur_*` | **PASS** — `exposure_eur` populated; `market_value_eur` NULL for linear |
| T-10 | BP wealth mapping | `wealth/service.rs` L159, L167 | **PASS** — subtotal wallet-only; `holdings_all.value_eur` uses `or(exposure_eur)` |
| T-11 | BQ baseline order | `portfolio/service.rs` L56–78 + integration `bq_priced_wallet_baseline_total_return_pct` | **PASS** — baseline after `compute_hybrid_pnl`; non-null `total_return_pct` on first priced recompute |
| T-12 | Regression shape | `regression_wealth_list_shape_unchanged` | **PASS** — holdings_all cap 50; required fields present |
| T-13 | V1 operator smoke | BACKEND_DEPLOY + EXCHANGE_SYNC + PNL_RECOMPUTE + `:18080` | **DEFERRED** — verify-work |

### T-1 output

```
test result: ok. 218 passed; 0 failed; 0 ignored; 0 measured
```

### T-2 output

```
running 4 tests
test bo_futures_wallet_priced_subtotal_nonzero ... ok
test bp_linear_exposure_eur_value_without_subtotal_merge ... ok
test bq_priced_wallet_baseline_total_return_pct ... ok
test regression_wealth_list_shape_unchanged ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured
```

## Code review vs decisions

| Decision | Contract | Review |
|----------|----------|--------|
| **DEC-0064** | Subtotal = sum wallet `market_value_eur` only; linear excluded | **PASS** — `wealth/service.rs` L127–131, L159; `pnl.rs` L56 sets `market_value_eur: None` for linear |
| **DEC-0080** | Bitunix futures wallet parse + linear unrealized EUR | **PASS** — `bitunix.rs` equity keys + unrealized aliases; `pnl.rs` unrealized conversion for linear |
| **DEC-0081** | `holdings_all` display surface; unified FX gate | **PASS** — L167 `value_eur` mapping; cap 50 unchanged; `fx_incomplete` gate preserved |
| **DEC-0038** | Total return denominator = wallet-priced `crypto_value_eur`; baseline on first priced sync | **PASS** — `portfolio/service.rs` baseline capture after pricing loop; integration asserts non-null return % |

## Acceptance row status (qa-stage)

| Row | qa-stage evidence | Status |
|-----|-------------------|--------|
| **BO** | Wallet parse hardening + integration `bo_futures_wallet_priced_subtotal_nonzero` + wiremock/unit tests | **PASS** at qa — live ~€2000 subtotal deferred V1 |
| **BP** | `exposure_eur` migration + pnl persist + integration `bp_linear_exposure_eur_*` + wealth mapping | **PASS** at qa — live Value EUR column deferred V1 |
| **BQ** | Baseline order fix + integration `bq_priced_wallet_baseline_total_return_pct` | **PASS** at qa — live Total return % deferred V1 |

## Non-blocking notes (carry to verify-work)

- `holdings_top` still filters on `market_value_eur` only — linear rows appear in `holdings_all` but not top-5 card; consistent with DEC-0064 wallet-only subtotal (not a defect).
- V1 requires operator sequence: **BACKEND_DEPLOY** (migration 017) → **EXCHANGE_SYNC** (Bitunix) → **PNL_RECOMPUTE** → smoke `GET /api/v1/wealth` on `:18080`.
- Implementation files remain uncommitted per sprint policy.

## Handoff

- **Next phase:** `/verify-work` (role: qa)
- **No return items** — `handoffs/qa_to_dev.md` unchanged (PASS; 0 blockers)

`fresh_context_marker`: qa-20260612-bug0023-qa-fresh  
`runtime_proof_id`: runtime-proof-qa-20260612-bug0023-001  
`phase_boundary`: qa → verify-work
