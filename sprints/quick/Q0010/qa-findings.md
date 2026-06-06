# QA Findings — Quick Q0010 / BUG-0006

**Work item:** BUG-0006 (defect)  
**Quick task:** Q0010  
**QA phase:** `/qa`  
**Date:** 2026-06-05  
**Verdict:** **PASS** (ready for `/verify-work`; P1 operator deploy + Firefly sync deferred)

## Scope

Firefly ingest + aggregate defects per `architecture-20260605-bug0006` (`handoffs/tl_to_dev.md`):

- **Q1** — Extract `category_id` from first split; `upsert_transaction` persists `category_id`
- **Q2** — `parse_sync_date`: strict YYYY-MM-DD → RFC3339 → 10-char prefix; warn on failure
- **Q3** — `normalize_split_amount` per DEC-0059 (withdrawal/deposit/transfer/unknown)
- **R1** — `TransactionAggregates` totals + `period_status`; NULL → `"Uncategorized"` label
- **P1** — Operator verify after deploy + manual Firefly sync backfill (~922 rows)

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0010/summary.md`, `sprints/quick/Q0010/plan-verify.json`, `docs/product/acceptance.md` (BUG-0006 rows P/Q/R), `docs/engineering/architecture.md` (§ BUG-0006), `decisions/DEC-0059.md`, `backend/src/firefly/mod.rs`, `backend/src/db/mod.rs`, `backend/src/transactions/types.rs`, `backend/src/transactions/repository.rs`, `backend/src/transactions/service.rs`, `backend/src/ai/tools/transactions.rs`, `backend/src/ai/privacy.rs`, `backend/src/ai/registry.rs`, `sprints/quick/Q0010/uat.md`, `sprints/quick/Q0010/progress.md`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Backend unit tests | `cd backend && cargo test --lib` | **PASS** (102/102) |
| T-2 | Q1 category_id ingest | Static review + `firefly::sync::tests` | **PASS** |
| T-3 | Q2 ISO date parse | Static review + `parse_sync_date_*` tests | **PASS** |
| T-4 | Q3 signed amounts (DEC-0059) | Static review + `normalize_split_amount_*` tests | **PASS** |
| T-5 | R1 aggregate totals + period_status | Static review + `transactions::repository::tests` + `service::tests` | **PASS** |
| T-6 | R1 Uncategorized label | `label_uncategorized_categories_maps_null_name` | **PASS** |
| T-7 | DEC-0032 privacy boundary | `blocks_raw_rows_when_disabled` + `aggregate_json_includes_summary_fields_without_raw_rows` | **PASS** |
| T-8 | Six-tool registry unchanged | `registry_has_six_tools_matching_migration` | **PASS** |
| T-9 | Frozen boundaries | No SQL migration; no PrivacyLayer change; no payload amount rewrite | **PASS** |
| T-10 | Rows P/Q/R live smoke | Omniflow deploy + operator Firefly sync | **DEFERRED** — verify-work |
| T-11 | Regression footer (OIDC + bundled-firefly) | Operator smoke per acceptance | **DEFERRED** — verify-work (plan-verify ADV-1) |

### Environment dependencies (non-blocking)

- **Operator deploy:** Q0010 backend image to omniflow before live acceptance rows P/Q/R.
- **P1 gates P:** Manual Firefly sync after deploy to upsert-backfill ~922 mirror rows (DEC-0002; no SQL migration).
- **SQL probe:** Confirm non-NULL `category_id`, parsed dates, signed `amount < 0` outflow rows post-sync.

## Acceptance criteria matrix (BUG-0006)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **(Q)** | Firefly sync persists `category_id`; period filter returns rows with non-NULL dates and signed amounts for outflow sums | **PASS** (code) / **DEFERRED** (runtime) | Q1: `extract_category_id` + `upsert_transaction` binds `$6`; Q2: `parse_sync_date` chain + warn; Q3: `normalize_split_amount` per DEC-0059; journal fixture test. Live mirror backfill **DEFERRED** until operator sync |
| **(R)** | `get_transactions` aggregate JSON includes period totals/counts and distinguishes empty vs uncategorized/zero-outflow under `allow_raw_transactions=false` | **PASS** (code) / **DEFERRED** (live) | R1: `PeriodSummary` SQL + `TransactionAggregates` fields; `compute_period_status` priority; `label_uncategorized_categories`; privacy test confirms summary without `raw_rows`. Live tool curl **DEFERRED** until deploy |
| **(P)** | AI Chat answers category/spending questions using `get_transactions` data when mirror rows exist | **DEFERRED** | P1 gated on deploy + sync; downstream of Q1–R1. AI Chat smoke **DEFERRED** until operator sync |
| Regression | Privacy redaction + six-tool registry preserved; OIDC deploy checks | **PASS** (unit) / **DEFERRED** (live) | `registry_has_six_tools_matching_migration`; `blocks_raw_rows_when_disabled`; full OIDC smoke at verify-work |

**Summary:** Q1–Q3 + R1 **PASS** on static/automated path; full acceptance runtime deferred with `OPERATOR_DEPLOY_PENDING` + `OPERATOR_SYNC_PENDING`.

## Architecture compliance

### Ingest chain (Q1 → Q2 → Q3)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Q1 extract | First split `category_id`; empty → None | `extract_category_id` + sync path line 361–373 | PASS |
| Q1 upsert | `category_id` in INSERT/ON CONFLICT | `db/mod.rs` `upsert_transaction` binds `$6` | PASS |
| Q2 parse order | Strict date → RFC3339 → prefix | `parse_sync_date` lines 183–196 | PASS |
| Q2 failure | Warn, return None | `tracing::warn!` on parse failure | PASS |
| Q3 withdrawal | Positive raw → negative signed | `normalize_split_amount` `Some("withdrawal")` | PASS |
| Q3 deposit | Positive raw → positive signed | `Some("deposit")` | PASS |
| Q3 transfer | Positive raw → negative signed | `Some("transfer")` | PASS |
| Q3 unknown | Positive → outflow; non-positive keeps raw | DEC-0059 table + debug log | PASS |

### Aggregates (R1)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Period totals | count, outflow, inflow, uncategorized | `period_summary` SQL + `TransactionAggregates` assembly | PASS |
| period_status priority | no_rows → zero_outflow → uncategorized → with_outflow | `compute_period_status` + unit tests | PASS |
| Uncategorized label | NULL `category_id` → `"Uncategorized"` | `label_uncategorized_categories` in service | PASS |
| Privacy (DEC-0032) | Summary fields present; `raw_rows` gated | `allow_raw_transactions` branch; privacy + service tests | PASS |
| Tool description | Mentions totals + period_status | `ai/tools/transactions.rs` description | PASS |

### Frozen boundaries

| Boundary | Verdict |
|----------|---------|
| No SQL migration backfill — DEC-0002 upsert on next sync | PASS |
| No rewrite of aggregate SQL to read unsigned amounts from payload | PASS |
| No PrivacyLayer change | PASS |
| No merge with BUG-0002–0005 | PASS |
| Six-tool registry unchanged | PASS |

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

## Advisories (carry-forward from plan-verify)

| ID | Note |
|----|------|
| ADV-1 | OIDC + six-tool regression footer — operator verify-work smoke post-P1 |
| ADV-2 | P1 blocked until operator manual Firefly sync backfill |
| ADV-3 | First-split-only category model — acceptable MVP per architecture |
| ADV-5 | Six-tool registry live confirm at verify-work alongside five other tools |

## Next phase

**`/verify-work`** — after operator deploys Q1–R1 and runs manual Firefly sync:

1. SQL probe: mirror count, `category_id`, dates, signed amounts
2. `get_transactions` tool/curl: totals + `period_status` enum
3. AI Chat category/spending question (row P)
4. Regression footer: OIDC + privacy + six-tool registry

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
