# CRS — US-0003

## Purpose

Enable household budgeters to detect, confirm, and monitor recurring subscriptions and standing orders with price-change alerts — built on US-0001 Firefly read-only sync and US-0002 forecast heuristics baseline.

## Scope

### In scope

- Subscription Detection Engine: amount, payee, text, regularity, intervals; confidence 95/80/60%
- Shared recurrence core extracted from forecast (DEC-0013)
- User confirm/reject workflow with persisted state (DEC-0015)
- Standing-order (Dauerauftrag) classification and separate UI tab (DEC-0016)
- Price increase/decrease and interval-change detection on confirmed subscriptions (DEC-0017)
- Subscription-scoped in-app alerts (banner + toast)
- Forecast integration: confirmed override + rejected exclusion (AC-8, DEC-0018)
- Post-sync detection inline before forecast recompute
- React `/subscriptions` page with pending cards, confirmed list, detail drawer
- Grafana Dashboard 2 (`uid: subscriptions`)
- Migration 003 subscription schema

### Out of scope

- Plan scenario cancel-impact (US-0004)
- Unified Alert Engine inbox (US-0005)
- AI tool layer detail (US-0006)
- Redis/async detection queue
- Firefly writes for subscription state

## Constraints

- Detection runs on synced Firefly transactions only (US-0001 dependency)
- User confirmation required before treating pattern as confirmed subscription
- Read-only toward Firefly III (DEC-0004)
- JWT-protected API (DEC-0006)
- Detection extends sync mutex per DEC-0010 / DEC-0018

## Acceptance criteria ref

`docs/product/acceptance.md` — section **US-0003** (8 criteria):

1. Detection engine with confidence score tiers 95/80/60%
2. Notification for new detection with confirm/reject actions
3. Confirmed subscriptions in list with interval and amount
4. Standing-order patterns detected separately
5. Price increase/decrease on confirmed subscription
6. Alert on new detection and price change
7. Grafana Dashboard 2 provisioned
8. Rejected patterns excluded from forecast and alerts

## Traceability

| Artifact | Path |
|----------|------|
| Backlog | `docs/product/backlog.md` — US-0003 |
| Architecture | `docs/engineering/architecture.md` — US-0003 |
| Research | R-0009, R-0010, R-0011, R-0012, R-0013, R-0014 |
| Decisions | DEC-0013 … DEC-0018 |
| User guide (execute) | `docs/user-guides/US-0003.md` |
