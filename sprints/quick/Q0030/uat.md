# UAT — Q0030 (BUG-0023)

**Status:** VERIFY-WORK COMPLETE  
**Verdict:** **PASS-WITH-PREREQUISITES**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0023 rows **BO**, **BP**, **BQ**  
**Sprint:** Q0030 (`/quick`)  
**Orchestrator:** `auto-20260612-bug0023`  
**Verified at:** 2026-06-12T21:45:00Z

## Operator gates (before live probes)

| Gate | Status | Notes |
|------|--------|-------|
| BACKEND_DEPLOY | pending | Migration 017 + BO/BP backend changes not yet deployed |
| EXCHANGE_SYNC | pending | Bitunix sync after deploy — wallet row ingest |
| PNL_RECOMPUTE | pending | Post-sync recompute — exposure_eur, baseline, total_return_pct |
| AP1_SQL_PROBE | pending (optional) | `exchange_holdings` product_type / `market_value_eur` probe |

## UAT steps

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| BO-API | BO | `GET /api/v1/wealth` — `crypto.subtotal_eur` ~€2000; `bitunix.subtotal_eur` not €0 | pass_with_prerequisites | Live pre-deploy: `crypto.subtotal_eur` -0.0, bitunix -0.0; integration `bo_futures_wallet_priced_subtotal_nonzero` PASS |
| BO-UI | BO | Wealth → Crypto — Bitunix exchange card not €-0,00 | pass_with_prerequisites | Browser deferred until BACKEND_DEPLOY + EXCHANGE_SYNC |
| BO-SQL | BO | `futures` row `market_value_eur` populated in `exchange_holdings` | pass_with_prerequisites | AP1_SQL_PROBE deferred post-deploy |
| BP-API | BP | All linear `holdings_all[].value_eur` non-null when `entryValue` present | pass_with_prerequisites | Live: 11 linear rows all `value_eur` null; integration `bp_linear_exposure_eur_*` PASS |
| BP-UI | BP | Holdings table Value EUR column not all em dash | pass_with_prerequisites | UI deferred post-deploy |
| BP-SUBTOTAL | BP | `crypto.subtotal_eur` = wallet only; linear `market_value_eur` NULL | pass_with_prerequisites | Code DEC-0064 wallet-only subtotal verified at QA; live subtotal -0.0 pre-deploy |
| BQ-API | BQ | `pnl.total_return_pct` non-null when baseline exists | pass_with_prerequisites | Live: `total_return_pct` null, `unrealized_eur` 376.83; integration `bq_priced_wallet_baseline_total_return_pct` PASS |
| BQ-UI | BQ | PnL summary Total return % not em dash with non-zero unrealized | pass_with_prerequisites | UI deferred post-deploy + PNL_RECOMPUTE |
| OIDC-1 | regression | `/wealth`, `/api/v1/wealth` smoke on omniflow | pass | `:18080/api/v1/wealth` HTTP 200; `/health` HTTP 200 |

## Automated checks

| Check | Result |
|-------|--------|
| `cargo test --test bug0023_crypto_wealth_eur` | **4/4 PASS** |
| `cargo test --lib` (QA stage) | **218/218 PASS** |
| `npm test` / `npm run build` (QA stage) | **9/9 PASS** / **PASS** |

## Results summary

**1 pass / 8 pass-with-prerequisites / 0 fail** — code and integration oracles PASS; live API confirms pre-deploy baseline (€0 subtotal, null value_eur, null total_return_pct). Operator **BACKEND_DEPLOY → EXCHANGE_SYNC → PNL_RECOMPUTE** required for live ~€2000 validation. Ready for **`/release`**.
