# Sprint Release Notes — S0005

**Sprint:** S0005  
**Date:** 2026-05-31  
**Stories:** US-0005  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — `bash tests/run-tests.sh` @ 2026-05-31T21:38:05Z (release run + QA/UAT evidence)
2. **QA completion gate:** PASS — `sprints/S0005/qa-findings.md`, no blockers
3. **UAT completeness gate:** PASS — `sprints/S0005/uat.json`, `sprints/S0005/uat.md` (6/6 AC)
4. **Isolation compliance gate:** PASS — execute/qa/verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose --profile minimal up --build`
- `runtime_mode`: local
- `runtime_context_ref`: `docs/engineering/runbook.md` (Project run steps); external TimescaleDB required for alerts/wealth persistence

## Connect

- `service_url`: `http://localhost:8080` (Flow Finance AI UI/API)
- `service_port`: `8080` (`FLOW_PORT` env override)
- `health_endpoint`: `http://localhost:8080/health` (liveness); `http://localhost:8080/health/ready` (readiness + DB ping)

Additional services (minimal profile):

| Service | URL | Port env |
|---------|-----|----------|
| Firefly III | `http://localhost:8081` | `FIREFLY_PORT` (default 8081) |
| Grafana | `http://localhost:3000` | `GRAFANA_PORT` (default 3000) |

Wealth UI: `http://localhost:8080/wealth` — Alerts inbox: `http://localhost:8080/alerts` (after auth or `AUTH_DEV_BYPASS=true`)

## Verify

- `verification_steps`:
  1. Copy `.env.example` to `.env`; set `DATABASE_*`, `FIREFLY_*`, and required compose placeholders.
  2. Provision external PostgreSQL **with TimescaleDB extension**; apply migrations including `backend/migrations/005_alerts_wealth.sql`.
  3. `docker compose --profile minimal up --build`.
  4. `curl -sf http://localhost:8080/health` and `curl -sf http://localhost:8080/health/ready`.
  5. Complete Firefly sync with asset accounts (`include_net_worth=true`); run forecast recompute (US-0002).
  6. Set active plan with category-targeted adjustments (US-0004) for budget drift evaluation.
  7. Run **Sync now** — confirm post-sync `"alerts"` phase; open `/wealth` (net worth, breakdown, mixed-currency banner, wealth chart).
  8. Open `/alerts` and header bell — acknowledge/dismiss active alerts; verify unread badge clears on acknowledge.
  9. Open Grafana at `http://localhost:3000`; confirm dashboard **Portfolio** (`uid=portfolio`) and Dashboard 1 `$scarcity_threshold` from `alert_config`.
  10. Run `bash tests/run-tests.sh`; optional `DATABASE_URL=... cargo test --test wealth_alerts_integration` for snapshot upsert + post-sync scarcity proof.
- `expected_health_signal`: HTTP 200 from `/health`; `GET /api/v1/wealth` returns breakdown after auth; `GET /api/v1/alerts/unread-count` reflects active unacknowledged alerts; Grafana `portfolio` panels query `FlowFinancePostgreSQL` without provisioning errors; scarcity reference line tracks `alert_config.scarcity_threshold_eur`.

## Credentials

- `credential_source_refs` (env names only):
  - `DATABASE_PASSWORD`, `DATABASE_USER`, `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`
  - `FIREFLY_PERSONAL_ACCESS_TOKEN`, `FIREFLY_APP_KEY`, `FIREFLY_DB_PASSWORD`
  - `OIDC_ISSUER_URL`, `OIDC_AUDIENCE`, `VITE_OIDC_AUTHORITY`, `VITE_OIDC_CLIENT_ID`
  - `GRAFANA_ADMIN_PASSWORD`
  - `AUTH_DEV_BYPASS` (dev-only; never production)
- `expected_value_source`: operator `.env` at repo root (from `.env.example`); Firefly PAT from Firefly UI; threshold defaults in `backend/config/default.toml` `[alerts]` mirrored to `alert_config` on startup — see `docs/user-guides/US-0005.md`.

## Known Issues

- External TimescaleDB (not embedded PostgreSQL) is an operator prerequisite for alert persistence and live wealth data.
- `wealth_alerts_integration`, `plans_integration`, `forecast_integration`, `subscriptions_integration`, and `firefly_integration` tests skipped without `DATABASE_URL`.
- Live scarcity, budget drift, and plan viability alerts require successful forecast, active plan, and synced transactions (post-sync `"alerts"` phase).
- Mixed-currency headline total sums native balances without FX — mandatory warning banner in React and Grafana (by design until US-0007).
- Crypto balances excluded from net worth total; US-0007 placeholder only.
- Subscription-scoped alerts remain on `/subscriptions` (US-0003); unified inbox does not duplicate them (cross-link in bell popover).
- Budget drift MTD proration may skew one-time plan deltas.
- OIDC live session requires IdP or `AUTH_DEV_BYPASS=true` for API/UI dev access.
- ECharts main bundle ~1 MB (`WealthChart` code-split; acceptable for MVP).

## Deliverables (US-0005)

- Migration `005_alerts_wealth.sql` — `alert_config`, `alerts`, `net_worth_snapshots` (DEC-0025–DEC-0027)
- WealthService — net worth breakdown, daily snapshots, history (DEC-0025)
- Alert Engine — scarcity, budget_drift, plan_viability evaluators (DEC-0026)
- AlertService — fingerprint dedup, acknowledge/dismiss lifecycle, `run_post_sync` (DEC-0027, DEC-0028)
- Sync — inline `"alerts"` phase after forecast+plan hook
- REST API — 6 JWT routes under `/api/v1/wealth/*` and `/api/v1/alerts/*`
- React `/wealth`, `/alerts`, header `AlertBell` with unread badge (DEC-0030)
- Grafana — dashboard `portfolio` (Dashboard 4 partial); Dashboard 1 `$scarcity_threshold` (DEC-0029)
- Operator guide: `docs/user-guides/US-0005.md`
- Tests — 36 unit tests; `wealth_alerts_integration` (optional with `DATABASE_URL`)

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0005 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.5.0-us0005`
