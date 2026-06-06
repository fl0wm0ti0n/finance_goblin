# Design Concept — US-0007

## Summary

US-0007 integrates **Binance, Bybit, and Bitunix** exchange data into household wealth: read-only exchange connectors, Portfolio Engine (realized/unrealized/total return PnL), EUR FX conversion, extended net worth including crypto, allocation planning scenarios, React `/wealth` Crypto tab, Settings exchange status display, Sync Status per-exchange rows, and **Grafana Dashboard 4 completion**.

Builds on US-0005 wealth framework and Alert Engine snapshot hook, US-0004 Plan Engine adjustments, and US-0006 `get_portfolio` tool extension.

## Goals

- Exchange connectors: unified `ExchangeConnector` trait; GET-only; spot + linear/USDT-M; Bitunix spot-first (DEC-0037, R-0032)
- Portfolio Engine: hybrid PnL — exchange fields for derivatives, avg-cost for spot; total return vs first-sync baseline (DEC-0038, R-0033)
- FX: Frankfurter for fiat/stablecoin; exchange tickers for alts; incomplete-FX banner (DEC-0039, R-0034)
- Secrets: TOML env names + Compose env only; Settings read-only; test-connection endpoint (DEC-0040, R-0035)
- Sync: `"exchanges"` phase before `"alerts"`; independent interval + exchange-only trigger (DEC-0041, R-0036)
- Migration 007 + extend `net_worth_snapshots`; `allocation_target` plan adjustment kind (DEC-0042, R-0037)
- React `/wealth` Overview + **Crypto** tab; Settings Crypto exchanges section
- Grafana Dashboard 4: crypto stat, allocation pie, performance time series (R-0026 completion)
- Extend `get_portfolio` with crypto totals + top-5 holdings (R-0031; no new AI tools)

## Non-goals

- Additional exchanges (Kraken, Coinbase, Bitpanda, OKX)
- On-chain wallet tracking; tax reporting; trading execution
- Encrypted DB vault; browser secret entry
- CoinGecko fallback (defer post-MVP)
- Binance Portfolio Margin, options, inverse-only modes
- New AI tools

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0037 | Unified connector trait + GET-only | Consistent signing/backoff; DEC-0004 read-only pattern (R-0032) |
| DEC-0038 | Hybrid PnL | Exchange PnL where available; avg-cost for spot; not tax FIFO (R-0033) |
| DEC-0039 | Two-layer FX | ECB daily for fiat; exchange prices for crypto; banner when incomplete (R-0034) |
| DEC-0040 | Env-only secrets | Mirrors AI/Firefly; acceptance "not transmitted externally" (R-0035) |
| DEC-0041 | Exchanges before alerts | Crypto must be in net-worth snapshot (R-0036) |
| DEC-0042 | Extend snapshots + plan kind | YAGNI vs separate hypertable; allocation via existing plan tables (R-0037) |

**UX references:** Finanzguru Gesamtvermögen with crypto slice; per-exchange connection status; allocation gap vs target; Grafana Dashboard 4 completion — see `docs/product/vision.md` discovery notes US-0007.

**Spec-pack traceability:** `docs/engineering/spec-pack/US-0007-crs.md`, `docs/engineering/spec-pack/US-0007-technical-specification.md`
