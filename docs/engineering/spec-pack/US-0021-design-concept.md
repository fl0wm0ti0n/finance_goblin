# Design Concept — US-0021

## Summary

US-0021 extends subscription Discover with a **transaction-first explorer**: paginated individual expense transactions, rich filters (account, payee, category, Geldbereich, date), recurring **hint badges** on filtered subsets, and **multi-select manual activate** — while preserving the US-0020 **Suggested patterns** tab unchanged.

## Goals

- Paginated transaction ledger search on Discover (AC-1)
- Rich filters including category and Geldbereich (AC-2)
- Pattern hints on filtered transactions without lowering global detection thresholds (AC-3)
- Multi-select tx group → confirm as subscription/standing order (AC-4)
- US-0020 tags, majority category, and auto-detection unchanged (AC-5)
- OIDC external profile smoke on new flows (AC-6)

## Non-goals

- Replacing the recurrence-candidate Patterns tab (DEC-0098 preserved)
- Firefly write-back
- Changes to auto-detection `min_emit_confidence` or pending/alert paths
- Keyset pagination MVP
- All-accounts unbounded search
- 2-tx weak hints (P2 GATE-HINT-2)
- Composite DB index (P2 GATE-IDX-1)

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0112 | New `GET /transactions/search` + preview-group | AC-1/AC-2; avoids breaking DEC-0098 |
| DEC-0113 | Dual mode: Transactions default \| Patterns | Operator transaction-first + US-0020 preserve |
| DEC-0114 | Separate hint pass; min 60; display only | AC-3 without AC-5 regression |
| DEC-0099 | Reuse `POST /discover/confirm` | AC-4; DEC-0085 merge unchanged |
| DEC-0111 | Geldbereich filter + labels | Reuse proven COALESCE + formatAccountRole |

## User experience

On `/subscriptions` **Discover**, operators land on **Transactions** mode by default. They pick an account, optionally filter by payee, category, Geldbereich, and date range, then browse up to 100 transactions per page. Rows that form recurring patterns show a **hint badge** (interval + confidence). Operators multi-select related rows and confirm as subscription or standing order — merged when payee+interval matches an existing pattern. The **Suggested patterns** sub-tab keeps the US-0020 recurrence-candidate workflow unchanged.
