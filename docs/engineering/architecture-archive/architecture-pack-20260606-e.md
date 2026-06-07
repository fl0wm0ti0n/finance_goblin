# Architecture archive pack (2026-06-06)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 13
- First archived heading: `## BUG-0010 — Forecast wrong numbers, empty wealth, misleading ML skip`
- Last archived heading: `## BUG-0010 — Forecast wrong numbers, empty wealth, misleading ML skip`
- Verification tuple (mandatory):
  - archived_body_lines=217
  - preamble_lines=10
  - retained_body_lines=2954

---

## BUG-0010 — Forecast wrong numbers, empty wealth, misleading ML skip

**Status:** architecture complete (2026-06-05)  
**Discovery:** `discovery-20260605-bug0010` in `handoffs/po_to_tl.md`  
**Research:** [R-0062](research.md#r-0062--firefly-account-balance-mirror-vs-forecastwealth-inputs)  
**Decisions:** **DEC-0065** (negative asset wealth visibility); **DEC-0066** (ML disabled metadata); extends **DEC-0060** (balance parse), **DEC-0007** (forecast algorithm), **DEC-0049** (ML default off), **DEC-0025** (net worth aggregation)  
**Sprint:** `/quick` **Q0013**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0010 rows **AA**, **AB**, **AC** (AC3 → **US-0013** epic)  
**Related:** BUG-0004 Q0011 (DEC-0060/L2 partial — residual wrong numbers); BUG-0009 (Grafana — separate); US-0013 (ML production on omniflow)

### Symptom chain (frozen)

Operator on US-0010 external profile post-BUG-0004: 922+ transactions synced; forecast shows **-25365.78** 3-month end balance; wealth **total_eur: 0**; forecast UI **"ML skipped: ML forecast unavailable"** despite ML never configured.

| Sub | Gap | Effect |
|-----|-----|--------|
| **AA** | Mirror `accounts.balance` wrong or stale vs Firefly `current_balance`; negative start drives DEC-0007 drift | Forecast series from **-3395.75** → implausible -25k end |
| **AB** | `load_asset_accounts` excludes `balance < 0`; zero-balance accounts dominate sum | Giro 114 absent; `total_eur: 0.0`; snapshots written but zero |
| **AC** | ML phase skipped when disabled → null `ml_skipped_reason`; UI default skip copy | Misleading "ML skipped" when profile is baseline-only by design |

**Math check (frozen):** -3395.75 + ~90d outflows ≈ -25365 — DEC-0007 behaves correctly given inputs. Fix is **inputs + visibility + honest ML posture**, not projection rewrite.

`isolation_scope`: artifact + repo source + public curl probes (R-0062); no host `.env` / `.env_prod` secrets read.

### Fix slices

```text
BUG-0010
├── AA — Balance mirror + forecast inputs (P0)
│   ├── AA1 — Trust Firefly current_balance ingest + sync diagnostics (extends DEC-0060)
│   └── AA3 — Warn when starting_balance <= 0 with tx history (API meta + UI)
├── AB — Wealth visibility (P0)
│   ├── AB1 — Include negative-balance asset accounts (DEC-0065)
│   └── AB2 — Zero-total empty-state with operator guidance
├── AC — ML posture (P0)
│   ├── AC1 — sidecar_disabled metadata on baseline (DEC-0066)
│   └── AC2 — UI three-state ML copy (not enabled / skipped / available)
└── V1 — Operator verify (P1)
    └── V1 — Full Firefly sync gate + omniflow probes AA/AB/AC
```

**Out of scope:** AC3 (US-0013 — stats-forecast on external profile); Grafana emptiness (BUG-0009); transaction-sign re-ingest (BUG-0006 DONE).

**Deploy order:** AA1 + AB1 + AC1 + AA3 + AB2 + AC2 in one PR (backend + frontend); operator **manual Full Firefly sync** + forecast recompute before V1 verify.

### AA1 — Account balance mirror ingest (frozen)

**Problem:** DEC-0060 fixed string parse but omniflow still shows wrong/zero mirror balances. Discovery confirms non-NULL values (-3395.75, 0.0) — parse works; issue is **source fidelity or stale sync**, not `.as_f64()`.

**Contract:**

| Step | Action |
|------|--------|
| Canonical field | `attributes.current_balance` via existing `parse_split_amount` (DEC-0060) — **no alternate field** |
| Payload | Full account `item` already stored in `payload` JSONB — ensure `account_role`, `active`, `include_net_worth`, `opening_balance` available for diagnostics |
| Sync diagnostics | On each account upsert, emit structured log `balance_ingest` with `firefly_id`, `name`, `raw_current_balance`, `parsed_balance`, `account_role` |
| Stale mirror gate | Document: account rows refresh only on **Full** Firefly sync (`sync_accounts` in Full path) — exchange-only runs do **not** update balances |
| Mismatch probe (execute) | Optional dev-only or sync-summary: compare parsed value against Firefly raw string; warn on parse failure |

**Alternatives rejected:**

| Alternative | Why |
|-------------|------|
| Recompute balance from mirrored transactions | Duplicates Firefly ledger; heavy; violates read-only mirror model |
| Use `opening_balance` instead of `current_balance` | Opening balance is historical snapshot only per Firefly docs |
| Negate asset balances at ingest | Would break signed forecast/wealth semantics |

**If Firefly source matches mirror after Full sync:** AA3 + AB2 surface data-quality guidance (reconcile in Firefly) — not an ingest code bug.

**Files:** `backend/src/firefly/mod.rs`

**Risks:** Operator expects funded accounts but Firefly ledger shows overdraft — product shows truthful negative numbers + warnings, not silent -25k.

### AA3 — Negative starting balance warning (frozen)

**Problem:** Forecast silently projects from negative `starting_balance` producing implausible milestones without operator signal.

**Contract:**

| Surface | Field / behavior |
|---------|------------------|
| `GET /api/v1/forecast/meta` | Add optional `balance_warnings: [{ account_id, starting_balance, reason: "negative_starting_balance" }]` when any asset account used in latest baseline has `balance <= 0` **and** `COUNT(transactions WHERE account_id = …) > 0` |
| `GET /api/v1/forecast/daily` (optional) | Per-account `starting_balance_warning: true` when condition met |
| `ForecastPage.tsx` | Banner when meta warnings present: "Starting balance is zero or negative — verify Firefly account balances or reconcile before trusting long-term forecast." |

**Alternatives rejected:** Hard-block forecast when negative — breaks legitimate overdraft scenarios; warning only.

**Files:** `backend/src/forecast/service.rs`, `backend/src/api/forecast.rs`, `frontend/src/pages/ForecastPage.tsx`

**Risks:** Extra DB query per recompute — cache in baseline metadata JSON to avoid repeat on read path.

### AB1 — Wealth negative account visibility — DEC-0065 (frozen)

**Problem:** `load_asset_accounts` filters `COALESCE(balance, 0) >= 0` (`wealth/repository.rs` L36). Acct 114 (negative Giro) excluded; 115/116 at 0 → `total_eur: 0`.

**Contract:**

```sql
-- Remove >= 0 predicate; keep active + include_net_worth filters
WHERE type = 'asset'
  AND COALESCE((payload->>'active')::boolean, true) = true
  AND COALESCE((payload->>'include_net_worth')::boolean, true) = true
```

| API field | Semantics |
|-----------|-----------|
| `AccountWealthRow.is_overdrawn` | `true` when `balance < 0` |
| `firefly.subtotal_eur` | Signed sum of included account balances |
| `total_eur` | Firefly subtotal + crypto (unchanged composition) |

**UI:** Overdrawn rows show amber badge / negative styling; `% of total` uses signed household sum (existing window fn).

**Alternatives rejected:** Separate liability model — US epic scope; clamp to zero — distorts net worth.

**Files:** `backend/src/wealth/repository.rs`, `backend/src/wealth/types.rs`, `backend/src/wealth/service.rs`, `frontend/src/pages/WealthPage.tsx`

**Risks:** Headline total dominated by overdraft — mitigated by AB2 guidance + per-row flag.

### AB2 — Wealth zero-total empty-state (frozen)

**Problem:** `total_eur == 0` with synced accounts present gives no operator guidance.

**Contract:** When `GET /api/v1/wealth` returns `accounts.length > 0` **and** `total_eur == 0` (or all balances zero), UI shows callout:

- "Account balances may be stale — trigger **Full Firefly sync** from Settings."
- Link to Firefly reconciliation docs (external).
- If any `is_overdrawn`, note signed total may be negative after AB1.

**Files:** `frontend/src/pages/WealthPage.tsx`

**Risks:** Copy-only; no backend change required beyond AB1 fields.

### AB3 — Snapshot re-verify (operator, frozen)

**Not a code task.** After AA1 deploy + **Full Firefly sync**, exchange-triggered snapshot upsert should reflect corrected mirror balances. Validated in **V1** via `GET /api/v1/wealth`, `GET /api/v1/wealth/history`, and non-zero or honestly negative `total_eur`.

### AC1 — ML disabled metadata — DEC-0066 (frozen)

**Problem:** When `forecast_ml.enabled = false`, sync skips ML block (`sync/mod.rs` L292–313) without `record_skip_on_baseline`. Meta returns `ml_skipped_reason: null`.

**Contract:**

| Path | Behavior |
|------|----------|
| Sync (Full path, post-baseline) | When `!config.forecast_ml.enabled`, call `forecast_ml.record_skip_on_baseline(baseline_id, &ForecastMlError::Disabled)` |
| Meta API | If config disabled and baseline metadata lacks `ml_skipped_reason`, derive `"sidecar_disabled"` in response (transitional stale rows) |

**Canonical reason codes (unchanged):** `sidecar_disabled`, `sidecar_unavailable`, `insufficient_history`, `sidecar_error`.

**Alternatives rejected:** New `ml_status: "unconfigured"` — use existing `skipped` + `sidecar_disabled`.

**Files:** `backend/src/sync/mod.rs`, `backend/src/api/forecast.rs`

**Risks:** Double-record on recompute — `merge_metadata` idempotent patch.

### AC2 — Forecast UI ML posture (frozen)

**Problem:** `ForecastPage.tsx` L47–53: `mlSkipReason` defaults to `"ML forecast unavailable"`; explain panel always says "ML skipped" when `!mlAvailable`.

**Contract — three UI states:**

| State | Condition | Copy (frozen intent) |
|-------|-----------|----------------------|
| **ML available** | `ml_computation_id` + `ml_status === "success"` | Existing model/seasonal copy |
| **ML not enabled** | `ml_skipped_reason === "sidecar_disabled"` (or meta derive) | "ML forecast is not enabled on this deployment. Baseline DEC-0007 forecast is authoritative." |
| **ML skipped** | Other non-null `ml_skipped_reason` | "ML skipped: {reason}. Baseline DEC-0007 forecast remains authoritative." |
| **Baseline only (legacy null)** | No ML id, null reason, ML enabled in config | "Baseline-only forecast — ML has not run yet." |

**Remove:** Default "ML forecast unavailable" as skip reason when reason is null and ML disabled.

**Long-term mode toggles:** Disable `ml_enhanced` / `compare` when not ML available (existing); tooltip cites correct state label.

**Files:** `frontend/src/pages/ForecastPage.tsx`, `frontend/src/lib/api.ts` (types if needed)

**Risks:** None — copy-only + reason wiring.

### Task map (Q0013)

| Order | Task | Layer | Acceptance |
|-------|------|-------|------------|
| 1 | **AA1** balance mirror diagnostics | backend firefly | **AA** |
| 2 | **AB1** negative wealth visibility | backend wealth + types | **AB** |
| 3 | **AC1** sidecar_disabled metadata | backend sync + forecast meta | **AC** |
| 4 | **AA3** negative start warning | backend forecast + frontend banner | **AA** |
| 5 | **AB2** wealth zero-total empty-state | frontend wealth | **AB** |
| 6 | **AC2** ML three-state UI | frontend forecast | **AC** |
| 7 | **V1** verify-work omniflow | uat / probes | **AA**, **AB**, **AC** |

**Operator gate (AA2 / AB3):** Manual **Full Firefly sync** after deploy required before V1 — account balance backfill via DEC-0002 upsert + baseline/wealth recompute.

### Test strategy

| Check | Type | Pass criteria |
|-------|------|---------------|
| AA1 | Unit | String/number `current_balance` → parsed upsert; `balance_ingest` log fields |
| AB1 | Unit/integration | Negative balance asset row included; `is_overdrawn: true` |
| AC1 | Unit | Disabled config → baseline metadata `ml_skipped_reason: sidecar_disabled` |
| AA3 | Unit | Negative start + tx count → meta warning |
| AC2 | Component/manual | Three copy states render per reason |
| V1 | Operator | Plausible forecast start OR explicit warning; wealth non-empty/honest total; ML not-enabled copy |

### Decisions (BUG-0010)

| Topic | Resolution |
|-------|------------|
| Negative asset wealth | **DEC-0065** — include with `is_overdrawn`, signed sum |
| ML disabled metadata | **DEC-0066** — persist + meta derive `sidecar_disabled` |
| Balance source | Trust Firefly `current_balance` (DEC-0060 parse) — no tx recompute |
| ML production on omniflow | **US-0013** (AC3) — out of scope |
| Merge with BUG-0009/0011 | **Rejected** |

### Next phase

**`/plan-verify` Q0013** — then `/execute`.

---

