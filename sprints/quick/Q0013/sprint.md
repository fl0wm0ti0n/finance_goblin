# Q0013 — BUG-0010 forecast/wealth/ML posture

| Field | Value |
|-------|-------|
| **ID** | Q0013 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0010 |
| **Created** | 2026-06-05 |
| **Architecture** | `architecture-20260605-bug0010` (`docs/engineering/architecture.md` § BUG-0010) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260605-q0013-bug0010`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0010 rows **(AA)**, **(AB)**, **(AC)** |
| **Task count** | 7 |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0010 on US-0010 external omniflow: correct Firefly balance mirror diagnostics (**AA1**), negative-account wealth visibility (**AB1**), ML disabled metadata (**AC1**), forecast/wealth UX warnings (**AA3**, **AB2**), honest ML copy (**AC2**); operator verify (**V1**) on `financegnome.omniflow.cc` after deploy + Full Firefly sync.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| AA — Balance mirror | AA1, AA3 | backend firefly + forecast |
| AB — Wealth | AB1, AB2 | backend wealth + frontend |
| AC — ML posture | AC1, AC2 | backend sync/meta + frontend |
| Verify | V1 | verify-work / acceptance mapping |

**Out of scope:** AC3 → **US-0013** (stats-forecast on external profile); BUG-0009 Grafana; transaction re-ingest.

## Task summary

| ID | Title | Est. | Depends | Acceptance hook |
|----|-------|------|---------|-----------------|
| AA1 | Balance mirror ingest + diagnostics | 2h | — | **(AA)** |
| AB1 | Negative asset wealth visibility (DEC-0065) | 2h | — | **(AB)** |
| AC1 | sidecar_disabled metadata (DEC-0066) | 1.5h | — | **(AC)** |
| AA3 | Negative starting balance warning | 2h | AA1 | **(AA)** |
| AB2 | Wealth zero-total empty-state | 1h | AB1 | **(AB)** |
| AC2 | Forecast ML three-state UI | 1.5h | AC1 | **(AC)** |
| V1 | verify-work omniflow probes | 1h | AA1–AC2 deploy + Full sync | **(AA)(AB)(AC)** |

**Total estimate:** ~11h (dev ~10h + operator V1 ~1h).

## Deploy order

```text
(AA1 + AB1 + AC1 + AA3 + AB2 + AC2) single PR  →  deploy image
                                              └→ manual Full Firefly sync (operator gate)
                                              └→ V1 verify-work on financegnome.omniflow.cc
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **(AA)** | AA1, AA3, V1 | Plausible signed forecast OR explicit negative-start warning; not silent -25k |
| **(AB)** | AB1, AB2, V1 | Non-empty wealth breakdown; acct 114 visible; honest total_eur |
| **(AC)** | AC1, AC2, V1 | ML not-enabled copy when disabled; no false "ML skipped" on null reason |
| Regression | post-V1 | Acceptance footer (OIDC + bundled-firefly) |

## Discovery → sprint mapping

| Discovery ID | Disposition |
|--------------|-------------|
| AA1 | Task **AA1** |
| AA2 | Operator gate **FULL_FIREFLY_SYNC** before V1 |
| AA3 | Task **AA3** |
| AB1 | Task **AB1** |
| AB2 | Task **AB2** |
| AB3 | **V1** wealth/history probes |
| AC1 | Task **AC1** |
| AC2 | Task **AC2** |
| AC3 | Deferred **US-0013** epic |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
