# Verify-work Findings — Quick Q0032 / BUG-0026

**Work item:** BUG-0026 (defect)  
**Quick task:** Q0032  
**Phase:** `/verify-work`  
**Date:** 2026-06-13  
**Orchestrator:** `auto-20260613-bug0026`  
**Decisions:** DEC-0089  
**QA agent:** fresh subagent (`verify-work-20260613-bug0026-qa-fresh`)

## Verdict

**PASS-WITH-PREREQUISITES** — Independent re-run confirms BZ/CA implementation under frozen gates **GATE-MONTH-1** / **GATE-LABEL-1** / **GATE-SCOPE-1** / **GATE-TEST-1** / **GATE-DEC-1**: `resolveForecastSummaryPoint` skips partial zero-income head; shared subtitle **"Forecast for July 2026"**; **DEC-0089** category filter unchanged. Automated gates **24/24** npm, **PASS** build. Live browser on account **114** reproduces pre-deploy bug (Income **0.00**, no subtitle) — consistent with pending **FRONTEND_DEPLOY**. API oracle confirms series[1] income **3266.16**. **0 blockers.** Ready for **`/release`**.

## Scope

**Inputs reviewed:** `handoffs/dev_to_qa.md` (Q0032 top section), `sprints/quick/Q0032/qa-findings.md`, `docs/product/acceptance.md` BUG-0026 (BZ/CA), `sprints/quick/Q0032/uat.md`, `frontend/src/pages/forecastSummaryMonth.ts`, `frontend/src/pages/forecastSummaryMonth.test.ts`, `frontend/src/pages/ForecastPage.tsx`. No host `.env`/secret files read.

## Operator gates

| Gate | Status | Action | Notes |
|------|--------|--------|-------|
| **FRONTEND_DEPLOY** | pending | Rebuild frontend only — no migration | Required before live BZ/CA UI probes on `/forecast` Monthly account **114** |

**Post-gate smoke:** `/forecast` Monthly account **114** — Income card **3266.16** matches July chart bar; subtitle **"Forecast for July 2026"** above four cards; category filter does not alter card values.

## Live probe — pre-deploy baseline (2026-06-13)

| Probe | HTTP / observation | Key fields | Interpretation |
|-------|-------------------|------------|----------------|
| `GET /health` | 200 | OK | Stack reachable |
| `GET /api/v1/forecast/monthly?account_id=114` | 200 | series[0] income 0.00; series[1] income **3266.16** | **BZ-API** oracle PASS — partial-month trap data present |
| Browser `/forecast` Monthly acct 114 | 200 | Income card **0.00**; no subtitle | Pre-deploy repro — H1/F1 fix not yet in running container |
| Category filter helper | visible | DEC-0089 text unchanged | Regression PASS |

### Forecast monthly API snapshot (account 114)

| Index | Month | Income |
|-------|-------|--------|
| 0 | 2026-06-01 | 0.00 |
| 1 | 2026-07-01 | **3266.16** |
| 2 | 2026-08-01 | 3266.16 |

Live baseline confirms operator **FRONTEND_DEPLOY** not yet applied; consistent with QA V1 deferral and Q0031 pass-with-prerequisites precedent.

## Per-row verdict (acceptance BZ / CA)

| Row | Verdict | Evidence |
|-----|---------|----------|
| **BZ** | **pass_with_prerequisites** | Vitest partialMonthTrap + resolveForecastSummaryPoint; browser Income 0.00 pre-deploy; API oracle 3266.16 at series[1] |
| **CA** | **pass_with_prerequisites** | Vitest subtitle PASS; browser subtitle absent pre-deploy — deferred FRONTEND_DEPLOY |

## Automated checks (verify-work re-run)

| Check | Result |
|-------|--------|
| `npm test` | **24/24 PASS** (2.31s) — forecastSummaryMonth 7/7 |
| `npm run build` | **PASS** — tsc + vite build (14.29s) |

### npm test output

```
Test Files  6 passed (6)
Tests       24 passed (24)
  forecastSummaryMonth.test.ts  7 passed
```

## UAT step matrix

| Step | Row | Result | Evidence |
|------|-----|--------|----------|
| BZ-UI | BZ | pass_with_prerequisites | Browser Income 0.00 pre-deploy; vitest July 3266.16 PASS |
| BZ-API | BZ | **pass** | Live API series[1] income 3266.16 |
| CA-UI | CA | pass_with_prerequisites | Subtitle absent pre-deploy; vitest PASS |
| DEC-0089 | regression | **pass** | Helper text + code review |
| OIDC-1 | regression | pass_with_prerequisites | /forecast + monthly API 200; fix deferred deploy |

## UAT matrix summary

| Result | Count |
|--------|-------|
| pass | **2** |
| pass_with_prerequisites | **3** |
| fail | **0** |
| pending | **0** |

## Runtime browser evidence

| Probe | navigation_url | reason_code | Ref |
|-------|----------------|-------------|-----|
| BZ-UI-browser | http://localhost:18080/forecast | UAT_BROWSER_PROBE_FAILED (expected pre-deploy) | `sprints/quick/Q0032/evidence/browser/bz-ui-probe-summary.txt` |
| CA-UI-browser | http://localhost:18080/forecast | UAT_BROWSER_PROBE_FAILED (expected pre-deploy) | same |

## Acceptance impact

| Row | Verify-work | Post-operator (release follow-up) |
|-----|-------------|-----------------------------------|
| **BZ** | pass_with_prerequisites | Income card **3266.16** matches July chart bar after FRONTEND_DEPLOY |
| **CA** | pass_with_prerequisites | Subtitle **"Forecast for July 2026"** visible above cards after FRONTEND_DEPLOY |

## Next phase

**`/release`** — release notes; operator gate checklist; backlog BUG-0026 remains open until post-deploy smoke PASS.

`fresh_context_marker`: verify-work-20260613-bug0026-qa-fresh  
`runtime_proof_id`: runtime-proof-verify-work-20260613-bug0026-001  
`phase_boundary`: verify-work → release
