# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## discovery-20260607-bug0014 — BUG-0014 post-rebuild omniflow discovery (hot pointer)`
- Last archived heading: `## discovery-20260607-bug0014 — BUG-0014 post-rebuild omniflow discovery (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=61
  - retained_body_lines=494

---

## discovery-20260607-bug0014 — BUG-0014 post-rebuild omniflow discovery (hot pointer)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-09  
**Bug:** BUG-0014  
**Orchestrator run:** `auto-20260607-bug0014-001`  
**Intake run:** `intake-20260607-post-rebuild-omniflow`  
**Next phase:** `/research`

### Summary

Post-rebuild omniflow cluster decomposes into **two ops-only sub-defects (AO/AT)**, **three confirmed code/UX gaps (AP/AQ/AS)**, and **one likely ops/stale item (AR)** — not a single regression. Operator must start **`stats-forecast`** and run **Full sync + recompute** before sprint attributes remaining Grafana emptiness to code.

### Sub-defect verdicts

| ID | Verdict | Root cause | Fix task |
|----|---------|------------|----------|
| **AO** | **CONFIRMED (ops)** | `FORECAST_ML_ENABLED=true`; `stats-forecast` not running → `sidecar_unavailable` | Ops gate; optional **AO1** Grafana static banner distinguishes sidecar down vs ML off |
| **AT** | **CONFIRMED (ops)** | Rebuild omitted third service; external overlay defines sidecar | Runbook **AT1** — three-service `compose up` smoke |
| **AP** | **CONFIRMED (code residual)** | Wealth subtotal sums `market_value_eur` only; `unrealized_eur: 411.74` but `subtotal_eur: 0`, `holdings_top: []`, count **7** — wallet equity not in aggregation or Q0020 not on host | **AP1** verify deploy + wallet row; **AP2** subtotal/display contract |
| **AQ** | **CONFIRMED (product gap)** | `unpriced_assets` never wired in wealth breakdown; portfolio forecast warning diverges; holdings table empty | **AQ1** native qty + EUR for all holdings; **AQ2** unified FX banner gating |
| **AR** | **LIKELY (ops/stale)** | **BUG-0013 AI** refuted for acct **114**; SQL unchanged; screenshot time range suspicious | **AR1** operator re-smoke only unless API≠Grafana |
| **AS** | **CONFIRMED (UI gap)** | `DELETE /api/v1/plans/:id` exists; no React delete-plan control; target types by design | **AS1** delete plan UI; **AS2** target-type help copy |

### Boundary split

| Class | IDs | Research focus |
|-------|-----|----------------|
| Ops gate | AO, AT | Compose/runbook; no sprint code until sidecar healthy |
| Code gaps | AP, AQ, AS | Architecture contracts + execute slices |
| Data/account | AR | Verify acct **114** + sync before panel SQL changes |

### Operator gates (mandatory before execute)

1. **BACKEND_FRONTEND_DEPLOY** — Q0020 / `DEC-0080` on rebuilt host.
2. **`stats-forecast` up** when ML enabled.
3. **Full Firefly sync** + **forecast recompute** on acct **114**.

### Risks

- Starting sidecar alone does not fix **AP** if wallet `market_value_eur` still NULL — verify image tag first.
- **AQ** native-currency display may need new decision — bound under research, not scope creep into US-0018.
- **AS** active-plan delete may need cascade rule — research before execute.
- Grafana `forecast-horizons.json` static text cites **US-0013** even when env opts in — misleading alongside React `sidecar_unavailable` copy.

### Research pointers (extend R-0079)

- Wallet row persistence after AN1 — SQL probe on `exchange_holdings` for `product_type=futures`.
- Subtotal vs unrealized display contract when linear excluded from subtotal (**DEC-0064** preserved).
- Active-plan delete semantics (`is_active` guard).
- Cashflow API vs Grafana divergence test for acct **114**.

### Artifacts updated

- `docs/product/backlog.md#BUG-0014`, `docs/product/vision.md`, `handoffs/resume_brief.md`, `docs/engineering/state.md`

**Evidence:** [R-0079](docs/engineering/research.md#r-0079--bug-0014-post-rebuild-omniflow-ml-sidecar-crypto-display-grafana-planning), `handoffs/intake_evidence/intake-20260607-post-rebuild-omniflow.json`

---

