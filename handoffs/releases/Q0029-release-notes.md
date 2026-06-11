# Quick Release Notes — Q0029 / BUG-0021

**Quick task:** Q0029  
**Bug:** BUG-0021 — Frontend UX polish (category filter delay, wealth role column)  
**Date:** 2026-06-11  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0021 rows **BK**, **BL**; browser/API/snapshot deploy operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --test bug0021_wealth_account_role` (4/4); `cargo test --lib` (213/213); `npm test` (9/9); `sprints/quick/Q0029/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/quick/Q0029/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS-WITH-PREREQUISITES — `sprints/quick/Q0029/uat.json`, `sprints/quick/Q0029/uat.md`, `handoffs/verify_work_to_release.md`; 7 steps — 1 pass, 6 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260611-bug0021-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Frontend UX polish per **DEC-0110** (static CategoryFilter on BK surfaces) and **DEC-0111** (wealth `account_role` COALESCE SQL path + `formatAccountRole` label map). Restores snappy category filter load and Firefly account-type Role column.

| Scope | Fix |
|-------|-----|
| **EA1** | `frontend/src/pages/ForecastPage.tsx` — static `CategoryFilter` import; Monthly tab no Suspense wrapper |
| **EA2** | `frontend/src/pages/WealthPage.tsx` — static `CategoryFilter` import; Overview no Suspense wrapper |
| **EA3** | `frontend/src/pages/PlanningPage.tsx` — static CategoryFilter P2 parity |
| **EB1** | `backend/src/wealth/repository.rs` — `COALESCE(attributes, root)` `account_role` SQL |
| **EB2** | `frontend/src/lib/accountRole.ts` — five canonical Role labels |
| **T1/G1** | `bug0021_wealth_account_role` 4/4; lib 213/213; npm 9/9 |
| **V1** | verify-work BK/BL oracles — pass; live API/UI/snapshot deferred deploy |

**Code proof:** bug0021 4/4; cargo lib 213/213; npm 9/9; mirror COALESCE 3/3 (`defaultAsset`, `savingAsset`, `cashWalletAsset`); static import + chunk audit PASS.

**Operator post-release:** Rebuild `flow-finance-ai` (**BACKEND_FRONTEND_DEPLOY**); optional Full sync for BL snapshot/Grafana oracle (**SNAPSHOT_UPSERT_OR_SYNC**).

---

## Run

**Target service:** `flow-finance-ai` (backend wealth SQL + frontend static imports).

**Deploy (backend + frontend):**

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
- `runtime_context_ref`: `docs/engineering/runbook.md` (§34 BUG-0021 hotfix)

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before BK browser + BL API/UI oracles):**

Running container `finance_goblin-flow-finance-ai-1` predates Q0029 (created 2026-06-09). Compose build may require `AUTHENTIK_SECRET_KEY` on external profile. Confirm `npm run build` passes before docker build.

**Operator gate — SNAPSHOT_UPSERT_OR_SYNC (optional for BL-SNAPSHOT / BL-GRAFANA):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
# Or wait for daily net_worth_snapshots upsert after deploy
```

---

## Connect

- `service_url`: `http://localhost:18080` (local) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 18080 (local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET http://localhost:18080/health` → JSON 200
- Forecast UI: `http://localhost:18080/forecast` (Monthly tab)
- Wealth UI: `http://localhost:18080/wealth` (Overview tab)
- BL API oracle: `GET /api/v1/wealth`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | External PostgreSQL — mirror COALESCE probe target |
| `AUTHENTIK_SECRET_KEY` | External compose profile build gate (set dummy for local external profile) |
| OIDC provider config | Omniflow OIDC-1 regression — Compose/env only |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(BK)** | Forecast → Monthly — CategoryFilter | Combobox interactive ≤1s; no multi-second **Loading category filter…** |
| **(BK)** | Wealth → Overview — CategoryFilter | Same snappy load; `CategoryTrendChart` remains lazy chunk only |
| **(BL)** | `GET /api/v1/wealth` | Asset accounts return non-null `account_role` (e.g. `defaultAsset`, `savingAsset`, `cashWalletAsset`) |
| **(BL)** | Wealth Account breakdown Role column | Human labels: Checking, Cash wallet, Savings, Shared, Credit card |
| **(BL)** | `net_worth_snapshots.payload.accounts` | `account_role` populated post-upsert (optional) |
| Regression | OIDC-1 omniflow `/api/v1/wealth` | HTTP 200 |

**Automated (release):**

```bash
cd backend && cargo test --test bug0021_wealth_account_role
cd backend && cargo test --lib
cd frontend && npm test && npm run build
```

**Live (operator):** UAT steps in `sprints/quick/Q0029/uat.json`.

---

## Credentials

- `DATABASE_URL` — external PostgreSQL (operator shell only — no inline secrets)
- OIDC provider config via Compose/env only

---

## Changes

| Area | Summary |
|------|---------|
| `frontend/src/pages/ForecastPage.tsx` | EA1 static CategoryFilter |
| `frontend/src/pages/WealthPage.tsx` | EA2 static CategoryFilter + EB2 Role column |
| `frontend/src/pages/PlanningPage.tsx` | EA3 P2 parity |
| `frontend/src/lib/accountRole.ts` | EB2 label map (new) |
| `backend/src/wealth/repository.rs` | EB1 COALESCE account_role SQL |
| `backend/tests/bug0021_wealth_account_role.rs` | T1/G1 regression (4/4) |
| Runbook | §34 BUG-0021 operator smoke |

**Linked decisions:** DEC-0110 (static CategoryFilter BK surfaces); DEC-0111 (account_role path + labels)  
**Research fulfilled:** R-0091  
**Deferred:** BK browser timing + BL API/UI/snapshot until **BACKEND_FRONTEND_DEPLOY**

---

## Known Issues

- Running container serves pre-Q0029 static/binary — EA/EB changes not live until rebuild
- `docker compose build` on external profile blocked without `AUTHENTIK_SECRET_KEY`
- Live `:18080` and omniflow API return `account_role: null` on all rows pre-deploy
- Snapshot payload `account_role` null until post-deploy upsert

---

## Regression scope

- `CategoryTrendChart` lazy + Suspense unchanged on Forecast/Wealth/Planning
- Subscription list (BUG-0020) unchanged
- Grafana provisioning (BUG-0019) unchanged
- Alert evaluation (BUG-0018) unchanged

---

## Rollback

```bash
git revert <Q0029-code-commits>
docker compose up -d --build flow-finance-ai
```

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0029 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0021-q0029`

## Milestone

**BUG-0021 released** — DEC-0110 static CategoryFilter + DEC-0111 account_role COALESCE/labels; BK/BL code oracles PASS; deploy operator-deferred per pass-with-prerequisites.
