# Design Concept — US-0002

## Summary

US-0002 delivers **cashflow forecasting MVP** for household budgeters: daily balance milestones (tomorrow, next week, month-end), monthly income/fixed/variable/free cashflow breakdown, and long-term balance projections (3 / 6 / 12 / 24 months). Forecasts are **precomputed after each successful Firefly sync**, persisted in TimescaleDB hypertables, exposed via REST API to a new React `/forecast` page with Apache ECharts, and visualized in Grafana Dashboards 1 (Cashflow) and 5 (Forecast horizons).

The design answers Finanzguru-style "where will my balance be?" without cloud dependency or Firefly writes. Trust signals (last computed timestamp, sync link, low-confidence flags) accompany charts.

## Goals

- Hybrid rule-based Forecast Engine derived from synced Firefly mirrors (DEC-0007, R-0006)
- TimescaleDB hypertables for daily balance and monthly cashflow snapshots (DEC-0008, R-0007)
- Automatic recompute after successful Firefly sync, extending sync mutex (DEC-0010)
- React `/forecast` with account selector, horizon tabs, ECharts charts (vision.md UX refs)
- Grafana Dashboard 1 + 5 provisioned as JSON with stable uids (DEC-0012, R-0008)
- Per-account forecast default with optional aggregate (DEC-0009)
- Retention of last 5 computations to bound storage (DEC-0011)

## Non-goals

- ML or seasonal models (US-0009)
- Subscription confirm/reject engine (US-0003) — lightweight recurring heuristics only
- Plan scenario overlays (US-0004)
- Alert Engine scarcity/drift firing (US-0005) — Grafana threshold lines visual only
- Grafana Dashboards 2–4 (Subscriptions, Budgets, Portfolio)
- Redis/async job queue for recompute
- Any write to Firefly III

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0007 | Hybrid rule-based algorithm | Recurring heuristics + rolling avg; simplest viable accuracy (R-0006) |
| DEC-0008 | Precomputed hypertable snapshots | Acceptance + Grafana stable series (R-0007) |
| DEC-0009 | Per-account primary | Matches Finanzguru account-centric UX; optional aggregate |
| DEC-0010 | Recompute in sync task | No Redis; mutex prevents races (R-0007) |
| DEC-0011 | Retain 5 computations | Bounded growth on self-hosted installs |
| DEC-0012 | Static Grafana scarcity €200 | Visual until US-0005 Alert Engine (R-0008) |

**UX references:** Finanzguru horizon pickers, ECharts daily/monthly/long-term patterns, Grafana Cashflow + Forecast dashboards — see `docs/product/vision.md`.

**Spec-pack traceability:** `docs/engineering/spec-pack/US-0002-crs.md`, `docs/engineering/spec-pack/US-0002-technical-specification.md`
