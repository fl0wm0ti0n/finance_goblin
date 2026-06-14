# UAT — Q0032 (BUG-0026)

**Status:** VERIFIED (verify-work populated)  
**Verdict:** **PASS-WITH-PREREQUISITES**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0026 rows **BZ**, **CA**  
**Sprint:** Q0032 (`/quick`)  
**Orchestrator:** `auto-20260613-bug0026`  
**Verified at:** 2026-06-13T14:45:00Z

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Result |
|-----|---------|-----------------------------------|--------|
| **BZ** | H1, F1, T1, G1, V1 | Income card consistent with chart for same labeled reference month — not 0.00 while chart shows ~€3000 bars | **pass_with_prerequisites** |
| **CA** | H1, F1, T1, G1, V1 | Summary cards show which month they represent — not unlabeled series[0] | **pass_with_prerequisites** |

## Operator gates (before live probes)

| Gate | Status | Notes |
|------|--------|-------|
| FRONTEND_DEPLOY | pending | Frontend rebuild only — H1/F1 summary month fix must be live |

## UAT steps

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| BZ-UI | BZ | `/forecast` Monthly account 114 — Income card ~3266.16 matches July chart bar | pass_with_prerequisites | Browser MCP: Income **0.00** pre-deploy; vitest partialMonthTrap July **3266.16** PASS |
| BZ-API | BZ | `GET /api/v1/forecast/monthly?account_id=114` series[1] income 3266.16 | **pass** | Live API: series[1] **3266.16**; series[0] **0.00** |
| CA-UI | CA | Subtitle "Forecast for July 2026" above four summary cards | pass_with_prerequisites | Browser MCP: subtitle absent pre-deploy; vitest subtitle PASS |
| DEC-0089 | regression | Category filter does not change card values | **pass** | Live helper text + code review monthlyQuery key |
| OIDC-1 | regression | `/forecast` /api/v1/forecast/monthly smoke on omniflow profile | pass_with_prerequisites | `/forecast` 200; monthly API 200; fix deferred FRONTEND_DEPLOY |

## Runtime browser evidence

| Probe | URL | Observation | Ref |
|-------|-----|-------------|-----|
| BZ-UI / CA-UI | http://localhost:18080/forecast | Account 114, Monthly tab — Income **0.00**, no subtitle (pre-deploy repro) | `sprints/quick/Q0032/evidence/browser/bz-ui-probe-summary.txt` |

## Results summary

| Result | Count |
|--------|-------|
| pass | **2** |
| pass_with_prerequisites | **3** |
| fail | **0** |
| pending | **0** |

**Acceptance impact:** BZ and CA verified at code/test layer; live UI confirmation deferred on operator **FRONTEND_DEPLOY** (consistent with Q0031/Q0029 precedent).

**Next phase:** `/release` (role: release)
