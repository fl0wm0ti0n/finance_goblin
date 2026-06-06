# Tasks — Q0017 (BUG-0007)

**Bug:** BUG-0007  
**Task count:** 7 (within `SPRINT_MAX_TASKS=12`)  
**Sprint-plan ref:** `sprint-plan-20260607-q0017-bug0007`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **A1** | Task **A1** | ILIKE category resolve cap 10; mirror MIN/MAX bounds query |
| **A2** | Task **A2** | `category_search` param; `category_matches[]`, `mirror_date_bounds`, `search_attempted` |
| **F1** | Task **F1** | `kind` enum; `merchant_names[]` + `patterns_count`; Counterparty-* guard |
| **E1** | Task **E1** | SYSTEM_PROMPT four rules; audit `result_rows` for both tools |
| **E2** | Task **E2** | Enrich `category_id` + `category_search` parameter descriptions |
| **T1** | Task **T1** | Frozen test contract from architecture § Test strategy |
| **V1** | Task **V1** | verify-work AI Chat smoke rows S/T/U; six-tool footer |

## Execute order

```text
A1 → A2
F1 (parallel with A1 once contracts clear)
  → E1 → E2
  → T1
  → single backend PR deploy
  → V1 verify-work
```

**Parallelism:** F1 independent of A1/A2 chain; E1 requires A2 + F1; T1 after A1–E2.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **(S)** | F1, E1, E2, V1 | Named subscription merchants after cancelable-total question |
| **(T)** | A1, A2, E1, E2, V1 | Strom/Amazon via category_search; 2023 Amazon cites mirror_date_bounds |
| **(U)** | A1, A2, F1, E1, V1 | Multi-tool fusion without user-supplied merchant names |
| Regression | T1, V1 | Six tools; `allow_raw_transactions=false` default |

| ID | Title | Est. | Status | Acceptance |
|----|-------|------|--------|------------|
| A1 | Category search SQL + mirror bounds | 3h | open | **(T)**, **(U)** |
| A2 | Tool schema + response assembly | 2h | open | **(T)**, **(U)** |
| F1 | Subscriptions schema + response + guard | 2h | open | **(S)** |
| E1 | SYSTEM_PROMPT + audit result_rows | 2h | open | **(S)**, **(T)**, **(U)** |
| E2 | Parameter schema descriptions | 0.5h | open | **(S)**, **(T)** |
| T1 | Unit/integration tests | 3h | open | regression |
| V1 | verify-work omniflow AI Chat smoke | 1h | open | **(S)**, **(T)**, **(U)** |

---

## A1 — Category search SQL + mirror bounds

**Status:** open  
**Depends on:** —  
**Estimate:** 3h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0007 **(T)**, **(U)**

### Description

Add repository methods: ILIKE `categories.name` resolve (cap 10, `category_matches_truncated` when exceeded); global mirror `MIN(date)` / `MAX(date)` bounds query. Extend types for `category_matches[]`, `mirror_date_bounds`, `search_attempted`.

**Files:** `backend/src/transactions/repository.rs`, `backend/src/transactions/types.rs`

### Done when

- [ ] ILIKE resolves `"strom"` → id 146, `"amazon"` → id 47 on fixture data
- [ ] Bounds query returns mirror date range (omniflow proof: 2025-06-05…2026-05-22)
- [ ] Cap 10 enforced; truncation flag when exceeded
- [ ] SQL matches DEC-0069 frozen contract

---

## A2 — Tool schema + response assembly

**Status:** open  
**Depends on:** A1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0007 **(T)**, **(U)**

### Description

Wire A1 into `TransactionService` and `get_transactions` AI tool: optional `category_search` param (min 2 chars after trim); `category_search` wins over `category_id` when both supplied; response always includes `mirror_date_bounds`, `category_matches[]`, `search_attempted`. Privacy: no `raw_rows`; DEC-0032 unchanged.

**Files:** `backend/src/transactions/service.rs`, `backend/src/ai/tools/transactions.rs`

### Done when

- [ ] Tool schema exposes `category_search` with min-length validation
- [ ] Response JSON includes all frozen fields per DEC-0069 A′
- [ ] Empty keyword match returns `category_matches: []`, `search_attempted: true`
- [ ] `allow_raw_transactions=false` → no `raw_rows`

