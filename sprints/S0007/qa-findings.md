# QA Findings — Sprint S0007 / US-0007

**Sprint:** S0007  
**Story:** US-0007  
**QA phase:** `/qa`  
**Date:** 2026-06-01  
**Verdict:** **PASS** (ready for `/verify-work`)

## Scope

Crypto exchange portfolio vertical slice: migration 007, FxService, Binance/Bybit/Bitunix read-only connectors, Portfolio Engine hybrid PnL, sync `"exchanges"` phase, extended wealth/portfolio REST API, React `/wealth` Crypto tab, Settings/Sync Status exchange UI, Grafana Dashboard 4 (`uid=portfolio`), tests, operator guide.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0007/summary.md`, `sprints/S0007/tasks.md`, `sprints/S0007/plan-verify.json`, `docs/product/acceptance.md` (US-0007), `backend/migrations/007_exchanges_portfolio.sql`, `backend/src/exchanges/`, `backend/src/portfolio/`, `backend/src/fx/`, `backend/src/wealth/`, `backend/src/sync/mod.rs`, `backend/tests/{exchange_signing,exchanges_portfolio_integration}.rs`, `frontend/src/pages/{WealthPage,SettingsPage,SyncStatusPage,PlanningPage}.tsx`, `grafana/provisioning/dashboards/analytics/portfolio.json`, `docs/user-guides/US-0007.md`, `.env.example`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Automated harness | `bash tests/run-tests.sh` | **PASS** (exit 0, `All US-0007 tests passed`) |
| T-2 | Firefly GET-only unit test | `cargo test --test firefly_readonly` | **PASS** (1/1) |
| T-3 | Backend unit tests | `cargo test --lib` | **PASS** (54/54; exchange/PnL/sync/AI portfolio cap tests included) |
| T-4 | Exchange signing tests | `cargo test --test exchange_signing` | **PASS** (4/4) |
| T-5 | Exchange portfolio integration | `cargo test --test exchanges_portfolio_integration` | **SKIP** — `DATABASE_URL` not set (harness skips; test early-returns when unset) |
| T-6 | Other integration suites | firefly/forecast/subscriptions/plans/wealth_alerts/ai_assistant | **SKIP** — `DATABASE_URL` not set |
| T-7 | Frontend build | `npm run build` (via harness) | **PASS** |
| T-8 | GET-only exchange HTTP audit | `exchanges::http::tests::get_only_audit_rejects_post`, `exchange_signing` write-method rejection | **PASS** |
| T-9 | Sync phase ordering | `sync::tests::exchanges_phase_after_forecast_in_full_run` | **PASS** |
| T-10 | User guide | Static review `docs/user-guides/US-0007.md` | **PASS** — prerequisites, env secrets, sync phases, wealth Crypto tab, allocation workflow, Grafana panels |
| T-11 | Runtime E2E (live keys, Sync exchanges, Grafana, allocation gap UI) | Not executed in QA environment | **Deferred** to `/verify-work` |

### Environment dependencies (non-blocking)

- **`DATABASE_URL`:** Required for `exchanges_portfolio_integration` migration-007 persistence path and full harness integration block. Static/unit coverage passes without DB (same pattern as S0001–S0006).
- **Exchange API keys (`BINANCE_*`, `BYBIT_*`, `BITUNIX_*`):** Required for live connector sync and Settings **Test connection** — deferred to verify-work.
- **US-0001–US-0006 operational stack:** Firefly sync, planning, OIDC — deferred to verify-work for combined wealth and AI `get_portfolio` E2E.

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | Connectors for Binance, Bybit, and Bitunix import balances, positions, trades, transfers, funding, and PnL | **PASS** | `ExchangeConnector` trait defines `sync_balances`, `sync_positions`, `sync_trades`, `sync_transfers`, `sync_funding`; `binance.rs`, `bybit.rs`, `bitunix.rs` implement trait; `ExchangeService::run_post_sync` upserts all entity types then `portfolio.recompute_pnl`; migration 007 tables + seed rows; Settings/Sync Status UI for three exchanges; GET-only HTTP audit tests. Live exchange API import deferred to verify-work. |
| AC-2 | Portfolio Engine calculates realized gains, unrealized gains, and total return | **PASS** | `compute_hybrid_pnl` → `PnlBreakdown { realized_eur, unrealized_eur, crypto_value_eur }`; `PortfolioEngine` computes `total_return_pct`; unit tests `portfolio::pnl::tests::avg_cost_fallback_unrealized`, `portfolio::avg_cost::tests::avg_cost_from_buys`; REST `/portfolio/pnl` and wealth PnL row; integration test `recompute_pnl` path when DB available. |
| AC-3 | Crypto holdings included in net worth view (extends US-0005) | **PASS** | Migration 007 `net_worth_snapshots.crypto_value_eur`, `firefly_value_eur`; `WealthService::compute_extended` with exchange repo; `/wealth` Overview combined total + Crypto tab; integration asserts `crypto_value_eur` column and extended breakdown. |
| AC-4 | Portfolio allocation scenarios supported (e.g. 50% ETF / 50% crypto) | **PASS** | Migration `allocation_target` plan enum; `plan/templates.rs` preset 50/50; `load_allocation_target` + `allocation_gap` in `wealth/service.rs`; PlanningPage template row; WealthPage allocation gap card. Runtime gap UI deferred to verify-work. |
| AC-5 | Grafana Dashboard 4 shows crypto slice and portfolio performance | **PASS** | `grafana/.../portfolio.json` `uid=portfolio`: Crypto value stat (`crypto_value_eur`), Firefly vs crypto pie, `total_return_pct` time series, FX incomplete panel from `payload.fx_incomplete`. Live Grafana provisioning deferred to verify-work. |
| AC-6 | API keys stored in self-hosted secrets/config, not transmitted externally | **PASS** | TOML stores `api_key_env` / `api_secret_env` names only (`config/mod.rs`); credentials from `std::env::var`; Settings shows env names + configured badge, no secret values; `exchange_api_responses_contain_no_secrets` integration test; `.env.example` documents vars; read-only connector design (no browser secret entry). |

**Summary:** 6/6 PASS (static/unit/harness path; live exchange sync, DB integration persistence, Grafana live, and full UI E2E deferred to verify-work with operator env).

## Generated baseline test evidence

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` + `typescript` |
| `generated_test_command` | `bash tests/run-tests.sh` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-06-01 — exit 0, message `All US-0007 tests passed` |
| `generated_test_paths_ref` | `backend/src/exchanges/*`, `backend/src/portfolio/*`, `backend/tests/exchange_signing.rs`, `backend/tests/exchanges_portfolio_integration.rs`, `frontend/` build |
| `generated_test_reason_code` | — |

