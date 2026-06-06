# CRS — US-0001

## Purpose

Canonical requirements snapshot for **US-0001 — Self-hosted platform foundation & Firefly read-only integration**. Defines the deployable platform layer that ingests Firefly data read-only and provides OIDC-protected operator UI — enabling all downstream analytics stories without mutating the Firefly ledger.

## Scope

### In scope

- Docker Compose profiles: `minimal` (flow-finance-ai, firefly-iii, grafana), `standard` (+redis), `full` (+ollama), optional `oidc` (+Authentik)
- External PostgreSQL configuration (`database.mode = "external"`) — never embedded
- Rust/Axum/Tokio backend: health endpoints, authenticated API skeleton, SQLx migrations
- Firefly Connector: GET-only, PAT auth, paginated sync of accounts, transactions, categories, budgets, tags, piggy banks
- Sync scheduler (Tokio cron) with configurable interval and manual trigger
- PostgreSQL mirror schema: sync metadata + entity tables with Firefly id upsert
- React/TypeScript/Tailwind/shadcn UI shell: Home, Sync Status, Settings; disabled nav placeholders
- OIDC auth: redirect login, session in sidebar footer, protected routes
- Grafana: service + datasource provisioning; optional Platform Health dashboard
- Read-only verification: integration test and/or Firefly request audit log

### Out of scope

- Forecast engine, subscription detection, planning, alerts, AI, crypto
- Grafana analytics dashboards 1–5
- Redis as application cache (container only in standard profile)
- TimescaleDB hypertables (extension enabled; tables deferred to US-0002)
- In-app OIDC or Firefly connection administration beyond display

## Acceptance criteria ref

`docs/product/acceptance.md` — section **US-0001** (8 criteria):

1. Docker Compose minimal profile starts without embedded PostgreSQL
2. External PostgreSQL configurable via TOML/env
3. Rust backend health + authenticated API skeleton
4. React UI with OIDC login redirect + session
5. Firefly Connector syncs all six entity types via `/api/v1`
6. Sync scheduler + manual trigger
7. No Firefly write operations (verified)
8. Dev environment connects to local Firefly III

**Architecture:** `docs/engineering/architecture.md` — section **# US-0001**  
**Decisions:** DEC-0001 … DEC-0006  
**Research:** R-0001 … R-0005
