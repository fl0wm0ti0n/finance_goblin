# Tasks — Q0010 (BUG-0006)

**Bug:** BUG-0006  
**Task count:** 5 (within `SPRINT_MAX_TASKS=12`)

| ID | Title | Est. | Status | Acceptance |
|----|-------|------|--------|------------|
| Q1 | Firefly `category_id` sync | 2h | open | **(Q)** |
| Q2 | ISO date parse in sync | 1.5h | open | **(Q)** |
| Q3 | Amount sign normalization (DEC-0059) | 2h | open | **(Q)** |
| R1 | TransactionAggregates contract extension | 3h | open | **(R)** |
| P1 | verify-work prep / acceptance mapping | 1h | open | **(P)** |

---

## Q1 — Firefly `category_id` sync

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0006 **(Q)** — `category_id` persisted on mirror `transactions` rows

### Description

Extract `category_id` from the first split in Firefly journal `attributes.transactions[]` and bind on `upsert_transaction`. NULL when absent in payload.

**Files:** `backend/src/firefly/mod.rs`, `backend/src/db/mod.rs`

### Done when

- [ ] Unit: fixture split JSON → `category_id` persisted on upsert
- [ ] `cargo test --lib` firefly/db PASS

---

## Q2 — ISO date parse in sync

**Status:** open  
**Depends on:** Q1  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0006 **(Q)** — period filter returns rows (non-NULL `date`)

### Description

Parse Firefly split dates as `YYYY-MM-DD` **or** ISO-8601/RFC3339 datetime; persist **date component only** (`NaiveDate`). Parse order: strict date → RFC3339 → first-10-char prefix fallback. Log `warn` when all parses fail.

**Files:** `backend/src/firefly/mod.rs`

### Done when

- [ ] Unit: ISO datetime string (e.g. `2025-06-01T12:00:00+02:00`) → non-NULL `NaiveDate`
- [ ] Unit: invalid date → NULL + warn (no panic)

---

## Q3 — Amount sign normalization (DEC-0059)

**Status:** open  
**Depends on:** Q2  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0006 **(Q)** — outflow sums align (`amount < 0` for expenses)

### Description

Normalize signed `amount` from split `type` before `upsert_transaction` per **DEC-0059**:

| Split `type` | Stored `amount` |
|--------------|-----------------|
| `withdrawal` | `-abs(amount)` |
| `deposit` | `+abs(amount)` |
| `transfer` | `-abs(amount)` on first split |
| missing / unknown | `-abs(amount)` when raw `amount > 0`, else raw value |

**Files:** `backend/src/firefly/mod.rs`

### Done when

- [ ] Unit: `withdrawal` / `deposit` / `transfer` → expected sign
- [ ] Re-sync backfill path documented (DEC-0002 upsert — no SQL migration)

---

## R1 — TransactionAggregates contract extension

**Status:** open  
**Depends on:** Q3  
**Estimate:** 3h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0006 **(R)** — totals + `period_status` under `allow_raw_transactions=false`

### Description

Extend `TransactionAggregates` and assemble in `TransactionsService::aggregates` + `GetTransactionsTool` passthrough.

**New fields:** `total_transaction_count`, `total_outflow`, `total_inflow`, `uncategorized_transaction_count`, `period_status`.

**`period_status` priority:** `no_rows_in_period` → `rows_zero_outflow` → `rows_uncategorized` → `rows_with_outflow`.

**Presentation:** NULL `category_id` → `category_name: "Uncategorized"` in service layer.

**Privacy:** DEC-0032 unchanged — new fields are aggregate-only; `raw_rows` still gated.

**Files:** `backend/src/transactions/types.rs`, `repository.rs`, `service.rs`, `backend/src/ai/tools/transactions.rs`

### Done when

- [ ] Integration: fixture mirror rows → JSON with totals + correct `period_status`
- [ ] Unit: privacy — summary fields present; `raw_rows` absent when raw disabled
- [ ] `cargo test --lib` transactions PASS

---

## P1 — verify-work prep / acceptance mapping

**Status:** open  
**Depends on:** R1 deployed + operator manual Firefly sync  
**Estimate:** 1h (operator + QA prep)  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0006 **(P)** — AI Chat uses non-empty aggregates

### Description

Prepare verify-work checklist for rows **P**, **Q**, **R**:

| Step | Action |
|------|--------|
| 1 | Deploy Q1–R1 backend image to omniflow |
| 2 | Trigger manual Firefly sync (backfill ~922 rows via upsert) |
| 3 | SQL probe: mirror `transactions` count, `category_id`, date range, amount sign |
| 4 | Invoke `get_transactions` (or AI Chat spending question) — non-empty totals / `by_category` |
| 5 | Confirm `period_status` distinguishes empty vs uncategorized vs zero-outflow |

**Artifacts:** `sprints/quick/Q0010/uat.md` smoke checklist (PENDING until verify-work)

### Done when

- [ ] UAT checklist maps P/Q/R to probe steps
- [ ] Operator smoke: AI Chat answers category/spending question without “no data” when mirror rows exist

---

## Execution order

1. **Q1** — category_id ingest  
2. **Q2** — ISO date parse  
3. **Q3** — amount sign (DEC-0059)  
4. **R1** — aggregate contract + AI tool passthrough  
5. **Deploy** → **manual Firefly sync** → **P1** verify-work

## Split decision

- **Why 5 tasks:** Maps architecture ingest chain Q1→Q2→Q3, aggregate R1, operator P1; ordered dependencies frozen in architecture.
- **Why not split Q0010a/b:** Shared deploy + sync backfill contract; 5 ≪ 12 threshold.
- **DEC-0059:** Governs Q3 only; no additional DEC in sprint.