## Runtime QA evidence

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed |
| `runtime_stack_profile` | `rust` + `node` + `grafana` |
| `runtime_mode` | `local` |
| `runtime_health_target` | Deferred — `/wealth`, `/api/v1/exchanges`, sync `"exchanges"` phase, Grafana `uid=portfolio` |
| `runtime_health_result` | `deferred` |
| `runtime_final_verdict` | `deferred` (verify-work) |
| `runtime_reason_code` | `RUNTIME_E2E_DEFERRED_VERIFY_WORK` |
| `runtime_evidence_refs` | `docs/user-guides/US-0007.md`, `handoffs/dev_to_qa.md` |

## Findings

### Blockers

None.

### Observations (non-blocking)

1. **Integration tests skipped without `DATABASE_URL`:** `exchanges_portfolio_integration` and prior-story integration suites not run in harness; static + unit coverage sufficient for QA gate (S0001–S0006 pattern).
2. **Live exchange / Grafana E2E not exercised:** Requires operator PostgreSQL, read-only API keys, and optional Firefly sync — deferred to verify-work.
3. **Known limitations (dev handoff):** First-sync avg-cost inaccuracy, CoinGecko FX fallback deferred, Bitunix futures stub when `enabled_futures=false` — documented, not QA blockers.
4. **Rust warnings:** Unused imports in exchange/portfolio modules — cosmetic, non-blocking.

## Decision gates

No critical issues requiring dev rework. Proceed to **`/verify-work`**.

## Next phase

Run `/verify-work` in a fresh subagent with operator-provisioned PostgreSQL (`DATABASE_URL`), optional read-only exchange keys, synced Firefly data, and stack up per `docs/user-guides/US-0007.md`.
