# CRS — US-0004

## Purpose

Enable household financial planners to model life-decision scenarios (leasing, savings mode, major purchases), compare versioned plans (v1/v2/v3), and track daily plan-vs-actual deviation with Grafana visibility — without mutating Firefly ledger data.

## Scope

**In scope (per `docs/product/backlog.md`):**

- Plan Engine with scenario templates and custom adjustment lines
- Plan versioning and side-by-side compare
- Daily plan-vs-Ist (planned, actual, deviation) for active plan
- React `/planning` route with Scenarios, Compare, Plan vs Actual tabs
- Grafana Dashboard 3 (Plan, Ist, Abweichung) for active plan
- Integration with US-0002 forecast baseline and US-0003 confirmed subscriptions (savings mode)

**Out of scope:**

- AI `simulate_plan` (US-0006)
- Crypto allocation scenarios (US-0007)
- Plan viability / budget-drift Alert Engine inbox (US-0005)
- Active plan overlay on `/forecast`
- Per-category Grafana breakdown panels

## Acceptance criteria ref

`docs/product/acceptance.md` — section **US-0004** (6 criteria):

1. Named plan with scenario adjustments (e.g. +300 €/month leasing)
2. Built-in templates: current (Ist), leasing, savings mode, house purchase
3. Plan versions v1/v2/v3 created and compared side-by-side
4. Daily plan-vs-Ist: planned, actual, deviation
5. Grafana Dashboard 3 reflects active plan
6. Scenario changes do not modify Firefly transaction data

## Architecture ref

`docs/engineering/architecture.md` — **US-0004**  
**Decisions:** DEC-0019 … DEC-0024  
**Research:** R-0015 … R-0020
