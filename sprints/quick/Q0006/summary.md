# Q0006 — Traefik auth popup every few seconds (fix)

## Root cause

1. **Traefik:** Every request (including `/api/v1/*` polls) went through the `auth` middleware. Failed/credential-less fetches returned `401` + `WWW-Authenticate: Basic realm="traefik"` → browser shows login dialog again and again.

2. **Backend:** `.env` had `AUTH_DEV_BYPASS=false` and placeholder `OIDC_ISSUER_URL` from `.env.example`. API required JWT Bearer tokens the SPA never sends (OIDC not configured in frontend build).

`credentials: include` (Q0005) is not enough — browsers do not reliably attach HTTP Basic credentials to `fetch()` for repeated API polls.

## Fix (`docker-compose.external.yml`)

| Router | Paths | Traefik `auth` |
|--------|-------|----------------|
| `financegnome-api` (priority 100) | `/api`, `/analytics`, `/health` | **none** |
| `financegnome` (priority 1) | everything else (SPA, static) | **auth** |

External profile environment:

- `AUTH_DEV_BYPASS=true` (overrides `.env` false for homelab)
- `OIDC_ISSUER_URL=""` (disables broken placeholder IdP)

## Security note

`/api` is reachable without Traefik basic-auth. Acceptable for homelab when edge trust is Traefik-protected UI + network; enable real OIDC + `AUTH_DEV_BYPASS=false` for stricter deployments.

## Verify

```bash
curl -sk -w "%{http_code}" https://financegnome.omniflow.cc/api/v1/sync/status   # expect 200 + JSON
curl -sk -w "%{http_code}" https://financegnome.omniflow.cc/                     # expect 401 (no basic auth)
```

Hard refresh browser after deploy.
