# PO to TL archive pack (2026-06-09)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 11
- First archived heading: `## research-20260609-bug0016 — BUG-0016 SPA fallback research (hot pointer)`
- Last archived heading: `## architecture-20260609-bug0016 — BUG-0016 SPA deep-link fallback architecture (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=38
  - retained_body_lines=498

---

## research-20260609-bug0016 — BUG-0016 SPA fallback research (hot pointer)
**From:** Tech Lead **To:** Architecture **Bug:** BUG-0016 **Run:** `intake-20260609-ui-audit` **Next:** `/architecture` — **DONE**; see architecture pointer below. **Full:** `handoffs/archive/po-to-tl-pack-20260609-d.md` · [R-0086](docs/engineering/research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik) · discovery `po-to-tl-pack-20260609-a.md` · AX
## architecture-20260609-bug0016 — BUG-0016 SPA deep-link fallback architecture (hot pointer)

**From:** Tech Lead  
**To:** Sprint-plan  
**Date:** 2026-06-09  
**Bug:** BUG-0016  
**Orchestrator run:** `intake-20260609-ui-audit`  
**Next phase:** `/sprint-plan`

### Summary

Axum-only SPA fallback via **DEC-0104**: `ServeDir::fallback(ServeFile::new(index.html))` returning HTTP 200 in `build_router`. Route order frozen (health → Grafana proxy **DEC-0057** → API → SPA). Traefik pass-through — no label change. `/callback` SPA shell only — no backend redirect.

### Decisions

| ID | Contract |
|----|----------|
| **DEC-0104** | Axum SPA fallback HTTP 200; extends DEC-0057 ordering; Traefik unchanged |

### Execute scope (P0)

| Task | Surface | Gate |
|------|---------|------|
| **AX1** | `backend/src/lib.rs` `build_router` | — |
| **AX2** | `backend/tests/` integration | after AX1 |
| **V1** | verify-work curl + browser smoke | AX matrix |

**Out of scope:** Traefik labels, React routes, backend `/callback` handler, Grafana panel data (BUG-0009 **Y**).

**Evidence:** [R-0086](docs/engineering/research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik), `docs/engineering/architecture.md` § **BUG-0016**, `docs/engineering/spec-pack/BUG-0016-*.md`

`triad_hot_surface`: architecture § BUG-0016 prepended (H1); --rollover + --check PASS (2026-06-09T22:00:00Z)

**Recommended sprint:** `/quick` **Q0024** (AX1 + AX2 + V1; ≤3 tasks)

---
