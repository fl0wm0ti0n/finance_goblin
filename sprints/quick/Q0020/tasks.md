# Tasks — Q0020 (BUG-0013)

**Bug:** BUG-0013  
**Task count:** 5 (3 mandatory + 2 optional P2; < `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260608-q0020-bug0013`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **AL1** | Task **AL1** | Panel id 5 planned CTE `pdc.ts::date <= CURRENT_DATE` per DEC-0079 |
| **AN1a** | Task **AN1** | `bitunix.rs` wallet `data[]` array parse + `unrealizedPNL` keys |
| **AN1b** | Task **AN1** | `pnl.rs` futures USDT equity + linear unrealized USDT→EUR |
| **AN1c** | Task **AN1** | Unit tests: array wallet mock + linear unrealized persist |
| **AJ1** | Task **AJ1** | Optional subscriptions empty-state copy |
| **AK2** | Task **AK2** | Optional portfolio performance min-snapshot footnote |
| **V1** | Task **V1** | verify-work omniflow smoke AI–AN |
| **AM1** | **Waived** | R-0077 — no execute unless HAR non-200 |

## Execute order

```text
AL1 ∥ AN1 (parallel — independent layers)
  → optional AJ1, AK2 (provisioning copy)
  → single release deploy
  → operator: BACKEND_FRONTEND_DEPLOY + GRAFANA_PROVISIONING_RELOAD
  → operator: Full Firefly sync + forecast recompute
  → V1 verify-work
```

**Parallelism:** AL1 (Grafana) and AN1 (backend) may proceed in parallel; V1 blocked on deploy + operator gates.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **AI** | V1 | Re-smoke cashflow/forecast baseline acct 114 — ops only |
| **AJ** | AJ1, V1 | Documented empty-state when 0 price-change events |
| **AK** | AN1, AK2, V1 | Crypto subtotal > 0; performance % footnote if <2 snapshots |
| **AL** | AL1, V1 | MTD capped planned — not 730-day sum |
| **AM** | V1 | ds/query 200 regression — waived R-0077 |
| **AN** | AN1, V1 | Wealth/portfolio crypto totals after sync |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| AL1 | MTD planned `<= CURRENT_DATE` + optional footnote | 1h | open | **AL** | P0 |
| AN1 | Wallet parse + linear unrealized EUR + tests | 4h | open | **AN**, **AK** | P0 |
| AJ1 | Price-changes empty-state copy | 0.5h | open | **AJ** | P2 optional |
| AK2 | Performance % min-snapshot footnote | 0.5h | open | **AK** | P2 optional |
| V1 | verify-work omniflow smoke | 2h | open | **AI**–**AN** | P0 |

---

## AL1 — Budgets MTD upper date bound

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0013 **AL**

### Description

Fix panel id **5** in `budgets.json`: add `AND pdc.ts::date <= CURRENT_DATE` to `planned` CTE per **DEC-0079**. Deviation row uses capped planned total. Optional text footnote when plan horizon starts mid-month.

**Files:** `grafana/provisioning/dashboards/analytics/budgets.json`

### Done when

- [ ] Planned CTE caps at `CURRENT_DATE` (not full 730-day horizon)
- [ ] Deviation row reflects capped planned minus actual
- [ ] Optional mid-month plan-start footnote if plan begins after month start
- [ ] SQL matches DEC-0079 frozen contract

---

## AN1 — Bitunix wallet parse + linear unrealized EUR

**Status:** open  
**Depends on:** —  
**Estimate:** 4h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0013 **AN**, **AK**

### Description

Implement **DEC-0080** in three sub-slices (single task):

1. **AN1a** — `bitunix.rs`: parse wallet `data` as array (first object with marginCoin/available); add `unrealizedPNL` to position/wallet `parse_f64_field` keys.
2. **AN1b** — `pnl.rs`: futures wallet EUR via stablecoin path; linear unrealized USDT→EUR; skip linear notional in `crypto_value_eur` sum (DEC-0064).
3. **AN1c** — Tests: mock array-shaped wallet response; linear unrealized persist assertion.

**Files:** `backend/src/exchanges/bitunix.rs`, `backend/src/portfolio/pnl.rs`

### Done when

- [ ] Array wallet mock produces futures wallet row with USDT equity
- [ ] `GET /api/v1/wealth` `crypto.subtotal_eur` > 0 when wallet equity > 0
- [ ] Linear rows get `unrealized_pnl_eur` from payload; not double-counted in subtotal
- [ ] `cargo test` for bitunix/pnl paths PASS

---

## AJ1 — Subscriptions price-changes empty-state copy (optional)

**Status:** open  
**Depends on:** —  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0013 **AJ**  
**Priority:** P2 optional — skip if sprint capacity tight

### Description

Add honest empty-state copy to **Price changes (90 days)** panel in `subscriptions.json` when 0 `price_increase`/`price_decrease` events — discovery confirmed expected empty, not defect.

**Files:** `grafana/provisioning/dashboards/analytics/subscriptions.json`

### Done when

- [ ] Panel shows documented empty-state message when no price-change events
- [ ] No SQL logic change required

---

## AK2 — Portfolio performance % min-snapshot footnote (optional)

**Status:** open  
**Depends on:** —  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0013 **AK**  
**Priority:** P2 optional

### Description

Document min snapshot count requirement on portfolio performance % panel — may remain NULL until ≥2 snapshots even after AN1 fixes crypto pricing.

**Files:** `grafana/provisioning/dashboards/analytics/portfolio.json`

### Done when

- [ ] Footnote or `noValue` explains insufficient snapshot history
- [ ] Does not fabricate return % without baseline

---

## V1 — verify-work omniflow smoke

**Status:** open  
**Depends on:** AL1, AN1 deploy + operator gates  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0013 **AI**–**AN**

### Description

Prepare `sprints/quick/Q0020/uat.md` smoke checklist. After deploy + Full sync + recompute, probe `financegnome.omniflow.cc`:

- **AL:** `/analytics/budgets` MTD planned/actual/deviation plausible (not −€150K)
- **AN/AK:** `/api/v1/wealth` crypto subtotal > 0; portfolio crypto stat non-zero
- **AI:** cashflow scarcity + forecast-horizons baseline acct 114 non-empty
- **AJ:** subscriptions price-changes empty-state or rows
- **AM:** `POST /analytics/grafana/api/ds/query` + annotations **200** regression
- Six `/analytics/{slug}` routes smoke

**Files:** `sprints/quick/Q0020/uat.md`, `docs/engineering/runbook.md` (optional operator gate notes)

### Done when

- [ ] Row **AL**: MTD capped — plausible values
- [ ] Row **AN/AK**: crypto subtotal and portfolio stat reflect priced holdings
- [ ] Row **AI**: baseline panels non-empty acct 114
- [ ] Row **AJ**: documented empty-state or event rows
- [ ] Row **AM**: ds/query 200 (waived unless HAR shows failure)
- [ ] Operator gates documented: deploy, Full sync, Grafana reload

**Operator gates:** BACKEND_FRONTEND_DEPLOY + GRAFANA_PROVISIONING_RELOAD + Full Firefly sync + forecast recompute before runtime probes.
