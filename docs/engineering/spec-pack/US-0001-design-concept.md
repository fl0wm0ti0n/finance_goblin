# Design Concept — US-0001

## Summary

US-0001 establishes **Flow Finance AI** as a deployable, self-hosted platform that mirrors Firefly III data read-only into an external PostgreSQL database and presents an OIDC-protected operator UI shell. The user sees sync health, entity counts, and configuration at a glance — with trust signals (read-only badge, last sync time) — while analytics features remain visible but disabled in navigation.

The design prioritizes **operator confidence**: Firefly stays untouched, external database is never embedded, and sync status is transparent. Grafana provides optional ops visibility; consumer-style analytics dashboards are deferred.

## Goals

- Deployable Docker Compose stack (`minimal` profile: flow-finance-ai + firefly-iii + grafana) with external PostgreSQL
- Read-only Firefly connector syncing accounts, transactions, categories, budgets, tags, piggy banks
- Configurable sync scheduler with manual trigger and visible progress/history
- OIDC-protected React UI shell: Home, Sync Status, Settings; disabled nav placeholders for future features
- Explicit read-only guarantee verifiable via audit log and integration test
- Minimal Grafana provisioning with optional Platform Health dashboard
- Clear API boundary: browser → Axum API (JWT); Axum → Firefly (GET-only, server-side PAT)

## Non-goals

- Cashflow forecasting, subscription detection, planning, AI, crypto integration
- Grafana analytics dashboards 1–5 (Cashflow, Subscriptions, Budgets, Portfolio, Forecast)
- Application dependency on Redis (container may exist in `standard` profile only)
- In-app OIDC provider administration
- Direct browser access to Firefly API
- Any write operation to Firefly III

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0001 | Authentik optional `oidc` profile | Self-hosted IdP without blocking minimal profile (R-0003) |
| DEC-0002 | 7-day overlap watermark sync | No native Firefly cursor; unreliable `updated_at` (R-0002) |
| DEC-0003 | Startup DB retry ~60s | External PG not in Compose health graph (R-0005) |
| DEC-0004 | GET-only Firefly client + audit | Acceptance read-only verification (R-0001) |
| DEC-0005 | Relational mirrors only | Hypertables when forecast ships in US-0002 (R-0004) |
| DEC-0006 | SPA JWT, not BFF | Simpler US-0001 auth; sufficient for skeleton API (R-0003) |

**UX references:** Finanzguru trust signals, Firefly entity vocabulary, shadcn sidebar shell, Grafana ops-only provisioning — see `docs/product/vision.md`.

**Spec-pack traceability:** `docs/engineering/spec-pack/US-0001-crs.md`, `docs/engineering/spec-pack/US-0001-technical-specification.md`
