# CRS — BUG-0016 SPA deep links return HTTP 404

## Purpose

Close operator/UI-audit defect **UI-001**: server returns 404 for React Router deep links because `build_router` uses plain `ServeDir` without SPA `index.html` fallback.

## Scope

### In scope

- **DEC-0104:** `ServeDir::fallback(ServeFile::new(index.html))` in `build_router` for `/app/static` and `frontend/dist`
- Route ordering unchanged: health → Grafana proxy (**DEC-0057**) → API → SPA fallback
- `/callback` served as SPA shell (React OIDC handler)
- `build_router` integration tests: deep-link 200; API/proxy non-HTML
- QA curl matrix (R-0086 §6) + operator browser smoke on omniflow

### Out of scope

- Traefik router label edits
- React route table changes
- Grafana embed panel data (BUG-0009)
- `axum_extra::SpaRouter` migration

## Acceptance criteria ref

`docs/product/acceptance.md` — BUG-0016:

- **(AX)** Direct navigation, hard-refresh, and bookmarks to `/forecast`, `/subscriptions`, `/planning`, `/sync`, `/analytics/{slug}` return **HTTP 200** with SPA shell — not 404 blank body. Applies on `:18080` and `financegnome.omniflow.cc`. OIDC-enabled deploy regression checks pass.

## Dependencies

- US-0010 external profile (Traefik pass-through)
- US-0011 / **DEC-0057** (Grafana proxy ordering)
- R-0086 research complete
