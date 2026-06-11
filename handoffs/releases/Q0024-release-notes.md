# Quick Release Notes — Q0024 / BUG-0016

**Quick task:** Q0024  
**Bug:** BUG-0016 — SPA deep links return HTTP 404  
**Date:** 2026-06-09  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0016 row AX; runtime operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --lib` (213/213), `cargo test --test spa_fallback_integration` (5/5), `npm test -- --run` (9/9) @ 2026-06-09 release; `sprints/quick/Q0024/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/quick/Q0024/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0024/uat.json`, `sprints/quick/Q0024/uat.md`, `handoffs/verify_work_to_release.md`; 8 steps — 3 pass, 5 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260609-bug0016-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

SPA deep-link HTTP 404 fixed per **DEC-0104** (`ServeDir::fallback(ServeFile::new(index.html))` in `build_router`) with **DEC-0057** route order preserved (health → Grafana → API → SPA). Integration tests prove HTTP 200 HTML shell for client routes and protected prefixes for `/health`, `/api/v1/*`, `/analytics/grafana/*`, `/assets/*`.

| Scope | Fix |
|-------|-----|
| **AX1** | `attach_spa_fallback` in `backend/src/lib.rs` — prod `/app/static` + dev `frontend/dist` branches |
| **AX2** | `backend/tests/spa_fallback_integration.rs` — 5/5 deep links + protected prefix regression |
| **V1** | verify-work curl/browser/OIDC smoke — pass-with-prerequisites pending **BACKEND_FRONTEND_DEPLOY** |

**Code proof:** `cargo test --lib` 213/213; `spa_fallback_integration` 5/5; frontend vitest 9/9.

**Operator post-release:** Rebuild `flow-finance-ai` with Q0024 AX1; run 7-step smoke checklist in `sprints/quick/Q0024/uat.json`.

---

## Run

**Target service:** `flow-finance-ai` (backend serves built SPA static + fallback).

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
- `runtime_context_ref`: `docs/engineering/runbook.md` (§29 BUG-0016 hotfix)

**Profile rule:** **`external`** for omniflow; local override via `docker-compose.override.yml`.

**Rebuild scope:** Recreate `flow-finance-ai` only — SPA fallback is backend-served static routing.

---

## Connect

- `service_url`: `http://localhost:18080` (local) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 18080 (local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health` → JSON 200
- Deep-link probes: `/forecast`, `/subscriptions`, `/planning`, `/sync`, `/analytics/cashflow`, `/callback`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `AUTH_DEV_BYPASS` | Local API-only dev only — not omniflow |
| Traefik basic auth | Omniflow browser smoke — operator shell / password manager |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(AX)** | curl deep links on `:18080` | HTTP 200 + `text/html` SPA shell (not 404 empty body) |
| **(AX)** | Protected prefixes | `/health` JSON; `/api/v1/*` JSON; `/analytics/grafana/*` proxy; `/assets/*` static |
| **(AX)** | Hard-refresh + bookmarks on omniflow | Correct React page renders (not blank 404) |
| **OIDC-1** | `/callback?code=…&state=…` | SPA shell only; React OidcCallback handles exchange |
| Regression | DEC-0057 Grafana proxy | `/analytics/grafana/*` not replaced by HTML fallback |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test spa_fallback_integration
cd frontend && npm test -- --run
```

**Live (operator post-deploy):** 7-step checklist in `sprints/quick/Q0024/uat.json` `operator_smoke_checklist` after **BACKEND_FRONTEND_DEPLOY**.

**Expected health signal:** `GET /health` → HTTP 200 JSON; `curl -sS -o /dev/null -w '%{http_code}' http://localhost:18080/forecast` → **200** after deploy.

---

## Credentials

- Traefik basic auth — operator shell / password manager
- OIDC provider config via Compose/env only — no inline secrets in release artifacts

---

## Changes

| Area | Summary |
|------|---------|
| `backend/src/lib.rs` | AX1 — `attach_spa_fallback` per DEC-0104; DEC-0057 merge order |
| `backend/tests/spa_fallback_integration.rs` | AX2 — deep links 200 HTML + protected prefix regression |
| `backend/tests/fixtures/spa/` | Minimal SPA fixture for integration tests |
| Runbook | §29 BUG-0016 operator smoke |

**Linked decisions:** DEC-0104 (SPA fallback), DEC-0057 (route order)  
**Research fulfilled:** R-0086  
**Deferred:** V1 omniflow/:18080 runtime smoke (operator **BACKEND_FRONTEND_DEPLOY**)

---

## Known Issues

- V1 runtime probes pass-with-prerequisites until **BACKEND_FRONTEND_DEPLOY**
- `:18080` deep links returned HTTP 404 on pre-Q0024 deploy at verify-work — expected until rebuild
- Omniflow root `/forecast` returned 401 (auth barrier) at verify-work — browser smoke deferred per BUG-0013/0014/0015 precedent

---

## Regression scope

- DEC-0057 Grafana embed proxy unchanged
- No Traefik label changes; no backend `/callback` redirect
- OIDC flow unchanged — `/callback` receives SPA shell only

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0024 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0016-q0024`

## Milestone

**BUG-0016 released** — SPA deep links return HTTP 200 HTML shell via Axum `index.html` fallback; operator curl/browser/OIDC smoke deferred per pass-with-prerequisites.
