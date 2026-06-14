# Verify-work Findings — Quick Q0034 / BUG-0025

**Work item:** BUG-0025 (defect)  
**Quick task:** Q0034  
**Phase:** `/verify-work`  
**Date:** 2026-06-14  
**Orchestrator:** `auto-20260613-bug0025`  
**Decisions:** DEC-0002 (extends; GATE-DEC-1 no new DEC)  
**QA agent:** fresh subagent (`verify-work-20260614-bug0025-qa-fresh`)

## Verdict

**PASS-WITH-PREREQUISITES** — Independent re-run confirms BW/BX/BY implementation under frozen gates **GATE-OVERLAP-1** / **GATE-SYNC-UX-1** / **GATE-REMED-1** / **GATE-TEST-1** / **GATE-DEC-1**: manual `trigger=manual` uses **365-day** lookback; scheduled path unchanged; `last_firefly_run` API + Sync Status hero in source; DEC-0002 callout + runbook remediation documented. Automated gates **cargo lib 221/221**, **bug0025 integration 3/3**, **npm 31/31**, **build PASS**. Live localhost reproduces pre-deploy symptom (Stromkosten expense-series **2026-05 only**; hero shows exchange `scheduled_exchanges` timestamp; `last_firefly_run` absent from API). **0 blockers.** Ready for **`/release`**.

## Scope

**Inputs reviewed:** `handoffs/dev_to_qa.md` (Q0034 top section), `sprints/quick/Q0034/qa-findings.md`, `docs/product/acceptance.md` BUG-0025 (BW/BX/BY), `sprints/quick/Q0034/uat.md`, `backend/src/firefly/mod.rs`, `backend/src/sync/mod.rs`, `backend/tests/bug0025_sync_transaction_window.rs`, `frontend/src/pages/SyncStatusPage.tsx`, `docs/engineering/runbook.md` § Backdated Firefly imports. No host `.env`/secret files read.

## Operator gates

| Gate | Status | Action | Notes |
|------|--------|--------|-------|
| **BACKEND_REBUILD** | pending | Rebuild/restart backend — B1 + B2 live | Required before `last_firefly_run` API + manual 365d ingest smoke |
| **FRONTEND_DEPLOY** | pending | Rebuild frontend — F1 Sync Status UX live | Required before BX callout + BY hero/trigger badge smoke |

**Post-gate smoke:** `/sync` — hero **Last Firefly sync** with trigger badge; exchange secondary when newer exchange-only run; DEC-0002 callout visible. Manual **Sync now** → `manual` in history; re-check `GET /api/v1/categories/expense-series?category_id=146` multi-month Stromkosten bars after manual Full sync.

## Live probe — pre-deploy baseline (2026-06-14)

| Probe | HTTP / observation | Key fields | Interpretation |
|-------|-------------------|------------|----------------|
| `GET /health` | 200 | OK | Stack reachable |
| `GET /api/v1/sync/status` | 200 | `last_run.trigger=scheduled_exchanges`; no `last_firefly_run` | **BY-API** pass_with_prerequisites — B2 not deployed |
| `GET /api/v1/categories/expense-series?category_id=146` | 200 | Only **2026-05** outflow **465.53** (4 tx) | **BW-API** pass_with_prerequisites — symptom repro; manual 365d fix not live |
| `GET /api/v1/sync/runs` | 200 | `manual`, `scheduled`, `scheduled_exchanges` triggers | **BY-HIST** PASS |
| Browser `/sync` | 200 | Hero **Last sync:** exchange timestamp; no DEC-0002 callout | Pre-deploy F1/B2 bundle |
| Browser `/forecast` Monthly | 200 | Category spending trend filter present | **BW-UI** deferred post-deploy + manual sync |

## Per-row verdict (acceptance BW / BX / BY)

