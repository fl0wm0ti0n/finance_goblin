# PO to TL archive pack (2026-06-09)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 11
- First archived heading: `## discovery-20260609-bug0016 — BUG-0016 SPA deep-link 404 discovery (hot pointer)`
- Last archived heading: `## discovery-20260609-bug0016 — BUG-0016 SPA deep-link 404 discovery (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=44
  - retained_body_lines=500

---

## discovery-20260609-bug0016 — BUG-0016 SPA deep-link 404 discovery (hot pointer)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-09  
**Bug:** BUG-0016  
**Orchestrator run:** `intake-20260609-ui-audit`  
**Next phase:** `/research`

### Summary

Post-US-0020 UI audit finding **UI-001**: direct navigation, hard refresh, and bookmarks to client routes return **HTTP 404** (empty body) instead of SPA shell. In-app sidebar navigation works — defect is **server-side routing only**, not React pages.

**Verdict:** **CONFIRMED** single infrastructure defect. Intake single-bug decomposition **unchanged** — one `index.html` fallback fix covers all client routes.

### Scope (discovery-refined)

| Category | Paths |
|----------|-------|
| **Must fix (acceptance AX)** | `/forecast`, `/subscriptions`, `/planning`, `/sync`, `/analytics/{slug}` |
| **Same contract (expanded)** | `/wealth`, `/alerts`, `/chat`, `/settings` |
| **Must preserve** | `/api/v1/*`, `/analytics/grafana/*`, `/callback`, static assets |
| **Environments** | `localhost:18080` override + `financegnome.omniflow.cc` external profile |

### Research pointers (for `/research`)

- Where fallback belongs: Axum `build_router` (`ServeDir` vs custom fallback handler) vs Traefik catch-all vs both
- Omniflow Traefik label parity with local compose
- OIDC `/callback` ordering — must not redirect to `/` before OIDC handler runs
- Regression matrix: curl **200** + HTML on client paths; API/Grafana proxy unchanged
- Supersedes BUG-0009 advisory on analytics SPA 404

### Artifacts updated

- `docs/product/vision.md` § BUG-0016 discovery
- `docs/product/backlog.md#BUG-0016`
- `handoffs/resume_brief.md`

**Evidence:** `handoffs/intake_evidence/intake-20260609-spa-deep-link.json`, `handoffs/intake_evidence/ui-audit-20260609-local.json` (UI-001), `docs/product/acceptance.md` row **AX**

`triad_hot_surface`: BUG-0016 discovery prepended; --rollover units=1 + --check PASS (2026-06-09T20:30:00Z)

---

