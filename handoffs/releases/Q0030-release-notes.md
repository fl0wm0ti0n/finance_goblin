# Quick Release Notes — Q0030 / BUG-0023

**Quick task:** Q0030  
**Bug:** BUG-0023 — Crypto Wealth EUR values missing (live regression)  
**Date:** 2026-06-12  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0023 rows **BO**, **BP**, **BQ**; live API/UI operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --test bug0023_crypto_wealth_eur` (4/4); `cargo test --lib` (218/218); `npm test` (9/9); `sprints/quick/Q0030/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/quick/Q0030/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS-WITH-PREREQUISITES — `sprints/quick/Q0030/uat.json`, `sprints/quick/Q0030/uat.md`, `sprints/quick/Q0030/verify-work-findings.md`; 9 steps — 1 pass, 8 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260612-bug0023-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Crypto wealth EUR display hardening per **DEC-0064** (wallet-only subtotal), **DEC-0080** (Bitunix wallet ingest + linear unrealized EUR), **DEC-0081** (holdings_all display surface), **DEC-0038** (wallet-priced baseline + total return %). Fixes BO subtotal €0, BP Value EUR column empty, BQ total return missing.

| Scope | Fix |
|-------|-----|
| **BO1–BO3** | `backend/src/exchanges/bitunix.rs` — equity fallback (`crossUnrealizedPNL` + `isolationUnrealizedPNL`), `code==0` reject, parse-skip warn, OpenAPI wiremock |
| **BP1** | `backend/migrations/017_bug0023_exposure_eur.sql` + `pnl.rs` — `exposure_eur` persist from `entryValue` |
| **BP2** | `backend/src/wealth/service.rs` — `holdings_all.value_eur = market_value_eur.or(exposure_eur)`; wallet-only subtotal |
| **BQ1** | `backend/src/portfolio/service.rs` — baseline captured before `total_return_pct` in same recompute |
| **T1/G1** | `bug0023_crypto_wealth_eur` 4/4; lib 218/218; npm 9/9 |
| **V1** | verify-work BO/BP/BQ oracles — pass; live API/UI deferred deploy |

**Code proof:** bug0023 4/4; cargo lib 218/218; npm 9/9; integration oracles `bo_futures_wallet_priced_subtotal_nonzero`, `bp_linear_exposure_eur_value_without_subtotal_merge`, `bq_priced_wallet_baseline_total_return_pct` PASS.

**Operator post-release:** Rebuild backend (**BACKEND_DEPLOY** + migration 017) → Bitunix exchange sync (**EXCHANGE_SYNC**) → PnL recompute (**PNL_RECOMPUTE**).

---

## Run

**Target service:** `flow-finance-ai` (backend only — bitunix ingest, pnl, wealth, portfolio).

**Deploy (backend):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

- `start_command`: docker compose commands above
- `runtime_mode`: local (`:18080`) and remote (omniflow external US-0010)
- `runtime_context_ref`: `docs/engineering/runbook.md` (§35 BUG-0023 hotfix)

**Operator gate — BACKEND_DEPLOY (required before BO/BP/BQ live oracles):**

Rebuild backend image; migration `017_bug0023_exposure_eur.sql` applies on startup via sqlx migrate. Confirm tests pass before docker build:

```bash
cd backend && cargo test --test bug0023_crypto_wealth_eur
cd backend && cargo test --lib
```

