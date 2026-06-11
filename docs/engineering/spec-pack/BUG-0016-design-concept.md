# Design Concept — BUG-0016

## Summary

BUG-0016 fixes server-side SPA routing so direct navigation, hard-refresh, and bookmarks to React client routes return **HTTP 200** with the Vite shell — not **404** with a blank body. Single backend change in `build_router` via **DEC-0104**; Traefik remains transparent pass-through.

## Goals

- **AX:** `/forecast`, `/subscriptions`, `/planning`, `/sync`, `/analytics/{slug}` return 200 + SPA shell on `:18080` and `financegnome.omniflow.cc`
- Preserve **DEC-0057** Grafana proxy at `/analytics/grafana/*`
- Preserve `/api/v1/*` JSON semantics (401/404, not HTML)
- `/callback` loads shell for OIDC `OidcCallback` — no backend redirect
- Integration test + QA curl regression matrix

## Non-goals

- Traefik label or nginx sidecar changes
- React Router config changes (already correct)
- Backend OIDC callback route
- Grafana panel data fixes (BUG-0009 **Y** remains operator auth smoke)

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0104 | `ServeDir::fallback(ServeFile)` | HTTP 200 per AX; R-0086 §3 |
| Placement | Axum-only | Covers all profiles; R-0086 §2 |
| Route order | health → grafana → api → SPA | R-0056 / DEC-0057 unchanged |
| Sprint shape | `/quick` ≤3 tasks | Code + tests + verify |

**Spec-pack traceability:** `docs/engineering/spec-pack/BUG-0016-crs.md`, `docs/engineering/spec-pack/BUG-0016-technical-specification.md`
