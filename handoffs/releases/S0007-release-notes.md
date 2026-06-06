# Sprint Release Notes — S0007

**Sprint:** S0007  
**Date:** 2026-06-02  
**Stories:** US-0007  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — `bash tests/run-tests.sh` @ 2026-06-02T00:30:00Z (release run; QA/UAT evidence aligned)
2. **QA completion gate:** PASS — `sprints/S0007/qa-findings.md`, no blockers
3. **UAT completeness gate:** PASS — `sprints/S0007/uat.json`, `sprints/S0007/uat.md` (6/6 AC)
4. **Isolation compliance gate:** PASS — execute/qa/verify-work/release checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose --profile minimal up --build`
- `runtime_mode`: local
- `runtime_context_ref`: `docs/engineering/runbook.md` (Project run steps); external TimescaleDB required; migration `007_exchanges_portfolio.sql` at backend startup

## Connect

- `service_url`: `http://localhost:8080` (Flow Finance AI UI/API)
- `service_port`: `8080` (`FLOW_PORT` env override)
- `health_endpoint`: `http://localhost:8080/health` (liveness); `http://localhost:8080/health/ready` (readiness + DB ping)

Additional services (minimal profile):

| Service | URL | Port env |
|---------|-----|----------|
| Firefly III | `http://localhost:8081` | `FIREFLY_PORT` (default 8081) |
| Grafana | `http://localhost:3000` | `GRAFANA_PORT` (default 3000) |

Crypto surfaces: `/wealth` Overview + **Crypto** tab; Settings **Crypto exchanges**; Sync Status per-exchange rows; Grafana dashboard `uid=portfolio`

## Verify

- `verification_steps`:
  1. Copy `.env.example` to `.env`; set `DATABASE_*`, `FIREFLY_*`, and read-only `BINANCE_*` / `BYBIT_*` / `BITUNIX_*` env vars.
  2. Provision external PostgreSQL **with TimescaleDB extension**; confirm migration `backend/migrations/007_exchanges_portfolio.sql` applies at backend startup.
  3. Enable exchanges in `backend/config/default.toml` `[exchanges]`; restart backend after env changes.
  4. `docker compose --profile minimal up --build`.
  5. `curl -sf http://localhost:8080/health` and `curl -sf http://localhost:8080/health/ready`.
  6. Settings → **Crypto exchanges** — **Test connection** per enabled exchange (read-only keys).
  7. Sync Status → **Sync exchanges now** — confirm `"exchanges"` phase and per-exchange entity counts.
  8. Open `/wealth` — Overview combined net worth; **Crypto** tab holdings, PnL row (realized / unrealized / total return).
  9. Create `allocation_target` plan on `/planning` (e.g. 50/50 template); confirm allocation gap card on `/wealth`.
  10. Grafana dashboard **Portfolio** (`uid=portfolio`) — crypto stat, allocation pie, total return series.
  11. Run `bash tests/run-tests.sh`; optional `DATABASE_URL=... cargo test --test exchanges_portfolio_integration` for migration-007 persistence proof.
  12. AI chat `get_portfolio` includes crypto totals and top holdings when exchanges connected.
- `expected_health_signal`: HTTP 200 from `/health`; `GET /api/v1/exchanges` lists configured exchanges; post-sync `net_worth_snapshots` includes `crypto_value_eur`; Settings never returns secret values

## Credentials

- `credential_source_refs` (env names only):
  - `DATABASE_PASSWORD`, `DATABASE_USER`, `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`
  - `FIREFLY_PERSONAL_ACCESS_TOKEN`, `FIREFLY_APP_KEY`, `FIREFLY_DB_PASSWORD`
  - `BINANCE_API_KEY`, `BINANCE_API_SECRET`
  - `BYBIT_API_KEY`, `BYBIT_API_SECRET`
  - `BITUNIX_API_KEY`, `BITUNIX_API_SECRET`
  - `OPENAI_API_KEY` (US-0006 AI; optional for portfolio-only verify)
  - `OIDC_ISSUER_URL`, `OIDC_AUDIENCE`, `VITE_OIDC_AUTHORITY`, `VITE_OIDC_CLIENT_ID`
  - `GRAFANA_ADMIN_PASSWORD`
  - `AUTH_DEV_BYPASS` (dev-only; never production)
- `expected_value_source`: operator `.env` at repo root (from `.env.example`); exchange secrets via env only — TOML stores `api_key_env` / `api_secret_env` **names**; see `docs/user-guides/US-0007.md`

## Known Issues

- External TimescaleDB is an operator prerequisite; `exchanges_portfolio_integration` skips without `DATABASE_URL`.
- Live exchange sync and Settings **Test connection** require operator read-only API keys.
- First-sync avg-cost PnL may be inaccurate until trade backfill completes (R-0033).
- FX incomplete banner for illiquid alts when Frankfurter + exchange tickers lack EUR pair (R-0034).
- Bitunix futures stub when `enabled_futures=false`; CoinGecko FX fallback deferred post-MVP.
- Binance per-symbol trade fan-out can extend sync mutex duration (R-0032, R-0036).
- Integration and runtime E2E deferred in CI/agent environment — operator steps above required for live proof.

## Deliverables (US-0007)

- Migration `007_exchanges_portfolio.sql` — exchange tables, portfolio PnL/baselines, `fx_rates`, crypto net-worth columns (DEC-0037–DEC-0042)
- `ExchangeConnector` trait — Binance (spot + USD-M), Bybit (UNIFIED), Bitunix (spot-first); GET-only HTTP layer
- `PortfolioEngine` — hybrid PnL (exchange-reported + avg-cost fallback)
- `FxService` — Frankfurter cache + exchange ticker EUR conversion
- Sync — `"exchanges"` phase before `"alerts"`; `POST /api/v1/sync/exchanges/trigger`
- REST — `/exchanges`, `/wealth/crypto`, `/portfolio/pnl`, extended `get_portfolio` AI tool
- React — `/wealth` Crypto tab, Settings exchange section, Sync Status exchange rows, allocation gap card
- Grafana — Dashboard 4 completion (`uid=portfolio`)
- Operator guide: `docs/user-guides/US-0007.md`
- Tests — 54 lib + 4 exchange_signing; `exchanges_portfolio_integration` (optional with `DATABASE_URL`)

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0007 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.7.0-us0007`
