# Sprint Release Notes — S0014

**Sprint:** S0014  
**Date:** 2026-06-08  
**Stories:** US-0013  
**Queue status:** released  
**Orchestrator:** `auto-20260608-us0013-001`  
**Decision:** DEC-0076

---

## Gate results

1. **Check-in test gate:** PASS — `bash scripts/compose-config-check.sh` exit 0; `cargo test --test forecast_ml_integration` 3/3 @ 2026-06-08 release
2. **QA completion gate:** PASS — `sprints/S0014/qa-findings.md` (AC-1–AC-9 verified; 0 blocking findings)
3. **UAT completeness gate:** PASS-with-prerequisites — `sprints/S0014/uat.md`, `sprints/S0014/uat.json`; code/test PASS; runtime omniflow ML smoke deferred pending **BACKEND_COMPOSE_DEPLOY** (S0010/S0012 precedent)
4. **Isolation compliance gate:** PASS — intake through qa checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — qa tuple `runtime-proof-qa-20260608-s0014-us0013-001`; release tuple at finalization
6. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai stats-forecast`
- `runtime_mode`: remote (omniflow external US-0010); ML opt-in via `FORECAST_ML_ENABLED=true`
- `runtime_context_ref`: `docs/engineering/runbook.md` § 7a Omniflow ML enablement; `docs/user-guides/US-0013.md`

**Profile rule:** use **`external` only** — do not combine with `minimal`, `standard`, `full`, or `bundled-firefly`.

**Operator gate — BACKEND_COMPOSE_DEPLOY (required before live ML smoke):**

```bash
# Set FORECAST_ML_ENABLED=true in operator .env first
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai stats-forecast
curl -sf "http://localhost:${STATS_FORECAST_PORT:-8091}/health"
```

Then trigger **Full Firefly sync** from Settings → Sync.

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (Traefik + basic-auth)
- `health_endpoint`: `GET /health`
- Sidecar health: `http://localhost:8091/health` (host remap; internal `http://stats-forecast:8090/health`)
- Forecast UI: `/forecast` (Compare tab); Wealth UI: `/wealth`; Grafana: `/analytics/forecast-horizons`

## Verify

- `verification_steps`:
  1. Deploy overlay per § 7a; set `FORECAST_ML_ENABLED=true`, `STATS_FORECAST_URL=http://stats-forecast:8090`, `STATS_FORECAST_PORT=8091` in operator `.env`.
  2. Confirm sidecar health: `curl -sf http://localhost:8091/health`.
  3. Recreate `flow-finance-ai`; trigger Full sync; confirm sync UI shows "ML forecast…" phase.
  4. `GET /api/v1/forecast/meta` → `ml_status: success` or documented skip reason (`sidecar_disabled`, `sidecar_unavailable`, `insufficient_history`).
  5. React `/forecast` Compare: baseline + ML overlay when `ml_enhanced` data exists; disabled copy reads "ML forecast is not enabled on this deployment".
  6. React `/wealth`: portfolio horizon cards (3/6/12 mo); FX incomplete amber banner when applicable.
  7. Grafana `$forecast_variant=ml_enhanced` panels return data after enablement + recompute.
  8. Automated: `bash scripts/compose-config-check.sh`; `cargo test --test forecast_ml_integration` (3/3).
- `expected_health_signal`: sidecar `/health` OK; backend `/health` OK; `ml_enhanced` computations persisted when history ≥ 12 monthly points; skip metadata on failure without sync abort (DEC-0052)

## Credentials

- `credential_source_refs` (env names only):
  - US-0010 block: `COMPOSE_FILE`, `COMPOSE_PROFILES`, `DATABASE_*`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, `TRAEFIK_HOST`
  - US-0013 ML: `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL`, `STATS_FORECAST_PORT`
- `expected_value_source`: operator `.env` at repo root (from `.env.example` ML block)

## Known Issues

- Omniflow ML smoke **PENDING** operator post-deploy — UAT-1 … UAT-9 pass-with-prerequisites at release (`BACKEND_COMPOSE_DEPLOY` + Full sync + ≥12 monthly history).
- Cold start: first sync after deploy may record `sidecar_unavailable` — re-sync after `/health` returns OK.
- StatsForecast sidecar RSS ~200–400 MB under load (R-0044); monitor on shared omniflow host.
- Integration tests require operator `DATABASE_URL` (carry-forward).

## Deliverables (US-0013)

| Slice | Tasks | Outcome |
|-------|-------|---------|
| S1 | T-0144 … T-0147 | External overlay adds `stats-forecast` on `external` profile; ML env passthrough; `.env.example` ML block; compose-config-check 3-service assert |
| S2 | T-0148, T-0149 | Verify-first — sync ML phase + health gate, skip metadata, `ml_enhanced` API/persistence confirmed |
| S3 | T-0150 … T-0152 | Verify-first — ForecastPage Compare + `sidecar_disabled` copy, WealthPage portfolio forecast + FX banner, Grafana ML panels |
| S4 | T-0153, T-0154 | Runbook § 7a Omniflow ML enablement; dual CI guard (compose-config-check + `forecast_ml_integration`) |

**Files changed:** `docker-compose.external.yml`, `.env.example`, `scripts/compose-config-check.sh`, `docs/engineering/runbook.md`

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0014 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.14.0-us0013`

## Milestone

**US-0013 released** — production ML forecast & wealth analytics hardening on omniflow external profile (DEC-0076).
