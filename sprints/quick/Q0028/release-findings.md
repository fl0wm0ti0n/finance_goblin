# Release Findings — Quick Q0028 / BUG-0020

**Quick task:** Q0028  
**Bug:** BUG-0020  
**Phase:** `/release`  
**Date:** 2026-06-11  
**Orchestrator:** `auto-20260610-bug0019`  
**Gate status:** **PASS**

## Gate audit (US-0039)

| gate | verdict | reason_code | remediation | evidence_refs |
|------|---------|-------------|-------------|---------------|
| check-in_test | pass | — | — | `bug0020_subscription_list_quality` 7/7; `bug0008_subscription_alerts` 8/8; `subscriptions_integration` 1/1; `sprints/quick/Q0028/qa-findings.md` |
| qa | pass | — | — | `sprints/quick/Q0028/qa-findings.md` (0 blockers) |
| uat | pass-with-prerequisites | — | — | `sprints/quick/Q0028/uat.json`, `handoffs/verify_work_to_release.md` — 9 pass, 2 pass_with_prerequisites, 0 fail |
| isolation | pass | — | — | `docs/engineering/state.md` (execute→verify-work checkpoints) |
| runtime_proof | pass | — | — | `runtime-proof-verify-work-20260610-bug0020-001`; release tuple at finalization |
| finalization | pass | — | — | `handoffs/releases/Q0028-release-notes.md`, backlog BUG-0020 DONE |
| publish | skipped | — | — | `RELEASE_PUBLISH_MODE=disabled` |

**Doc gates:** README feature coverage `skipped` (`README_FEATURE_COVERAGE_ENFORCE=0`)

## Release verdict

**PASS** — BUG-0020 finalized; acceptance **BI**, **BJ** checked; migration 016 applied manually at verify-work; container rebuild deferred (ForecastPage TS6133).

## Blocking findings

None.

## Non-blocking findings

- **BACKEND_FRONTEND_DEPLOY** — `docker build` fails `ForecastPage.tsx` TS6133; running image pre-Q0028 (DA2 All-tab filter + DA3 guard not live)
- **MIGRATION_016_APPLY** — manual psql apply succeeded; `_sqlx_migrations` v16 not registered (migration 15 checksum conflict)
- **BI-ALL** — pass-with-prerequisites; API simulation confirms DA2 filter; browser visual pending deploy
- **All-tab scope change** — rejected/inactive hidden per DEC-0109; document in operator comms

## Deployment steps

1. **Fix TS6133** — remove unused `hasForecast` in `frontend/src/pages/ForecastPage.tsx`; confirm `npm run build` passes
2. **Rebuild backend + frontend:**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

3. **Register migration 016** (if not already in `_sqlx_migrations`):

```bash
cd backend && sqlx migrate run
```

4. **Full sync regression:**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
```

5. **Verify operator gates** — BI-API, BJ oracles, BI-ALL browser smoke per `sprints/quick/Q0028/uat.json`

## Rollback

```bash
git revert <Q0028-migration-and-code-commits>
# Restore DB from backup if reconcile ran in production
docker compose up -d --build flow-finance-ai
```

## Operator follow-up (post-release, optional)

1. **sqlx migration 15 checksum** — resolve `_sqlx_migrations` conflict so v16 registers cleanly on future deploys
2. **BI-ALL browser smoke** — confirm All tab hides rejected/inactive after rebuild
3. **Omniflow OIDC-1** — repeat list endpoint smoke on `https://financegnome.omniflow.cc`

## Operator smoke checklist

1. Fix ForecastPage TS6133 + `npm run build` — **PENDING**
2. Rebuild `flow-finance-ai` — **PENDING**
3. Confirm migration 016 effects (6 confirmed, 6/6 display_category_id) — **DONE** (verify-work manual apply)
4. `GET /api/v1/subscriptions?status=confirmed` — ≤1 per payee_key — **PASS**
5. BJ oracle samples (netflix/kindle/youtube/hgp/florian) — **PASS**
6. `/subscriptions` All tab — no triplicate Strom / duplicate YouTube — **PENDING** (deploy)
7. Full sync — no new YouTube dup — **PASS**
8. discover/tags regression — **PASS**

## Rerun criteria

N/A — release finalization PASS.
