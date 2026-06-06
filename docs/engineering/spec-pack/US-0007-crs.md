# CRS — US-0007

## Purpose

Integrate Binance, Bybit, and Bitunix crypto exchange balances and PnL into household net worth, enabling complete wealth visibility and allocation planning scenarios.

## Scope

**In scope:** Read-only exchange connectors (3 start set); Portfolio Engine (realized/unrealized/total return); crypto slice in net worth (US-0005 extension); allocation target planning template; Grafana Dashboard 4 completion; env-only API key storage; React `/wealth` Crypto tab; Settings exchange status; Sync Status per-exchange rows.

**Out of scope:** Additional exchanges; on-chain wallets; tax reporting; trading; encrypted DB vault; browser credential entry; CoinGecko fallback (post-MVP).

See `docs/product/backlog.md#us-0007` for full boundaries.

## Acceptance criteria ref

`docs/product/acceptance.md` — section **US-0007** (6 criteria):

1. Connectors for Binance, Bybit, Bitunix import balances, positions, trades, transfers, funding, PnL
2. Portfolio Engine calculates realized, unrealized, and total return
3. Crypto holdings included in net worth view (extends US-0005)
4. Portfolio allocation scenarios supported (e.g. 50% ETF / 50% crypto)
5. Grafana Dashboard 4 shows crypto slice and portfolio performance
6. API keys stored in self-hosted secrets/config, not transmitted externally

## Dependencies

- US-0001 platform, sync mutex, Settings shell
- US-0005 `WealthService`, `net_worth_snapshots`, Dashboard 4 partial
- US-0004 Plan Engine `plan_adjustments`
- US-0006 `get_portfolio` tool (extends only)

## Decisions

DEC-0037 … DEC-0042 — see `docs/engineering/decisions.md`

## Research

R-0032 … R-0037 — see `docs/engineering/research.md`
