# QA Report — Q0029 / BUG-0021

**From:** QA (`/qa`)  
**To:** Verify-work (`/verify-work`)  
**Date:** 2026-06-11  
**Bug:** BUG-0021  
**Sprint:** Q0029 (`/quick`)  
**Orchestrator:** `auto-20260611-bug0021`  
**Verdict:** **PASS**

## Summary

Independent QA verified DEC-0110 (static CategoryFilter on BK surfaces) and DEC-0111 (wealth `account_role` COALESCE SQL + `formatAccountRole` label map). Static review, automated gates, and read-only mirror probes align with the accepted contract. Live `:18080` API still returns null roles on the pre-deploy backend — expected at qa stage; BK/BL runtime oracles deferred to verify-work after operator deploy.

## Independent test re-run (QA)

| Suite | Result |
|-------|--------|
| `cargo test --lib` | **213/213 PASS** (1.51s) |
| `cargo test --test bug0021_wealth_account_role` | **4/4 PASS** (exit 0; integration seed skipped — migration 015 checksum drift) |
| `cargo test load_asset_accounts_includes_negative_balances --lib` | **1/1 PASS** |
| `npm test` | **9/9 PASS** |
| `npm run build` | **PASS** |

## Static review

| Area | Result |
|------|--------|
| EA1 ForecastPage | **PASS** — static `CategoryFilter` import; Suspense removed on Monthly tab; `hasForecast` removed; `CategoryTrendChart` lazy+Suspense unchanged |
| EA2 WealthPage | **PASS** — static import; Suspense removed on Overview card; Role column uses `formatAccountRole` |
| EA3 PlanningPage | **PASS** — static import parity (P2) |
| EB1 SQL | **PASS** — `COALESCE(attributes, root)` in `load_asset_accounts` + test SQL constant |
| EB2 label map | **PASS** — matches DEC-0111 table (`defaultAsset`→Checking, etc.); unknown enum → raw; null → em dash |
| Bundle (DEC-0110) | **PASS** — no separate CategoryFilter lazy chunk; `CategoryTrendChart` chunk remains |
| Blast radius | **PASS** — 6 files: 4 modified + 2 new (`accountRole.ts`, `bug0021_wealth_account_role.rs`) |

## Read-only runtime probes (pre-deploy baseline)

| Probe | Result |
|-------|--------|
| Mirror COALESCE SQL (3 asset accounts) | **PASS** — `effective_role` populated from `attributes.account_role` |
| `GET :18080/api/v1/wealth` | **EXPECTED pre-deploy** — `account_role: null` on Giro/savings/cash wallet |
| `_sqlx_migrations` max version | **15** (016 not applied — unrelated BUG-0020; integration migrate blocked by 015 checksum) |

No data modified (QA seed row deleted); no `.env`/secret files read.

## Tasks verified at qa

| ID | Status |
|----|--------|
| EA1 | **PASS** |
| EA2 | **PASS** |
| EB1 | **PASS** |
| EB2 | **PASS** |
| EA3 | **PASS** (P2 parity) |
| T1 | **PASS** (with migrate-env caveat) |
| G1 | **PASS** |
| V1 | **DEFERRED** — verify-work |

## Acceptance row status (qa-stage)

| Row | Status |
|-----|--------|
| **BK** | **PASS** at qa (code + build chunk audit); live browser ≤1 s deferred V1 |
| **BL** | **PASS** at qa (SQL + label map + mirror probe); live API/UI/snapshot deferred V1 |

## Blocking findings

**0** — hand off to `/verify-work`.

`fresh_context_marker`: qa-20260611-bug0021-qa-fresh  
`runtime_proof_id`: runtime-proof-qa-20260611-bug0021-001  
`phase_boundary`: qa → verify-work
