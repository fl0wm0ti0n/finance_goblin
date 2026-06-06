# Verify-work → Dev Handoff

**From:** Verify-work (`/verify-work`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-07  
**Work item:** BUG-0007  
**Quick task:** Q0017  
**Orchestrator:** `auto-20260607-bug0007-001`

## Verdict

**FAIL** — V1 omniflow smoke after **BACKEND_DEPLOY**; row **(S)** blocked. **(T)** / **(U)** partial — core `category_search` / bounds / multi-tool paths work; privacy layer undoes subscription enumeration.

## Per-row verdict

| Row | Verdict | Action |
|-----|---------|--------|
| **(S)** | **FAIL** | Exempt subscription `display_name` / `merchant_names[]` from `PrivacyLayer` counterparty hashing in `get_subscriptions` tool output |
| **(T)** | **PARTIAL** | Optional: apply category filter for `group_by: month` + `category_search`; prompt mirror-window default |
| **(U)** | **PARTIAL** | Unblocked when **(S)** fixed |
| Regression | **PASS** | No action |

## What passed

- **BACKEND_DEPLOY** — new image running, health 200
- **category_search** in audit (replaces pre-fix `category_id: "amazon"` / `"Strom"`)
- Amazon **1,079.35 €**, Strom **465.53 €** in mirror window via AI Chat
- Amazon Jan–Oct 2023 empty-state cites **mirror_date_bounds**
- Multi-tool fusion without user-supplied merchant names
- Six-tool registry; `allow_raw_transactions: false`

## What failed

- **S-1 / S-2:** LLM sees `Counterparty-*` instead of merchant names (YouTube, Apple, Cursor, etc.) because `PrivacyLayer::walk_value` redacts all long strings in tool JSON
- **T-b undated queries:** LLM picks 2023 window → false empty (operator phrasing advisory)
- **T-b-1 + group_by month:** Returns all-outflow monthly totals when category_search set (service gap)

## Recommended fix (minimal, S blocker)

In `backend/src/ai/privacy.rs` (or tool-specific path in `GetSubscriptionsTool`):

- Do **not** hash `display_name`, `merchant_names`, or subscription pattern label fields when redacting `get_subscriptions` results
- Preserve `redact_counterparties` for transaction payee/description fields

## Operator steps (post-fix)

1. Re-deploy backend (same compose command as verify-work)
2. Re-run **`/verify-work`** — S-1/S-2 probes in `sprints/quick/Q0017/uat.md`

## Evidence

- `sprints/quick/Q0017/verify-work-findings.md`
- `handoffs/verify_work_report.md`
- `sprints/quick/Q0017/uat.md`

---

**Next phase:** **`/execute`** in a fresh subagent/chat. **Do not `/release`** until verify-work PASS.
