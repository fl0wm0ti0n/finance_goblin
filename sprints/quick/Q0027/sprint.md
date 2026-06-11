# Q0027 — BUG-0019 Grafana metrics wrong (cashflow zeros, sync entity counts)

| Field | Value |
|-------|-------|
| **ID** | Q0027 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0019 |
| **Created** | 2026-06-10 |
| **Architecture** | `docs/engineering/architecture.md` § BUG-0019 (DEC-0108) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260610-q0027-bug0019`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0019 rows **BG**, **BH** |
| **Task count** | 6 (all P0 mandatory) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0019 via the **DEC-0108 provisioning-only contract**: fix `$account_id`
default selection (`sort: 0` + `current`) in cashflow + forecast-horizons (**CA**,
gate **BG**), qualify cashflow latest-success subqueries with
`model_kind = 'baseline'` (**CA** latent), and replace Platform Health panel 2
SQL with per-entity mirror `COUNT(*)` UNION ALL (**CB**, gate **BH**). Bump
dashboard `version` in every edited JSON; static JSON guard + operator
verify-work close the BG/BH gates. **No backend, frontend, or migration changes.**

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| CA variable + subquery fix (DEC-0108) | CA1, CA2 | `grafana/provisioning/dashboards/analytics/cashflow.json` |
| CA variable fix (forecast-horizons) | CA3 | `grafana/provisioning/dashboards/analytics/forecast-horizons.json` |
| CB mirror-count panel (DEC-0108) | CB1 | `grafana/provisioning/dashboards/platform-health.json` |
| Static JSON guard | G1 | jq assertions on the three dashboards |
| Verify | V1 | `uat.md` + operator Grafana re-provision smoke |

**Ops-only (not execute tasks):** Operator **GRAFANA_PROVISIONING_RELOAD**
(`docker compose restart grafana`) before V1 runtime probes; **incremental
0-new-tx sync rerun** for the BH regression proof.

**Out of scope (DEC-0108 forbidden):** `backend/src/db/mod.rs` `upsert_cursor`;
`backend/src/firefly/mod.rs` sync semantics; `AnalyticsEmbedPage.tsx`; migrations;
hardcoding account id `114`; embed-forwarded `var-account_id` (CA-B deferred);
`ml_enhanced` stuck-computation cleanup (flagged to PO — separate bug).

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| CA1 | cashflow.json `$account_id` `sort: 0` + `current` + version bump | 0.5h | — | **BG** | P0 |
| CA2 | cashflow.json panels 1–3 `model_kind = 'baseline'` subquery | 0.5h | CA1 | **BG** | P0 |
| CA3 | forecast-horizons.json `$account_id` `sort: 0` + `current` + version bump | 0.5h | — | **BG** | P0 |
| CB1 | platform-health.json panel 2 mirror COUNT(*) UNION ALL + version bump | 1h | — | **BH** | P0 |
| G1 | Static JSON guard (jq assertions, valid JSON) | 0.5h | CA1–CB1 | **BG**, **BH** static | P0 |
| V1 | verify-work Grafana re-provision smoke | 1.5h | G1 + reload | **BG**, **BH** | P0 |

**Total estimate:** ~4.5h (3h dev + ~1.5h operator V1).

## Deploy order

```text
(CA1 → CA2) ∥ CA3 ∥ CB1   (three JSON files; CA2 after CA1 in same file)
  → G1 (static JSON guard PASS)
  → operator: GRAFANA_PROVISIONING_RELOAD (docker compose restart grafana)
  → V1 verify-work (kiosk embed + direct Grafana; Full sync + 0-new-tx incremental rerun for BH)
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **BG** | CA1, CA2, CA3, G1, V1 | Cashflow panels 1–2 non-zero (negative) series, default account = 114; matches `GET /api/v1/forecast/monthly?account_id=114` (25 points, non-zero from Jul 2026); kiosk embed **and** direct Grafana |
| **BH** | CB1, G1, V1 | Platform Health panel 2 `transactions` = `SELECT COUNT(*) FROM transactions` (922) after Full sync **and again** after a subsequent 0-new-tx incremental run |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| CA-1 | Task **CA1** |
| CA-2 | Task **CA2** |
| CA-3 | Task **CA3** |
| CB-1 | Task **CB1** |
| Static gate | Task **G1** |
| BG/BH runtime gates | Task **V1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
