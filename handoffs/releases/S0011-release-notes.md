# Sprint Release Notes — S0011

**Sprint:** S0011  
**Date:** 2026-06-03  
**Stories:** US-0011  
**Queue status:** released

---

## Gate results

1. **Check-in test gate:** PASS — verify-work re-ran targeted suites @ 2026-06-03 (`analytics_proxy_integration` 4/4, `product_routes_regression` 4/4, `grafana_upstream` 3/3, `npm run build`)
2. **QA completion gate:** PASS — `sprints/S0011/qa-findings.md`, `sprints/S0011/qa.json` (7/7 AC; 0 blockers)
3. **UAT completeness gate:** PASS-with-prerequisites — `sprints/S0011/uat.json`; omniflow host deferred `OMNIFLOW_HOST_UNAVAILABLE` (S0010 precedent)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build`
- `runtime_mode`: omniflow external (unchanged US-0010); Grafana anonymous Viewer + same-origin proxy (DEC-0057)
- `runtime_context_ref`: `docs/engineering/runbook.md` (Omniflow external deploy + US-0011 analytics); operator guide `docs/user-guides/US-0011.md`

**Profile rule:** use **`external` only** — do not combine with `bundled-firefly`.

Greenfield dev: `docker compose --profile minimal --profile bundled-firefly up --build`

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (Traefik + basic-auth middleware `auth`)
- `analytics_routes`: `/analytics/{slug}` (six dashboards) + proxied Grafana at `/analytics/grafana/`
- `health_endpoint`: `https://financegnome.omniflow.cc/health`; proxy health `https://financegnome.omniflow.cc/analytics/grafana/api/health` (edge auth required)

## Verify

- `verification_steps`:
  1. Deploy per US-0010 runbook (external profile).
  2. Confirm `.env` includes `GRAFANA_UPSTREAM` (default `http://grafana:3000`) and `VITE_GRAFANA_EMBED_BASE=/analytics/grafana` per `.env.example`.
  3. In-network: `curl -sf http://flow-finance-ai:8080/health` → OK JSON.
  4. Edge (with Traefik basic-auth): `curl -sfI https://financegnome.omniflow.cc/health` → 200.
  5. Edge proxy: `curl -s -o /dev/null -w "%{http_code}" https://financegnome.omniflow.cc/analytics/grafana/api/health` → 200 (after auth).
  6. SPA: open each `/analytics/{slug}` (platform-health, cashflow, subscriptions, budgets, portfolio, forecast-horizons) — iframe loads under same origin.
  7. Sidebar **Analytics** group lists all six routes; Wealth portfolio link targets `/analytics/portfolio`.
  8. Regression: Forecast, Wealth, Planning, Subscriptions, Alerts pages still load.
  9. Automated: `cargo test --test analytics_proxy_integration --test product_routes_regression`; `npm run build`.
  10. Optional: Grafana Live WebSocket refresh on one dashboard (`/analytics/grafana/api/live/`).
- `expected_health_signal`: proxy returns 200; iframes render; no public `GRAFANA_TRAEFIK_HOST` required; CSP `frame-src 'self'`

## Credentials

- `credential_source_refs` (env names only):
  - US-0010 block: `COMPOSE_FILE`, `COMPOSE_PROFILES`, `DATABASE_PASSWORD`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, `TRAEFIK_HOST`, `TRAEFIK_MIDDLEWARE`, `GRAFANA_ADMIN_PASSWORD`
  - US-0011: `GRAFANA_UPSTREAM`, `VITE_GRAFANA_EMBED_BASE`; optional deprecated `VITE_GRAFANA_URL`; `GRAFANA_TRAEFIK_HOST` empty by default
- `expected_value_source`: operator `.env` at repo root on omniflow host (from `.env.example`)

## Known Issues

- Omniflow analytics smoke **PENDING** operator post-deploy — iframe ×6, proxy health, Grafana Live WS (`OMNIFLOW_HOST_UNAVAILABLE` at QA/verify-work).
- `compose-config-check.sh` may fail on env-dependent `DATABASE_HOST` external merge assert (unrelated to US-0011).
- Integration tests require operator `DATABASE_URL` (carry-forward).
- If anonymous Grafana insufficient, escalate auth-proxy DEC — do not enable public `GRAFANA_TRAEFIK_HOST` without new DEC (DEC-0057 gate).

## Deliverables (US-0011)

- Axum reverse proxy `/analytics/grafana/` (prefix strip, framing rewrite, WebSocket `/api/live/`, SSRF allowlist)
- Six `AnalyticsEmbedPage` routes under `/analytics/:slug`
- Analytics sidebar nav; Wealth → in-app portfolio analytics
- Grafana anonymous Viewer + `GF_SECURITY_ALLOW_EMBEDDING` in compose
- CSP `frame-src 'self'`; `.env.example` contract
- Tests: `analytics_proxy_integration`, `product_routes_regression`, grafana upstream allowlist
- Operator guide: `docs/user-guides/US-0011.md`
- Decision: DEC-0057

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0011 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.11.0-us0011`

## Milestone

**US-0011 released** — unified analytics in financegnome SPA; backlog continues with US-0012 (OPEN).
