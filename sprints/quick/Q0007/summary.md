# Q0007 — BUG-0001 omniflow production regressions

## Scope

Quick execute for BUG-0001 per `architecture-20260604-bug0001` in `handoffs/tl_to_dev.md`.

| Task | Deliverable | Status |
|------|-------------|--------|
| **A1** | `DevBypassAuthProvider` on `AuthContext` when `!isOidcConfigured`; wired in `main.tsx`; Vitest for `ChatPanel` | Done |
| **B1** | `GF_SERVER_ROOT_URL` in `docker-compose.yml` + `.env.example` | Done |
| **B2** | Proxy HTML rewrite | Skipped — B1 sufficient |

## Sub-defect A — AuthProvider/useAuth crash

**Problem:** Q0005 skipped `AuthProvider` when OIDC unset, but `AppLayout` and `ChatPanel` still call `useAuth()` unconditionally.

**Fix:** `DevBypassAuthProvider` mounts instead of `react-oidc-context` `AuthProvider`, providing a stub `AuthContextProps` value (`user` undefined → dev-bypass API path; OIDC UI already gated on `isOidcConfigured`).

## Sub-defect B — Grafana `/public/` 404 at site root

**Problem:** Grafana 11 HTML emits root-absolute asset URLs; prefix-strip proxy forwards HTML unchanged → browser resolves `/public/…` against site origin.

**Fix:** `GF_SERVER_ROOT_URL` with trailing slash so Grafana generates browser-visible URLs under `/analytics/grafana/`. `GF_SERVER_SERVE_FROM_SUB_PATH` remains unset (DEC-0057).

## Test evidence

```bash
cd frontend && npm test    # 2 passed (DevBypassAuthProvider + ChatPanel stub mount)
cd frontend && npm run build  # PASS
```

## Operator redeploy

B1 requires Grafana container recreate to pick up `GF_SERVER_ROOT_URL`. A1 requires frontend image rebuild.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana
```

Optional `.env` override for non-omniflow hosts:

```bash
GF_SERVER_ROOT_URL=http://localhost:8080/analytics/grafana/
```

Hard refresh browser after deploy. Verify BUG-0001 acceptance rows A+B per `docs/product/acceptance.md`.

## B2 gate

Proxy HTML rewrite (`rewrite_grafana_html`) not implemented — env-first B1 addresses DEC-0057 risk row; implement B2 only if operator smoke still shows site-root `/public/` 404.

## Closure (verify-work 2026-06-04)

**Status:** DONE — verify-work PASS; acceptance checked.

| Row | Verdict | Evidence |
|-----|---------|----------|
| **A** (auth stub) | PASS | vitest 2/2; build PASS; browser Chat advisory (Traefik auth) |
| **B** (Grafana assets) | PASS | omniflow curl — six embeds 200; prefixed assets 200; site-root 401 not 404 |
| B2 gate | Closed | B1 sufficient per live smoke |

**Artifacts:** `sprints/quick/Q0007/uat.md`, `handoffs/verify_work_to_release.md`, `docs/product/acceptance.md` (BUG-0001 checked)
