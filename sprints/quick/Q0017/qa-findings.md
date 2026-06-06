# QA Findings — Quick Q0017 / BUG-0007 (S-privacy re-run)

**Work item:** BUG-0007 (defect)  
**Quick task:** Q0017  
**QA phase:** `/qa` (re-run after S privacy fix)  
**Date:** 2026-06-07  
**Orchestrator:** `auto-20260607-bug0007-001`  
**Verdict:** **PASS** (code QA complete; V1 omniflow smoke deferred until **BACKEND_DEPLOY**)

## Scope

Verify-work loop fix for **(S)** — exempt subscription `display_name` and `merchant_names[]` from `PrivacyLayer` counterparty hashing in `get_subscriptions` output (`handoffs/dev_to_qa.md`, `backend/src/ai/privacy.rs`). Prior A1–E2 + T1 scope unchanged per **DEC-0069**.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0017/summary.md`, `sprints/quick/Q0017/uat.md`, `decisions/DEC-0069.md`, `backend/src/ai/privacy.rs`, `backend/tests/bug0007_ai_discovery.rs`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Full lib regression | `cd backend && cargo test --lib` | **PASS** (150/150) |
| T-2 | S privacy — labels preserved | `get_subscriptions_preserves_display_name_and_merchant_names` | **PASS** |
| T-3 | S privacy — other strings hashed | `get_subscriptions_still_redacts_other_long_strings` | **PASS** |
| T-4 | BUG-0007 contract suite | `cargo test --test bug0007_ai_discovery` | **PASS** (8/8) |
| T-5 | Six-tool registry | `six_tool_registry_unchanged` | **PASS** |
| T-6 | A1–E2 + T1 prior scope | Prior qa-findings matrix (unchanged) | **PASS** (static) |
| T-7 | Rows S/T/U live smoke | Omniflow deploy + AI Chat probes | **DEFERRED** — verify-work (V1) |

### Test output (T-1)

```
running 150 tests
...
test ai::privacy::tests::get_subscriptions_preserves_display_name_and_merchant_names ... ok
test ai::privacy::tests::get_subscriptions_still_redacts_other_long_strings ... ok
...
test result: ok. 150 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.23s
```

### Test output (T-4)

```
running 8 tests
test get_transactions_schema_has_category_search ... ok
test get_subscriptions_schema_has_kind_enum ... ok
test category_search_cap_ten_with_truncation_flag ... ok
test category_search_ilike_resolves_strom_and_amazon ... ok
test mirror_date_bounds_from_transactions ... ok
test category_search_short_keyword_rejected ... ok
test aggregates_include_discovery_fields ... ok
test six_tool_registry_unchanged ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Acceptance criteria matrix (BUG-0007)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **(S)** | Chat enumerates subscription merchant names; no Counterparty-* on labels | **PASS** (code) / **DEFERRED** (runtime) | S fix: `walk_value_for_tool` preserves label keys; 2 new privacy unit tests PASS. Live S-1/S-2 **DEFERRED** |
| **(T)** | Strom/Amazon via category resolution; 2023 empty cites mirror_date_bounds | **PASS** (code) / **DEFERRED** (runtime) | A1/A2 unchanged; prior tests PASS. Live T-a/T-b **DEFERRED** |
| **(U)** | Multi-tool fusion without user-supplied merchant names | **PASS** (code) / **DEFERRED** (runtime) | A′+F+E static path unchanged. Live U-1 **DEFERRED** |
| Footer | Six tools; `allow_raw_transactions=false` default | **PASS** | Six-tool test PASS; privacy default PASS; `cargo test --lib` 150/150 PASS |

## S privacy fix (verify-work loop)

| Finding | Fix | Verified |
|---------|-----|----------|
| S-1/S-2 — LLM sees Counterparty-* instead of merchant names | Tool-aware walk preserves `display_name` / `merchant_names` for `get_subscriptions` | **PASS** — unit tests T-2, T-3 |
| Regression — counterparty redaction elsewhere | Other strings still hashed; IBAN redaction still applies | **PASS** — T-3 |

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

## Verify-work readiness

| Gate | Status |
|------|--------|
| Code QA (A1–E2 + S privacy fix) | **READY** |
| `cargo test --lib` regression | **READY** — 150/150 PASS |
| Operator deploy Q0017 backend | **PENDING** |
| V1 omniflow AI Chat smoke (S/T/U rows) | **PENDING** — blocked on **BACKEND_DEPLOY** |

## Next phase

**`/verify-work`** — operator **BACKEND_DEPLOY** then V1 omniflow AI Chat smoke per `sprints/quick/Q0017/uat.md` (S-1/S-2 re-run).

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
