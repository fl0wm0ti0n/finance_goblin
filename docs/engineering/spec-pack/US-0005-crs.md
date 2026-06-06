# CRS — US-0005

## Purpose

Canonical requirements snapshot for US-0005 — Wealth analysis, budget drift & scarcity alerts. See `docs/product/backlog.md` and `docs/product/acceptance.md`.

## Scope

**In scope:** Net worth aggregation (Firefly asset accounts; crypto placeholder until US-0007); Alert Engine (scarcity threshold, budget drift %, plan viability warnings); Grafana Dashboard 4 partial; React alert inbox with header bell; threshold config centralization (TOML + Dashboard 1 wiring).

**Out of scope:** Full crypto PnL (US-0007); Grafana Alertmanager rules; AI tool implementation (US-0006); subscription alert migration; multi-currency FX conversion.

## Acceptance criteria ref

`docs/product/acceptance.md` — section **US-0005**:

1. Net worth view aggregates Firefly-linked accounts (giro, savings, etc.)
2. Scarcity alert fires when projected balance falls below configurable threshold (e.g. 200 €)
3. Budget drift alert fires when category spending exceeds plan by configurable % (e.g. +20%)
4. Plan viability alert fires when active scenario becomes infeasible per forecast
5. Alert inbox in React UI lists active alerts with acknowledge/dismiss
6. Grafana Dashboard 4 shows total wealth (non-crypto until US-0007)

## Architecture mapping

| AC | Component |
|----|-----------|
| Net worth | `WealthService` + `/wealth` + `net_worth_snapshots` (DEC-0025) |
| Scarcity alert | `AlertService::evaluate_scarcity` + `alert_config` (DEC-0026, DEC-0029) |
| Budget drift | `AlertService::evaluate_budget_drift` vs active plan category targets (DEC-0026) |
| Plan viability | `AlertService::evaluate_plan_viability` on plan overlay (DEC-0026) |
| Alert inbox | `/alerts` + header bell + acknowledge/dismiss API (DEC-0027, DEC-0030) |
| Grafana Dashboard 4 | `portfolio.json` uid `portfolio` (DEC-0030) |

## Dependencies

- US-0001: mirror `accounts`, sync pipeline
- US-0002: `forecast_balance_daily`, scarcity input path
- US-0003: subscription alerts boundary (unchanged)
- US-0004: active plan, category-targeted adjustments for budget drift

## Research

R-0021 (net worth), R-0022 (evaluation rules), R-0023 (persistence/lifecycle), R-0024 (sync pipeline), R-0025 (threshold centralization), R-0026 (Dashboard 4)

## Decisions

DEC-0025 … DEC-0030 — see `docs/engineering/decisions.md`
