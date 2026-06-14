# Release Findings — Quick Q0032 / BUG-0026

**Quick task:** Q0032  
**Bug:** BUG-0026  
**Phase:** `/release`  
**Date:** 2026-06-13  
**Orchestrator:** `auto-20260613-bug0026`  
**Gate status:** **PASS**

## Gate audit (US-0039)

| gate | verdict | reason_code | remediation | evidence_refs |
|------|---------|-------------|-------------|---------------|
| check-in_test | pass | — | — | `npm test` 24/24; `npm run build` PASS; `sprints/quick/Q0032/qa-findings.md`; `sprints/quick/Q0032/verify-work-findings.md` |
| qa | pass | — | — | `sprints/quick/Q0032/qa-findings.md` (0 blockers) |
| uat | pass-with-prerequisites | — | — | `sprints/quick/Q0032/uat.json`, `sprints/quick/Q0032/uat.md`, `sprints/quick/Q0032/verify-work-findings.md` — 2 pass, 3 pass_with_prerequisites, 0 fail |
| isolation | pass | — | — | `docs/engineering/state.md` (execute→verify-work checkpoints) |
| runtime_proof | pass | — | — | `runtime-proof-verify-work-20260613-bug0026-001`; release tuple at finalization |
| finalization | pass | — | — | `handoffs/releases/Q0032-release-notes.md`, backlog BUG-0026 DONE |
| publish | skipped | — | — | `RELEASE_PUBLISH_MODE=disabled` |

**Doc gates:** README feature coverage `skipped` (`README_FEATURE_COVERAGE_ENFORCE=0`); project README coverage `pass` (frontend-only scope; no new US id)

## Release verdict

**PASS** — BUG-0026 finalized; acceptance **BZ**, **CA** checked; operator **FRONTEND_DEPLOY** deferred.

## Blocking findings

None.

## Non-blocking findings

- **FRONTEND_DEPLOY** — running container predates Q0032; browser reproduces Income **0.00** + no subtitle pre-deploy
- **BZ-UI / CA-UI** — pass-with-prerequisites; `forecastSummaryMonth` vitest 7/7 PASS; live UI deferred deploy
- **BZ-API** — **pass** — live `GET /api/v1/forecast/monthly?account_id=114` series[1] income **3266.16**
- **DEC-0089** — **pass** — category filter unchanged on cards; helper text present
- **OIDC-1** — pass-with-prerequisites; `/forecast` + monthly API 200; fix not live until FRONTEND_DEPLOY

## Deployment steps

1. **Confirm tests pass:**

```bash
cd frontend && npm test && npm run build
```

2. **Rebuild frontend (no migration):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override:

```bash
docker compose up -d --build flow-finance-ai
```

3. **Verify operator gates** — BZ/CA oracles per `sprints/quick/Q0032/uat.json`

## Rollback

```bash
git revert <Q0032-code-commits>
docker compose up -d --build flow-finance-ai
```

## Operator follow-up (post-release, optional)

1. **BZ UI oracle** — `/forecast` Monthly account **114** — Income card **3266.16** matches July chart bar
2. **CA UI oracle** — subtitle **"Forecast for July 2026"** above four summary cards
3. **DEC-0089 regression** — category filter does not alter card values
4. **Omniflow OIDC-1** — repeat `/forecast` smoke on `https://financegnome.omniflow.cc`

## Operator smoke checklist

1. `npm test` + `npm run build` — **PASS** (release)
2. Rebuild `flow-finance-ai` — **PENDING**
3. BZ Income card **3266.16** on account **114** — **PENDING** (deploy)
4. CA subtitle **Forecast for July 2026** — **PENDING** (deploy)
5. BZ-API series[1] income **3266.16** — **PASS** (verify-work)
6. DEC-0089 category filter regression — **PASS** (verify-work)
7. OIDC-1 omniflow `/forecast` — **PASS** (verify-work partial)

## Rerun criteria

N/A — release finalization PASS.
