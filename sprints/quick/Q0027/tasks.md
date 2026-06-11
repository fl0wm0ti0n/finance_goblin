# Tasks — Q0027 (BUG-0019)

**Bug:** BUG-0019  
**Task count:** 6 (all P0 mandatory; 6/12 under `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260610-q0027-bug0019`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **CA-1** | Task **CA1** | cashflow `$account_id` `sort: 0` + `current`; version bump |
| **CA-2** | Task **CA2** | cashflow panels 1–3 `AND model_kind = 'baseline'` |
| **CA-3** | Task **CA3** | forecast-horizons `$account_id` `sort: 0` + `current`; version bump |
| **CB-1** | Task **CB1** | platform-health panel 2 mirror COUNT(*) UNION ALL; version bump |
| Static gate | Task **G1** | jq assertions per DEC-0108 verification gates |
| BG/BH runtime | Task **V1** | verify-work after GRAFANA_PROVISIONING_RELOAD |

## Execute order

```text
(CA1 → CA2) ∥ CA3 ∥ CB1
  → G1 static guard
  → operator: GRAFANA_PROVISIONING_RELOAD (docker compose restart grafana)
  → V1 verify-work
```

**Parallelism:** CA3 and CB1 are disjoint files — may run parallel to CA1/CA2.
CA2 follows CA1 (same file, single coherent edit acceptable). G1 after all JSON
edits; V1 blocked on Grafana reload.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **BG** | CA1, CA2, CA3, G1, V1 | Default account = 114 (highest \|balance\|); cashflow panels 1–2 non-zero matching `GET /api/v1/forecast/monthly?account_id=114`; kiosk embed + direct Grafana |
| **BH** | CB1, G1, V1 | Panel 2 `transactions` = 922 mirror count after Full sync **and** after 0-new-tx incremental rerun |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| CA1 | cashflow.json `$account_id` `sort: 0` + `current` | 0.5h | open | **BG** | P0 |
| CA2 | cashflow.json panels 1–3 `model_kind = 'baseline'` | 0.5h | open | **BG** | P0 |
| CA3 | forecast-horizons.json `$account_id` `sort: 0` + `current` | 0.5h | open | **BG** | P0 |
| CB1 | platform-health.json panel 2 mirror COUNT(*) | 1h | open | **BH** | P0 |
| G1 | Static JSON guard | 0.5h | open | **BG**, **BH** | P0 |
| V1 | verify-work Grafana re-provision smoke | 1.5h | open | **BG**, **BH** | P0 |

---

## CA1 — cashflow.json `$account_id` `sort: 0` + `current` default

**Status:** open  
**Depends on:** —  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0019 **BG** — **DEC-0108** CA variable fix

### Description

In `grafana/provisioning/dashboards/analytics/cashflow.json`:

1. `$account_id` template variable: change `"sort": 1` → `"sort": 0` so the SQL
   `ORDER BY ABS(COALESCE(a.balance,0)) DESC` order is preserved (funded account
   first — no alphabetical re-sort).
2. Add a `"current"` block defaulting to the first (highest-|balance|) option so
   the provisioned default is deterministic. Mirror the existing
   `forecast_variant` `current` structure in `forecast-horizons.json` (DEC-0108
   risk note: wrong `current` shape → variable shows "None").
3. Bump the dashboard `version` field (cache-bust on re-provision).

**Forbidden:** hardcoding account id `114` in the variable query or `current`
value semantics beyond the provisioned snapshot default; variable SQL changes.

**Files:** `grafana/provisioning/dashboards/analytics/cashflow.json`

### Done when

- [ ] `account_id` variable has `"sort": 0`
- [ ] `current` present with valid Grafana shape (text/value)
- [ ] Dashboard `version` bumped
- [ ] JSON valid (`jq . cashflow.json` exits 0)

### Verification

`jq '.templating.list[] | select(.name=="account_id") | .sort' cashflow.json` → `0`;
`jq '.templating.list[] | select(.name=="account_id") | .current' cashflow.json` → non-null.

---

## CA2 — cashflow.json panels 1–3 `model_kind = 'baseline'` subquery

**Status:** open  
**Depends on:** CA1 (same file)  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0019 **BG** — **DEC-0108** CA subquery fix

### Description

In `cashflow.json` panels 1–3, the latest-success computation subquery currently
selects the latest success **without** a `model_kind` filter. Add
`AND model_kind = 'baseline'` to each latest-success subquery to align panel
computation choice with the API default (R-0089 §3) and prevent divergence the
day `ml_enhanced` succeeds.

`forecast-horizons.json` already filters via `$forecast_variant`
(current=Baseline) — **unchanged**.

**Files:** `grafana/provisioning/dashboards/analytics/cashflow.json` (panels 1–3 `rawSql` only)

### Done when

- [ ] All three latest-success subqueries contain `AND model_kind = 'baseline'`
- [ ] No other panel SQL semantics changed
- [ ] JSON valid

### Verification

`jq -r '.panels[0:3][].targets[].rawSql' cashflow.json | grep -c "model_kind = 'baseline'"` → 3 (one per panel).

---

## CA3 — forecast-horizons.json `$account_id` `sort: 0` + `current`

**Status:** open  
**Depends on:** — (disjoint file; may parallel CA1)  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0019 **BG** — **DEC-0108** CA variable fix (latent twin)

### Description

Same defect as CA1 in
`grafana/provisioning/dashboards/analytics/forecast-horizons.json`: `$account_id`
variable `"sort": 1` → `"sort": 0`; add `"current"` defaulting to the first
option (mirror the file's own `forecast_variant` `current` structure); bump
dashboard `version`. Panels already filter `$forecast_variant` — no panel SQL
changes.

**Files:** `grafana/provisioning/dashboards/analytics/forecast-horizons.json`

### Done when

- [ ] `account_id` variable has `"sort": 0`
- [ ] `current` present with valid shape
- [ ] Dashboard `version` bumped
- [ ] JSON valid; no panel `rawSql` changed

### Verification

`jq '.templating.list[] | select(.name=="account_id") | .sort, .current' forecast-horizons.json`.

---

## CB1 — platform-health.json panel 2 mirror COUNT(*) UNION ALL

**Status:** open  
**Depends on:** — (disjoint file)  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0019 **BH** — **DEC-0108** CB panel fix

### Description

Replace `grafana/provisioning/dashboards/platform-health.json` panel 2 ("Records
synced per entity") `rawSql` — currently reading per-run
`sync_cursors.records_synced` — with the DEC-0108 mirror-count UNION ALL:

```sql
SELECT e.entity_type,
       e.records_total,
       c.last_successful_sync_at
FROM (
  SELECT 'transactions' AS entity_type, COUNT(*) AS records_total FROM transactions
  UNION ALL SELECT 'accounts',    COUNT(*) FROM accounts
  UNION ALL SELECT 'categories',  COUNT(*) FROM categories
  UNION ALL SELECT 'budgets',     COUNT(*) FROM budgets
  UNION ALL SELECT 'tags',        COUNT(*) FROM tags
  UNION ALL SELECT 'piggy_banks', COUNT(*) FROM piggy_banks
) e
LEFT JOIN sync_cursors c ON c.entity_type = e.entity_type
ORDER BY e.entity_type
```

Entity list = the six `upsert_cursor` call sites in `backend/src/firefly/mod.rs`;
mirror table names equal entity_type strings. A `records_synced` column may be
retained **only if** relabeled "records last run". Bump dashboard `version`.

**Forbidden:** edits to `upsert_cursor` / sync semantics; cursor watermark
bookkeeping untouched.

**Files:** `grafana/provisioning/dashboards/platform-health.json` (panel 2 + version)

### Done when

- [ ] Panel 2 SQL counts the six mirror tables (UNION ALL) joined with `sync_cursors`
- [ ] No bare `records_synced` read as the count column (relabel-only exception per DEC-0108)
- [ ] Dashboard `version` bumped
- [ ] JSON valid

### Verification

`jq -r '.panels[1].targets[].rawSql' platform-health.json` references all six
mirror tables and `LEFT JOIN sync_cursors`.

---

## G1 — Static JSON guard

**Status:** open  
**Depends on:** CA1, CA2, CA3, CB1  
**Estimate:** 0.5h  
**Acceptance hook:** DEC-0108 verification gates — static guard for **BG**, **BH**

### Description

Run and record the DEC-0108 static assertions over the three edited dashboards
(review checklist; one-shot jq commands in `progress.md` or a throwaway script —
no CI change required):

1. `cashflow.json` + `forecast-horizons.json`: `account_id` variable `sort == 0`
   and `current` non-null.
2. `cashflow.json` panels 1–3 `rawSql` contain `model_kind = 'baseline'`.
3. `platform-health.json` panel 2 `rawSql` references the six mirror tables and
   not bare `records_synced` as count source.
4. All three files parse (`jq . file` exit 0) and each `version` is bumped vs git HEAD~.

**Files:** read-only over the three dashboard JSONs; results recorded in `sprints/quick/Q0027/progress.md`

### Done when

- [ ] All four assertion groups PASS, recorded in progress.md
- [ ] No assertion requires a backend/frontend file — provisioning-only blast radius confirmed

### Verification

jq command outputs pasted in progress.md; `git diff --stat` shows only the three JSON files.

---

## V1 — verify-work Grafana re-provision smoke

**Status:** open  
**Depends on:** G1 + operator GRAFANA_PROVISIONING_RELOAD  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0019 **BG**, **BH**

### Description

Prepare `sprints/quick/Q0027/uat.md` smoke checklist. After
**GRAFANA_PROVISIONING_RELOAD** (`docker compose restart grafana`):

1. **BG default** — open Cashflow via kiosk embed (`AnalyticsEmbedPage`) **and**
   direct Grafana URL (covers no-`var-account_id` path): default account = 114
   (highest |balance|); panels 1–2 show non-zero (negative) series.
2. **BG oracle** — compare `GET /api/v1/forecast/monthly?account_id=114`
   (25 points, non-zero from Jul 2026); optional DB probe per R-0089 (latest
   baseline success rows for 114 non-zero).
3. **BG twin** — Forecast Horizons dashboard default account = 114.
4. **BH full** — after Full sync: Platform Health panel 2 `transactions` =
   `SELECT COUNT(*) FROM transactions` (922).
5. **BH regression** — trigger a subsequent **incremental sync with 0 new
   transactions**; panel 2 `transactions` still = mirror count (the exact
   regression scenario).
6. **OIDC** — re-run BG/BH on omniflow profile (provisioning-only blast radius;
   no backend image change).

**Files:** `sprints/quick/Q0027/uat.md`, `sprints/quick/Q0027/uat.json`

### Done when

- [ ] Rows **BG**, **BH** probed per acceptance.md matrix (kiosk + direct Grafana)
- [ ] BH proven after Full sync **and** after 0-new-tx incremental rerun
- [ ] `uat.md` and `uat.json` populated with results
- [ ] Operator gate documented: **GRAFANA_PROVISIONING_RELOAD**

**Operator gates:** **GRAFANA_PROVISIONING_RELOAD** (`docker compose restart grafana`)
before runtime probes; Full sync + incremental rerun for BH.
