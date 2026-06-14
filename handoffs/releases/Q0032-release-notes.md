# Quick Release Notes — Q0032 / BUG-0026

**Quick task:** Q0032  
**Bug:** BUG-0026 — Forecast monthly Income card 0.00 while chart shows income bars  
**Date:** 2026-06-13  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0026 rows **BZ**, **CA**; live BZ/CA browser operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `npm test` (24/24); `npm run build` PASS; `sprints/quick/Q0032/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/quick/Q0032/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS-WITH-PREREQUISITES — `sprints/quick/Q0032/uat.json`, `sprints/quick/Q0032/uat.md`, `sprints/quick/Q0032/verify-work-findings.md`; 5 steps — 2 pass, 3 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260613-bug0026-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Forecast monthly summary card mismatch fix per **DEC-0089** (category filter scope unchanged). Frontend-only: cards resolve labeled reference month via `resolveForecastSummaryPoint` (skip partial zero-income head) and show shared subtitle above the card grid.

| Scope | Fix |
|-------|-----|
| **H1** | `frontend/src/pages/forecastSummaryMonth.ts` — `resolveForecastSummaryPoint`, `formatForecastSummarySubtitle` |
| **F1** | `frontend/src/pages/ForecastPage.tsx` — useMemo + subtitle above card grid; card values from resolved point |
| **T1** | `frontend/src/pages/forecastSummaryMonth.test.ts` — 7 vitest cases including partial-month trap (June 0 → July **3266.16**) |
| **G1** | `npm test` 24/24; `npm run build` PASS — frontend-only blast radius |
| **V1** | verify-work BZ/CA oracles — BZ-API live **3266.16** PASS; BZ/CA browser deferred **FRONTEND_DEPLOY** |

**Code proof:** forecastSummaryMonth 7/7; npm 24/24; npm build PASS; live API series[1] income **3266.16** (account **114**).

**Operator post-release:** Rebuild frontend only (**FRONTEND_DEPLOY** — no migration).

---

## Run

**Target service:** `flow-finance-ai` (frontend SPA only — no backend changes).

**Deploy (frontend rebuild):**

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
- `runtime_context_ref`: `docs/engineering/runbook.md` (§37 BUG-0026 hotfix)

**Operator gate — FRONTEND_DEPLOY (required before BZ/CA live UI smoke):**

Running container predates Q0032; browser reproduces Income **0.00** + no subtitle pre-deploy. Confirm tests pass before docker build:

```bash
cd frontend && npm test && npm run build
```

No migration. Backend unchanged — API already returns series[1] income **3266.16**.

---

## Connect

- `service_url`: `http://localhost:18080` (local) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 18080 (local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET http://localhost:18080/health` → JSON 200
- Forecast UI: `http://localhost:18080/forecast` (Monthly tab, account **114**)
- BZ API oracle: `GET /api/v1/forecast/monthly?account_id=114` → series[1] income **3266.16**

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | External PostgreSQL — forecast data for account **114** smoke |
| `AUTHENTIK_SECRET_KEY` | External compose profile build gate (set dummy for local external profile) |
| OIDC provider config | Omniflow OIDC-1 regression — Compose/env only |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(BZ)** | `/forecast` Monthly account **114** | Income card **3266.16** matches July chart bar — not **0.00** with unexplained chart bars |
| **(CA)** | Summary cards subtitle | **"Forecast for July 2026"** above four cards — not unlabeled `series[0]` |
| Regression | Category filter on `/forecast` | Card values unchanged per **DEC-0089** |
| Regression | OIDC-1 omniflow `/forecast` + monthly API | HTTP 200 after deploy |

**Automated (release):**

```bash
cd frontend && npm test && npm run build
```

**Live (operator):** UAT steps in `sprints/quick/Q0032/uat.json`.

---

## Credentials

- `DATABASE_URL` — external PostgreSQL (operator shell only — no inline secrets)
- OIDC provider config via Compose/env only

---

## Changes

| Area | Summary |
|------|---------|
| `frontend/src/pages/forecastSummaryMonth.ts` | H1 pure helper — GATE-MONTH-1 skip partial zero-income head; GATE-LABEL-1 subtitle |
| `frontend/src/pages/ForecastPage.tsx` | F1 wire resolved point + subtitle above card grid |
| `frontend/src/pages/forecastSummaryMonth.test.ts` | T1 7 vitest cases including account **114** partial-month trap |
| Runbook | §37 BUG-0026 operator smoke |

**Linked decisions:** DEC-0089 (forecast actuals-only + category filter scope)  
**Research fulfilled:** R-0098  
**Deferred:** BZ/CA live browser smoke until **FRONTEND_DEPLOY**

---

## Known Issues

- Running container serves pre-Q0032 frontend — Income card **0.00** + no subtitle until rebuild
- Live `:18080` confirms pre-deploy baseline per verify-work browser probes
- AI-mapped callout still keys off `series[0]?.ai_mapped` — pre-existing; optional P2 alignment

---

## Regression scope

- Plan delete selector (BUG-0022) unchanged
- Crypto wealth EUR (BUG-0023) unchanged
- CategoryFilter static import (BUG-0021) unchanged
- Backend forecast API contract unchanged

---

## Rollback

```bash
git revert <Q0032-code-commits>
docker compose up -d --build flow-finance-ai
```

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0032 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0026-q0032`

## Milestone

**BUG-0026 released** — forecast summary month selection + labeled subtitle; BZ/CA code oracles PASS; deploy operator-deferred per pass-with-prerequisites.
