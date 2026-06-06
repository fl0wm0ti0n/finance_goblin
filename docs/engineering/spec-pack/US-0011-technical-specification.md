# Technical Specification — US-0011

## Overview

Implement DEC-0057: Axum reverse proxy at `/analytics/grafana/*`, React iframe shell at `/analytics/{slug}`, and Grafana anonymous Viewer configuration. SPA build uses `VITE_GRAFANA_EMBED_BASE`; backend uses `GRAFANA_UPSTREAM`.

## Components

| Layer | Change |
|-------|--------|
| `backend/src/analytics/` (new) | Proxy handler: strip prefix, forward to upstream, WS upgrade, header rewrite |
| `backend/src/lib.rs` | `build_router`: merge analytics routes before static fallback |
| `backend/src/config` | Parse `GRAFANA_UPSTREAM` with allowlisted host |
| `frontend/src/pages/AnalyticsEmbedPage.tsx` (new) | iframe wrapper + slug/uid props |
| `frontend/src/App.tsx` | Six analytics routes |
| `frontend/src/components/AppLayout.tsx` | Analytics nav group |
| `frontend/src/pages/WealthPage.tsx` | Link to `/analytics/portfolio` |
| `docker-compose.yml` | Grafana `GF_AUTH_ANONYMOUS_*`, `GF_SECURITY_ALLOW_EMBEDDING` |
| `.env.example` | Proxy + embed vars; deprecate `VITE_GRAFANA_URL` note |
| `docs/user-guides/US-0011.md` | Execute — operator + future-chart guideline |

## Interfaces

### Proxy

- **Public:** `GET|POST|HEAD /analytics/grafana/{*path}` (+ WebSocket upgrade)
- **Upstream:** `{GRAFANA_UPSTREAM}/{path}?{query}`
- **Not authenticated** via `/api/v1` JWT middleware

### SPA routes

| Path | uid | slug |
|------|-----|------|
| `/analytics/platform-health` | platform-health | platform-health |
| `/analytics/cashflow` | cashflow | cashflow |
| `/analytics/subscriptions` | subscriptions | subscriptions |
| `/analytics/budgets` | budgets | budgets |
| `/analytics/portfolio` | portfolio | portfolio |
| `/analytics/forecast-horizons` | forecast-horizons | forecast-horizons |

## Non-functional

- **Security:** upstream host allowlist; no operator secrets in frontend bundle beyond public embed base path
- **Compatibility:** omniflow external profile; Grafana 11.0.0 image unchanged
- **Performance:** proxy streaming for large Grafana assets; avoid buffering entire panel responses
- **Testing:** smoke `GET /analytics/grafana/api/health` (or equivalent) behind dev stack; iframe load manual/QA on one dashboard minimum

## Traceability

- DEC-0057, R-0054, `docs/engineering/architecture.md#US-0011`
