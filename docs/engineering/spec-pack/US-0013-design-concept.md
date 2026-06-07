# Design Concept — US-0013

## Summary

US-0013 closes the **production ML enablement gap** on the omniflow external profile: wire the existing US-0009 StatsForecast sidecar into `docker-compose.external.yml`, document operator opt-in env vars, verify sync/API/UI/Grafana end-to-end, and extend runbook + CI guards. **No new forecasting models** — compose overlay + verification + ops repeatability.

## Goals

- Start `stats-forecast` on `--profile external` via overlay profile-merge (DEC-0076)
- Operator opt-in: `FORECAST_ML_ENABLED=true` + documented `STATS_FORECAST_URL` (DEC-0049 preserved elsewhere)
- Post-sync `forecast_ml` phase produces `model_kind=ml_enhanced` rows when history ≥12 months
- React Compare + wealth portfolio overlay + Grafana ML panels show data after enablement
- Runbook + CI prove sidecar path without production secrets

## Non-goals

- New ML model research (US-0009 ladder sufficient)
- Monthly bucket attribution (BUG-0012 / US-0015)
- Grafana empty-state-only work (BUG-0009 DONE — banner when ML off)
- Embedded Rust ML (augurs) — rejected at US-0009 (R-0044)

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0076 | Overlay additive `external` profile on `stats-forecast` | One container; traefik network; port 8091 host remap |
| DEC-0049 | Default-off until explicit enable | No silent ML on minimal/external |
| DEC-0052 | ML failure never fails sync | Omniflow availability |
| DEC-0066 | `sidecar_disabled` degraded copy | Honest UI when ML off |
| DEC-0007 | Baseline authoritative | Alerts/plan hooks unchanged |

## User experience

Operators follow runbook steps: merge compose files → set env → Full sync → verify `/forecast` Compare and Grafana forecast-horizons `$forecast_variant=ml_enhanced`. End users see ML bands and Compare overlay when enabled; degraded messaging when disabled per DEC-0066.
