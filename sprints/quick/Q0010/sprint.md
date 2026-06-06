# Q0010 — BUG-0006 AI get_transactions empty despite synced mirror rows

| Field | Value |
|-------|-------|
| **ID** | Q0010 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0006 |
| **Created** | 2026-06-05 |
| **Architecture** | `architecture-20260605-bug0006` (`docs/engineering/architecture.md` § BUG-0006) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260605-q0010-bug0006`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0006 rows **(P)**, **(Q)**, **(R)** |
| **Task count** | 5 |
| **Next phase** | `/execute` |

## Goal

Close BUG-0006 on US-0010 external omniflow: fix three Firefly mirror ingest gaps (**Q1** category, **Q2** ISO date, **Q3** amount sign per **DEC-0059**) and extend `get_transactions` aggregate contract (**R1**); operator verify (**P1**) on `financegnome.omniflow.cc` after deploy + manual Firefly sync backfill.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| Q — Mirror ingest | Q1, Q2, Q3 | backend (`firefly/mod.rs`, `db/mod.rs`) |
| R — Aggregate contract | R1 | backend transactions + AI tool |
| P — Operator verify | P1 | verify-work prep / acceptance mapping |

**Out of scope:** BUG-0002–0005; PrivacyLayer changes; SQL migration backfill; rewriting aggregate SQL to read unsigned amounts from `payload`.

## Task summary

| ID | Title | Est. | Depends | Acceptance hook |
|----|-------|------|---------|-----------------|
| Q1 | Firefly `category_id` sync | 2h | — | **(Q)** |
| Q2 | ISO date parse in sync | 1.5h | Q1 | **(Q)** |
| Q3 | Amount sign normalization (DEC-0059) | 2h | Q2 | **(Q)** |
| R1 | TransactionAggregates contract extension | 3h | Q3 | **(R)** |
| P1 | verify-work prep / acceptance mapping | 1h | R1 deploy + sync | **(P)** |

**Total estimate:** ~9.5h (dev ~8.5h + operator P1 ~1h).

## Deploy order

```text
(Q1 → Q2 → Q3 → R1) single backend PR  →  deploy image  →  manual Firefly sync (backfill ~922 rows)
                                                              └→ P1 verify-work on financegnome.omniflow.cc
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **(Q)** | Q1, Q2, Q3 | Mirror rows have `category_id`, non-NULL dates, signed amounts; aggregates non-empty for period |
| **(R)** | R1 | Tool JSON includes totals, `uncategorized_transaction_count`, `period_status`; `raw_rows` absent when privacy off |
| **(P)** | P1 | AI Chat category/spending question uses aggregate data; SQL probe matches tool totals |
| Regression | post-P1 | Acceptance footer (OIDC + privacy constraints) |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
