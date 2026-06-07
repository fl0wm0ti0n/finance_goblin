# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## architecture-20260607-bug0014 — BUG-0014 post-rebuild omniflow architecture (hot pointer)`
- Last archived heading: `## architecture-20260607-bug0014 — BUG-0014 post-rebuild omniflow architecture (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=42
  - retained_body_lines=496

---

## architecture-20260607-bug0014 — BUG-0014 post-rebuild omniflow architecture (hot pointer)

**From:** Tech Lead  
**To:** Sprint-plan  
**Date:** 2026-06-09  
**Bug:** BUG-0014  
**Orchestrator run:** `auto-20260607-bug0014-001`  
**Next phase:** `/sprint-plan`

### Summary

Ops gates **AO/AT/AR** + code tasks **AO1, AQ1, AQ2, AS1** + conditional **AP2** + **V1**. **DEC-0081** (AQ), **DEC-0082** (AS delete guard), **DEC-0083** (AS target_type). AO1 extends DEC-0066/DEC-0076; AP2 gated on AP1 SQL probe.

### Decisions

| ID | Sub | Contract |
|----|-----|----------|
| **DEC-0081** | AQ | `holdings_all` + unified `fx_incomplete` |
| **DEC-0082** | AS1 | 409 on active plan delete |
| **DEC-0083** | AS2 | Remove invalid `account`; DB enum + help |

### Execute scope (P0)

| Task | Surface | Gate |
|------|---------|------|
| **AO1** | `forecast-horizons.json` panel 13 | — |
| **AQ1** | `wealth/service.rs`, `types.rs` | — |
| **AQ2** | `WealthPage.tsx`, `api.ts` | after AQ1 |
| **AS1** | `plans.rs`, `PlanningPage.tsx` | — |
| **AP2** | `wealth/service.rs` | AP1 priced + subtotal 0 |
| **V1** | verify-work AO–AT | operator gates |

**Ops-only:** AO/AT runtime, AP1 operator SQL probe. **P1:** AS2. **P2:** AR1 if API≠Grafana.

**Evidence:** [R-0079 §6](docs/engineering/research.md#r-0079--bug-0014-post-rebuild-omniflow-ml-sidecar-crypto-display-grafana-planning), `docs/engineering/architecture.md` § BUG-0014

`triad_hot_surface`: architecture prepended post-rollover; --check PASS (2026-06-09T23:45:00Z)

**Recommended sprint:** `/quick` **Q0022** (≤12 tasks)

---

