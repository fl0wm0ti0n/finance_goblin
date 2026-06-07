# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 9
- First archived heading: `## architecture-20260607-bug0014 — BUG-0014 post-rebuild omniflow architecture (hot pointer)`
- Last archived heading: `## research-20260607-bug0014 — BUG-0014 post-rebuild omniflow research (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=99
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

`triad_hot_surface`: architecture prepended; --rollover + --check pending

**Recommended sprint:** `/quick` **Q0022** (AO1 + AQ1 + AQ2 + AS1 + V1 + conditional AP2; ≤12 tasks)

---

## research-20260607-bug0014 — BUG-0014 post-rebuild omniflow research (hot pointer)

**From:** Tech Lead  
**To:** Architecture  
**Date:** 2026-06-09  
**Bug:** BUG-0014  
**Orchestrator run:** `auto-20260607-bug0014-001`  
**Next phase:** `/architecture`

### Summary

Research traces AP to **missing priced futures wallet row** (deploy Q0020 + Full sync gate before code) and AQ to **wealth layer not wiring PnL `unpriced_assets` / all-holdings display**. AR is **verify-only** until API≠Grafana proven on acct **114**. AS is **delete-plan UI + target_type help** (UI `account` option invalid vs DB enum). AO1 is **Grafana forecast-horizons static banner** — dual-scenario copy or Postgres `ml_skipped_reason` variable.

### Fix surfaces (execute after architecture)

| Sub | Task | Primary surface |
|-----|------|-----------------|
| **AP1** | Deploy + SQL wallet probe | Operator gate; `exchange_holdings` bitunix futures row |
| **AP2** | Subtotal hardening (if AP1 priced) | `backend/src/wealth/service.rs` |
| **AQ1** | Wire unpriced + all holdings | `wealth/service.rs`, `wealth/types.rs`, `WealthPage.tsx` |
| **AR** | Operator re-smoke | acct **114**, default time range, API daily vs Grafana SQL |
| **AS1** | Delete plan UI | `PlanningPage.tsx` + active-plan guard in `plan/service.rs` |
| **AS2** | Target-type help | `PlanningPage.tsx` — remove `account`, add copy |
| **AO1** | ML banner copy | `grafana/.../forecast-horizons.json` panel id 13 |

### AR verify gate (condensed)

Full sync + recompute → `$account_id=114` → time `now-30d`..`now+6M` → `GET /forecast/daily?account_id=114` non-zero → match panel SQL. **AR1 code only if API non-zero and Grafana zero.**

### Decisions

**None at research** — AQ display contract, AS delete guard, target_type alignment → architecture (see `decisions.md` BUG-0014 research note).

**Evidence:** [R-0079 §6](docs/engineering/research.md#r-0079--bug-0014-post-rebuild-omniflow-ml-sidecar-crypto-display-grafana-planning)

`triad_hot_surface`: research hot; discovery archived in place

---

