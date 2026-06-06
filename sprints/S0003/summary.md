# Sprint S0003 Summary — US-0003 Subscription Detection

## Context pack (refresh 2026-05-31)

- **Status:** CLOSED / released (`0.3.0-us0003`)
- **Story:** US-0003 DONE — 8/8 acceptance, queue `released`
- **Evidence:** handoffs/releases/S0003-release-notes.md, sprints/S0003/release-findings.md
- **Next story:** US-0004 (financial planning, scenarios & plan-vs-actual) — discovery phase

**Story:** US-0003  
**Sprint:** S0003  
**Date:** 2026-05-31

## Delivered

| Layer | Deliverable |
|-------|-------------|
| Recurrence core | `backend/src/recurrence/` — confidence tiers 95/80/60 (DEC-0013, DEC-0014) |
| Database | `003_subscriptions.sql` — subscription_patterns lifecycle + satellites (DEC-0015) |
| Backend engine | Subscription detection, Dauerauftrag classify, price-change dual threshold (DEC-0016, DEC-0017) |
| Sync hook | Subscriptions phase before forecast; failure-tolerant (DEC-0018) |
| Forecast override | Confirmed replaces heuristic; rejected fingerprints excluded (AC-8) |
| API | 7 routes under `/api/v1/subscriptions/*` |
| Frontend | `/subscriptions` — tabs, confirm/reject, alerts banner, lazy ECharts price history |
| Grafana | Dashboard `subscriptions` in Analytics folder (R-0014) |
| Tests | 18 unit tests; `subscriptions_integration` (SKIP without DATABASE_URL) |
| Docs | `docs/user-guides/US-0003.md` |

## Task completion

T-0025 … T-0036 — all complete.

## Test results

```
bash tests/run-tests.sh PASS
- cargo test --lib: 18 passed
- subscriptions_integration: SKIP (DATABASE_URL unset)
- forecast_integration: SKIP (DATABASE_URL unset)
- firefly_integration: SKIP (DATABASE_URL unset)
- frontend build: PASS (PriceHistoryChart lazy chunk)
```

## Key decisions applied

DEC-0013 (recurrence core), DEC-0014 (confidence tiers), DEC-0015 (schema), DEC-0016 (Dauerauftrag), DEC-0017 (price threshold), DEC-0018 (sync pipeline order)

## Known limitations

- Integration tests require operator TimescaleDB with `DATABASE_URL`
- Live confirm/reject and price-change flows need synced recurring expense data (≥3 occurrences)
- Sync-triggered detection + forecast latency should be monitored under production volume
- ECharts main bundle ~1 MB (PriceHistoryChart code-split)

## Release closure

- Released 2026-05-31; runtime E2E deferred to operator environment (documented in UAT)
- Carry-forward: OIDC E2E needs IdP or `AUTH_DEV_BYPASS=true`; integration tests need `DATABASE_URL`
