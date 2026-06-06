# Verify-work Findings — Quick Q0017 / BUG-0007 (re-run)

**Work item:** BUG-0007 (defect)  
**Quick task:** Q0017  
**Phase:** `/verify-work` (re-run after S privacy fix)  
**Orchestrator:** `auto-20260607-bug0007-001`  
**Date:** 2026-06-07  
**Verdict:** **PASS** — row **(S)** unblocked; **(T)** partial (non-blocking advisory); **(U)** pass; proceed **`/release`**

## Summary

Re-ran verify-work after execute S-privacy fix and operator **BACKEND_DEPLOY** (`AUTHENTIK_SECRET_KEY=unused-external-profile docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai`). Container healthy on `https://financegnome.omniflow.cc` (922 transactions, `allow_raw_transactions: false`). V1 AI Chat smoke confirms subscription merchant names are preserved in `get_subscriptions` tool output — **(S)** passes. **(T)** retains prior partial status (LLM `group_by: month` + `category_search` advisory; undated period defaults). **(U)** multi-tool fusion with named merchants passes now that S is unblocked.

## Operator gate

| Gate | Status |
|------|--------|
| BACKEND_DEPLOY | **CLEARED** — container recreated 2026-06-07 |
| Mirror data (922 tx) | **PASS** |
| AI provider configured | **PASS** (OpenAI gpt-4o-mini) |

## Local gates

| Check | Result |
|-------|--------|
| `cargo test --lib` | **PASS** (150/150) |
| `cargo test --test bug0007_ai_discovery` | **PASS** (8/8) |
| `get_subscriptions_preserves_display_name_and_merchant_names` | **PASS** |
| `get_subscriptions_still_redacts_other_long_strings` | **PASS** |

## Per-row verdict (acceptance S / T / U)

| Row | Verdict | Summary |
|-----|---------|---------|
| **(S)** | **PASS** | S-1 lists **YouTube 12,99 €**, **Netflix 6,37 €**, **Mitgliedsbeitrag - Florian Gabriel 21,00 €** from tool data. S-2 enumerates **12 named merchants** (GOOGLE*YOUTUBE, NETFLIX, CURSOR, APPLE.COM/BILL, etc.) — **no Counterparty-*** in LLM output. |
| **(T)** | **PARTIAL** | **T-a PASS:** Amazon Jan–Oct 2023 empty cites mirror window **05. Juni 2025 bis 22. Mai 2026**. **T-b PASS (no group_by):** Strom **465,53 €** via `category_search: strom`. **T-b advisory:** LLM often adds `group_by: month` → unfiltered **68.096,34 €** totals for Strom/Amazon; undated queries may default to 2023. |
| **(U)** | **PASS** | U-1 invokes `get_subscriptions` + `get_transactions` without user merchant names; response lists named subscription merchants and fuses spend data for requested window. |

| Regression | Verdict |
|------------|---------|
| REG-1 six-tool registry | **PASS** (`bug0007_ai_discovery` 8/8) |
| REG-2 privacy default | **PASS** — `allow_raw_transactions: false`; payee top-spend still **Counterparty-*** |

## Live probe evidence (2026-06-07 re-run)

| Step | Probe | HTTP | Tools | Result |
|------|-------|------|-------|--------|
| Health | `GET /api/v1/settings` (via omniflow) | 200 | — | `database_mode: external`, privacy aggregate-only |
| Entities | `GET /api/v1/sync/entities` | 200 | — | 922 transactions, 75 categories |
| S-1 | Cancelable streaming total | 200 | `get_subscriptions` | **PASS** — YouTube, Netflix, Mitgliedsbeitrag named |
| S-2 | *"liste mir die dienste auf"* | 200 | `get_subscriptions` | **PASS** — 12 named merchants, no Counterparty-* |
| T-b-1r | Strom Jun 2025–May 2026 (no group_by) | 200 | `get_transactions` | **PASS** — **465,53 €**, `category_search: strom` |
| T-b-2r | Amazon Jun 2025–May 2026 | 200 | `get_transactions` | **PARTIAL** — `category_search: amazon`; LLM `group_by: month` → 68k (service gap) |
| T-a | Amazon Jan–Oct 2023 | 200 | `get_transactions` | **PASS** — empty + cites mirror bounds |
| U-1 | Subs + online spend (explicit window) | 200 | `get_subscriptions`, `get_transactions` | **PASS** — fusion + named merchants |
| REG-2 | Top payees May 2026 | 200 | `get_transactions` | **PASS** — Counterparty-* redaction preserved |

### Audit highlights (`GET /api/v1/ai/audit`)

| Session probe | Tool | Args (summary) | Status |
|---------------|------|----------------|--------|
| S-1 | `get_subscriptions` | `{}` | ok |
| S-2 | `get_subscriptions` | `{}` | ok |
| T-b-1r | `get_transactions` | `category_search: strom`, 2025-06-01..2026-05-31 | ok |
| T-a | `get_transactions` | `category_search: amazon`, 2023-01-01..2023-10-31 | ok |
| U-1 | `get_subscriptions` + `get_transactions` | category + period args | ok |

## Root cause resolution (S)

`backend/src/ai/privacy.rs` — `walk_value_for_tool` preserves `display_name` / `merchant_names` strings under `get_subscriptions`; other long strings and `get_transactions` payee fields still hash to Counterparty-*.

## Release gate

| Gate | Status |
|------|--------|
| QA PASS (code) | yes |
| Verify-work PASS | **yes** |
| Acceptance S/T/U | S **PASS**, T **PARTIAL** (non-blocking), U **PASS** |
| Release proceed | **yes** |

## Operator advisory (non-blocking)

1. **T-b `group_by: month`:** When LLM adds `group_by: month` with `category_search`, monthly totals may be unfiltered — prompt or service fix deferred.
2. **Undated queries:** LLM may default to 2023 empty window — mirror-window explicit phrasing recommended.

## Next steps

1. **`/release`** — check BUG-0007 acceptance; publish release notes

---

**Stop here.** Continue in a **new** subagent/chat with **`/release`**.
