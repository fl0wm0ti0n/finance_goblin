# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## architecture-20260607-bug0014 — BUG-0014 post-rebuild omniflow architecture (hot pointer)`
- Last archived heading: `## architecture-20260607-bug0014 — BUG-0014 post-rebuild omniflow architecture (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=60
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

Post-research cluster formalized as **ops gates (AO/AT/AR)** + **five code tasks** + verify-work — not a feature epic. **DEC-0081** (AQ holdings + FX), **DEC-0082** (AS active-plan delete guard), **DEC-0083** (AS target_type UI); AO1 extends DEC-0066/DEC-0076; AP2 conditional on AP1 deploy gate; AR1 only if API≠Grafana.

### Decisions

| ID | Sub | Contract |
|----|-----|----------|
| **DEC-0081** | AQ | `holdings_all` + unified `fx_incomplete`; DEC-0064 subtotal preserved |
| **DEC-0082** | AS1 | 409 on active plan delete |
| **DEC-0083** | AS2 | Remove invalid `account`; DB enum values + help |

### Execute scope (P0)

| Task | Surface | Gate |
|------|---------|------|
| **AO1** | `forecast-horizons.json` panel 13 | — |
| **AQ1** | `wealth/service.rs`, `types.rs` | — |
| **AQ2** | `WealthPage.tsx`, `api.ts` | after AQ1 |
| **AS1** | `plans.rs`, `PlanningPage.tsx` | — |
| **AP2** | `wealth/service.rs` | AP1 priced + subtotal 0 |
| **V1** | verify-work AO–AT | operator gates |

### P1/P2 / ops-only

| Item | Action |
|------|--------|
| **AS2** | P1 — target_type help copy |
| **AR1** | P2 — cashflow.json only if API≠Grafana |
| **AO/AT** | Ops — three-service compose + Full sync (no execute) |
| **AP1** | Operator SQL wallet probe (prerequisite, not sprint task) |

### Operator gates (before V1)

1. **BACKEND_FRONTEND_DEPLOY** (Q0020 / DEC-0080)
2. **stats-forecast** up when ML enabled
3. **Full Firefly sync** + forecast recompute acct **114**
4. **AP1** SQL probe on `exchange_holdings`

### Artifacts updated

- `docs/engineering/architecture.md` § **BUG-0014**
- `decisions/DEC-0081.md`, `DEC-0082.md`, `DEC-0083.md`
- `docs/engineering/decisions.md`, `docs/engineering/state.md`

`triad_hot_surface`: architecture prepended; --rollover units=2,2,1 + --check PASS (2026-06-09T23:45:00Z)

**Recommended sprint:** `/quick` **Q0022** (AO1 + AQ1 + AQ2 + AS1 + V1 + conditional AP2; ≤12 tasks)

---