| Row | Verdict | Evidence |
|-----|---------|----------|
| **BW** | **pass_with_prerequisites** | Integration 3/3 manual 365d start + ingest PASS; live API only 2026-05 Stromkosten; deferred BACKEND_REBUILD + manual Sync now |
| **BX** | **pass_with_prerequisites** | Runbook `#backdated-firefly-imports` + cursor-reset SQL PASS; callout absent pre-deploy — deferred FRONTEND_DEPLOY |
| **BY** | **pass_with_prerequisites** | History trigger column PASS; hero/API pre-deploy show exchange-only last_run — deferred BACKEND_REBUILD + FRONTEND_DEPLOY |

## Automated checks (verify-work re-run)

| Check | Result |
|-------|--------|
| `cargo test --lib` | **221/221 PASS** (1.99s) |
| `cargo test --test bug0025_sync_transaction_window` | **3/3 PASS** (0.00s) |
| `npm test` | **31/31 PASS** (2.26s) |
| `npm run build` | **PASS** — tsc + vite build (13.66s) |

## UAT step matrix

| Step | Row | Result | Evidence |
|------|-----|--------|----------|
| BW-API | BW | pass_with_prerequisites | Live API: only 2026-05 outflow 465.53; integration manual 365d PASS; deferred BACKEND_REBUILD |
| BW-UI | BW | pass_with_prerequisites | Browser /forecast Monthly; pre-deploy symptom; deferred deploy + manual Sync now |
| BX-UI | BX | pass_with_prerequisites | Browser /sync: DEC-0002 callout absent pre-deploy; source F1 PASS |
| BX-DOC | BX | **pass** | runbook.md `#backdated-firefly-imports` + cursor-reset SQL documented |
| BY-API | BY | pass_with_prerequisites | sync/status missing `last_firefly_run`; last_run=scheduled_exchanges |
| BY-UI | BY | pass_with_prerequisites | Hero "Last sync:" shows exchange timestamp pre-deploy |
| BY-HIST | BY | **pass** | sync/runs: manual, scheduled, scheduled_exchanges triggers distinguished |
| OIDC-1 | regression | pass_with_prerequisites | /sync + /forecast HTTP 200; BW/BX/BY fix not live until deploy gates |

## UAT matrix summary

| Result | Count |
|--------|-------|
| pass | **2** |
| pass_with_prerequisites | **6** |
| fail | **0** |
| pending | **0** |

## Runtime browser evidence

| Probe | navigation_url | reason_code | Ref |
|-------|----------------|-------------|-----|
| BY-UI-browser | http://localhost:18080/sync | UAT_BROWSER_PROBE_FAILED (expected pre-deploy) | `sprints/quick/Q0034/evidence/browser/bw-by-sync-probe-summary.txt` |
| BX-UI-browser | http://localhost:18080/sync | UAT_BROWSER_PROBE_FAILED (expected pre-deploy) | same + `bw-by-sync-probe-01.png` |
| BW-UI-browser | http://localhost:18080/forecast | UAT_BROWSER_PROBE_FAILED (expected pre-deploy) | same |

## Acceptance impact

| Row | Verify-work | Post-operator (release follow-up) |
|-----|-------------|-----------------------------------|
| **BW** | pass_with_prerequisites | Multi-month Stromkosten bars after BACKEND_REBUILD + manual Sync now |
| **BX** | pass_with_prerequisites | DEC-0002 callout visible on `/sync` after FRONTEND_DEPLOY |
| **BY** | pass_with_prerequisites | `last_firefly_run` hero + exchange secondary after BACKEND_REBUILD + FRONTEND_DEPLOY |

## Next phase

**`/release`** — release notes; operator gate checklist; backlog BUG-0025 remains open until post-deploy smoke PASS.

`fresh_context_marker`: verify-work-20260614-bug0025-qa-fresh  
`runtime_proof_id`: runtime-proof-verify-work-20260614-bug0025-001  
`phase_boundary`: verify-work → release
