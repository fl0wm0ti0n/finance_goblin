# Architecture archive pack (2026-06-05)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 6
- Retained units in hot file: 6
- First archived heading: `## US-0001 — Platform foundation & Firefly read-only integration`
- Last archived heading: `## US-0006 — AI financial assistant with privacy-safe tool layer`
- Verification tuple (mandatory):
  - archived_body_lines=1718
  - retained_body_lines=2906
  - preamble_lines=865

---

## US-0001 — Platform foundation & Firefly read-only integration

**Status:** architecture complete (2026-05-31)  
**Research:** R-0001, R-0002, R-0003, R-0004, R-0005  
**Decisions:** DEC-0001, DEC-0002, DEC-0003, DEC-0004, DEC-0005, DEC-0006  
**Spec-pack:** `docs/engineering/spec-pack/US-0001-{design-concept,crs,technical-specification}.md`

### System context

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                         Operator / Browser                               │
└───────────────┬───────────────────────────────┬─────────────────────────┘
                │ OIDC login                     │ HTTPS (UI + API)
                ▼                                ▼
┌───────────────────────┐              ┌─────────────────────────────────┐
│  IdP (Authentik)      │              │  flow-finance-ai container       │
│  [optional oidc       │              │  ┌─────────────┐ ┌─────────────┐ │
│   Compose profile]    │              │  │ React SPA   │ │ Axum API    │ │
└───────────────────────┘              │  │ (shadcn)    │ │ (Rust/Tokio)│ │
                                       │  └──────┬──────┘ └──────┬──────┘ │
                                       │         │ JWT Bearer     │        │
                                       │         └────────────────┘        │
                                       │                  │                 │
                                       │         ┌────────┴────────┐        │
                                       │         │ Sync Scheduler  │        │
                                       │         │ (Tokio cron)    │        │
                                       │         └────────┬────────┘        │
                                       │                  │                 │
                                       │         ┌────────┴────────┐        │
                                       │         │ Firefly Connector│        │
                                       │         │ (GET-only, PAT)  │        │
                                       │         └────────┬────────┘        │
                                       └──────────────────┼─────────────────┘
                                                          │ GET /api/v1/*
                                                          ▼
┌───────────────────────┐    ┌────────────────────────────────────────────┐
│  firefly-iii          │    │  External PostgreSQL + TimescaleDB         │
│  (Compose minimal)    │    │  (never embedded in Compose)               │
│  own external DB      │    │  flow_finance_ai DB: mirror tables + sync  │
└───────────────────────┘    └────────────────────────────────────────────┘
                                                          ▲
┌───────────────────────┐                                 │ read (datasource)
│  grafana              │─────────────────────────────────┘
│  (Compose minimal)    │   optional Platform Health dashboard
└───────────────────────┘
```

### Components

#### 1. Firefly Connector (read-only)

| Aspect | Design |
|--------|--------|
| Auth | Personal Access Token via `Authorization: Bearer` (preferred per R-0001, DEC-0001); OAuth2 client-credentials as future alternative |
| HTTP | **GET-only** client; reject any non-GET at compile-time wrapper + runtime guard (DEC-0004) |
| Endpoints | `/api/v1/accounts`, `/transactions`, `/categories`, `/budgets`, `/tags`, `/piggy_banks` |
| Pagination | `page` + `limit=500`; terminate on empty page or missing `links.next` (R-0002) |
| Incremental | Watermark `last_successful_sync_at`; fetch transactions with `start=<watermark_date - 7 days>`; upsert by Firefly `id` (DEC-0002, R-0002) |
| Reference entities | Full-list sync each run (low volume); upsert by Firefly `id` |
| Resilience | Exponential backoff retry on 5xx/429; validate `meta.pagination.total_pages` vs fetched count (R-0002) |
| State machine | `idle → running → success \| failed`; persist per-entity progress for Sync Status UI |

**Alternative considered:** Search API `updated_at_after` filter — rejected as sole incremental signal due to unreliable `updated_at` (R-0002, GitHub #8282).

#### 2. PostgreSQL mirror schema

External PostgreSQL with TimescaleDB extension enabled by operator (R-0004). US-0001 uses **relational mirror tables only**; hypertables deferred to US-0002 (DEC-0005).

| Table | Purpose |
|-------|---------|
| `sync_runs` | Run id, started_at, finished_at, status, trigger (scheduled \| manual), error summary |
| `sync_cursors` | Entity type, last_successful_sync_at, records_synced, last_error |
| `accounts` | Firefly `id` (PK), type, name, iban, currency, balance, raw JSONB snapshot, synced_at |
| `transactions` | Firefly `id` (PK), account refs, date, amount, description, category/tag refs, raw JSONB, synced_at |
| `categories` | Firefly `id` (PK), name, raw JSONB, synced_at |
| `budgets` | Firefly `id` (PK), name, amount, period, raw JSONB, synced_at |
| `tags` | Firefly `id` (PK), tag, raw JSONB, synced_at |
| `piggy_banks` | Firefly `id` (PK), name, target_amount, current_amount, raw JSONB, synced_at |
| `firefly_request_audit` | Optional: method, path, status_code, requested_at (read-only verification) |

Migrations via SQLx at startup (`sqlx::migrate!` per R-0004). First migration: `CREATE EXTENSION IF NOT EXISTS timescaledb`.

**Alternative considered:** Normalized transaction split tables — deferred; JSONB snapshot + indexed columns sufficient for US-0001 ingest and US-0002 forecast input.

#### 3. Sync scheduler

- **Engine:** `tokio-cron-scheduler` (or equivalent Tokio-native cron) in backend process (Projectplan stack).
- **Config:** `[sync] interval_seconds` in TOML/env (default 3600); overridable without rebuild.
- **Manual trigger:** `POST /api/v1/sync/trigger` (authenticated); spawns async job, returns 202 + run id.
- **Concurrency:** Mutex/flag prevents overlapping sync runs; second trigger returns 409 with active run id.
- **Observability:** Emit metrics for Grafana (duration, records per entity, error count).

#### 4. Rust/Axum backend

| Module | Responsibility |
|--------|----------------|
| `config` | TOML + env overlay; `database.mode = "external"` required |
| `db` | SQLx pool, migrations, repository traits |
| `auth` | JWT validation middleware via IdP JWKS (DEC-0006, R-0003) |
| `firefly` | GET-only connector, pagination, upsert |
| `sync` | Scheduler, state machine, manual trigger handler |
| `api` | Axum routes, OpenAPI-ready skeleton |
| `health` | `GET /health` (public), `GET /health/ready` (DB connectivity) |

Startup: external DB connection retry with exponential backoff, max ~60s (DEC-0003, R-0005).

#### 5. React UI shell

Per discovery UX refs (`docs/product/vision.md`):

| Route | Content |
|-------|---------|
| `/` (Home) | Welcome card, sync summary stats, read-only badge, links to Sync Status / Settings |
| `/sync` | Entity count cards by Firefly type, last sync time, manual Sync now, sync history table |
| `/settings` | Firefly URL + auth method display, DB mode (external), sync interval (read-only), OIDC issuer display |
| Nav placeholders | Forecast, Subscriptions, Planning, Wealth, AI — disabled + "Coming soon" |

**Layout:** `SidebarProvider` + collapsible icon sidebar; header with sync-status pill and read-only indicator; OIDC user menu in footer (R-0003).

**Data fetching:** TanStack Query polling `/api/v1/sync/status` and entity counts.

#### 6. OIDC authentication

- **Frontend:** `react-oidc-context` + `oidc-client-ts`; redirect login, `onSigninCallback`, silent renew (R-0003).
- **Backend:** Stateless JWT validation on protected routes; public `/health` only (DEC-0006).
- **IdP:** Authentik via optional Compose `oidc` profile; external IdP URL via env supported (DEC-0001).
- **Config:** `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, redirect URIs via Compose/env — no in-app IdP admin in US-0001.

#### 7. Docker Compose profiles

Per Projectplan and R-0005 (DEC-0003):

| Profile | Services | Notes |
|---------|----------|-------|
| `minimal` | `flow-finance-ai`, `firefly-iii`, `grafana` | US-0001 acceptance baseline; no embedded PostgreSQL |
| `standard` | minimal + `redis` | Container present; app **does not depend** on Redis until US-0002+ |
| `full` | standard + `ollama` | Reserved for US-0008 local AI |
| `oidc` | Authentik stack (server, worker, redis, postgres for IdP) | Optional; not in minimal acceptance path |

**External DB wiring:**
- `database.mode = "external"` in TOML; `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`, `DATABASE_USER`, `DATABASE_PASSWORD` from env.
- Linux dev: `extra_hosts: ["host.docker.internal:host-gateway"]` on services needing host DB (R-0005).
- Firefly III uses separate external database (`FIREFLY_DB_*` env vars).

#### 8. Grafana minimal provisioning

- Datasource: PostgreSQL/TimescaleDB → external DB (env-substituted provisioning YAML).
- **Optional Platform Health dashboard:** sync duration, last successful sync, records synced per entity, sync error rate, backend health/uptime (R-0005).
- Dashboards 1–5 (analytics) explicitly **out of scope** — US-0002+.

#### 9. API boundaries (frontend ↔ backend)

| Boundary | Contract |
|----------|----------|
| Auth | SPA sends `Authorization: Bearer <access_token>`; backend validates JWT via JWKS |
| Public | `GET /health`, `GET /health/ready` |
| Protected | All `/api/v1/*` except health — 401 without valid token |
| Sync read | `GET /api/v1/sync/status`, `GET /api/v1/sync/runs`, `GET /api/v1/sync/entities` |
| Sync write | `POST /api/v1/sync/trigger` (manual sync — **Flow DB only**, not Firefly) |
| Settings read | `GET /api/v1/settings` (non-secret config display) |
| Firefly | **No direct browser → Firefly API**; all Firefly traffic server-side via connector |

**Alternative considered:** Backend-for-Frontend (BFF) with session cookies — rejected for US-0001 simplicity; SPA + JWT is sufficient (DEC-0006, R-0003).

### Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Accidental Firefly writes | GET-only client, allowlist, integration test, audit log | DEC-0004, R-0001 |
| Large transaction history slow initial sync | Progress reporting in Sync Status; date-window pagination; configurable interval | R-0002 |
| Unreliable Firefly `updated_at` | 7-day overlap watermark; do not rely on Search API alone | DEC-0002, R-0002 |
| External TimescaleDB not installed | Document operator prerequisite; migration fails clearly on missing extension | R-0004, R-0005 |
| External DB unreachable at startup | Retry/backoff loop ~60s; clear error in logs and `/health/ready` | DEC-0003, R-0005 |
| IdP operational overhead | Keep Authentik out of minimal profile; document external IdP | DEC-0001, R-0003 |
| Linux host DB connectivity | `host.docker.internal:host-gateway` in Compose | R-0005 |
| PAT token exposure | Server-side only; never expose to browser; env/secret mount | R-0001 |

### Decisions (US-0001)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0001 | OIDC IdP default | Authentik optional `oidc` profile; external IdP via env |
| DEC-0002 | Sync watermark | Date-window + 7-day overlap; upsert by Firefly id |
| DEC-0003 | External DB startup | Exponential backoff retry, max ~60s |
| DEC-0004 | Firefly read-only | GET-only HTTP client + audit + integration test |
| DEC-0005 | Schema scope | Relational mirrors in US-0001; hypertables in US-0002 |
| DEC-0006 | API auth pattern | SPA bearer JWT + JWKS validation; no BFF |

Full records: `decisions/DEC-0001.md` … `decisions/DEC-0006.md`

### Out of scope (US-0001)

- Forecast engine, subscription detection, planning, AI, crypto
- Grafana analytics dashboards 1–5
- Redis application dependency (container only in standard profile)
- Hypertables and forecast persistence
- In-app OIDC provider administration

### Next phase

`/sprint-plan` — decompose US-0001 into sprint tasks (connector, schema, Compose, UI shell, OIDC, Grafana provisioning, read-only verification test).

---

## US-0002 — Cashflow forecasting & Grafana analytics (MVP)

**Status:** architecture complete (2026-05-31)  
**Research:** R-0006, R-0007, R-0008 (extends R-0004, R-0005)  
**Decisions:** DEC-0007, DEC-0008, DEC-0009, DEC-0010, DEC-0011, DEC-0012  
**Spec-pack:** `docs/engineering/spec-pack/US-0002-{design-concept,crs,technical-specification}.md`  
**Depends on:** US-0001 mirror tables, sync scheduler, Grafana datasource provisioning

### System context

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Browser — /forecast (React + ECharts)                                       │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ JWT Bearer
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum)                                                      │
│  ┌──────────────┐    ┌─────────────────┐    ┌──────────────────────────┐  │
│  │ SyncService  │───▶│ ForecastService │───▶│ forecast::* modules       │  │
│  │ (US-0001)    │    │ recompute hook  │    │ recurring / rolling /     │  │
│  └──────┬───────┘    └────────┬────────┘    │ project / repository      │  │
│         │ success              │ write         └────────────┬─────────────┘  │
│         │                      │                            │                │
│         ▼                      ▼                            ▼                │
│  Firefly GET-only sync   forecast_computations      forecast_balance_daily   │
│                          (relational)               forecast_cashflow_monthly│
│                                                   (TimescaleDB hypertables)  │
└─────────────────────────────────────────────────────────────────────────────┘
                                ▲
                                │ read latest computation_id
┌───────────────────────────────┴─────────────────────────────────────────────┐
│  Grafana — Dashboard 1 (cashflow) + Dashboard 5 (forecast-horizons)         │
│  PostgreSQL datasource uid: FlowFinancePostgreSQL                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Components

#### 1. Forecast Engine

Rule-based cashflow projection per **DEC-0007** and **R-0006**. No ML (US-0009); no subscription confirm/reject (US-0003).

| Layer | Input | Output |
|-------|-------|--------|
| **Starting balance** | `accounts.balance` for asset account | Day-0 balance |
| **Recurring inference** | `transactions` grouped by payee/description; cadence + ±5% amount tolerance | Scheduled future inflows/outflows |
| **Variable residual** | Non-recurring tx; 3-month rolling daily rate (95th pct cap) | Daily avg variable component |
| **Transfer netting** | Firefly transfer type / own-account pairs | Excluded from cashflow (R-0006) |
| **Category buckets** | Category name → income / fixed / variable via TOML defaults | Monthly decomposition |
| **Day-by-day path** | Layers combined | `balance(d)` for d = 1…730 (~24 months) |
| **Milestones** | Derived from daily path | Tomorrow, +7d, month-end; 3/6/12/24 month end balances |

**Confidence metadata:** If account has <90 days of transactions, set `low_confidence: true` on computation and API meta (widen rolling window to all available history).

**Alternative considered:** Pure rolling average — rejected (misses recurring salary/rent). Full subscription engine first — rejected (US-0003 scope).

#### 2. TimescaleDB hypertable migration

Migration `002_forecast_hypertables.sql` (SQLx at startup per R-0004):

| Object | Type | Purpose |
|--------|------|---------|
| `forecast_computations` | Relational | UUID id, `sync_run_id`, `computed_at`, `status`, `error_message`, `low_confidence` (per account JSONB optional) |
| `forecast_balance_daily` | Hypertable (`ts`, 7-day chunks) | Projected date, `account_id` (TEXT → `accounts.firefly_id`), `computation_id`, `balance` |
| `forecast_cashflow_monthly` | Hypertable (`ts`, 30-day chunks) | Month start, `account_id`, `computation_id`, income/fixed/variable/free_cashflow |

**Latest snapshot query pattern (R-0007):**

```sql
SELECT id FROM forecast_computations
WHERE status = 'success'
ORDER BY computed_at DESC
LIMIT 1;
```

API/Grafana filter all series by that `computation_id`. On failed recompute, serve prior successful snapshot with `stale: true` in meta (R-0007).

**Retention:** Keep last 5 successful computations; cascade-delete forecast rows for removed IDs (**DEC-0011**).

**Alternative considered:** On-demand compute — rejected (acceptance + DEC-0008).

#### 3. Recompute on sync completion

Per acceptance and **DEC-0010**, extend US-0001 sync lifecycle:

```text
execute_run(run_id):
  1. sync_reference_entities + sync_transactions  (existing)
  2. finish_sync_run(success) on ingest OK
  3. ForecastService::recompute(sync_run_id)      (NEW — before mutex release)
  4. active_run = None
```

- Mutex covers sync **and** recompute; `POST /api/v1/sync/trigger` returns 409 during both.
- Forecast failure does **not** fail sync run; prior forecast remains with `stale` metadata.
- Sync Status `state: running` spans recompute duration.

**Alternative considered:** Redis job queue — deferred (no app Redis dependency in US-0002 per US-0001 architecture).

#### 4. Forecast REST API

All routes JWT-protected (DEC-0006). Read-only toward Firefly.

| Method | Path | Query | Response |
|--------|------|-------|----------|
| GET | `/api/v1/forecast/meta` | — | `{ computation_id, computed_at, stale, low_confidence, sync_run_id }` |
| GET | `/api/v1/forecast/accounts` | — | Asset accounts for selector |
| GET | `/api/v1/forecast/daily` | `account_id` | `{ series: [{date, balance}], milestones: { tomorrow, next_week, month_end } }` |
| GET | `/api/v1/forecast/monthly` | `account_id` | `{ series: [{month, income, fixed_costs, variable_costs, free_cashflow}] }` |
| GET | `/api/v1/forecast/long-term` | `account_id`, `horizon=3\|6\|12\|24` | `{ series: [{date, balance}], end_balance }` |
| GET | `/api/v1/forecast/aggregate` | optional `horizon` | Summed series across asset accounts (**DEC-0009**; same-currency MVP) |

Default account scope: **per-account**; React selector defaults to first asset account by name.

#### 5. React `/forecast` page

Enable Forecast nav (replace US-0001 disabled placeholder per `docs/product/vision.md`).

| UI element | Implementation |
|------------|----------------|
| Layout | shadcn Card, Select (account), Tabs or ToggleGroup (Daily \| Monthly \| Long-term) |
| Daily | Stat cards (tomorrow, next week, month-end) + ECharts line chart (current month projection) |
| Monthly | Grouped bar chart (income, fixed, variable, free cashflow) + summary stat row |
| Long-term | Line/area chart; pills 3 / 6 / 12 / 24 months |
| Data | TanStack Query → forecast API endpoints |
| Trust | "Last computed" from `/forecast/meta`; link to Sync Status; empty state if no transactions |

**Dependencies:** Add `echarts` + `echarts-for-react` (or equivalent) to frontend.

#### 6. Grafana Dashboards 1 & 5

Extend US-0001 file provisioning (**R-0008**, **DEC-0012**).

| Dashboard | uid | Folder | Panels |
|-----------|-----|--------|--------|
| Cashflow | `cashflow` | Analytics | Balance time series + forecast overlay; scarcity threshold line (default €200, visual only) |
| Forecast horizons | `forecast-horizons` | Analytics | Stat + time-series per 1 / 3 / 6 / 12 months; optional 24-month panel |

**Provisioning changes:**

- `grafana/provisioning/datasources/postgres.yaml` — add `uid: FlowFinancePostgreSQL`
- Add `grafana/provisioning/dashboards/analytics/cashflow.json`
- Add `grafana/provisioning/dashboards/analytics/forecast-horizons.json`
- Dashboard provider: second provider block for `Analytics` folder or set `"folder": "Analytics"` in JSON

**Template variable:** `$account_id` — `SELECT firefly_id AS __value, name AS __text FROM accounts WHERE type = 'asset' ORDER BY name`

**Query pattern:** Filter `forecast_balance_daily` / `forecast_cashflow_monthly` by latest successful `computation_id` and `$account_id`; use `$__timeFilter(ts)`.

Platform Health dashboard unchanged (US-0001).

### Backend module layout

| Module | Responsibility |
|--------|----------------|
| `forecast::recurring` | Lightweight payee/cadence detection (R-0006) |
| `forecast::rolling` | 3-month variable residual averages |
| `forecast::categories` | Category → bucket mapping from config |
| `forecast::project` | Day-by-day balance path + milestones |
| `forecast::repository` | Hypertable writes, latest computation reads, retention cleanup |
| `forecast::service` | `recompute(sync_run_id)`, orchestrates per asset account |
| `api::forecast` | Axum handlers for forecast routes |

`AppState` gains `forecast: ForecastService`. `SyncService::execute_run` calls forecast after successful ingest.

### Config additions (TOML)

```toml
[forecast]
rolling_window_days = 90
sparse_history_days = 90
retention_count = 5
recurring_amount_tolerance_pct = 5

[forecast.category_buckets]
# defaults: salary→income, rent→fixed, etc.
```

Scarcity threshold for Grafana remains static in dashboard JSON until US-0005 (**DEC-0012**).

### Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Recurring heuristic false positives | `low_confidence` metadata; US-0003 refinement | R-0006, DEC-0007 |
| Sparse transaction history | Widen window; flag; do not fail recompute | R-0006 |
| Recompute latency blocks sync | Mutex extension acceptable for MVP; monitor duration | DEC-0010, R-0007 |
| Hypertable migration on external DB | Operator TimescaleDB prerequisite documented | R-0004, R-0007 |
| Grafana datasource uid mismatch | Set explicit uid in YAML with dashboard JSON | R-0008, DEC-0012 |
| Internal transfer double-count | Exclude transfers in projection layer | R-0006 |
| Multi-currency aggregate | MVP: per-account; aggregate same-currency only | DEC-0009 |
| Failed recompute after good sync | Serve last snapshot + `stale: true` | R-0007, DEC-0010 |

### Decisions (US-0002)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0007 | Forecast algorithm | Hybrid rule-based: recurring heuristics + 3-month rolling avg |
| DEC-0008 | Storage | Precomputed hypertable snapshots on sync |
| DEC-0009 | Account scope | Per-account primary; optional aggregate endpoint |
| DEC-0010 | Recompute trigger | Extends sync mutex; inline after successful ingest |
| DEC-0011 | Retention | Keep last 5 successful computations |
| DEC-0012 | Grafana scarcity | Static €200 threshold; stable dashboard/datasource uids |

Full records: `decisions/DEC-0007.md` … `decisions/DEC-0012.md`

### Out of scope (US-0002)

- ML forecasting (US-0009)
- Subscription confirm/reject adjusting forecasts (US-0003)
- Plan scenario overlays (US-0004)
- Alert Engine firing on scarcity (US-0005) — Dashboard 1 markers visual only
- Grafana Dashboards 2–4
- Redis job queue for async recompute

### Next phase

`/sprint-plan` — decompose US-0002 into sprint **S0002** tasks (migration, Forecast Engine, API, sync hook, React `/forecast`, Grafana dashboards 1 & 5, tests).

---

## US-0003 — Subscription detection, price changes & alerts

**Status:** architecture complete (2026-05-31)  
**Research:** R-0009, R-0010, R-0011, R-0012, R-0013, R-0014 (extends R-0006, R-0008, DEC-0010)  
**Decisions:** DEC-0013, DEC-0014, DEC-0015, DEC-0016, DEC-0017, DEC-0018  
**Spec-pack:** `docs/engineering/spec-pack/US-0003-{design-concept,crs,technical-specification}.md`  
**Depends on:** US-0001 mirror tables + sync scheduler; US-0002 forecast engine + recurring heuristics (DEC-0007)

### System context

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Browser — /subscriptions (React + ECharts price history in detail drawer)   │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ JWT Bearer
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum)                                                      │
│  ┌──────────────┐    ┌─────────────────────┐    ┌───────────────────────┐  │
│  │ SyncService  │───▶│ SubscriptionService │───▶│ ForecastService       │  │
│  │ (US-0001)    │    │ run_detection       │    │ recompute (+ override)│  │
│  └──────┬───────┘    └─────────┬───────────┘    └───────────┬───────────┘  │
│         │ sync                 │ subscriptions phase         │ forecast     │
│         ▼                      ▼                             ▼               │
│  Firefly GET-only sync   recurrence::detect + classify   confirmed override │
│                          subscription_patterns + alerts  rejected exclusion │
└─────────────────────────────────────────────────────────────────────────────┘
                                ▲
                                │ read subscription_* tables
┌───────────────────────────────┴─────────────────────────────────────────────┐
│  Grafana — Dashboard 2 (uid: subscriptions) — Analytics folder               │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Components

#### 1. Shared recurrence core (`backend/src/recurrence/`)

Extract grouping, cadence stability, amount tolerance, and descriptor normalization from `forecast/recurring.rs` into a shared module per **DEC-0013** and **R-0009**. Forecast keeps a thin wrapper; Subscription Engine adds confidence scoring, persistence, and Dauerauftrag classification.

| Submodule | Responsibility |
|-----------|----------------|
| `normalize` | Lowercase trim, collapse whitespace, strip trailing reference tokens (regex: card suffixes, alphanum codes like `P3E460`) |
| `group` | Group expense transactions (`amount < 0`) by normalized payee key; exclude internal transfers (reuse `is_transfer`) |
| `cadence` | Median inter-tx days; classify weekly/biweekly/monthly/quarterly/annual windows; `is_stable_cadence` |
| `amount` | Median amount; tolerance bands per confidence tier |
| `detect` | `detect_recurrence_groups(transactions, config) -> Vec<RecurrenceGroup>` with linked transaction ids |

**Minimum evidence:** ≥3 occurrences spanning ≥2 billing cycles; patterns below 60% confidence **not emitted** (**DEC-0014**).

**Confidence tiers (DEC-0014, R-0009):**

| Tier | Criteria |
|------|----------|
| **95** | ≥4 occurrences; all intervals within cadence tolerance; all amounts within ±5% of median |
| **80** | ≥3 occurrences; stable cadence; all amounts within ±10% of median |
| **60** | ≥3 occurrences; stable cadence; ≤1 interval outlier OR amounts within ±15% of median |

**Fingerprint dedup:** `hash(normalized_key + interval_days + round(median_amount, 2))`; skip if fingerprint in `subscription_rejections` or matches existing confirmed pattern (**DEC-0015**).

**Alternative considered:** Fork separate detection engine — rejected (drift from forecast layer per DEC-0007). ML/DBSCAN — rejected (US-0009 scope).

#### 2. Subscription Engine (`backend/src/subscriptions/`)

| Submodule | Responsibility |
|-----------|----------------|
| `classify` | Dauerauftrag vs subscription rules (**DEC-0016**, R-0010) |
| `detection` | Orchestrate recurrence core → upsert `subscription_patterns`; emit alerts |
| `price_change` | Dual-threshold amount/interval change on confirmed patterns (**DEC-0017**, R-0011) |
| `repository` | SQLx CRUD for patterns, price events, rejections, alerts |
| `service` | `run_detection(sync_run_id) -> DetectionResult` |
| `types` | `ConfirmedRecurring`, `DetectionResult`, API DTOs |

**Detection service contract (R-0013):**

```rust
pub struct DetectionResult {
    pub confirmed_recurring: Vec<ConfirmedRecurring>,
    pub rejected_fingerprints: HashSet<String>,
}

impl SubscriptionService {
    pub async fn run_detection(&self, sync_run_id: Uuid) -> Result<DetectionResult, SubscriptionError>;
}
```

**`run_detection` steps:**

1. Load expense transactions (default window: last 12 months; full rescan on first run or weekly config)
2. Run shared `recurrence::detect` → candidate groups with confidence
3. Classify `kind`: `subscription` | `standing_order` (**DEC-0016**)
4. Upsert `subscription_patterns` (`pending` for new/changed; upgrade confidence on stronger evidence)
5. Skip fingerprints in `subscription_rejections`
6. For **confirmed** patterns: append billing to `subscription_price_events`; check price/interval changes; insert `subscription_alerts`
7. Mark `inactive` when gap > 2× interval without new transaction
8. Return confirmed list + rejection set for forecast hook

**Dauerauftrag classification (DEC-0016, R-0010):**

| Rule | Signal |
|------|--------|
| Exact-amount | Amount CV < 0.02 AND all amounts identical to €0.01 |
| Near-exact | All amounts within ±1% AND interval monthly/quarterly |
| Category boost | ≥50% txs match config category list (`rent`, `miete`, `insurance`, …) |
| Large fixed | Median ≤ −€200 AND amount CV < 0.01 |
| Default | `subscription` |

Optional TOML `standing_order_payee_patterns` merges with rules (not sole signal). User may override `kind` on confirm.

**Price-change detection (DEC-0017, R-0011):**

- Scope: **confirmed** subscriptions only, after each detection pass
- Fire when **both** `|delta| ≥ €1.00` AND `|delta|/|previous| × 100 ≥ 5%` (TOML-configurable)
- Append `subscription_price_events` on every billing occurrence (powers ECharts + Grafana)
- Interval change when new median differs beyond cadence tolerance (e.g. monthly → weekly)
- Alert types: `new_detection`, `price_change`, `interval_change`

**Failure semantics:** detection failure → log warning, continue to forecast with prior confirmed state; sync run still `success` if ingest succeeded (mirrors forecast failure handling per DEC-0010).

#### 3. Migration `003_subscriptions.sql`

Relational tables per **DEC-0015** and **R-0012** (no hypertables — row volume ≪ forecast series):

| Object | Purpose |
|--------|---------|
| `subscription_status` enum | `pending`, `confirmed`, `rejected`, `inactive` |
| `subscription_kind` enum | `subscription`, `standing_order` |
| `subscription_alert_type` enum | `new_detection`, `price_change`, `interval_change` |
| `subscription_patterns` | Single lifecycle entity: fingerprint, status, kind, payee_key, display_name, interval_days, current_amount, confidence_pct, timestamps |
| `subscription_pattern_transactions` | M:N link pattern ↔ mirror `transactions` |
| `subscription_price_events` | Append-only billing + change events |
| `subscription_rejections` | Fingerprint PK; permanent exclusion from detection + forecast |
| `subscription_alerts` | Page-scoped alerts with nullable `read_at` |

Indexes: `(status, kind)`, `(last_seen_at DESC)`, `(pattern_id, occurred_at DESC)`, `(read_at, created_at DESC)`.

**State transitions:**

- Detection → `pending` (new/changed candidate)
- User confirm → `confirmed` (+ optional kind override)
- User reject → `rejected` + insert `subscription_rejections(fingerprint)`
- Gap > 2× interval → `inactive` (history retained; no new alerts)

**Alternative considered:** Separate `candidates` + `subscriptions` tables — rejected (duplicate columns, awkward promotion). Firefly tags for state — rejected (DEC-0004 read-only).

#### 4. Sync pipeline extension

Extends **DEC-0010** mutex with subscriptions phase before forecast per **DEC-0018** and **R-0013**:

```text
execute_run(run_id):
  1. Firefly entity + transaction sync          (phase: "sync")
  2. finish_sync_run(success) on ingest OK
  3. SubscriptionService::run_detection(run_id) (phase: "subscriptions")  ← NEW
  4. ForecastService::recompute(run_id, detection_result)                 (phase: "forecast")
  5. active_run = None; phase = None
```

- Mutex covers sync + detection + forecast; `POST /api/v1/sync/trigger` returns 409 during any phase
- Sync Status UI displays `phase`: `"sync"` | `"subscriptions"` | `"forecast"`
- Detection **before** forecast ensures confirmed subs override heuristics in same cycle (AC-8)

**Alternative considered:** Detection after forecast — rejected (one-cycle lag). Async Redis queue — deferred until combined latency > ~30s (DEC-0010 precedent).

**Incremental detection window:** default last 12 months of transactions; configurable `detection_window_days`; optional weekly full rescan via TOML.

#### 5. Forecast override hook

Extend `ForecastService::recompute` to accept `DetectionResult` (**R-0013**, AC-8):

```rust
pub async fn recompute(
    &self,
    sync_run_id: Uuid,
    subscription_context: Option<&DetectionResult>,
) -> Result<Uuid, ForecastError>;
```

**Override rules in `project_account`:**

1. Load `confirmed_recurring` from detection result (or repository fallback if detection skipped)
2. For each confirmed pattern: **replace** heuristic `RecurringPattern` with same `payee_key` (confirmed amount + interval take precedence)
3. Exclude any transaction group whose fingerprint is in `rejected_fingerprints`
4. Heuristic-only patterns for payees without confirmed/rejected state unchanged (DEC-0007 baseline)

Confirmed patterns use user-validated amount/interval; rejected patterns never appear in projection or subscription alerts.

#### 6. Subscription REST API

All routes JWT-protected (DEC-0006). Read-only toward Firefly.

| Method | Path | Query / Body | Response |
|--------|------|--------------|----------|
| GET | `/api/v1/subscriptions` | `status`, `kind` | List patterns with confidence badge metadata |
| GET | `/api/v1/subscriptions/{id}` | — | Pattern detail + linked transaction count |
| POST | `/api/v1/subscriptions/{id}/confirm` | `{ "kind"?: "subscription"\|"standing_order" }` | Updated pattern |
| POST | `/api/v1/subscriptions/{id}/reject` | `{ "reason"?: string }` | Pattern rejected + fingerprint stored |
| GET | `/api/v1/subscriptions/{id}/price-history` | — | `{ events: [{ occurred_at, amount, event_type, … }] }` |
| GET | `/api/v1/subscriptions/alerts` | `unread=true` | Alert list for banner/toast |
| PATCH | `/api/v1/subscriptions/alerts/{id}/read` | — | Mark alert read |

**Alert delivery (R-0011):** MVP in-app banner on `/subscriptions` when unread alerts exist + toast after sync for new detection/price change. Global header bell deferred to US-0005.

#### 7. React `/subscriptions` page

Enable Subscriptions nav (replace US-0001 disabled placeholder per `docs/product/vision.md`).

| UI element | Implementation |
|------------|----------------|
| Layout | shadcn Tabs: **All** \| **Pending review** \| **Standing orders** |
| Pending | Card per candidate: payee, interval, amount, confidence Badge (95/80/60); Confirm / Reject actions |
| Confirm dialog | Optional kind override dropdown (subscription vs standing order) |
| Confirmed | Table: display name, interval, amount, kind badge, last seen |
| Detail drawer | Sheet with linked transactions + ECharts price history line |
| Alerts | Banner when unread alerts; toast on new items after sync poll |
| Empty state | No patterns detected yet; link to Sync Status |

**Data:** TanStack Query → subscription API; poll `/subscriptions/alerts?unread=true` after sync completion.

#### 8. Grafana Dashboard 2 (Subscriptions)

Extend US-0002 file provisioning (**R-0014**, **DEC-0012** pattern).

| Dashboard | uid | Folder | Panels |
|-----------|-----|--------|--------|
| Subscriptions | `subscriptions` | Analytics | Confirmed count stat; monthly spend stat (interval-normalized); pending count stat; price-change table (90d); new detections time series |

**Provisioning:** add `grafana/provisioning/dashboards/analytics/subscriptions.json`.

**Panel query notes (R-0014):**

- Monthly spend stat: SQL normalizes weekly (×4.33), annual (÷12), etc. to monthly equivalent
- Price changes: **table** with before/after columns (not time series)
- New detections: daily count time series by `subscription_patterns.created_at`
- MVP scope: **global** (no per-account variable); patterns are payee-scoped

Platform Health, Dashboard 1, and Dashboard 5 unchanged.

### Backend module layout

| Module | Responsibility |
|--------|----------------|
| `recurrence::*` | Shared detection core extracted from forecast (DEC-0013) |
| `forecast::recurring` | Thin wrapper calling `recurrence::detect` for projection |
| `subscriptions::{classify,detection,price_change,repository,service,types}` | Subscription Engine |
| `api::subscriptions` | Axum handlers |
| `sync` | Inject `SubscriptionService`; extend phase reporting |

`AppState` gains `subscriptions: SubscriptionService`. `SyncService::execute_run` wires detection → forecast with `DetectionResult`.

### Config additions (TOML)

```toml
[subscriptions]
detection_window_days = 365
full_rescan_interval_days = 7
confidence_tiers = { high = 95, medium = 80, low = 60 }
price_change_min_eur = 1.0
price_change_min_pct = 5.0
inactive_grace_days = 5
standing_order_category_patterns = ["rent", "miete", "insurance", "versicherung", "utilities", "nebenkosten", "loan", "darlehen"]
standing_order_payee_patterns = []  # optional operator override

[subscriptions.confidence_tolerance_pct]
high = 5
medium = 10
low = 15
```

### Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Descriptor normalization drift splits groups | Regex strip trailing codes; document known limitation | R-0009 |
| 60% tier false positives | Strong confirm/reject UX before alerts propagate | R-0009, DEC-0014 |
| Dauerauftrag misclassification | Rule-based + user kind override on confirm | R-0010, DEC-0016 |
| Promotional pricing false alarms | Document known limitation; dual threshold reduces noise | R-0011 |
| Sync mutex duration grows | Monitor combined sync+detection+forecast; defer queue if > ~30s | DEC-0018, R-0013 |
| Fingerprint collisions | Conservative normalization; unique constraint on fingerprint | R-0012 |
| Grafana monthly-spend SQL complexity | Shared normalization expression; empty-state panels | R-0014 |
| Annual subs need 2+ years data | Flag low confidence; require user confirm | R-0009 |

### Decisions (US-0003)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0013 | Recurrence core | Extract shared `recurrence` module; forecast wrapper + subscription engine |
| DEC-0014 | Confidence tiers | 95/80/60% tiers; emit only ≥60%; min 3 txs |
| DEC-0015 | Persistence schema | Single lifecycle `subscription_patterns` + satellites; rejection fingerprints |
| DEC-0016 | Dauerauftrag | Rule-based classification + optional config patterns; user override on confirm |
| DEC-0017 | Price-change threshold | Dual: ≥€1.00 AND ≥5%; configurable in TOML |
| DEC-0018 | Sync pipeline order | Inline `subscriptions` phase before forecast in sync mutex |

Full records: `decisions/DEC-0013.md` … `decisions/DEC-0018.md`

### Out of scope (US-0003)

- Plan cancel-impact scenarios (US-0004)
- Unified Alert Engine inbox / header bell (US-0005)
- AI `get_subscriptions` tool implementation (US-0006)
- Redis/async job queue for detection
- Per-account Grafana variable for subscriptions (MVP global)
- Any write to Firefly III

### Next phase

`/plan-verify` — confirm S0003 task coverage against acceptance criteria, then `/execute` T-0025 … T-0036.

---

## US-0004 — Financial planning, scenarios & plan-vs-actual

**Status:** architecture complete (2026-05-31)  
**Research:** R-0015, R-0016, R-0017, R-0018, R-0019, R-0020 (extends R-0006, R-0007, R-0012, R-0008, DEC-0010)  
**Decisions:** DEC-0019, DEC-0020, DEC-0021, DEC-0022, DEC-0023, DEC-0024  
**Spec-pack:** `docs/engineering/spec-pack/US-0004-{design-concept,crs,technical-specification}.md`  
**Depends on:** US-0001 mirror tables + sync; US-0002 forecast snapshots; US-0003 confirmed subscriptions (savings-mode template)

### System context

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Browser — /planning (React + ECharts: compare bar, plan-vs-Ist dual line)   │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ JWT Bearer
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum)                                                      │
│  ┌──────────────┐    ┌─────────────────────┐    ┌───────────────────────┐  │
│  │ PlanService  │───▶│ PlanEngine::overlay │───▶│ plan_daily_cashflow   │  │
│  │ (CRUD+active)│    │ on forecast baseline│    │ (TimescaleDB hypertable)│  │
│  └──────┬───────┘    └─────────┬───────────┘    └───────────────────────┘  │
│         │ plan save            │ reads latest forecast_computation          │
│         ▼                      ▼                                             │
│  plans / plan_versions / plan_adjustments          mirror transactions (Ist) │
│                                                                               │
│  ForecastService::recompute (post-sync) ──hook──▶ recompute active plan only │
└─────────────────────────────────────────────────────────────────────────────┘
                                ▲
                                │ read plan_* + transactions
┌───────────────────────────────┴─────────────────────────────────────────────┐
│  Grafana — Dashboard 3 (uid: budgets) — Plan / Ist / Abweichung              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Components

#### 1. Plan Engine — delta overlay (`backend/src/plan/`)

Apply user-defined **adjustments** atop the latest successful forecast computation (**DEC-0019**, **R-0015**). Do not fork or duplicate `ForecastService` projection logic.

| Submodule | Responsibility |
|-----------|----------------|
| `types` | `PlanAdjustment`, `PlanTemplate`, `PlanComputationMeta`, API DTOs |
| `overlay` | Map adjustments → daily net-cashflow deltas; subscription removals match `payee_key` |
| `project` | Merge baseline `forecast_balance_daily` / cashflow layers with overlay; emit `plan_daily_cashflow` |
| `templates` | Preset delta bundles for built-in scenarios (editable before save) |
| `repository` | SQLx CRUD for plans, versions, adjustments, computations |
| `service` | `recompute_version`, `apply_template`, `create_version`, `set_active` |

**Adjustment model (R-0015):**

```rust
pub struct PlanAdjustment {
    pub direction: AddOutflow | RemoveOutflow | AddInflow | RemoveInflow,
    pub amount: Decimal,              // positive magnitude
    pub frequency: Monthly | Weekly | Quarterly | OneTime,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub target_type: Household | Subscription | Category | CustomLabel,
    pub target_ref: Option<String>,   // subscription UUID or category firefly_id
    pub label: String,
}
```

**Overlay algorithm:**

1. Resolve `baseline_computation_id` = latest successful `forecast_computations.id`
2. Load baseline daily net cashflow from forecast engine output (household aggregate per **DEC-0021**)
3. For each adjustment, project recurring impact onto future calendar days (reuse cadence math from forecast recurring layer)
4. One-time deltas apply on `effective_from` only
5. `target_type = subscription` + `remove_outflow` → zero matching confirmed subscription recurring amount in overlay layer (**US-0003** `payee_key` match)
6. Persist `plan_computations` + `plan_daily_cashflow` rows; record `forecast_computation_id` binding (**DEC-0023**)

**Built-in templates (defaults, user-editable on apply — TOML overridable):**

| Template | Default deltas |
|----------|----------------|
| **Current (Ist)** | none (baseline only) |
| **Leasing** | `+€300/month` household outflow, label "Leasing" |
| **Savings mode** | auto-suggest confirmed subscriptions (US-0003); user confirms selection; optional `−€100/month` discretionary cut |
| **House purchase** | `+€X/month` savings transfer (default €500; category or custom label) |
| **Custom** | empty; user adds lines |

**Savings mode flow (R-0015, R-0018):** `GET /api/v1/plans/templates/savings-mode/suggestions` returns confirmed `subscription_patterns`; user selects IDs before `POST .../apply-template`.

**Alternative considered:** Independent Plan Engine re-running full projection — rejected (drift vs DEC-0007; R-0015). Mutate forecast hypertables — rejected (breaks Ist baseline; DEC-0004).

#### 2. Plan versioning (`DEC-0020`, **R-0016**)

| Concept | Rule |
|---------|------|
| Named plan | Container: `name`, `template`, `is_active` |
| Versions | `version_number` 1–3 per plan; **hard cap 3** |
| Latest version | `is_latest=true`; **editable in-place** |
| Create new version | Freeze prior (`frozen_at` set); copy adjustments to new row; new `is_latest` |
| v4 attempt | HTTP **409** with message to archive or create new named plan |

**Compare metrics (table-first, ECharts grouped bar secondary):**

- Monthly delta sum (net recurring impact €/month)
- Projected month-end balance (household aggregate, current + next month)
- Optional: free-cashflow delta vs Current (Ist) baseline

#### 3. Active plan semantics (**DEC-0024**)

- Exactly **one** plan with `is_active=true` globally (`plans_one_active` partial unique index)
- `set_active` API: transactional deactivate-all → activate target plan's latest version
- Active plan drives: plan-vs-Ist API, React default, Grafana `$active_plan_version` variable
- Race mitigation: row-level lock on `plans` during activate; reject if concurrent version edit on stale `is_latest`

#### 4. Migration `004_plans.sql` (**DEC-0022**, **R-0018**)

| Object | Purpose |
|--------|---------|
| `plan_template` enum | `current`, `leasing`, `savings_mode`, `house_purchase`, `custom` |
| `plan_adjustment_*` enums | direction, frequency, target |
| `plans` | Named plan + `is_active` |
| `plan_versions` | v1–v3; `is_latest`, `frozen_at`, `baseline_computation_id` |
| `plan_adjustments` | Delta lines per version |
| `plan_computations` | Recompute metadata; links `forecast_computation_id` |
| `plan_daily_cashflow` | Hypertable: `ts`, `planned_net`, optional `planned_balance` |

Indexes: `plans_one_active` (partial unique), `plan_versions_one_latest` (partial unique per plan), `(version_id, computation_id, ts DESC)`.

Retention: last **3** successful `plan_computations` per version (lighter than forecast's 5).

**Alternative considered:** JSONB adjustment blob — rejected (Grafana SQL needs relational grain). Store plan series in forecast hypertables — rejected (namespace collision).

#### 5. Plan-vs-Ist computation (**DEC-0021**, **R-0017**)

**Primary metric:** household **daily net cashflow** (signed sum of non-transfer asset-account transactions per calendar day).

```sql
-- Ist (actual) — read-only from mirror
SELECT t.date AS day, SUM(t.amount::numeric) AS actual_net
FROM transactions t
JOIN accounts a ON a.firefly_id = t.account_id
WHERE a.type = 'asset'
  AND COALESCE(t.payload->>'type', '') != 'transfer'
GROUP BY t.date;
```

**Planned:** `plan_daily_cashflow.planned_net` for active latest version's latest successful computation.

**Deviation (Abweichung):** `deviation = actual_net - planned_net` (positive = better than plan).

**API response metadata:**

- `actuals_stale: bool` when `last_successful_sync_at` lags
- `plan_stale: bool` when plan computation older than latest forecast or recompute failed
- `reporting_currency: string` — MVP single household currency (**R-0017**)

**Secondary drill-down:** per-category actual vs category-targeted plan deltas in React **Plan vs Actual** tab only (not Grafana MVP).

**Category drill-down deferred** to US-0005 budget-drift Grafana panels.

#### 6. Recompute triggers (**DEC-0023**, **R-0019**)

| Trigger | Behavior |
|---------|----------|
| Plan mutation (save, template apply, adjustment CRUD, new version, `set_active`) | `tokio::spawn` recompute affected version **outside** sync mutex |
| Post-forecast success (sync or manual) | If active plan exists → recompute active latest version with new `forecast_computation_id` |
| Sync mutex | **No** `"planning"` phase (plan work ≪ forecast; DEC-0010 latency precedent) |

```rust
// End of ForecastService::recompute success:
if let Some(active) = plan_service.active_plan().await? {
    plan_service.recompute_version(active.latest_version_id, computation_id).await?;
}
```

Failure: plan recompute failure does not fail sync; API serves last successful plan snapshot with `plan_stale=true`.

**`/forecast` overlay:** **deferred** — planning-only surface in US-0004; optional "View in Planning" link from forecast page (**R-0019**).

#### 7. Plan REST API

All routes JWT-protected (DEC-0006). No Firefly writes (DEC-0004).

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/plans` | List plans with latest version summary |
| POST | `/api/v1/plans` | Create plan `{ name, template? }` |
| GET | `/api/v1/plans/{id}` | Plan detail + versions |
| PATCH | `/api/v1/plans/{id}` | Rename plan |
| DELETE | `/api/v1/plans/{id}` | Delete plan (cascade versions) |
| POST | `/api/v1/plans/{id}/activate` | Set global active plan |
| GET | `/api/v1/plans/{id}/versions` | List v1–v3 |
| POST | `/api/v1/plans/{id}/versions` | Create new version from latest (409 if at cap) |
| GET | `/api/v1/plans/{id}/versions/{vid}` | Version + adjustments |
| PATCH | `/api/v1/plans/{id}/versions/{vid}` | Edit adjustments (latest only; 409 if frozen) |
| POST | `/api/v1/plans/{id}/versions/{vid}/adjustments` | Add adjustment line |
| PATCH | `/api/v1/plans/{id}/versions/{vid}/adjustments/{aid}` | Update line |
| DELETE | `/api/v1/plans/{id}/versions/{vid}/adjustments/{aid}` | Remove line |
| POST | `/api/v1/plans/{id}/versions/{vid}/apply-template` | Apply template deltas `{ template, overrides? }` |
| GET | `/api/v1/plans/templates/savings-mode/suggestions` | Confirmed subscriptions for savings mode |
| GET | `/api/v1/plans/{id}/compare` | Side-by-side v1/v2/v3 metrics |
| GET | `/api/v1/plans/active/plan-vs-actual` | Daily planned/actual/deviation for active plan `{ month?: YYYY-MM }` |
| POST | `/api/v1/plans/{id}/versions/{vid}/recompute` | Manual recompute (operator/debug) |

#### 8. React `/planning` page

Enable Planning nav (replace US-0001 disabled placeholder per `docs/product/vision.md`).

| UI element | Implementation |
|------------|----------------|
| Header | Active plan selector (`Select`); "Set active" action |
| Tabs | **Scenarios** \| **Compare** \| **Plan vs Actual** |
| Scenarios | Template cards (Current, Leasing, Savings, House, Custom); adjustment table (amount, frequency, target); "Create new version" when editing frozen history |
| Savings mode | Modal: suggested confirmed subscriptions with checkboxes before apply |
| Compare | Metrics table v1/v2/v3 + ECharts grouped bar (month-end balance, monthly delta sum) |
| Plan vs Actual | Dual-line ECharts (planned vs actual) + daily table (planned, Ist, deviation); current month default |
| Stale badges | Show when `actuals_stale` or `plan_stale` from API |
| Empty state | No plans yet; CTA to create from template |

**Data:** TanStack Query → plan API; poll after plan save for recompute completion (optional short polling or return computation status in PATCH response).

#### 9. Grafana Dashboard 3 (Budgets) (**DEC-0024**, **R-0020**)

Extend US-0002/0003 file provisioning (**DEC-0012** pattern).

| Dashboard | uid | Folder | Panels |
|-----------|-----|--------|--------|
| Budgets | `budgets` | Analytics | Plan time series; Ist time series; Abweichung (deviation); MTD summary table; active plan stat |

**Template variable:** `$active_plan_version` — latest version of `is_active=true` plan.

**MVP scope:** **household aggregate only** — no per-category Grafana panels (category drill-down in React; US-0005 for budget drift).

**Empty state:** dashboard annotation when no active plan — "Select active plan in Flow Finance Planning UI".

**Deviation SQL:** FULL OUTER JOIN actuals and planned on calendar day; cast to `timestamptz` for Grafana time axis (R-0020).

Platform Health, Dashboards 1, 2, 5 unchanged.

### Backend module layout

| Module | Responsibility |
|--------|----------------|
| `plan::{types,overlay,project,templates,repository,service}` | Plan Engine |
| `api::plans` | Axum handlers |
| `forecast::service` | Post-recompute active-plan hook |
| `sync` | No new phase; hook only via forecast completion |

`AppState` gains `plans: PlanService`.

### Config additions (TOML)

```toml
[plans]
leasing_default_monthly_eur = 300.0
house_purchase_default_savings_eur = 500.0
savings_mode_discretionary_cut_eur = 100.0
max_versions_per_plan = 3
computation_retention_per_version = 3
reporting_currency = "EUR"   # MVP single-currency household aggregate
```

### Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Baseline staleness | Bind `forecast_computation_id`; expose `plan_stale` / `actuals_stale` | R-0015, R-0019 |
| Savings mode Ist lag | Deviation surfaces "plan assumes cancelled, still paying" | R-0017 |
| Version cap surprise | Clear 409 message on v4; UX copy on Compare tab | R-0016, DEC-0020 |
| Active plan race | Transactional activate; lock `is_latest` edits | R-0018, DEC-0024 |
| Grafana empty active plan | Annotation + friendly stat text | R-0020 |
| Multi-currency | MVP single reporting currency in config | R-0017 |
| Plan/forecast recompute race | Version-level guard; `is_latest` check before write | R-0019 |
| Future days in deviation chart | Document planned extends beyond today; Ist stops at sync date | R-0020 |

### Decisions (US-0004)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0019 | Plan projection model | Delta overlay on latest forecast baseline; templates as preset adjustments |
| DEC-0020 | Version semantics | Hybrid: latest editable; freeze on new version; max 3 versions per plan |
| DEC-0021 | Plan-vs-Ist metric | Household daily net cashflow; deviation = actual − planned |
| DEC-0022 | Persistence schema | Migration 004: plans, versions, adjustments, computations, plan_daily hypertable |
| DEC-0023 | Recompute triggers | Plan save (async) + post-forecast hook; no sync mutex phase; defer /forecast overlay |
| DEC-0024 | Active plan & Dashboard 3 | Single global active plan; Grafana uid `budgets`; household aggregate MVP |

Full records: `decisions/DEC-0019.md` … `decisions/DEC-0024.md`

### Out of scope (US-0004)

- AI `simulate_plan` chat tool (US-0006)
- Crypto allocation scenarios (US-0007)
- Plan viability / budget-drift Alert Engine (US-0005)
- Active plan overlay on `/forecast` page
- Per-category Grafana Dashboard 3 panels
- Any write to Firefly III
- Multi-currency conversion

### Next phase

`/sprint-plan` — S0004 task decomposition against 6 acceptance criteria.

---

## US-0005 — Wealth analysis, budget drift & scarcity alerts

**Status:** architecture complete (2026-05-31)  
**Research:** R-0021, R-0022, R-0023, R-0024, R-0025, R-0026 (extends R-0006, R-0007, R-0015, R-0017, R-0018, R-0008, DEC-0010, DEC-0018, DEC-0023)  
**Decisions:** DEC-0025, DEC-0026, DEC-0027, DEC-0028, DEC-0029, DEC-0030  
**Spec-pack:** `docs/engineering/spec-pack/US-0005-{design-concept,crs,technical-specification}.md`  
**Depends on:** US-0001 mirror accounts; US-0002 forecast snapshots; US-0003 subscription alerts boundary; US-0004 active plan + category-targeted adjustments

### System context

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Browser — /wealth (net worth) + /alerts (inbox) + header bell (Popover)   │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ JWT Bearer
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum)                                                      │
│  Sync mutex: sync → subscriptions → forecast (+ plan hook) → alerts → done  │
│                                                                               │
│  ┌──────────────┐    ┌─────────────────┐    ┌────────────────────────────┐  │
│  │ WealthService│───▶│ net_worth_      │    │ AlertService::evaluate     │  │
│  │ (breakdown)  │    │ snapshots       │    │ scarcity | budget_drift |  │  │
│  └──────────────┘    └─────────────────┘    │ plan_viability           │  │
│         ▲                      ▲               └─────────────┬──────────────┘  │
│         │                      │                             │ upsert/resolve │
│  mirror accounts               │ post-sync upsert            ▼                │
│  (asset, include_net_worth)    │                    alerts + alert_config     │
│                                │                             ▲                │
│  forecast_balance_daily ───────┴── scarcity input            │ TOML [alerts]  │
│  plan_daily_cashflow ───────────── plan viability             │ startup mirror │
│  transactions (MTD) ────────────── budget drift               └────────────────│
└─────────────────────────────────────────────────────────────────────────────┘
                                ▲
                                │ read accounts, snapshots, alert_config
┌───────────────────────────────┴─────────────────────────────────────────────┐
│  Grafana — Dashboard 4 (uid: portfolio) + Dashboard 1 $scarcity_threshold    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Components

#### 1. Net worth aggregation (`backend/src/wealth/`)

Sum synced Firefly **asset** accounts for household wealth headline (**DEC-0025**, **R-0021**).

| Submodule | Responsibility |
|-----------|----------------|
| `types` | `NetWorthBreakdown`, `AccountWealthRow`, `WealthHistoryPoint`, API DTOs |
| `repository` | Query mirror `accounts`; upsert/read `net_worth_snapshots` |
| `service` | `compute_breakdown`, `upsert_daily_snapshot`, `history` |

**Account filter:**

```sql
SELECT firefly_id, name, type, currency, balance,
       payload->>'account_role' AS account_role
FROM accounts
WHERE type = 'asset'
  AND COALESCE((payload->>'active')::boolean, true) = true
  AND COALESCE((payload->>'include_net_worth')::boolean, true) = true
  AND balance >= 0   -- MVP: exclude ccAsset debt until liability integration
ORDER BY name;
```

**Household total:** `SUM(balance)` with reporting currency EUR default. If `COUNT(DISTINCT currency) > 1` → `mixed_currency=true` + UI disclaimer (no FX conversion in MVP).

**Crypto:** excluded from total; static placeholder row in React ("Connect exchanges — US-0007").

**Daily snapshots:** upsert one `net_worth_snapshots` row per calendar day during post-sync alerts phase; `payload` JSONB stores per-account breakdown array for trend chart + Grafana.

**Alternative considered:** On-demand compute without snapshots — rejected (Dashboard 4 wealth-over-time + acceptance trend requirement).

#### 2. Alert Engine (`backend/src/alerts/`)

Centralized post-sync evaluation (**DEC-0026**, **R-0022**). Types: `scarcity`, `budget_drift`, `plan_viability`.

| Submodule | Responsibility |
|-----------|----------------|
| `types` | `AlertType`, `AlertSeverity`, `AlertStatus`, `EvalContext`, `AlertsConfig` |
| `evaluate` | `evaluate_scarcity`, `evaluate_budget_drift`, `evaluate_plan_viability` |
| `repository` | Fingerprint upsert, resolve, acknowledge, dismiss, unread count |
| `service` | `run_post_sync`, `list`, `acknowledge`, `dismiss` |

**Evaluation contract:**

```rust
AlertService::run_post_sync(run_id, EvalContext {
    forecast_computation_id,
    plan_computation_id: Option<Uuid>,
    config: AlertsConfig,
}) -> AlertEvalResult {
    // 1. WealthService::upsert_daily_snapshot(run_id)
    // 2. evaluate scarcity, budget_drift, plan_viability
    // 3. upsert/resolve alerts per fingerprint (DEC-0027)
}
```

**Scarcity (household aggregate — DEC-0026):**
- Sum `forecast_balance_daily.balance` across asset accounts per projected day
- Breach when any day in `[today, today+45d]` OR current-month month-end < `scarcity_threshold_eur`
- Severity: `critical` if breach tomorrow or current balance already below; else `warning`
- Fingerprint: `scarcity:household:{YYYY-MM}`

**Budget drift (category-targeted plan adjustments only — DEC-0026):**
- Load active plan latest version adjustments where `target_type = category`
- MTD actual = sum expense amounts (abs) for category in current month from mirror
- MTD target = prorated monthly plan delta (`monthly × days_elapsed / days_in_month`)
- Fire when `actual > target × (1 + budget_drift_pct/100)`
- Fingerprint: `budget_drift:category:{firefly_id}:{YYYY-MM}`

**Plan viability (active plan overlay — DEC-0026):**
- Primary: `planned_balance` at current month-end < 0 (household from plan overlay)
- Secondary: current + next month-end both < 0
- Requires active plan; skip when none
- Fingerprint: `plan_viability:{plan_id}:{version_id}`

**Alternative considered:** Per-account scarcity alerts — rejected (noisy; Dashboard 1 retains per-account visual threshold).

#### 3. Alert persistence & lifecycle (**DEC-0027**, **R-0023**)

Migration 005 `alerts` table with partial unique index on `fingerprint` WHERE `status IN ('active', 'acknowledged')`.

| Status | Behavior |
|--------|----------|
| `active` | Surfaces in bell + `/alerts`; unread if `acknowledged_at IS NULL` |
| `acknowledged` | Muted styling; excluded from unread count |
| `dismissed` | Hidden from bell + active list while condition persists |
| `resolved` | Condition cleared; historical only |

**Dismiss semantics (DEC-0027):** hide until condition clears or re-triggers — not permanent suppress. Re-breach after clear creates new row.

**US-0003 boundary:** `subscription_alerts` unchanged. Optional header link to subscription unread count (read-only cross-link).

#### 4. Migration `005_alerts_wealth.sql`

| Object | Purpose |
|--------|---------|
| `alert_type`, `alert_severity`, `alert_status` enums | Alert taxonomy |
| `alert_config` | Singleton threshold mirror for Grafana + engine (DEC-0029) |
| `alerts` | Unified inbox persistence with fingerprint dedup |
| `net_worth_snapshots` | Daily wealth trend (`UNIQUE(snapshot_date)`) |

Indexes: `alerts_active_fingerprint` (partial unique), `alerts_status_triggered`, `alerts_unread` (partial), `net_worth_snapshots(snapshot_date DESC)`.

Seed: `INSERT INTO alert_config DEFAULT VALUES;`

**Alternative considered:** Redis dedup — deferred (PostgreSQL sufficient at household scale).

#### 5. Threshold centralization (**DEC-0029**, **R-0025**)

**TOML:**

```toml
[alerts]
scarcity_threshold_eur = 200.0
budget_drift_pct = 20.0
```

**Startup:** load TOML → in-memory `AlertsConfig` + UPSERT `alert_config` singleton.

**Dashboard 1 migration:** replace static `200` in `cashflow.json` refId B with Grafana query variable `$scarcity_threshold`:

```sql
-- Template variable query
SELECT scarcity_threshold_eur AS __value, 'Scarcity threshold (€)' AS __text
FROM alert_config WHERE id = 1;

-- refId B (constant threshold line)
SELECT ts AS time, $scarcity_threshold::numeric AS value
FROM forecast_balance_daily
WHERE account_id = '$account_id'
  AND computation_id = (SELECT id FROM forecast_computations WHERE status='success' ORDER BY computed_at DESC LIMIT 1)
  AND $__timeFilter(ts)
ORDER BY ts LIMIT 1;
```

Supersedes DEC-0012 static €200 hardcode; stable uids unchanged.

#### 6. Sync pipeline — `"alerts"` phase (**DEC-0028**, **R-0024**)

Extends DEC-0018 pipeline:

```
1. Firefly sync              (phase: "sync")
2. Subscription detection    (phase: "subscriptions")
3. Forecast recompute        (phase: "forecast")
   └─ inline active plan refresh (DEC-0023, awaited inside forecast)
4. Net worth snapshot        (phase: "alerts")
5. Alert Engine evaluate     (phase: "alerts")
6. Clear mutex
```

- Inline in same Tokio task (DEC-0010 precedent)
- Failure non-blocking: log warning; preserve last alert/snapshot state
- Defer async queue if combined pipeline > ~30s

**Alternative considered:** Alerts before forecast — rejected (needs latest forecast/plan snapshots).

#### 7. Wealth & Alert REST API

All routes JWT-protected (DEC-0006). No Firefly writes (DEC-0004).

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/wealth` | Net worth breakdown + metadata (`mixed_currency`, `reporting_currency`, crypto placeholder flag) |
| GET | `/api/v1/wealth/history?days=90` | Trend series from `net_worth_snapshots` |
| GET | `/api/v1/alerts?status=active` | Inbox list (excludes dismissed unless `?include_dismissed=true`) |
| GET | `/api/v1/alerts/unread-count` | Header bell badge |
| PATCH | `/api/v1/alerts/{id}/acknowledge` | Mark read |
| PATCH | `/api/v1/alerts/{id}/dismiss` | Suppress until clear/re-trigger |
| POST | `/api/v1/alerts/evaluate` | Manual re-eval (operator/debug; optional stretch) |

Subscription alerts remain at `/api/v1/subscriptions/alerts/*` (US-0003).

#### 8. React `/wealth` page (**DEC-0030**, **R-0026**)

Enable Wealth nav (replace US-0001 disabled placeholder per `docs/product/vision.md`).

| UI element | Implementation |
|------------|----------------|
| Overview tab | Net worth stat card (EUR headline); mixed-currency warning `Alert` banner |
| Account breakdown | Table: name, role label, currency, balance, % of total (same-currency group when mixed) |
| Crypto placeholder | Static row "Connect exchanges — US-0007" (`included_in_total=false`) |
| Optional chart | ECharts stacked bar by account type OR wealth-over-time line from `/wealth/history` |
| Grafana link | External link to `{GRAFANA_URL}/d/portfolio` |
| Stale badge | When `last_successful_sync_at` lags |

**Data:** TanStack Query → wealth API.

#### 9. React `/alerts` inbox + header bell (**DEC-0030**, **R-0023**)

| UI element | Implementation |
|------------|----------------|
| Header bell | `Bell` icon + unread badge; shadcn `Popover` preview (latest 5 alerts) |
| `/alerts` page | Table: type icon, severity badge, title, message, `triggered_at`; Acknowledge + Dismiss buttons |
| Subscription cross-link | Optional "View subscription alerts (N)" in popover when subscription unread > 0 |
| Empty state | "No active alerts" with last sync timestamp |
| Polling | TanStack Query on unread-count; refresh after sync completes |

**Unread:** `status=active AND acknowledged_at IS NULL`. Dismissed excluded from bell.

#### 10. Grafana Dashboard 4 (Portfolio partial) (**DEC-0030**, **R-0026**)

Extend US-0002/0003/0004 file provisioning (**DEC-0012** pattern).

| Dashboard | uid | Folder | Panels |
|-----------|-----|--------|--------|
| Portfolio | `portfolio` | Analytics | Total wealth stat; account count; mixed-currency warning; account breakdown table; wealth-over-time from snapshots; crypto placeholder text; optional pie by account_role |

File: `grafana/provisioning/dashboards/analytics/portfolio.json`, `"id": null`.

**Wealth-over-time SQL:**

```sql
SELECT snapshot_date::timestamptz AS time, total_eur AS value
FROM net_worth_snapshots
WHERE $__timeFilter(snapshot_date::timestamptz)
ORDER BY 1;
```

Platform Health, Dashboards 1–3, 5 unchanged except Dashboard 1 threshold variable (DEC-0029).

### Backend module layout

| Module | Responsibility |
|--------|----------------|
| `wealth::{types,repository,service}` | Net worth breakdown + snapshots |
| `alerts::{types,evaluate,repository,service}` | Alert Engine |
| `api::wealth`, `api::alerts` | Axum handlers |
| `sync` | Extend with `"alerts"` phase calling `AlertService::run_post_sync` |
| `config` | `[alerts]` TOML section + startup DB mirror |

`AppState` gains `wealth: WealthService`, `alerts: AlertService`.

### Config additions (TOML)

```toml
[alerts]
scarcity_threshold_eur = 200.0
budget_drift_pct = 20.0
reporting_currency = "EUR"   # MVP headline; per-account native currency in breakdown

[wealth]
snapshot_retention_days = 365   # optional; prune older snapshots on upsert job
```

### Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Mixed-currency headline misleading | `mixed_currency` flag + mandatory UI banner | R-0021, DEC-0025 |
| Budget drift MTD proration skew | Document one-time delta limitation; use category-targeted only | R-0022, DEC-0026 |
| Plan viability on stale baseline | Bind computation IDs in alert `context`; `stale` metadata | R-0022, R-0019 |
| Mutex duration growth | Monitor combined pipeline; defer queue if > ~30s | R-0024, DEC-0028 |
| US-0003 alert boundary confusion | Cross-link only; document separate surfaces | R-0023, DEC-0030 |
| DEC-0012 threshold drift | Dashboard 1 `$scarcity_threshold` from `alert_config` | R-0025, DEC-0029 |
| Snapshot gaps on failed sync | Flat line acceptable; show last snapshot date in UI | R-0021 |
| Acknowledged-but-active UX | Copy: "Acknowledged — condition still active" | R-0023 |

### Decisions (US-0005)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0025 | Net worth aggregation | Asset sum; mixed-currency warning; daily snapshots; crypto excluded |
| DEC-0026 | Alert evaluation rules | Household scarcity; category-targeted budget drift; plan month-end viability |
| DEC-0027 | Alert persistence | Migration 005; fingerprint dedup; lifecycle states |
| DEC-0028 | Sync alerts phase | Inline `"alerts"` phase after forecast+plan hook |
| DEC-0029 | Threshold centralization | TOML → `alert_config` mirror; Grafana `$scarcity_threshold` (supersedes DEC-0012 hardcode) |
| DEC-0030 | Unified inbox UI | `/wealth`, `/alerts`, header bell; subscription alerts unchanged |

Full records: `decisions/DEC-0025.md` … `decisions/DEC-0030.md`

### Out of scope (US-0005)

- Full crypto PnL and exchange connectors (US-0007)
- Grafana Alertmanager rule provisioning
- AI `get_budget_status` / `get_portfolio` tool implementation detail (US-0006)
- Subscription alert migration to unified inbox
- Multi-currency FX conversion
- Redis/async alert evaluation queue
- Permanent per-entity alert suppress list
- Any write to Firefly III

### Next phase

`/sprint-plan` — S0005 task decomposition against 6 acceptance criteria.

---

## US-0006 — AI financial assistant with privacy-safe tool layer

**Status:** architecture complete (2026-05-31)  
**Research:** R-0027, R-0028, R-0029, R-0030, R-0031 (extends R-0015, R-0019, R-0021, R-0022, DEC-0004, DEC-0006)  
**Decisions:** DEC-0031, DEC-0032, DEC-0033, DEC-0034, DEC-0035, DEC-0036  
**Spec-pack:** `docs/engineering/spec-pack/US-0006-{design-concept,crs,technical-specification}.md`  
**Depends on:** US-0001 OIDC + Settings shell; US-0002 `ForecastService`; US-0003 `SubscriptionService`; US-0004 `PlanService` + overlay; US-0005 `WealthService` + `AlertService`

### System context

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Browser — header AI Sheet drawer + /chat full page (shared ChatPanel)       │
│            Settings AI & Privacy (read-only TOML) + tool audit table         │
└───────────────────────────────┬──────────────────────────────────────────────┘
                                │ JWT Bearer (fetch + ReadableStream SSE)
                                ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum)                                                       │
│                                                                               │
│  POST /api/v1/chat/stream ──▶ AiOrchestrator (async-openai loop)           │
│         │                         │                                           │
│         │                         ├─▶ AiTool registry (6 tools)             │
│         │                         │       └─▶ *Service only (no DbPool)       │
│         │                         ├─▶ PrivacyLayer::redact_tool_result        │
│         │                         ├─▶ truncate/summarize (8 KB cap)           │
│         │                         └─▶ ai_tool_audit insert (redacted args)  │
│         │                                                                     │
│  GET /api/v1/ai/audit ──▶ Settings audit table                               │
│  GET /api/v1/settings ──▶ includes [ai] + [privacy] read-only display      │
└───────────────────────────────┬──────────────────────────────────────────────┘
                                │ in-process service calls only
                                ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  ForecastService | SubscriptionService | PlanService | WealthService         │
│  AlertService | TransactionsService (new read-only aggregates)               │
└──────────────────────────────────────────────────────────────────────────────┘
                                │
                                ▼ OpenAI HTTPS (server-side only; key from env)
                         OpenAI Chat Completions API
```

### Components

#### 1. AI orchestration (`backend/src/ai/`)

OpenAI Chat Completions tool-calling loop via `async-openai` (**DEC-0031**, **R-0027**).

| Submodule | Responsibility |
|-----------|----------------|
| `orchestrator` | Multi-turn loop: stream → tool calls → execute → append tool messages → repeat |
| `registry` | Static `Vec<Arc<dyn AiTool>>`; build OpenAI `tools` array from JSON schemas |
| `tools::*` | Six tool implementations delegating to services |
| `types` | `ToolContext`, `ToolError`, `ToolInvocationMeta`, SSE event DTOs |
| `provider` | OpenAI client factory; stub trait for US-0008 |

**Trait contract:**

```rust
pub trait AiTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, ctx: &ToolContext, args: serde_json::Value)
        -> Result<serde_json::Value, ToolError>;
}

pub struct ToolContext {
    pub transactions: TransactionsService,
    pub subscriptions: SubscriptionService,
    pub forecast: ForecastService,
    pub plans: PlanService,
    pub wealth: WealthService,
    pub alerts: AlertService,
    pub privacy: PrivacyConfig,
    pub user_subject: String,
    pub session_id: Uuid,
    // NO DbPool — tools call services only (acceptance + DEC-0004)
}
```

**Orchestration loop:**

1. Append user message to in-memory thread (from request body)
2. `client.chat().create_stream(request)` with `tools` + `tool_choice: auto`
3. On `FinishReason::ToolCalls` → validate args (serde + JSON Schema) → execute tools (parallel `tokio::join!` when multiple)
4. `PrivacyLayer::redact_tool_result` → truncate to `max_tool_result_bytes` → append `role: tool` messages
5. Insert `ai_tool_audit` row per invocation (redacted `args_summary`)
6. Repeat until `FinishReason::Stop` or **max_tool_rounds** (default **5**)
7. Forward token deltas + `tool_start`/`tool_end`/`done` SSE events to client

**System prompt (minimal):** use tools for factual data; never invent balances; respect privacy aggregates; cite tools used.

**Alternative considered:** Assistants API with hosted threads — rejected (external thread storage conflicts with self-hosted privacy posture).

#### 2. Privacy layer (`backend/src/ai/privacy.rs`)

Central middleware on **tool output** before OpenAI and audit (**DEC-0032**, **R-0028**).

| Setting | Default | Behavior |
|---------|---------|----------|
| `allow_raw_transactions` | `false` | `get_transactions` returns category/month aggregates only |
| `redact_iban` | `true` | ISO IBAN → `[IBAN_REDACTED]` (MOD-97 where possible) |
| `redact_counterparties` | `true` | payee/description → `Counterparty-{hash8}` (SHA-256 + app pepper) |

```rust
pub struct PrivacyLayer { config: PrivacyConfig, pepper: String }
impl PrivacyLayer {
    pub fn redact_tool_result(&self, tool_name: &str, value: serde_json::Value)
        -> serde_json::Value;
    pub fn summarize_args(&self, args: &serde_json::Value) -> serde_json::Value;
}
```

- Recursive JSON walker on known field names (`iban`, `payee`, `description`, `counterparty`, `destination_name`) + regex on string leaves
- When raw disabled and model needs row detail → structured `{ "error": "raw_transactions_disabled", "hint": "use category aggregates" }` — not empty payload
- Settings: **read-only TOML display** for MVP — runtime toggle deferred (requires reload endpoint + change audit)

**Alternative considered:** Per-tool redaction only — rejected (bypass risk when new tool omits scrubbing).

#### 3. Six-tool registry (**DEC-0035**, **R-0031**)

Every tool calls **in-process `*Service` methods** — never `sqlx::query` from `ai/`, never Firefly HTTP.

| Tool | Service / method | Key args |
|------|------------------|----------|
| `get_transactions` | **NEW** `TransactionsService::aggregates(filter)` | `period`, `category_id?`, `group_by` |
| `get_subscriptions` | `SubscriptionService::list_patterns` + price summary | `status`, `kind`, `include_price_events?` |
| `get_forecast` | `ForecastService` latest computation | `horizon`, `account_id?` |
| `get_budget_status` | `AlertService` + active plan MTD compose | `category_id?` — mirrors R-0022 budget drift inputs |
| `get_portfolio` | `WealthService::compute_breakdown` + optional `history(90)` | `include_history?` |
| `simulate_plan` | `PlanService::project_ephemeral(draft)` | `template?`, `plan_id?`, `version_number?`, `adjustments?` |

**`simulate_plan` resolution order (read-only, no persistence):**

1. `plan_id` (+ optional `version_number`) → load version adjustments → `project_readonly`
2. Else `template` → template defaults as draft adjustments → ephemeral project
3. Else active plan latest version
4. Else `{ "error": "no_plan_context" }`

Shares `plan::overlay` math with persisted recompute — no drift (**R-0031**).

**Payload limits:** max **8 KB** serialized per tool result (`[ai] max_tool_result_bytes`); series downsampled to 30 points + `{ min, max, latest }`; overflow → `{ "truncated": true, "summary": {...} }`.

**Example query → tool mapping (acceptance):**

| Query | Primary tool(s) |
|-------|-----------------|
| Leasing affordability | `simulate_plan` (template=leasing) |
| Subscription price increases | `get_subscriptions` (price events) |
| Budget overrun this month | `get_budget_status` + `get_transactions` |
| Savings if cancel subscription | `simulate_plan` (savings_mode) + `get_subscriptions` |
| Top spending categories | `get_transactions` (aggregates) |

#### 4. TransactionsService (`backend/src/transactions/`)

New read-only service for mirror `transactions` + `categories` (**R-0031**).

```rust
TransactionsService::aggregates(AggregateFilter {
    period_start, period_end,
    category_id: Option<i64>,
    group_by: Category | Month,
}) -> TransactionAggregates
```

- When `allow_raw_transactions=false`: category totals, tx counts, inflow/outflow — no row arrays
- When `true`: capped list (max **20** rows, default last 30 days) — still passes through `PrivacyLayer`
- No dedicated REST endpoint required for MVP — internal to AI tool path; optional `GET /api/v1/transactions/aggregates` stretch for debugging

**Alternative considered:** HTTP self-calls to hypothetical REST — rejected (latency + auth recursion).

#### 5. Chat SSE API (**DEC-0033**, **R-0029**)

**DEC-0006 gate closed:** Bearer JWT on POST SSE sufficient — no BFF/cookie auth.

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/v1/chat/stream` | **Primary** SSE stream for ChatPanel |
| POST | `/api/v1/chat/completions` | Non-streaming fallback (tests/admin) |
| GET | `/api/v1/ai/audit` | Operator audit log (`limit`, `offset`) |

**Request body:**

```json
{
  "messages": [{ "role": "user", "content": "..." }],
  "session_id": "uuid-optional"
}
```

**SSE event types:**

```text
event: token       data: {"delta":"The "}
event: tool_start  data: {"tool":"get_forecast","call_id":"..."}
event: tool_end    data: {"tool":"get_forecast","duration_ms":42,"status":"ok"}
event: done        data: {"message_id":"...","tools_used":[...]}
event: error       data: {"code":"provider_error","message":"..."}
```

**Handler pattern:**

- `require_auth` JWT middleware
- Per-user in-memory token bucket rate limit (`[ai] rate_limit_per_min`, default 20/10min on `sub`)
- `mpsc` channel → `Sse::new(ReceiverStream)` + 15s keep-alive
- `CancellationToken` aborts OpenAI stream on client disconnect
- Headers: `Content-Type: text/event-stream`, `Cache-Control: no-cache`, `X-Accel-Buffering: no`

**Chat history:** ephemeral client-side — React state + optional `sessionStorage`; **no DB thread persistence** in US-0006.

**Frontend transport:** `fetch()` + `ReadableStream` (not `EventSource` — lacks POST + Authorization).

#### 6. Migration `006_ai_audit.sql` (**DEC-0034**, **R-0030**)

```sql
CREATE TABLE ai_tool_audit (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  session_id      UUID NOT NULL,
  user_subject    TEXT NOT NULL,
  tool_name       TEXT NOT NULL CHECK (tool_name IN (
                    'get_transactions','get_subscriptions','get_forecast',
                    'get_budget_status','get_portfolio','simulate_plan')),
  args_summary    JSONB NOT NULL DEFAULT '{}',
  result_status   TEXT NOT NULL CHECK (result_status IN ('ok','error')),
  result_rows     INT,
  duration_ms     INT NOT NULL,
  error_message   TEXT,
  model           TEXT,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ai_tool_audit_created ON ai_tool_audit (created_at DESC);
CREATE INDEX ai_tool_audit_tool ON ai_tool_audit (tool_name, created_at DESC);
CREATE INDEX ai_tool_audit_session ON ai_tool_audit (session_id, created_at DESC);
```

**What NOT to store:** full prompts/responses, unredacted payloads, API keys, raw transaction rows.

**Retention (startup job):** **500 row cap** + **90-day purge** (whichever stricter); configurable `[ai] audit_retention_days`.

**Alternative considered:** JSONL log file only — rejected (Settings UI needs indexed query).

#### 7. React chat UI (**DEC-0036**, discovery UX)

Enable AI nav at `/chat`; header **AI** button opens shadcn **`Sheet`** drawer — both share `ChatPanel`.

| UI element | Implementation |
|------------|----------------|
| Header AI button | Opens `Sheet` side drawer (~400px); same `ChatPanel` as `/chat` |
| `/chat` route | Full-page chat; enable sidebar nav (replace disabled placeholder) |
| `ChatPanel` | Message list, input, streaming token render, abort on unmount |
| Empty state | Suggested prompt chips (Projectplan examples) |
| Privacy badge | Header/drawer: "Privacy: aggregates only" when `allow_raw_transactions=false` |
| Tool transparency | Collapsible "Tools used" under assistant messages (name + timestamp; no raw JSON) |
| Settings AI & Privacy | Read-only `[ai]` + `[privacy]` TOML table; audit log table from `/ai/audit` |

**Suggested prompt chips (empty thread):**

- "Kann ich mir ein Leasing Auto leisten?"
- "Welche Abos sind teurer geworden?"
- "Warum bin ich diesen Monat über Budget?"
- "Wie viel spare ich wenn ich Netflix kündige?"
- "Top Ausgabenkategorien diesen Monat"

**Data:** client-side thread state; SSE via authenticated `fetch`; TanStack Query for settings/audit only.

**Alternative considered:** WebSockets — rejected (SSE sufficient; simpler through proxies).

### Backend module layout

| Module | Responsibility |
|--------|----------------|
| `ai::{orchestrator,registry,privacy,provider,tools,types}` | Orchestration + privacy + six tools |
| `transactions::{types,repository,service}` | Read-only aggregates for `get_transactions` |
| `plan::service` | Add `project_ephemeral`, `project_readonly` |
| `api::chat`, `api::ai_audit` | SSE + audit handlers |
| `config` | `[ai]`, `[privacy]` TOML sections |

`AppState` gains `ai: AiService` (orchestrator + registry), `transactions: TransactionsService`.

OpenAI API key: env `OPENAI_API_KEY` only (`[ai] api_key_env`); never in TOML plaintext, browser, or audit rows.

### Config additions (TOML)

```toml
[ai]
provider = "openai"              # US-0008 extends
model = "gpt-4o-mini"
api_key_env = "OPENAI_API_KEY"
max_tool_rounds = 5
max_completion_tokens = 1024
max_tool_result_bytes = 8192
request_timeout_secs = 60
rate_limit_per_min = 20
audit_retention_days = 90
audit_max_rows = 500

[privacy]
allow_raw_transactions = false
redact_iban = true
redact_counterparties = true
```

Extend `GET /api/v1/settings` response with `ai` and `privacy` sections (secrets excluded).

### Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Privacy bypass via nested JSON | Central walker + field-name allowlist + integration test | R-0028, DEC-0032 |
| Prompt injection / tool arg abuse | Strict JSON Schema + serde validation; registry allowlist | R-0027, DEC-0031 |
| OpenAI key exposure | Env-only; server-side calls; never in audit/settings API | R-0027, R-0030 |
| Ephemeral vs persisted plan drift | Share `plan::overlay` module for `project_ephemeral` | R-0031, DEC-0035 |
| SSE proxy buffering | `X-Accel-Buffering: no` + keep-alive events | R-0029, DEC-0033 |
| Missing transactions API | New `TransactionsService` in US-0006 scope | R-0031 |
| `get_budget_status` drift from Alert Engine | Compose from same R-0022 rules via `AlertService` | R-0031, DEC-0035 |
| US-0008 scope creep | Provider stub trait only; OpenAI path first | R-0027 |
| Context window overflow | 8 KB tool cap + series downsample | R-0031, DEC-0035 |
| Settings read-only confusion | User guide documents TOML edit + restart | R-0028 |

### Decisions (US-0006)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0031 | AI orchestration | `AiTool` trait registry; async-openai loop; max 5 rounds; services-only `ToolContext` |
| DEC-0032 | Privacy layer | Central middleware; Projectplan defaults; aggregates when raw disabled |
| DEC-0033 | Chat SSE API | POST `/api/v1/chat/stream`; Bearer JWT; ephemeral client threads |
| DEC-0034 | Audit persistence | Migration 006 `ai_tool_audit`; 500 cap + 90-day purge; redacted args only |
| DEC-0035 | Tool mapping | Six in-process service tools; `project_ephemeral`; 8 KB result cap |
| DEC-0036 | React chat UX | Header Sheet drawer + `/chat` + Settings AI & Privacy + audit table |

Full records: `decisions/DEC-0031.md` … `decisions/DEC-0036.md`

### Out of scope (US-0006)

- Local/self-hosted AI providers (US-0008) — stub `AiProvider` trait only
- ML-enhanced forecasts (US-0009)
- Grafana AI dashboard
- DB-persisted chat threads
- Runtime-editable privacy toggles in UI
- Inline ECharts in chat responses
- User message pre-redaction (defer)
- Any write to Firefly III or direct AI SQL access

### Next phase

`/sprint-plan` — S0006 task decomposition against 6 acceptance criteria.

---

