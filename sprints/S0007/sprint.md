# Sprint S0007

**ID:** S0007  
**Story:** US-0007 — Crypto exchange portfolio integration  
**Status:** PLANNED  
**Created:** 2026-06-01

## Goal

Deliver crypto exchange portfolio integration on top of US-0001–US-0006: migration 007 schema, `ExchangeConnector` implementations (Binance/Bybit/Bitunix), `FxService` EUR conversion, `PortfolioEngine` hybrid PnL, sync `"exchanges"` phase before alerts, extended wealth/portfolio REST API, React `/wealth` Crypto tab, Settings read-only exchange section, Sync Status per-exchange rows, plan `allocation_target` template, Grafana Dashboard 4 completion, tests, and operator user guide.

## Scope

- Migration `007_exchanges_portfolio.sql` — exchange + portfolio + fx tables; alter `net_worth_snapshots` (DEC-0042)
- Config: TOML `[exchanges]`, `[portfolio]`; env-only secrets; extend `GET /api/v1/settings` (DEC-0040)
- `FxService`: Frankfurter fiat/stablecoin + exchange ticker alts; `fx_incomplete` flag (DEC-0039)
- `ExchangeConnector` trait + Binance/Bybit/Bitunix GET-only implementations (DEC-0037)
- `PortfolioEngine`: hybrid realized/unrealized/total return; baselines; avg-cost fallback (DEC-0038)
- Sync pipeline: `"exchanges"` phase before `"alerts"`; exchange-only partial trigger (DEC-0041)
- REST API: exchanges list/test/trigger; extended `/wealth`, `/wealth/crypto`, `/portfolio/pnl`
- Plan Engine: `allocation_target` adjustment kind + template preset (DEC-0042)
- React: `/wealth` Overview + Crypto tab; Settings crypto exchanges; Sync Status per-exchange rows
- Grafana Dashboard 4 completion: crypto stat, allocation pie, performance series (R-0026)
- Extend `get_portfolio` AI tool with crypto totals + top-5 holdings (DEC-0035)
- Tests and operator user guide (`docs/user-guides/US-0007.md`)

**Out of scope:** Additional exchanges (Kraken, Coinbase, OKX), on-chain wallets, tax FIFO, trading execution, encrypted DB vault, browser secret entry, CoinGecko fallback, new AI tools, Firefly writes.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Binance per-symbol trade fan-out | Limit to active symbols; log phase timing; longer exchange interval | R-0032, R-0036 |
| Bitunix signing complexity | Spot-first; futures stub default off; mock-server test early | R-0032 |
| Avg-cost inaccuracy on first sync | 90-day backfill; prefer exchange PnL; document in UI | R-0033 |
| FX incomplete for illiquid alts | Exclude from subtotal + banner | R-0034 |
| Mutex duration growth | Log exchanges phase separately; recommend longer interval in guide | R-0036, DEC-0018 |
| Secret rotation requires restart | Document env + restart flow in user guide | R-0035 |
| PnL reconciliation drift | Prefer exchange cumulative; warning on Sync Status | R-0033 |
| `get_portfolio` 8 KB overflow | Top-5 holdings + summarize series | R-0031, DEC-0035 |

## Definition of Done

- All 12 sprint tasks complete (`T-0073` … `T-0084`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0007
- Binance/Bybit/Bitunix connectors import balances, positions, trades, transfers, funding via GET-only HTTP
- Portfolio Engine computes realized, unrealized, and total return
- Net worth headline includes crypto EUR subtotal; placeholder removed when connected
- Allocation target template compares current vs target weights
- Grafana Dashboard 4 shows crypto slice and portfolio performance
- API keys env-only — never in API responses, browser, or audit
- User guide published at `docs/user-guides/US-0007.md`
- No Firefly write operations introduced

## Architecture references

- `docs/engineering/architecture.md` — US-0007
- Decisions: DEC-0037 … DEC-0042
- Research: R-0032 … R-0037
- Depends on: US-0001 sync mutex; US-0005 `WealthService` + Dashboard 4 partial; US-0004 Plan Engine; US-0006 `get_portfolio` tool
