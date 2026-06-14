# UAT — Q0034 (BUG-0025)

**Status:** POPULATED — verify-work complete  
**Verdict:** **PASS-WITH-PREREQUISITES** (2 pass / 6 pass-with-prerequisites / 0 fail)  
**Acceptance:** `docs/product/acceptance.md` — BUG-0025 rows **BW**, **BX**, **BY**  
**Sprint:** Q0034 (`/quick`)  
**Orchestrator:** `auto-20260613-bug0025`  
**Verified at:** 2026-06-14T17:55:00Z

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Result |
|-----|---------|-----------------------------------|--------|
| **BW** | B1, T1, G1, V1 | Multi-month Stromkosten bars after manual Full sync — not 2026-05 only | pass_with_prerequisites |
| **BX** | B1, D1, F1, G1, V1 | Backdated ingest or documented DEC-0002 limitation + remediation | pass_with_prerequisites |
| **BY** | B2, F1, G1, V1 | Sync now = Full Firefly; history/hero distinguish Firefly vs exchange runs | pass_with_prerequisites |

## Operator gates (post-release smoke)

| Gate | Status | Notes |
|------|--------|-------|
| BACKEND_REBUILD | pending | B1 manual lookback + B2 API field |
| FRONTEND_DEPLOY | pending | F1 Sync Status hero + callout |

## UAT steps

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| BW-API | BW | `GET /api/v1/categories/expense-series?category_id=146` multi-month after manual Sync now | pass_with_prerequisites | Live: only 2026-05 465.53; integration 3/3 PASS |
| BW-UI | BW | `/forecast` Category spending trend **Wohnen - Stromkosten** — bars per month | pass_with_prerequisites | Browser pre-deploy symptom; deferred deploy |
| BX-UI | BX | `/sync` DEC-0002 info callout + runbook link visible | pass_with_prerequisites | Callout absent pre-deploy; source PASS |
| BX-DOC | BX | `docs/engineering/runbook.md` backdated import + cursor reset section | **pass** | `#backdated-firefly-imports` + cursor-reset SQL |
| BY-API | BY | `GET /api/v1/sync/status` — `last_firefly_run` distinct from exchange-only `last_run` | pass_with_prerequisites | Field absent pre-deploy; B2 source PASS |
| BY-UI | BY | Hero **Last Firefly sync** + trigger badge; exchange secondary when newer | pass_with_prerequisites | Hero shows exchange timestamp pre-deploy |
| BY-HIST | BY | Sync history `trigger` column — manual vs scheduled_exchanges | **pass** | manual / scheduled / scheduled_exchanges distinguished |
| OIDC-1 | regression | `/sync`, `/forecast`, sync trigger smoke on OIDC profile | pass_with_prerequisites | HTTP 200; fix deferred deploy gates |

## Results summary

| Result | Count |
|--------|-------|
| pass | **2** |
| pass_with_prerequisites | **6** |
| fail | **0** |
| **Total** | **8** |

**Acceptance linkage:** BUG-0025 rows **BW**, **BX**, **BY** verified at verify-work stage with operator deploy gates pending. Automated evidence: `sprints/quick/Q0034/verify-work-findings.md`, `sprints/quick/Q0034/uat.json`.

**Next phase:** `/release`