---

## F1 — Subscriptions schema + response + guard

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0007 **(S)**

### Description

Enrich `get_subscriptions` AI tool: `kind` enum (`subscription` | `standing_order`); add `patterns_count`, `merchant_names[]` (deduped `display_name`); reject `Counterparty-*` prefix in `status`/`kind` → InvalidArgs. REST `list_patterns` behavior unchanged (BUG-0008 isolation).

**Files:** `backend/src/ai/tools/subscriptions.rs`

### Done when

- [ ] OpenAI schema includes `kind` enum values
- [ ] Response adds `patterns_count` and `merchant_names[]`
- [ ] Counterparty-* args rejected with InvalidArgs
- [ ] No change to alert count, list filters, or detection thresholds

---

## E1 — SYSTEM_PROMPT + audit result_rows

**Status:** open  
**Depends on:** A2, F1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0007 **(S)**, **(T)**, **(U)**

### Description

Update `SYSTEM_PROMPT` with four frozen rules: enumerate all subscription `display_name`; use `category_search` for merchant/category keywords; cite `mirror_date_bounds` on empty period; never pass `Counterparty-*` as enum filters. Populate `audit.result_rows`: bucket count (or `total_transaction_count`) for `get_transactions`; `patterns_count` for `get_subscriptions`. No `tool_choice: required` (DEC-0045).

**Files:** `backend/src/ai/orchestrator.rs`

### Done when

- [ ] Four prompt rules match DEC-0069 E contract
- [ ] Audit insert receives non-NULL `result_rows` for both tools
- [ ] Local provider path unchanged (no forced tool_choice)

---

## E2 — Parameter schema descriptions

**Status:** open  
**Depends on:** A2, F1  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0007 **(S)**, **(T)**

### Description

Enrich OpenAI function parameter descriptions: `category_id` vs `category_search` distinction; `category_search` min 2 chars; `kind`/`status` valid enum values; explicit warning against `Counterparty-*` values.

**Files:** `backend/src/ai/tools/transactions.rs`, `backend/src/ai/tools/subscriptions.rs`

### Done when

- [ ] Schema descriptions guide LLM away from keyword-as-category_id misuse
- [ ] Enum guard rationale documented in parameter docs

---

## T1 — Unit/integration tests

**Status:** open  
**Depends on:** A1, A2, F1, E1, E2  
**Estimate:** 3h  
**Acceptance hook:** architecture § Test strategy (regression)

### Description

Frozen test contract: A1 ILIKE + bounds fixtures; A2 tool response shape; F1 Counterparty guard + merchant_names dedup; E1 audit result_rows; privacy six-tool count + `allow_raw_transactions=false` → no `raw_rows`.

**Files:** `backend/tests/` or module `#[cfg(test)]`

### Done when

- [ ] All architecture § Test strategy checks have automated coverage
- [ ] `cargo test` PASS (or documented DATABASE_URL prerequisite)
- [ ] Six-tool registry count asserted

---

## V1 — verify-work omniflow AI Chat smoke

**Status:** open  
**Depends on:** A1–E2, T1 deploy  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0007 **(S)**, **(T)**, **(U)**

### Description

Prepare `sprints/quick/Q0017/uat.md` smoke checklist. After backend deploy, probe `financegnome.omniflow.cc` AI Chat:

1. **S:** cancelable streaming total → *"liste mir die dienste auf"* → named merchants from tool data
2. **T-b:** Strom and Amazon spend in mirror window → category_search-backed amounts
3. **T-a:** Amazon Jan–Oct 2023 → explicit empty-state citing `mirror_date_bounds` (2025-06-05…)
4. Regression: six-tool count; `allow_raw_transactions=false`

**Files:** `sprints/quick/Q0017/uat.md`, `docs/engineering/runbook.md` (optional)

### Done when

- [ ] Row **(S)**: subscription merchant enumeration PASS
- [ ] Row **(T)**: Strom/Amazon amounts + 2023 bounds empty-state PASS
- [ ] Row **(U)**: multi-tool fusion without user merchant names PASS
- [ ] Six-tool + privacy footer PASS

**Operator gate:** Backend deploy after A1–E2+T1 before V1 runtime probes.
