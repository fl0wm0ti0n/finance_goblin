# Sprint S0011

**ID:** S0011  
**Story:** US-0011 — Unified analytics UI in financegnome (Grafana in-app)  
**Status:** PLANNED  
**Created:** 2026-06-02

## Goal

Deliver **DEC-0057** unified analytics shell: same-origin reverse proxy at `/analytics/grafana/` → `GRAFANA_UPSTREAM`, six React kiosk iframe pages under `/analytics/{slug}`, **Analytics** sidebar group, Wealth portfolio migration to in-app embed, Grafana anonymous Viewer compose env, SPA `frame-src 'self'`, env contract deprecating `VITE_GRAFANA_URL`, automated proxy/regression tests, and operator user guide — single-URL analytics on `financegnome.omniflow.cc` without public Grafana host.

## Scope

- `GRAFANA_UPSTREAM` in backend `AppConfig` with allowlisted host validation (R-0056, DEC-0057)
- Axum analytics proxy module — prefix strip, WebSocket upgrade, `X-Frame-Options` rewrite; merged **before** SPA static fallback, **outside** `/api/v1` JWT stack
- Grafana compose env: `GF_AUTH_ANONYMOUS_ENABLED`, `GF_AUTH_ANONYMOUS_ORG_ROLE=Viewer`, `GF_SECURITY_ALLOW_EMBEDDING`
- `.env.example`: `GRAFANA_UPSTREAM`, `VITE_GRAFANA_EMBED_BASE`; deprecate `VITE_GRAFANA_URL`
- `AnalyticsEmbedPage` + six routes per discovery route map
- `AppLayout` **Analytics** nav group (six links)
- `WealthPage` → `/analytics/portfolio` (remove default external Grafana tab)
- SPA CSP `frame-src 'self'` (R-0056 §2)
- Proxy smoke + WebSocket integration tests
- ECharts page regression tests (`/forecast`, `/wealth`, `/planning`, `/subscriptions`, `/alerts`)
- `docs/user-guides/US-0011.md` — single-URL UX, future-chart guideline, canonical UX table

**Out of scope:** Public `GRAFANA_TRAEFIK_HOST` default embed path; Grafana JSON/SQL uid changes (DEC-0012); `GF_SERVER_SERVE_FROM_SUB_PATH`; auth-proxy OIDC headers; full React port of Grafana panels; secondary ECharts → `/analytics/*` cross-links (optional follow-up).

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| WebSocket/live refresh through proxy | T-0127 smoke on dashboard with auto-refresh | R-0056 §4, DEC-0057 |
| Framing headers block iframe | Proxy strips `X-Frame-Options`; CSP `frame-src 'self'` | R-0056 §2, T-0126 |
| Anonymous Grafana too permissive | Viewer role only; internal network; no public router | DEC-0056, DEC-0057 |
| Upstream SSRF misconfig | Allowlist `grafana` host in config validation | DEC-0057, T-0119 |
| Dev vs Docker upstream mismatch | Document `localhost:3000` vs `http://grafana:3000` in user guide | T-0129 |
| Traefik basic-auth + iframe | Same-origin embed reuses edge auth (expected) | R-0056 §5 |
| Omniflow smoke insufficient Viewer isolation | DEC-0057 decision gate — stop for new DEC, do not expose Grafana publicly | DEC-0057 |

## Definition of Done

- All 11 sprint tasks complete (`T-0119` … `T-0129`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0011 (7 AC)
- Proxy reachable at `/analytics/grafana/` with prefix strip and framing headers suitable for iframe
- Six `/analytics/{slug}` routes render kiosk iframes without `target=_blank`
- Analytics sidebar lists all six dashboards
- Wealth primary portfolio analytics is in-app `/analytics/portfolio`
- Existing ECharts product pages pass regression tests
- `docs/user-guides/US-0011.md` documents single-URL UX and future-chart guideline
- No Grafana dashboard uid/SQL changes; no public Grafana Traefik router for default embed path

## Architecture references

- `docs/engineering/architecture.md` — US-0011
- `decisions/DEC-0057.md`
- Research: R-0054, R-0056
- Depends on: US-0010 internal Grafana networking (DEC-0056, released)
- Spec-pack: `docs/engineering/spec-pack/US-0011-*.md`
- Route map: `handoffs/po_to_tl.md#discovery-20260602-us0011`
