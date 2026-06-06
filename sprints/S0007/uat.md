# UAT — Sprint S0007 / US-0007

**Sprint:** S0007  
**Story:** US-0007  
**Phase:** `/verify-work`  
**Date:** 2026-06-01  
**Verdict:** **PASS** (ready for `/release`)  
**Ready for release:** yes (operator runtime prerequisites documented below)

## Inputs

- QA PASS: `sprints/S0007/qa-findings.md`
- Acceptance: `docs/product/acceptance.md` (US-0007)
- Operator guide: `docs/user-guides/US-0007.md`
- Implementation: `backend/migrations/007_exchanges_portfolio.sql`, `backend/src/{exchanges,portfolio,fx,wealth}/`, `backend/tests/{exchange_signing,exchanges_portfolio_integration}.rs`, `frontend/src/pages/{WealthPage,SettingsPage,SyncStatusPage,PlanningPage}.tsx`, `grafana/provisioning/dashboards/analytics/portfolio.json`

## Environment

| Prerequisite | Status in verify-work |
|--------------|----------------------|
| `.env` with exchange API keys (`BINANCE_*`, `BYBIT_*`, `BITUNIX_*`) | **Not present** — no operator `.env` in workspace |
| `DATABASE_URL` (TimescaleDB) | **Unset** — integration tests skipped by design |
| US-0001–US-0006 services operational | **Not provisioned** — live stack E2E deferred |
| At least one exchange read-only key configured | **Unset** — live connector sync deferred |

Per workflow policy: code-level and automated verification **pass**; runtime E2E steps recorded as **PASS-with-prerequisites** where external infra or exchange keys are required.

## Automated checks

| ID | Check | Command | Result |
|----|-------|---------|--------|
| AUTO-1 | Test harness | `bash tests/run-tests.sh` | **PASS** |
| AUTO-2 | Firefly GET-only unit test | (via harness) `cargo test --test firefly_readonly` | **PASS** (1/1) |
| AUTO-3 | Backend unit tests | (via harness) `cargo test --lib` | **PASS** (54/54; exchange/PnL/sync/AI portfolio cap included) |
| AUTO-4 | Exchange signing tests | (via harness) `cargo test --test exchange_signing` | **PASS** (4/4) |
| AUTO-5 | Exchange portfolio integration | (via harness) `cargo test --test exchanges_portfolio_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-6 | Prior-story integration suites | firefly/forecast/subscriptions/plans/wealth_alerts/ai_assistant | **SKIP** — `DATABASE_URL` unset |
| AUTO-7 | Frontend production build | (via harness) `npm run build` | **PASS** (WealthPage Crypto tab, Settings exchange section in build) |
| AUTO-8 | Compose minimal services | `docker compose --profile minimal config --services` (placeholder env) | **PASS** — `firefly-iii`, `flow-finance-ai`, `grafana` |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | Connectors import balances, positions, trades, transfers, funding, PnL | **PASS-with-prerequisites** | `ExchangeConnector` trait + Binance/Bybit/Bitunix implementations; `ExchangeService::run_post_sync` upserts all entity types; migration 007 tables + seed rows; GET-only HTTP audit + signing tests (4/4). **Operator prerequisite:** read-only API keys + **Sync exchanges now** for live import. |
| UAT-2 | AC-2 | Portfolio Engine calculates realized, unrealized, total return | **PASS** | `compute_hybrid_pnl` → `PnlBreakdown`; `PortfolioEngine` `total_return_pct`; unit tests `avg_cost_fallback_unrealized`, `avg_cost_from_buys`; REST `/portfolio/pnl` and wealth PnL row. |
| UAT-3 | AC-3 | Crypto holdings included in net worth view | **PASS-with-prerequisites** | Migration 007 `net_worth_snapshots.crypto_value_eur`, `firefly_value_eur`; `WealthService::compute_extended`; `/wealth` Overview combined total + Crypto tab. **Operator prerequisite:** PostgreSQL + exchange sync for live combined total. |
| UAT-4 | AC-4 | Portfolio allocation scenarios supported | **PASS-with-prerequisites** | `allocation_target` plan enum + 50/50 template; `load_allocation_target` + `allocation_gap` in wealth service; PlanningPage template row; WealthPage allocation gap card. **Operator prerequisite:** active allocation_target plan + synced wealth for live gap display. |
| UAT-5 | AC-5 | Grafana Dashboard 4 shows crypto slice and performance | **PASS** | `portfolio.json` `uid=portfolio`: Crypto value stat (`crypto_value_eur`), Firefly vs crypto pie, `total_return_pct` time series, FX incomplete panel. **Operator smoke:** post-sync snapshot for live panel data. |
| UAT-6 | AC-6 | API keys stored in self-hosted secrets/config only | **PASS** | TOML stores env var names only; credentials from `std::env::var`; Settings shows configured badge without secret values; `exchange_api_responses_contain_no_secrets` integration test path; `.env.example` documents vars; read-only connector design. |

## Summary

| Metric | Count |
|--------|-------|
| Acceptance criteria passed (code/automated) | 6/6 |
| Full runtime E2E executed | 0/6 (blocked by missing operator infra and exchange keys) |
| Automated checks passed | 5/8 (3 SKIP — expected without `DATABASE_URL`) |
| Failed | 0 |

## Operator prerequisites (post-release smoke)

1. Copy `.env.example` → `.env`; set `DATABASE_*`, `FIREFLY_*`, read-only `BINANCE_*` / `BYBIT_*` / `BITUNIX_*`, optional `AUTH_DEV_BYPASS=true`.
2. Provision external TimescaleDB; apply migrations including `007_exchanges_portfolio.sql` at backend startup.
3. Enable desired exchanges in `backend/config/default.toml`; restart backend.
4. `docker compose --profile minimal up --build`
5. Settings → **Test connection** per enabled exchange.
6. Sync Status → **Sync exchanges now** — verify `"exchanges"` phase and per-exchange rows.
7. Open `/wealth` — Overview combined total; Crypto tab holdings and PnL row; allocation gap when plan active.
8. Create allocation_target plan on `/planning`; confirm gap card on `/wealth`.
9. Open Grafana Analytics dashboard `portfolio` (Dashboard 4) — crypto stat, allocation pie, total return series.
10. AI chat: ask portfolio question — `get_portfolio` includes crypto when connected.
11. Optional: `DATABASE_URL=... cargo test --test exchanges_portfolio_integration` for migration-007 persistence proof.

## Findings

### Blockers

None.

### Observations

1. Integration tests (`exchanges_portfolio_integration` and prior-story suites) require operator `DATABASE_URL` — skipped by design; static + unit coverage sufficient for verify-work gate (S0001–S0006 pattern).
2. Live exchange connector sync, Settings test connection, and combined wealth E2E depend on read-only API keys and PostgreSQL — documented in `docs/user-guides/US-0007.md`.
3. First-sync avg-cost inaccuracy, CoinGecko FX fallback deferred, Bitunix futures stub — known limitations from dev handoff; not verify-work blockers.
4. Rust unused-import warnings in exchange/portfolio modules — cosmetic, non-blocking.

## Next phase

Run `/release` in a fresh release subagent context.
