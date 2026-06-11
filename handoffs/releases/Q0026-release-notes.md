# Quick Release Notes — Q0026 / BUG-0018

**Quick task:** Q0026  
**Bug:** BUG-0018 — Alert evaluation SQL failure (balance ambiguous)  
**Date:** 2026-06-10  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0018 rows BE–BF; runtime operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --lib` (213/213), `cargo test --test wealth_alerts_integration` (3/3), `npm test -- --run` (9/9) @ 2026-06-10 release; `sprints/quick/Q0026/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/quick/Q0026/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0026/uat.json`, `sprints/quick/Q0026/uat.md`, `handoffs/verify_work_to_release.md`; 7 steps — 2 pass, 5 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260610-bug0018-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Post-sync wealth alert evaluation fixed per **DEC-0107** — qualify `fbd.balance` and `fbd.ts` in `evaluate_scarcity` daily aggregate query. Restores scarcity alert pipeline without PostgreSQL **42702**; downstream **BF** alerts API and header bell unblocked.

| Scope | Fix |
|-------|-----|
| **BE1** | `backend/src/alerts/evaluate.rs` — DEC-0107 `fbd.balance` + `fbd.ts` in SELECT, WHERE, GROUP BY |
| **T1** | `backend/tests/wealth_alerts_integration.rs` — scarcity post-sync regression gate (3/3) |
| **V1** | verify-work sync + alerts smoke — pass-with-prerequisites pending **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** |

**Code proof:** `cargo test --lib` 213/213; `wealth_alerts_integration` 3/3; frontend vitest 9/9.

**Operator post-release:** Rebuild `flow-finance-ai` with Q0026 BE1 alert SQL fix; run 7-step smoke checklist in `sprints/quick/Q0026/uat.json`.

---

## Run

**Target service:** `flow-finance-ai` (backend alert evaluation SQL fix only — no migration).

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before runtime probes):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

- `start_command`: docker compose commands above
- `runtime_mode`: local (`:18080`) and remote (omniflow external US-0010)
- `runtime_context_ref`: `docs/engineering/runbook.md` (§31 BUG-0018 hotfix)

**Profile rule:** **`external`** for omniflow; local override via `docker-compose.override.yml`.

**Rebuild scope:** Recreate `flow-finance-ai` only — runtime SQL string fix in `evaluate_scarcity`.

**Operator gate — FULL_FIREFLY_SYNC (required before alerts API / header bell probes):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
# Wait for sync status success; confirm alert evaluation phase completes without 42702
```

---

## Connect

- `service_url`: `http://localhost:18080` (local) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 18080 (local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health` → JSON 200
- Alerts API: `GET /api/v1/alerts?status=active`
- Subscription alerts: `GET /api/v1/subscriptions/alerts`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `AUTH_DEV_BYPASS` | Local API-only dev only — not omniflow |
| Traefik basic auth | Omniflow browser smoke — operator shell / password manager |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(BE)** | Sync logs during alert eval phase | No `alert evaluation failed` / PostgreSQL 42702 |
| **(BF)** | `GET /api/v1/alerts?status=active` | Rows when household scarcity rule matches funded fixture |
| **(BF)** | Header Alerts bell | Non-empty active preview when rules match |
| **(BF)** | `GET /api/v1/subscriptions/alerts` | Dedup regression per BUG-0008 / DEC-0071 |
| Regression | OIDC-enabled deploy | Standard OIDC smoke per acceptance footnote |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test wealth_alerts_integration
cd frontend && npm test -- --run
```

**Live (operator post-deploy):** 7-step checklist in `sprints/quick/Q0026/uat.json` `operator_smoke_checklist` after **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**.

**Expected health signal:** `GET /health` → HTTP 200 JSON; post-sync logs free of 42702; alerts API returns active rows when scarcity conditions met.

---

## Credentials

- Traefik basic auth — operator shell / password manager
- OIDC provider config via Compose/env only — no inline secrets in release artifacts
- `DATABASE_URL` — external PostgreSQL with TimescaleDB (unchanged — no new migration)

---

## Changes

| Area | Summary |
|------|---------|
| `backend/src/alerts/evaluate.rs` | BE1 — DEC-0107 qualify `fbd.balance` + `fbd.ts` in `evaluate_scarcity` |
| `backend/tests/wealth_alerts_integration.rs` | T1 — scarcity post-sync regression gate |
| Runbook | §31 BUG-0018 operator smoke |

**Linked decisions:** DEC-0107 (scarcity SQL qualification)  
**Research fulfilled:** R-0088  
**Deferred:** V1 omniflow/:18080 runtime smoke (operator **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**)

---

## Known Issues

- V1 runtime probes pass-with-prerequisites until **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**
- `:18080` pre-Q0026 deploy may still log 42702 — expected until rebuild
- Omniflow browser/OIDC smoke deferred — auth barrier per BUG-0013/0014/0015/0016/0017 precedent
- New scarcity alerts may appear after fix when rules match — expected, not regression

---

## Regression scope

- Sync warn-only semantics unchanged (R-0024) — alert eval errors logged, sync still succeeds
- Subscription alert dedup (BUG-0008 / DEC-0071) unchanged — separate sync phase
- Sibling evaluators (`evaluate_budget_drift`, `evaluate_plan_viability`) unchanged
- SPA deep-link fallback (BUG-0016 / DEC-0104) unchanged
- OIDC flow unchanged

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0026 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0018-q0026`

## Milestone

**BUG-0018 released** — post-sync alert evaluation SQL qualification; operator sync/alerts smoke deferred per pass-with-prerequisites.
