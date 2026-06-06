# Tasks — Q0011 (BUG-0004)

**Bug:** BUG-0004  
**Task count:** 7 (within `SPRINT_MAX_TASKS=12`)  
**Sprint-plan ref:** `sprint-plan-20260605-q0011-bug0004`

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **(I)** | I1 | Exchange sync terminal status; `finished_at` set |
| **(K)** | K1 | Portfolio ds/query **200**; no UNION syntax error |
| **(L)** | L1, L2, L3 | Wealth + forecast non-empty after Full sync backfill |
| **(J)** | J1, J2 | Payee grouping + pending/threshold UX |

| ID | Title | Est. | Status | Acceptance |
|----|-------|------|--------|------------|
| I1 | finish_sync_run on ExchangesOnly | 1.5h | open | **(I)** |
| K1 | Portfolio pie UNION SQL fix | 1h | open | **(K)** |
| L1 | Account balance parse (DEC-0060) | 1.5h | open | **(L)** |
| L2 | Wealth NULL balance filter | 1h | open | **(L)** |
| J1 | Payee key fallbacks (DEC-0061) | 2h | open | **(J)** |
| J2 | Subscriptions empty-state UX | 1.5h | open | **(J)** |
| L3 | verify-work omniflow probes | 1h | open | **(I–L)** |

---

## I1 — finish_sync_run on ExchangesOnly

**Status:** open  
**Depends on:** —  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0004 **(I)**

### Description

Call `finish_sync_run(success|failed)` when `RunMode::ExchangesOnly` completes `run_exchanges_and_alerts`. Mirror Full-path error handling; do not double-finish Full runs.

**Files:** `backend/src/sync/mod.rs`

### Done when

- [ ] Unit: ExchangesOnly success → `finish_sync_run(..., "success", ...)`
- [ ] Unit: ExchangesOnly error → `finish_sync_run(..., "failed", ...)`
- [ ] `cargo test --lib` sync PASS

---

## K1 — Portfolio pie UNION SQL fix

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0004 **(K)**

### Description

Wrap each `ORDER BY ... LIMIT 1` branch in parentheses before `UNION ALL` in portfolio panel id 8 raw SQL.

**Files:** `grafana/provisioning/dashboards/analytics/portfolio.json`

### Done when

- [ ] SQL fixture or manual probe: no `syntax error at or near "UNION"`
- [ ] Grafana ds/query portfolio pie → **200**

---

## L1 — Account balance parse (DEC-0060)

**Status:** open  
**Depends on:** —  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0004 **(L)**

### Description

Use `parse_split_amount` for Firefly `attributes.current_balance` in `sync_accounts` instead of `.as_f64()` only.

**Files:** `backend/src/firefly/mod.rs`

### Done when

- [ ] Unit: string `"1234.56"` and number → persisted balance
- [ ] Re-sync backfill documented (DEC-0002 upsert — no migration)

---

## L2 — Wealth NULL balance filter

**Status:** open  
**Depends on:** L1  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0004 **(L)**

### Description

Change `load_asset_accounts` filter from `balance >= 0` to `COALESCE(balance, 0) >= 0`.

**Files:** `backend/src/wealth/repository.rs`

### Done when

- [ ] Unit/integration: NULL balance asset row included in query result
- [ ] `cargo test --lib` wealth PASS

---

## J1 — Payee key fallbacks (DEC-0061)

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0004 **(J)**

### Description

Add `extract_payee_source` priority chain: description → first split `counterparty_name` → `destination_name`; apply `payee_key()` in `by_payee()`.

**Files:** `backend/src/recurrence/group.rs`

### Done when

- [ ] Unit: counterparty-only fixture → non-empty payee group key
- [ ] Unit: generic description skipped when counterparty present
- [ ] `cargo test --lib` recurrence PASS

---

## J2 — Subscriptions empty-state UX

**Status:** open  
**Depends on:** J1  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0004 **(J)**

### Description

Enhance `SubscriptionsPage` empty states: document detection thresholds (≥3 txs, ≥60% confidence, Full sync required, DEC-0061 payee keys); show pending-count banner when filtered tab empty but pending patterns exist.

**Files:** `frontend/src/pages/SubscriptionsPage.tsx`

### Done when

- [ ] Empty API state shows threshold copy + Sync link
- [ ] Standing/all tab with pending-only data shows banner linking to Pending review
- [ ] `npm test` / build PASS

---

## L3 — verify-work omniflow probes

**Status:** open  
**Depends on:** I1–J2 deployed + operator **Full Firefly sync**  
**Estimate:** 1h (operator + QA prep)  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0004 **(I)**, **(J)**, **(K)**, **(L)**

### Description

Prepare verify-work checklist mapping rows I–L:

| Step | Probe |
|------|-------|
| 1 | Deploy Q0011 image |
| 2 | Manual **Full** Firefly sync (account balance backfill) |
| 3 | Manual exchange sync → terminal status **(I)** |
| 4 | `POST /analytics/grafana/api/ds/query` portfolio pie **200** **(K)** |
| 5 | `GET /api/v1/wealth` non-empty accounts **(L)** |
| 6 | `GET /api/v1/forecast/daily?account_id=<asset>` non-zero series **(L)** |
| 7 | Subscriptions UI pending banner / thresholds **(J)** |

**Artifacts:** `sprints/quick/Q0011/uat.md`

### Done when

- [ ] UAT checklist maps I/J/K/L to probe steps
- [ ] Operator smoke PASS on financegnome.omniflow.cc

---

## Execution order

1. **I1** — exchange sync terminal status  
2. **K1** — portfolio UNION SQL  
3. **L1** — account balance parse  
4. **L2** — wealth NULL filter  
5. **J1** — payee fallbacks  
6. **J2** — subscriptions UX  
7. **Deploy** → **Full Firefly sync** → **L3** verify-work

## Split decision

- **Why 7 tasks:** Maps architecture slices I/K/L/J + operator L3; 7 ≪ 12 threshold.
- **Why not split Q0011a/b:** Shared deploy + Full sync backfill gate; single omniflow verify pass.
- **DEC-0060 / DEC-0061:** Govern L1 and J1; no additional DEC in sprint.
