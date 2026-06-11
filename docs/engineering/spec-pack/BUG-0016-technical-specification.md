# Technical Specification — BUG-0016

## Overview

Implement **DEC-0104**: replace plain `ServeDir` fallback with `ServeDir::fallback(ServeFile::new(index.html))` returning HTTP 200 for missing non-API paths. No frontend or Traefik changes.

## Components

| Layer | Change | Decision |
|-------|--------|----------|
| `backend/src/lib.rs` | `build_router`: SPA fallback with `ServeDir` + `ServeFile` | DEC-0104 |
| `backend/tests/` (or `lib` tests) | Integration: deep links 200; API/proxy not HTML | DEC-0104 |
| `docker-compose.external.yml` | **No change** | R-0086 §5 |
| `frontend/src/App.tsx` | **No change** | Routes already correct |

## Interfaces

### SPA fallback (DEC-0104)

- **Trigger:** Request path has no matching file under static dir and no prior router match
- **Response:** `200 text/html` — contents of `index.html`
- **Static dirs:** `/app/static` (prod image); `frontend/dist` (local dev when prod dir absent)

### Protected prefixes (must not regress)

| Prefix | Expected |
|--------|----------|
| `/api/v1/*` | JSON (401/404 per auth), not HTML |
| `/analytics/grafana/*` | Grafana proxy response (**DEC-0057**) |
| `/assets/*` | Hashed static files when present |
| `/callback` | SPA shell — React `OidcCallback` |

### Primary AX curl paths

| Path | Expected |
|------|----------|
| `GET /forecast` | 200 HTML with `#root` or Vite shell |
| `GET /subscriptions` | 200 HTML shell |
| `GET /planning` | 200 HTML shell |
| `GET /sync` | 200 HTML shell |
| `GET /analytics/cashflow` | 200 HTML shell |

## Non-functional

- **Compatibility:** localhost `:18080`, Docker prod, omniflow external profile
- **Security:** SPA shell is public HTML; API remains JWT/OIDC gated
- **Testing:** `cargo test` integration for `build_router`; QA curl script in verify-work
- **Regression:** BUG-0009 panel data — operator authenticated smoke unchanged

## Traceability

- DEC-0104, R-0086, `docs/engineering/architecture.md` § **BUG-0016**
