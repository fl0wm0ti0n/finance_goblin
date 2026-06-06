# Sprint S0007 Summary — US-0007 crypto exchange portfolio

**Status:** Released (`0.7.0-us0007`, 2026-06-02)  
**Tasks completed:** 12/12 (T-0073 … T-0084)

## Deliverables

### Database (T-0073)
- Migration `007_exchanges_portfolio.sql`: exchange tables, portfolio PnL/baselines, `fx_rates`, `net_worth_snapshots` crypto columns, `allocation_target` plan enum values
- Seed rows for binance/bybit/bitunix

### Backend config & FX (T-0074)
- TOML `[exchanges]`, `[portfolio]` with env-only secrets pattern
- `FxService` Frankfurter cache + exchange ticker EUR conversion
- Settings API exposes exchange/portfolio sections without secrets

### Exchange connectors (T-0075, T-0076)
- `ExchangeConnector` trait, GET-only HTTP layer with 429 backoff
- Binance (spot + USD-M), Bybit (UNIFIED), Bitunix (spot-first)
- `ExchangeService` factory, test connection, post-sync orchestration

### Portfolio Engine (T-0077)
- Hybrid PnL: exchange-reported + avg-cost fallback
- Baseline capture, reconciliation warning, daily snapshots

### Sync integration (T-0078)
- `"exchanges"` phase before `"alerts"` in full pipeline
- Exchange-only scheduler tick and `POST /api/v1/sync/exchanges/trigger`
- Trade retention prune at startup

### REST API (T-0079)
- `/exchanges`, `/exchanges/{id}/test`, extended `/wealth`, `/wealth/crypto`, `/wealth/history`, `/portfolio/pnl`
- `allocation_target` plan template preset
- Extended `get_portfolio` AI tool with crypto + top-5 holdings (8 KB cap)

### Frontend (T-0080, T-0081)
- `/wealth` Overview + Crypto tabs, PnL row, FX incomplete banner, allocation gap card
- Settings Crypto exchanges section with test connection
- Sync Status per-exchange rows + Sync exchanges now

### Grafana (T-0082)
- Dashboard 4: crypto stat, allocation pie, performance series, FX incomplete panel

### Verification (T-0073–T-0084)
- Unit tests: GET-only audit, signing, PnL hybrid, sync phase ordering, 8 KB cap
- Integration test `exchanges_portfolio_integration` (DATABASE_URL)
- `tests/run-tests.sh` updated
- Operator guide `docs/user-guides/US-0007.md`

## Test results

- `cargo test --lib`: 54 passed
- `cargo test --test exchange_signing`: 4 passed
- `cargo test --test firefly_readonly`: passed
- `npm run build`: passed
- Integration tests skip without `DATABASE_URL`

## Known limitations

- First-sync avg-cost may be inaccurate until trade backfill completes
- CoinGecko FX fallback deferred post-MVP
- Bitunix futures stub when `enabled_futures=false`

## Post-release refresh

- Context compacted 2026-06-02T01:00:00Z (`/refresh-context`); checkpoints archived to `docs/engineering/state-archive/state-pack-20260602-s0007.md`
- Next story: US-0008 (local AI providers); backlog drain active
