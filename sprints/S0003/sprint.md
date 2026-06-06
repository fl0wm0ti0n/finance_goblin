# Sprint S0003

**ID:** S0003  
**Story:** US-0003 — Subscription detection, price changes & alerts  
**Status:** PLANNED  
**Created:** 2026-05-31

## Goal

Deliver subscription intelligence on top of US-0001 sync and US-0002 forecast: shared recurrence core, Subscription Engine with confidence tiers and Dauerauftrag classification, migration 003 persistence, inline detection after Firefly sync, forecast override for confirmed/rejected patterns, JWT-protected subscription API, React `/subscriptions` page with confirm/reject workflow and price history, Grafana Dashboard 2, tests, and operator user guide.

## Scope

- Extract `recurrence/` module from `forecast/recurring.rs`; forecast thin wrapper (DEC-0013)
- Migration `003_subscriptions.sql` — `subscription_patterns` lifecycle + satellites + enums (DEC-0015)
- Subscription Engine: `classify`, `detection`, `price_change`, `repository`, `service` (DEC-0014, DEC-0016, DEC-0017)
- Sync pipeline: `subscriptions` phase before forecast recompute; phase reporting (DEC-0018)
- Forecast override: `recompute` accepts `DetectionResult`; confirmed override + rejected exclusion (AC-8)
- REST API: 7 subscription endpoints including alerts (DEC-0006)
- React: enable Subscriptions nav; tabs All | Pending | Standing orders; cards, table, detail drawer + ECharts
- Grafana: `subscriptions.json` — uid `subscriptions`, Analytics folder (R-0014, DEC-0012 pattern)
- Tests and operator user guide (`docs/user-guides/US-0003.md`)

**Out of scope:** Plan cancel scenarios (US-0004), unified Alert Engine inbox / header bell (US-0005), AI `get_subscriptions` tool (US-0006), Redis/async detection queue, per-account Grafana variable (MVP global), any Firefly writes.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Recurrence refactor regresses forecast | Behavior-preserving extraction; run existing recurring tests | DEC-0013 |
| Sync mutex duration grows | Log sync / subscriptions / forecast phases separately; defer queue if > ~30s | DEC-0018, R-0013 |
| 60% tier false positives | Pending confirm/reject UX; alerts only after user action on low tier | DEC-0014 |
| Dauerauftrag misclassification | Rule-based + optional config patterns; kind override on confirm | DEC-0016, R-0010 |
| Price-change noise | Dual threshold ≥€1 AND ≥5% on confirmed only | DEC-0017 |
| Grafana monthly-spend SQL | Interval-normalized expression; empty-state panels | R-0014 |
| Descriptor normalization drift | Regex strip trailing codes; document limitation | R-0009 |

## Definition of Done

- All 12 sprint tasks complete (`T-0025` … `T-0036`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0003
- Detection runs inline after successful Firefly sync before forecast recompute
- Confirmed subscriptions override forecast heuristics; rejected fingerprints excluded
- `/subscriptions` renders pending review, confirmed list, standing orders tab, price history
- Grafana Dashboard 2 loads with uid `subscriptions`
- User guide published at `docs/user-guides/US-0003.md`

## Architecture references

- `docs/engineering/architecture.md` — US-0003
- Decisions: DEC-0013 … DEC-0018
- Research: R-0009 … R-0014
- Depends on: US-0001 mirror tables + sync scheduler; US-0002 forecast engine + recurring heuristics
