# Q0023 — BUG-0015 confirm persistence after rebuild

| Field | Value |
|-------|-------|
| **ID** | Q0023 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0015 |
| **Created** | 2026-06-07 |
| **Architecture** | `architecture-20260607-bug0015` (`docs/engineering/architecture.md` § BUG-0015) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260607-q0023-bug0015`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0015 rows **AU**–**AW** |
| **Task count** | 5 (all P0 mandatory) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0015 confirm-once trust on US-0010 external omniflow: **DEC-0084** Layer 1 card `payee_key` normalization (**AU1**), **DEC-0085**/**DEC-0086** Layer 2 payee+interval skip+merge with ±3d tolerance and in-place fingerprint rotation (**AU2**–**AU4**), operator verify + rebuild smoke (**V1**) on `financegnome.omniflow.cc` after deploy + H2 SQL probe + Full sync.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| AU — Confirm persists after rebuild | AU1, AU2, AU3, V1 | normalization + detection merge |
| AV — No duplicate pending | AU1–AU4, V1 | repository + detection + stale map |
| AW — Alert reconciliation | AU3, V1 | merge path suppresses new_detection |
| Verify | V1 | uat + operator rebuild smoke |

**Ops-only (not execute tasks):** H2 postgres persistence SQL probe; operator container rebuild.

**Out of scope:** Reopen BUG-0008; alert-only dedup; merchant identity table; UI changes; global amount drop from fingerprint.

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| AU1 | Card billing `payee_key` normalization | 2h | — | **AU**, **AV** | P0 |
| AU2 | Payee+interval maps + merge upsert + index | 4h | AU1 | **AU**, **AV** | P0 |
| AU3 | Detection skip + merge path | 3h | AU2 | **AU**, **AV**, **AW** | P0 |
| AU4 | Stale inactive by payee+interval | 2h | AU2 | **AV** | P0 |
| V1 | verify-work rebuild smoke | 2h | AU1–AU4 + deploy | **AU**–**AW** | P0 |

**Total estimate:** ~13h (11h dev + ~2h operator V1).

## Deploy order

```text
AU1 → AU2 → (AU3 ∥ AU4) single backend release
  → operator: BACKEND_FRONTEND_DEPLOY
  → operator: confirm Cursor + Apple baseline
  → operator: rebuild flow-finance-ai only (postgres untouched)
  → operator: POSTGRES_PERSISTENCE_PROBE (H2 SQL before Full sync)
  → operator: FULL_FIREFLY_SYNC + detection
  → V1 verify-work
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **AU** | AU1, AU2, AU3, V1 | Confirmed merchants remain confirmed after rebuild + Full sync |
| **AV** | AU1–AU4, V1 | No duplicate pending; merge/skip on payee+interval |
| **AW** | AU3, V1 | Unread alerts reconcile — no spurious new_detection |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| AU1 | Task **AU1** |
| AU2 | Task **AU2** |
| AU3 | Task **AU3** |
| AU4 | Task **AU4** |
| V1 | Task **V1** |
| H2 SQL probe | **Ops gate** — not sprint task |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
