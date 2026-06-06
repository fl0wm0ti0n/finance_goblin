# Tasks — Q0013 (BUG-0010)

**Bug:** BUG-0010  
**Task count:** 7 (within `SPRINT_MAX_TASKS=12`)  
**Sprint-plan ref:** `sprint-plan-20260605-q0013-bug0010`

## Discovery → sprint mapping

| Discovery ID | Disposition | Notes |
|--------------|-------------|-------|
| **AA1** | Task **AA1** | Balance mirror ingest + sync diagnostics |
| **AA2** | Operator gate **FULL_FIREFLY_SYNC** | Manual Full Firefly sync + recompute **before V1** — not a code task |
| **AA3** | Task **AA3** | Negative starting balance warning |
| **AB1** | Task **AB1** | Negative asset wealth visibility (DEC-0065) |
| **AB2** | Task **AB2** | Zero-total empty-state guidance |
| **AB3** | Validated in **V1** | Snapshot re-verify via `wealth` + `wealth/history` probes — not a code task |
| **AC1** | Task **AC1** | `sidecar_disabled` metadata (DEC-0066) |
| **AC2** | Task **AC2** | ML three-state UI copy |
| **AC3** | Deferred **US-0013** | ML sidecar on external profile — epic, out of scope |

## Execute order

```text
AA1 → AB1 → AC1 → AA3 → AB2 → AC2
  → single PR deploy
  → operator manual Full Firefly sync (AA2 gate)
  → V1 verify-work (includes AB3 snapshot check)
```

**Parallelism:** AA1, AB1, AC1 may start in parallel; AA3 after AA1; AB2 after AB1; AC2 after AC1.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **(AA)** | AA1, AA3, V1 | Plausible forecast OR negative-start warning |
| **(AB)** | AB1, AB2, V1 | Non-empty wealth; acct 114 visible |
| **(AC)** | AC1, AC2, V1 | ML not-enabled copy; no false skip banner |

| ID | Title | Est. | Status | Acceptance |
|----|-------|------|--------|------------|
| AA1 | Balance mirror ingest + diagnostics | 2h | done | **(AA)** |
| AB1 | Negative asset wealth visibility (DEC-0065) | 2h | done | **(AB)** |
| AC1 | sidecar_disabled metadata (DEC-0066) | 1.5h | done | **(AC)** |
| AA3 | Negative starting balance warning | 2h | done | **(AA)** |
| AB2 | Wealth zero-total empty-state | 1h | done | **(AB)** |
| AC2 | Forecast ML three-state UI | 1.5h | done | **(AC)** |
| V1 | verify-work omniflow probes | 1h | done | **(AA)(AB)(AC)** |

---

## AA1 — Balance mirror ingest + diagnostics

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0010 **(AA)**

### Description

Extend `sync_accounts` per architecture § AA1: trust `attributes.current_balance` via `parse_split_amount` (DEC-0060); emit structured `balance_ingest` tracing fields (`firefly_id`, `name`, `raw_current_balance`, `parsed_balance`, `account_role`); warn on parse failure.

**Files:** `backend/src/firefly/mod.rs`

### Done when

- [ ] Unit: string/number `current_balance` → parsed upsert unchanged (DEC-0060 regression)
- [ ] Unit: parse failure logs warn, upserts NULL balance
- [ ] Structured log fields present on account sync
- [ ] `cargo test --lib` firefly PASS

---

## AB1 — Negative asset wealth visibility (DEC-0065)

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0010 **(AB)**

### Description

Remove `COALESCE(balance, 0) >= 0` from `load_asset_accounts`. Add `is_overdrawn: bool` to `AccountWealthRow` when `balance < 0`. Signed sum in `firefly.subtotal_eur`.

**Files:** `backend/src/wealth/repository.rs`, `backend/src/wealth/types.rs`, `backend/src/wealth/service.rs`, `frontend/src/pages/WealthPage.tsx`

### Done when

- [ ] Unit/integration: negative balance asset row included
- [ ] API JSON includes `is_overdrawn: true` for negative accounts
- [ ] Wealth UI shows overdrawn styling
- [ ] `cargo test --lib` wealth PASS

---

## AC1 — sidecar_disabled metadata (DEC-0066)

**Status:** open  
**Depends on:** —  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0010 **(AC)**

### Description

When `!forecast_ml.enabled`, after baseline recompute call `record_skip_on_baseline(baseline_id, Disabled)`. Meta API derives `ml_skipped_reason: sidecar_disabled` when config disabled and metadata null.

**Files:** `backend/src/sync/mod.rs`, `backend/src/api/forecast.rs`

### Done when

- [ ] Unit: disabled config → baseline metadata `ml_skipped_reason: sidecar_disabled`
- [ ] Meta endpoint returns derived reason for stale rows when ML off
- [ ] Enabled config path unchanged (no double-record regression)
- [ ] `cargo test --lib` sync/forecast PASS

---

## AA3 — Negative starting balance warning

**Status:** open  
**Depends on:** AA1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0010 **(AA)**

### Description

Add `balance_warnings` to forecast meta when asset account has `balance <= 0` and transaction history. ForecastPage banner when warnings present.

**Files:** `backend/src/forecast/service.rs`, `backend/src/api/forecast.rs`, `frontend/src/pages/ForecastPage.tsx`, `frontend/src/lib/api.ts`

### Done when

- [ ] Unit: negative start + tx count → warning in meta
- [ ] Zero balance + no txs → no warning
- [ ] UI banner renders when warnings present
- [ ] `cargo test --lib` + `npm test` PASS

---

## AB2 — Wealth zero-total empty-state

**Status:** open  
**Depends on:** AB1  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0010 **(AB)**

### Description

When accounts present but `total_eur == 0` (or all zero), show operator callout: Full Firefly sync guidance + reconcile hint.

**Files:** `frontend/src/pages/WealthPage.tsx`

### Done when

- [ ] Zero-total with accounts shows guidance callout
- [ ] Non-zero total hides callout
- [ ] `npm run build` PASS

---

## AC2 — Forecast ML three-state UI

**Status:** open  
**Depends on:** AC1  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0010 **(AC)**

### Description

Replace default "ML forecast unavailable" skip copy. Three states: ML available, ML not enabled (`sidecar_disabled`), ML skipped (other reasons).

**Files:** `frontend/src/pages/ForecastPage.tsx`

### Done when

- [ ] `sidecar_disabled` shows "not enabled on this deployment" copy
- [ ] Null reason + ML off does not show "ML skipped"
- [ ] Other skip reasons show "ML skipped: {reason}"
- [ ] `npm test` PASS

---

## V1 — verify-work omniflow probes

**Status:** open  
**Depends on:** AA1, AB1, AC1, AA3, AB2, AC2 deploy + Full Firefly sync  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0010 **(AA)(AB)(AC)**

### Description

Prepare `sprints/quick/Q0013/uat.md` smoke checklist. After deploy + manual Full Firefly sync, probe forecast/wealth/meta on `financegnome.omniflow.cc`.

**Files:** `sprints/quick/Q0013/uat.md`

### Done when

- [ ] Row **(AA)**: plausible forecast start OR negative-start warning visible
- [ ] Row **(AB)**: wealth includes acct 114 (or all synced assets); honest total
- [ ] Row **(AC)**: meta `sidecar_disabled`; UI not-enabled copy
- [ ] Regression footer documented (OIDC + bundled-firefly)

**Operator gate:** Manual Full Firefly sync required before V1 probes.
