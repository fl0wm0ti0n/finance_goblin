# Quick Release Notes — Q0008 / BUG-0002

**Quick task:** Q0008  
**Bug:** BUG-0002 — Omniflow production integration (Firefly sync + risk-score + exchange settings)  
**Date:** 2026-06-05  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0002)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (103/103), `cd frontend && npm test` (2/2), `npm run build` @ 2026-06-05 (release re-run)
2. **QA completion gate:** PASS — `sprints/quick/Q0008/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0008/uat.md`, `handoffs/verify_work_to_release.md`; omniflow rows C/D/E live curl PASS (OIDC regression deferred)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

---

## Run

**Target service (external profile):** `flow-finance-ai` — must rebuild/recreate for C2 + D1 + E1 + E2 (+ frontend Planning types).

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Full stack (if other services stale):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build
```

**Operator prerequisite (C1):** non-empty `FIREFLY_PERSONAL_ACCESS_TOKEN` in operator `.env` (names only); whitespace-only treated as unset.

- `runtime_mode`: omniflow external (US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§ Omniflow §10 BUG-0002)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- **C:** Firefly sync via in-network `http://firefly:8080` + PAT
- **D:** `GET /api/v1/plans/risk-score` — always **200** (`ok` or `no_score`)
- **E:** Settings `/api/v1/settings` — Bitunix effective enabled when credentials present

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `FIREFLY_BASE_URL` | `http://firefly:8080` |
| `FIREFLY_PERSONAL_ACCESS_TOKEN` | Non-empty Firefly PAT |
| `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` | Optional — E row smoke |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **C** | `GET /api/v1/sync/status` | 200; `last_run.status: success`; no 401 |
| **C** | `GET /api/v1/sync/entities` | 200; non-zero transaction count |
| **C** | `GET /api/v1/sync/runs` | 200 (no blocking 404 on sync routes) |
| **D** | `GET /api/v1/plans/risk-score` | HTTP **200**; `no_score` or `ok` JSON |
| **E** | `GET /api/v1/settings` | Bitunix `enabled: true`, `configured: true` when env set |
| **E** | Binance row | `enabled: false` when unset (E2 default) |
| **E** (optional) | `POST /api/v1/exchanges/bitunix/test` | 200 connection payload |
| Stack | `GET /health` | 200 |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd frontend && npm test
cd frontend && npm run build
```

**Boundaries preserved:** no Traefik routing changes; no JWT/proxy stack changes; OIDC regression deferred to operator browser smoke.

---

## Deliverables

| Slice | Change |
|-------|--------|
| **C2** | Empty PAT guard; `pat_configured()`; sync preflight `firefly_personal_access_token_missing` |
| **C1** | Runbook + `.env.example` PAT verification notes |
| **D1** | `plans/risk-score` always 200 empty-state; PlanningPage types |
| **E1** | `effective_enabled()` in settings_view + startup mirror |
| **E2** | `exchanges.binance.enabled=false` in `default.toml` |

## Evidence refs

- `sprints/quick/Q0008/summary.md`, `qa-findings.md`, `uat.md`, `verify-work-findings.md`
- `handoffs/verify_work_to_release.md`, `dev_to_qa.md`
- `handoffs/tl_to_dev.md` (architecture-20260604-bug0002)

## Known issues (non-blocking)

- OIDC-enabled deploy regression: operator browser smoke recommended
- Planning page risk badge when active plan exists (D-3 browser) — API contract satisfied

## Operator advisory

After deploy, confirm Firefly manual sync succeeds and Planning page loads without risk-score **404**. Optional: hard refresh with Traefik basic-auth for OIDC regression check.
