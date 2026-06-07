# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## discovery-20260608-us0013 — US-0013 production ML hardening discovery`
- Last archived heading: `## discovery-20260608-us0013 — US-0013 production ML hardening discovery`
- Verification tuple (mandatory):
  - archived_body_lines=84
  - retained_body_lines=483

---

## discovery-20260608-us0013 — US-0013 production ML hardening discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-08  
**Story:** US-0013  
**Orchestrator run:** auto-20260608-us0013-001  
**Next phase:** `/research`

### Summary

Discovery confirms **US-0013 closes the AC3 production enablement gap** deferred from BUG-0010 Q0013 — not new ML research or UI greenfield. US-0009 already ships sync `forecast_ml`, sidecar client, API `variant=ml_enhanced`, React Compare/bands, wealth portfolio outlook, and Grafana ML panels. Omniflow today is **baseline-only by design** (DEC-0049): `stats-forecast` never starts on `--profile external`, `[forecast_ml] enabled=false`, zero `ml_enhanced` rows. Execute work is **compose overlay + operator opt-in + verification smoke + runbook/CI**.

### Partial implementation review

| Area | Status | Execute note |
|------|--------|--------------|
| `docker-compose.yml` `stats-forecast` | `profiles: [full]` only | S1: external overlay or dual-profile |
| `docker-compose.external.yml` | No sidecar; backend traefik-only network | S1: add sidecar on traefik + env passthrough |
| `.env.example` | `STATS_FORECAST_PORT=8091` only | S1: document `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL` |
| Backend sync/API | **Done** (US-0009) | S2: verify on external after enable |
| React Forecast/Wealth | **Done** (US-0009) | S3: omniflow smoke only |
| Grafana forecast-horizons | **Done** (queries) | S3: `$forecast_variant=ml_enhanced` smoke |
| Runbook | Generic ML steps | S4: omniflow section |
| `forecast_ml_integration` test | wiremock + DATABASE_URL | S4: CI/compose extension |

### Critical compose finding

External overlay puts `flow-finance-ai` on **`traefik` network only** (host ports reset). Sidecar must share reachable DNS with backend. Recommended S1 pattern: `stats-forecast` block in `docker-compose.external.yml` with `profiles: [external]`, `networks: [traefik]`, port `${STATS_FORECAST_PORT:-8091}:8090`, plus `FORECAST_ML_ENABLED` / `STATS_FORECAST_URL` on `flow-finance-ai`.

### UX confirmation (unchanged from intake)

- Forecast Long-term: Baseline \| ML-enhanced \| Compare segmented control; confidence bands; Compare overlay
- Degraded: `sidecar_disabled` copy + disabled ML tabs (DEC-0066) — **already in ForecastPage**
- Wealth: portfolio outlook cards + FX incomplete banner (R-0034) when ML data present
- Grafana: ML panels populate post-enablement; BUG-0009 banner when ML off

### Discovery decomposition evidence

- Feature/workflow count: compose + config + verify sync/API/UI/Grafana + runbook/CI (moderate — **single epic retained**)
- Cross-cutting impact: compose, `.env.example`, runbook, CI; backend/React/Grafana mostly verify-only
- Acceptance breadth: **10 rows unchanged**
- Risk surface: traefik network sidecar reachability, profile union duplication, min history gate, host memory (R-0044)

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#US-0013` | Discovery notes + partial impl audit | pass |
| `docs/product/acceptance.md` US-0013 | 10 criteria still valid | pass |
| `docker-compose.yml` + `docker-compose.external.yml` | Sidecar gap documented | pass |
| `backend/src/forecast_ml/` + `sync/mod.rs` | ML pipeline present | pass |
| `frontend/src/pages/ForecastPage.tsx` | Compare + degraded copy | pass |
| R-0071 | Enablement path linked | pass |

`triad_hot_surface`: pending post-write rollover + `--check`

### Open questions (carry to `/research`)

| Topic | Question |
|-------|----------|
| **Profile union** | `profiles: [full, external]` on base vs external-only overlay — avoid duplicate sidecar on `full+external` |
| **Network** | Confirm traefik-only sidecar attachment vs dual-network backend |
| **Sidecar SLO** | Health probe timing on shared omniflow host (R-0044) |
| **Min history** | Production mirror ≥12 monthly points post–Full sync or skip path (DEC-0051) |
| **CI scope** | Compose `config --services` assert vs wiremock integration only |

### Recommended next steps

1. `/research` — resolve open questions; extend R-0071 with external overlay compose pattern + network contract
2. `/architecture` — DEC for external-profile ML enablement (env, network, profile union)
3. `/sprint-plan` — materialize US-0013-S1..S4 tasks

### Evidence

- Vision: `docs/product/vision.md` (Discovery notes US-0013 2026-06-08)
- Backlog: `docs/product/backlog.md#US-0013` (#### Discovery notes 2026-06-08)
- Intake: `handoffs/intake_evidence/intake-20260608-us0013.json`
- Research: [R-0071](docs/engineering/research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile)
- Acceptance: `docs/product/acceptance.md` § US-0013 (10 rows, unchanged)
- Architecture: `docs/engineering/architecture.md` § US-0009

---

