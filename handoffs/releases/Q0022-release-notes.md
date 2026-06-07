# Quick Release Notes — Q0022 / BUG-0014

**Quick task:** Q0022  
**Bug:** BUG-0014 — Post-rebuild omniflow cluster (ML sidecar, crypto display, Grafana zeros, planning delete)  
**Date:** 2026-06-07  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0014 rows AO–AT; AP/AR runtime operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (177/177) @ 2026-06-07 release
2. **QA completion gate:** PASS — `sprints/quick/Q0022/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0022/uat.json`, `sprints/quick/Q0022/uat.md`, `handoffs/verify_work_to_release.md`; 14 steps — 4 pass, 8 pass_with_prerequisites, 2 skipped, 0 fail
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260607-bug0014-q0022-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Post-rebuild omniflow cluster fixes per **DEC-0081** (AQ holdings cap + unified `fx_incomplete`), **DEC-0082** (AS1 plan delete + active 409 guard), **DEC-0083** (AS2 target_type select + help), plus Grafana ML copy (**AO1**) on US-0010 external profile. Conditional **AP2** and **AR1** skipped pending operator gates.

| Scope | Fix |
|-------|-----|
| **AO** | Panel 13 dual-scenario ML copy; US-0013 + stats-forecast named when sidecar down |
| **AQ** | `holdings_all` cap 50; unified `unpriced_assets` / `fx_incomplete` API; WealthPage native qty + EUR table + FX banner |
| **AS** | DELETE plan UI + confirm modal; active plan delete disabled; 409 on active; five `target_type` options + help |
| **AP** | AP2 defensive subtotal — **skipped** (AP1_SQL_PROBE deferred post-deploy) |
| **AR** | AR1 cashflow.json — **skipped** (AR-API/AR-GRAF partial probe deferred post Full sync) |
| **AT** | Ops-only — three-service compose (`stats-forecast`) when `FORECAST_ML_ENABLED=true` |

**Code proof:** `cargo test --lib` 177/177; wealth 4/4; plan_delete 1/1; grafana_provisioning 6/6; frontend vitest 6/6.

**Operator post-release:** Deploy Q0020+Q0022 bundle; run 14-step smoke per `sprints/quick/Q0022/uat.json` `operator_smoke_checklist`.

---

## Run

**Target services (external profile):** `flow-finance-ai` (AQ/AS backend + frontend), `grafana` (AO1 dashboard JSON), `stats-forecast` (AT ops gate when ML enabled).

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana stats-forecast
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before wealth/planning probes):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — THREE_SERVICE_COMPOSE (required before AO-1 / AT-1):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d stats-forecast
```

**Operator gate — GRAFANA_PROVISIONING_RELOAD (required before AO live panel):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate grafana
```

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) + forecast recompute acct 114 before AR probes.

**Operator gate — AP1_SQL_PROBE:** Run architecture SQL on `exchange_holdings` before AP2 evaluation:

```sql
SELECT product_type, asset, quantity, market_value_eur, unrealized_pnl_eur
FROM exchange_holdings WHERE exchange_id='bitunix'
ORDER BY product_type, asset;
```

- `start_command`: docker compose commands above + Full sync from Settings UI
- `runtime_mode`: remote (omniflow external US-0010)
- `runtime_context_ref`: `docs/engineering/runbook.md` (§24 BUG-0014 hotfix)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

