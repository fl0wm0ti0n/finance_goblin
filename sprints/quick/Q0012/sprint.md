# Q0012 — BUG-0005 exchange sync multi-product (Bitunix futures)

| Field | Value |
|-------|-------|
| **ID** | Q0012 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0005 |
| **Created** | 2026-06-05 |
| **Architecture** | `architecture-20260605-bug0005` (`docs/engineering/architecture.md` § BUG-0005) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260605-q0012-bug0005`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0005 rows **(M)**, **(N)**, **(O)** |
| **Task count** | 6 |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0005 on US-0010 external omniflow: implement Bitunix futures header-auth client (**N1**), `effective_enabled_futures` policy (**N3**), futures wallet ingestion (**M1**), open positions sync (**N2**), dual-path test connection (**N4**); operator verify (**O1**) on `financegnome.omniflow.cc` after deploy + exchange sync.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| N — Futures REST infra | N1, N3 | backend exchanges + config |
| M — Wallet ingestion | M1 | backend bitunix |
| N — Positions + test | N2, N4 | backend bitunix |
| Verify | O1 | verify-work / acceptance mapping |

**Out of scope:** `sync_funding` implementation; Bybit/Binance relabeling; multi margin-coin beyond USDT; BUG-0006 merge.

## Task summary

| ID | Title | Est. | Depends | Acceptance hook |
|----|-------|------|---------|-----------------|
| N1 | Futures header-auth client (DEC-0062) | 2.5h | — | **(N)** |
| N3 | effective_enabled_futures (DEC-0063) | 1h | — | **(N)** |
| M1 | Futures wallet balance ingestion | 2h | N1, N3 | **(M)** |
| N2 | sync_positions (DEC-0064) | 2h | N1, N3 | **(M)**, **(N)** |
| N4 | Dual-path test_connection | 1.5h | N1, N3 | **(N)** |
| O1 | verify-work omniflow probes | 1h | N1–N4 deploy + exchange sync | **(M)(N)(O)** |

**Total estimate:** ~10h (dev ~9h + operator O1 ~1h).

## Deploy order

```text
(N1 + N3 + M1 + N2 + N4) single PR  →  deploy image  →  manual exchange sync
                                                      └→ O1 verify-work on financegnome.omniflow.cc
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **(M)** | M1, N2, O1 | Holdings include non-spot `product_type` when futures exposure exists |
| **(N)** | N1, N2, N3, N4, O1 | fapi header auth; sync + test populate data; effective futures enabled |
| **(O)** | M1, N2, O1 | Wealth crypto subtotal reflects spot + futures wallet |
| Regression | post-O1 | Acceptance footer (OIDC + bundled-firefly) |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
