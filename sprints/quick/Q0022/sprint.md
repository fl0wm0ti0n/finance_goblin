# Q0022 — BUG-0014 post-rebuild omniflow cluster

| Field | Value |
|-------|-------|
| **ID** | Q0022 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0014 |
| **Created** | 2026-06-10 |
| **Architecture** | `architecture-20260609-bug0014` (`docs/engineering/architecture.md` § BUG-0014) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260610-q0022-bug0014`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0014 rows **AO**–**AT** |
| **Task count** | 8 (5 mandatory + 1 P1 optional + 2 conditional) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0014 post-rebuild operator cluster on US-0010 external omniflow: **DEC-0081** AQ holdings/FX (**AQ1**, **AQ2**), **DEC-0082** AS delete guard (**AS1**), **DEC-0083** AS target_type UI (**AS2**), Grafana dual-scenario ML banner (**AO1**), conditional **AP2** wealth subtotal hardening, operator verify + regression smoke (**V1**) on `financegnome.omniflow.cc` after deploy + operator gates.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| AO — ML banner | AO1 | grafana `forecast-horizons.json` panel 13 |
| AQ — Holdings + FX (DEC-0081) | AQ1, AQ2 | backend wealth + WealthPage |
| AS — Planning UX (DEC-0082/0083) | AS1, AS2 | backend plans + PlanningPage |
| AP — Subtotal hardening (gated) | AP2 | backend `wealth/service.rs` |
| AR — Cashflow (conditional) | AR1 | grafana `cashflow.json` |
| Verify | V1 | uat + operator omniflow smoke |

**Ops-only (not execute tasks):** AO runtime (start sidecar), AT (three-service compose smoke), AP1 (SQL probe gate).

**Out of scope:** ExchangePriceBook tier-2; Grafana dynamic Postgres ML variable; `target_type` enum expansion; AR1 unless API≠Grafana.

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| AO1 | Grafana panel 13 dual-scenario ML copy | 1h | — | **AO** | P0 |
| AQ1 | `holdings_all` + `unpriced_assets` / `fx_incomplete` | 3h | — | **AQ** | P0 |
| AQ2 | WealthPage native+EUR + unified FX banner | 2h | AQ1 | **AQ** | P0 |
| AS1 | Delete plan UI + active 409 guard | 2.5h | — | **AS** | P0 |
| AS2 | target_type select + help copy | 1h | — | **AS** | P1 optional |
| AP2 | Defensive subtotal + count annotation | 1.5h | AQ1, AP1 gate | **AP** | P0 conditional |
| AR1 | Cashflow Grafana variable fix | 1.5h | V1 AR gate | **AR** | P2 conditional |
| V1 | verify-work omniflow smoke | 2h | AO1, AQ1, AQ2, AS1 | **AO**–**AT** | P0 |

**Total estimate:** ~14h (9.5h mandatory + 1h optional + 3h conditional/gated).

## Deploy order

```text
(AO1 + AQ1 + AQ2 + AS1 [+ AS2]) single release
  → operator: BACKEND_FRONTEND_DEPLOY + THREE_SERVICE_COMPOSE
  → operator: FULL_FIREFLY_SYNC + forecast recompute acct 114
  → operator: AP1 SQL probe → AP2 if gate passes
  → operator: GRAFANA_PROVISIONING_RELOAD
  → AR1 only if V1 AR verify gate fails
  → V1 verify-work
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **AO** | AO1, V1 | ML available or accurate sidecar-down copy — not misleading US-0013 disabled |
| **AP** | AP2, V1 | crypto.subtotal_eur > 0 when wallet priced; AP2 only after AP1 gate |
| **AQ** | AQ1, AQ2, V1 | holdings_all native+EUR; unified fx_incomplete + unpriced_assets |
| **AR** | AR1, V1 | Cashflow acct 114 non-zero; AR1 only API≠Grafana |
| **AS** | AS1, AS2, V1 | Plan delete + 409 active guard; valid target_type + help |
| **AT** | V1 | stats-forecast running on external profile when ML enabled |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| AO1 | Task **AO1** |
| AQ1 | Task **AQ1** |
| AQ2 | Task **AQ2** |
| AS1 | Task **AS1** |
| AS2 | Task **AS2** (P1 optional) |
| AP2 | Task **AP2** (conditional on AP1) |
| AR1 | Task **AR1** (conditional on V1 AR gate) |
| V1 | Task **V1** |
| AP1 | **Ops gate** — not sprint task |
| AO/AT runtime | **Ops-only** — verify in V1 |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
