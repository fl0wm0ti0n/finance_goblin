# Quick Release Notes — Q0007 / BUG-0001

**Quick task:** Q0007  
**Bug:** BUG-0001 — Omniflow production regressions (auth + Grafana analytics)  
**Date:** 2026-06-04  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0001)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd frontend && npm test` (2/2), `npm run build` @ 2026-06-04 (release re-run)
2. **QA completion gate:** PASS — `sprints/quick/Q0007/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0007/uat.md`, `handoffs/verify_work_to_release.md`; omniflow row B live curl PASS; row A unit/build PASS (browser advisory)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

---

## Run

**Target services (external profile):** `flow-finance-ai`, `grafana` — both must rebuild/recreate for A1 + B1.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana
```

**Full stack (if other services stale):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build
```

- `runtime_mode`: omniflow external (US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§ Omniflow §9 BUG-0001)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- `analytics_routes`: six SPA embeds under `/analytics/{slug}`; Grafana proxy `/analytics/grafana/`
- **A1:** `DevBypassAuthProvider` supplies `AuthContext` when `!isOidcConfigured`
- **B1:** `GF_SERVER_ROOT_URL` (trailing slash) — default `https://financegnome.omniflow.cc/analytics/grafana/` in compose

Optional non-omniflow override in operator `.env`:

```bash
GF_SERVER_ROOT_URL=http://localhost:8080/analytics/grafana/
```

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **A** | `cd frontend && npm test && npm run build` | 2/2; build OK |
| **A** | Hard refresh SPA + open **AI Chat** (Traefik basic-auth) | No `useAuth` / `user` TypeError |
| **B** | Six embed URLs `GET /analytics/grafana/d/{uid}/{slug}?kiosk=tv` | HTTP 200 each |
| **B** | Prefixed assets `/analytics/grafana/public/build/…`, `public/img/…` | HTTP 200 |
| **B** | Site-root `/public/build/…` | **401** (Traefik auth), not **404** |
| Stack | `GET /health`, `GET /analytics/grafana/api/health` | 200 |

**Automated (release):**

```bash
cd frontend && npm test
cd frontend && npm run build
```

**Boundaries preserved:** no `GF_SERVER_SERVE_FROM_SUB_PATH`; no B2 proxy HTML rewrite; no JWT/proxy stack changes.

---

## Deliverables

| Slice | Change |
|-------|--------|
| **A1** | `frontend/src/auth/DevBypassAuthProvider.tsx`; wired in `main.tsx`; `ChatPanel.test.tsx` |
| **B1** | `GF_SERVER_ROOT_URL` in `docker-compose.yml` + `.env.example` |
| **B2** | Skipped — B1 sufficient per verify-work |

## Evidence refs

- `sprints/quick/Q0007/summary.md`, `qa-findings.md`, `uat.md`
- `handoffs/verify_work_to_release.md`, `dev_to_qa.md`, `qa_to_release.md`
- `handoffs/tl_to_dev.md` (architecture-20260604-bug0001)
- `decisions/DEC-0057.md`

## Known issues (non-blocking)

- Row **A** browser console + ChatPanel click: operator advisory when Traefik `auth` blocks curl on SPA shell
- If row B regresses (site-root `/public/` **404**): escalate **B2** per `handoffs/tl_to_dev.md`

## Operator advisory

Hard refresh `https://financegnome.omniflow.cc/` after deploy; confirm **AI Chat** opens without AuthProvider errors.
