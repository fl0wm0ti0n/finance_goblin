# Q0017 — BUG-0007 AI merchant/category discovery

| Field | Value |
|-------|-------|
| **ID** | Q0017 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0007 |
| **Created** | 2026-06-07 |
| **Architecture** | `architecture-20260607-bug0007` (`docs/engineering/architecture.md` § BUG-0007) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260607-q0017-bug0007`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0007 rows **(S)**, **(T)**, **(U)** |
| **Task count** | 7 |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0007 on US-0010 external omniflow: extend `get_transactions` with `category_search` + `mirror_date_bounds` (**A1**, **A2**), enrich `get_subscriptions` schema and guard (**F1**), fix orchestrator prompt + audit observability (**E1**, **E2**), backend tests (**T1**), operator AI Chat verify (**V1**) on `financegnome.omniflow.cc` after backend deploy.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| A′ — Category search + mirror bounds | A1, A2 | backend transactions + AI tool |
| F — Subscriptions schema enrichment | F1 | backend AI tool |
| E — Orchestrator + audit | E1, E2 | backend orchestrator + tool schemas |
| Tests | T1 | backend tests |
| Verify | V1 | uat + operator AI Chat smoke |

**Out of scope:** seventh tool; RAG (note V); `allow_raw_transactions` default flip; payee aggregates; BUG-0008 alert/list changes; Firefly re-sync; frontend changes unless optional audit UI hint.

## Task summary

| ID | Title | Est. | Depends | Acceptance hook |
|----|-------|------|---------|-----------------|
| A1 | Category search SQL + mirror bounds | 3h | — | **(T)**, **(U)** |
| A2 | Tool schema + response assembly | 2h | A1 | **(T)**, **(U)** |
| F1 | Subscriptions schema + response + guard | 2h | — | **(S)** |
| E1 | SYSTEM_PROMPT + audit result_rows | 2h | A2, F1 | **(S)**, **(T)**, **(U)** |
| E2 | Parameter schema descriptions | 0.5h | A2, F1 | **(S)**, **(T)** |
| T1 | Unit/integration tests | 3h | A1–E2 | regression |
| V1 | verify-work omniflow AI Chat smoke | 1h | A1–E2, T1 deploy | **(S)**, **(T)**, **(U)** |

**Total estimate:** ~13.5h (~12.5h dev + ~1h operator V1).

## Deploy order

```text
(A1 + A2 + F1 + E1 + E2 + T1) single backend PR  →  deploy backend on omniflow
                                                    └→ V1 verify-work AI Chat smoke
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **(S)** | F1, E1, E2, V1 | Cancelable streaming total → follow-up lists named merchants from tool data |
| **(T)** | A1, A2, E1, E2, V1 | Strom/Amazon amounts via category_search; 2023 Amazon cites mirror_date_bounds |
| **(U)** | A1, A2, F1, E1, V1 | Multi-tool fusion without user-supplied merchant names |
| Regression | T1, V1 | Six-tool registry; `allow_raw_transactions=false` default |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| A1 | Task **A1** |
| A2 | Task **A2** |
| F1 | Task **F1** |
| E1 | Task **E1** |
| E2 | Task **E2** |
| T1 | Task **T1** |
| V1 | Task **V1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
