# Summary — Sprint S0014 / US-0013

**Story:** US-0013 — Production ML hardening  
**Sprint:** S0014  
**Status:** RELEASED (`0.14.0-us0013`, 2026-06-08)  
**Orchestrator:** `auto-20260608-us0013-001`  
**Date:** 2026-06-08  
**Decision:** DEC-0076

## Delivered

| Slice | Tasks | Outcome |
|-------|-------|---------|
| S1 | T-0144 … T-0147 | External overlay adds `stats-forecast` on `external` profile (traefik network, port 8091); ML env passthrough on `flow-finance-ai`; `.env.example` ML block; compose-config-check 3-service assert |
| S2 | T-0148, T-0149 | Verify-first audit — sync ML phase + health gate, skip metadata, `ml_enhanced` API/persistence confirmed; no code gaps |
| S3 | T-0150 … T-0152 | Verify-first audit — ForecastPage Compare + `sidecar_disabled` copy, WealthPage portfolio forecast + FX banner, Grafana ML panels + BUG-0009 banner; no code gaps |
| S4 | T-0153, T-0154 | Runbook § Omniflow ML enablement; dual CI guard (compose-config-check + `forecast_ml_integration`) confirmed in `tests/run-tests.sh` |

## Files changed

- `docker-compose.external.yml` — stats-forecast overlay + ML env passthrough
- `.env.example` — omniflow ML enablement block
- `scripts/compose-config-check.sh` — 3-service external set + traefik/ML asserts
- `docs/engineering/runbook.md` — § 7a Omniflow ML enablement

## Verification (no gaps found)

| Area | Path | Contract verified |
|------|------|-------------------|
| Config merge | `backend/src/config/mod.rs` | `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL` env override |
| Sync phase | `backend/src/sync/mod.rs` | `forecast_ml` after baseline; phase label "ML forecast…" |
| Health gate | `backend/src/forecast_ml/service.rs` | `health_ok()` before recompute |
| Skip metadata | `backend/src/forecast_ml/service.rs` | `record_skip_on_baseline` → `sidecar_disabled` / `sidecar_unavailable` / `insufficient_history` |
| Min history | `backend/config/default.toml` | `min_monthly_points = 12` unchanged |
| API | `backend/src/api/forecast.rs` | `variant=ml_enhanced` 6–24 month series |
| Forecast UX | `frontend/src/pages/ForecastPage.tsx` | Compare control; `sidecar_disabled` copy (DEC-0066) |
| Wealth UX | `frontend/src/pages/WealthPage.tsx` | Portfolio horizons; signed totals; FX incomplete banner |
| Grafana | `grafana/.../forecast-horizons.json` | `$forecast_variant` default `baseline`; ML panels; status banner |

## Tests

| Check | Result |
|-------|--------|
| `scripts/compose-config-check.sh` | PASS |
| `cargo test --test forecast_ml_integration` | PASS (3/3) |

## Operator gate

**BACKEND_COMPOSE_DEPLOY** required before UAT omniflow ML smoke — deploy S1 overlay, set `FORECAST_ML_ENABLED=true`, restart services, Full sync after `/health` OK.

## Release

Released 2026-06-08 — acceptance AC-1–AC-9 checked; omniflow ML smoke pass-with-prerequisites pending **BACKEND_COMPOSE_DEPLOY**.
