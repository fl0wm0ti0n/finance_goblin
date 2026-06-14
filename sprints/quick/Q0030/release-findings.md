# Release Findings — Quick Q0030 / BUG-0023

**Quick task:** Q0030  
**Bug:** BUG-0023  
**Phase:** `/release`  
**Date:** 2026-06-12  
**Orchestrator:** `auto-20260612-bug0023`  
**Gate status:** **PASS**

## Gate audit (US-0039)

| gate | verdict | reason_code | remediation | evidence_refs |
|------|---------|-------------|-------------|---------------|
| check-in_test | pass | — | — | `bug0023_crypto_wealth_eur` 4/4; `cargo test --lib` 218/218; `npm test` 9/9; `sprints/quick/Q0030/qa-findings.md` |
| qa | pass | — | — | `sprints/quick/Q0030/qa-findings.md` (0 blockers) |
| uat | pass-with-prerequisites | — | — | `sprints/quick/Q0030/uat.json`, `sprints/quick/Q0030/uat.md`, `sprints/quick/Q0030/verify-work-findings.md` — 1 pass, 8 pass_with_prerequisites, 0 fail |
| isolation | pass | — | — | `docs/engineering/state.md` (execute→verify-work checkpoints) |
| runtime_proof | pass | — | — | `runtime-proof-verify-work-20260612-bug0023-001`; release tuple at finalization |
| finalization | pass | — | — | `handoffs/releases/Q0030-release-notes.md`, backlog BUG-0023 DONE |
| publish | skipped | — | — | `RELEASE_PUBLISH_MODE=disabled` |

**Doc gates:** README feature coverage `skipped` (`README_FEATURE_COVERAGE_ENFORCE=0`)

## Release verdict

**PASS** — BUG-0023 finalized; acceptance **BO**, **BP**, **BQ** checked; operator deploy/sync/recompute deferred (**BACKEND_DEPLOY → EXCHANGE_SYNC → PNL_RECOMPUTE**).

## Blocking findings

None.

## Non-blocking findings

- **BACKEND_DEPLOY** — running container predates Q0030 BO/BP/BQ changes; migration 017 not yet applied on live `:18080`
- **BO-API / BO-UI / BO-SQL** — pass-with-prerequisites; integration `bo_futures_wallet_priced_subtotal_nonzero` PASS; live `crypto.subtotal_eur` -0.0 pre-deploy
- **BP-API / BP-UI / BP-SUBTOTAL** — pass-with-prerequisites; integration `bp_linear_exposure_eur_*` PASS; live all `value_eur` null pre-deploy
- **BQ-API / BQ-UI** — pass-with-prerequisites; integration `bq_priced_wallet_baseline_total_return_pct` PASS; live `total_return_pct` null despite `unrealized_eur` 376.83 pre-deploy
- **AP1_SQL_PROBE** — optional post-deploy gate for `exchange_holdings` futures `market_value_eur`

## Deployment steps

1. **Confirm tests pass:**

```bash
cd backend && cargo test --test bug0023_crypto_wealth_eur
cd backend && cargo test --lib
```

2. **Rebuild backend (migration 017 auto-applies):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override:

```bash
docker compose up -d --build flow-finance-ai
```

3. **Exchange sync:**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
```

4. **Confirm PnL recompute** — verify `GET /api/v1/wealth` post-sync per operator gates

5. **Verify operator gates** — BO/BP/BQ oracles per `sprints/quick/Q0030/uat.json`

## Rollback

```bash
git revert <Q0030-code-commits>
docker compose up -d --build flow-finance-ai
```

## Operator follow-up (post-release, optional)

1. **BO API oracle** — `GET /api/v1/wealth` `crypto.subtotal_eur` ~€2000; Bitunix card not €0
2. **BP API/UI oracle** — linear `holdings_all[].value_eur` non-null; Value EUR column populated
3. **BQ API/UI oracle** — `pnl.total_return_pct` non-null with non-zero unrealized
4. **AP1_SQL_PROBE** — optional `exchange_holdings` futures `market_value_eur` check
5. **Omniflow OIDC-1** — repeat `/wealth` `/api/v1/wealth` smoke on `https://financegnome.omniflow.cc`

## Operator smoke checklist

1. `cargo test --test bug0023_crypto_wealth_eur` + `cargo test --lib` — **PASS** (release)
2. Rebuild `flow-finance-ai` — **PENDING**
3. Exchange sync (Bitunix) — **PENDING** (deploy)
4. PnL recompute — **PENDING** (post-sync)
5. `GET /api/v1/wealth` subtotal ~€2000 — **PENDING** (deploy+sync)
6. Linear `value_eur` non-null — **PENDING** (deploy+recompute)
7. `total_return_pct` non-null — **PENDING** (deploy+recompute)
8. OIDC-1 omniflow wealth API — **PASS** (verify-work)

## Rerun criteria

N/A — release finalization PASS.
