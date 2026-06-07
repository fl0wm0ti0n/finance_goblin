# QA findings — S0014 / US-0013

**Story:** US-0013 — Production ML forecast & wealth analytics hardening  
**Sprint:** S0014  
**Phase:** `/qa`  
**Orchestrator:** `auto-20260608-us0013-001`  
**Decision:** DEC-0076  
**QA agent:** fresh subagent (`qa-20260608-s0014-us0013`)  
**Date:** 2026-06-08  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — All blocking acceptance criteria (AC-1 through AC-9) satisfied via code review and automated guards. Prerequisite AC-10 (BUG-0010 AA/AB/AC) already checked at intake. Runtime omniflow UAT deferred to `/verify-work` pending operator **BACKEND_COMPOSE_DEPLOY**. Hand off to `/verify-work`.

## Test plan

| # | Check | Method | Result |
|---|-------|--------|--------|
| 1 | AC-1 external overlay + env docs | Code review `docker-compose.external.yml`, `.env.example`; run `scripts/compose-config-check.sh` | **PASS** |
| 2 | AC-2 sidecar health gate | Code review `backend/src/forecast_ml/service.rs`, `sidecar.rs` | **PASS** |
| 3 | AC-3 sync ML phase + skip metadata | Code review `backend/src/sync/mod.rs`, `forecast_ml/service.rs` | **PASS** |
| 4 | AC-4 ml_enhanced API + min history | Code review `backend/src/api/forecast.rs`, `config/default.toml` | **PASS** |
| 5 | AC-5 Forecast Compare + degraded UX | Code review `frontend/src/pages/ForecastPage.tsx` | **PASS** |
| 6 | AC-6 Wealth portfolio overlay | Code review `frontend/src/pages/WealthPage.tsx`, `backend/src/wealth/portfolio_forecast.rs` | **PASS** |
| 7 | AC-7 Grafana ML panels + banner | Code review `grafana/.../forecast-horizons.json` | **PASS** |
| 8 | AC-8 runbook omniflow ML | Code review `docs/engineering/runbook.md` § 7a | **PASS** |
| 9 | AC-9 CI dual guard | Run `forecast_ml_integration`; inspect `tests/run-tests.sh` | **PASS** |
| 10 | Prerequisite AC-10 | Intake evidence `docs/product/acceptance.md` BUG-0010 checked | **PASS** (pre-verified) |
| 11 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |

## Automated test output

```
$ bash scripts/compose-config-check.sh
==> compose-config-check: all checks passed
EXIT_CODE=0

$ cargo test --test forecast_ml_integration
running 3 tests
test grafana_dashboard_has_forecast_variant ... ok
test ml_skip_records_metadata_not_sync_failure ... ok
test sidecar_client_mock_success ... ok
test result: ok. 3 passed; 0 failed
EXIT_CODE=0
```

## Acceptance criteria results

| AC | Requirement | Result | Evidence |
|----|-------------|--------|----------|
| AC-1 | External overlay adds `stats-forecast` on `external` profile; ML env vars documented; compose-check PASS | **PASS** | `docker-compose.external.yml` lines 54–59 (stats-forecast traefik + port 8091); lines 17–19 (FORECAST_ML_ENABLED/STATS_FORECAST_URL passthrough); `.env.example` lines 59–68; `compose-config-check.sh` asserts 3-service set + ML env defaults — QA run exit 0 |
| AC-2 | `[forecast_ml] enabled=true` on external merge resolves sidecar health before sync ML phase | **PASS** | `config/mod.rs` lines 971–977 env merge; `service.rs` lines 55–60 `health_ok()` gate before recompute; `sidecar.rs` lines 73–78 GET `/health` |
| AC-3 | Post-sync `forecast_ml` phase after baseline; skip metadata on failure; sync UI shows "ML forecast…" | **PASS** | `sync/mod.rs` lines 292–318 ML phase after baseline with `record_skip_on_baseline` on Err; line 452 phase label `"ML forecast…"`; `service.rs` lines 283–297 skip reasons `sidecar_disabled` / `sidecar_unavailable` / `insufficient_history` |
| AC-4 | `model_kind=ml_enhanced` persisted with p10/p90; API `variant=ml_enhanced` returns 6–24 month series | **PASS** | `default.toml` `min_monthly_points = 12`; `config/mod.rs` `default_min_monthly_points() -> 12`; `api/forecast.rs` lines 357–409 horizons `[3,6,12,24]` + `ml_enhanced` variant with bands; `repository.rs` inserts `model_kind='ml_enhanced'` |
| AC-5 | React `/forecast` Compare shows baseline + ML overlay; `sidecar_disabled` copy per DEC-0066 | **PASS** | `ForecastPage.tsx` lines 25, 303–313 Compare segmented control; lines 53–75 `sidecar_disabled` → "ML forecast is not enabled on this deployment" (not generic unavailable); compare chart lines 402–420 |
| AC-6 | Wealth ML portfolio overlay; signed totals; FX incomplete banner | **PASS** | `WealthPage.tsx` lines 99–100 signed overdrawn copy; lines 218–233 portfolio horizons; lines 235–242 FX incomplete warning banner; `portfolio_forecast.rs` `fx_incomplete_warning` field wired from `api/wealth.rs` |
| AC-7 | Grafana ML panels with `$forecast_variant=ml_enhanced`; default `baseline`; ML status banner when off | **PASS** | `forecast-horizons.json` variable default `baseline` (lines 223–230); ML panel SQL uses `model_kind='ml_enhanced'`; text panel id 13 "ML forecast not enabled on this deployment" (BUG-0009 banner, lines 167–174) |
| AC-8 | Operator runbook documents omniflow ML enablement | **PASS** | `runbook.md` § 7a (lines 2124–2182): compose profile union, env vars, health probe, min history ≥12, degraded-mode table, cold start, memory note, user-guide cross-link |
| AC-9 | CI fixture proves sidecar invoke + overlay persist without production secrets | **PASS** | `forecast_ml_integration.rs`: wiremock sidecar POST, skip metadata on unavailable, Grafana variant contract; both guards wired in `tests/run-tests.sh` lines 49 and 70 — QA run 3/3 PASS |
| AC-10 | Prerequisite BUG-0010 AA/AB/AC DONE | **PASS** | `docs/product/acceptance.md` line 154 checked at intake |

## Findings summary

| ID | Severity | Finding | Blocking US-0013 |
|----|----------|---------|------------------|
| — | — | No findings | — |

**Blocking findings:** 0  
**Critical findings:** 0

## Operator gate (non-blocking for QA)

Runtime omniflow ML smoke (UAT-1 … UAT-9 in `sprints/S0014/uat.md`) requires operator **BACKEND_COMPOSE_DEPLOY**: deploy S1 overlay, set `FORECAST_ML_ENABLED=true`, restart `flow-finance-ai` + `stats-forecast`, confirm `curl localhost:8091/health`, Full Firefly sync. Deferred to `/verify-work` — pass-with-prerequisites pattern consistent with prior sprints.

## Handoff

**Next phase:** `/verify-work` in fresh subagent/chat  
**Stop reason:** QA_PASS — US-0013 AC-1 through AC-9 verified; no `handoffs/qa_to_dev.md` required
