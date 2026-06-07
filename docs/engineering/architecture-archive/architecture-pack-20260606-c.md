# Architecture archive pack (2026-06-06)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 14
- First archived heading: `## BUG-0004 — Post-sync pipeline empty analytics`
- Last archived heading: `## BUG-0004 — Post-sync pipeline empty analytics`
- Verification tuple (mandatory):
  - archived_body_lines=211
  - preamble_lines=10
  - retained_body_lines=2969

---

## BUG-0004 — Post-sync pipeline empty analytics

**Status:** architecture complete (2026-06-05)  
**Discovery:** `discovery-20260605-bug0004` in `handoffs/po_to_tl.md`  
**Research:** [R-0061](research.md#r-0061--post-sync-analytics-pipeline-empty-data-paths)  
**Decisions:** **DEC-0060** (Firefly account balance parse); **DEC-0061** (subscription payee key fallbacks); extends **DEC-0002** (upsert backfill), **DEC-0014** (confidence tiers), **DEC-0041** (exchanges sync phase)  
**Sprint:** `/quick` **Q0011** (recommended)  
**Acceptance:** `docs/product/acceptance.md` rows **I**, **J**, **K**, **L**  
**Related:** BUG-0006 Q0010 (transaction sign/date may improve subscription expense filter — coordinate, do not merge); BUG-0005 OPEN — separate track

### Symptom chain (frozen)

Operator on US-0010 external profile: 922+ transactions synced; post-recovery stack healthy. Four independent wiring gaps produce empty analytics and misleading sync status:

| Sub | Gap | Effect |
|-----|-----|--------|
| **I** | `RunMode::ExchangesOnly` never calls `finish_sync_run` | DB `sync_runs` stuck `running`; status UI misleading |
| **J** | Payee grouping uses normalized `description` only; detection Full-sync only | Long bank-memo keys; 0 confirmed until operator action; empty UX on confirmed-only tabs |
| **K** | Invalid `UNION ALL` SQL in portfolio pie panel | Grafana ds/query **500** `syntax error at or near "UNION"` |
| **L1** | Account `current_balance` parsed with `.as_f64()` only | Firefly string balances → mirror `accounts.balance` **NULL** |
| **L2** | Wealth query `balance >= 0` excludes NULL | `GET /api/v1/wealth` → `accounts: []` |
| **L3** | Forecast `starting_balance = balance.unwrap_or(0.0)` | Flat **0.00** series despite recompute rows |

`isolation_scope`: artifact + repo source reads + public curl probes (R-0061); no host `.env` / `.env_prod` secrets read.

### Fix slices

```text
BUG-0004
├── I — Sync lifecycle (P0)
│   └── I1 — finish_sync_run on ExchangesOnly terminal path
├── K — Grafana SQL (P0, no backend deploy dependency)
│   └── K1 — Portfolio pie UNION subquery wrap
├── L — Wealth / forecast data path (P0, ordered)
│   ├── L1 — Parse Firefly account current_balance string/number (DEC-0060)
│   └── L2 — Wealth asset query includes NULL balances (COALESCE)
├── J — Subscriptions (P1)
│   ├── J1 — Payee key fallbacks (DEC-0061)
│   └── J2 — Empty-state UX: pending count + detection thresholds
└── L3 — Operator verify (P1)
    └── L3 — omniflow probes rows I–L (post Full sync + recompute)
```

**Deploy order:** I1 + K1 + L1 + L2 + J1 + J2 in one PR (backend + Grafana JSON + frontend); operator runs **manual Full Firefly sync** to backfill account balances (DEC-0002 upsert) before L3 verify. Exchange-only sync after I1 fixes terminal status but does **not** run subscription detection — document in J2 empty-state.

### I1 — Exchange sync terminal status (frozen)

**Problem:** `execute_run` calls `finish_sync_run` only on `RunMode::Full` Firefly path (`sync/mod.rs` L236–257). `ExchangesOnly` (`manual_exchanges`, `scheduled_exchanges`) runs `run_exchanges_and_alerts` then clears in-memory `active_run` without persisting terminal status.

**Contract:**

| Event | Action |
|-------|--------|
| `run_exchanges_and_alerts` returns `Ok(())` | `finish_sync_run(pool, run_id, "success", None)` |
| `run_exchanges_and_alerts` returns `Err(e)` | `finish_sync_run(pool, run_id, "failed", Some(&e.to_string()))`; propagate error; clear phase + active_run (mirror Full error path) |

Apply in `execute_run` **after** `run_exchanges_and_alerts` when `mode == RunMode::ExchangesOnly`, **or** unconditionally at end of `execute_run` for both modes (Full already finished Firefly phase earlier — do **not** double-finish Full run).

**Recommended implementation:** At end of `execute_run`, if `mode == RunMode::ExchangesOnly`, call `finish_sync_run` based on `run_exchanges_and_alerts` result. Full mode keeps existing Firefly-phase finish only.

**Stuck historical rows (decision gate):** **Rejected** one-shot SQL migration marking orphaned `running` rows `failed` on deploy — out of scope; I1 fixes forward path only. Operators may ignore stale rows or manual cleanup.

**Files:** `backend/src/sync/mod.rs`

**Risks:** Double `finish_sync_run` if refactor breaks Full/ExchangesOnly guard — unit test both modes.

### K1 — Portfolio Grafana UNION SQL (frozen)

**Problem:** Panel id **8** in `portfolio.json` L80:

```sql
SELECT ... ORDER BY snapshot_date DESC LIMIT 1 UNION ALL SELECT ... ORDER BY ... LIMIT 1
```

PostgreSQL requires each `ORDER BY`/`LIMIT` branch wrapped in parentheses.

**Contract:** Rewrite as:

```sql
(SELECT 'Firefly' AS metric, COALESCE(firefly_value_eur, total_eur)::float AS value
 FROM net_worth_snapshots ORDER BY snapshot_date DESC LIMIT 1)
UNION ALL
(SELECT 'Crypto', COALESCE(crypto_value_eur, 0)::float
 FROM net_worth_snapshots ORDER BY snapshot_date DESC LIMIT 1)
```

**Alternatives rejected:** Single-row pivot with `MAX(snapshot_date)` subquery — valid but higher churn; subquery wrap is minimal fix.

**Verify:** `POST /analytics/grafana/api/ds/query` with portfolio pie raw SQL → **200** (acceptance **K**).

**Files:** `grafana/provisioning/dashboards/analytics/portfolio.json`

**Risks:** Other panels with similar UNION pattern — scan analytics folder in execute; out of scope unless probe fails.

### L1 — Account balance parse — DEC-0060 (frozen)

**Problem:** `sync_accounts` passes `item["attributes"]["current_balance"].as_f64()` (`firefly/mod.rs` L261). Firefly API returns balance as **JSON string** (e.g. `"1234.56"`) → NULL mirror balances.

**Contract:** Reuse existing `parse_split_amount(value: &Value) -> Option<f64>` (already handles number + trimmed string) for `current_balance`. Pass result to `upsert_account`.

**Re-sync backfill:** DEC-0002 upsert on next **Full** sync — no SQL migration. Operator must trigger Full sync post-deploy before L3 wealth/forecast probes.

**Alternatives rejected:** Separate `parse_account_balance` helper — duplicates `parse_split_amount`; read balance from `payload` at query time — spreads parse logic.

**Files:** `backend/src/firefly/mod.rs`

**Risks:** Locale-specific decimal commas — Firefly uses dot decimals per R-0001; log warn on parse failure.

### L2 — Wealth NULL balance filter (frozen)

**Problem:** `load_asset_accounts` filters `AND balance >= 0` (`wealth/repository.rs` L36). SQL NULL comparisons exclude all NULL-balance asset rows even when account should contribute `0` to net worth.

**Contract:** Replace predicate with:

```sql
AND COALESCE(balance, 0) >= 0
```

Service layer already uses `a.balance.unwrap_or(0.0)` for aggregation (`wealth/service.rs`). Negative balances (credit/overdraft asset accounts) remain excluded.

**Downstream:** Forecast `starting_balance = account.balance.unwrap_or(0.0)` (`forecast/service.rs` L105) populates non-zero series once L1 backfills balances; L2 ensures accounts appear in wealth API before backfill completes.

**Files:** `backend/src/wealth/repository.rs`

**Risks:** Including NULL-as-zero before L1 backfill shows zero balances — acceptable interim vs empty `accounts: []`.

### J1 — Payee key fallbacks — DEC-0061 (frozen)

**Problem:** `by_payee()` keys only on `payee_key(description)` (`recurrence/group.rs` L17). Firefly journals often carry merchant identity in split/counterparty fields while `description` is generic bank text.

**Contract:** Add `extract_payee_source(tx: &TransactionRow) -> Option<String>` with **first non-empty** after normalization:

| Priority | Source |
|----------|--------|
| 1 | `tx.description` |
| 2 | First split `counterparty_name` from `payload.attributes.transactions[0]` |
| 3 | First split `destination_name` from same path |
| 4 | Top-level `payload.attributes.external_url` **rejected** — not stable merchant key |

Apply `payee_key()` to chosen source string. Skip row when all sources empty.

**Expense filter unchanged:** `amount < 0` (or `amount >= 0` skip) + `is_transfer` skip + min amount 0.01 — aligns with DEC-0014 tiers after BUG-0006 Q3 sign fix on same mirror rows.

**Detection trigger unchanged:** Full sync only (DEC-0018) — J2 documents this.

**Alternatives rejected:** Mirror dedicated `payee_name` column — heavier migration; regex on description only — insufficient for bank memos.

**Files:** `backend/src/recurrence/group.rs` (new helper + `by_payee`); optional unit tests in `recurrence/normalize.rs`

**Risks:** Over-merging distinct merchants sharing counterparty — monitor via confidence tiers; operator confirm/reject unchanged.

### J2 — Subscriptions empty-state UX (frozen)

**Problem:** Operator perceives "empty subscriptions" when **0 confirmed** despite **11 pending** patterns; Standing orders tab filters `status=confirmed` + `kind=standing_order`; All tab empty only when API returns `[]`.

**Contract:**

| Condition | UI behavior |
|-----------|-------------|
| `patterns.length === 0` (API empty) | Empty card: link to Sync Status; copy documents thresholds: **≥3** matching expenses, **≥60%** confidence (DEC-0014), payee key from description + counterparty fallbacks (DEC-0061); note detection runs on **Full Firefly sync** only |
| `pending_count > 0` && current tab shows no rows (e.g. Standing orders) | Banner: "{n} pattern(s) pending review" with link/button to **Pending review** tab |
| Pending tab | Unchanged confirm/reject flow |

**Files:** `frontend/src/pages/SubscriptionsPage.tsx`

**Risks:** Copy-only fix does not auto-confirm — acceptance **J** allows documented thresholds OR surfaced patterns; J1+J2 together satisfy "or documents detection thresholds".

### Task map (Q0011)

| Order | Task | Layer | Acceptance |
|-------|------|-------|------------|
| 1 | **I1** finish_sync_run ExchangesOnly | backend sync | **I** |
| 2 | **K1** portfolio UNION SQL | Grafana JSON | **K** |
| 3 | **L1** account balance parse (DEC-0060) | backend firefly | **L** |
| 4 | **L2** wealth NULL filter | backend wealth | **L** |
| 5 | **J1** payee key fallbacks (DEC-0061) | backend recurrence | **J** |
| 6 | **J2** subscriptions empty-state UX | frontend | **J** |
| 7 | **L3** verify-work omniflow probes | verify-work | **I–L** |

**Count:** 7 tasks (≤ `SPRINT_MAX_TASKS` 12) → **`/quick` Q0011**; no split.

### Test strategy

| Check | Type | Pass criteria |
|-------|------|---------------|
| I1 | Unit | ExchangesOnly mock run → `finish_sync_run` called with `success`; error path → `failed` |
| K1 | Fixture / manual | Portfolio pie SQL executes without UNION syntax error |
| L1 | Unit | String `"1234.56"` and number `1234.56` → same `f64` on upsert |
| L2 | Unit / integration | NULL balance asset row returned by `load_asset_accounts` |
| J1 | Unit | Fixture payload with counterparty_name only → grouped payee key |
| J2 | Component / manual | Empty + pending scenarios render threshold copy and pending banner |
| L3 | Operator | Post Full sync: sync status terminal; portfolio ds/query 200; wealth non-empty; forecast non-zero for funded account; subscriptions pending/confirmed UX |

**Post-deploy operator steps:** Deploy Q0011 → **manual Full Firefly sync** (account balance backfill) → optional manual exchange sync (I1 probe) → L3 verify-work.

### Decisions (BUG-0004)

| Topic | Resolution |
|-------|------------|
| Account balance parse | **DEC-0060** — reuse `parse_split_amount` for `current_balance` |
| Payee key source | **DEC-0061** — description → counterparty_name → destination_name |
| Re-sync backfill | **Upsert on next Full sync** (DEC-0002) — no migration script |
| Stuck `running` rows | **Forward fix only (I1)** — no deploy-time cleanup |
| Merge with BUG-0006 | **Rejected** — separate Q0010 track; coordinate transaction sign for J1 expense filter |

### Next phase

**`/sprint-plan` Q0011** — validate `sprints/quick/Q0011/task.json`; then `/plan-verify` → `/execute`.

---

