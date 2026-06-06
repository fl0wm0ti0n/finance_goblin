# Technical Specification — US-0001

## Overview

Flow Finance AI US-0001 implements a monorepo-style deployment: Rust/Axum backend and React SPA served from a single `flow-finance-ai` container (or separate dev processes), connecting to external PostgreSQL/TimescaleDB and a read-only Firefly III instance. Sync runs on a Tokio cron scheduler; mirrored data lives in relational tables for downstream US-0002+ consumption.

**Firefly read-only guarantee:** All Firefly traffic uses HTTP GET only. Enforcement per DEC-0004.

## Components

### Backend (`flow-finance-ai` / Rust)

| Crate/module | Responsibility |
|--------------|----------------|
| `config` | Load TOML + env; validate `database.mode = "external"` |
| `db` | SQLx `PgPool`, embedded migrations, repository layer |
| `auth::jwt` | JWKS fetch/cache, Bearer token validation middleware |
| `firefly::client` | GET-only Reqwest wrapper, PAT header, path allowlist |
| `firefly::sync` | Pagination (limit=500), upsert, watermark (DEC-0002) |
| `sync::scheduler` | Tokio cron; configurable `sync.interval_seconds` |
| `sync::state` | Run lifecycle: idle → running → success/failed |
| `api::routes` | Axum router: health, sync, settings |
| `audit` | Optional `firefly_request_audit` writes |

**Startup sequence:** config load → DB retry loop (DEC-0003) → migrations → JWKS init → scheduler start → Axum bind.

### Database (external PostgreSQL + TimescaleDB)

Migration 001: `CREATE EXTENSION IF NOT EXISTS timescaledb`.

| Table | Key columns |
|-------|-------------|
| `sync_runs` | id, started_at, finished_at, status, trigger, error_message |
| `sync_cursors` | entity_type, last_successful_sync_at, records_synced |
| `accounts` | firefly_id PK, type, name, currency, balance, payload JSONB |
| `transactions` | firefly_id PK, date, amount, account_id, payload JSONB |
| `categories` | firefly_id PK, name, payload JSONB |
| `budgets` | firefly_id PK, name, amount, payload JSONB |
| `tags` | firefly_id PK, tag, payload JSONB |
| `piggy_banks` | firefly_id PK, name, target_amount, payload JSONB |
| `firefly_request_audit` | id, method, path, status_code, requested_at |

Indexes: `firefly_id` unique per table; `transactions(date)` for range queries.

### Frontend (React/TypeScript)

| Path | Component | API deps |
|------|-----------|----------|
| `/` | HomePage | `GET /api/v1/sync/status` |
| `/sync` | SyncStatusPage | status, runs, entities, `POST /api/v1/sync/trigger` |
| `/settings` | SettingsPage | `GET /api/v1/settings` |
| `/callback` | OidcCallback | — |

**Layout:** `SidebarProvider`, config-driven nav array, TanStack Query for polling.

**Auth:** `AuthProvider` from `react-oidc-context`; protected route wrapper; token attached to API client.

### Firefly Connector

```
SyncJob
  ├─ sync_reference_entities()  // accounts, categories, budgets, tags, piggy_banks — full list
  └─ sync_transactions()        // paginated GET /api/v1/transactions?start=&limit=500&page=N
       └─ upsert ON CONFLICT (firefly_id)
```

Config: `firefly.base_url`, `firefly.personal_access_token` (env secret).

### Docker Compose

```yaml
# Profiles (conceptual)
services:
  flow-finance-ai:   profiles: [minimal, standard, full]
  firefly-iii:       profiles: [minimal, standard, full]
  grafana:           profiles: [minimal, standard, full]
  redis:             profiles: [standard, full]
  ollama:            profiles: [full]
  authentik-*:       profiles: [oidc]
```

Env: `DATABASE_*`, `FIREFLY_*`, `OIDC_*`, `SYNC_INTERVAL_SECONDS`.  
Linux: `extra_hosts: ["host.docker.internal:host-gateway"]`.

### Grafana

Provisioning volumes:
- `provisioning/datasources/postgres.yaml` — external DB via env
- `provisioning/dashboards/platform-health.json` — optional (sync metrics)

## Interfaces

### Public HTTP (Axum)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | No | Liveness |
| GET | `/health/ready` | No | DB + migrations ready |

### Protected HTTP (Axum)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/v1/sync/status` | JWT | Current sync state, last run summary |
| GET | `/api/v1/sync/runs` | JWT | Paginated sync history |
| GET | `/api/v1/sync/entities` | JWT | Entity counts by Firefly type |
| POST | `/api/v1/sync/trigger` | JWT | Start manual sync (202 + run id) |
| GET | `/api/v1/settings` | JWT | Non-secret config display |

### Firefly API (outbound, server-side only)

| Method | Path | Notes |
|--------|------|-------|
| GET | `/api/v1/accounts` | Full list, paginated if needed |
| GET | `/api/v1/transactions` | Paginated; date filter for incremental |
| GET | `/api/v1/categories` | Full list |
| GET | `/api/v1/budgets` | Full list |
| GET | `/api/v1/tags` | Full list |
| GET | `/api/v1/piggy_banks` | Full list |

**Prohibited:** Any POST, PUT, PATCH, DELETE to Firefly.

### Frontend ↔ Backend

- Base URL: same origin in prod; dev proxy to Axum port
- Header: `Authorization: Bearer <access_token>`
- Content-Type: `application/json`
- Errors: 401 → redirect login; 409 → sync already running

## Non-functional

| Attribute | Target |
|-----------|--------|
| Availability | `/health` for Compose healthcheck |
| Security | PAT server-side only; JWT on all protected routes; HTTPS in prod |
| Read-only | GET-only Firefly client; audit log; integration test gate |
| Performance | Initial sync: progress UI; incremental < overlap window volume |
| Config | TOML file + env override; secrets via env/mount |
| Observability | Tracing (Tracing crate); Grafana Platform Health optional |
| Portability | Compose profiles; external DB; Linux/Mac/Windows dev via host-gateway |
| Migrations | SQLx embedded; run at startup; `SQLX_OFFLINE=true` in CI/Docker build |

**Dependencies:** US-0002+ consume mirror tables; no upstream story dependencies.

**References:** R-0001–R-0005, DEC-0001–DEC-0006, `docs/engineering/architecture.md#us-0001`.
