# PO to TL archive pack (2026-06-09)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 12
- First archived heading: `## research-20260609-bug0016 — BUG-0016 SPA deep-link fallback research (hot pointer)`
- Last archived heading: `## research-20260609-bug0016 — BUG-0016 SPA deep-link fallback research (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=47
  - retained_body_lines=500

---

## research-20260609-bug0016 — BUG-0016 SPA deep-link fallback research (hot pointer)

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-09  
**Bug:** BUG-0016  
**Orchestrator run:** `intake-20260609-ui-audit`  
**Next phase:** `/architecture`

### Summary

Web + code research completed for BUG-0016 SPA deep-link 404. Added **[R-0086](docs/engineering/research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik)** — root cause is `ServeDir` fallback without `index.html` rewrite; **Axum-only fix** recommended; Traefik unchanged (transparent reverse proxy). Supersedes BUG-0009 analytics SPA 404 advisory. No host `.env` or secrets read.

### Key findings

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Root cause** | [R-0086 §1](docs/engineering/research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik) | `build_router` `ServeDir` returns 404 for missing paths — not React bug |
| **Placement** | [R-0086 §2](docs/engineering/research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik) | **Axum `build_router` only** — covers `:18080` + omniflow; reject Traefik/nginx sidecar |
| **Implementation** | [R-0086 §3](docs/engineering/research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik) | `ServeDir::fallback(ServeFile::new(index.html))` for **HTTP 200** (AX requires 200, not `not_found_service`) |
| **Protected paths** | [R-0086 §4](docs/engineering/research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik) | Keep merge order: health → grafana proxy → API → SPA; `/callback` = React-only shell |
| **Traefik** | [R-0086 §5](docs/engineering/research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik) | No label change for MVP — both routers forward to same backend |
| **Regression** | [R-0086 §6](docs/engineering/research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik) | Curl 200+HTML on AX paths; API/Grafana/assets unchanged |

### Architecture decision gates

1. **DEC candidate** — Axum-only SPA fallback contract (`ServeDir` + `ServeFile`, 200 status, static dir paths)
2. **Route ordering** — confirm no reorder vs DEC-0057 grafana proxy
3. **`/callback`** — explicit non-goal for backend redirect handler
4. **Test plan** — `build_router` integration test + QA curl matrix (AX + protected prefixes)
5. **Sprint shape** — recommend `/quick` (≤3 tasks)

### Risks surfaced (carry to architecture)

1. API accidentally receiving `index.html` if fallback precedes API merge
2. Wrong status helper (`not_found_service` → 404 fails AX curl gate)
3. Traefik `/analytics/*` no-auth router — pre-existing; document, do not widen scope

### Recommended next steps

1. `/architecture` — DEC for SPA fallback contract; regression matrix in execute/QA
2. `/sprint-plan` — `/quick` task(s): `build_router` fix + verify-work curl/browser smoke

`triad_hot_surface`: research entry R-0086 appended; po_to_tl research prepended; --check PASS (2026-06-09T21:00:00Z)

---

