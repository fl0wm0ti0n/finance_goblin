# Design Concept — US-0003

## Summary

US-0003 delivers **subscription intelligence** for household budgeters: automatic recurring-pattern detection with confidence tiers (95/80/60%), confirm/reject workflow, standing-order (Dauerauftrag) separation, price-change detection with subscription-scoped alerts, forecast integration for confirmed/rejected patterns, a React `/subscriptions` page, and Grafana Dashboard 2.

The design extends US-0002 forecast heuristics via a shared recurrence core without duplicating algorithm logic. User confirmation gates forecast override and price-change alerts — false positives stay in pending until explicitly confirmed.

## Goals

- Shared `recurrence` module extracted from forecast recurring heuristics (DEC-0013, R-0009)
- Subscription Engine with confidence scoring, Dauerauftrag classification, price-change detection (DEC-0014–DEC-0017)
- Migration 003 lifecycle schema with rejection fingerprint persistence (DEC-0015, R-0012)
- Post-sync detection inline before forecast recompute (DEC-0018, R-0013)
- Forecast override: confirmed patterns replace heuristics; rejected excluded (AC-8)
- React `/subscriptions` with pending cards, confirmed table, standing-order tab, detail drawer with ECharts price history
- In-app alerts: banner + toast on `/subscriptions` (R-0011)
- Grafana Dashboard 2 (`uid: subscriptions`) provisioned as JSON (R-0014, DEC-0012 pattern)

## Non-goals

- Plan cancel-impact scenarios (US-0004)
- Unified Alert Engine inbox / global header bell (US-0005)
- AI `get_subscriptions` tool wiring (US-0006 implements consumer)
- Redis/async job queue for detection
- Per-account Grafana subscription variable (MVP global scope)
- Any write to Firefly III

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0013 | Shared recurrence core | Avoid drift from forecast layer (R-0009) |
| DEC-0014 | 95/80/60% tiers; min emit 60 | Backlog acceptance + noise control |
| DEC-0015 | Single lifecycle table + rejection fingerprints | Simplest queryable model (R-0012) |
| DEC-0016 | Rule-based Dauerauftrag | No operator whitelist burden (R-0010) |
| DEC-0017 | Dual €1 + 5% price threshold | Materiality without rounding noise (R-0011) |
| DEC-0018 | Subscriptions before forecast in mutex | Same-cycle forecast override (R-0013) |

**UX references:** Finanzguru pending review cards, Firefly payee vocabulary, shadcn Tabs/Cards/Badges, ECharts price history, Grafana Dashboard 2 — see `docs/product/vision.md`.

**Spec-pack traceability:** `docs/engineering/spec-pack/US-0003-crs.md`, `docs/engineering/spec-pack/US-0003-technical-specification.md`
