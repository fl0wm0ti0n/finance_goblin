# Quick Release Notes — Q0025 / BUG-0017

**Quick task:** Q0025  
**Bug:** BUG-0017 — Post-sync forecast recompute cluster  
**Date:** 2026-06-10  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0017 rows AY–BD; runtime operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --lib` (213/213), `cargo test --test forecast_integration` (3/3), `npm test -- --run` (9/9) @ 2026-06-10 release; `sprints/quick/Q0025/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/quick/Q0025/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0025/uat.json`, `sprints/quick/Q0025/uat.md`, `handoffs/verify_work_to_release.md`; 11 steps — 5 pass, 6 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260610-bug0017-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Post-sync forecast recompute cluster fixed per **DEC-0105** (audit CHECK migration) and **DEC-0106** (FK CASCADE + ml_enhanced-first retention order). ForecastPage loading guard per **BD** `isFetched` contract.

| Scope | Fix |
|-------|-----|
| **AY1** | `015_bug0017_ai_audit_forecast.sql` — `forecast_bucket_assignment` tool_name + extended `result_status` values |
| **BA1** | `015_bug0017_forecast_fk_cascade.sql` — `ON DELETE CASCADE` on `paired_baseline_id` |
| **BA2** | `repository.rs` — ml_enhanced-first retention delete order |
| **BD1** | `ForecastPage.tsx` — `showLoading`/`showEmpty` guard when meta pending |
| **T1** | `forecast_integration.rs` — paired retention prune without FK violation (3/3) |
| **V1** | verify-work sync/audit/planning/Forecast smoke — pass-with-prerequisites pending **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** |

**Code proof:** `cargo test --lib` 213/213; `forecast_integration` 3/3; frontend vitest 9/9.

**Operator post-release:** Rebuild `flow-finance-ai` with Q0025 migrations + repository + ForecastPage; run 9-step smoke checklist in `sprints/quick/Q0025/uat.json`.

---

## Run

**Target service:** `flow-finance-ai` (backend migrations + forecast retention + frontend guard).

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
- `runtime_context_ref`: `docs/engineering/runbook.md` (§30 BUG-0017 hotfix)

**Profile rule:** **`external`** for omniflow; local override via `docker-compose.override.yml`.

**Rebuild scope:** Recreate `flow-finance-ai` only — migrations apply on startup; frontend guard bundled in image.

**Operator gate — FULL_FIREFLY_SYNC (required before audit/meta/planning probes):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
# Wait for sync status success; confirm forecast recompute completes
```

---

## Connect

- `service_url`: `http://localhost:18080` (local) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 18080 (local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health` → JSON 200
- Forecast meta: `GET /api/v1/forecast/meta`
- Audit probe: `SELECT * FROM ai_tool_audit WHERE tool_name='forecast_bucket_assignment' LIMIT 5`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `FORECAST_ML_ENABLED` | ML-enhanced forecast gate (BB probe) |
| `AUTH_DEV_BYPASS` | Local API-only dev only — not omniflow |
| Traefik basic auth | Omniflow browser smoke — operator shell / password manager |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(AY)** | Sync logs during recompute | No `ai_tool_audit_tool_name_check` WARN for `forecast_bucket_assignment` |
| **(AZ)** | Audit rows after recompute | `low_confidence` (and extended statuses) persist without CHECK violation |
| **(BA)** | `GET /api/v1/forecast/meta` post-sync | Fresh `computation_id`, `stale=false`; no FK WARN in logs |
| **(BB)** | ML meta + month-bucket SQL | Honest `ml_skipped_reason` when gate fails; ML selectable when gate passes |
| **(BC)** | Planning Compare after recompute | **Plan stale** badge clears |
| **(BD)** | Forecast nav from Home | Loading skeleton during pending; no false **No forecast data yet** when meta has `computation_id` |
| Regression | OIDC-enabled deploy | Standard OIDC smoke per acceptance footnote |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test forecast_integration
cd frontend && npm test -- --run
```

**Live (operator post-deploy):** 9-step checklist in `sprints/quick/Q0025/uat.json` `operator_smoke_checklist` after **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**.

**Expected health signal:** `GET /health` → HTTP 200 JSON; `GET /api/v1/forecast/meta` → `stale=false` with `computation_id` after Full sync.

---

## Credentials

- Traefik basic auth — operator shell / password manager
- OIDC provider config via Compose/env only — no inline secrets in release artifacts
- `DATABASE_URL` — external PostgreSQL with TimescaleDB (migrations apply on startup)

---

## Changes

| Area | Summary |
|------|---------|
| `backend/migrations/015_bug0017_ai_audit_forecast.sql` | AY1 — DEC-0105 audit CHECK extension |
| `backend/migrations/015_bug0017_forecast_fk_cascade.sql` | BA1 — DEC-0106 FK CASCADE |
| `backend/src/forecast/repository.rs` | BA2 — ml_enhanced-first retention order |
| `frontend/src/pages/ForecastPage.tsx` | BD1 — `isFetched` loading/empty guard |
| `backend/tests/forecast_integration.rs` | T1 — paired retention integration test |
| Runbook | §30 BUG-0017 operator smoke |

**Linked decisions:** DEC-0105 (audit CHECK), DEC-0106 (FK CASCADE + retention order)  
**Research fulfilled:** R-0087  
**Deferred:** V1 omniflow/:18080 runtime smoke (operator **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**)

---

## Known Issues

- V1 runtime probes pass-with-prerequisites until **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**
- `:18080` pre-Q0025 deploy showed 0 `forecast_bucket_assignment` audit rows and `plan_stale=true` — expected until rebuild
- Omniflow browser/OIDC smoke deferred — auth barrier per BUG-0013/0014/0015/0016 precedent
- BB month-bucket SQL probe deferred to operator per R-0087

---

## Regression scope

- Sync pipeline success semantics unchanged — recompute errors logged but sync still succeeds
- True `insufficient_history` ML gate preserved — must not mask with stale baseline
- SPA deep-link fallback (BUG-0016 / DEC-0104) unchanged
- OIDC flow unchanged

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0025 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0017-q0025`

## Milestone

**BUG-0017 released** — post-sync forecast recompute audit CHECK + FK retention + ForecastPage loading guard; operator sync/audit/planning/Forecast smoke deferred per pass-with-prerequisites.
