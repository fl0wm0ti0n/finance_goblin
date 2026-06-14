# Technical Specification — US-0021

## Overview

Implement **DEC-0112** transaction search API, **DEC-0113** dual-mode Discover UX, and **DEC-0114** hint pass boundary — extending US-0020 without modifying auto-detection or DEC-0098 patterns contract.

## Components

| Layer | Change |
|-------|--------|
| `backend/src/subscriptions/repository.rs` | **Extend** — parameterized tx search SQL + COUNT + accounts JOIN |
| `backend/src/subscriptions/discovery.rs` | **Extend** or new `transaction_search.rs` — search orchestration + hint pass |
| `backend/src/api/subscriptions.rs` | **Extend** — GET search, POST preview-group |
| `backend/src/recurrence/detect.rs` | **Read-only** reuse — no threshold edits |
| `backend/src/subscriptions/detection.rs` | **Unchanged** — AC-5 guard |
| `frontend/src/pages/SubscriptionsPage.tsx` | **Extend** — dual mode, filters, tx table, multi-select confirm |
| `frontend/src/lib/api.ts` | **Extend** — search + preview types |
| `docs/user-guides/US-0021.md` | **New** — at execute |

## Interfaces

### `GET /api/v1/subscriptions/transactions/search`

**Query:** `account_id` (required), `payee`, `category_id`, `account_role`, `date_from`, `date_to`, `amount_min`/`amount_max` (P2), `recurring_hint`, `page`, `limit` (max 100)

**Response:** `transactions[]` with optional `recurring_hint`; `meta { page, limit, total_count, has_more, truncated, window_days }`

### `POST /api/v1/subscriptions/transactions/preview-group`

**Body:** `{ "transaction_ids": ["…"] }` (≥2)

**Response:** `{ payee_key, interval_days, median_amount, transaction_ids }`

### `POST /api/v1/subscriptions/discover/confirm` (unchanged — DEC-0099)

Preview-group output feeds existing confirm body. Merge/rejection rules unchanged.

### `GET /api/v1/subscriptions/discover` (unchanged — DEC-0098)

Patterns sub-tab only. No contract change.

## Hint pass (internal)

1. Load SQL-filtered txs (≤500 for hint scan)
2. `detect_recurrence_groups` with `min_emit_confidence: 60`
3. Attach `recurring_hint` to matching rows
4. No writes, no alerts, no pending creation

## Sequencing (S0020 sprint-plan input)

1. TX1 repository SQL → TX2 search + hints → TX3 routes
2. UI1 dual-mode shell → PT1 patterns extraction
3. UI2 filters → UI3 table → UI4 confirm flow
4. T1 integration tests → T2 regression → R1 user guide
5. V1 OIDC smoke

## Verification

- Integration: search filters (category, role, date); pagination cap 100; hint on account 114 SEPA fixture
- Integration: preview-group → confirm merge/create; 409 on rejected payee-interval
- Regression: `DetectionPipeline` ordering unchanged; `run_discover` candidate cap 50 unchanged
- UI: Transactions default; Patterns tab DEC-0098 parity; truncated banner copy
- Operator repro: `localhost:18080`, account **114**, SEPA-Lastschrift individual rows + hint badge