**Operator gate — EXCHANGE_SYNC (required after deploy):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
# Or Settings → Sync → exchange sync for Bitunix
```

**Operator gate — PNL_RECOMPUTE (required after exchange sync):**

Post-sync PnL recompute populates `exposure_eur`, wallet `market_value_eur`, baseline snapshot, and `total_return_pct`. Triggered automatically after exchange sync in normal pipeline; confirm via wealth API.

---

## Connect

- `service_url`: `http://localhost:18080` (local) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 18080 (local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET http://localhost:18080/health` → JSON 200
- Wealth UI: `http://localhost:18080/wealth` (Crypto tab)
- BO/BP/BQ API oracle: `GET /api/v1/wealth`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | External PostgreSQL — migration 017 + holdings probe |
| `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` | Bitunix exchange sync (operator shell only) |
| `AUTHENTIK_SECRET_KEY` | External compose profile build gate (set dummy for local external profile) |
| OIDC provider config | Omniflow OIDC-1 regression — Compose/env only |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(BO)** | `GET /api/v1/wealth` | `crypto.subtotal_eur` ~€2000 order of magnitude; `bitunix.subtotal_eur` not €0 |
| **(BO)** | Wealth → Crypto — Bitunix card | Not **€-0,00** with 11 open positions |
| **(BO)** | `exchange_holdings` futures row | `market_value_eur` populated (optional AP1_SQL_PROBE) |
| **(BP)** | `GET /api/v1/wealth` | Linear `holdings_all[].value_eur` non-null when `entryValue` present |
| **(BP)** | Holdings table Value EUR column | Not all em dash |
| **(BP)** | Subtotal contract | `crypto.subtotal_eur` = wallet only; linear `market_value_eur` NULL per DEC-0064 |
| **(BQ)** | `GET /api/v1/wealth` | `pnl.total_return_pct` non-null when baseline exists |
| **(BQ)** | PnL summary Total return % | Not em dash with non-zero unrealized EUR |
| Regression | OIDC-1 omniflow `/api/v1/wealth` | HTTP 200 |

**Automated (release):**

```bash
cd backend && cargo test --test bug0023_crypto_wealth_eur
cd backend && cargo test --lib
cd frontend && npm test && npm run build
```

**Live (operator):** UAT steps in `sprints/quick/Q0030/uat.json`.

---

## Credentials

- `DATABASE_URL` — external PostgreSQL (operator shell only — no inline secrets)
- `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` — exchange sync credentials via Compose/env only
- OIDC provider config via Compose/env only

---

## Changes

| Area | Summary |
|------|---------|
| `backend/migrations/017_bug0023_exposure_eur.sql` | BP1 `exposure_eur` column |
| `backend/src/exchanges/bitunix.rs` | BO1–BO3 wallet ingest hardening |
| `backend/src/exchanges/repository.rs` | exposure_eur persist path |
| `backend/src/portfolio/pnl.rs` | BP1 entryValue → exposure_eur; linear market_value_eur NULL |
| `backend/src/portfolio/service.rs` | BQ1 baseline-before-return order |
| `backend/src/wealth/service.rs` | BP2 value_eur mapping; wallet-only subtotal |
| `backend/tests/bug0023_crypto_wealth_eur.rs` | T1/G1 regression (4/4) |
| Runbook | §35 BUG-0023 operator smoke |

**Linked decisions:** DEC-0064 (wallet-only subtotal); DEC-0080 (Bitunix wallet parse); DEC-0081 (holdings_all display); DEC-0038 (wallet-priced return denominator)  
**Research fulfilled:** R-0093  
**Deferred:** BO/BP/BQ live ~€2000 oracle until **BACKEND_DEPLOY → EXCHANGE_SYNC → PNL_RECOMPUTE**

---

## Known Issues

- Running container serves pre-Q0030 backend — BO/BP/BQ changes not live until rebuild
- Live `:18080` confirms pre-deploy baseline: `crypto.subtotal_eur` -0.0, all `value_eur` null, `total_return_pct` null
- `holdings_top` still filters on `market_value_eur` only — linear rows in `holdings_all` but not top-5 card (DEC-0064, not a defect)

---

## Regression scope

- CategoryFilter static import (BUG-0021) unchanged
- Subscription list (BUG-0020) unchanged
- Grafana provisioning (BUG-0019) unchanged
- Alert evaluation (BUG-0018) unchanged
- `fx_incomplete` gate preserved

---

## Rollback

```bash
git revert <Q0030-code-commits>
docker compose up -d --build flow-finance-ai
```

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0030 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0023-q0030`

## Milestone

**BUG-0023 released** — wallet ingest hardening + exposure_eur display + baseline ordering; BO/BP/BQ code oracles PASS; deploy operator-deferred per pass-with-prerequisites.
