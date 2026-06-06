# Q0016 — BUG-0009 Grafana empty panels & account overview

| Field | Value |
|-------|-------|
| **ID** | Q0016 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0009 |
| **Created** | 2026-06-06 |
| **Architecture** | `architecture-20260606-bug0009` (`docs/engineering/architecture.md` § BUG-0009) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260606-q0016-bug0009`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0009 rows **(Y)**, **(Z)** |
| **Task count** | 6 |
| **Next phase** | `/execute` |

## Goal

Close BUG-0009 on US-0010 external omniflow: fix portfolio breakdown SQL and cross-account overview (**Z1**, **Z2**), correct `$account_id` default to funded account (**Y1**), add ML honest empty-state (**Y2**), SQL/provisioning tests (**T1**), operator verify + six-route smoke (**V1**) on `financegnome.omniflow.cc` after Grafana provisioning reload.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| Z — Portfolio breakdown + overview | Z1, Z2 | grafana `portfolio.json` |
| Y — Account variable + ML empty-state | Y1, Y2 | grafana `cashflow.json`, `forecast-horizons.json` |
| Tests | T1 | SQL fixtures + optional JSON snapshot |
| Verify | V1 | uat + operator smoke |

**Out of scope:** US-0013 (ML enablement); React `/forecast` API reorder; seventh overview dashboard; Grafana dynamic hide rules; backend changes.

## Task summary

| ID | Title | Est. | Depends | Acceptance hook |
|----|-------|------|---------|-----------------|
| Z1 | Portfolio breakdown SQL subquery + LATERAL | 1h | — | **(Z)** |
| Z2 | Cross-account overview table + grid layout | 1.5h | Z1 | **(Z)** |
| Y1 | `$account_id` ABS(balance) variable query | 1h | — | **(Y)** |
| Y2 | ML banner + noValue on ML panels | 1h | — | **(Y)** |
| T1 | SQL fixtures + provisioning snapshot test | 1.5h | Z1, Y1 | **(Y)**, **(Z)** |
| V1 | verify-work omniflow smoke | 1h | Z1–Y2, T1 deploy | **(Y)**, **(Z)** |

**Total estimate:** ~7h (provisioning + tests; no backend deploy dependency beyond image rebuild for Grafana JSON).

## Deploy order

```text
(Z1 + Z2 + Y1 + Y2 + T1) single PR  →  deploy / Grafana provisioning reload
                                      └→ V1 verify-work on financegnome.omniflow.cc
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **(Y)** | Y1, Y2, T1, V1 | Default-load cashflow/forecast non-empty; ML banner + noValue; ds/query 200 |
| **(Z)** | Z1, Z2, T1, V1 | Portfolio overview 3 account rows; `total_eur` stat visible; six routes smoke |
| Regression | post-V1 | BUG-0003 H ds/query 200; US-0011 six `/analytics/{slug}` routes |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| Z1 | Task **Z1** |
| Z2 | Task **Z2** |
| Y1 | Task **Y1** |
| Y2 | Task **Y2** |
| T1 | Task **T1** |
| V1 | Task **V1** (Y3/Z3 operator smoke folded in) |
| Z3 docs | Optional text in Z2 or V1 runbook — supplementary `/wealth` link |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
