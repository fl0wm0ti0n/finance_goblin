# Tasks — Q0022 (BUG-0014)

**Bug:** BUG-0014  
**Task count:** 8 (5 mandatory + 1 P1 optional + 2 conditional; < `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260610-q0022-bug0014`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **AO1** | Task **AO1** | Panel id 13 dual-scenario markdown per DEC-0066/DEC-0076 |
| **AQ1** | Task **AQ1** | DEC-0081 holdings_all + wire unpriced_assets / fx_incomplete |
| **AQ2** | Task **AQ2** | WealthPage table + unified FX banner |
| **AS1** | Task **AS1** | DEC-0082 delete + 409 active guard |
| **AS2** | Task **AS2** | DEC-0083 target_type UI — P1 optional |
| **AP2** | Task **AP2** | Conditional — AP1 SQL gate: priced futures + subtotal 0 |
| **AR1** | Task **AR1** | Conditional — V1 AR gate: API≠Grafana |
| **V1** | Task **V1** | verify-work omniflow smoke AO–AT |
| **AP1** | **Ops gate** | Operator SQL probe — not sprint task |
| **AO/AT runtime** | **Ops-only** | Three-service compose + sidecar start |

## Execute order

```text
AO1 ∥ AQ1 ∥ AS1 (parallel — independent layers)
  → AQ2 (after AQ1)
  → optional AS2
  → single release deploy
  → operator: BACKEND_FRONTEND_DEPLOY + THREE_SERVICE_COMPOSE + FULL_FIREFLY_SYNC
  → operator: AP1 SQL probe → AP2 if gate passes
  → operator: GRAFANA_PROVISIONING_RELOAD
  → AR1 only if V1 AR verify gate fails (may defer to follow-up)
  → V1 verify-work
```

**Parallelism:** AO1 (Grafana), AQ1 (backend wealth), AS1 (plans) may proceed in parallel; AQ2 blocked on AQ1; V1 blocked on deploy + operator gates.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **AO** | AO1, V1 | ML overlay or accurate sidecar-unreachable guidance |
| **AP** | AP2, V1 | Non-zero crypto subtotal when wallet priced |
| **AQ** | AQ1, AQ2, V1 | Native qty + EUR; unified FX incomplete |
| **AR** | AR1, V1 | Cashflow acct 114 non-zero; AR1 conditional |
| **AS** | AS1, AS2, V1 | Plan delete + target_type UX |
| **AT** | V1 | Three-service compose smoke |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| AO1 | Grafana panel 13 dual-scenario ML copy | 1h | open | **AO** | P0 |
| AQ1 | holdings_all + unpriced_assets / fx_incomplete | 3h | open | **AQ** | P0 |
| AQ2 | WealthPage native+EUR + FX banner | 2h | open | **AQ** | P0 |
| AS1 | Delete plan + active 409 guard | 2.5h | open | **AS** | P0 |
| AS2 | target_type select + help | 1h | open | **AS** | P1 optional |
| AP2 | Defensive subtotal + count annotation | 1.5h | open | **AP** | P0 conditional |
| AR1 | Cashflow Grafana variable fix | 1.5h | open | **AR** | P2 conditional |
| V1 | verify-work omniflow smoke | 2h | open | **AO**–**AT** | P0 |

---

## AO1 — Grafana forecast-horizons panel 13 dual-scenario ML copy

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0014 **AO**

### Description

Replace panel id **13** static markdown in `forecast-horizons.json` with **dual-scenario** copy per architecture:

1. ML not configured — set `FORECAST_ML_ENABLED` + US-0013 runbook link
2. ML configured but sidecar unreachable — start `stats-forecast` per DEC-0076 three-service compose

Reject option B (Postgres `ml_skipped_reason` variable) this sprint.

**Files:** `grafana/provisioning/dashboards/analytics/forecast-horizons.json` panel **13** only.

### Done when

- [ ] Panel distinguishes not-configured vs sidecar-unreachable scenarios
- [ ] US-0013 runbook link on not-configured path
- [ ] `stats-forecast` service name on unreachable path
- [ ] No React `ForecastPage.tsx` change (DEC-0066 already handles API path)

---

## AQ1 — Wealth holdings_all + unpriced_assets + fx_incomplete

**Status:** open  
**Depends on:** —  
**Estimate:** 3h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0014 **AQ** — **DEC-0081**

### Description

Implement DEC-0081 backend contract:

- Add `holdings_all` (cap 50, priced first by `value_eur` desc)
- Wire `unpriced_assets` from PnL breakdown
- `fx_incomplete = pnl.fx_incomplete || !unpriced_assets.is_empty()`
- Retain `holdings_top` as priced top-5 subset

**Files:** `backend/src/wealth/service.rs`, `backend/src/wealth/types.rs`

### Done when

- [ ] `holdings_all` fields: asset, quantity, product_type, value_eur, unrealized_pnl_eur, native_unit
- [ ] Linear rows visible with unrealized EUR; excluded from subtotal (DEC-0064)
- [ ] `fx_incomplete` and `unpriced_assets` match PnL output
- [ ] `cargo test` wealth paths PASS

---

## AQ2 — WealthPage native qty + EUR display + unified FX banner

**Status:** open  
**Depends on:** AQ1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0014 **AQ**

### Description

Update WealthPage crypto tab to render `holdings_all` with native quantity and EUR columns; unify FX incomplete banner with API `fx_incomplete` and `unpriced_assets` list.

**Files:** `frontend/src/pages/WealthPage.tsx`, `frontend/src/lib/api.ts`

### Done when

- [ ] Table shows all holdings (not holdings_top only)
- [ ] Native unit and EUR columns; `—` when unpriced
- [ ] Banner matches API flag and lists unpriced assets
- [ ] Portfolio forecast warning uses same gate

---

## AS1 — PlanningPage delete plan + active-plan 409 guard

**Status:** open  
**Depends on:** —  
**Estimate:** 2.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0014 **AS** — **DEC-0082**

### Description

Backend: return **409** `active_plan_delete_forbidden` when deleting `is_active` plan. Frontend: delete control with confirm modal; disable active plan delete with tooltip; DEC-0077 error surface.

**Files:** `backend/src/api/plans.rs` or `plan/service.rs`, `frontend/src/pages/PlanningPage.tsx`

### Done when

- [ ] DELETE non-active plan succeeds
- [ ] DELETE active plan returns 409 with structured error
- [ ] UI confirm modal; active plan delete disabled
- [ ] Plans query invalidated after delete

---

## AS2 — Planning adjustment target_type select + help copy (optional P1)

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0014 **AS** — **DEC-0083**  
**Priority:** P1 optional

### Description

Remove invalid `account` option; expose `household`, `subscription`, `category`, `custom_label`, `allocation_target` with inline help copy.

**Files:** `frontend/src/pages/PlanningPage.tsx`, `frontend/src/lib/api.ts`

### Done when

- [ ] No `account` in select or API types
- [ ] Five valid enum values with help paragraph
- [ ] Existing saved adjustments unaffected

---

## AP2 — Defensive crypto subtotal + holdings count annotation (conditional)

**Status:** open  
**Depends on:** AQ1, **AP1 operator gate**  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0014 **AP**  
**Gate:** AP1 SQL — futures row priced AND API `subtotal_eur = 0`

### Description

Per DEC-0080 extension: if `sum(market_value_eur) == 0` AND `pnl.crypto_value_eur > 0`, use portfolio snapshot for subtotal display. Annotate exchange card holdings count (e.g. "N holdings (M priced)"). Log defensive fallback. **Do not** merge linear notional into subtotal.

**Files:** `backend/src/wealth/service.rs` only.

### Done when

- [ ] AP1 gate documented and passed before execute
- [ ] Defensive subtotal with logging
- [ ] Count annotation when priced/unpriced diverge
- [ ] DEC-0064 subtotal rules preserved

---

## AR1 — Cashflow Grafana variable fix (conditional P2)

**Status:** open  
**Depends on:** V1 AR verify gate  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0014 **AR**  
**Trigger:** API `forecast/daily` acct 114 non-zero AND cashflow panel SQL zero

### Description

Fix Grafana variable/computation mismatch in `cashflow.json` only when V1 proves API≠Grafana divergence. Default path is verify-only.

**Files:** `grafana/provisioning/dashboards/analytics/cashflow.json`

### Done when

- [ ] V1 AR gate failure documented before execute
- [ ] Panel id 1 returns rows matching API for acct 114
- [ ] `$account_id` default 114; time range aligned with verify steps

---

## V1 — verify-work omniflow smoke AO–AT

**Status:** open  
**Depends on:** AO1, AQ1, AQ2, AS1 deploy + operator gates  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0014 **AO**–**AT**

### Description

Prepare `sprints/quick/Q0022/uat.md` smoke checklist. After deploy + gates, probe `financegnome.omniflow.cc`:

- **AO:** `forecast/meta` ML status; `/analytics/forecast-horizons` banner
- **AP:** `GET /api/v1/wealth` subtotal + holdings_all
- **AQ:** Wealth crypto tab native+EUR; FX banner
- **AR:** `/analytics/cashflow` acct 114; API daily forecast compare
- **AS:** `/planning` delete flow + target_type
- **AT:** `docker ps` three-service compose
- Six `/analytics/{slug}` routes regression

### Done when

- [ ] Rows **AO**–**AT** probed per acceptance.md
- [ ] AR1 trigger evaluated (execute or waive)
- [ ] AP2 trigger evaluated via AP1 SQL
- [ ] Operator gates documented in uat.md

**Operator gates:** BACKEND_FRONTEND_DEPLOY + THREE_SERVICE_COMPOSE + FULL_FIREFLY_SYNC + GRAFANA_PROVISIONING_RELOAD + AP1_SQL_PROBE before runtime probes.
