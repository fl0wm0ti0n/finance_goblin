# UAT — Sprint S0014 / US-0013

**Sprint:** S0014  
**Story:** US-0013  
**Phase:** verify-work complete  
**Status:** PASS (pass-with-prerequisites)  
**Plan-verified at:** 2026-06-08T09:50:00Z  
**QA verified at:** 2026-06-08T10:50:00Z  
**Verified at:** 2026-06-08T11:05:00Z  
**Orchestrator:** `auto-20260608-us0013-001`  
**Decision:** DEC-0076

## Inputs

- Acceptance: `docs/product/acceptance.md` § US-0013
- Architecture: `docs/engineering/architecture.md` § US-0013, `decisions/DEC-0076.md`
- Research: R-0071; addenda R-0043, R-0044, R-0045, R-0053, R-0062
- User guide: `docs/user-guides/US-0013.md`
- Runbook: `docs/engineering/runbook.md` § 7a Omniflow ML enablement
- QA evidence: `sprints/S0014/qa-findings.md`
- Verify-work: `sprints/S0014/verify-work-findings.md`

## Operator gate (pre-runtime smoke)

| Gate | Action |
|------|--------|
| **BACKEND_COMPOSE_DEPLOY** | Deploy S1 overlay + env; restart `flow-finance-ai` + `stats-forecast`; set `FORECAST_ML_ENABLED=true` in operator `.env` |
| **Full Firefly sync** | Run Full sync + recompute after sidecar healthy (`curl localhost:8091/health`) |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | External overlay adds `stats-forecast` on `external` profile; ML env vars documented; compose-check PASS | **pass** | T-0144–T-0147; compose-config-check exit 0 (verify-work re-run) |
| UAT-2 | AC-2 | Backend resolves sidecar health on traefik network before sync ML phase | **pass_with_prerequisites** | T-0148 code PASS; live health probe pending **BACKEND_COMPOSE_DEPLOY** |
| UAT-3 | AC-3 | Post-sync `forecast_ml` phase runs; ML failure records skip metadata; sync UI shows "ML forecast…" | **pass_with_prerequisites** | T-0148 code PASS + `ml_skip_records_metadata_not_sync_failure`; live sync UI pending deploy |
| UAT-4 | AC-4 | `ml_enhanced` persisted with bands; GET `/api/v1/forecast?variant=ml_enhanced` non-empty after Full sync | **pass_with_prerequisites** | T-0149 code PASS + `sidecar_client_mock_success`; live API pending deploy + Full sync |
| UAT-5 | AC-5 | React `/forecast` Compare shows baseline + ML overlay; degraded copy uses `sidecar_disabled` | **pass_with_prerequisites** | T-0150 code PASS; live Compare tab pending deploy |
| UAT-6 | AC-6 | Wealth API/UI ML portfolio overlay; signed totals correct; FX incomplete banner when applicable | **pass_with_prerequisites** | T-0151 code PASS; live Wealth/Crypto tab pending deploy |
| UAT-7 | AC-7 | Grafana forecast-horizons ML panels return data with `$forecast_variant=ml_enhanced` | **pass_with_prerequisites** | T-0152 code PASS + `grafana_dashboard_has_forecast_variant`; live Grafana pending deploy |
| UAT-8 | AC-8 | Runbook documents omniflow ML enablement (compose, env, health, min history, degraded mode) | **pass** | T-0153; `runbook.md` § 7a |
| UAT-9 | AC-9 | CI fixture proves sidecar invoke + overlay persist (wiremock + compose assert) | **pass** | T-0154; `forecast_ml_integration` 3/3 (verify-work re-run) |
| UAT-10 | Prerequisite | BUG-0010 AA/AB/AC DONE — baseline authoritative | **pass** (pre-checked) | Q0013 release |

## Omniflow smoke steps (post-deploy — operator)

### Forecast (AC-5 / T-0150)

1. Open `/forecast` → Long-term tab
2. With ML disabled: confirm copy reads "ML forecast is not enabled on this deployment" (not generic "ML forecast unavailable")
3. After ML enabled + Full sync: Compare tab shows baseline + ML overlay when `ml_enhanced` data exists

### Wealth (AC-6 / T-0151)

1. Open `/wealth` → Crypto tab
2. After ML enabled + Full sync: portfolio horizon cards (3/6/12 mo) render when data present
3. With incomplete FX: amber "FX incomplete" banner (warning, not hard block)
4. Overview tab: signed total includes overdrawn accounts (negative reduces net worth)

### Grafana (AC-7 / T-0152)

1. Open `/analytics/forecast-horizons`
2. Default `$forecast_variant` = Baseline — panels load non-empty for funded account
3. Switch to ML Enhanced — ML panels populate after Full sync with ML data
4. With ML off: "ML forecast not enabled on this deployment" banner visible (BUG-0009)

## Results summary

- **Verdict:** **PASS** — 9/9 acceptance criteria satisfied at code/test level; 6/9 runtime probes **pass-with-prerequisites** pending operator **BACKEND_COMPOSE_DEPLOY**
- **Blocking pass:** AC-1 (compose/config), AC-8 (runbook), AC-9 (CI fixture)
- **Pass-with-prerequisites:** AC-2 through AC-7 (omniflow ML smoke)
- **Prerequisite:** AC-10 (BUG-0010) pre-checked at intake
- **Blockers:** none

## Next phase

**`/release`** — check US-0013 acceptance rows; publish release notes; update backlog → DONE