**Grafana warning:** Do **not** click **Save** on analytics dashboards after variable changes — persisted `current` blocks override provisioning JSON (see runbook §17).

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- `service_port`: 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health`
- Analytics routes: `/analytics/cashflow`, `/analytics/forecast-horizons`, `/analytics/portfolio`, `/analytics/budgets`, `/analytics/subscriptions`, `/analytics/alerts`
- Wealth API: `GET /api/v1/wealth`
- Planning: `/planning`
- Forecast meta: `GET /api/v1/forecast/meta`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `FORECAST_ML_ENABLED` | `true` when ML overlay expected |
| `STATS_FORECAST_URL` | `http://stats-forecast:8090` on external profile |
| `BITUNIX_*` | Read-only exchange keys for AP/AQ probes |
| `DATABASE_HOST` | **`postgres`** on external profile |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(AO)** | `/analytics/forecast-horizons` panel 13 | Dual-scenario ML copy; not permanent US-0013-not-enabled when env opts in |
| **(AO-1)** | `GET /api/v1/forecast/meta` | `ml_computation_id` set or accurate sidecar-down copy |
| **(AP)** | AP1 SQL + `GET /api/v1/wealth` | `crypto.subtotal_eur` > 0 when futures priced; reopen AP2 only if priced + subtotal 0 |
| **(AQ)** | Wealth crypto tab | Native qty + EUR table; unified FX banner when `fx_incomplete` |
| **(AR)** | `GET /api/v1/forecast/daily?account_id=114` + `/analytics/cashflow` | Non-zero signed balances; reopen AR1 only if API≠Grafana |
| **(AS)** | `/planning` | Delete non-active plan; 409 on active; five target_type options |
| **(AT)** | `docker ps \| grep stats-forecast` | Container running when ML enabled |
| Regression | Six `/analytics/{slug}` routes | Embed without transport errors |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test grafana_provisioning_bug0009
cd backend && cargo test --lib plan_delete_api_tests
cd frontend && npm test -- --run
```

**Live (operator post-deploy):** 14-step checklist in `sprints/quick/Q0022/uat.json` after all operator gates complete.

**Expected health signal:** `GET /health` → HTTP 200; wealth crypto subtotal populated when exchange sync succeeded; planning delete returns 409 for active plan.

---

## Credentials

- Traefik basic auth — operator shell / password manager
- `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` — operator `.env` only; never inline in artifacts
- No inline secrets in release artifacts

---

## Changes

| Area | Summary |
|------|---------|
| `grafana/provisioning/dashboards/analytics/forecast-horizons.json` | AO1 — panel 13 dual-scenario ML copy |
| `backend/src/wealth/types.rs`, `backend/src/wealth/service.rs` | AQ1 — holdings cap 50 + unified fx_incomplete |
| `backend/src/portfolio/service.rs`, `backend/src/portfolio/repository.rs` | AQ1 — PnL payload wire |
| `frontend/src/lib/api.ts`, `frontend/src/pages/WealthPage.tsx` | AQ2 — native qty + EUR table + FX banner |
| `backend/src/plan/service.rs`, `backend/src/api/plans.rs` | AS1 — active delete 409 guard |
| `frontend/src/pages/PlanningPage.tsx`, `planningFeedback.tsx` | AS1/AS2 — delete UI + target_type select |
| `backend/tests/grafana_provisioning_bug0009.rs` | AO1 test update |
| `frontend/src/pages/planningFeedback.test.ts` | AS1 error parse test |
| Runbook | §24 BUG-0014 operator smoke |

**Linked decisions:** DEC-0081, DEC-0082, DEC-0083 (extends DEC-0080, DEC-0076, DEC-0066)  
**Research fulfilled:** R-0079  
**Deferred:** AP2 (AP1_SQL_PROBE gate); AR1 (AR-API/AR-GRAF partial probe)

---

## Known Issues

- V1 omniflow runtime probes pass-with-prerequisites until **BACKEND_FRONTEND_DEPLOY**, **THREE_SERVICE_COMPOSE**, **GRAFANA_PROVISIONING_RELOAD**, **FULL_FIREFLY_SYNC**, **AP1_SQL_PROBE**
- AP2 and AR1 remain conditional — reopen only when operator gates show priced futures + subtotal 0 (AP) or API≠Grafana on acct 114 (AR)
- Omniflow API health returned 404 at verify-work — deploy pending per BUG-0013 precedent

---

## Regression scope

- BUG-0013 DEC-0080 Bitunix pricing unchanged
- US-0014 planning mode semantics preserved
- DEC-0064 crypto subtotal rules preserved
- US-0013 ML overlay contract unchanged

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0022 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0014-q0022`

## Milestone

**BUG-0014 released** — omniflow post-rebuild cluster fixes for ML copy, wealth crypto display, plan delete UX; operator smoke deferred per pass-with-prerequisites.
