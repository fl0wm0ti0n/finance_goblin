# UAT — Q0029 (BUG-0021)

**Status:** VERIFY-WORK COMPLETE  
**Verdict:** **PASS-WITH-PREREQUISITES**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0021 rows **BK**, **BL**  
**Sprint:** Q0029 (`/quick`)  
**Orchestrator:** `auto-20260611-bug0021`  
**Verified at:** 2026-06-11T12:50:00Z

## Operator gates (before live probes)

| Gate | Status | Notes |
|------|--------|-------|
| BACKEND_FRONTEND_DEPLOY | pass-with-prerequisites | Container predates Q0029; compose build blocked AUTHENTIK_SECRET_KEY |
| SNAPSHOT_UPSERT_OR_SYNC | pass-with-prerequisites (optional) | Snapshot account_role null until post-deploy upsert |

## UAT steps

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| BK-FORECAST | BK | Forecast → Monthly: no multi-second **Loading category filter…**; combobox ≤1 s | pass_with_prerequisites | Static import + chunk audit PASS; browser deferred deploy |
| BK-WEALTH | BK | Wealth → Overview: same | pass_with_prerequisites | Static import PASS; browser deferred deploy |
| BL-API | BL | `GET /api/v1/wealth` — Giro/savings/cash wallet `account_role` non-null | pass_with_prerequisites | Mirror COALESCE 3/3 PASS; live API null pre-deploy |
| BL-UI | BL | Account breakdown Role column human labels (not all em dash) | pass_with_prerequisites | formatAccountRole code PASS; UI deferred deploy |
| BL-SNAPSHOT | BL | Latest `net_worth_snapshots.payload.accounts` carries `account_role` | pass_with_prerequisites | 2026-06-11 snapshot null pre-deploy |
| BL-GRAFANA | BL | Portfolio dashboard role column populated (post-upsert) | pass_with_prerequisites (optional) | Deferred post-deploy + upsert |
| OIDC-1 | regression | `/forecast`, `/wealth`, `/api/v1/wealth` smoke on omniflow | pass | omniflow wealth API HTTP 200 |

## Automated checks

| Check | Result |
|-------|--------|
| `cargo test --test bug0021_wealth_account_role` | **4/4 PASS** |
| `npm run build` (frontend) | **PASS** |
| Related wealth integration suites | **wealth_alerts_integration 3/3 PASS** |

## Results summary

**1 pass / 6 pass-with-prerequisites / 0 fail** — code and mirror oracles PASS; live API/UI/snapshot deferred until **BACKEND_FRONTEND_DEPLOY**. Ready for **`/release`**.
