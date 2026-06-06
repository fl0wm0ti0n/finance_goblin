# Sprint Summary — Q0017 (BUG-0007)

**Bug:** BUG-0007  
**Sprint:** Q0017 (`/quick`)  
**Release date:** 2026-06-07  
**Status:** **RELEASED** — verify-work PASS; release PASS

## Release summary

DEC-0069 A′+E+F bundle shipped: `category_search` on `get_transactions`, enriched `get_subscriptions` schema, orchestrator prompt + audit `result_rows`, S privacy label exemption for subscription merchant names.

| Row | Verdict | Notes |
|-----|---------|-------|
| **(S)** | PASS | 12 named merchants in AI Chat (no Counterparty-*) |
| **(T)** | PARTIAL | Strom **465,53 €** pass; `group_by: month` advisory non-blocking |
| **(U)** | PASS | Multi-tool fusion with named merchants |

## Verify-work loop fix (2026-06-07)

| Finding | Fix |
|---------|-----|
| S-1/S-2 — PrivacyLayer hashes subscription merchant names to Counterparty-* | Exempt `display_name` / `merchant_names[]` in `get_subscriptions` tool output (`privacy.rs`) |

## Prior QA loop fix (2026-06-07)

| Finding | Fix |
|---------|-----|
| B-1 — missing `GroupBy` import in `repository.rs` tests | Added `GroupBy` to test module imports |

## Tasks delivered

| ID | Status | Summary |
|----|--------|---------|
| A1 | done | ILIKE category search (cap 10, truncation flag); global mirror MIN/MAX bounds query |
| A2 | done | `category_search` param + response fields; precedence over `category_id`; privacy unchanged |
| F1 | done | `kind` enum; `merchant_names[]` + `patterns_count`; Counterparty-* guard |
| E1 | done | SYSTEM_PROMPT four discovery rules; audit `result_rows` for both tools |
| E2 | done | Parameter schema descriptions for category_id/category_search and enum filters |
| T1 | done | `bug0007_ai_discovery.rs` + module unit tests |
| S-fix | done | Privacy exemption for subscription label fields in `get_subscriptions` |
| V1 | done | Omniflow AI Chat smoke — verify-work PASS |

## Tests

| Check | Result |
|-------|--------|
| `cargo test --lib` | **PASS** (150/150) |
| `cargo test --test bug0007_ai_discovery` | **PASS** (8/8) |
| Six-tool registry | **PASS** |
| `allow_raw_transactions=false` → no `raw_rows` | **PASS** |
| Subscription label privacy exemption | **PASS** (2 unit tests) |

## Evidence

- Decision: **DEC-0069**
- Verify-work: `sprints/quick/Q0017/verify-work-findings.md`, `handoffs/verify_work_to_release.md`
- UAT: `sprints/quick/Q0017/uat.md`
- Archive: `docs/engineering/state-archive/state-pack-20260607-q0017-bug0007.md`

## Known advisory (non-blocking)

LLM may combine `group_by: month` with `category_search` producing inflated totals — documented at release; future prompt/tool guard optional.

## Next queue item

**BUG-0008** — subscription alerts vs list mismatch & under-detection (coordinate: preserve REST list + alert semantics)
