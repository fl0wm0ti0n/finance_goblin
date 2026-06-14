# Technical Specification — BUG-0023

## Overview

Fix Bitunix futures wallet ingest (BO), add display-only linear `exposure_eur` from `entryValue` (BP), and verify total return path resolves when wallet is priced (BQ). Extends **DEC-0064** / **DEC-0080** — no subtotal contract change.

## Components

| Layer | Change | Gate |
|-------|--------|------|
| `backend/src/exchanges/bitunix.rs` | Wallet equity keys, `code==0`, parse-skip logging | BO |
| `backend/migrations/017_bug0023_exposure_eur.sql` | Nullable `exposure_eur` on `exchange_holdings` | BP |
| `backend/src/portfolio/pnl.rs` | Linear `entryValue` → `exposure_eur` at recompute | BP |
| `backend/src/exchanges/repository.rs` | Persist/load `exposure_eur` | BP |
| `backend/src/wealth/service.rs` | `value_eur = market_value_eur.or(exposure_eur)`; subtotal unchanged | BP |
| `backend/src/portfolio/service.rs` | Baseline + total return verify | BQ |
| `backend/tests/` | BO wiremock, BP/BQ integration | T1 |

## Interfaces

### Wallet ingest (BO)

- **Endpoint:** `GET /api/v1/futures/account?marginCoin=USDT`
- **Success:** `code == 0` and parse yields `product_type=futures` row with `market_value_eur` after recompute
- **Failure:** `warn!` diagnostic when parse returns `None`; must not fail positions sync

### Wealth API (unchanged shape)

- `GET /api/v1/wealth` → `crypto.subtotal_eur` = `sum(market_value_eur)` (wallet only)
- `crypto.holdings_all[].value_eur` = wallet `market_value_eur` OR linear `exposure_eur`
- `crypto.holdings_all[].value_eur` for linear must **not** affect subtotal
- `pnl.total_return_pct` non-null when `crypto_value_eur > 0` and baseline captured

### SQL probe (operator)

```sql
SELECT product_type, asset, quantity, market_value_eur, exposure_eur, unrealized_pnl_eur
FROM exchange_holdings WHERE exchange_id = 'bitunix' ORDER BY product_type, asset;
```

## Non-functional

- **Compatibility:** localhost `:18080`, omniflow external profile; OIDC smoke unchanged
- **Migration:** Nullable column; backward compatible
- **Testing:** OpenAPI wiremock fixture; integration BO/BP/BQ; `cargo test` + `npm run build`
- **Observability:** Wallet parse skip must emit structured warn (no silent failure)

## Traceability

- [R-0093 §5](docs/engineering/research.md#r-0093--bug-0023-crypto-wealth-eur-values-live-regression)
- `docs/engineering/architecture.md` § **BUG-0023**
- **DEC-0064**, **DEC-0080**, **DEC-0081**, **DEC-0038** (no new DEC)
