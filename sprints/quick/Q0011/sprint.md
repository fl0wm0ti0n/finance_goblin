# Q0011 — BUG-0004 post-sync pipeline empty analytics

| Field | Value |
|-------|-------|
| **ID** | Q0011 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0004 |
| **Created** | 2026-06-05 |
| **Architecture** | `architecture-20260605-bug0004` (`docs/engineering/architecture.md` § BUG-0004) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260605-q0011-bug0004`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0004 rows **(I)**, **(J)**, **(K)**, **(L)** |
| **Task count** | 7 |
| **Next phase** | `/execute` (plan-verify PASS 2026-06-05) |

## Goal

Close BUG-0004 on US-0010 external omniflow: fix exchange sync terminal status (**I1**), portfolio Grafana SQL (**K1**), Firefly account balance parse + wealth NULL handling (**L1**, **L2**), subscription payee grouping + empty-state UX (**J1**, **J2**); operator verify (**L3**) on `financegnome.omniflow.cc` after deploy + **Full Firefly sync** backfill.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| I — Sync lifecycle | I1 | backend (`sync/mod.rs`) |
| K — Grafana SQL | K1 | `grafana/.../portfolio.json` |
| L — Wealth / forecast data | L1, L2 | backend (`firefly/mod.rs`, `wealth/repository.rs`) |
| J — Subscriptions | J1, J2 | backend recurrence + frontend |
| Verify | L3 | verify-work / acceptance mapping |

**Out of scope:** BUG-0005/0006 merge; stuck historical `sync_runs` SQL cleanup; subscription auto-confirm; exchange multi-product (BUG-0005).

## Task summary

| ID | Title | Est. | Depends | Acceptance hook |
|----|-------|------|---------|-----------------|
| I1 | finish_sync_run on ExchangesOnly | 1.5h | — | **(I)** |
| K1 | Portfolio pie UNION SQL fix | 1h | — | **(K)** |
| L1 | Account balance parse (DEC-0060) | 1.5h | — | **(L)** |
| L2 | Wealth NULL balance filter | 1h | L1 | **(L)** |
| J1 | Payee key fallbacks (DEC-0061) | 2h | — | **(J)** |
| J2 | Subscriptions empty-state UX | 1.5h | J1 | **(J)** |
| L3 | verify-work omniflow probes | 1h | I1–J2 deploy + Full sync | **(I–L)** |

**Total estimate:** ~9.5h (dev ~8.5h + operator L3 ~1h).

## Deploy order

```text
(I1 + K1 + L1 + L2 + J1 + J2) single PR  →  deploy image  →  manual Full Firefly sync (account balance backfill)
                                                              └→ L3 verify-work on financegnome.omniflow.cc
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **(I)** | I1 | Exchange sync run terminal status; no stuck `running` on new runs |
| **(K)** | K1 | Portfolio ds/query **200**; no UNION syntax error |
| **(L)** | L1, L2, L3 | Wealth accounts non-empty; forecast series non-zero for funded account; snapshots populate |
| **(J)** | J1, J2 | Improved payee grouping; pending banner + threshold copy in UI |
| Regression | post-L3 | Acceptance footer (OIDC + bundled-firefly) |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
