# Release Findings — Quick Q0034 / BUG-0025

**Quick task:** Q0034  
**Bug:** BUG-0025  
**Phase:** `/release`  
**Date:** 2026-06-14  
**Orchestrator:** `auto-20260613-bug0025`  
**Gate status:** **PASS**

## Gate audit (US-0039)

| gate | verdict | reason_code | remediation | evidence_refs |
|------|---------|-------------|-------------|---------------|
| check-in_test | pass | — | — | cargo lib 221/221; bug0025 integration 3/3; npm 31/31; build PASS; `sprints/quick/Q0034/qa-findings.md`; `sprints/quick/Q0034/verify-work-findings.md` |
| qa | pass | — | — | `sprints/quick/Q0034/qa-findings.md` (0 blockers) |
| uat | pass-with-prerequisites | — | — | `sprints/quick/Q0034/uat.json`, `sprints/quick/Q0034/uat.md`, `sprints/quick/Q0034/verify-work-findings.md` — 2 pass, 6 pass_with_prerequisites, 0 fail |
| isolation | pass | — | — | `docs/engineering/state.md` (execute→verify-work checkpoints) |
| runtime_proof | pass | — | — | `runtime-proof-verify-work-20260614-bug0025-001`; release tuple at finalization |
| finalization | pass | — | — | `handoffs/releases/Q0034-release-notes.md`, backlog BUG-0025 DONE |
| publish | skipped | — | — | `RELEASE_PUBLISH_MODE=disabled` |

**Doc gates:** README feature coverage `skipped` (`README_FEATURE_COVERAGE_ENFORCE=0`); project README coverage `pass` (backend+frontend scope; no new US id)

**Version-doc gates:** `RELEASE_CHANGELOG_ENFORCE=1` — workflow-only release `bug0025-q0034`; semver path `[Unreleased]` only; validation `skipped` (no semver bind)

## Release verdict

**PASS** — BUG-0025 finalized; acceptance **BW**, **BX**, **BY** checked; operator **BACKEND_REBUILD** + **FRONTEND_DEPLOY** deferred.

## Blocking findings

None.

## Non-blocking findings

- **BACKEND_REBUILD** — running container predates Q0034; `last_firefly_run` absent; manual 365d ingest not live
- **FRONTEND_DEPLOY** — Sync Status hero shows exchange timestamp pre-deploy; DEC-0002 callout absent
- **BW-API** — pass_with_prerequisites — live category **146** only 2026-05; integration manual 365d 3/3 PASS
- **BW-UI** — pass_with_prerequisites — browser /forecast pre-deploy symptom; deferred deploy + manual Sync now
- **BX-UI** — pass_with_prerequisites — callout absent pre-deploy; source F1 PASS
- **BX-DOC** — **pass** — runbook `#backdated-firefly-imports` + cursor-reset SQL
- **BY-API** — pass_with_prerequisites — `last_firefly_run` field absent pre-deploy; B2 source PASS
- **BY-UI** — pass_with_prerequisites — hero exchange timestamp pre-deploy; F1 source PASS
- **BY-HIST** — **pass** — sync/runs trigger column distinguishes manual/scheduled/scheduled_exchanges
- **GATE-OVERLAP-1** — **pass** — manual 365d; scheduled watermark−7d unchanged
- **GATE-SYNC-UX-1** — **pass** — `last_firefly_run` hero; exchange secondary when newer
- **GATE-REMED-1** — **pass** — runbook cursor-reset SQL
- **GATE-TEST-1** — **pass** — integration 3/3
- **GATE-DEC-1** — **pass** — extends DEC-0002; no new DEC
- **OIDC-1** — pass_with_prerequisites; `/sync` + `/forecast` HTTP 200; fix deferred deploy gates

## Deployment steps

1. **Confirm tests pass:**

```bash
cd backend && cargo test --lib && cargo test --test bug0025_sync_transaction_window
cd frontend && npm test && npm run build
```

2. **Rebuild backend + frontend (no migration):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override:

```bash
docker compose up -d --build flow-finance-ai
```

3. **Manual Full sync** — `/sync` → **Sync now**

4. **Verify operator gates** — BW/BX/BY oracles per `sprints/quick/Q0034/uat.json`

## Rollback

```bash
git revert <Q0034-code-commits>
docker compose up -d --build flow-finance-ai
```

## Operator follow-up (post-release, optional)

1. **BW oracle** — `GET /api/v1/categories/expense-series?category_id=146` multi-month after manual Sync now
2. **BW UI oracle** — `/forecast` Category spending trend **Wohnen - Stromkosten** — bars per month
3. **BX UI oracle** — `/sync` DEC-0002 callout + runbook link visible
4. **BY API oracle** — `GET /api/v1/sync/status` — `last_firefly_run` distinct from exchange-only `last_run`
5. **BY UI oracle** — hero **Last Firefly sync** + trigger badge; exchange secondary when newer
6. **Omniflow OIDC-1** — repeat `/sync` + `/forecast` smoke on `https://financegnome.omniflow.cc`

## Operator smoke checklist

1. `cargo test --lib` + bug0025 3/3 + `npm test` + build — **PASS** (release)
2. Rebuild `flow-finance-ai` — **PENDING**
3. Manual **Sync now** — **PENDING** (post-deploy)
4. BW multi-month Stromkosten expense-series — **PENDING** (deploy + sync)
5. BX DEC-0002 callout — **PENDING** (deploy)
6. BY `last_firefly_run` hero — **PENDING** (deploy)
7. BY-HIST trigger column — **PASS** (verify-work)
8. OIDC-1 `/sync` + `/forecast` — **PASS** (verify-work partial)

## Rerun criteria

N/A — release finalization PASS.
