# Verify-work Findings — Quick Q0030 / BUG-0023

**Work item:** BUG-0023 (defect)  
**Quick task:** Q0030  
**Phase:** `/verify-work`  
**Date:** 2026-06-12  
**Orchestrator:** `auto-20260612-bug0023`  
**Decisions:** DEC-0064, DEC-0080, DEC-0081, DEC-0038  
**QA agent:** fresh subagent (`verify-work-20260612-bug0023-qa-fresh`)

## Verdict

**PASS-WITH-PREREQUISITES** — Code and integration oracles confirm BO/BP/BQ implementation; live `:18080` probe documents expected **pre-deploy baseline** (symptoms still present). Operator sequence **BACKEND_DEPLOY → EXCHANGE_SYNC → PNL_RECOMPUTE** required before live ~€2000 validation. **0 blockers.** Ready for **`/release`**.

## Scope

**Inputs reviewed:** `handoffs/dev_to_qa.md` (Q0030 section), `sprints/quick/Q0030/qa-findings.md`, `docs/product/acceptance.md` BUG-0023 (BO/BP/BQ), `sprints/quick/Q0030/uat.md`, `sprints/quick/Q0030/uat.json`. No host `.env`/secret files read.

## Operator gates

| Gate | Status | Action | Notes |
|------|--------|--------|-------|
| **BACKEND_DEPLOY** | pending | Rebuild backend; apply migration `017_bug0023_exposure_eur.sql` | Required before live BO/BP/BQ oracles |
| **EXCHANGE_SYNC** | pending | Bitunix full/exchange sync after deploy | Wallet row ingest + linear positions refresh |
| **PNL_RECOMPUTE** | pending | Post-sync PnL recompute | Populates `exposure_eur`, wallet `market_value_eur`, baseline + `total_return_pct` |
| **AP1_SQL_PROBE** | pending (optional) | SQL probe on `exchange_holdings` `product_type` / `market_value_eur` | Architecture § BUG-0023 optional gate |

**Post-gate smoke:** `GET http://localhost:18080/api/v1/wealth` — expect `crypto.subtotal_eur` ~€2000 order of magnitude; linear `holdings_all[].value_eur` non-null; `pnl.total_return_pct` non-null when baseline exists.

## Live probe — pre-deploy baseline (2026-06-12)

| Probe | HTTP | Key fields | Interpretation |
|-------|------|------------|----------------|
| `GET /health` | 200 | OK | Stack reachable |
| `GET /api/v1/wealth` | 200 | See below | Pre-deploy baseline — bug symptoms still visible |

### Wealth API snapshot (pre-deploy)

| Field | Value | Expected post-gate |
|-------|-------|-------------------|
| `crypto.subtotal_eur` | **-0.0** | ~€2000 (wallet-priced) |
| `crypto.exchanges[0].id` | `bitunix` | — |
| `crypto.exchanges[0].subtotal_eur` | **-0.0** | Non-zero |
| `crypto.exchanges[0].holdings_count` | 11 | — |
| Linear `holdings_all[].value_eur` | **all null** (11 rows) | Non-null when `entryValue` present |
| `holdings_all[].unrealized_pnl_eur` | populated (e.g. SOL 177.92) | — |
| `pnl.unrealized_eur` | **376.83** | — |
| `pnl.total_return_pct` | **null** | Non-null after recompute + baseline |

Live baseline confirms operator gates not yet applied; consistent with QA V1 deferral and BUG-0021 pass-with-prerequisites precedent.

## Per-row verdict (acceptance BO / BP / BQ)

| Row | Verdict | Evidence |
|-----|---------|----------|
| **BO** | **pass_with_prerequisites** | Integration `bo_futures_wallet_priced_subtotal_nonzero` + wiremock/unit tests PASS; live API `crypto.subtotal_eur` / Bitunix card still €0 pre-deploy |
| **BP** | **pass_with_prerequisites** | Integration `bp_linear_exposure_eur_value_without_subtotal_merge` PASS; migration 017 + wealth mapping verified at QA; live all `value_eur` null pre-deploy |
| **BQ** | **pass_with_prerequisites** | Integration `bq_priced_wallet_baseline_total_return_pct` PASS; live `total_return_pct` null despite `unrealized_eur` 376.83 pre-deploy |
| **OIDC regression** | **pass** | `GET /api/v1/wealth` HTTP 200 on `:18080`; `/health` HTTP 200 |

## Automated checks (verify-work re-run)

| Check | Result |
|-------|--------|
| `cargo test --test bug0023_crypto_wealth_eur` | **4/4 PASS** |
| QA-stage `cargo test --lib` | **218/218 PASS** (qa-findings) |
| QA-stage `npm test` / `npm run build` | **9/9 PASS** / **PASS** (qa-findings) |

## UAT matrix summary

| Result | Count |
|--------|-------|
| pass | **1** (OIDC-1) |
| pass_with_prerequisites | **8** (BO-API, BO-UI, BO-SQL, BP-API, BP-UI, BP-SUBTOTAL, BQ-API, BQ-UI) |
| fail | **0** |

## Acceptance impact

| Row | Verify-work | Post-operator (release follow-up) |
|-----|-------------|-----------------------------------|
| **BO** | pass_with_prerequisites | Live ~€2000 subtotal after BACKEND_DEPLOY + EXCHANGE_SYNC |
| **BP** | pass_with_prerequisites | Value EUR column + API non-null after deploy + recompute |
| **BQ** | pass_with_prerequisites | Total return % non-null after PNL_RECOMPUTE |

## Next phase

**`/release`** — release notes; operator gate checklist; backlog BUG-0023 remains open until post-deploy smoke PASS.

`fresh_context_marker`: verify-work-20260612-bug0023-qa-fresh  
`runtime_proof_id`: runtime-proof-verify-work-20260612-bug0023-001  
`phase_boundary`: verify-work → release
