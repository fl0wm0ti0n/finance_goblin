# Plan-verify findings — Q0017 / BUG-0007

**Date:** 2026-06-07  
**Role:** QA (`/plan-verify`)  
**Orchestrator:** `auto-20260607-bug0007-001`  
**Verdict:** **PASS**

## Test plan (plan-verify scope)

| Check | Source | Pass criteria |
|-------|--------|---------------|
| Acceptance triad | `acceptance.md` BUG-0007 rows S/T/U | Each row maps to ≥1 task with verify steps |
| Task completeness | `tasks.md`, `task.json` | 7 tasks; no orphans; dependencies acyclic |
| Architecture alignment | `architecture.md` § BUG-0007 | A1, A2, F1, E1, E2, T1, V1 match DEC-0069 frozen contracts |
| Decision alignment | `DEC-0069.md` | A′+E+F bundle; six-tool preserved; BUG-0008 coordinate-only |
| Discovery coverage | `backlog.md` BUG-0007 | Sub-defects S/T/U addressed; V (RAG) deferred per note |
| Frozen boundaries | `task.json` | No seventh tool; privacy default unchanged; no BUG-0008 merge |
| UAT readiness | V1 task spec | `uat.md` + operator AI Chat smoke S/T/U + regression footer planned |

## Findings

### Coverage matrix

| Row | Criterion (summary) | Tasks | Covered |
|-----|---------------------|-------|---------|
| **(S)** | AI enumerates subscription/streaming merchant/payee names from `get_subscriptions` and/or `get_transactions` when operator asks to list services — not generic-only or "cannot retrieve" when mirror holds recurring patterns | F1, E1, E2, V1 | Yes |
| **(T)** | Merchant/category queries (Strom, Amazon, streaming) return data-backed amounts or explicit empty-state — not blanket "no expenses" when mirror plausibly contains matches | A1, A2, E1, E2, V1 | Yes |
| **(U)** | AI fuses category, subscription, account, and amount signals in tool orchestration without requiring user-supplied merchant names | A1, A2, F1, E1, V1 | Yes |
| Regression | Six-tool registry; `allow_raw_transactions=false` default; OIDC/bundled-firefly footer | T1, V1 | Yes (footer advisory) |

### Task → acceptance map

| Task | Acceptance hooks | DEC-0069 slice |
|------|------------------|----------------|
| A1 | **(T)**, **(U)** | ILIKE category resolve cap 10; global mirror MIN/MAX bounds |
| A2 | **(T)**, **(U)** | `category_search` param; response extensions; precedence rules |
| F1 | **(S)** | `kind` enum; `merchant_names[]`, `patterns_count`; Counterparty-* guard |
| E1 | **(S)**, **(T)**, **(U)** | Four SYSTEM_PROMPT rules; audit `result_rows` for both tools |
| E2 | **(S)**, **(T)** | Parameter schema descriptions for category_id vs category_search |
| T1 | regression | Frozen test contract from architecture § Test strategy |
| V1 | **(S)**, **(T)**, **(U)** | verify-work omniflow AI Chat smoke after **BACKEND_DEPLOY** |

### Dependency review

- **Order:** A1 → A2; F1 parallel with A1; E1 → E2 after A2+F1; T1 after A1–E2; single backend PR deploy → V1
- **Circular deps:** none
- **Operator gates:** **BACKEND_DEPLOY** before V1 runtime probes on `financegnome.omniflow.cc`

### DEC-0069 contract checklist

| Contract element | Sprint task | Status |
|------------------|-------------|--------|
| A′ `category_search` ILIKE resolve cap 10 | A1, A2 | Mapped |
| A′ `category_search` wins over `category_id` | A2 | Mapped |
| A′ `mirror_date_bounds` on every response | A1, A2 | Mapped |
| A′ T-a empty period cites bounds | A2, E1, V1 | Mapped |
| F `kind` enum + `merchant_names[]` + `patterns_count` | F1 | Mapped |
| F Counterparty-* enum guard | F1 | Mapped |
| F REST `list_patterns` unchanged (BUG-0008) | F1, frozen_boundaries | Mapped |
| E four SYSTEM_PROMPT rules | E1 | Mapped |
| E audit `result_rows` population | E1 | Mapped |
| E parameter schema descriptions | E2 | Mapped |
| Six-tool registry unchanged | T1, frozen_boundaries | Mapped |
| RAG (V) deferred | frozen_boundaries | Respected |
| Payee aggregates (B) deferred | frozen_boundaries | Respected |

### Gaps

**0 gaps** — all acceptance rows S/T/U have primary task coverage with executable verify steps aligned to DEC-0069.

### Orphans

**0 orphans** — all seven tasks reference S, T, U, and/or regression hooks.

### Advisories (non-blocking)

1. **ADV-1:** Acceptance **(T)** text mentions "category/description/account search"; frozen fix implements **category_search + mirror_date_bounds** only (description/payee search rejected under DEC-0032). Discovery split T-a/T-b and architecture § BUG-0007 reconcile scope — not a plan-verify blocker.
2. **ADV-2:** Acceptance **(U)** mentions transaction name/description; fix path is **multi-tool fusion via category names + subscription display_names + orchestrator prompt** (DEC-0069 U mapping). Payee/description search deferred — execute must not expand scope without new DEC.
3. **ADV-3:** OIDC + bundled-firefly regression footer has no dedicated dev task; operator verify-work post-V1 — precedent from Q0016 ADV-1.
4. **ADV-4:** V1 **(U)** probe lacks a frozen German prompt script (unlike S/T); execute should define a natural-language query that requires category + subscription fusion without merchant names (e.g. utility + streaming spend overview).
5. **ADV-5:** `backend/src/ai/registry.rs` six-tool verification listed in DEC-0069 anchor — covered by T1 assertion, not a standalone task (acceptable).
6. **ADV-6:** LLM may still ignore enriched payloads — mitigated by E1 prompt + E2 schema + audit observability; runtime proof deferred to V1/verify-work.
7. **ADV-7:** Integration tests require operator `DATABASE_URL` (TimescaleDB) — documented in state carry-forward; T1 may skip integration in CI without fixture URL.

## Recommendation

Approve sprint for **`/execute`**. No `handoffs/qa_to_dev.md` fix list required.
