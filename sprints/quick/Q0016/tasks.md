# Tasks — Q0016 (BUG-0009)

**Bug:** BUG-0009  
**Task count:** 6 (within `SPRINT_MAX_TASKS=12`)  
**Sprint-plan ref:** `sprint-plan-20260606-q0016-bug0009`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **Z1** | Task **Z1** | Latest-snapshot subquery + `LATERAL jsonb_array_elements`; remove global `LIMIT 1` |
| **Z2** | Task **Z2** | Overview table title + grid placement on portfolio dashboard only |
| **Y1** | Task **Y1** | `ORDER BY ABS(COALESCE(balance,0)) DESC, name`; omit `current` |
| **Y2** | Task **Y2** | Text banner + `noValue: "ML unavailable"`; no dynamic hide |
| **T1** | Task **T1** | 3-account breakdown fixture; variable order fixture; optional JSON snapshot |
| **V1** | Task **V1** | verify-work smoke rows Y/Z; six analytics routes; runbook `current` warning |

## Execute order

```text
Z1 → Z2 → Y1 → Y2
  → T1 (parallel with Z1/Y1 once contracts clear)
  → single PR deploy + Grafana provisioning reload
  → V1 verify-work
```

**Parallelism:** Y1 and Y2 independent after Z1/Z2; T1 may start once Z1 and Y1 SQL contracts are frozen.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **(Y)** | Y1, Y2, T1, V1 | Default-load cashflow non-flat series; ML banner honest; ds/query 200 |
| **(Z)** | Z1, Z2, T1, V1 | Portfolio overview 3 rows; stat row visible; six routes smoke |

| ID | Title | Est. | Status | Acceptance |
|----|-------|------|--------|------------|
| Z1 | Portfolio breakdown SQL subquery + LATERAL | 1h | done | **(Z)** |
| Z2 | Cross-account overview table + grid layout | 1.5h | done | **(Z)** |
| Y1 | `$account_id` ABS(balance) variable query | 1h | done | **(Y)** |
| Y2 | ML banner + noValue on ML panels | 1h | done | **(Y)** |
| T1 | SQL fixtures + provisioning snapshot test | 1.5h | done | **(Y)**, **(Z)** |
| V1 | verify-work omniflow smoke | 1h | prep | **(Y)**, **(Z)** |

---

## Z1 — Portfolio breakdown SQL subquery + LATERAL

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0009 **(Z)**

### Description

Fix panel id 5 in `portfolio.json`: replace global `LIMIT 1` on cross-join with latest-snapshot subquery + `CROSS JOIN LATERAL jsonb_array_elements(payload->'accounts')`. Sort by `ABS(balance) DESC`.

**Files:** `grafana/provisioning/dashboards/analytics/portfolio.json`

### Done when

- [ ] Breakdown query returns all accounts from latest snapshot (not 1 of N)
- [ ] SQL matches DEC-0068 frozen contract
- [ ] Z2 reuses same SQL pattern

---

## Z2 — Cross-account overview table + grid layout

**Status:** open  
**Depends on:** Z1  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0009 **(Z)**

### Description

Upgrade portfolio dashboard for AC Z: verify stat row (`total_eur`, `account_count`, mixed-currency warning) visible above fold; retitle panel id 5 to "All accounts (latest snapshot)"; reposition grid so overview table sits immediately below stat row. Optional supplementary text panel linking `/wealth` (Z3 — not AC substitute).

**Files:** `grafana/provisioning/dashboards/analytics/portfolio.json`

### Done when

- [ ] Overview table uses Z1 SQL; columns: name, role, currency, balance
- [ ] Stat row visible in kiosk embed
- [ ] Portfolio dashboard only — no seventh dashboard

---

## Y1 — `$account_id` ABS(balance) variable query

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0009 **(Y)**

### Description

Update `$account_id` templating query in `cashflow.json` and `forecast-horizons.json`: `ORDER BY ABS(COALESCE(a.balance, 0)) DESC, a.name ASC`. Remove any saved `current` block from provisioning JSON.

**Files:** `grafana/provisioning/dashboards/analytics/cashflow.json`, `forecast-horizons.json`

### Done when

- [ ] Both dashboards use ABS(balance) sort
- [ ] No `current` block in provisioning JSON
- [ ] `refresh: 1` preserved on variable

---

## Y2 — ML banner + noValue on ML panels

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0009 **(Y)**

### Description

Add text panel above ML section in `forecast-horizons.json` explaining ML not enabled (US-0013 boundary). Set `fieldConfig.defaults.noValue` → `"ML unavailable"` on ML time-series and stat panels. No dynamic hide rules; `$forecast_variant` unchanged.

**Files:** `grafana/provisioning/dashboards/analytics/forecast-horizons.json`

### Done when

- [ ] Banner copy aligned with DEC-0066 posture
- [ ] ML panels show `noValue` when `ml_enhanced` absent
- [ ] No US-0013 sidecar work in scope

---

## T1 — SQL fixtures + provisioning snapshot test

**Status:** open  
**Depends on:** Z1, Y1  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0009 **(Y)**, **(Z)**

### Description

Frozen test contract from architecture § Test strategy: 3-account snapshot JSON → breakdown query returns **3 rows**; variable query ABS sort picks funded account (114) over zero wallet (116); optional provisioning JSON snapshot asserts no `current` block and ML banner present.

**Files:** test fixtures under repo test tree; optional snapshot of `grafana/provisioning/dashboards/analytics/*.json`

### Done when

- [ ] SQL fixture: breakdown 3-row pass
- [ ] SQL fixture: variable order picks funded over zero wallet
- [ ] Optional JSON snapshot for Y2 banner + Y1 query string
- [ ] CI or `cargo test` / documented test runner PASS

---

## V1 — verify-work omniflow smoke

**Status:** open  
**Depends on:** Z1, Z2, Y1, Y2, T1 deploy + Grafana provisioning reload  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0009 **(Y)**, **(Z)**

### Description

Prepare `sprints/quick/Q0016/uat.md` smoke checklist. After deploy + provisioning reload, probe `financegnome.omniflow.cc`: `/analytics/cashflow` default load → non-flat series; `/analytics/portfolio` → overview 3 rows + `total_eur` stat; `/analytics/forecast` → ML banner visible; six `/analytics/{slug}` routes; `POST /analytics/grafana/api/ds/query` **200** regression (BUG-0003 H). Document runbook warning: do not save variables in Grafana UI (bakes `current`).

**Files:** `sprints/quick/Q0016/uat.md`, `docs/engineering/runbook.md` (optional `current` warning)

### Done when

- [ ] Row **(Y)**: cashflow/forecast default-load non-empty; ML banner honest
- [ ] Row **(Z)**: portfolio overview 3 account rows; cross-account value visible
- [ ] Six analytics routes smoke PASS
- [ ] ds/query 200 regression documented
- [ ] `/wealth` documented as supplementary (Z3)

**Operator gate:** Grafana provisioning reload after Z1–Y2+T1 deploy before V1 runtime probes.
