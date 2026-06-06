# Quick Release Notes — Q0009 / BUG-0003

**Quick task:** Q0009  
**Bug:** BUG-0003 — Omniflow production API 500 cascade, Bitunix test, Grafana SQL  
**Date:** 2026-06-05  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0003)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (103/103), `cd frontend && npm test` (2/2), `npm run build` @ 2026-06-05 (release re-run)
2. **QA completion gate:** PASS — `sprints/quick/Q0009/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0009/uat.md`, `handoffs/verify_work_to_release.md`; omniflow rows F/G/H live curl PASS (OIDC regression deferred)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

---

## Run

**Target services (external profile):** `flow-finance-ai` + `grafana` — rebuild/recreate for G1 code; operator **F1** required for acceptance F/H.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana
```

**Operator prerequisite (F1):** `DATABASE_HOST=postgres` in operator `.env` (not `host.docker.internal`); recreate after change.

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai grafana
```

- `runtime_mode`: omniflow external (US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§ Omniflow §11 BUG-0003)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- **F:** Product `GET /api/v1/*` endpoints via in-network `postgres` DB host
- **G:** Bitunix connector registered via `effective_enabled()` when credentials present
- **H:** Grafana Postgres datasource reaches `postgres` on traefik network

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (F1 / H) |
| `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` | Optional — G row smoke |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **F** | `GET /api/v1/settings` | `database_host: postgres`, `database_mode: external` |
| **F** | `GET /api/v1/alerts/unread-count` | **200** &lt;0.1s (not 500 ~30s) |
| **F** | `GET /api/v1/sync/entities` | **200**; non-zero transaction count |
| **F** | `GET /api/v1/exchanges`, `/subscriptions`, `/ai/audit` | **200** in normal latency |
| **G** | `POST /api/v1/exchanges/bitunix/test` | **200** connection payload (not 400 unknown exchange) |
| **H** | `POST /analytics/grafana/api/ds/query` | **200**; `SELECT 1` executes |
| Stack | `GET /health` | 200 |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd frontend && npm test
cd frontend && npm run build
```

**Boundaries preserved:** no Traefik routing changes; G2 futures auth spike skipped (gated); OIDC regression deferred to operator browser smoke.

---

## Deliverables

| Slice | Change |
|-------|--------|
| **F2** | `.env.example` + runbook mis-host guard; compose comment |
| **G1** | `effective_enabled()` in `ExchangeService::build_connectors` / `new()` |
| **F1** | Runbook operator steps — `DATABASE_HOST=postgres`, recreate services |
| **G2** | Skipped (gated — no auth failure after G1+F1) |

## Evidence refs

- `sprints/quick/Q0009/summary.md`, `qa-findings.md`, `uat.md`, `verify-work-findings.md`
- `handoffs/verify_work_to_release.md`, `dev_to_qa.md`
- `handoffs/tl_to_dev.md` (architecture-20260605-bug0003)

## Known issues (non-blocking)

- OIDC-enabled deploy regression: operator browser smoke recommended
- Grafana duplicate dashboard UID provisioning warnings (H2 deferred)

## Operator advisory

After deploy, confirm `database_host: postgres` via settings API and sample product GETs return **200**. Optional: hard refresh with Traefik basic-auth for OIDC regression check.
