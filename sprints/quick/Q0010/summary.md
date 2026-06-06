# Sprint Summary — Q0010 (BUG-0006)

**Bug:** BUG-0006  
**Sprint:** Q0010 (`/quick`)  
**Execute date:** 2026-06-05  
**Release date:** 2026-06-05  
**Status:** **DONE / released**

## Tasks delivered

| ID | Status | Summary |
|----|--------|---------|
| Q1 | done | Extract `category_id` from first split; `upsert_transaction` persists `category_id` |
| Q2 | done | `parse_sync_date`: YYYY-MM-DD → RFC3339 → 10-char prefix; warn on failure |
| Q3 | done | `normalize_split_amount` per DEC-0059 (withdrawal/deposit/transfer/unknown) |
| R1 | done | `TransactionAggregates` totals + `period_status`; NULL → "Uncategorized" label |
| P1 | done | verify-work PASS re-run 2 on omniflow; rows P/Q/R evidenced |

## Tests

| Check | Result |
|-------|--------|
| `cargo test --lib` | **PASS** (123/123 at release) |
| `npm test` | **PASS** (2/2) |
| `npm run build` | **PASS** |

## Acceptance (released)

| Row | Verdict |
|-----|---------|
| **(P)** | PASS — AI Chat category/spending answers for populated months |
| **(Q)** | PASS — 917/922 `category_id`, 919/922 `date`, 865/922 `amount < 0` post-sync |
| **(R)** | PASS — aggregate totals + empty vs populated period distinction |

## Files changed (primary)

- `backend/src/firefly/mod.rs` (Q1–Q3 + unit tests)
- `backend/src/db/mod.rs` (`upsert_transaction` + `category_id`)
- `backend/src/transactions/types.rs` (R1 contract)
- `backend/src/transactions/repository.rs` (`period_summary`)
- `backend/src/transactions/service.rs` (assembly + Uncategorized label)
- `backend/src/ai/tools/transactions.rs` (tool description passthrough)

## Evidence

- `handoffs/releases/Q0010-release-notes.md`
- `sprints/quick/Q0010/verify-work-findings.md`
- `handoffs/verify_work_to_release.md`
- `docs/engineering/state-archive/state-pack-20260605-q0010-bug0006.md`

## Decisions

- DEC-0059 (Firefly mirror amount sign normalization)
- R-0060 fulfilled
