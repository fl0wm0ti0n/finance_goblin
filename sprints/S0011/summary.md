# Sprint S0011 summary — US-0011 unified analytics

**Story:** US-0011  
**Sprint:** S0011  
**Phase:** released (2026-06-03, `0.11.0-us0011`)  
**Decision:** DEC-0057  
**Context refresh:** 2026-06-03T02:00:00Z — handoffs/resume_brief.md points to US-0012 discovery

## Delivered

- **T-0119:** `GRAFANA_UPSTREAM` in `AppConfig` with host allowlist (`grafana`, `localhost`, `127.0.0.1`)
- **T-0120:** Axum reverse proxy at `/analytics/grafana/` (prefix strip, framing header rewrite, no `Set-Cookie` forward, WebSocket `/api/live/`)
- **T-0121:** Grafana anonymous Viewer + `GF_SECURITY_ALLOW_EMBEDDING` in `docker-compose.yml`
- **T-0122:** `.env.example` — `GRAFANA_UPSTREAM`, `VITE_GRAFANA_EMBED_BASE`; deprecated `VITE_GRAFANA_URL`
- **T-0123:** `AnalyticsEmbedPage` + `/analytics/:slug` (six dashboards)
- **T-0124:** Analytics sidebar nav group in `AppLayout`
- **T-0125:** Wealth portfolio link → in-app `/analytics/portfolio`
- **T-0126:** CSP `frame-src 'self'` in `frontend/index.html`
- **T-0127:** `analytics_proxy_integration` tests (wiremock)
- **T-0128:** `product_routes_regression` tests
- **T-0129:** `docs/user-guides/US-0011.md`

## Tests

| Suite | Result |
|-------|--------|
| `cargo test --test analytics_proxy_integration` | PASS (4) |
| `cargo test --test product_routes_regression` | PASS (4) |
| `cargo test --lib config::tests` (grafana allowlist) | PASS |
| `npm run build` (frontend) | PASS |
| `compose-config-check.sh` | FAIL — pre-existing `DATABASE_HOST` external merge assert (env-dependent; unrelated to US-0011) |

## Release

- **Version:** `0.11.0-us0011`
- **Notes:** `handoffs/releases/S0011-release-notes.md`
- **Findings:** `sprints/S0011/release-findings.md`
- **Gates:** QA PASS, verify-work PASS, omniflow runtime deferred (`OMNIFLOW_HOST_UNAVAILABLE`)
