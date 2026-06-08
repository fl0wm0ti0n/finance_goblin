# Research

## Entry format

Each research entry uses the R-xxxx ID format with semi-structured fields.

**Required fields**: ID (R-xxxx), Date (YYYY-MM-DD), Topic (short description).
**Optional fields**: Query, Sources, Findings, Linked (US-xxxx/DEC-xxxx refs),
Confidence (high/medium/low, default: medium), Status (current/outdated/superseded,
default: current).

### Auto-increment convention

Assign the next sequential R-xxxx ID by incrementing from the highest existing
entry in this file. Before creating a new entry, read this file to determine the
current highest ID.

### Cross-referencing guidance

Reference research entries by ID in other artifacts using the format "per R-xxxx".
Examples:
- In decisions: "per R-0001, prompt-level isolation is unreliable"
- In architecture: "R-0003 confirms the library supports streaming"
- In handoffs: "see R-0002 for competitor analysis"

Agents, commands, and handoffs should cite entry IDs whenever research informed
a decision or recommendation.

---

## R-0001 — Firefly III REST API integration baseline

**Date:** 2026-05-31  
**Topic:** Firefly III API v1 for read-only Flow Finance AI connector  
**Query:** Firefly III REST API authentication, versioning, endpoints for accounts/transactions  
**Sources:**
- [Firefly III API Overview](https://www.mintlify.com/firefly-iii/firefly-iii/api/overview)
- [How to use the API](https://docs.firefly-iii.org/how-to/firefly-iii/features/api/)
- [API Authentication](https://www.mintlify.com/firefly-iii/firefly-iii/api/authentication)
- [GitHub firefly-iii/firefly-iii](https://github.com/firefly-iii/firefly-iii)

**Findings:**
- All endpoints prefixed `/api/v1/`; JSON REST API covers accounts, transactions, categories, budgets, tags, piggy banks
- Authentication via OAuth2 (Laravel Passport) or **Personal Access Token** (`Authorization: Bearer`) — PAT simplest for self-hosted connector
- Accept header must include `application/json` or `application/vnd.api+json`
- No default rate limits in Firefly itself; implement retry/backoff for large sync jobs
- CORS configurable for browser access; server-side Rust connector preferred for token security
- API is intended to become sole interface even for Firefly UI (work in progress)

**Linked:** US-0001  
**Confidence:** high  
**Status:** current

---

## R-0002 — Firefly III pagination & incremental sync strategy

**Date:** 2026-05-31  
**Topic:** Page-based API traversal and incremental sync patterns for the read-only connector  
**Query:** Firefly III `/api/v1/transactions` pagination, date filters, incremental sync without native cursor  
**Sources:**
- [Firefly III API Overview (pagination)](https://mintlify.wiki/firefly-iii/firefly-iii/api/overview)
- [Search API](https://www.mintlify.com/firefly-iii/firefly-iii/api/search)
- [Search reference — updated_at filters](https://docs.firefly-iii.org/references/firefly-iii/search/)
- [OpenAPI v1 — listTransaction](https://raw.githubusercontent.com/firefly-iii/api-docs/main/dist/firefly-iii-6.4.17-v1.yaml)
- [Pagination bug #7427](https://github.com/firefly-iii/firefly-iii/issues/7427)
- [updated_at reliability #8282](https://github.com/firefly-iii/firefly-iii/issues/8282)

**Findings:**
- No native cursor, `If-Modified-Since`, or `updated_since` query param on list endpoints — incremental sync must be application-defined (extends R-0001)
- `GET /api/v1/transactions` supports `page`, `limit` (default 50/page, max 65536), and optional `start`/`end` date filters (`YYYY-MM-DD`, inclusive)
- Paginated JSON:API responses include `meta.pagination` (`total`, `count`, `per_page`, `current_page`, `total_pages`) and HATEOAS `links.next` for termination detection
- Reference entities (accounts, categories, budgets, tags, piggy banks) are low-volume — full-list sync each run is acceptable; use `limit=500` where supported to reduce round-trips
- **Recommended sync strategy for US-0001:**
  1. **Initial sync:** paginate all transactions with `limit=500`; upsert by Firefly resource `id`
  2. **Incremental sync:** persist `last_successful_sync_at` watermark; fetch transactions with `start=<watermark_date - overlap_days>` (suggest 7-day overlap) to catch backdated/edited rows; dedupe by Firefly `id` in local store
  3. **Optional refinement (US-0002+):** Search API query `updated_at_after:YYYY-MM-DD` for narrower windows — but treat `updated_at` as unreliable (known API bug where updates may not bump timestamp per #8282)
  4. **Pre-flight:** `GET /api/v1/search/transactions?query=...` count endpoint to estimate volume before full pull
- Pagination bug (#7427) affected high page numbers; fixed in patch release — still implement retry with backoff and validate `meta.pagination.total_pages` vs fetched count
- Large histories: prefer date-windowed pulls over single mega-request even at max `limit`; export endpoint (`/api/v1/data/export/transactions`) is CSV-only and not suitable for structured ingest

**Linked:** US-0001  
**Confidence:** high  
**Status:** current

---

## R-0003 — Self-hosted OIDC provider options & React/Axum integration

**Date:** 2026-05-31  
**Topic:** IdP selection for US-0001 UI auth and integration patterns  
**Query:** Authentik vs Keycloak vs Authelia for self-hosted Docker; React SPA + Axum backend OIDC wiring  
**Sources:**
- [Authentik vs Keycloak — selfhosting.sh](https://selfhosting.sh/compare/authentik-vs-keycloak/)
- [Keycloak vs Authentik 2026 — OSSAlt](https://ossalt.com/guides/keycloak-vs-authentik-2026)
- [Authentik vs Authelia vs Keycloak 2026 — Selfhostr](https://selfhostr.com/comparatifs/authentik-vs-authelia-vs-keycloak-2026/)
- [react-oidc-context](https://github.com/authts/react-oidc-context)
- [axum-oidc-client crate](https://crates.io/crates/axum-oidc-client)

**Findings:**
- **Authentik:** modern admin UI, visual flow builder, OIDC/OAuth2/SAML; requires server + worker + PostgreSQL + Redis (~1.5–2 GB RAM); best fit for greenfield self-hosted stacks needing full IdP without enterprise LDAP/SAML complexity
- **Keycloak:** CNCF/Red Hat, mature SAML/LDAP/AD federation, heavier JVM footprint (~2–4 GB); choose when enterprise directory federation or Red Hat support path matters
- **Authelia:** lightweight forward-auth/proxy model; OIDC provider support exists but is narrower — better for reverse-proxy SSO than as primary IdP for a custom React SPA with backend API JWT validation
- **Firefly-adjacent:** Firefly III OAuth is for Firefly API access (connector auth per R-0001), not a substitute for Flow Finance UI login — keep IdP separate
- **Recommended for US-0001 dev/prod:** **Authentik** as optional Compose profile service (`oidc` profile) OR document external IdP URL via env — avoids coupling minimal profile to IdP infra while satisfying acceptance
- **React frontend:** `react-oidc-context` + `oidc-client-ts` — `AuthProvider` with authority/client_id/redirect_uri; `useAuth` hook for session; mandatory `onSigninCallback` to strip URL payload; enable `automaticSilentRenew`
- **Axum backend:** validate IdP JWTs via JWKS (`Authorization: Bearer`) on protected API routes; `axum-oidc-client` crate offers PKCE session middleware if BFF pattern chosen — for SPA-first US-0001, prefer stateless JWT validation middleware + public `/health` endpoint
- **Risks:** IdP adds 2–4 containers when embedded in Compose; callback URL must match IdP app registration exactly; CORS/origin config needed between React dev server and IdP

**Linked:** US-0001  
**Confidence:** high  
**Status:** current

---

## R-0004 — Rust Axum + SQLx + TimescaleDB foundation patterns

**Date:** 2026-05-31  
**Topic:** Backend bootstrap, migrations, and time-series schema prep for platform foundation  
**Query:** Axum + SQLx project structure, migration workflow, TimescaleDB hypertable setup via SQLx migrations  
**Sources:**
- [SQLx migrate! macro docs](https://docs.rs/sqlx/latest/sqlx/macro.migrate.html)
- [HDA — Database Migrations with SQLx](https://hda.daz.is/data/database-migrations/)
- [TimescaleDB create_hypertable()](https://docs.timescale.com/api/latest/hypertable/create_hypertable/)
- [TimescaleDB migration guide](https://oneuptime.com/blog/post/2026-01-27-timescaledb-migration/view)
- [Axum + SQLx production stack](https://devcheolu.com/en/posts/REA8G6eGFYSfWm5Qd9rE)

**Findings:**
- **Project layout:** `PgPoolOptions::new().max_connections(N).connect(DATABASE_URL)` at startup; `AppState { db: PgPool, ... }` shared via Axum `State` — `PgPool` is internally `Arc`, no double-wrap needed
- **Migrations:** `sqlx::migrate!("./migrations").run(&pool).await?` embeds SQL at compile time; run on app startup for US-0001 simplicity; add `build.rs` with `cargo:rerun-if-changed=migrations` for stable rebuilds
- **CI/offline builds:** `cargo sqlx prepare` → commit `.sqlx/` metadata; set `SQLX_OFFLINE=true` in Dockerfile so compile-time `query!` macros work without live DB
- **TimescaleDB in migrations:** first migration should `CREATE EXTENSION IF NOT EXISTS timescaledb;` (requires TimescaleDB-enabled PostgreSQL on external host — operator prerequisite)
- **US-0001 schema scope:** relational tables only — `sync_runs`, `sync_cursors`, entity mirror tables (accounts, transactions, categories, budgets, tags, piggy_banks); defer hypertables to US-0002 forecast storage
- **Hypertable pattern (US-0002 prep):** `CREATE TABLE ... (ts TIMESTAMPTZ NOT NULL, ...)` then `SELECT create_hypertable('table', 'ts', if_not_exists => true);` — time column must be `NOT NULL`; use `chunk_time_interval => INTERVAL '1 day'` for daily forecast granularity
- **Risks:** external PostgreSQL must have TimescaleDB extension installed by operator (not bundled in Compose per acceptance); migration deadlocks possible with TimescaleDB background workers on large hypertable creation — lock `_timescaledb_config.bgw_job` if needed; pool sizing: start with `max_connections = 2 * cores + 1`, tune under load

**Linked:** US-0001, US-0002  
**Confidence:** high  
**Status:** current

---

## R-0005 — Docker Compose multi-service with external PostgreSQL

**Date:** 2026-05-31  
**Topic:** Compose profiles, external DB connectivity, and startup ordering for US-0001 stack  
**Query:** Docker Compose patterns when PostgreSQL is external; healthchecks; host gateway for dev  
**Sources:**
- [Docker Compose startup order](https://docs.docker.com/compose/how-tos/startup-order/)
- [depends_on with healthchecks](https://oneuptime.com/blog/post/2026-01-16-docker-compose-depends-on-healthcheck/view)
- [extra_hosts host.docker.internal](https://oneuptime.com/blog/post/2026-02-08-how-to-use-docker-compose-extrahosts-configuration/view)
- [host.docker.internal on Linux](https://stackoverflow.com/questions/70725881/what-is-the-equivalent-of-add-host-host-docker-internalhost-gateway-in-a-comp)

**Findings:**
- **Acceptance constraint:** PostgreSQL never embedded in Compose — `database.mode = "external"` in TOML with host/port/user/password from env; Firefly III also needs its own external DB (separate database name)
- **External DB connectivity:** `depends_on: condition: service_healthy` only works for in-compose services — app must implement connection retry/backoff loop at startup for external PostgreSQL (exponential backoff, max ~60s)
- **Dev on Linux:** add `extra_hosts: ["host.docker.internal:host-gateway"]` to services needing host-reachable DB; Docker Desktop Mac/Windows provides this automatically
- **Minimal profile (US-0001):** `flow-finance-ai` (backend+frontend), `firefly-iii`, `grafana` — no postgres container; standard profile adds `redis`; optional `oidc` profile adds Authentik stack (per R-0003)
- **In-compose healthchecks:** define `/health` HTTP check on `flow-finance-ai`; Firefly readiness via HTTP to `/` or API ping; Grafana via `/api/health` — use `depends_on: condition: service_healthy` between in-compose dependents only
- **Config pattern:** single `.env` / TOML with `DATABASE_HOST`, `DATABASE_PORT`, `FIREFLY_DB_*`, `FIREFLY_APP_KEY`, `OIDC_ISSUER_URL`; never commit secrets
- **Grafana provisioning:** mount `provisioning/datasources` and optional `provisioning/dashboards` as volumes; datasource points to external TimescaleDB/PostgreSQL via env substitution
- **Risks:** operator must configure `pg_hba.conf`/`listen_addresses` on external PostgreSQL to allow Docker bridge subnet; Firefly III requires pre-created database + app key; no Compose-level guarantee external DB is TimescaleDB-enabled

**Linked:** US-0001  
**Confidence:** high  
**Status:** current

---

## R-0006 — Rule-based personal finance forecast algorithms (MVP baseline)

**Date:** 2026-05-31  
**Topic:** Deterministic cashflow forecasting without ML for US-0002 Forecast Engine  
**Query:** Personal finance cashflow projection algorithms — recurring detection heuristics, rolling averages, day-by-day balance forecast, monthly income/cost decomposition  
**Sources:**
- [cashflow-app — layered forecast architecture](https://github.com/NikolasMarkou/cashflow-app)
- [Vivid Account Insights — rule-based recurring projections](https://github.com/PeterCassell92/Vivid-Account-Insights)
- [subscription-leak-radar — cadence inference heuristics](https://github.com/ZhenyuanPAN822/subscription-leak-radar)
- [refund-radar — recurring charge detection rules](https://github.com/andreolf/refund-radar)
- [Glean — cash flow forecasting best practices 2025](https://www.glean.com/perspectives/cash-flow-management-forecast)

**Findings:**
- **MVP scope (per US-0002 / US-0009 boundary):** rule-based projection only — no ML, SARIMA, or seasonal models; subscription engine (US-0003) not required but lightweight recurrence heuristics improve accuracy over pure rolling averages
- **Recommended layered model (simplest viable):**
  1. **Starting balance:** current synced Firefly asset-account balance (from mirror tables per US-0001)
  2. **Deterministic layer:** infer likely recurring inflows/outflows from transaction history using payee/description grouping, interval stability (weekly/monthly/quarterly), and amount tolerance (±5% or fixed € threshold); project forward by repeating detected cadence — *not* full US-0003 confirm/reject UX, but same heuristics at lower confidence
  3. **Variable residual layer:** for non-recurring spend/income, use **3-month rolling average** daily or monthly rate (configurable window); cap outliers at 95th percentile to reduce one-off spikes
  4. **Category decomposition (monthly view):** map Firefly categories/tags to **income / fixed / variable** buckets via config defaults + operator overrides; monthly forecast = sum projected recurring by bucket + rolling residual by bucket
  5. **Day-by-day balance path:** for each future day *d*, `balance(d) = balance(d-1) + scheduled_recurring(d) + variable_daily_avg`; aggregate to week-end and month-end milestones for daily AC
- **Horizon handling:**
  - **Daily:** project 1–45 days forward; expose tomorrow, +7 days, last day of current month as named milestones
  - **Monthly:** current month + next 24 months of income/fixed/variable/free cashflow
  - **Long-term:** reuse same daily path; sample end-of-month balance at 3/6/12/24 month offsets
- **Sparse history fallback:** if <90 days of transactions for an account, widen rolling window to all available data and flag `low_confidence` in API metadata; do not fail recompute
- **Alternatives considered:**
  - *Pure rolling average only* — simpler but misses salary/rent cadence; unacceptable accuracy for primary use case
  - *Full subscription engine first* — better long-term but violates story split; defer confirm/reject to US-0003
  - *On-demand compute without persistence* — fails acceptance ("persisted in TimescaleDB hypertables")
- **Recompute trigger:** full recompute after successful Firefly sync (per backlog); store as versioned snapshot keyed by `computation_id` + `computed_at` (see R-0007)
- **Risks:** false-positive recurring detection inflates certainty; sparse/new accounts produce flat projections; category→bucket mapping requires sensible defaults; transfer between own accounts must be netted (exclude internal transfers like R-0002 transfer-netting pattern)

**Linked:** US-0002, US-0003, US-0009  
**Confidence:** high  
**Status:** current

---

## R-0007 — TimescaleDB hypertable schema for forecast snapshots

**Date:** 2026-05-31  
**Topic:** Forecast time-series storage design extending R-0004 foundation patterns  
**Query:** TimescaleDB hypertable schema for precomputed forecast series; chunk interval; indexes; latest-snapshot query pattern for API and Grafana  
**Sources:**
- [TimescaleDB — create hypertable](https://docs.timescale.com/use-timescale/latest/hypertables/create/)
- [TimescaleDB — sizing hypertable chunks](https://docs.timescale.com/use-timescale/latest/hypertables/about-hypertables/)
- [Timescale pg-aiguide — hypertable setup skill](https://github.com/timescale/pg-aiguide/blob/main/skills/setup-timescaledb-hypertables/SKILL.md)
- [OneUptime — hypertable design guide 2026](https://oneuptime.com/blog/post/2026-01-26-timescaledb-hypertables/view)
- [R-0004](docs/engineering/research.md#r-0004--rust-axum--sqlx--timescaledb-foundation-patterns) (US-0001 migration baseline)

**Findings:**
- **Storage strategy:** **precompute and persist** full forecast series on sync completion — not on-demand with cache (acceptance requires hypertable persistence; sync-triggered recompute per backlog)
- **Recommended schema (extends R-0004 relational mirrors):**
  ```sql
  -- Run metadata (relational, not hypertable)
  CREATE TABLE forecast_computations (
    id              UUID PRIMARY KEY,
    sync_run_id     BIGINT REFERENCES sync_runs(id),
    computed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    status          TEXT NOT NULL,  -- running | success | failed
    error_message   TEXT
  );

  -- Daily balance projection points (hypertable)
  CREATE TABLE forecast_balance_daily (
    ts              TIMESTAMPTZ NOT NULL,  -- projected calendar date
    account_id      BIGINT NOT NULL REFERENCES accounts(id),
    computation_id  UUID NOT NULL REFERENCES forecast_computations(id),
    balance         NUMERIC(18,2) NOT NULL
  ) WITH (timescaledb.hypertable, timescaledb.partition_column = 'ts', timescaledb.chunk_interval = '7 days');

  -- Monthly cashflow decomposition (hypertable)
  CREATE TABLE forecast_cashflow_monthly (
    ts              TIMESTAMPTZ NOT NULL,  -- first day of projected month
    account_id      BIGINT NOT NULL REFERENCES accounts(id),
    computation_id  UUID NOT NULL REFERENCES forecast_computations(id),
    income          NUMERIC(18,2) NOT NULL DEFAULT 0,
    fixed_costs     NUMERIC(18,2) NOT NULL DEFAULT 0,
    variable_costs  NUMERIC(18,2) NOT NULL DEFAULT 0,
    free_cashflow   NUMERIC(18,2) NOT NULL DEFAULT 0
  ) WITH (timescaledb.hypertable, timescaledb.partition_column = 'ts', timescaledb.chunk_interval = '30 days');
  ```
- **Chunk interval:** personal-finance volume is ≪1M rows/day — use **7-day** chunks for daily balance, **30-day** for monthly (Timescale default 7-day also acceptable for both; avoid 1-day chunks — unnecessary chunk explosion per Timescale sizing guidance)
- **Indexes (add after hypertable creation):**
  - `CREATE INDEX ON forecast_balance_daily (account_id, computation_id, ts DESC);`
  - `CREATE INDEX ON forecast_cashflow_monthly (account_id, computation_id, ts DESC);`
  - `CREATE INDEX ON forecast_computations (computed_at DESC);`
- **Latest snapshot access:** API and Grafana query `WHERE computation_id = (SELECT id FROM forecast_computations WHERE status='success' ORDER BY computed_at DESC LIMIT 1)` — architecture may add a `latest_forecast_computation` view or `is_current` flag; keep MVP query simple
- **Retention:** keep last **5** successful computations (delete older via app job or `forecast_computation_retention` config); hypertable rows cascade-delete with computation metadata — prevents unbounded growth
- **Migration approach:** SQLx migration in US-0002; `CREATE EXTENSION IF NOT EXISTS timescaledb` already expected from US-0001 (R-0004); use `create_hypertable(..., if_not_exists => true)` if converting existing tables
- **Compression / continuous aggregates:** defer — row volume too low for MVP benefit; revisit if history retention widens (US-0009 ML overlays)
- **Alternatives considered:**
  - *Single unified hypertable with metric_type column* — fewer tables but awkward mixed granularities; rejected for clarity
  - *On-demand compute, hypertable stores only query cache* — rejected (acceptance + Grafana needs stable series)
  - *Store in relational tables without hypertables* — rejected (acceptance explicitly requires hypertables)
- **Risks:** external TimescaleDB prerequisite still operator-owned (R-0004/R-0005); recompute latency grows with account count × horizon days; concurrent sync + recompute needs mutex (extend US-0001 sync mutex pattern); failed computation must not serve stale data — API returns last successful snapshot with `stale=true` metadata

**Linked:** US-0002, R-0004, R-0005  
**Confidence:** high  
**Status:** current

---

## R-0008 — Grafana dashboard-as-code for analytics dashboards

**Date:** 2026-05-31  
**Topic:** Provisioning Dashboard 1 (Cashflow) and Dashboard 5 (Forecast horizons) alongside US-0001 Platform Health pattern  
**Query:** Grafana file provisioning with Docker Compose; PostgreSQL/TimescaleDB panel queries; stable UIDs; folder layout; scarcity threshold reference lines  
**Sources:**
- [Grafana — provisioning dashboards](https://grafana.com/docs/grafana/latest/administration/provisioning/#dashboards)
- [Grafana — PostgreSQL query editor / TimescaleDB macros](https://grafana.com/docs/grafana/latest/datasources/postgres/query-editor/)
- [TimescaleDB + Grafana time_bucket tutorial](https://www.bookstack.cn/read/TimescaleDB-2.0-en/eb48cfaa0771b19c.md)
- [Sven van Ginkel — dashboard provisioning as code](https://svenvg.com/posts/grafana-observability-dashboard-provisioning-as-code/)
- Existing US-0001 artifacts: `grafana/provisioning/datasources/postgres.yaml`, `grafana/provisioning/dashboards/platform-health.json` (R-0005)

**Findings:**
- **Extend existing pattern (R-0005):** mount `./grafana/provisioning:/etc/grafana/provisioning:ro` — add JSON files beside `platform-health.json`; provider YAML already scans `/etc/grafana/provisioning/dashboards` (`dashboards.yaml` / `dashboard.yaml`)
- **Folder organization:** add second provider or use `foldersFromFilesStructure: true` with subdirectories:
  - `grafana/provisioning/dashboards/platform/platform-health.json` (existing — optional move)
  - `grafana/provisioning/dashboards/analytics/cashflow.json` — Dashboard 1
  - `grafana/provisioning/dashboards/analytics/forecast-horizons.json` — Dashboard 5
  - Simpler MVP: keep flat directory, set `"folder": "Analytics"` in a dedicated provider block
- **Stable identifiers (required for reproducible provisioning):**
  - Datasource: add explicit `uid: FlowFinancePostgreSQL` to `postgres.yaml` (panels in `platform-health.json` already reference this uid)
  - Dashboard 1: `uid: cashflow`, title `Cashflow`
  - Dashboard 5: `uid: forecast-horizons`, title `Forecast Horizons`
  - Set `"id": null` in JSON; never commit numeric `"id"`
- **Panel query patterns (PostgreSQL datasource, `timescaledb: true` already set):**
  ```sql
  -- Historical + forecast balance overlay (Dashboard 1, time series)
  SELECT ts AS "time", balance AS value
  FROM forecast_balance_daily
  WHERE computation_id = (SELECT id FROM forecast_computations WHERE status='success' ORDER BY computed_at DESC LIMIT 1)
    AND account_id = $account_id
    AND $__timeFilter(ts)
  ORDER BY 1;
  ```
  - Use `$__timeFilter(ts)` macro; with TimescaleDB enabled, `$__timeGroup(ts, $__interval)` expands to `time_bucket()` automatically
  - **Scarcity threshold (Dashboard 1):** use panel **Thresholds** or **Standard options → Thresholds** with constant line at configurable value — read from Grafana **dashboard variable** backed by a small config query or static custom variable `${SCARCITY_THRESHOLD}` injected via provisioning env *only in provider YAML*, not inside dashboard JSON (Grafana docs: env vars in dashboard JSON not supported); MVP: hardcode default `200` in panel threshold with comment to wire US-0005 Alert Engine later
- **Dashboard 5 horizon panels:** one time-series panel per horizon (1/3/6/12 months) plus optional 24-month panel to match React selector; each panel filters `forecast_balance_daily` from `now()` to `now() + INTERVAL 'N months'`; monthly decomposition panel queries `forecast_cashflow_monthly`
- **Template variables:** `$account_id` — `SELECT id AS __value, name AS __text FROM accounts WHERE account_type = 'asset' ORDER BY name`
- **Workflow:** prototype panels in Grafana UI → Export → remove `id` → commit JSON → set `allowUiUpdates: false` for GitOps-only or `true` for dev iteration (provisioned file wins on restart per Grafana docs)
- **Alternatives considered:**
  - *Grafana HTTP API / Terraform* — heavier ops; rejected for self-hosted Compose MVP
  - *Manual UI-only dashboards* — not reproducible; rejected
  - *Prometheus metrics for forecast data* — duplicate storage; rejected (data already in TimescaleDB)
- **Risks:** datasource uid mismatch breaks all panels if not set explicitly; bind-mount filesystem may not emit inotify events — keep `updateIntervalSeconds: 30` (already configured); dashboard JSON schemaVersion must match Grafana 11.0.0 image; scarcity threshold duplication until US-0005 centralizes config

**Linked:** US-0002, R-0004, R-0005  
**Confidence:** high  
**Status:** current

---

## R-0009 — Subscription detection engine patterns & confidence scoring

**Date:** 2026-05-31  
**Topic:** Recurring-pattern detection for US-0003 Subscription Engine extending US-0002 forecast heuristics  
**Query:** Personal finance subscription detection algorithms — payee grouping, cadence inference, amount tolerance, confidence tiers (95/80/60%), shared module vs fork of `detect_patterns`  
**Sources:**
- [R-0006](docs/engineering/research.md#r-0006--rule-based-personal-finance-forecast-algorithms-mvp-baseline) (US-0002 recurring baseline)
- Existing implementation: `backend/src/forecast/recurring.rs` (`detect_patterns`, ±5% tolerance, min 3 txs, weekly/monthly/quarterly cadence)
- [GiGurra/subscription-detector](https://github.com/GiGurra/subscription-detector) — payee grouping, configurable amount tolerance (default 35%), monthly pattern check
- [ZhenyuanPAN822/subscription-leak-radar](https://github.com/ZhenyuanPAN822/subscription-leak-radar) — cadence inference (weekly/monthly/quarterly/annual), price-change flags
- [BBVA AI Factory — recurring pattern analysis](https://www.bbvaaifactory.com/financial-habits-analysis/) — delta-t intervals, median periodicity, cadence tolerance margins
- [Tapix — subscription detection signals](https://www.tapix.io/resources/post/detection-solution-for-subscriptions) — lifecycle signals (start, renew, price change, gaps)
- [Spade recurring transaction guide](https://docs.spade.com/reference/recurring-transaction-guide) — min 3 months history, merchant identity normalization

**Findings:**
- **Reuse vs fork (discovery open question):** **Extract shared recurrence core** — move grouping, interval median, cadence stability, and amount tolerance from `forecast/recurring.rs` into a shared module (e.g. `backend/src/recurrence/`). Forecast keeps lightweight `detect_patterns` wrapper; Subscription Engine adds confidence scoring, persistence, user confirm/reject, and Dauerauftrag classification (R-0010). Avoid duplicating algorithm logic; avoid ML/DBSCAN for MVP (BBVA/Ntropy approaches overkill for single-household volume).
- **Detection pipeline (extends R-0006 layer 2):**
  1. **Input:** expense transactions from mirror `transactions` (amount < 0); exclude internal transfers (existing forecast netting)
  2. **Group key:** normalized payee/description — lowercase trim, collapse whitespace, strip trailing reference tokens (regex: trailing alphanum codes like `P3E460`, card last-4 suffixes)
  3. **Minimum evidence:** ≥3 occurrences spanning ≥2 billing cycles (Spade recommends ≥3 months for monthly)
  4. **Cadence:** median inter-transaction days; classify weekly (6–8), biweekly (13–16), monthly (27–32), quarterly (85–95), annual (350–380); reject if >50% intervals outside cadence tolerance (existing `is_stable_cadence` windows)
  5. **Amount stability:** median amount; all occurrences within tolerance band (see confidence tiers below)
  6. **Lifecycle:** mark ACTIVE if last charge within `interval + grace_days` (default grace 5 for monthly per subscription-detector); INACTIVE candidate if gap exceeds 2× interval
- **Confidence tiers (maps to backlog 95/80/60%):**
  | Tier | Label | Criteria |
  |------|-------|----------|
  | **95** | High | ≥4 occurrences; all intervals within cadence tolerance; all amounts within ±5% of median; same group key |
  | **80** | Medium | ≥3 occurrences; all intervals within cadence tolerance; all amounts within ±10% of median |
  | **60** | Low | ≥3 occurrences; cadence stable on median but ≤1 interval outlier OR amounts within ±15% of median |
  - Patterns below 60% threshold: **do not emit** as candidates (reduces false-positive queue noise)
- **Candidate dedup:** fingerprint = `hash(normalized_key + interval_days + round(median_amount, 2))`; skip if matching confirmed subscription exists; skip if fingerprint in `subscription_rejections` (R-0012)
- **Post-detection states:** `pending` (new/changed candidate) → user `confirmed` | `rejected`; only `confirmed` patterns propagate to forecast and price-change alerts
- **Alternatives considered:**
  - *Fork separate engine with copy-paste heuristics* — rejected (drift from forecast layer per DEC-0007)
  - *Single polymorphic entity without candidate queue* — rejected (confirm/reject UX requires pending state distinct from confirmed)
  - *ML recurrence model (Ntropy)* — rejected (US-0009 scope; needs labeled training data)
- **Risks:** descriptor normalization insufficient for bank-export-style payee drift; annual subscriptions need ≥2 years data for reliable cadence; false positives at 60% tier require strong confirm/reject UX before alerts propagate

**Linked:** US-0003, R-0006, DEC-0007  
**Confidence:** high  
**Status:** current

### BUG-0008 addendum (2026-06-08)

Discovery confirmed under-detection root cause: payee-only grouping (`by_payee` / `extract_payee_source`) fragments SEPA bank-memo strings; `RecurrenceGroup.category_ids` collected but unused for grouping key. Live probe: **12** patterns from **922+** txs (6 pending). Recall levers and phased bundle in **[R-0069 §2](docs/engineering/research.md#r-0069--bug-0008-detection-recall-levers-ai-path-boundary)**; alert dedup must land before threshold tuning per R-0068 §6. Honor [R-0065 § BUG-0008 coordinate](docs/engineering/research.md#bug-0008-coordination-do-not-merge-scope).

---

## R-0010 — Dauerauftrag (standing order) vs subscription classification

**Date:** 2026-05-31  
**Topic:** Heuristics to separate fixed standing orders from discretionary subscriptions for US-0003  
**Query:** Standing order vs direct debit vs subscription detection; Dauerauftrag classification rules; whitelist vs rule-based  
**Sources:**
- [GoCardless — standing orders guide](https://gocardless.com/en-us/guides/posts/guide-to-standing-orders/) — fixed amount, fixed interval, payer-controlled push payment
- [Which? — direct debit vs standing order](https://www.which.co.uk/money/banking/banking-security-and-payment-methods/direct-debits-and-standing-orders-explained-aU1tE5d00CI5) — standing order = fixed regular payment set by payer; subscriptions often card CPA with variable amounts
- [ArXiv 2305.18430 — weakly supervised transaction classification](https://arxiv.org/pdf/2305.18430) — frequency/amount labeling functions for recurring vs non-recurring
- US-0003 discovery: separate tab for Dauerauftrag; Firefly category vocabulary from mirror tables

**Findings:**
- **Conceptual distinction:**
  - **Standing order (Dauerauftrag):** payer-initiated, **fixed amount** (rent, insurance premium, loan payment), high regularity, often large outflows
  - **Subscription:** merchant-initiated pull (direct debit/card-on-file), **semi-variable amount** (usage tiers, tax, promotional pricing), descriptor drift common
- **Recommended approach: rule-based primary + optional config override** (not whitelist-only):
  1. **Exact-amount rule:** coefficient of variation (stddev/|mean|) across occurrence amounts < **0.02** (2%) AND all amounts identical to €0.01 → classify `standing_order`
  2. **Near-exact rule:** all amounts within ±**1%** (stricter than subscription ±5%) AND interval monthly/quarterly → classify `standing_order`
  3. **Category boost:** if ≥50% of linked Firefly transactions carry category names matching config list (`rent`, `miete`, `insurance`, `versicherung`, `utilities`, `nebenkosten`, `loan`, `darlehen`) → classify `standing_order` even at ±2% variance
  4. **Large fixed outflow:** median amount ≤ −€200 AND amount CV < 0.01 → `standing_order`
  5. **Default:** patterns passing subscription detection but failing standing-order rules → `subscription`
  6. **Optional config whitelist:** TOML `standing_order_payee_patterns: ["landlord", "versicherung"]` — operator override; merges with rules (not sole signal)
- **Storage:** single `subscription_patterns` entity with `kind` enum (`subscription` | `standing_order`); user may override kind on confirm (dropdown in confirm dialog)
- **UI:** filter/tab by `kind`; standing orders excluded from "subscription spend" stat on main page but included in total fixed outflows
- **Alternatives considered:**
  - *Payee whitelist only* — rejected (operator burden; misses unlisted rent/insurance payees)
  - *Separate detection engine* — rejected (same recurrence core; classification is post-processing step)
  - *Firefly tags as sole signal* — rejected (not all operators tag consistently)
- **Risks:** insurance premiums with annual adjustment look like subscriptions; rent with occasional Nebenkosten adjustments may flip classification; user override needed for edge cases

**Linked:** US-0003, R-0009  
**Confidence:** high  
**Status:** current

### BUG-0008 addendum (2026-06-08)

Standing-order classification is not the primary recall blocker (live: 3 rejected including Strom standing_order). Recall gains come from payee normalization + optional category-aware grouping (R-0069 §2.2–2.3). Classification thresholds unchanged until architecture DEC; coordinate table still applies.

---

## R-0011 — Subscription price-change detection & alert thresholds

**Date:** 2026-05-31  
**Topic:** Amount and cadence change detection for confirmed subscriptions; noise-reduction thresholds  
**Query:** Price increase/decrease detection recurring payments; minimum delta thresholds; interval/frequency shift detection; in-app alert delivery  
**Sources:**
- [Yodlee — Subscription Changed insight](https://developer.yodlee.com/resources/yodlee/insights-details/docs/subscription-changed) — dual threshold: CHANGE AMOUNT (€) AND CHANGE PERCENT; user-configurable
- [subscription-leak-radar](https://github.com/ZhenyuanPAN822/subscription-leak-radar) — flags when latest charge meaningfully higher than first observed
- [techinterview — price history LLD](https://www.techinterview.org/post/3233468711/lld-price-history/) — append-only history, skip insert if change below materiality threshold (~95% noise reduction)
- US-0003 discovery: subscription-scoped alerts (not US-0005 inbox); ECharts price history in detail drawer

**Findings:**
- **Scope:** price-change detection runs **only on confirmed** subscriptions (not pending candidates) after each sync detection pass
- **Amount change detection:**
  1. Compare latest linked transaction amount to previous confirmed `current_amount` (or prior occurrence in same pattern)
  2. Fire event when **both** conditions met (Yodlee dual-threshold pattern, adapted):
     - `|delta| ≥ €1.00` (CHANGE AMOUNT minimum), AND
     - `|delta| / |previous| × 100 ≥ 5%` (CHANGE PERCENT minimum)
  3. Defaults configurable via TOML: `subscription_price_change_min_eur: 1.0`, `subscription_price_change_min_pct: 5.0`
  4. Direction: `increase` | `decrease`; store `previous_amount`, `new_amount`, `delta_pct`, `detected_at`, `sync_run_id`
  5. **Skip noise:** round amounts to 2 decimals; ignore changes where both thresholds fail (e.g. €9.99 → €10.01 = 0.2% — no event)
- **Frequency/interval change:** recompute median interval on confirmed pattern after new transaction; fire `interval_change` event when new median differs from stored `interval_days` by > cadence tolerance (e.g. monthly 28→31 days = no event; monthly→weekly = event)
- **Price history source:** append row to `subscription_price_events` on each confirmed billing occurrence (not only on change) — powers ECharts detail drawer and Grafana panels
- **Alert delivery (discovery open question):**
  - **MVP:** in-app **banner on `/subscriptions`** when unread alerts exist + **toast** on new detection/price change after sync; persist in `subscription_alerts` table with `read_at` nullable
  - **Defer:** global header notification bell with unread count — optional stretch; US-0005 owns unified alert inbox
  - Alert types: `new_detection` (pending candidate created), `price_change`, `interval_change`
- **Alternatives considered:**
  - *Any amount delta triggers alert* — rejected (cent-level rounding noise)
  - *Percent-only threshold* — rejected (€0.50 increase on €5 sub = 10% but immaterial; dual threshold handles both)
  - *US-0005 Alert Engine for MVP* — rejected (scope); subscription alerts are page-scoped per backlog
- **Risks:** promotional/discounted cycles cause false increase alerts when promo ends; annual billing with single observation cannot detect change until second year; currency rounding on foreign subscriptions

### BUG-0008 addendum (2026-06-08)

Code audit: `process_confirmed` also calls bare `insert_alert` every sync pass — **same dedup gap as `new_detection`** (price_change alerts accumulate). Apply shared fingerprint dedup per **[R-0068 §1](docs/engineering/research.md#r-0068--bug-0008-subscription-alert-dedup-unread-count-contract-orphan-lifecycle)**. MVP page-scoped banner contract unchanged; header bell remains US-0005-only (R-0068 §3).

**Linked:** US-0003, R-0009, R-0012, BUG-0008, R-0068  
**Confidence:** high  
**Status:** current

---

## R-0012 — Subscription persistence schema (candidates, confirmed, rejections, events)

**Date:** 2026-05-31  
**Topic:** PostgreSQL schema for subscription detection state, price history, and alerts  
**Query:** `subscription_candidates` + `subscriptions` + `subscription_price_events` vs single polymorphic entity; rejection fingerprint persistence  
**Sources:**
- [R-0004](docs/engineering/research.md#r-0004--rust-axum--sqlx--timescaledb-foundation-patterns) (SQLx migration pattern)
- [R-0007](docs/engineering/research.md#r-0007--timescaledb-hypertable-schema-for-forecast-snapshots) (relational metadata + optional hypertables)
- [Price history LLD](https://www.techinterview.org/post/3233468711/lld-price-history/) — append-only events + current-state column
- Existing: `backend/migrations/001_initial.sql` mirror tables, `002_forecast_hypertables.sql`

**Findings:**
- **Recommended model: single lifecycle table + event/alert satellites** (not separate `candidates` + `subscriptions` tables):
  ```sql
  -- Migration 003: subscription intelligence
  CREATE TYPE subscription_status AS ENUM ('pending', 'confirmed', 'rejected', 'inactive');
  CREATE TYPE subscription_kind AS ENUM ('subscription', 'standing_order');
  CREATE TYPE subscription_alert_type AS ENUM ('new_detection', 'price_change', 'interval_change');

  CREATE TABLE subscription_patterns (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fingerprint       TEXT NOT NULL UNIQUE,
    status            subscription_status NOT NULL DEFAULT 'pending',
    kind              subscription_kind NOT NULL DEFAULT 'subscription',
    payee_key         TEXT NOT NULL,
    display_name      TEXT NOT NULL,
    interval_days     INT NOT NULL,
    current_amount    NUMERIC(18,2) NOT NULL,
    confidence_pct    SMALLINT NOT NULL CHECK (confidence_pct IN (60, 80, 95)),
    first_seen_at     DATE NOT NULL,
    last_seen_at      DATE NOT NULL,
    confirmed_at      TIMESTAMPTZ,
    rejected_at       TIMESTAMPTZ,
    detection_run_id  UUID,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
  );

  CREATE TABLE subscription_pattern_transactions (
    pattern_id        UUID NOT NULL REFERENCES subscription_patterns(id) ON DELETE CASCADE,
    transaction_id    BIGINT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    PRIMARY KEY (pattern_id, transaction_id)
  );

  CREATE TABLE subscription_price_events (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_id        UUID NOT NULL REFERENCES subscription_patterns(id) ON DELETE CASCADE,
    event_type        TEXT NOT NULL CHECK (event_type IN ('billing', 'price_increase', 'price_decrease', 'interval_change')),
    amount            NUMERIC(18,2) NOT NULL,
    previous_amount   NUMERIC(18,2),
    delta_pct         NUMERIC(8,2),
    interval_days     INT,
    occurred_at       DATE NOT NULL,
    sync_run_id       UUID,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
  );

  CREATE TABLE subscription_rejections (
    fingerprint       TEXT PRIMARY KEY,
    rejected_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    reason            TEXT
  );

  CREATE TABLE subscription_alerts (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_id        UUID REFERENCES subscription_patterns(id) ON DELETE SET NULL,
    alert_type        subscription_alert_type NOT NULL,
    title             TEXT NOT NULL,
    body              TEXT,
    read_at           TIMESTAMPTZ,
    sync_run_id       UUID,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
  );

  CREATE INDEX ON subscription_patterns (status, kind);
  CREATE INDEX ON subscription_patterns (last_seen_at DESC);
  CREATE INDEX ON subscription_price_events (pattern_id, occurred_at DESC);
  CREATE INDEX ON subscription_alerts (read_at, created_at DESC);
  ```
- **State transitions:**
  - Detection creates/updates `pending` rows; upgrades confidence if stronger evidence
  - User confirm → `confirmed`, sets `confirmed_at`, preserves `kind`
  - User reject → `rejected`, insert `subscription_rejections(fingerprint)`, soft-hide from UI
  - Gap > 2× interval without new tx → `inactive` (keep history, stop alerts)
- **No hypertable for MVP:** row volume ≪ forecast series; relational tables sufficient (R-0007 pattern deferred); revisit if price-event history exceeds ~100k rows
- **Forecast integration hook:** confirmed patterns exported as `ConfirmedRecurring` list (payee_key, amount, interval_days); rejected fingerprints passed as exclusion set (AC-8)
- **Alternatives considered:**
  - *Separate `subscription_candidates` + `subscriptions` tables* — rejected (duplicate columns; awkward promotion migration)
  - *Polymorphic single table with JSONB evidence blob* — rejected (harder to query/index for Grafana)
  - *Store state in Firefly tags* — rejected (read-only Firefly guarantee per DEC-0004)
- **Risks:** fingerprint collisions if normalization too aggressive; CASCADE delete on pattern removes price history — acceptable for MVP; rejection list grows unbounded (negligible at household scale)

### BUG-0008 addendum (2026-06-08)

Schema gap: `subscription_alerts` has **no fingerprint column or partial unique index** — root cause of W (83 unread `new_detection` vs 6 pending). Recommended migration adds `fingerprint TEXT NOT NULL` + `CREATE UNIQUE INDEX subscription_alerts_unread_fingerprint ON subscription_alerts (fingerprint) WHERE read_at IS NULL` per R-0068 §1.2 (mirrors R-0023 unified-alert pattern). Orphan cleanup on confirm/reject via lifecycle hooks (R-0068 §4).

**Linked:** US-0003, R-0009, R-0010, R-0011, DEC-0004, BUG-0008, R-0068  
**Confidence:** high  
**Status:** current

---

## R-0013 — Post-sync subscription detection pipeline & forecast integration

**Date:** 2026-05-31  
**Topic:** Sync mutex integration order, detection trigger, and forecast override hook for US-0003  
**Query:** Inline detection in sync mutex vs async job; detection before/after forecast recompute; extends DEC-0010  
**Sources:**
- [DEC-0010](decisions/DEC-0010.md) — inline recompute extends sync mutex; US-0003+ may add queue if latency problematic
- Existing: `backend/src/sync/mod.rs` — phase `"sync"` → `"forecast"` → clear mutex
- [R-0007](docs/engineering/research.md#r-0007--timescaledb-hypertable-schema-for-forecast-snapshots) — recompute latency scales with accounts × horizon

**Findings:**
- **Recommended pipeline order (extends DEC-0010 mutex):**
  ```
  1. Firefly entity + transaction sync (existing)
  2. Subscription detection pass     ← NEW (phase: "subscriptions")
  3. Forecast recompute              ← existing (phase: "forecast"; consumes confirmed subs)
  4. Clear active_run mutex
  ```
  Detection **before** forecast ensures confirmed subscriptions override recurring heuristics in same sync cycle (AC-8); rejected fingerprints excluded in step 3.
- **Inline in mutex (MVP):** same Tokio task as sync per DEC-0010 precedent — no Redis/job queue until latency proven > ~30s combined
- **SyncService phase reporting:** extend `phase` lock values: `"sync"` | `"subscriptions"` | `"forecast"`; Sync Status UI shows current phase
- **Detection service contract:**
  ```rust
  SubscriptionService::run_detection(sync_run_id) -> DetectionResult {
    // 1. Load expense transactions since watermark (or full history on first run)
    // 2. Run shared recurrence core (R-0009)
    // 3. Classify kind (R-0010)
    // 4. Upsert subscription_patterns; skip subscription_rejections fingerprints
    // 5. For confirmed patterns: check price/interval changes (R-0011); emit subscription_alerts
    // 6. Return ConfirmedRecurring[] + RejectedFingerprint[] for forecast
  }
  ```
- **Failure semantics:** detection failure → log warning, continue to forecast with prior confirmed state (mirrors forecast failure handling); sync run still `success` if ingest succeeded
- **Manual trigger:** `POST /api/v1/sync/trigger` returns 409 while any phase active (unchanged semantics)
- **API surface (architecture detail):**
  - `GET /api/v1/subscriptions?status=&kind=` — list patterns
  - `POST /api/v1/subscriptions/{id}/confirm` — body optional `{ kind }`
  - `POST /api/v1/subscriptions/{id}/reject`
  - `GET /api/v1/subscriptions/{id}/price-history`
  - `GET /api/v1/subscriptions/alerts?unread=true`
  - `PATCH /api/v1/subscriptions/alerts/{id}/read`
- **Forecast hook:** extend `ForecastService::recompute` to accept `confirmed_recurring: &[ConfirmedRecurring]` and `rejected_fingerprints: &HashSet<String>`; confirmed patterns replace heuristic matches for same payee_key; rejected excluded entirely
- **Alternatives considered:**
  - *Detection after forecast* — rejected (one sync cycle lag before confirmed subs affect projection)
  - *Async job queue (Redis)* — deferred (DEC-0010); introduce only if combined sync+detection+forecast exceeds threshold
  - *Fire-and-forget spawn* — rejected (race with next sync per DEC-0010)
- **Risks:** mutex duration grows by detection pass (~O(transactions) grouping); large histories may need incremental detection window (last 12 months default, full rescan weekly); phase UI must communicate longer "running" state

### BUG-0008 addendum (2026-06-08)

Pipeline step 5 (`emit subscription_alerts`) must become **idempotent upsert** before step 4 threshold tuning (R-0069). Add `GET /api/v1/subscriptions/alerts/unread-count` to API surface (R-0068 §2). AI enrichment **not** in sync mutex — deferred async path only if architecture approves (R-0069 §3). W fix (dedup) is **prerequisite** for X recall work — fixing X first re-amplifies W (discovery risk #1).

**Linked:** US-0003, DEC-0010, DEC-0007, R-0009, R-0012, BUG-0008, R-0068, R-0069  
**Confidence:** high  
**Status:** current

---

## R-0014 — Grafana Dashboard 2 (Subscriptions) provisioning

**Date:** 2026-05-31  
**Topic:** Dashboard-as-code for subscription analytics extending R-0008 pattern  
**Query:** Grafana Dashboard 2 panels — subscription spend, price changes, new detections; event table vs time series; account variable scope  
**Sources:**
- [R-0008](docs/engineering/research.md#r-0008--grafana-dashboard-as-code-for-analytics-dashboards) (provisioning pattern, stable uids, PostgreSQL queries)
- [R-0012](docs/engineering/research.md#r-0012--subscription-persistence-schema-candidates-confirmed-rejections-events) (schema for panel queries)
- US-0003 discovery: uid `subscriptions`; panels for spend stat, price changes, new/pending counts

**Findings:**
- **Extend R-0008 pattern:** add `grafana/provisioning/dashboards/analytics/subscriptions.json`; uid **`subscriptions`**, title **Subscriptions**, folder **Analytics**
- **Recommended panels:**
  | Panel | Type | Query source |
  |-------|------|--------------|
  | Confirmed subscription count | Stat | `SELECT COUNT(*) FROM subscription_patterns WHERE status='confirmed' AND kind='subscription'` |
  | Monthly subscription spend | Stat | normalized monthly equivalent for all confirmed subscriptions |
  | Pending review count | Stat | `SELECT COUNT(*) FROM subscription_patterns WHERE status='pending'` |
  | Price change events | **Table** (MVP) | join `subscription_price_events` + `subscription_patterns`; filter price_increase/decrease; last 90 days |
  | New detections (rolling) | Time series | daily count of new patterns by `created_at` |
- **Event table vs time series (discovery open question):** **Table for price changes** (discrete events with before/after columns); **time series for detection volume** (daily count trend). Price-change time series loses direction context unless split into two series.
- **Template variable:** `$scope` with options `global` (default) | per-account — MVP **global only** (patterns are payee-scoped, not account-scoped in R-0012 schema); defer account filter to architecture if multi-account attribution added via `subscription_pattern_transactions → transactions.account_id`
- **Datasource:** uid `FlowFinancePostgreSQL` (DEC-0012); `"id": null` in JSON
- **Alternatives considered:**
  - *Price changes as time series only* — rejected (hard to show delta columns)
  - *Prometheus metrics exporter* — rejected (duplicate storage per R-0008)
  - *Manual dashboard* — rejected (not reproducible)
- **Risks:** monthly-spend stat requires interval normalization logic in SQL (weekly × 4.33, annual ÷ 12); panel queries must handle empty state gracefully; bind-mount update interval 30s (R-0008)

**Linked:** US-0003, R-0008, R-0012, DEC-0012  
**Confidence:** high  
**Status:** current

---

## R-0015 — Plan Engine delta overlay on forecast baseline

**Date:** 2026-05-31  
**Topic:** Scenario modeling architecture for US-0004 Plan Engine extending US-0002 forecast  
**Query:** Fork US-0002 forecast computation with delta overlay vs independent Plan Engine projection layer; sensitivity-scenario overlay patterns for personal finance what-if analysis  
**Sources:**
- [R-0006](docs/engineering/research.md#r-0006--rule-based-personal-finance-forecast-algorithms-mvp-baseline) (forecast layered model)
- [R-0007](docs/engineering/research.md#r-0007--timescaledb-hypertable-schema-for-forecast-snapshots) (precomputed forecast snapshots)
- [FP&A Trends — scenario overlays vs versioning](https://fpa-trends.com/article/evolution-scenario-planning/)
- [Oracle EPM — inherited scenarios](https://docs.oracle.com/pls/topic/lookup?ctx=cloud&id=EPBUG-GUID-5FD6625F-0B5E-44A7-A5EA-D3B53A871E53)
- [Runway — what-if scenario sandbox](https://runway.com/blog/what-if-scenarios-in-finance-and-how-to-use-them-right)
- Existing: `backend/src/forecast/project.rs`, `backend/src/forecast/service.rs`

**Findings:**
- **Reuse vs fork (discovery open question):** **Delta overlay on latest forecast baseline** — do not fork or duplicate the full forecast engine. Store only explicit user-defined deltas; apply them at projection time atop the latest successful `forecast_computations` snapshot (R-0007). Matches FP&A "sensitivity overlay" pattern: baseline dataset unchanged, handful of assumption overrides recalculate derived outputs (FP&A Trends).
- **Baseline source:** **Current (Ist) scenario** = latest successful forecast computation with zero plan deltas. Scenario templates and custom plans inherit this baseline; deltas adjust the projected recurring/cashflow layer only — never Firefly mirror data (DEC-0004).
- **Delta model (explicit user adjustments):**
  ```text
  PlanAdjustment {
    direction: add_outflow | remove_outflow | add_inflow | remove_inflow
    amount: NUMERIC(18,2)          -- signed magnitude; direction disambiguates
    frequency: monthly | weekly | quarterly | one_time  -- default monthly
    effective_from: DATE           -- default today
    effective_to: DATE | NULL      -- NULL = indefinite
    target_type: subscription | category | custom_label | household
    target_ref: UUID | TEXT | NULL -- subscription_pattern.id or categories.firefly_id
  }
  ```
- **Template → delta bundles (preset, user-editable on apply):**
  | Template | Default deltas |
  |----------|----------------|
  | **Current (Ist)** | none (baseline only) |
  | **Leasing** | `+€300/month` household outflow (custom label "Leasing") |
  | **Savings mode** | remove selected confirmed subscriptions (US-0003 pick-list) + optional `−€100/month` discretionary cut |
  | **House purchase** | `+€X/month` savings transfer (custom label or savings category) |
- **Application algorithm (extends `project_account` logic per R-0006):**
  1. Load latest successful forecast `computation_id` daily/monthly series as baseline
  2. For each plan adjustment, project recurring impact onto future calendar days (same cadence math as forecast recurring layer)
  3. One-time deltas apply on `effective_from` date only
  4. Subscription-targeted removals zero out matching confirmed recurring amounts from baseline projection layer (reuse US-0003 `payee_key` match)
  5. Emit plan-specific daily net-cashflow and month-end balance series (stored separately from forecast hypertables — R-0018)
- **Alternatives considered:**
  - *Independent Plan Engine re-running full projection* — rejected (duplicates forecast logic; drift risk vs DEC-0007)
  - *Store only monthly deltas, derive daily on read* — rejected for Grafana/dashboard consistency (needs stable daily grain per acceptance)
  - *Mutate forecast hypertables in-place* — rejected (breaks Ist baseline; violates read-only separation)
- **Risks:** delta effective-date edge cases at month boundaries; subscription removal requires confirmed sub still active; overlay order when multiple deltas target same payee; baseline stale if forecast recompute fails post-sync

**Linked:** US-0004, R-0006, R-0007, R-0012, DEC-0004, DEC-0007, DEC-0010  
**Confidence:** high  
**Status:** current

---

## R-0016 — Plan scenario versioning (immutable snapshots vs editable drafts)

**Date:** 2026-05-31  
**Topic:** Version semantics for named plans with v1/v2/v3 compare in US-0004  
**Query:** Immutable snapshots on "new version" vs editable in-place with history; FP&A budget version control patterns  
**Sources:**
- [Lumel — FP&A version control guide](https://lumel.com/blog/planning/budget-forecast-versions-control-management/)
- [Glencoyne — financial model version control](https://www.glencoyne.com/guides/version-control-financial-models)
- [DEV — audit trail immutable snapshot pattern](https://dev.to/jeremiah_say/the-audit-trail-pattern-architecture-for-immutable-sustainability-data-1lf)
- US-0004 discovery: v1/v2/v3 side-by-side compare; create new version from prior

**Findings:**
- **Recommended hybrid (discovery open question):**
  - **Named plan** = container with metadata (`name`, `template`, `is_active`, `created_at`)
  - **Versions** = numbered snapshots **v1, v2, v3** (hard cap **3** per plan for MVP — matches acceptance and Finanzguru UX reference)
  - **Draft semantics:** only the **latest version** is editable in-place until user clicks "Create new version" — then current state is **frozen** (immutable) and a new editable copy becomes latest
  - **"Create new version"** copies all `plan_adjustments` from source version into new version row; prior version rows never mutate (append-only version history)
- **Compare view metrics (side-by-side v1/v2/v3):**
  - Monthly delta sum (net recurring impact €/month)
  - Projected month-end balance (household aggregate, current month + next month)
  - Optional: projected free cashflow delta vs Current (Ist) baseline
  - **Table-first with grouped bar chart** (discovery UX: ECharts grouped bar secondary to metrics table — table answers "which version wins?" faster for 3 versions)
- **Active plan selection:** exactly one plan marked `is_active=true` globally (DB partial unique index); drives plan-vs-Ist API, React default, and Grafana Dashboard 3 variable
- **Version limit enforcement:** on attempt to create v4, return 409 with message to delete/archive oldest or create new named plan — prevents unbounded compare UX breadth
- **Alternatives considered:**
  - *Fully immutable every save* — rejected (too heavy for iterative what-if tuning during single session)
  - *Unlimited versions with git-like branching* — rejected (acceptance caps at v1/v2/v3; scope creep)
  - *Editable in-place with audit log only* — rejected (compare view needs stable version boundaries)
- **Risks:** users may expect v4+ for long-running plans; frozen version still references live subscription IDs that may be rejected later; active plan switch mid-month affects Ist comparison continuity

**Linked:** US-0004, R-0015  
**Confidence:** high  
**Status:** current

---

## R-0017 — Plan-vs-Ist daily computation & aggregation grain

**Date:** 2026-05-31  
**Topic:** Daily planned vs actual (Ist) deviation metric and Firefly actuals aggregation for US-0004  
**Query:** Household free cashflow vs category budget spend vs account balance path; daily plan-vs-Ist primary number; Firefly read-only actuals aggregation  
**Sources:**
- [Firefly III Insights API — expense by category/budget](https://www.mintlify.com/firefly-iii/firefly-iii/api/insights)
- [FinToolSuite — budget variance calculator](https://fintoolsuite.com/en/tools/budget/budget-vs-actual-variance-calculator/)
- [R-0002](docs/engineering/research.md#r-0002--firefly-iii-pagination--incremental-sync-strategy) (mirror transaction ingest)
- [R-0006](docs/engineering/research.md#r-0006--rule-based-personal-finance-forecast-algorithms-mvp-baseline) (category buckets income/fixed/variable)
- Existing mirror schema: `transactions`, `categories`, `budgets` in `001_initial.sql`

**Findings:**
- **Primary metric (discovery open question):** **Household daily net cashflow** — sum of all non-transfer transaction amounts per calendar day across asset accounts:
  ```sql
  -- Ist (actual) daily aggregate (read-only from mirror)
  SELECT t.date AS day,
         SUM(t.amount::numeric) AS actual_net_cashflow
  FROM transactions t
  JOIN accounts a ON a.firefly_id = t.account_id
  WHERE a.type = 'asset'
    AND (t.payload->>'type' IS NULL OR t.payload->>'type' != 'transfer')
    AND t.date BETWEEN $month_start AND $month_end
  GROUP BY t.date
  ORDER BY t.date;
  ```
  - Positive = net inflow day; negative = net outflow day
  - Matches monthly `free_cashflow` decomposition vocabulary from R-0006 without requiring Firefly budget API writes
- **Planned daily series:** from active plan version's precomputed `plan_daily_cashflow` (R-0018) — same net-cashflow sign convention
- **Deviation (Abweichung):** `deviation = actual_net_cashflow - planned_net_cashflow` (signed; positive = better than plan, negative = worse)
- **Default view:** current calendar month; include days with zero transactions (actual = 0) for continuous chart
- **Secondary drill-down (React table, not primary Grafana MVP):** per-category actual spend vs category-targeted plan deltas when `target_type = category`; defer per-Firefly-budget panel until US-0005 budget-drift alerts
- **Ist lag handling:** if `last_successful_sync_at` < today, mark API response `actuals_stale=true` with `last_sync_at`; do not extrapolate actuals — show null/gap for days after last synced transaction date
- **Savings mode actuals:** removing a subscription from plan does not change Ist until user actually cancels in real life — deviation correctly surfaces "plan assumes cancelled, still paying"
- **Alternatives considered:**
  - *Account balance path* — rejected as primary (balance mixes historical noise; plan deltas are cashflow-oriented)
  - *Firefly budget limits as plan* — rejected (Flow plans are explicit user deltas, not Firefly budget mutation; Firefly budgets remain vocabulary/drill-down only)
  - *Monthly-only comparison* — rejected (acceptance requires daily plan-vs-Ist)
- **Risks:** multi-currency households need single reporting currency assumption (MVP: primary account currency); internal transfers excluded but split transactions may mis-aggregate; sparse sync days show flat zero actual

**Linked:** US-0004, R-0015, R-0016, R-0002, DEC-0004  
**Confidence:** high  
**Status:** current

---

## R-0018 — Plan persistence schema (plans, versions, adjustments, daily snapshots)

**Date:** 2026-05-31  
**Topic:** PostgreSQL schema for US-0004 Plan Engine — migration 004  
**Query:** Plan entity model, version storage, adjustment lines, precomputed daily series for API/Grafana  
**Sources:**
- [R-0004](docs/engineering/research.md#r-0004--rust-axum--sqlx--timescaledb-foundation-patterns) (SQLx migration pattern)
- [R-0007](docs/engineering/research.md#r-0007--timescaledb-hypertable-schema-for-forecast-snapshots) (computation metadata + hypertable pattern)
- [R-0012](docs/engineering/research.md#r-0012--subscription-persistence-schema-candidates-confirmed-rejections-events) (lifecycle entity + satellites)
- [R-0015](docs/engineering/research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline) (delta model)
- [R-0016](docs/engineering/research.md#r-0016--plan-scenario-versioning-immutable-snapshots-vs-editable-drafts) (version semantics)

**Findings:**
- **Recommended schema (migration 004):**
  ```sql
  CREATE TYPE plan_template AS ENUM ('current', 'leasing', 'savings_mode', 'house_purchase', 'custom');
  CREATE TYPE plan_adjustment_direction AS ENUM ('add_outflow', 'remove_outflow', 'add_inflow', 'remove_inflow');
  CREATE TYPE plan_adjustment_frequency AS ENUM ('monthly', 'weekly', 'quarterly', 'one_time');
  CREATE TYPE plan_adjustment_target AS ENUM ('household', 'subscription', 'category', 'custom_label');

  CREATE TABLE plans (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    template        plan_template NOT NULL DEFAULT 'custom',
    is_active       BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
  );
  CREATE UNIQUE INDEX plans_one_active ON plans (is_active) WHERE is_active = true;

  CREATE TABLE plan_versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id         UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    version_number  SMALLINT NOT NULL CHECK (version_number BETWEEN 1 AND 3),
    label           TEXT,
    is_latest       BOOLEAN NOT NULL DEFAULT true,
    frozen_at       TIMESTAMPTZ,           -- set when superseded by newer version
    baseline_computation_id UUID REFERENCES forecast_computations(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (plan_id, version_number)
  );
  CREATE UNIQUE INDEX plan_versions_one_latest ON plan_versions (plan_id) WHERE is_latest = true;

  CREATE TABLE plan_adjustments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version_id      UUID NOT NULL REFERENCES plan_versions(id) ON DELETE CASCADE,
    direction       plan_adjustment_direction NOT NULL,
    amount          NUMERIC(18,2) NOT NULL CHECK (amount > 0),
    frequency       plan_adjustment_frequency NOT NULL DEFAULT 'monthly',
    effective_from  DATE NOT NULL DEFAULT CURRENT_DATE,
    effective_to    DATE,
    target_type     plan_adjustment_target NOT NULL DEFAULT 'household',
    target_ref      TEXT,                  -- subscription UUID or category firefly_id
    label           TEXT NOT NULL,
    sort_order      SMALLINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
  );

  CREATE TABLE plan_computations (
    id              UUID PRIMARY KEY,
    version_id      UUID NOT NULL REFERENCES plan_versions(id) ON DELETE CASCADE,
    forecast_computation_id UUID NOT NULL REFERENCES forecast_computations(id),
    computed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    status          TEXT NOT NULL,         -- running | success | failed
    error_message   TEXT
  );

  CREATE TABLE plan_daily_cashflow (
    ts              TIMESTAMPTZ NOT NULL,  -- calendar day
    version_id      UUID NOT NULL REFERENCES plan_versions(id) ON DELETE CASCADE,
    computation_id  UUID NOT NULL REFERENCES plan_computations(id) ON DELETE CASCADE,
    planned_net     NUMERIC(18,2) NOT NULL,
    planned_balance NUMERIC(18,2)           -- optional cumulative for compare metrics
  );
  SELECT create_hypertable('plan_daily_cashflow', 'ts', if_not_exists => true, chunk_time_interval => INTERVAL '7 days');

  CREATE INDEX ON plan_daily_cashflow (version_id, computation_id, ts DESC);
  CREATE INDEX ON plan_adjustments (version_id, sort_order);
  ```
- **Retention:** keep last **3** successful `plan_computations` per version (lighter than forecast's 5 — plan rows keyed by version); cascade delete on version removal
- **Savings mode helper:** optional join table `plan_adjustment_subscriptions(adjustment_id, pattern_id)` when template auto-populates from confirmed subs — or store `target_ref = pattern_id` with `target_type = subscription`
- **Alternatives considered:**
  - *JSONB blob per version for adjustments* — rejected (Grafana/SQL queries need relational grain)
  - *Store plan series inside forecast hypertables* — rejected (namespace collision; baseline isolation)
  - *No hypertable for plan_daily* — acceptable at household scale but hypertable keeps parity with R-0007 Grafana `$__timeFilter` patterns
- **Risks:** `baseline_computation_id` on frozen version becomes stale reference (informational only); partial unique index on `is_active` requires transactional deactivate-before-activate; CASCADE delete removes compare history

**Linked:** US-0004, R-0015, R-0016, R-0017, R-0007, DEC-0005  
**Confidence:** high  
**Status:** current

---

## R-0019 — Plan recompute trigger & forecast baseline hook

**Date:** 2026-05-31  
**Topic:** When to recompute plan projections; sync mutex integration; active plan overlay on `/forecast`  
**Query:** Recompute on plan save only vs extend sync mutex after forecast; forecast page overlay scope  
**Sources:**
- [DEC-0010](decisions/DEC-0010.md) — sync mutex inline recompute
- [DEC-0018](decisions/DEC-0018.md) — subscriptions → forecast pipeline order
- [R-0013](docs/engineering/research.md#r-0013--post-sync-subscription-detection-pipeline--forecast-integration) (post-sync phase pattern)
- Existing: `backend/src/sync/mod.rs` phases `sync` → `subscriptions` → `forecast`

**Findings:**
- **Recompute triggers (discovery open question):**
  1. **On plan mutation** — create/update plan, apply template, add/edit/delete adjustment, create new version, switch active plan → recompute affected version(s) immediately (async Tokio task, **outside** sync mutex)
  2. **After forecast recompute** — when active plan exists, recompute active latest version using new `forecast_computation_id` baseline (hook at end of `ForecastService::recompute`, not new sync phase)
  3. **NOT in sync mutex** — plan overlay is O(deltas × horizon_days) ≪ forecast O(accounts × transactions); adding `"planning"` phase risks unnecessary mutex extension (R-0013 latency precedent)
- **Implementation hook:**
  ```rust
  // After forecast success in sync/mod.rs OR plan API handler:
  if let Some(active) = plan_service.active_plan().await? {
      plan_service.recompute_version(active.latest_version_id, forecast_computation_id).await?;
  }
  ```
- **Failure semantics:** plan recompute failure does not fail sync run (mirror forecast failure handling); API serves last successful plan computation with `stale=true`
- **Active plan on `/forecast` (discovery open question):** **Defer overlay to `/planning` only in US-0004** — keep `/forecast` as pure Ist baseline per US-0002 acceptance; optional read-only "View in Planning" link. Avoid dual-page drift; US-0006 `simulate_plan` tool consumes plan API separately
- **Baseline binding:** each `plan_computation` records `forecast_computation_id` used — compare view shows which forecast baseline backed each version
- **Alternatives considered:**
  - *Sync mutex `"planning"` phase* — rejected unless profiling shows >30s combined (DEC-0010 deferral pattern)
  - *On-demand plan compute without persistence* — rejected (Grafana Dashboard 3 + acceptance require stored series)
  - *Recompute all plans on sync* — rejected (wasteful; only active plan needed for Ist/Grafana MVP)
- **Risks:** race if user edits plan during post-sync recompute — use version-level lock or `is_latest` guard; stale plan if forecast fails but plan succeeds on old baseline

**Linked:** US-0004, R-0015, R-0018, DEC-0010, DEC-0018  
**Confidence:** high  
**Status:** current

---

## R-0020 — Grafana Dashboard 3 (Budgets: plan/ist/deviation) provisioning

**Date:** 2026-05-31  
**Topic:** Dashboard-as-code for US-0004 plan-vs-Ist analytics extending R-0008/R-0014 pattern  
**Query:** Dashboard 3 MVP household aggregate vs per-category panels; Plan/Ist/Abweichung SQL; uid `budgets`  
**Sources:**
- [R-0008](docs/engineering/research.md#r-0008--grafana-dashboard-as-code-for-analytics-dashboards) (provisioning, stable uids, `$__timeFilter`)
- [R-0014](docs/engineering/research.md#r-0014--grafana-dashboard-2-subscriptions-provisioning) (analytics folder pattern)
- [R-0017](docs/engineering/research.md#r-0017--plan-vs-ist-daily-computation--aggregation-grain) (Ist aggregation SQL)
- [R-0018](docs/engineering/research.md#r-0018--plan-persistence-schema-plans-versions-adjustments-daily-snapshots) (plan_daily_cashflow schema)
- [Sql-Financial-Variance-Analysis — BvA grain](https://github.com/gilchrist-jose/Sql-Financial-Variance-Analysis-Mazaya) (day/account grain join discipline)
- Existing: `grafana/provisioning/dashboards/analytics/cashflow.json`, `subscriptions.json`

**Findings:**
- **Extend R-0008/R-0014 pattern:** add `grafana/provisioning/dashboards/analytics/budgets.json`; uid **`budgets`**, title **Budgets**, folder **Analytics**, `"id": null`
- **MVP scope (discovery open question):** **Household aggregate only** — no per-category Grafana panels in US-0004 (category drill-down stays in React `/planning` Plan vs Actual tab); defer per-category Grafana to US-0005 budget-drift work
- **Template variable:** `$active_plan_version` — query latest version of active plan:
  ```sql
  SELECT pv.id AS __value, p.name || ' v' || pv.version_number AS __text
  FROM plan_versions pv
  JOIN plans p ON p.id = pv.plan_id
  WHERE p.is_active = true AND pv.is_latest = true
  LIMIT 1;
  ```
- **Recommended panels:**
  | Panel | Type | Series |
  |-------|------|--------|
  | **Plan** (planned daily net) | Time series | `plan_daily_cashflow.planned_net` for latest successful computation |
  | **Ist** (actual daily net) | Time series | mirror `transactions` daily aggregate (R-0017 SQL) |
  | **Abweichung** (deviation) | Time series | `actual - planned` via join on calendar day |
  | Plan vs Ist summary | Table | month-to-date sums: planned, actual, deviation, deviation % |
  | Active plan info | Stat/text | plan name, version, last computed_at |
- **Deviation panel SQL (join discipline — same day grain):**
  ```sql
  WITH actuals AS (
    SELECT t.date AS day, SUM(t.amount::numeric) AS actual_net
    FROM transactions t
    JOIN accounts a ON a.firefly_id = t.account_id
    WHERE a.type = 'asset'
      AND COALESCE(t.payload->>'type', '') != 'transfer'
      AND $__timeFilter(t.date::timestamptz)
    GROUP BY t.date
  ),
  planned AS (
    SELECT ts::date AS day, planned_net
    FROM plan_daily_cashflow
    WHERE version_id = '$active_plan_version'
      AND computation_id = (
        SELECT id FROM plan_computations
        WHERE version_id = '$active_plan_version' AND status = 'success'
        ORDER BY computed_at DESC LIMIT 1
      )
      AND $__timeFilter(ts)
  )
  SELECT COALESCE(a.day, p.day)::timestamptz AS time,
         COALESCE(a.actual_net, 0) - COALESCE(p.planned_net, 0) AS value
  FROM actuals a
  FULL OUTER JOIN planned p ON a.day = p.day
  ORDER BY 1;
  ```
- **Empty states:** when no active plan, show dashboard annotation "No active plan — select one in Flow Finance Planning UI"; stat panels return 0 with friendly text
- **Alternatives considered:**
  - *Per-category breakdown panels* — deferred (UX breadth; US-0005 budget drift)
  - *Reuse Dashboard 1 with overlay* — rejected (uid/folder separation; Budgets is plan-specific per Projectplan)
  - *Prometheus exporter for plan metrics* — rejected (duplicate storage per R-0008)
- **Risks:** `$active_plan_version` variable empty when no plan active; FULL OUTER JOIN on date types needs timestamptz cast for Grafana time axis; planned series extends future days while Ist stops at today — deviation panel should visually distinguish "forecast period" vs "historical" (optional shade per R-0008 threshold pattern)

**Linked:** US-0004, R-0008, R-0014, R-0017, R-0018, DEC-0012  
**Confidence:** high  
**Status:** current

---

## R-0021 — Net worth aggregation (Firefly asset accounts & snapshots)

**Date:** 2026-05-31  
**Topic:** Household net worth computation from synced Firefly mirrors; reporting currency; trend storage for US-0005  
**Query:** Firefly asset account types, include_net_worth flag, multi-currency balance aggregation, wealth-over-time snapshot patterns  
**Sources:**
- [Firefly III Accounts API](https://www.mintlify.com/firefly-iii/firefly-iii/api/accounts) — asset types, `include_net_worth`, account roles
- [Firefly III account types reference](https://docs.firefly-iii.org/references/firefly-iii/account-types/) — defaultAsset, savingAsset, cashWalletAsset, sharedAsset
- [Firefly III API currency conversion](https://docs.firefly-iii.org/references/firefly-iii/api/) — per-object currency + optional `pc_*` primary-currency fields
- [Kubera multi-currency net worth](https://www.kubera.com/blog/multi-currency-portfolio-tracker) — base-currency reporting pattern
- Existing mirror: `backend/migrations/001_initial.sql` (`accounts.type`, `balance`, `currency`, `payload`)
- [R-0001](docs/engineering/research.md#r-0001--firefly-iii-rest-api-integration-baseline), [DEC-0021](decisions/DEC-0021.md)

**Findings:**
- **Scope (US-0005 MVP):** sum **asset** accounts only; exclude liabilities, expense, revenue; **crypto excluded** from total (placeholder UI row until US-0007)
- **Account filter:**
  ```sql
  SELECT firefly_id, name, type, currency, balance,
         payload->>'account_role' AS account_role
  FROM accounts
  WHERE type = 'asset'
    AND COALESCE((payload->>'active')::boolean, true) = true
    AND COALESCE((payload->>'include_net_worth')::boolean, true) = true
  ORDER BY name;
  ```
  - Firefly roles in scope: `defaultAsset`, `sharedAsset`, `savingAsset`, `cashWalletAsset`, `ccAsset` (credit-card asset accounts with positive balance treated as asset per Firefly model)
  - `cash` pseudo-accounts handled via Firefly cash wallet asset type when present in mirror
- **Household total:** `SUM(balance)` grouped by reporting currency assumption — **EUR default** per DEC-0021 / Projectplan
- **Multi-currency MVP (discovery open question):** **sum native balances with mixed-currency warning banner** — do **not** implement FX conversion in US-0005:
  - If `COUNT(DISTINCT currency) > 1`, API returns `mixed_currency=true` + per-account native amounts; headline total sums numeric balances as-is with UI disclaimer ("Mixed currencies — totals approximate until conversion in US-0007+")
  - Prefer Firefly `payload->>'native_balance'` or synced `balance` as stored at ingest; optional future: read `pc_balance` if connector stores primary-currency conversion from Firefly API
- **Per-account breakdown:** table columns — name, account_role label, currency, balance (native), % of household (within same currency group only when mixed)
- **Wealth-over-time (discovery open question):** **store daily snapshots** after successful sync — not on-demand only:
  ```sql
  CREATE TABLE net_worth_snapshots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    snapshot_date   DATE NOT NULL,
    sync_run_id     UUID REFERENCES sync_runs(id),
    total_eur       NUMERIC(18,2) NOT NULL,   -- sum per MVP rules; mixed-currency caveat in metadata
    account_count   INT NOT NULL,
    mixed_currency  BOOLEAN NOT NULL DEFAULT false,
    payload         JSONB NOT NULL DEFAULT '{}',  -- per-account breakdown array
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (snapshot_date)
  );
  ```
  - Upsert one row per calendar day on post-sync hook; powers React ECharts line + Grafana Dashboard 4 time series without re-scanning full transaction history
  - Row volume ≈365/year — relational table sufficient (no hypertable required for MVP; optional Timescale conversion if US-0007 extends history)
- **Crypto placeholder:** React `/wealth` shows static row "Connect exchanges — US-0007" with `included_in_total=false`; Grafana stat panel subtitle notes "excludes crypto"
- **Alternatives considered:**
  - *Live FX conversion to reporting currency* — rejected (needs exchange-rate source; US-0007 scope)
  - *Include liabilities in net worth* — rejected (Projectplan "wealth" phase focuses on asset aggregation; liabilities deferred)
  - *On-demand compute without snapshots* — rejected (Dashboard 4 trend + acceptance "total wealth" needs stable daily series)
- **Risks:** mixed-currency sum misleading without banner; `include_net_worth=false` accounts must be respected; ccAsset negative balances (debt) should be excluded or clamped — MVP: include only accounts with `balance >= 0` OR document ccAsset debt as out of scope until liability integration

**Linked:** US-0005, US-0007, R-0001, DEC-0021  
**Confidence:** high  
**Status:** current

---

## R-0022 — Alert Engine evaluation rules (scarcity, budget drift, plan viability)

**Date:** 2026-05-31  
**Topic:** Deterministic alert condition evaluation for US-0005 Alert Engine  
**Query:** Scarcity threshold on forecast path; budget drift % vs plan; plan viability infeasibility heuristics; household vs per-account scope  
**Sources:**
- [Yomio — tiered spending alerts](https://yomio.app/en/blog/how-to-stop-overspending-using-alerts) — percentage thresholds, actionable context
- [Billcut — scenario-based proactive alerts](https://www.billcut.com/blogs/scenario-based-alerts-in-finance-apps-think-ahead/) — predict risk before crisis using forecast + recurring patterns
- [spend-pulse](https://github.com/jbornhorst1524/spend-pulse) — pace-based triggers vs static thresholds
- [R-0006](docs/engineering/research.md#r-0006--rule-based-personal-finance-forecast-algorithms-mvp-baseline) — forecast balance path
- [R-0007](docs/engineering/research.md#r-0007--timescaledb-hypertable-schema-for-forecast-snapshots) — `forecast_balance_daily`, `forecast_cashflow_monthly`
- [R-0015](docs/engineering/research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline) — plan overlay on baseline
- [R-0017](docs/engineering/research.md#r-0017--plan-vs-ist-daily-computation--aggregation-grain) — category actuals aggregation
- [R-0018](docs/engineering/research.md#r-0018--plan-persistence-schema-plans-versions-adjustments-daily-snapshots) — `plan_daily_cashflow.planned_balance`

**Findings:**
- **Engine shape:** single `AlertService::evaluate(sync_run_id, context)` invoked post-sync with read-only access to latest forecast computation, active plan computation, mirror transactions, and TOML `[alerts]` config
- **Scarcity alert (AC-2):**
  - **Scope (discovery open question):** **household aggregate minimum balance path** — sum `forecast_balance_daily.balance` across all asset accounts per projected day; fire when **any** projected day in horizon `[today, today+45d]` OR **current-month month-end** balance falls below `scarcity_threshold_eur` (default €200)
  - Per-account scarcity deferred — too noisy for multi-account households; Dashboard 1 `$account_id` variable retains per-account visual threshold line
  - Severity: `warning` when first breach day within 14 days; `critical` when breach is tomorrow or already below threshold on latest actual balance
  - Message template: `"Projected balance €{min_balance} on {date} — below €{threshold} scarcity threshold"`
  - Entity ref: `household` (constant key for dedup)
- **Budget drift alert (AC-3):**
  - **Grain (discovery open question):** **active-plan category-targeted adjustments only** — not all Firefly categories, not Firefly budget entities:
    1. Load active plan latest version adjustments where `target_type = 'category'`
    2. For each targeted category, compute **MTD actual spend** = sum of expense amounts (negative `transactions.amount`, abs value) for category in current calendar month
    3. Compute **MTD plan target** = prorated monthly plan delta impact for that category (monthly amount × `days_elapsed / days_in_month`, or sum of daily planned outflow from plan series if category-specific daily available)
    4. Fire when `actual > target × (1 + budget_drift_pct/100)` (default +20%)
  - Skip categories with no plan target (no alert — avoids noise from Firefly budgets unused by Flow plans per R-0017)
  - Severity: `warning` at threshold; `critical` at 2× threshold
  - Entity ref: `category:{firefly_id}`
  - Example: "Lebensmittel spend €420 MTD — 24% above plan target €340"
- **Plan viability alert (AC-4):**
  - **Rule (discovery open question):** fire when **active plan** latest successful computation shows **infeasible month-end balance**:
    1. **Primary trigger:** projected `planned_balance` at end of **current month** < 0 (household aggregate from plan overlay per R-0015)
    2. **Secondary trigger (sustained deficit):** **2 consecutive** projected month-end balances (current + next month) both < 0
    3. Do **not** require N consecutive daily below-zero (too sensitive to mid-month timing)
  - Requires active plan; no alert when no plan selected
  - Uses plan overlay path (not raw Ist forecast) — answers "is my leasing scenario viable?" per Projectplan
  - Entity ref: `plan:{plan_id}:version:{version_id}`
  - Message: `"Leasing plan projects €-{deficit} month-end balance — scenario may not be viable"`
- **Evaluation inputs freshness:** use forecast/plan computation IDs from current sync cycle; if plan recompute still running, evaluate on last successful plan snapshot with `stale=true` metadata (mirrors R-0019)
- **Alternatives considered:**
  - *Pace-based drift (spend-pulse style)* — deferred (needs historical pace baseline; US-0009 ML scope)
  - *All Firefly categories vs plan* — rejected (no plan target = false positives)
  - *Free-cashflow deficit only* — rejected (month-end balance clearer for "can I afford leasing?" framing)
- **Risks:** category MTD proration inaccurate for one-time mid-month plan deltas; household scarcity masks single-account overdraft; plan viability on stale baseline after failed forecast

**Linked:** US-0005, R-0006, R-0007, R-0015, R-0017, R-0018, DEC-0021  
**Confidence:** high  
**Status:** current

---

## R-0023 — Alert persistence, deduplication & lifecycle (acknowledge / dismiss)

**Date:** 2026-05-31  
**Topic:** PostgreSQL schema and state machine for unified Alert Engine inbox (US-0005)  
**Query:** Alert dedup fingerprint; cooldown until condition clears; acknowledge vs dismiss semantics; boundary with US-0003 subscription_alerts  
**Sources:**
- [Notification system design — dedup + rate limits](https://sujeet.pro/articles/design-notification-system) — fingerprint, preference, lifecycle states
- [Redis alert deduplication pattern](https://oneuptime.com/blog/post/2026-03-31-redis-alert-deduplication/view) — SET NX EX fingerprint (adapted to PostgreSQL for MVP)
- [Batch alerting throttle config](https://oneuptime.com/blog/post/2026-01-30-batch-processing-alerting/view) — window + max alerts per entity
- [R-0011](docs/engineering/research.md#r-0011--subscription-price-change-detection--alert-thresholds) — subscription alerts remain page-scoped
- [R-0012](docs/engineering/research.md#r-0012--subscription-persistence-schema-candidates-confirmed-rejections-events) — satellite alert table pattern

**Findings:**
- **Boundary with US-0003:** `subscription_alerts` table **unchanged** — unified inbox covers `scarcity`, `budget_drift`, `plan_viability` only; optional header link "View subscription alerts" queries `/api/v1/subscriptions/alerts?unread=true` count (read-only cross-link, no migration)
- **Recommended schema (migration 005):**
  ```sql
  CREATE TYPE alert_type AS ENUM ('scarcity', 'budget_drift', 'plan_viability');
  CREATE TYPE alert_severity AS ENUM ('info', 'warning', 'critical');
  CREATE TYPE alert_status AS ENUM ('active', 'acknowledged', 'dismissed', 'resolved');

  CREATE TABLE alerts (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type        alert_type NOT NULL,
    severity          alert_severity NOT NULL DEFAULT 'warning',
    status            alert_status NOT NULL DEFAULT 'active',
    fingerprint       TEXT NOT NULL,
    entity_ref        TEXT NOT NULL,          -- household | category:{id} | plan:{id}:version:{vid}
    title             TEXT NOT NULL,
    message           TEXT NOT NULL,
    context           JSONB NOT NULL DEFAULT '{}',  -- threshold, actual, projected_date, etc.
    sync_run_id       UUID REFERENCES sync_runs(id),
    triggered_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    acknowledged_at   TIMESTAMPTZ,
    dismissed_at      TIMESTAMPTZ,
    resolved_at       TIMESTAMPTZ,
    last_evaluated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
  );

  CREATE UNIQUE INDEX alerts_active_fingerprint
    ON alerts (fingerprint)
    WHERE status IN ('active', 'acknowledged');

  CREATE INDEX alerts_status_triggered ON alerts (status, triggered_at DESC);
  CREATE INDEX alerts_unread ON alerts (triggered_at DESC)
    WHERE status = 'active' AND acknowledged_at IS NULL;
  ```
- **Fingerprint (dedup open question):**
  - `scarcity`: `scarcity:household:{YYYY-MM}` — one active scarcity alert per calendar month per breach episode
  - `budget_drift`: `budget_drift:category:{firefly_id}:{YYYY-MM}` — one per category per month
  - `plan_viability`: `plan_viability:{plan_id}:{version_id}` — one per plan version until resolved
  - On evaluate: if fingerprint exists with status `active|acknowledged` → update `last_evaluated_at` + context, **do not** insert duplicate
  - When condition **clears** → set `status=resolved`, `resolved_at=now()`; next breach creates new row
- **Acknowledge (AC-5):** `PATCH /api/v1/alerts/{id}/acknowledge` — sets `acknowledged_at`; removes from header bell unread count; remains visible on `/alerts` with muted styling
- **Dismiss (discovery open question):** **hide until condition clears or re-triggers** — not permanent suppress:
  - Sets `status=dismissed`, `dismissed_at=now()`
  - While condition still true, suppress UI surfacing (exclude from bell + active list)
  - If condition clears then re-fires → new alert row (new `triggered_at`) even if prior was dismissed
  - Permanent per-entity suppress deferred to TOML override list
- **No Redis for MVP:** PostgreSQL partial unique index sufficient at household scale (DEC-0010 precedent — defer Redis until proven need)
- **Alternatives considered:**
  - *Merge subscription alerts into same table* — rejected (backlog boundary; different UX surfaces)
  - *Permanent dismiss per entity+type* — rejected (user may miss re-emerging crisis)
  - *Redis SETNX dedup* — deferred (standard profile has redis container but no app dependency yet per R-0004)
- **Risks:** monthly fingerprint may miss intra-month re-breach after resolve; acknowledged alerts still "active" may confuse users — UI copy: "Acknowledged — still active"

**Linked:** US-0005, US-0003, R-0011, R-0012, R-0022  
**Confidence:** high  
**Status:** current

---

## R-0024 — Post-sync Alert Engine pipeline & net-worth snapshot hook

**Date:** 2026-05-31  
**Topic:** Sync mutex integration order for Alert Engine and net-worth snapshots extending DEC-0010 / DEC-0018 / DEC-0023  
**Query:** Inline alerts phase vs async job; evaluation after plan recompute; failure semantics  
**Sources:**
- [DEC-0010](decisions/DEC-0010.md) — sync mutex inline recompute
- [DEC-0018](decisions/DEC-0018.md) — subscriptions → forecast order
- [DEC-0023](decisions/DEC-0023.md) — plan refresh awaited inside forecast success path
- Existing: `backend/src/sync/mod.rs` phases `sync` → `subscriptions` → `forecast`
- Existing: `backend/src/forecast/service.rs` — `refresh_active_after_forecast` awaited before forecast returns Ok
- [R-0013](docs/engineering/research.md#r-0013--post-sync-subscription-detection-pipeline--forecast-integration) — inline mutex precedent
- [R-0019](docs/engineering/research.md#r-0019--plan-recompute-trigger--forecast-baseline-hook) — plan hook placement
- [R-0021](docs/engineering/research.md#r-0021--net-worth-aggregation-firefly-asset-accounts--snapshots), [R-0022](docs/engineering/research.md#r-0022--alert-engine-evaluation-rules-scarcity-budget-drift-plan-viability)

**Findings:**
- **Recommended pipeline order (extends DEC-0018):**
  ```
  1. Firefly sync                    (phase: "sync")
  2. Subscription detection          (phase: "subscriptions")
  3. Forecast recompute              (phase: "forecast")
     └─ inline: active plan refresh  (awaited inside ForecastService per DEC-0023)
  4. Net worth snapshot upsert       (phase: "alerts" — start)
  5. Alert Engine evaluate           (phase: "alerts")
  6. Clear mutex
  ```
- **Inline in mutex (discovery open question):** **yes** — Alert Engine runs synchronously after forecast+plan hook in same sync task per DEC-0010 pattern; evaluation is O(accounts × horizon_days + category_targets) ≪ forecast cost
- **Defer async queue:** only if combined sync+detection+forecast+alerts exceeds ~30s on operator hardware (same threshold as R-0013)
- **SyncService contract:**
  ```rust
  AlertService::run_post_sync(run_id, EvalContext {
    forecast_computation_id,
    plan_computation_id: Option<Uuid>,  // active plan latest success
    config: AlertsConfig,
  }) -> AlertEvalResult {
    // 1. Upsert net_worth_snapshot for today (R-0021)
    // 2. Evaluate scarcity, budget_drift, plan_viability (R-0022)
    // 3. Upsert/resolves alerts per fingerprint rules (R-0023)
  }
  ```
- **Phase reporting:** extend `phase` lock to `"alerts"` during step 4–5; Sync Status UI shows "Evaluating alerts…"
- **Failure semantics:** alert evaluation failure → log warning, sync run still `success` if ingest+forecast succeeded (mirrors subscription/forecast failure handling); last alert state preserved
- **Manual re-eval API:** `POST /api/v1/alerts/evaluate` (operator/debug) — optional stretch; primary path is post-sync
- **REST surface (architecture detail):**
  - `GET /api/v1/wealth` — net worth breakdown + snapshot metadata
  - `GET /api/v1/wealth/history?days=90` — trend series from `net_worth_snapshots`
  - `GET /api/v1/alerts?status=active` — inbox list
  - `GET /api/v1/alerts/unread-count` — header bell badge
  - `PATCH /api/v1/alerts/{id}/acknowledge`
  - `PATCH /api/v1/alerts/{id}/dismiss`
- **Alternatives considered:**
  - *Async Tokio spawn after mutex* — rejected (race with next sync; stale evaluation window)
  - *Alerts before forecast* — rejected (scarcity/plan viability need latest forecast/plan snapshots)
  - *Separate cron for alerts* — rejected (acceptance: proactive post-sync evaluation)
- **Risks:** mutex duration grows ~100–500ms for alert pass; plan async recompute on manual edit may lag alert evaluation until next sync — document "alerts refresh on sync" in UI

**Linked:** US-0005, DEC-0010, DEC-0018, DEC-0023, R-0013, R-0019, R-0021, R-0022, R-0023  
**Confidence:** high  
**Status:** current

---

## R-0025 — Alert threshold config centralization & Dashboard 1 scarcity wiring

**Date:** 2026-05-31  
**Topic:** TOML `[alerts]` config mirrored to DB for Grafana; replace Dashboard 1 static €200 (DEC-0012)  
**Query:** Grafana variable from PostgreSQL config query vs env injection; Config from query results for thresholds  
**Sources:**
- [Grafana community — variables not supported in threshold fields](https://community.grafana.com/t/set-treshold-values-by-variable/161736) — thresholds reject `$variable` syntax
- [Grafana Config from query results #45803](https://github.com/grafana/grafana/issues/45803) — map query fields to panel config
- [Grafana PostgreSQL template variables](https://grafana.com/docs/grafana/latest/datasources/postgres/template-variables/) — query variables for panel SQL
- [R-0008](docs/engineering/research.md#r-0008--grafana-dashboard-as-code-for-analytics-dashboards) — scarcity hardcode €200 pending US-0005
- Existing: `grafana/provisioning/dashboards/analytics/cashflow.json` — `200 AS value` in refId B
- [DEC-0012](decisions/DEC-0012.md) — static threshold decision to be superseded

**Findings:**
- **TOML config (backlog):**
  ```toml
  [alerts]
  scarcity_threshold_eur = 200.0
  budget_drift_pct = 20.0
  # optional future: plan_viability_deficit_eur = 0.0
  ```
  - Loaded at app startup into `AlertsConfig`; validated `> 0`
- **DB mirror for Grafana (recommended over env injection):**
  ```sql
  CREATE TABLE alert_config (
    id                      SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    scarcity_threshold_eur  NUMERIC(18,2) NOT NULL DEFAULT 200,
    budget_drift_pct        NUMERIC(5,2) NOT NULL DEFAULT 20,
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
  );
  INSERT INTO alert_config DEFAULT VALUES;
  ```
  - On startup and on config reload: `UPSERT alert_config` from TOML — single-row singleton
  - Alert Engine reads from in-memory `AlertsConfig` (hot path); Grafana reads from `alert_config` (same values)
- **Dashboard 1 migration (discovery open question):** replace hardcoded `200` with **Grafana query variable** — not threshold UI field:
  ```sql
  -- Template variable $scarcity_threshold (type: query, refresh on load)
  SELECT scarcity_threshold_eur AS __value, 'Scarcity threshold (€)' AS __text
  FROM alert_config WHERE id = 1;
  ```
  - Update refId B SQL in `cashflow.json`:
    ```sql
    SELECT ts AS time, $scarcity_threshold AS value
    FROM forecast_balance_daily
    WHERE account_id = '$account_id'
      AND computation_id = (...)
      AND $__timeFilter(ts)
    ORDER BY ts LIMIT 1
    ```
  - Variables **work in raw SQL** targets (unlike threshold config panel fields per Grafana community #161736)
- **Do not use Config from query results for MVP** — single constant series via `$scarcity_threshold` variable is simpler and matches existing refId B pattern (R-0008)
- **Panel title:** rename to `"Balance forecast with scarcity threshold (${scarcity_threshold} €)"` or static "Balance forecast with scarcity threshold"
- **Alternatives considered:**
  - *Env var in provisioning YAML only* — rejected (Alert Engine and Grafana diverge; no single source)
  - *API endpoint regenerates dashboard JSON* — rejected (ops complexity)
  - *Hardcode until US-0006* — rejected (acceptance requires configurable threshold)
- **Risks:** Grafana variable type coercion (numeric vs string) — cast in SQL: `$scarcity_threshold::numeric`; config change requires app restart or reload endpoint to sync DB mirror

**Linked:** US-0005, US-0002, R-0008, DEC-0012, R-0022  
**Confidence:** high  
**Status:** current

---

## R-0026 — Grafana Dashboard 4 (Portfolio / wealth partial) provisioning

**Date:** 2026-05-31  
**Topic:** Dashboard-as-code for US-0005 net worth analytics extending R-0008 / R-0014 / R-0020 pattern  
**Query:** uid `portfolio`; total wealth stat; account breakdown; wealth-over-time from snapshots; crypto deferred  
**Sources:**
- [R-0008](docs/engineering/research.md#r-0008--grafana-dashboard-as-code-for-analytics-dashboards) — provisioning pattern, stable uids
- [R-0014](docs/engineering/research.md#r-0014--grafana-dashboard-2-subscriptions-provisioning) — analytics folder
- [R-0020](docs/engineering/research.md#r-0020--grafana-dashboard-3-budgets-planistdeviation-provisioning) — Dashboard 3 SQL patterns
- [R-0021](docs/engineering/research.md#r-0021--net-worth-aggregation-firefly-asset-accounts--snapshots) — aggregation rules + `net_worth_snapshots`
- [Empower / Kubera net worth dashboard patterns](https://blog.investogy.com/best-investment-tracking-apps/) — headline total + account breakdown
- Existing dashboards: `grafana/provisioning/dashboards/analytics/*.json`

**Findings:**
- **Extend R-0008 pattern:** add `grafana/provisioning/dashboards/analytics/portfolio.json`
  - uid **`portfolio`**, title **Portfolio**, folder **Analytics**, `"id": null`, tags `["analytics", "wealth", "portfolio"]`
- **Recommended panels (MVP partial — US-0007 completes crypto/performance):**
  | Panel | Type | Query source |
  |-------|------|--------------|
  | **Total wealth (non-crypto)** | Stat | `SELECT COALESCE(SUM(balance), 0) FROM accounts WHERE type='asset' AND COALESCE((payload->>'include_net_worth')::boolean, true) AND COALESCE((payload->>'active')::boolean, true)` |
  | **Account count** | Stat | count of included asset accounts |
  | **Mixed currency warning** | Text/markdown | `SELECT CASE WHEN COUNT(DISTINCT currency)>1 THEN 'Mixed currencies — total approximate' ELSE '' END FROM accounts WHERE type='asset'` |
  | **Account breakdown** | Table | name, account_role, currency, balance, `%` of total (window function) |
  | **Wealth over time** | Time series | `SELECT snapshot_date::timestamptz AS time, total_eur AS value FROM net_worth_snapshots WHERE $__timeFilter(snapshot_date::timestamptz) ORDER BY 1` |
  | **By account type** | Pie/donut (optional) | group by `payload->>'account_role'` |
  | **Crypto placeholder** | Text | static markdown: "Crypto & exchange balances — US-0007" |
- **Template variables:** none required for MVP global household scope (matches R-0014 global default)
- **Link from React `/wealth`:** `GRAFANA_URL/d/portfolio` (uid route)
- **Empty states:** zero accounts → stat shows €0 with annotation "Sync Firefly accounts first"
- **Alternatives considered:**
  - *Compute wealth time series from raw balances each query* — rejected (no historical balance in mirror without snapshots per R-0021)
  - *Merge into Dashboard 1 cashflow* — rejected (Projectplan Dashboard 4 distinct; DEC-0012 uid separation)
  - *Crypto stub panel with zero* — rejected (misleading; text placeholder preferred)
- **Risks:** mixed-currency stat sum misleading — pair with warning panel; snapshot gaps on failed sync days show flat lines — acceptable; pie chart with single account redundant — optional panel

**Linked:** US-0005, US-0007, R-0008, R-0014, R-0020, R-0021, DEC-0012  
**Confidence:** high  
**Status:** current

---

## R-0027 — OpenAI tool-calling orchestration in Axum (async-openai)

**Date:** 2026-05-31  
**Topic:** AI Tool Layer orchestration loop, registry pattern, and OpenAI Chat Completions integration for US-0006  
**Query:** OpenAI function calling in Rust/Axum; async-openai tool loop; trait-based tool registry vs static JSON schemas; multi-step tool execution  
**Sources:**
- [OpenAI — Function calling guide](https://platform.openai.com/docs/guides/function-calling) — 5-step tool loop, JSON schema tools, strict mode, tool call outputs with `call_id`
- [64bit/async-openai](https://github.com/64bit/async-openai) — Chat Completions + SSE streaming, `CreateChatCompletionRequestArgs`, `FinishReason::ToolCalls`
- [async-openai tool-call-stream example](https://github.com/64bit/async-openai/blob/main/examples/tool-call-stream/src/main.rs) — stream initial response, collect tool calls, execute, append tool messages
- [openai-func-enums](https://github.com/frankfralick/openai-func-enums) — enum-based tool definitions (alternative)
- Existing: `backend/src/auth/mod.rs` (JWT middleware), service layer pattern (`ForecastService`, `PlanService`, `AlertService`)
- [DEC-0004](decisions/DEC-0004.md), [DEC-0006](decisions/DEC-0006.md)

**Findings:**
- **API surface (discovery open question):** **Chat Completions with `tools`** (not Responses API) for MVP — mature `async-openai` support, simpler multi-turn loop, aligns with US-0008 OpenAI-compatible local endpoints later
- **Recommended orchestration module:** `backend/src/ai/` with:
  ```rust
  pub trait AiTool: Send + Sync {
      fn name(&self) -> &'static str;
      fn description(&self) -> &'static str;
      fn parameters_schema(&self) -> serde_json::Value;  // JSON Schema object
      async fn execute(&self, ctx: &ToolContext, args: serde_json::Value)
          -> Result<serde_json::Value, ToolError>;
  }
  pub struct ToolContext {
      pub forecast: ForecastService,
      pub subscriptions: SubscriptionService,
      pub plans: PlanService,
      pub wealth: WealthService,
      pub alerts: AlertService,
      pub privacy: PrivacyConfig,   // R-0028
      // NO DbPool / NO sqlx — tools call services only
  }
  ```
- **Registry:** static `Vec<Arc<dyn AiTool>>` with exactly six tools registered at startup; build OpenAI `tools` array from `parameters_schema()` — **not** `openai-func-enums` for MVP (extra proc-macro dep; six tools manageable with `serde_json::json!` or `schemars` derive on arg structs)
- **Orchestration loop (per OpenAI 5-step flow):**
  1. Append user message to thread messages
  2. `client.chat().create_stream(request)` with `tools` + `tool_choice: auto`
  3. On `FinishReason::ToolCalls` → parse args JSON → execute each tool **in-process** (parallel `tokio::join!` when multiple calls)
  4. Append assistant tool-call message + `role: tool` messages with `tool_call_id`
  5. Repeat until `FinishReason::Stop` or **max_rounds** (default **5**) — prevents runaway agent loops
- **Strict mode:** set `"strict": true` on function definitions where args are fully enumerable (e.g. `horizon` enum on `get_forecast`); omit on free-text fields
- **Config (TOML `[ai]`):**
  ```toml
  [ai]
  provider = "openai"           # US-0008 extends
  model = "gpt-4o-mini"         # cost/latency default for self-hosted
  api_key_env = "OPENAI_API_KEY"
  max_tool_rounds = 5
  max_completion_tokens = 1024
  request_timeout_secs = 60
  ```
  - API key from env/secrets only — never in TOML plaintext, never exposed to browser
- **System prompt (minimal):** instruct model to use tools for factual finance data; never invent balances; respect privacy aggregates when `allow_raw_transactions=false`; cite tool names used
- **Tool transparency metadata:** orchestrator collects `{ tool_name, started_at, duration_ms, status }` per invocation for SSE events (R-0029) and audit rows (R-0030)
- **Alternatives considered:**
  - *openai-func-enums* — deferred (nice ergonomics; six tools don't justify new dep)
  - *Assistants API with hosted threads* — rejected (external thread storage; conflicts with self-hosted privacy posture)
  - *Direct SQL in tool handlers* — rejected (acceptance + DEC-0004 boundary)
  - *Static schema files only (no trait)* — rejected (execution still needs Rust dispatch; trait keeps testability)
- **Risks:** prompt injection may coerce harmful tool arg shapes — validate args with serde + JSON Schema constraints; model may hallucinate tool names — registry allowlist only; token cost on multi-round loops — cap rounds + summarize large tool outputs (R-0031); reasoning-model `reasoning` items in stream — defer until model selection requires it

**Linked:** US-0006, DEC-0004, DEC-0006, R-0028, R-0029, R-0030, R-0031  
**Confidence:** high  
**Status:** current

---

## R-0028 — Privacy redaction middleware for AI tool outputs

**Date:** 2026-05-31  
**Topic:** Central privacy layer enforcing Projectplan TOML defaults on tool payloads before OpenAI and audit persistence  
**Query:** Privacy middleware on tool output vs per-tool redaction; `allow_raw_transactions=false` semantics; IBAN/counterparty masking patterns  
**Sources:**
- [Projectplan.md](../../Projectplan.md) — `[privacy]` options: `allow_raw_transactions`, `redact_iban`, `redact_counterparties`
- [Microsoft PII Shield patterns](https://techcommunity.microsoft.com/blog/azuredevcommunityblog/introducing-pii-shield-a-privacy-proxy-for-every-llm-call/4514726) — regex + context keywords for IBAN/financial IDs; replace vs hash operators
- [mcp-server-gdpr-pii-redactor](https://github.com/vinkius-labs/mcp-server-gdpr-pii-redactor) — deterministic regex with structural validation (Luhn, MOD-97 IBAN)
- [anonyma](https://github.com/izaccavalheiro/anonyma) — IBAN MOD-97 validator; redact/mask/pseudonymize strategies
- US-0006 discovery open questions; [DEC-0004](decisions/DEC-0004.md)

**Findings:**
- **Placement (discovery open question):** **Central privacy middleware on tool output** — single `PrivacyLayer::redact_tool_result(tool_name, value) -> Value` invoked by orchestrator after every `AiTool::execute`, before (a) serializing to OpenAI tool message and (b) writing audit arg summary (R-0030). Per-tool helpers only for shape-specific aggregation, not ad-hoc string scrubbing
- **Config (TOML `[privacy]`, Projectplan defaults):**
  ```toml
  [privacy]
  allow_raw_transactions = false
  redact_iban = true
  redact_counterparties = true
  ```
  - Loaded at startup into `PrivacyConfig`; exposed read-only in Settings AI & Privacy (discovery: **read-only TOML display** for MVP — runtime toggle deferred; requires config reload endpoint + audit of changes)
- **`allow_raw_transactions=false` (discovery open question):** **`get_transactions` returns aggregates only** — never individual transaction rows to the model:
  ```json
  {
    "period": "2026-05",
    "top_categories": [{ "category": "Lebensmittel", "total_eur": -420.50, "tx_count": 23 }],
    "total_inflow_eur": 3200.00,
    "total_outflow_eur": -2850.00,
    "raw_transactions_included": false
  }
  ```
  - When `true`, return capped list (max **20** rows, last 30 days default) with IBAN/counterparty redaction still applied if those flags set
  - If model requests raw detail while disabled → tool returns structured `{ "error": "raw_transactions_disabled", "hint": "use category aggregates" }` — not empty payload
- **`redact_iban=true`:** regex detect ISO IBAN (MOD-97 check where possible) in any string field; replace with `"[IBAN_REDACTED]"`; also scan `accounts.iban` column paths in structured output
- **`redact_counterparties=true`:** replace payee/description/counterparty strings with deterministic pseudonym `"Counterparty-{hash8}"` where `hash8 = first 8 hex of SHA-256(normalized_name + app_pepper)` — stable within deployment so model can correlate repeated merchants without leaking names to OpenAI
- **Implementation shape:**
  ```rust
  pub struct PrivacyLayer { config: PrivacyConfig, pepper: String }
  impl PrivacyLayer {
      pub fn redact_json(&self, value: &mut serde_json::Value);
      pub fn summarize_args(&self, args: &serde_json::Value) -> serde_json::Value; // for audit
  }
  ```
  - Walk JSON tree recursively; redact known field names (`iban`, `payee`, `description`, `counterparty`, `destination_name`) plus regex pass on string leaves
- **OpenAI prompt path:** user chat messages are **not** pre-redacted for MVP (user may type their own IBAN); system prompt warns model not to echo sensitive identifiers; optional future: redact outbound user messages (US-0008 scope creep if heavy)
- **Alternatives considered:**
  - *Per-tool redaction only* — rejected (bypass risk if new tool forgets scrubbing)
  - *Token vault with rehydration (llm-hasher pattern)* — rejected for MVP (needs Ollama/NER infra; US-0008)
  - *Block all tool output when raw disabled* — rejected (model needs aggregates for example queries)
- **Risks:** regex IBAN false positives in descriptions; counterparty hash collisions negligible at household scale; aggregate-only mode may frustrate "show my last 5 transactions" — document in suggested prompts; Settings read-only may confuse operators expecting toggles

**Linked:** US-0006, DEC-0004, R-0027, R-0030, R-0031  
**Confidence:** high  
**Status:** current

---

## R-0029 — Chat streaming (SSE) with JWT auth (DEC-0006 gate)

**Date:** 2026-05-31  
**Topic:** Streaming chat API design for React ChatPanel; SSE over POST; Bearer JWT compatibility  
**Query:** SSE vs buffered chat MVP; OpenAI stream proxy; DEC-0006 BFF reconsideration gate  
**Sources:**
- [OpenAI — Streaming API responses](https://platform.openai.com/docs/guides/streaming-responses) — `stream=true`, delta chunks, tool-call streaming
- [async-openai SSE support](https://github.com/64bit/async-openai) — `create_stream` on chat completions
- [Axum SSE + AI streaming pattern](https://ellix.ai/blog/streaming-ai-responses) — `Sse`, `Event`, mpsc channel, abort on disconnect
- [OpenAI Node.js streaming guide](https://nodewire.net/openai-api-nodejs-tutorial/) — abort controller on client disconnect saves cost
- Existing: `backend/src/auth/mod.rs` — `require_auth` Bearer JWT; [DEC-0006](decisions/DEC-0006.md)

**Findings:**
- **Streaming vs buffered (discovery open question):** **SSE token stream for chat MVP** — acceptance UX ("Chat UI accepts natural-language questions") implies responsive drawer; buffered-only rejected for perceived latency
- **DEC-0006 gate:** **Bearer JWT on POST SSE is sufficient — no BFF/cookie auth required.** Same `require_auth` middleware as other `/api/v1/*` routes; frontend uses `fetch()` + `ReadableStream` (not native `EventSource`, which lacks POST + Authorization header support in browsers)
- **Endpoints:**
  | Method | Path | Purpose |
  |--------|------|---------|
  | POST | `/api/v1/chat/completions` | Non-streaming fallback (optional; tests/admin) |
  | POST | `/api/v1/chat/stream` | **Primary** SSE stream for ChatPanel |
  | GET | `/api/v1/chat/sessions/{id}` | Optional stretch — defer thread persistence (R-0030 companion) |
- **SSE event types (JSON payload per `data:` line):**
  ```text
  event: token        data: {"delta":"The "}
  event: tool_start   data: {"tool":"get_forecast","call_id":"..."}
  event: tool_end     data: {"tool":"get_forecast","duration_ms":42,"status":"ok"}
  event: done         data: {"message_id":"...","tools_used":[...]}
  event: error        data: {"code":"provider_error","message":"..."}
  ```
  - Final `done` event carries tool transparency list for collapsible UI row (no raw JSON args)
- **Axum handler pattern:**
  - Validate JWT → parse `{ messages, session_id? }` body
  - `let (tx, rx) = mpsc::channel(32)`
  - Spawn orchestrator task (R-0027) forwarding OpenAI deltas → `tx`
  - Return `Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))`
  - On request drop / client disconnect: abort OpenAI stream via `CancellationToken` (save tokens)
- **Response headers:** `Content-Type: text/event-stream`, `Cache-Control: no-cache`, `Connection: keep-alive`, `X-Accel-Buffering: no` (reverse-proxy flush)
- **Chat history (discovery open question):** **Ephemeral client-side thread for MVP** — React `ChatPanel` state + `sessionStorage` optional; **no DB thread persistence** in US-0006 (reduces migration scope; US-0008 may add if needed)
- **Rate limiting (discovery open question):** **Per-user throttle** — simple in-memory token bucket on `AuthUser.subject` (e.g. 20 requests / 10 min) in `AiService`; configurable `[ai] rate_limit_per_min`; defer Redis-backed limiter
- **Alternatives considered:**
  - *WebSockets* — rejected (SSE sufficient for server→client tokens; simpler through proxies)
  - *BFF cookie session for SSE* — rejected (DEC-0006 gate closed; fetch+Bearer works)
  - *Buffered JSON only* — rejected (UX)
- **Risks:** long-lived SSE connections through corporate proxies — keep-alive events required; partial stream failure — always emit terminal `error` or `done` event; tool-call streaming interleaved with tokens — UI should show "Using tools…" state on `tool_start`; multi-tab same user shares rate limit

**Linked:** US-0006, DEC-0006, R-0027, R-0030  
**Confidence:** high  
**Status:** current

---

## R-0030 — AI tool audit log persistence (migration 006)

**Date:** 2026-05-31  
**Topic:** Operator-visible tool invocation audit trail; schema, retention, and redaction boundaries  
**Query:** Dedicated `ai_tool_audit` table vs structured log file; retention cap vs time purge; GDPR/compliance posture for self-hosted MVP  
**Sources:**
- [Knowlee — AI audit trail implementation](https://www.knowlee.ai/blog/ai-audit-trail-implementation-guide) — JSONL vs relational tradeoffs; retention matrix
- [Markaicode — LLM chat log archiving](https://www.markaicode.com/compliance-llm-chat-log-archiving/) — append-only audit table, redacted prompt storage, indexed queries
- [SaaS audit trail 2026](https://viprasol.com/blog/saas-audit-trail/) — actor/resource/action model, partition-friendly PK
- [R-0004](docs/engineering/research.md#r-0004--rust-axum--sqlx--timescaledb-foundation-patterns) — SQLx migration pattern
- US-0006 acceptance: "Tool call audit log available for operator review"

**Findings:**
- **Storage (discovery open question):** **Dedicated PostgreSQL table `ai_tool_audit`** in migration **006** — not log-file-only. Operator Settings table needs indexed query; household row volume ≪ hypertable threshold
- **Recommended schema:**
  ```sql
  CREATE TABLE ai_tool_audit (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id      UUID NOT NULL,
    user_subject    TEXT NOT NULL,           -- JWT sub (or "dev-bypass")
    tool_name       TEXT NOT NULL CHECK (tool_name IN (
                      'get_transactions','get_subscriptions','get_forecast',
                      'get_budget_status','get_portfolio','simulate_plan')),
    args_summary    JSONB NOT NULL DEFAULT '{}',  -- redacted per R-0028
    result_status   TEXT NOT NULL CHECK (result_status IN ('ok','error')),
    result_rows     INT,                     -- optional count hint
    duration_ms     INT NOT NULL,
    error_message   TEXT,                    -- sanitized, no stack traces
    model           TEXT,                    -- e.g. gpt-4o-mini
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
  );
  CREATE INDEX ai_tool_audit_created ON ai_tool_audit (created_at DESC);
  CREATE INDEX ai_tool_audit_tool ON ai_tool_audit (tool_name, created_at DESC);
  CREATE INDEX ai_tool_audit_session ON ai_tool_audit (session_id, created_at DESC);
  ```
- **What NOT to store:** full OpenAI prompts/responses, unredacted tool payloads, API keys, raw transaction rows — violates privacy layer purpose
- **Write path:** orchestrator inserts row **after** each tool execution completes (success or error); `args_summary` via `PrivacyLayer::summarize_args`
- **Read path:** `GET /api/v1/ai/audit?limit=100&offset=0` — JWT-protected; Settings AI & Privacy table UI; columns: timestamp, tool, session (truncated), duration, status
- **Retention (discovery open question):** **Fixed row cap 500** + **90-day time purge** on startup job (whichever stricter):
  ```sql
  DELETE FROM ai_tool_audit
  WHERE id NOT IN (
    SELECT id FROM ai_tool_audit ORDER BY created_at DESC LIMIT 500
  );
  DELETE FROM ai_tool_audit WHERE created_at < now() - INTERVAL '90 days';
  ```
  - Self-hosted MVP: no FINRA 7-year requirement unless operator enables compliance profile; document in user guide
- **Immutability:** append-only for MVP — no UPDATE; DELETE only via retention job (operator-visible policy)
- **Alternatives considered:**
  - *JSONL file as system of record* — rejected (no Settings UI query without extra ingestion)
  - *Reuse `firefly_request_audit`* — rejected (different domain; Firefly GET audit per DEC-0004)
  - *Unbounded retention* — rejected (GDPR storage-minimization; household scale still grows)
- **Risks:** JWT `sub` alone may not be human-readable in Settings — optional display name from OIDC claims later; retention job DELETE vs compliance expectations — document configurable `[ai] audit_retention_days`; session_id UUID opaque — sufficient for correlating multi-tool turns

**Linked:** US-0006, R-0027, R-0028, R-0004, DEC-0004  
**Confidence:** high  
**Status:** current

---

## R-0031 — Six-tool registry mapping & `simulate_plan` read-only contract

**Date:** 2026-05-31  
**Topic:** Map registered AI tools to existing Flow services/REST surfaces; ephemeral plan simulation without persistence  
**Query:** `simulate_plan` active plan vs named scenario; `get_budget_status`/`get_portfolio` mapping; missing transactions API; payload truncation  
**Sources:**
- Existing APIs: `backend/src/api/{forecast,subscriptions,plans,wealth,alerts}.rs`
- [R-0015](docs/engineering/research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline) — plan overlay model
- [R-0019](docs/engineering/research.md#r-0019--plan-recompute-trigger--forecast-baseline-hook) — plan recompute triggers
- [R-0022](docs/engineering/research.md#r-0022--alert-engine-evaluation-rules-scarcity-budget-drift-plan-viability) — budget drift + plan viability rules
- [R-0021](docs/engineering/research.md#r-0021--net-worth-aggregation-firefly-asset-accounts--snapshots) — net worth aggregation
- [Projectplan.md](../../Projectplan.md) — six tool names + example queries
- [DEC-0004](decisions/DEC-0004.md)

**Findings:**
- **Boundary:** every tool calls **in-process `*Service` methods** (same logic as REST handlers) — never `sqlx::query` from `ai/` module, never Firefly HTTP client
- **Tool → service mapping (MVP):**

  | Tool | Service / method | Notes |
  |------|------------------|-------|
  | `get_transactions` | **NEW** `TransactionsService::aggregates(filter)` | No REST endpoint today — add read-only internal service querying mirror `transactions` + `categories`; privacy layer returns aggregates when raw disabled (R-0028) |
  | `get_subscriptions` | `SubscriptionService::list_patterns(status, kind)` | Wrap list + optional price-history summary; default `status=confirmed` for spend questions |
  | `get_forecast` | `ForecastService` via repository latest computation | Args: `horizon` (daily/monthly/long-term), optional `account_id`; return summarized series (max **30** points) + milestones |
  | `get_budget_status` | `AlertService` + active plan adjustments | Compose MTD category spend vs plan targets (R-0022 budget drift inputs) + active `budget_drift` alerts; not a standalone REST route today |
  | `get_portfolio` | `WealthService::compute_breakdown()` + optional `history(90)` | Maps to `/api/v1/wealth`; exclude crypto placeholder from total narrative |
  | `simulate_plan` | `PlanService` read-only projection | See below — **no** `POST /plans`, **no** activate, **no** Firefly |

- **`simulate_plan` (discovery open question):** **Ephemeral overlay projection** — read-only what-if without persisting plans:
  ```json
  {
    "type": "object",
    "properties": {
      "template": { "enum": ["leasing","savings_mode","house_purchase","custom"] },
      "plan_id": { "type": "string", "format": "uuid" },
      "version_number": { "type": "integer", "minimum": 1, "maximum": 3 },
      "adjustments": { "type": "array", "items": { "$ref": "#/PlanAdjustmentDraft" } }
    }
  }
  ```
  - **Resolution order:**
    1. If `plan_id` (+ optional `version_number`) → load existing version adjustments → `PlanService::project_readonly(version_id)` → return compare metrics (month-end balance, monthly delta sum, viability flag)
    2. Else if `template` → apply template defaults (R-0015: leasing +€300/mo, etc.) as **draft adjustments** → project without DB write
    3. Else if active plan exists → project active latest version (affordability against current scenario)
    4. Else → return `{ "error": "no_plan_context", "hint": "specify template or plan_id" }`
  - **Affordability example ("Kann ich mir ein Leasing Auto leisten?"):** model calls `simulate_plan({ "template": "leasing" })` → returns projected month-end balance vs Ist baseline + `viable: bool` (balance ≥ 0 at month-end per R-0022 plan viability rule)
  - **Cancel savings example:** `simulate_plan({ "template": "savings_mode", "adjustments": [{ "remove_subscription_payee": "Netflix" }] })` → ephemeral removal delta on forecast baseline
  - **Implementation:** add `PlanService::project_ephemeral(draft: PlanDraft) -> PlanProjectionSummary` sharing overlay math from R-0015; distinct from `recompute_version` which persists `plan_computations`
- **Example query → tool mapping (acceptance):**
  | Query (Projectplan) | Primary tool(s) |
  |---------------------|-----------------|
  | Leasing affordability | `simulate_plan` (template=leasing) + optional `get_forecast` |
  | Which subs got more expensive | `get_subscriptions` (include price events summary) |
  | Why over budget this month | `get_budget_status` + `get_transactions` (category aggregates) |
  | Savings if cancel Netflix | `simulate_plan` (savings_mode / remove sub) + `get_subscriptions` |
  | Top spending categories | `get_transactions` (aggregates by category, current month) |
- **Payload limits (discovery open question):** orchestrator **truncates/summarizes** tool JSON before appending to model context:
  - Max serialized size **8 KB** per tool result (config `[ai] max_tool_result_bytes`)
  - Long series → downsample to 30 points + summary stats `{ min, max, latest }`
  - Exceeding limit → replace with `{ "truncated": true, "summary": {...} }`
- **Alternatives considered:**
  - *HTTP self-calls to REST* — rejected (extra latency, auth recursion; use services directly)
  - *simulate_plan only on active plan* — rejected (can't answer template what-ifs without existing plan)
  - *Persist ephemeral simulations* — rejected (scope creep; pollutes plan list)
- **Risks:** new `TransactionsService` scope within US-0006; `get_budget_status` composes logic not yet exposed — must stay in sync with Alert Engine rules (R-0022); ephemeral projection drift if overlay math diverges from persisted recompute — share same `overlay` module; large subscription lists — summarize to top-N by monthly spend

**Linked:** US-0006, US-0002, US-0003, US-0004, US-0005, R-0015, R-0019, R-0021, R-0022, R-0027, R-0028, DEC-0004  
**Confidence:** high  
**Status:** current

---

## R-0032 — Exchange connector REST patterns (Binance, Bybit, Bitunix)

**Date:** 2026-06-01  
**Topic:** Read-only exchange connector auth, endpoint map, rate limits, and MVP product scope for US-0007 start set  
**Query:** HMAC signing differences; balances/positions/trades/transfers/funding endpoints; read-only API key permissions; spot vs futures scope  
**Sources:**
- [Binance Spot Account Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints) — `GET /api/v3/account`, `GET /api/v3/myTrades`
- [Binance USDⓈ-M Futures Account](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Account-Information-V3) — `GET /fapi/v2/account`, position endpoints
- [Binance Wallet deposit/withdraw history](https://developers.binance.com/docs/wallet/change-log) — `GET /sapi/v1/capital/deposit/hisrec`, `GET /sapi/v1/capital/withdraw/history`
- [Bybit V5 Integration Guide](https://bybit-exchange.github.io/docs/v5/guide) — header auth (`X-BAPI-*`), unified account model
- [Bybit Get Wallet Balance](https://bybit-exchange.github.io/docs/v5/account/wallet-balance) — `GET /v5/account/wallet-balance`
- [Bybit Transaction Log](https://bybit-exchange.github.io/docs/v5/account/transaction-log) — funding, transfers, settlement
- [Bitunix Spot User API](https://www.bitunix.com/api-docs/spots/en_us/user/) — `GET /api/spot/v1/user/account`
- [Bitunix Signature](https://www.bitunix.com/api-docs/futures/common/sign.html) — double SHA256 with `nonce` header
- [R-0001](docs/engineering/research.md#r-0001--firefly-iii-rest-api-integration-baseline) — connector trait precedent

**Findings:**
- **Unified connector trait (architecture detail):** one `ExchangeConnector` async trait per exchange implementing `test_connection`, `sync_balances`, `sync_positions`, `sync_trades`, `sync_transfers`, `sync_funding` — normalizes to shared DTOs (`ExchangeHolding`, `ExchangeTrade`, `ExchangeTransfer`, `ExchangeFundingEvent`)
- **Auth patterns:**

  | Exchange | Signature | Required headers / params | Notes |
  |----------|-----------|---------------------------|-------|
  | **Binance** | HMAC SHA256 hex of query string | `X-MBX-APIKEY`, `timestamp`, `signature`, optional `recvWindow` | Separate base URLs: spot `api.binance.com`, futures `fapi.binance.com`, wallet `api.binance.com/sapi` |
  | **Bybit v5** | HMAC SHA256 hex | `X-BAPI-API-KEY`, `X-BAPI-TIMESTAMP`, `X-BAPI-SIGN`, `X-BAPI-RECV-WINDOW` | Unified account preferred — single wallet call covers spot+derivatives |
  | **Bitunix** | Double SHA256 | `api-key`, `nonce` (32 chars), `timestamp`, `sign` | Spot host `openapi.bitunix.com`; futures host `fapi.bitunix.com`; distinct signing from Binance/Bybit |

- **Read-only keys (discovery open question):** **mandatory** — operator creates keys with **Read** permission only; disable withdraw/trade/transfer on key creation:
  - Binance: "Enable Reading" only; IP whitelist recommended for self-hosted
  - Bybit: `readOnly=1` on key creation
  - Bitunix: create key without trade/withdraw permissions per portal
  - Connector MUST NOT call POST/trade/withdraw endpoints (audit + integration test per DEC-0004 pattern)
- **Recommended endpoint map (MVP import surfaces):**

  | Surface | Binance | Bybit v5 | Bitunix |
  |---------|---------|----------|---------|
  | Balances | `GET /api/v3/account?omitZeroBalances=true` | `GET /v5/account/wallet-balance?accountType=UNIFIED` | `GET /api/spot/v1/user/account` |
  | Positions | `GET /fapi/v2/positionRisk` (USD-M) | `GET /v5/position/list?category=linear` (+ inverse if enabled) | Defer futures positions unless operator enables — spot MVP first |
  | Trades | `GET /api/v3/myTrades` per symbol; `GET /fapi/v1/userTrades` per symbol | `GET /v5/execution/list` (7-day default window; paginate with `startTime`/`endTime`) | Spot trade history endpoint per Bitunix order docs (paginate) |
  | Transfers | `GET /sapi/v1/capital/deposit/hisrec`, `GET /sapi/v1/capital/withdraw/history` | `GET /v5/asset/deposit/query-record`, `GET /v5/asset/withdraw/query-record` | Spot deposit/withdraw history if exposed; else transfers from internal ledger only |
  | Funding / income | `GET /fapi/v1/income` (funding fee, realized PnL types) | `GET /v5/account/transaction-log?category=linear` (funding, settlement) | Futures funding via `fapi.bitunix.com` if futures enabled |

- **Incremental sync watermarks:** store per-exchange `last_trade_time`, `last_transfer_time`, `last_funding_time` in `exchange_sync_state`; first run = 90-day backfill (Binance wallet history default window) then overlap 1 day
- **Rate limits:** respect response headers (`X-MBX-USED-WEIGHT`, Bybit `X-Bapi-Limit-Status`); sequential exchange sync inside mutex; exponential backoff on HTTP 429; cap `myTrades` symbol fan-out by syncing symbols with non-zero balance + symbols with open positions only
- **MVP product scope (discovery open question):** **Spot + linear/USDT-M derivatives** where position PnL fields exist — **exclude** Binance Portfolio Margin (`papi`), options, inverse-only advanced modes, on-chain wallets:
  - Binance: spot wallet + USD-M futures account
  - Bybit: UNIFIED account (covers spot + linear per v5 intro)
  - Bitunix: **spot-first** in S0007; futures connector stub behind `enabled_futures=false` default until trade/position endpoints validated in execute
- **Alternatives considered:**
  - *CCXT library* — rejected for MVP (Rust ecosystem thin; hides exchange quirks needed for PnL field mapping; adds dependency audit surface)
  - *WebSocket-only ingestion* — rejected (needs REST backfill for history; WS as future optimization)
  - *Full derivatives parity day one* — rejected (Bitunix doc surface smaller; spot delivers acceptance path for balances + holdings)
- **Risks:** Binance `myTrades` requires per-symbol calls — large account symbol fan-out; Bybit execution list 7-day pagination window needs loop; Bitunix dual-domain + custom signing increases test burden; regional API hosts (Bybit EU/TR/KZ) may need config `base_url` override

**Linked:** US-0007, R-0001, DEC-0004  
**Confidence:** high  
**Status:** current

---

## R-0033 — Portfolio PnL methodology (realized, unrealized, total return)

**Date:** 2026-06-01  
**Topic:** Wealth-tracking PnL computation for Portfolio Engine — exchange-reported vs locally computed cost basis  
**Query:** FIFO/average-cost vs exchange fields; baseline for total return; spot vs futures; tax reporting boundary  
**Sources:**
- [Bybit Wallet Balance](https://bybit-exchange.github.io/docs/v5/account/wallet-balance) — `unrealisedPnl`, `cumRealisedPnl` per coin
- [Bybit Transaction Log](https://bybit-exchange.github.io/docs/v5/account/transaction-log) — funding, cashFlow, settlement semantics
- [Binance Futures Account Information V3](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Account-Information-V3) — `totalUnrealizedProfit`, position-level PnL
- [Binance Income History](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-Income-History) — `GET /fapi/v1/income` types REALIZED_PNL, FUNDING_FEE
- [IRS Rev. Proc. 2024-28 / Notice 2025-07](https://www.irs.gov/pub/irs-drop/n-25-07.pdf) — tax FIFO vs specific ID (inform scope boundary only)
- [R-0021](docs/engineering/research.md#r-0021--net-worth-aggregation-firefly-asset-accounts--snapshots) — snapshot baseline pattern

**Findings:**
- **Scope boundary:** US-0007 PnL is **wealth analytics**, not tax reporting (acceptance + backlog exclude tax). Do **not** implement IRS FIFO/HIFO tax-lot engine in MVP.
- **Recommended hybrid methodology (discovery open question):**

  | Metric | Primary source | Fallback |
  |--------|----------------|----------|
  | **Unrealized gains** | Exchange position fields (`unrealisedPnl`, `totalUnrealizedProfit`, position `unRealizedProfit`) | Mark-to-market: `qty × mark_price − avg_entry × qty` from synced trades |
  | **Realized gains** | Exchange cumulative fields (`cumRealisedPnl`) + income ledger (`fapi/v1/income`, Bybit `transaction-log` TRADE/SETTLEMENT) | Local sum of closed-trade PnL from trade history since watermark |
  | **Total return** | `(current_portfolio_value_eur − baseline_value_eur) / baseline_value_eur` | Baseline = first successful exchange sync snapshot per household |

- **Spot holdings without exchange PnL:** compute **average-cost** locally from imported `myTrades`:
  ```text
  avg_cost = sum(buy_qty × buy_price + fees) / sum(buy_qty)   // per asset per exchange wallet
  unrealized = mark_value − (remaining_qty × avg_cost)
  realized   += sum(sell_proceeds − allocated_cost − fees) on each sell event
  ```
  - Stablecoin treats cost ≈ 1 USD unless cost basis known from trades
- **Derivatives:** prefer exchange-reported unrealized/realized — local reconstruction from fills is error-prone (funding, liquidation, cross-margin)
- **Funding fees:** import as `ExchangeFundingEvent` rows; include in realized PnL subtotal (separate line in UI) but exclude from "spot trade realized" stat card
- **Baseline / total return:** on first successful sync per enabled exchange, persist `portfolio_baselines` row `{ exchange_id, baseline_value_eur, baseline_at }`; total return updates daily via post-sync snapshot — **not** lifetime tax cost basis
- **Cross-exchange aggregation:** sum unrealized/realized **after** EUR conversion (R-0034); expose per-exchange breakdown in API payload
- **Reconciliation policy:** when exchange cumulative realized ≠ local trade sum by >1%, prefer **exchange cumulative** for display and log `pnl_reconciliation_warning` in sync metadata (operator-visible on Sync Status)
- **Alternatives considered:**
  - *Full local FIFO lots* — rejected (tax-grade complexity; out of scope; sparse trade history on first sync)
  - *Trust exchange PnL only* — rejected (Binance spot lacks single PnL field; need local avg-cost)
  - *Mark-only without realized* — rejected (acceptance requires realized + unrealized + total return)
- **Risks:** first-sync 90-day trade backfill misses older cost basis → avg-cost inaccurate until history complete; cross-margin transfers between spot/futures wallets distort per-wallet realized; funding sign conventions differ (Bybit docs: positive funding = receive)

**Linked:** US-0007, R-0021, R-0032, R-0034  
**Confidence:** high  
**Status:** current

---

## R-0034 — FX conversion for crypto → EUR reporting currency

**Date:** 2026-06-01  
**Topic:** Convert mixed exchange balances to EUR headline net worth; extend R-0021 mixed-currency handling  
**Query:** Exchange ticker prices vs external rate API vs manual map; stablecoin treatment; Firefly + crypto combined total  
**Sources:**
- [Frankfurter API](https://frankfurter.dev/) — ECB daily rates; self-hostable Docker; no API key for public endpoint
- [Frankfurter self-host deploy guide](https://frankfurter.dev/deploy/) — `lineofflight/frankfurter` container
- [CoinGecko Portfolio Tracking](https://docs.coingecko.com/docs/portfolio-tracking) — `/simple/price` with `vs_currencies=eur` (optional fallback)
- [Bybit Wallet Balance](https://bybit-exchange.github.io/docs/v5/account/wallet-balance) — native `usdValue` per coin from exchange
- [Binance Spot Ticker Price](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints) — `GET /api/v3/ticker/price`
- [R-0021](docs/engineering/research.md#r-0021--net-worth-aggregation-firefly-asset-accounts--snapshots), [DEC-0021](decisions/DEC-0021.md)

**Findings:**
- **Reporting currency:** EUR default per DEC-0021 — crypto slice MUST convert to EUR for combined headline (acceptance: crypto in net worth)
- **Recommended two-layer FX model:**

  | Asset class | Conversion path | Refresh |
  |-------------|-----------------|---------|
  | **Stablecoins** (USDT, USDC, BUSD, FDUSD) | Treat as USD → EUR via Frankfurter `GET /v2/rate/USD/EUR` | Daily cache (ECB business-day) |
  | **Fiat** (USD, GBP in wallet) | Frankfurter pair to EUR | Daily cache |
  | **Crypto** (BTC, ETH, alts) | **Primary:** exchange quote → USDT/USD → EUR; **Fallback:** Binance `ticker/price` for `{ASSET}USDT` × USDT/EUR | Per sync (mark-to-market) |
  | **Bybit unified coins** | Use response `usdValue` when present × USD/EUR | Per sync |

- **Implementation sketch:**
  ```rust
  // FxService::to_eur(amount, asset, source: ExchangePriceBook) -> Result<EurAmount, FxError>
  // Cache table fx_rates(date, base, quote, rate, provider) — upsert daily for fiat pairs
  ```
- **Firefly accounts (unchanged from R-0021):** non-EUR Firefly balances still use mixed-currency warning for Firefly subtotal; **do not block** crypto EUR conversion — API returns `{ firefly: { mixed_currency, subtotal_native }, crypto: { subtotal_eur, fx_complete }, total_eur }`
- **Incomplete FX handling:** if price missing for an altcoin, exclude from crypto subtotal and set `fx_incomplete=true` + banner listing unpriced assets (extend US-0005 mixed-currency banner pattern)
- **Frankfurter vs CoinGecko (discovery open question):** **Frankfurter primary for fiat/stablecoin** (free, self-hostable, ECB-aligned for EU household); **exchange prices primary for crypto** (matches exchange valuation user sees). CoinGecko optional `[portfolio] price_fallback=coingecko` behind API key env — **defer to post-MVP** unless exchange lacks pair
- **Self-hosted preference:** document optional Frankfurter sidecar in Compose `standard` profile; default public `api.frankfurter.dev` with 24h cache + offline last-good rate
- **Alternatives considered:**
  - *Manual reporting-currency map in TOML* — rejected (operator burden; stale prices)
  - *Firefly primary-currency fields for crypto* — rejected (crypto not in Firefly for MVP)
  - *Live forex tick data* — rejected (ECB daily sufficient for household net worth; aligns with Frankfurter)
- **Risks:** USDT≠USD peg drift (acceptable for wealth view; document ±0.5% caveat); weekend/holiday Frankfurter stale rate; illiquid alts without USDT pair fail conversion; Binance price for asset not held on Bybit may diverge from Bybit mark

**Linked:** US-0007, US-0005, R-0021, R-0026, R-0033, DEC-0021  
**Confidence:** high  
**Status:** current

---

## R-0035 — Exchange API secret storage (self-hosted)

**Date:** 2026-06-01  
**Topic:** Secure storage pattern for exchange API keys — TOML/env vs DB vault vs Docker secrets  
**Query:** Settings UX credential edit; parity with OpenAI/Firefly patterns; acceptance "not transmitted externally"  
**Sources:**
- Existing: `backend/src/config/mod.rs` — `AiConfig.api_key_env`, `FireflyConfig.personal_access_token` + `FIREFLY_PERSONAL_ACCESS_TOKEN` env override
- `backend/tests/ai_assistant_integration.rs` — `config_uses_env_for_api_key_not_toml_field`
- [Bitunix API sign docs](https://www.bitunix.com/api-docs/futures/common/sign.html) — secretKey paired with api-key at creation
- US-0007 acceptance: "API keys stored in self-hosted secrets/config, not transmitted externally"
- [R-0032](docs/engineering/research.md#r-0032--exchange-connector-rest-patterns-binance-bybit-bitunix)

**Findings:**
- **Recommended pattern (discovery open question):** **TOML metadata + env-only secrets** — mirror `AiConfig` / Firefly PAT override; **no encrypted DB vault in MVP**:

  ```toml
  [exchanges]
  enabled = true
  interval_seconds = 3600   # independent of Firefly interval allowed

  [exchanges.binance]
  enabled = true
  api_key_env = "BINANCE_API_KEY"
  api_secret_env = "BINANCE_API_SECRET"
  base_url = "https://api.binance.com"   # optional regional override

  [exchanges.bybit]
  enabled = false
  api_key_env = "BYBIT_API_KEY"
  api_secret_env = "BYBIT_API_SECRET"

  [exchanges.bitunix]
  enabled = false
  api_key_env = "BITUNIX_API_KEY"
  api_secret_env = "BITUNIX_API_SECRET"
  spot_base_url = "https://openapi.bitunix.com"
  ```

  ```yaml
  # docker-compose.yml (operator)
  environment:
    BINANCE_API_KEY: ${BINANCE_API_KEY}
    BINANCE_API_SECRET: ${BINANCE_API_SECRET}
  ```

- **Runtime resolution:** `ExchangeConfig::credentials()` reads env vars named in TOML; missing secret → exchange marked `connection_state=not_configured`; never log secret values; redact in sync error messages
- **Settings UI (discovery open question):** **read-only display** — show enabled/disabled, masked "configured ✓" badge, last test result, link to operator doc for env setup — **mirror US-0006 privacy TOML+restart pattern** (no in-browser secret entry, no localStorage)
- **Test connection:** `POST /api/v1/exchanges/{id}/test` calls read-only balance endpoint server-side; returns latency + permission check (reject keys with withdraw enabled if detectable via `apiRestrictions` / account flags)
- **Transmission boundary:** secrets used only in backend outbound HTTPS to exchange hosts; never returned in API responses, never sent to OpenAI tools, never written to `sync_runs` payload — store `exchange_id` + error code only
- **Alternatives considered:**
  - *Encrypted secrets in PostgreSQL* — rejected (key management complexity; master key rotation; out of scope for self-hosted MVP)
  - *Docker secrets file mount only* — supported as operator option but env vars sufficient with Compose `env_file`
  - *Settings form POST storing keys* — rejected (browser exposure; violates acceptance spirit)
- **Risks:** operator commits secrets to git `.env` — document `.gitignore` + example env template; shared JWT session cannot edit secrets but can trigger test/sync (acceptable); key rotation requires container restart

**Linked:** US-0007, US-0006, R-0032, DEC-0004  
**Confidence:** high  
**Status:** current

---

## R-0036 — Post-sync exchange pipeline & scheduler integration

**Date:** 2026-06-01  
**Topic:** Extend DEC-0028 sync mutex with exchange sync phase before net-worth snapshot; manual trigger; Sync Status UX  
**Query:** Pipeline order vs parallel async; shared vs independent cron; failure semantics  
**Sources:**
- Existing: `backend/src/sync/mod.rs` — phases `sync` → `subscriptions` → `forecast` → `alerts`
- [R-0024](docs/engineering/research.md#r-0024--post-sync-alert-engine-pipeline--net-worth-snapshot-hook) — alerts phase owns net-worth snapshot upsert
- [R-0013](docs/engineering/research.md#r-0013--post-sync-subscription-detection-pipeline--forecast-integration) — inline mutex precedent
- [DEC-0028](decisions/DEC-0028.md) — inline `"alerts"` phase after forecast+plan hook
- US-0007 backlog discovery — exchanges phase before net-worth snapshot

**Findings:**
- **Recommended pipeline order (extends DEC-0028 / R-0024):**
  ```
  1. Firefly sync                 (phase: "sync")
  2. Subscription detection       (phase: "subscriptions")
  3. Forecast recompute           (phase: "forecast")
     └─ inline: active plan refresh (DEC-0023)
  4. Exchange sync                (phase: "exchanges")     ← NEW
  5. Net worth snapshot + alerts  (phase: "alerts")
  ```
- **Why before alerts:** `AlertService::run_post_sync` upsert (R-0021) must include crypto EUR subtotal in `total_eur` and extended `payload.crypto` — exchange data required first
- **Inline in mutex (discovery open question):** **yes** — same `SyncService::execute_run` task; avoids stale wealth between exchange import and snapshot; consistent with DEC-0010
- **Scheduler:** extend Tokio cron with **two triggers** sharing mutex:
  - Firefly interval — existing `[sync] interval_seconds`
  - Exchange interval — `[exchanges] interval_seconds` (default **same** as Firefly; operator may set longer e.g. 3600 vs 900 to respect rate limits)
  - When exchange-only tick fires and Firefly not due → run phases **4–5 only** (`exchanges` → `alerts`) skipping 1–3 if Firefly sync not stale OR run lightweight "exchange-only" path
- **Manual triggers:**
  - Existing `POST /api/v1/sync/trigger` — full pipeline 1–5
  - New `POST /api/v1/sync/exchanges/trigger` — phases 4–5 only (Sync Status "Sync exchanges now")
- **ExchangeService contract:**
  ```rust
  ExchangeService::run_post_sync(run_id) -> ExchangeSyncResult {
    // foreach enabled connector (binance, bybit, bitunix):
    //   sync balances → positions → incremental trades/transfers/funding
    //   upsert exchange_sync_state watermark
    // PortfolioEngine::recompute_pnl(run_id)
  }
  ```
- **Failure semantics:** single exchange failure → mark that exchange `error` in sync metadata; continue other exchanges; alerts phase still runs with partial crypto data + `fx_incomplete` flags; overall run `success_with_warnings` if Firefly+forecast OK
- **Sync Status UI rows:** per exchange — name, status badge, last sync, counts `{ balances, positions, trades, transfers, funding }`, error message (redacted)
- **Defer async queue:** if combined pipeline exceeds ~45s (raised from 30s due to exchange fan-out), log metric and recommend longer exchange interval — do not split in MVP
- **Alternatives considered:**
  - *Parallel async exchange job after mutex* — rejected (snapshot race; headline net worth briefly excludes crypto)
  - *Exchanges after alerts* — rejected (violates acceptance — crypto must be in net worth snapshot)
  - *Fully independent exchange cron outside mutex* — rejected (double snapshot complexity); partial path above satisfies rate-limit need
- **Risks:** mutex duration growth (monitor DEC-0018); exchange-only run without fresh forecast still evaluates scarcity on last forecast — acceptable; Binance symbol fan-out dominates phase 4 latency

**Linked:** US-0007, US-0005, DEC-0010, DEC-0023, DEC-0028, R-0013, R-0024, R-0032, R-0033, R-0034  
**Confidence:** high  
**Status:** current

---

## R-0037 — Portfolio persistence schema & snapshot payload extension

**Date:** 2026-06-01  
**Topic:** Flow DB tables for exchange holdings/trades and `net_worth_snapshots` crypto extension; Grafana Dashboard 4 completion inputs  
**Query:** Separate `portfolio_snapshots` hypertable vs extend R-0021 table; allocation template storage; `get_portfolio` payload  
**Sources:**
- [R-0021](docs/engineering/research.md#r-0021--net-worth-aggregation-firefly-asset-accounts--snapshots) — `net_worth_snapshots` relational daily upsert
- [R-0026](docs/engineering/research.md#r-0026--grafana-dashboard-4-portfolio--wealth-partial-provisioning) — Dashboard 4 partial panels
- [R-0031](docs/engineering/research.md#r-0031--six-tool-registry-mapping--simulate_plan-read-only-contract) — `get_portfolio` wraps WealthService
- [R-0018](docs/engineering/research.md#r-0018--plan-persistence-schema-plans-versions-adjustments-daily-snapshots) — plan adjustments pattern for allocation template

**Findings:**
- **Migration 007 recommended tables:**

  ```sql
  CREATE TABLE exchange_connections (
    id            TEXT PRIMARY KEY,  -- 'binance' | 'bybit' | 'bitunix'
    enabled       BOOLEAN NOT NULL DEFAULT false,
    connection_state TEXT NOT NULL DEFAULT 'not_configured',
    last_sync_at  TIMESTAMPTZ,
    last_error    TEXT,
    payload       JSONB NOT NULL DEFAULT '{}'
  );

  CREATE TABLE exchange_holdings (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange_id   TEXT NOT NULL REFERENCES exchange_connections(id),
    asset         TEXT NOT NULL,
    quantity      NUMERIC(24,8) NOT NULL,
    market_value_eur NUMERIC(18,2),
    unrealized_pnl_eur NUMERIC(18,2),
    avg_cost_eur  NUMERIC(18,8),
    product_type  TEXT NOT NULL DEFAULT 'spot',  -- spot | linear | inverse
    synced_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (exchange_id, asset, product_type)
  );

  CREATE TABLE exchange_trades (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange_id   TEXT NOT NULL,
    external_id   TEXT NOT NULL,
    symbol        TEXT NOT NULL,
    side          TEXT NOT NULL,
    quantity      NUMERIC(24,8) NOT NULL,
    price         NUMERIC(24,8) NOT NULL,
    fee           NUMERIC(18,8),
    fee_asset     TEXT,
    realized_pnl  NUMERIC(18,8),
    executed_at   TIMESTAMPTZ NOT NULL,
    payload       JSONB NOT NULL DEFAULT '{}',
    UNIQUE (exchange_id, external_id)
  );

  CREATE TABLE portfolio_pnl_snapshots (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    snapshot_date     DATE NOT NULL,
    sync_run_id       UUID REFERENCES sync_runs(id),
    realized_pnl_eur  NUMERIC(18,2) NOT NULL DEFAULT 0,
    unrealized_pnl_eur NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_return_pct  NUMERIC(8,4),
    crypto_value_eur  NUMERIC(18,2) NOT NULL DEFAULT 0,
    payload           JSONB NOT NULL DEFAULT '{}',
    UNIQUE (snapshot_date)
  );

  CREATE TABLE fx_rates (
    rate_date   DATE NOT NULL,
    base        TEXT NOT NULL,
    quote       TEXT NOT NULL DEFAULT 'EUR',
    rate        NUMERIC(18,8) NOT NULL,
    provider    TEXT NOT NULL DEFAULT 'frankfurter',
    PRIMARY KEY (rate_date, base, quote)
  );
  ```

- **Extend `net_worth_snapshots` (discovery open question):** **extend existing table** — add columns `crypto_value_eur`, `firefly_value_eur`, `total_return_pct`; expand `payload` JSON with `{ crypto_holdings[], exchanges[], fx_incomplete }` — **no separate hypertable** for MVP (≈365 rows/year sufficient per R-0021)
- **Allocation target template (discovery open question):** extend Plan Engine with new adjustment kind `allocation_target` on existing `plan_adjustments` (R-0018 pattern) — weights `{ etf_traditional_pct, crypto_pct, cash_pct }` summing 100; compare current buckets from wealth API:
  - `etf_traditional` = Firefly asset subtotal EUR
  - `crypto` = crypto subtotal EUR
  - `cash` = Firefly cashWalletAsset + stablecoin holdings
- **Grafana Dashboard 4 (R-0026 completion):** replace placeholder panel SQL to read `net_worth_snapshots.crypto_value_eur`; allocation pie from `payload->'allocation'`; performance time series from `total_return_pct` or combined `total_eur`
- **`get_portfolio` extension (R-0031):** include `crypto_value_eur`, top **5** holdings by value, PnL summary — stay within 8 KB cap via summarization
- **Alternatives considered:**
  - *Separate `portfolio_snapshots` hypertable* — rejected (duplicate daily grain with net_worth; YAGNI)
  - *Dedicated `allocation_targets` table* — rejected for MVP (plan adjustment kind sufficient)
  - *Store raw exchange JSON blobs only* — rejected (need normalized holdings for wealth UI + Grafana SQL)
- **Risks:** `exchange_trades` volume unbounded — retention job 2 years default; payload JSON size for Grafana — keep top-N only; multi-product-type same asset (spot+perp) needs distinct rows

**Linked:** US-0007, US-0004, US-0005, US-0006, R-0018, R-0021, R-0026, R-0031, R-0033, R-0034, R-0036  
**Confidence:** high  
**Status:** current

---

## R-0038 — Ollama OpenAI-compatible API & Compose full profile

**Date:** 2026-06-02  
**Topic:** Ollama `/v1/chat/completions` contract, tool-calling support, default Compose wiring for US-0008  
**Query:** Ollama OpenAI compatibility matrix; default `base_url`; dummy API key; recommended tool-calling models; Compose `full` profile dependency graph  
**Sources:**
- [Ollama — OpenAI compatibility](https://docs.ollama.com/api/openai-compatibility) — `/v1/chat/completions` supports chat, streaming, JSON mode, **tools**; **`tool_choice` not supported** (omit from request)
- [Ollama — Tool calling](https://docs.ollama.com/capabilities/tool-calling) — native `/api/chat` + OpenAI-compat path; parallel + multi-turn agent loops; streaming accumulates `tool_calls` deltas
- [Ollama model search — tool capability](https://ollama.com/search?c=tool) — curated tool-capable models
- Existing: `docker-compose.yml` — `ollama` service on profile `[full]`, port `11434`, volume `ollama_data`
- [R-0005](docs/engineering/research.md#r-0005--docker-compose-multi-service-with-external-postgresql) — profile pattern; no embedded PostgreSQL
- [R-0027](docs/engineering/research.md#r-0027--openai-tool-calling-orchestration-in-axum-async-openai) — orchestrator sends `tool_choice: "auto"` today

**Findings:**
- **Default Ollama base URL (in-compose):** `http://ollama:11434/v1` — trailing `/v1` required; client posts to `{base_url}/chat/completions` (same as existing `OpenAiProvider` hardcoded path pattern)
- **Provider mode mapping:** backlog `provider = "ollama"` is a **preset** of OpenAI-compatible client with default `base_url` above; not a separate HTTP protocol (native `/api/chat` rejected — would fork orchestrator request types per R-0040)
- **Authentication:** Ollama ignores bearer token by default; set `api_key_env = "OLLAMA_API_KEY"` with dummy value `ollama` (or any non-empty string) when SDK/client requires `Authorization` header — **optional for Ollama**, required when operator enables Ollama auth proxy
- **Request compatibility gap:** Ollama OpenAI-compat docs list **`tool_choice` unsupported** — orchestrator must **omit** `tool_choice` when `provider` is `ollama` or `openai_compatible` targeting Ollama; models still receive `tools` array and emit `tool_calls` when capable
- **Recommended operator models (six-tool finance assistant):**

  | Model tag | VRAM (Q4 approx) | Tool reliability | Operator note |
  |-----------|------------------|------------------|---------------|
  | `llama3.1:8b` | ~5.5 GB | Good (dev) | Fast iteration; acceptable for Settings test |
  | `qwen2.5:14b` | ~9.5 GB | Very good (prod default) | Best balance for household hardware |
  | `qwen2.5:7b` | ~5 GB | Good | Minimum GPU path |
  | `mistral-small` | ~15 GB | Very good | Alternative if Qwen unavailable |

  Document in user guide: `docker compose --profile full exec ollama ollama pull qwen2.5:14b` before first chat; match TOML `model` to pulled tag exactly
- **Compose `full` profile (existing + recommended docs/wiring):**
  ```yaml
  # Already present — ollama on profile [full]
  flow-finance-ai:
    profiles: [minimal, standard, full]
    environment:
      AI_PROVIDER: ${AI_PROVIDER:-openai}   # operator override optional
    depends_on:
      firefly-iii:
        condition: service_started
      # Document: when AI_PROVIDER=ollama, operator should add:
      # ollama:
      #   condition: service_started
  ```
  - **Decision (discovery open question):** **do not conditionally inject `depends_on: ollama` in Compose YAML** — Compose cannot branch on env; instead document in operator guide that `provider = "ollama"` requires `--profile full` and manual `depends_on` snippet or always start full stack for local AI
  - **Optional Ollama healthcheck:** `curl -f http://localhost:11434/api/tags` — defer to execute if startup race observed; backend already retries HTTP via `request_timeout_secs`
  - **Backend env for compose:** add `OLLAMA_HOST=http://ollama:11434` only if using Ollama-specific SDK — **not needed** when using OpenAI-compat HTTP client (R-0040)
- **Alternatives considered:**
  - *Native Ollama `/api/chat` client* — rejected (second request/response schema; US-0008 scope is HTTP client swap only)
  - *Embed model pull in backend startup* — rejected (out of scope; operator CLI)
  - *Ollama in `standard` profile* — rejected (GPU/VRAM optional; keep `full` per Projectplan)
- **Risks:** model not pulled → connection errors at chat time; `tool_choice` sent to Ollama may be ignored or error on future versions — omit defensively; CPU-only Ollama latency may exceed `request_timeout_secs=60` on first token — document hardware expectations

**Linked:** US-0008, US-0001, R-0005, R-0027, R-0040, R-0041  
**Confidence:** high  
**Status:** current

---

## R-0039 — LM Studio, LocalAI & vLLM OpenAI-compatible endpoint variance

**Date:** 2026-06-02  
**Topic:** Host-run and gateway local inference servers reachable via `openai_compatible` provider mode  
**Query:** Default ports/base URLs; tool-calling prerequisites; streaming delta variance; auth requirements  
**Sources:**
- [LM Studio — Tool Use (OpenAI compat)](https://lmstudio.ai/docs/developer/openai-compat/tools) — `/v1/chat/completions` + `tools`; streams tool calls via `delta.tool_calls`; dummy API key accepted
- [LM Studio — Local server](https://lmstudio.ai/docs/developer/core/server) — default `http://localhost:1234`; `lms server start --port 1234`
- [LocalAI — OpenAI Functions and Tools](https://localai.io/features/openai-functions/index.html) — drop-in OpenAI API; backend-specific tool parsers (llama.cpp auto, vLLM requires `tool_parser` in model YAML)
- [vLLM — Tool Calling](https://docs.vllm.ai/en/latest/features/tool_calling) — requires server flags `--enable-auto-tool-choice` + `--tool-call-parser <parser>`; supports `tool_choice` auto/required/none
- [DEV — Function calling for local LLMs](https://dev.to/lingdas1/function-calling-for-local-llms-deepseek-qwen-glm-4-langchain-4iac) — local `tool_choice: "required"` often unsupported; streaming+tools less reliable than buffered
- [R-0029](docs/engineering/research.md#r-0029--chat-streaming-sse-with-jwt-auth-dec-0006-gate) — SSE streams tool-call deltas

**Findings:**
- **Unified config mode `openai_compatible`:** operator sets `base_url` to any OpenAI-compat root ending in `/v1`:

  | Server | Typical base URL (host → container) | Default port | Tool calling notes |
  |--------|-------------------------------------|--------------|-------------------|
  | **LM Studio** | `http://host.docker.internal:1234/v1` | 1234 | Tools via OpenAI schema; LM Studio parses model text → `tool_calls`; any non-empty `api_key` |
  | **LocalAI** | `http://localai:8080/v1` or host gateway | 8080 | Tools supported; parser varies by backend — llama.cpp models work OOTB; vLLM backend needs model YAML `tool_parser` |
  | **vLLM** | `http://vllm:8000/v1` | 8000 | **Server must start with** `--enable-auto-tool-choice --tool-call-parser llama3_json` (or family-specific parser); else 400 on tool requests |
  | **Ollama (host)** | `http://host.docker.internal:11434/v1` | 11434 | Same as R-0038; omit `tool_choice` |

  Backend in Docker reaching host LM Studio: reuse `extra_hosts: host.docker.internal:host-gateway` (R-0005 pattern)
- **API key policy (discovery open question):** **`api_key_env` optional for local providers** — resolve as `Option<String>`; send `Authorization: Bearer` only when env var non-empty; LM Studio/vLLM/Ollama accept dummy keys; LocalAI may require token when `API_KEY` enabled — operator sets env accordingly
- **Endpoint variance matrix (orchestrator impact):**

  | Feature | OpenAI | Ollama | LM Studio | LocalAI | vLLM |
  |---------|--------|--------|-----------|---------|------|
  | `tools` array | ✅ | ✅ | ✅ | ✅ | ✅ (with flags) |
  | `tool_choice: auto` | ✅ | ⚠️ omit | ✅ | ✅ | ✅ (with flags) |
  | `tool_choice: required` | ✅ | ❌ | ⚠️ partial | ⚠️ | ✅ (v0.8.3+) |
  | Streaming tool deltas | ✅ | ✅ (accumulate) | ✅ (chunked) | ✅ | ✅ (AsyncLLMEngine) |
  | Parallel tool calls | ✅ | ✅ | ✅ | ✅ | ✅ |

  **Recommendation:** default local path uses **non-streaming tool rounds** inside orchestrator (already `stream: false` for tool loop per R-0027); streaming only for final assistant tokens (R-0029) — reduces LocalAI/LM Studio streaming+tools fragility
- **Model naming:** OpenAI-compat servers expect loaded model id string (LM Studio UI id, vLLM `--model` path, LocalAI gallery name) — expose via TOML `[ai] model` unchanged; test endpoint validates model reachable via `GET /v1/models` when supported (fallback: minimal completion)
- **Alternatives considered:**
  - *Separate provider types per vendor* — rejected for MVP (identical HTTP contract; config differs only in URL + quirks — see R-0040)
  - *Ship vLLM in Compose full profile* — rejected (CUDA-only, heavy ops; document as advanced operator path)
  - *Detect parser from model name in backend* — rejected (operator/server concern; document vLLM flags in user guide)
- **Risks:** vLLM misconfiguration silent until first chat — test endpoint must surface 400 tool-parser errors; LM Studio headless (`lms`) version drift — pin version in user guide; host.docker.internal unreachable on some Linux setups — document explicit host IP fallback (R-0005)

**Linked:** US-0008, R-0027, R-0029, R-0038, R-0040, R-0041  
**Confidence:** high  
**Status:** current

---

## R-0040 — AI provider factory pattern (`AiProvider` HTTP client swap)

**Date:** 2026-06-02  
**Topic:** Extend stub `AiProvider` trait and refactor orchestrator off hardcoded `OpenAiProvider` + OpenAI URL  
**Query:** Factory vs enum match; unified `OpenAiCompatibleProvider` vs separate Ollama type; config schema; parity with `ExchangeService` factory  
**Sources:**
- Existing: `backend/src/ai/provider.rs` — stub `AiProvider` trait; `OpenAiProvider` hardcodes `https://api.openai.com/v1/chat/completions`
- Existing: `backend/src/ai/orchestrator.rs` — takes `&OpenAiProvider` for `complete` / `run_stream`
- Existing: `backend/src/exchanges/service.rs` — `ExchangeService` factory by exchange id (R-0032 precedent)
- Existing: `backend/src/config/mod.rs` — `AiConfig` with `provider`, `model`, `api_key_env` (no `base_url` yet)
- [R-0027](docs/engineering/research.md#r-0027--openai-tool-calling-orchestration-in-axum-async-openai) — Chat Completions + tools contract
- [R-0035](docs/engineering/research.md#r-0035--exchange-api-secret-storage-self-hosted) — TOML metadata + env secrets pattern

**Findings:**
- **Discovery open question — unified vs separate Ollama type:** **single `OpenAiCompatibleProvider` struct** + **`provider` enum presets** — Ollama is not a distinct HTTP stack:
  ```rust
  pub enum AiProviderKind { OpenAi, Ollama, OpenAiCompatible }

  pub struct OpenAiCompatibleProvider {
      client: Client,
      base_url: String,       // e.g. https://api.openai.com/v1 | http://ollama:11434/v1
      api_key: Option<String>,
      pub model: String,
      pub max_completion_tokens: u32,
      omit_tool_choice: bool, // true for Ollama
      label: &'static str,    // "openai" | "ollama" | "openai_compatible"
  }
  ```
  Factory resolves URLs:
  - `openai` → `https://api.openai.com/v1`, `api_key` required from env, `omit_tool_choice = false`
  - `ollama` → `base_url` default `http://ollama:11434/v1`, `api_key` optional, `omit_tool_choice = true`
  - `openai_compatible` → `base_url` from TOML (required), `api_key` optional, `omit_tool_choice = true` (safe default for mixed local endpoints)
- **Extended `AiProvider` trait (stub → real):**
  ```rust
  pub trait AiProvider: Send + Sync {
      fn name(&self) -> &str;
      fn is_configured(&self) -> bool;
      fn is_local(&self) -> bool;
      fn display_label(&self) -> &str;  // "Cloud · OpenAI" | "Local · Ollama"
      async fn chat_completion(&self, req: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProviderError>;
      async fn chat_completion_stream(&self, req: ChatCompletionRequest) -> Result<reqwest::Response, ProviderError>;
  }
  ```
  Move existing `OpenAiProvider` HTTP methods onto trait implementor; delete hardcoded URL constant
- **Factory entry point:**
  ```rust
  pub fn build_provider(config: &AiConfig) -> Result<Arc<dyn AiProvider>, ProviderError> {
      match config.provider_kind() {
          AiProviderKind::OpenAi => { /* require api key */ }
          AiProviderKind::Ollama => { /* default base_url */ }
          AiProviderKind::OpenAiCompatible => { /* require base_url */ }
      }
  }
  ```
  Wire in `AiService` at startup (mirror `ExchangeService`); inject `Arc<dyn AiProvider>` into orchestrator handlers — **orchestrator signatures change from `&OpenAiProvider` to `&dyn AiProvider`**
- **TOML schema extension:**
  ```toml
  [ai]
  provider = "openai"           # openai | ollama | openai_compatible
  base_url = ""                 # required when openai_compatible; ignored for openai; default for ollama
  model = "gpt-4o-mini"         # or qwen2.5:14b / loaded LM Studio id
  api_key_env = "OPENAI_API_KEY" # optional for ollama/openai_compatible local
  # ... existing max_tool_rounds, timeouts unchanged
  ```
  Validation at startup: fail fast with clear log if `openai_compatible` missing `base_url` or `openai` missing API key
- **Chat/orchestrator boundary unchanged:** same `ChatCompletionRequest`, six tools, PrivacyLayer, audit — **HTTP client layer only** (acceptance AC4)
- **Alternatives considered:**
  - *async-openai crate with `with_base_url`* — viable but US-0006 already uses reqwest directly; keep reqwest for minimal diff and stream control
  - *Separate `OllamaProvider` duplicate struct* — rejected (100% duplicate HTTP code)
  - *Runtime provider switching* — rejected (backlog: restart required)
- **Risks:** trait object async methods add one vtable indirection (negligible); mis-typed `provider` string — parse enum with serde alias + startup error; orchestrator refactor touches chat API — keep changes confined to `provider.rs`, `service.rs`, `orchestrator.rs`, `chat.rs`

**Linked:** US-0008, US-0006, R-0027, R-0029, R-0035, R-0038, R-0039  
**Confidence:** high  
**Status:** current

---

## R-0041 — Local model tool-calling reliability & orchestrator fallback

**Date:** 2026-06-02  
**Topic:** Six-tool finance assistant on local models — model selection, request tuning, degraded behavior when tools fail  
**Query:** Recommended Ollama models; behavior when `tool_calls` missing or args malformed; context window vs 8 KB payloads  
**Sources:**
- [Ollama — Tool calling](https://docs.ollama.com/capabilities/tool-calling) — multi-turn agent loop; parallel tools; streaming accumulation
- [Ollama OpenAI compat — unsupported `tool_choice`](https://docs.ollama.com/api/openai-compatibility)
- [DEV — Local LLM function calling comparison](https://dev.to/lingdas1/function-calling-for-local-llms-deepseek-qwen-glm-4-langchain-4iac) — temperature 0.3 for tool selection; ≤5 tools optimal; max-round guard
- Existing: `backend/src/ai/orchestrator.rs` — `max_tool_rounds`, `tool_choice: Some("auto")`, JSON arg parse via tool registry
- [DEC-0035](decisions/DEC-0035.md) — `max_tool_result_bytes = 8192`
- [R-0031](docs/engineering/research.md#r-0031--six-tool-registry-mapping--simulate_plan-read-only-contract) — six tools, summarization

**Findings:**
- **Model selection (discovery open question):** document **operator defaults** — dev: `llama3.1:8b`; production: **`qwen2.5:14b`** (best tool-selection reliability vs VRAM on 12–16 GB GPUs); avoid models without tool-calling tag on ollama.com
- **Request tuning for local providers:**
  - Omit `tool_choice` when `omit_tool_choice = true` (R-0038/R-0040)
  - Optional TOML `[ai] temperature = 0.3` for local providers (new field, default 0.7 OpenAI / 0.3 local) — lower temperature improves deterministic tool name selection
  - Keep exactly **six tools** registered — within optimal ≤8 tool count for local models
  - System prompt addition for local: *"You have access to finance tools. When the user asks about balances, spending, subscriptions, or forecasts, you MUST call the appropriate tool before answering."*
- **When local model omits `tool_calls` (discovery open question):**
  1. If assistant returns **text content** with `finish_reason=stop` → **return text to user** (graceful degradation) + emit SSE `warning` event `{ "code": "no_tool_calls", "hint": "model answered without tools" }` for operator visibility
  2. **Do not** auto-retry with `tool_choice: required` on local (unsupported on Ollama — R-0038)
  3. Optional **single soft retry** (architecture decision): append system nudge *"Use a tool to fetch data."* and re-prompt once — cap at 1 retry to avoid loops; gate behind `[ai] local_tool_nudge_retry = true` default **true** for local only
- **Malformed tool arguments:**
  - Parse `call.function.arguments` with `serde_json::from_str` — on failure, append `role: tool` message with `{ "error": "invalid_arguments", "detail": "..." }` and continue loop (model may self-correct) — same as OpenAI path
  - Unknown tool name → registry allowlist rejects before execute; return structured error to model
- **Context window vs payloads:** local 7B–14B models often **8k–32k** context — six tool schemas ≈2–3k tokens; tool results capped at 8192 bytes (DEC-0035) — **sufficient** if summarization unchanged; warn in user guide when using 8B on CPU with long chat history (client-side thread truncation optional, not US-0008 scope)
- **Streaming + tools:** keep tool rounds **non-streaming** (`stream: false`); stream only final assistant pass (R-0029) — avoids partial `tool_calls` JSON parse failures on LM Studio/Ollama
- **Alternatives considered:**
  - *Hard-fail chat when no tool_calls* — rejected (blocks legitimate conversational replies)
  - *Fallback to OpenAI when local fails* — rejected (violates AC5 privacy boundary)
  - *Reduce tool set for local* — rejected (AC4 requires same six tools)
- **Risks:** local models may hallucinate numbers without tools — system prompt + UI badge "Local model — verify via tools row"; soft retry doubles latency; qwen vs llama family parser differences on vLLM — operator documentation burden

**Linked:** US-0008, US-0006, DEC-0035, R-0027, R-0029, R-0031, R-0038, R-0039, R-0040  
**Confidence:** medium  
**Status:** current

---

## R-0042 — Settings provider status, test endpoint & AC5 network isolation

**Date:** 2026-06-02  
**Topic:** Settings API shape, Test AI provider button, audit provider metadata, E2E proof of no outbound OpenAI when local selected  
**Query:** `provider_configured` response fields; test endpoint contract; Compose vs wiremock isolation strategy; audit row provider name  
**Sources:**
- Existing: `backend/src/api/exchanges.rs` — `POST /api/v1/exchanges/{id}/test` pattern (R-0035)
- Existing: `backend/tests/ai_assistant_integration.rs` — config/env tests; no provider HTTP mock yet
- Existing: `wiremock` dev-dependency in backend (exchange tests)
- US-0008 acceptance AC5: "Chat functionality verified end-to-end with local provider (no external API call when local selected)"
- [R-0030](docs/engineering/research.md#r-0030--ai-tool-audit-log-persistence-migration-006) — `ai_tool_audit.model` column exists; no `provider` column yet
- [R-0035](docs/engineering/research.md#r-0035--exchange-api-secret-storage-self-hosted) — read-only Settings display

**Findings:**
- **Settings GET extension (discovery open question)** — extend existing settings/AI payload (or `GET /api/v1/settings/ai`):
  ```json
  {
    "provider": "ollama",
    "provider_label": "Local · Ollama",
    "base_url": "http://ollama:11434/v1",
    "model": "qwen2.5:14b",
    "is_local": true,
    "provider_configured": true,
    "privacy": { "...": "unchanged" }
  }
  ```
  - `provider_configured`: `true` when factory `build_provider()` succeeds (API key present for OpenAI; base URL reachable optional — false only on config validation failure)
  - `is_local`: `true` for `ollama` and `openai_compatible`; `false` for `openai`
  - Never return API key values; `base_url` safe to expose (internal Docker DNS)
- **Test AI provider endpoint:** `POST /api/v1/ai/test` (JWT required)
  - Body optional `{ "prompt": "ping" }` default `"Reply OK."`
  - Executes **minimal non-streaming** `chat/completions` with **no tools** — validates model reachability + latency
  - Response: `{ "ok": true, "latency_ms": 842, "model": "qwen2.5:14b", "provider": "ollama", "sample": "OK" }` or `{ "ok": false, "error": "connection refused" }`
  - Does **not** write audit row (unlike real chat) — optional `last_test_at` in-memory cache per R-0035 exchange test pattern
- **Audit provider metadata (discovery open question):** add optional column `provider TEXT` to `ai_tool_audit` in migration **008** (or reuse/extend `model` field with prefix — **prefer explicit `provider` column** for operator filters); populate from `AiProvider::name()` on each tool invocation
- **AC5 network isolation verification (discovery open question):** **two-layer strategy:**

  1. **CI/unit — wiremock guard (required):**
     - Register wiremock for `https://api.openai.com` → panic if matched
     - Configure `AiConfig { provider: "ollama", base_url: "http://127.0.0.1:<wiremock_port>/v1", ... }`
     - Run orchestrator `complete()` with mocked Ollama-compat response including `tool_calls`
     - Assert OpenAI mock **zero requests** + local mock **≥1 request**

  2. **Operator E2E — Compose full profile (documented manual/UAT):**
     - `docker compose --profile full up` with `[ai] provider = "ollama"`
     - Pull model; run example query from US-0006 acceptance
     - Optional: run backend container with `network_mode` blocking WAN except `ollama` — **defer automated network namespace test** (fragile in CI); wiremock layer satisfies AC5 automation

  - **Reject** relying solely on manual tcpdump — document as optional operator verification
- **Chat UI provider badge:** map `provider_label` + `is_local` → `Badge` variant (`Local · Ollama` / `Cloud · OpenAI` / `Local · Compatible`); when `provider_configured=false`, disable input + Alert (discovery UX)
- **Alternatives considered:**
  - *Test endpoint runs full six-tool loop* — rejected (slow, flaky, writes audit noise)
  - *DNS-level outbound block in Rust* — rejected (operating system concern; config `base_url` enforcement sufficient)
  - *Skip migration — encode provider in model string* — rejected (breaks operator queries)
- **Risks:** wiremock test doesn't catch misconfigured `base_url` pointing to OpenAI proxy — validate `is_local` paths never default to `api.openai.com`; test endpoint success doesn't guarantee tool calling — document separate tool smoke in user guide

**Linked:** US-0008, US-0006, R-0029, R-0030, R-0035, R-0040  
**Confidence:** high  
**Status:** current

---

## R-0043 — Self-hosted statistical forecasting for US-0009 (discovery)

**Date:** 2026-06-01  
**Topic:** ML-enhanced personal finance forecasts without cloud ML — seasonal models, confidence intervals, explainability  
**Query:** Self-hosted SARIMA/ETS/MSTL options; layered rule baseline + statistical overlay; confidence bands for 6–24 month horizons; operator trust patterns  
**Sources:**
- [Nixtla StatsForecast](https://github.com/Nixtla/statsforecast) — AutoARIMA, AutoETS, MSTL, probabilistic intervals, fast Python/numba
- [NikolasMarkou/cashflow-app](https://github.com/NikolasMarkou/cashflow-app) — layered rules + SARIMA/SARIMAX + explainability JSON for household cashflow
- [mlnjsh/timesage](https://github.com/mlnjsh/timesage) — plain-English SARIMAX diagnostics, seasonality EDA API
- [R-0006](docs/engineering/research.md#r-0006--rule-based-personal-finance-forecast-algorithms-mvp-baseline) — US-0002 baseline boundary
- [R-0007](docs/engineering/research.md#r-0007--timescaledb-hypertable-schema-for-forecast-snapshots) — snapshot storage pattern
- [R-0022](docs/engineering/research.md#r-0022--alert-engine-evaluation-rules-scarcity-budget-drift-plan-viability) — plan viability inputs for risk score

**Findings:**
- **Recommended architecture direction (PO discovery, confirm at `/architecture`):** **Preserve DEC-0007 baseline** as authoritative `model_kind=baseline`; add optional **`model_kind=ml_enhanced`** computation in same sync pass after baseline succeeds — layered overlay pattern (cashflow-app Layer 0 + Layer 1) avoids replacing rule-based trust path
- **Model family (self-hosted, no cloud APIs):** Prefer **StatsForecast** `AutoETS` / `MSTL` (multiple seasonality) on monthly net-cashflow aggregate per account + household aggregate; defer deep learning and external ONNX unless research proves need — household row volume too small for TiRex-style ML residuals
- **Confidence bands:** Use library **prediction intervals** (default 90% → map to p10/p90 for ECharts band); expose `level` in config (90/95)
- **Minimum history gate:** Require **≥12 monthly net-cashflow points** (or ≥365 days daily converted) before ML pass; else `ml_skipped=true` with reason `insufficient_history` — aligns with R-0006 sparse fallback philosophy
- **Seasonal surfacing:** Persist detected periods (e.g. `period=12`, strength) in computation metadata JSON; Monthly tab callout reads metadata — no separate seasonality engine in React
- **Explainability payload:** `model_family`, `seasonal_periods[]`, `backtest_wmape`, `holdout_months`, `low_confidence` — optional collapsible UI; satisfies self-hosted trust constraint
- **Execution options (open for research):**
  1. *Python sidecar* — small FastAPI/stdio service in Compose `full` profile calling StatsForecast — **recommended probe** (ecosystem maturity)
  2. *Rust `linfa` / `smartcore`* — single binary, fewer deps — validate MSTL/ETS parity before commit
  3. *Subprocess CLI* — reject for production (ops fragility)
- **Portfolio forecast:** Separate univariate series on `portfolio_snapshots.total_eur` (US-0007); same StatsForecast path with shorter minimum history (≥8 weekly points)
- **Risk score (deterministic MVP):** Weighted index from (a) count of projected month-end balances &lt; 0 in next 6 months on active plan overlay, (b) min projected balance vs scarcity threshold, (c) optional crypto snapshot volatility percentile — **not** a black-box classifier; maps to AC "risk assessment score for active plan scenarios"
- **Alternatives considered:**
  - *Replace baseline with AutoARIMA only* — rejected (breaks DEC-0007, US-0002 AC, Grafana baseline queries)
  - *Prophet / cloud AutoML* — rejected (heavy deps, cloud-adjacent; out of discovery scope)
  - *DBSCAN / Ntropy ML recurrence* — rejected (R-0009 scope; labeled data absent)
- **Risks:** Python sidecar adds Compose service; MSTL overfits on &lt;24 months data; ML/baseline divergence without Compare UI confuses users; sync mutex latency — benchmark target &lt;30s ML pass on household scale

**Linked:** US-0009, US-0002, US-0004, US-0007, R-0006, R-0007, R-0008, R-0022, R-0044, R-0045, R-0046, R-0047, R-0048, R-0049, R-0050, R-0051  
**Confidence:** high  
**Status:** current

**Research phase update (2026-06-01):** Discovery direction confirmed by R-0044–R-0051. **Execution model:** Python StatsForecast sidecar in Compose `full` profile (mirror Ollama pattern) — not embedded augurs (R-0044). **Schema:** `model_kind` discriminator on `forecast_computations` + nullable band columns on existing hypertables (R-0049) — not separate ML tables. **Seasonal:** AutoETS default; MSTL when ≥24 monthly points (R-0045). **History gate:** ≥12 monthly net-cashflow points (unchanged); portfolio ≥8 weekly snapshots (R-0047). **Risk:** deterministic weighted index, not ML classifier (R-0048). **Sync:** ML pass after baseline success inside `"forecast"` mutex sub-phase; plan hook remains baseline-only per DEC-0023 (R-0050). See linked entries for architecture handoff.

---

## R-0044 — StatsForecast sidecar vs Rust augurs execution model

**Date:** 2026-06-01  
**Topic:** Self-hosted ML forecast execution — Python StatsForecast sidecar, embedded Rust augurs, or subprocess  
**Query:** Compose footprint, latency, seasonal model parity, cross-validation/backtest availability for US-0009  
**Sources:**
- [Nixtla StatsForecast](https://github.com/Nixtla/statsforecast) — AutoETS, MSTL, numba-accelerated `forecast()` + `cross_validation()`
- [StatsForecast uncertainty intervals](https://nixtlaverse.nixtla.io/statsforecast/docs/tutorials/uncertaintyintervals.html) — `level=[90]` → lo/hi columns
- [augurs (Grafana)](https://github.com/grafana/augurs) — Rust port of statsforecast; API still evolving
- [augurs-ets crate docs](https://docs.rs/augurs-ets/latest/augurs_ets/) — "Seasonal models are not yet implemented"
- [R-0043](docs/engineering/research.md#r-0043--self-hosted-statistical-forecasting-for-us-0009-discovery) — layered baseline + overlay
- Existing: `docker-compose.yml` Ollama `full` profile pattern (US-0008 / R-0038)

**Findings:**
- **Recommended for architecture:** **Python StatsForecast sidecar** — small FastAPI service (`stats-forecast`) in Compose **`full` profile only**; backend calls `POST /v1/forecast` over internal HTTP with 60s timeout; **disabled by default** when sidecar absent (`[forecast_ml] enabled = false` in TOML)
- **Sidecar contract (MVP):**
  ```json
  POST /v1/forecast
  { "series_id": "household", "freq": "MS", "points": [{"ds":"2024-01-01","y":1200.0}, ...],
    "horizon": 24, "level": [90], "model": "auto" }
  → { "model_family": "AutoETS", "seasonal_periods": [12], "forecasts": [{"ds","y","y_lo","y_hi"}],
      "backtest_wmape": 0.18, "low_confidence": false }
  ```
- **Image footprint:** `python:3.11-slim` + `statsforecast` + `numba` ≈ **180–250 MB** pulled; **~80–120 MB RSS** at idle; single-series household forecast **<2s** warm (numba JIT first call ~5s — acceptable inside sync mutex once per sync)
- **Compose wiring (mirror Ollama):**
  - Service `stats-forecast` with `profiles: [full]`; backend env `STATS_FORECAST_URL=http://stats-forecast:8090`
  - No conditional `depends_on` — document `--profile full` + TOML `[forecast_ml] enabled = true` (same pattern as R-0038 Ollama)
- **Why not augurs-in-Rust (primary path):**
  - `augurs-ets` documents incomplete seasonal AutoETS and unstable API — AC1 "seasonal patterns detected" needs reliable `season_length=12` ETS/MSTL today
  - No built-in expanding-window `cross_validation()` equivalent for `backtest_wmape` metadata — would need custom holdout code
  - Grafana-maintained but "not official Grafana project" with slower maintenance cadence
- **Why not subprocess CLI:** ops fragility, no healthcheck, harder AC5-style isolation testing — rejected
- **Optional future path:** spike `augurs` 0.10.x behind feature flag once seasonal ETS parity proven — defer to post-US-0009 unless sidecar footprint blocks operators
- **Alternatives considered:**
  - *Embedded augurs in backend binary* — rejected for US-0009 (seasonal gap + compile weight)
  - *MLForecast / deep learning* — rejected (household row volume; MLOps scope creep)
  - *External cloud forecasting APIs* — rejected (backlog explicit out-of-scope)
- **Risks:** Python sidecar adds second runtime in `full` profile; numba JIT cold start on first sync after deploy; sidecar down must degrade gracefully (`ml_skipped`, baseline unaffected)

**Linked:** US-0009, R-0043, R-0038, DEC-0010  
**Confidence:** high  
**Status:** current

---

## R-0045 — Seasonal model selection (AutoETS, MSTL, fallback)

**Date:** 2026-06-01  
**Topic:** Seasonal statistical models for household net-cashflow and monthly decomposition  
**Query:** AutoETS vs MSTL vs rule-based month-of-year factors; minimum history; seasonality metadata for Monthly tab callout  
**Sources:**
- [StatsForecast AutoETS](https://nixtlaverse.nixtla.io/statsforecast/docs/models/autoets.html) — `season_length=12`, probabilistic intervals
- [StatsForecast multiple seasonalities (MSTL)](https://nixtlaverse.nixtla.io/statsforecast/docs/tutorials/multipleseasonalities.html) — `season_length=[12]`, trend via AutoETS/AutoARIMA
- [StatsForecast cross-validation](https://nixtlaverse.nixtla.io/statsforecast/docs/tutorials/crossvalidation.html) — holdout WMAPE for model selection
- [augurs-seasons periodogram](https://docs.augu.rs/) — seasonality detection (sidecar may use StatsForecast STL or `seasonal_strength` heuristic)
- [R-0006](docs/engineering/research.md#r-0006--rule-based-personal-finance-forecast-algorithms-mvp-baseline) — baseline must remain authoritative
- [R-0043](docs/engineering/research.md#r-0043--self-hosted-statistical-forecasting-for-us-0009-discovery)

**Findings:**
- **Input series (per account + household aggregate):**
  1. **Monthly net cashflow** — sum `free_cashflow` from mirror-derived monthly aggregates (same buckets as DEC-0007 monthly view); freq `MS`
  2. **Daily balance path for long-term bands** — ML projects monthly net-cashflow deltas, then **integrates onto baseline daily balance path** (layered overlay — do not replace DEC-0007 day-by-day mechanics)
- **Model selection ladder:**
  | History | Model | Rationale |
  |---------|-------|-----------|
  | <12 monthly points | **skip ML** | `ml_skipped: insufficient_history` (R-0043 gate) |
  | 12–23 monthly points | **AutoETS(`season_length=12`)** | Single seasonality; stable on short household series |
  | ≥24 monthly points + `seasonal_strength ≥ 0.35` | **MSTL(`season_length=[12]`, trend=AutoETS)** | Annual seasonality with robust trend; avoid MSTL on <24 points (overfit risk per cross-validation guidance) |
  | AutoETS/MSTL fit failure | **SeasonalNaive(`season_length=12`)** fallback inside sidecar | Always return a series; set `low_confidence: true` |
- **Seasonal detection metadata (AC1):** persist in `forecast_computations.metadata`:
  ```json
  { "seasonal_periods": [12], "seasonal_strength": 0.42, "model_family": "MSTL+AutoETS",
    "seasonal_detected": true }
  ```
  Monthly tab callout reads `seasonal_detected` + `seasonal_periods` — no client-side seasonality engine
- **Backtest / explainability:** sidecar runs `cross_validation(h=6, n_windows=3, step_size=6)` on last 24 months when available; compute **WMAPE** → `backtest_wmape` in metadata; flag `low_confidence` when WMAPE > 0.35 or n_windows < 2
- **Rule-based month-of-year factors:** rejected as primary — misses trend/level shifts; acceptable only as sidecar internal fallback when ETS diverges (SeasonalNaive)
- **Alternatives considered:**
  - *MSTL always* — rejected (<24 months overfit)
  - *Prophet* — rejected (heavy deps; not statsforecast ecosystem)
  - *Apply seasonality to baseline engine* — rejected (violates DEC-0007 boundary)
- **Risks:** false seasonal detection on sparse/noisy household data; salary timing shifts break fixed period-12 assumption; MSTL adds ~2× sidecar latency vs AutoETS alone

**Linked:** US-0009, R-0043, R-0044, R-0006, DEC-0007  
**Confidence:** high  
**Status:** current

---

## R-0046 — ML confidence bands storage, API, and ECharts mapping

**Date:** 2026-06-01  
**Topic:** p10–p90 confidence bands for 6–24 month ML projections in DB, REST API, React ECharts, Grafana  
**Query:** StatsForecast `level` parameter mapping; hypertable column design; long-term compare UX data shape  
**Sources:**
- [StatsForecast probabilistic forecasting tutorial](https://nixtlaverse.nixtla.io/statsforecast/docs/tutorials/uncertaintyintervals.html) — symmetric prediction intervals at `level=90` → lo-90 / hi-90 columns
- [R-0007](docs/engineering/research.md#r-0007--timescaledb-hypertable-schema-for-forecast-snapshots) — existing hypertable pattern
- [R-0008](docs/engineering/research.md#r-0008--grafana-dashboard-as-code-for-analytics-dashboards) — Dashboard 5 provisioning
- Existing: `/api/v1/forecast/long-term`, `forecast-horizons.json` baseline-only queries

**Findings:**
- **Interval mapping:** StatsForecast `level=[90]` produces **90% prediction intervals** — map to UI labels **p10/p90** (lower/upper bound columns); median forecast → `balance` (point forecast). Document in API that intervals are **symmetric prediction intervals**, not quantile regression — sufficient for AC2 band shading
- **Storage (extends R-0007 — same hypertables, ML computation_id):**
  ```sql
  ALTER TABLE forecast_balance_daily
    ADD COLUMN balance_p10 NUMERIC(18,2),
    ADD COLUMN balance_p90 NUMERIC(18,2);
  -- NULL for baseline model_kind rows; populated for ml_enhanced rows only
  ```
  Monthly ML decomposition bands optional in metadata JSON only (MVP) — long-term AC drives daily band storage
- **API extensions:**
  - `GET /api/v1/forecast/long-term?account_id=&horizon=6|12|24&variant=baseline|ml_enhanced` — default `variant=baseline` (backward compatible)
  - ML response adds `bands: { p10, p90 }` per point + top-level `model_family`, `seasonal_periods`, `backtest_wmape`, `low_confidence`
  - `GET /api/v1/forecast/meta` returns `{ baseline_computation_id, ml_computation_id, ml_status, ml_skipped_reason }`
  - `GET /api/v1/forecast/compare?account_id=&horizon=` — convenience endpoint returning both series + delta at horizon end (Compare AC)
- **React ECharts (Long-term tab):** area series between `balance_p10` and `balance_p90` with line overlay on `balance`; Compare mode dual lines + single band (ML only)
- **`get_forecast` AI tool (US-0006):** add optional `variant` param defaulting `baseline`; include band summary at horizon end when `ml_enhanced` — **no new tool** (display extension only per discovery)
- **Alternatives considered:**
  - *Separate `forecast_ml_daily` hypertable* — rejected (duplicate indexes; same ts/account/computation grain)
  - *Store bands only in JSONB metadata* — rejected (Grafana SQL panels need columnar access)
  - *95% bands* — defer; config `[forecast_ml] interval_level = 90` default matches AC p10–p90 labeling
- **Risks:** symmetric intervals understate tail risk for skewed spend; band width explodes on `low_confidence` series — UI must dim band opacity when flag set

**Linked:** US-0009, US-0002, US-0006, R-0007, R-0008, R-0044, R-0045  
**Confidence:** high  
**Status:** current

---

## R-0047 — Portfolio performance forecast from exchange snapshots

**Date:** 2026-06-01  
**Topic:** 3/6/12 month crypto portfolio EUR projection on `/wealth` when US-0007 data present  
**Query:** Input series selection; minimum history; FX gaps; storage and API shape  
**Sources:**
- [R-0037](docs/engineering/research.md#r-0037--portfolio-persistence-schema--snapshot-payload-extension) — `portfolio_pnl_snapshots.crypto_value_eur`
- [R-0034](docs/engineering/research.md#r-0034--fx-conversion-for-crypto--eur-reporting-currency) — FX incomplete for illiquid alts
- [R-0021](docs/engineering/research.md#r-0021--net-worth-aggregation-firefly-asset-accounts--snapshots) — `net_worth_snapshots` household totals
- [R-0044](docs/engineering/research.md#r-0044--statsforecast-sidecar-vs-rust-augurs-execution-model) — sidecar reuse

**Findings:**
- **Recommended input series:** **`portfolio_pnl_snapshots.crypto_value_eur`** ordered by `snapshot_date` — aligns with Wealth **Crypto tab** scope (not full net worth which mixes Firefly + crypto)
- **Resampling:** upsample to **weekly** (`W-MON`) last-value-held per R-0037 daily upsert cadence; sidecar freq `W`
- **Minimum history gate:** **≥8 weekly points** (~2 months of syncs); else `portfolio_forecast_skipped: insufficient_history` in ML computation metadata — Wealth tab hides outlook row
- **Model:** AutoETS(`season_length=52`) when ≥52 weekly points; else AutoETS non-seasonal or SeasonalNaive — simpler than cashflow MSTL (single univariate EUR series)
- **Horizons:** sidecar `h=13` (≈3mo), `h=26` (≈6mo), `h=52` (≈12mo) weeks; API returns `{ horizons: { "3m": { point, p10, p90 }, ... } }`
- **Storage:**
  ```sql
  CREATE TABLE forecast_portfolio_weekly (
    ts TIMESTAMPTZ NOT NULL,
    computation_id UUID NOT NULL REFERENCES forecast_computations(id) ON DELETE CASCADE,
    value_eur NUMERIC(18,2) NOT NULL,
    value_p10 NUMERIC(18,2),
    value_p90 NUMERIC(18,2)
  );
  SELECT create_hypertable('forecast_portfolio_weekly', 'ts', chunk_time_interval => INTERVAL '30 days', if_not_exists => TRUE);
  ```
  Link via `model_kind=ml_enhanced` computation sharing same `sync_run_id` as baseline
- **FX / completeness:** when `FxService` marks incomplete conversion for >20% of crypto value (R-0034), set `portfolio_forecast_low_confidence: true` and show warning banner on Wealth tab — do not skip forecast entirely
- **Alternatives considered:**
  - *Per-asset forecasts* — rejected (N small series; illiquid alt noise; AC asks portfolio-level outlook)
  - *`net_worth_snapshots.total_eur`* — rejected ( mixes Firefly balances; not Crypto-tab scoped)
  - *Trading signals / rebalancing* — rejected (backlog out-of-scope)
- **Risks:** short exchange history after first connect produces wild bands; PnL methodology changes (R-0033) shift series level — forecast is indicative only

**Linked:** US-0009, US-0007, R-0034, R-0037, R-0044  
**Confidence:** high  
**Status:** current

---

## R-0048 — Deterministic plan-scenario risk score (0–100)

**Date:** 2026-06-01  
**Topic:** Risk assessment score for active plan scenarios — formula, inputs, persistence, UI surfaces  
**Query:** Weighted index vs ML classifier; reuse R-0022 plan_viability; optional crypto volatility proxy  
**Sources:**
- [R-0022](docs/engineering/research.md#r-0022--alert-engine-evaluation-rules-scarcity-budget-drift-plan-viability) — plan viability triggers, scarcity path
- [R-0015](docs/engineering/research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline) — plan overlay on baseline forecast
- [Billcut scenario-based alerts](https://www.billcut.com/blogs/scenario-based-alerts-in-finance-apps-think-ahead/) — proactive risk framing
- [R-0043](docs/engineering/research.md#r-0043--self-hosted-statistical-forecasting-for-us-0009-discovery) — deterministic MVP direction

**Findings:**
- **Recommended:** **Deterministic weighted index** — not ML classifier; fully explainable `components` JSON for Planning UI tooltip and AC4 trust constraint
- **Inputs (active plan latest successful `plan_computation`):**
  1. **Balance stress (45%):** from `plan_daily_cashflow.planned_balance` household aggregate — count projected month-ends in next **6 months** where balance < 0; map: 0 months → 0, 1 → 50, ≥2 → 85; add +15 if any month-end < −€500
  2. **Plan viability (40%):** mirror R-0022 rules without creating alerts — current month-end < 0 → 80; consecutive current+next month-end < 0 → 100; else 0
  3. **Crypto volatility proxy (15%):** when `portfolio_pnl_snapshots` has ≥8 weekly points — CV = std/mean of last 12 weekly `crypto_value_eur` changes; map CV 0→0, 0.15→50, ≥0.30→100; **0 weight component** when exchanges disabled
  4. **Optional ML divergence modifier (±5 capped):** when ML enabled and `ml_enhanced` 6mo p10 balance below `[alerts] scarcity_threshold_eur` while baseline 6mo end above — add +5 to final score (surface in `components.ml_divergence`)
- **Formula:**
  ```
  raw = 0.45*balance_stress + 0.40*plan_viability + 0.15*crypto_volatility + ml_divergence_modifier
  risk_score = clamp(round(raw), 0, 100)
  ```
- **Thresholds for UI badge:** 0–29 Low (green), 30–59 Medium (amber), 60–100 High (red) — match Alert severity vocabulary
- **Persistence:**
  ```sql
  CREATE TABLE plan_risk_scores (
    plan_computation_id UUID PRIMARY KEY REFERENCES plan_computations(id) ON DELETE CASCADE,
    score SMALLINT NOT NULL CHECK (score BETWEEN 0 AND 100),
    band TEXT NOT NULL,  -- low | medium | high
    components JSONB NOT NULL DEFAULT '{}',
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```
- **Compute trigger:** after active plan refresh (DEC-0023 hook) and again after ML pass if ML changes divergence modifier — idempotent upsert by `plan_computation_id`
- **UI:** Planning Scenarios tab badge + Compare column; not a new Alert type (avoid duplicate inbox noise with R-0022 plan_viability alerts)
- **Alternatives considered:**
  - *Random forest / classifier* — rejected (black-box; training data absent)
  - *Reuse alert severity only* — rejected (AC wants 0–100 score on scenarios, not binary alert)
  - *ML forecast bands as primary risk input* — rejected (plan overlay answers "scenario viability" framing per R-0022)
- **Risks:** prorated category targets skew viability on mid-month edits; crypto CV noisy on small portfolios; operators may over-trust score precision — show component breakdown

**Linked:** US-0009, US-0004, US-0005, US-0007, R-0022, R-0015, R-0043, R-0047  
**Confidence:** high  
**Status:** current

---

## R-0049 — Migration 009 schema for ML overlay, bands, portfolio, risk

**Date:** 2026-06-01  
**Topic:** SQLx migration extending forecast/plan persistence for US-0009 without breaking US-0002 queries  
**Query:** `model_kind` discriminator vs separate tables; backward-compatible Grafana/API latest-snapshot pattern  
**Sources:**
- [R-0007](docs/engineering/research.md#r-0007--timescaledb-hypertable-schema-for-forecast-snapshots) — baseline schema
- [R-0046](docs/engineering/research.md#r-0046--ml-confidence-bands-storage-api-and-echarts-mapping) — band columns
- [R-0047](docs/engineering/research.md#r-0047--portfolio-performance-forecast-from-exchange-snapshots) — portfolio hypertable
- [R-0048](docs/engineering/research.md#r-0048--deterministic-plan-scenario-risk-score-0-100) — plan_risk_scores
- Existing: `backend/migrations/002_forecast_hypertables.sql`, `004_plans.sql`

**Findings:**
- **Recommended migration `009_forecast_ml.sql`:**
  ```sql
  -- Discriminator on computation metadata (baseline rows default unchanged behavior)
  ALTER TABLE forecast_computations
    ADD COLUMN IF NOT EXISTS model_kind TEXT NOT NULL DEFAULT 'baseline',
    ADD COLUMN IF NOT EXISTS paired_baseline_id UUID REFERENCES forecast_computations(id);

  CREATE INDEX IF NOT EXISTS idx_forecast_computations_kind_computed
    ON forecast_computations (model_kind, computed_at DESC)
    WHERE status = 'success';

  -- Nullable bands on existing daily hypertable (NULL for baseline)
  ALTER TABLE forecast_balance_daily
    ADD COLUMN IF NOT EXISTS balance_p10 NUMERIC(18,2),
    ADD COLUMN IF NOT EXISTS balance_p90 NUMERIC(18,2);

  -- Portfolio ML weekly series (ml_enhanced computation only)
  CREATE TABLE IF NOT EXISTS forecast_portfolio_weekly ( ... );  -- per R-0047
  SELECT create_hypertable(...);

  CREATE TABLE IF NOT EXISTS plan_risk_scores ( ... );  -- per R-0048
  ```
- **Pairing baseline ↔ ML:** on ML success, set `paired_baseline_id` on ML row to baseline `computation_id` from same `sync_run_id`; API compare resolves pair via `sync_run_id` + kinds
- **Latest-snapshot queries (backward compatible):**
  ```sql
  -- Baseline (existing Grafana default — unchanged)
  SELECT id FROM forecast_computations
  WHERE status='success' AND model_kind='baseline'
  ORDER BY computed_at DESC LIMIT 1;

  -- ML
  SELECT id FROM forecast_computations
  WHERE status='success' AND model_kind='ml_enhanced'
  ORDER BY computed_at DESC LIMIT 1;
  ```
- **Retention (DEC-0011):** retain last **5 successful per model_kind** — delete cascade includes portfolio weekly rows
- **Metadata JSON extensions on `forecast_computations.metadata`:**
  ```json
  { "ml_status": "success|skipped|failed", "ml_skipped_reason": "insufficient_history|sidecar_disabled|sidecar_error",
    "model_family": "AutoETS", "seasonal_periods": [12], "backtest_wmape": 0.21, "low_confidence": false,
    "portfolio_forecast_skipped": false }
  ```
- **Alternatives considered:**
  - *Separate `forecast_ml_*` tables* — rejected (duplicate indexes; Grafana joins harder)
  - *Single computation with embedded variant column on each row* — rejected (breaks DEC-0011 retention and clear baseline authority)
  - *`model_kind` only in metadata JSON* — rejected (Grafana `$forecast_variant` filter needs indexed column)
- **Risks:** migration on large hypertables adds nullable columns — low lock time at household scale; existing tests insert without `model_kind` — default `baseline` preserves compatibility

**Linked:** US-0009, R-0007, R-0046, R-0047, R-0048, DEC-0011, DEC-0008  
**Confidence:** high  
**Status:** current

---

## R-0050 — Sync mutex ML phase integration and history gates

**Date:** 2026-06-01  
**Topic:** Post-baseline ML recompute inside sync lifecycle; phase reporting; failure semantics; latency budget  
**Query:** Extend DEC-0010 mutex vs async job; interaction with DEC-0023 plan hook and DEC-0041 exchange phase  
**Sources:**
- [DEC-0010](decisions/DEC-0010.md) — inline recompute in sync task
- [DEC-0023](decisions/DEC-0023.md) — plan refresh on baseline forecast only
- [DEC-0041](decisions/DEC-0041.md) — exchange phase after forecast
- [R-0024](docs/engineering/research.md#r-0024--post-sync-alert-engine-pipeline--net-worth-snapshot-hook) — pipeline order
- Existing: `backend/src/sync/mod.rs`, `backend/src/forecast/service.rs`

**Findings:**
- **Recommended pipeline order (extends DEC-0010 / R-0024):**
  ```text
  1. sync → 2. subscriptions → 3. forecast (baseline DEC-0007)
       └─ inline: active plan refresh (DEC-0023, baseline computation_id)
     → 4. forecast_ml (NEW sub-phase, same mutex)
       └─ optional: portfolio forecast + risk score refresh
     → 5. exchanges → 6. alerts → clear mutex
  ```
- **Phase reporting:** keep top-level phase `"forecast"` for baseline; set `"forecast_ml"` during step 4 — Sync Status UI shows sub-label "ML forecast…" (optional string in existing phase field)
- **Implementation sketch:**
  ```rust
  let baseline_id = forecast.recompute(run_id, ctx).await?;
  if config.forecast_ml.enabled {
      forecast_ml.recompute(run_id, baseline_id).await?;  // never fails sync
  }
  run_exchanges_and_alerts(run_id, baseline_id).await?;
  ```
- **Failure semantics (mirror baseline):**
  - ML sidecar error → insert `forecast_computations` row `model_kind=ml_enhanced`, `status=failed` OR skip row and set `ml_status=skipped` on baseline metadata — **prefer skip row + metadata** to avoid polluting retention
  - Sync run remains `success` if ingest + baseline forecast succeeded
  - Alerts/scarcity continue using **baseline** computation_id (unchanged)
- **History gates (configurable TOML `[forecast_ml]`):**
  | Gate | Default | Skip reason |
  |------|---------|-------------|
  | Monthly net-cashflow points | ≥12 | `insufficient_history` |
  | Portfolio weekly points | ≥8 | `portfolio_forecast_skipped` |
  | Sidecar reachable | healthcheck OK | `sidecar_unavailable` |
- **Latency budget:** baseline + ML + portfolio **<30s combined** on reference hardware (DEC-0010 deferral threshold); sidecar timeout 60s hard cap; log `duration_ms` per sub-phase
- **TOML defaults:** `[forecast_ml] enabled = false`, `sidecar_url = "http://stats-forecast:8090"`, `min_monthly_points = 12`, `interval_level = 90`
- **Alternatives considered:**
  - *Async Tokio spawn after mutex* — rejected (race with next sync; stale ML/baseline pair)
  - *ML before plan hook* — rejected (plan must bind baseline per DEC-0023)
  - *Separate manual ML trigger only* — rejected (AC6 compare needs fresh pair each sync)
- **Risks:** mutex duration growth when exchanges + ML combined; first-deploy numba JIT stall; operator forgets `full` profile — document in user guide

**Linked:** US-0009, DEC-0010, DEC-0023, DEC-0041, R-0044, R-0049  
**Confidence:** high  
**Status:** current

---

## R-0051 — Grafana Dashboard 5 ML extensions and `$forecast_variant`

**Date:** 2026-06-01  
**Topic:** Extend `forecast-horizons.json` with ML/baseline overlay, confidence band panel, seasonal stat, portfolio row  
**Query:** Template variable pattern; SQL for band time series; uid stability per DEC-0012  
**Sources:**
- [R-0008](docs/engineering/research.md#r-0008--grafana-dashboard-as-code-for-analytics-dashboards) — provisioning pattern, stable uids
- [Grafana PostgreSQL datasource macros](https://grafana.com/docs/grafana/latest/datasources/postgres/query-editor/)
- Existing: `grafana/provisioning/dashboards/analytics/forecast-horizons.json` (uid `forecast-horizons`)
- [R-0046](docs/engineering/research.md#r-0046--ml-confidence-bands-storage-api-and-echarts-mapping), [R-0047](docs/engineering/research.md#r-0047--portfolio-performance-forecast-from-exchange-snapshots)

**Findings:**
- **Dashboard identity:** keep **`uid: forecast-horizons`** unchanged (DEC-0012); bump `version`; add panels as new rows — no second dashboard
- **Template variable `$forecast_variant`:**
  ```json
  { "name": "forecast_variant", "type": "custom",
    "options": [{"text":"Baseline","value":"baseline"}, {"text":"ML Enhanced","value":"ml_enhanced"}],
    "current": {"value":"baseline"} }
  ```
- **Computation subquery (replace all panel computation_id selects):**
  ```sql
  SELECT id FROM forecast_computations
  WHERE status='success' AND model_kind='$forecast_variant'
  ORDER BY computed_at DESC LIMIT 1
  ```
- **New panels (AC5):**
  | Panel | Type | Query notes |
  |-------|------|-------------|
  | Baseline vs ML end balance | stat row | duplicate stat panels side-by-side with fixed variant subqueries OR repeat row |
  | Confidence band path | timeseries | `balance` + `balance_p10` + `balance_p90` where variant=ml_enhanced; hidden when baseline selected |
  | Seasonal detected | stat | `SELECT metadata->>'seasonal_detected' FROM forecast_computations WHERE model_kind='ml_enhanced' ...` |
  | Portfolio 3/6/12 mo | stat row | `forecast_portfolio_weekly` at horizon offsets; show "N/A" when no rows |
  | Risk score (active plan) | stat | join `plan_risk_scores` to latest active plan computation — optional cross-link |
- **Compare overlay panel:** timeseries with two targets — baseline computation (blue) + ml_enhanced computation (orange) — `$forecast_variant` set to `ml_enhanced` enables band fill on ML target only
- **Scarcity threshold:** reuse Dashboard 1 pattern — constant reference line optional on band panel
- **Alternatives considered:**
  - *New Dashboard 6* — rejected (AC5 specifies Dashboard 5 extensions only)
  - *Prometheus export of ML series* — rejected (duplicate storage per R-0008)
  - *Grafana ML plugin* — rejected (self-hosted SQL datasource sufficient)
- **Risks:** empty ML computation when variant=ml_enhanced and ML skipped — panels show "No data" with dashboard description note; metadata JSON path typos break seasonal stat

**Linked:** US-0009, R-0008, R-0046, R-0047, R-0048, DEC-0012  
**Confidence:** high  
**Status:** current

---

## R-0052 — External Compose integration on omniflow Traefik host

**Date:** 2026-05-31  
**Question:** How should Flow Finance AI attach to existing Firefly/Postgres/Traefik on the operator Debian host without duplicate services?

**Findings:**

- Host Firefly stack: `/workdir/firefly/docker-compose.yml`, project `firefly`, container `firefly`, Traefik rule `Host(\`finance.omniflow.cc\`)`, middleware `auth`, network `traefik` (172.20.0.37)
- Shared Postgres: `/workdir/services/docker-compose.yml`, container `postgres`, user `flow`, published `5432`, network `traefik` (172.20.0.20)
- Traefik: `/workdir/networking/docker-compose.yml`, global middleware `auth` via `credentials.passwd`, cert resolver `myresolver`, wildcard `*.omniflow.cc`
- finance_goblin should use `networks.traefik.external: true`, drop bundled `firefly-iii` in external profile, set `FIREFLY_BASE_URL=http://firefly:8080`, `DATABASE_HOST=postgres`
- Port conflict: host `8090` used by `firefly_product_manager` — remap `STATS_FORECAST_PORT` when `full` profile used alongside host stack

**Alternatives considered:**

- *host.docker.internal for DB/Firefly* — rejected on Linux external network; Docker DNS to named containers is reliable on shared `traefik` network
- *Edit host firefly compose in-repo* — out of scope; finance_goblin only documents alignment

**Linked:** US-0010, US-0001, R-0001  
**Confidence:** high  
**Status:** current

---

## R-0053 — US-0010 omniflow external deploy: TimescaleDB, profile guard, Traefik env, Grafana, OIDC, CI

**Date:** 2026-06-01  
**Question:** Resolve US-0010 discovery open questions for attaching Flow Finance AI to shared host `postgres`/`firefly`/`traefik` without duplicate services (extends R-0052).

**Sources:**
- [TimescaleDB — enable on existing PostgreSQL](https://severalnines.com/blog/how-enable-timescaledb-existing-postgresql-database/)
- [TimescaleDB install guide](https://www.tigerdata.com/docs/get-started/choose-your-path/install-timescaledb)
- [Compose profiles spec](https://github.com/compose-spec/compose-spec/blob/main/15-profiles.md)
- [Compose merge `!reset`](https://compose-spec.github.io/compose-spec/13-merge.html)
- [Docker Compose variable interpolation](https://docs.docker.com/compose/how-tos/environment-variables/variable-interpolation/)
- [Traefik community — env in compose labels](https://community.traefik.io/t/environment-variable-access-in-the-docker-compose-file/18802)
- [Docker Compose `config --services` tests](https://github.com/docker/compose/blob/master/tests/acceptance/cli_test.py)
- [R-0004](docs/engineering/research.md#r-0004--rust-axum--sqlx--timescaledb-foundation-patterns), [R-0052](docs/engineering/research.md#r-0052--external-compose-integration-on-omniflow-traefik-host)
- Repo: `backend/migrations/001_initial.sql`, `docker-compose.yml`, `docker-compose.external.yml`, `frontend/src/auth/oidc.ts`, sprint UAT compose patterns (S0001–S0009)

**Findings:**

### 1. TimescaleDB on shared host `postgres`

- Flow Finance AI **requires** TimescaleDB: migration `001_initial.sql` runs `CREATE EXTENSION IF NOT EXISTS timescaledb;` at startup; US-0002+ hypertable migrations fail without it (R-0004).
- Host container is `postgres:latest` (generic image per R-0052) — **extension availability is not guaranteed** until operator verifies on `flow_finance_ai` database.
- **Server-level prerequisite:** TimescaleDB packages + `shared_preload_libraries = 'timescaledb'` in `postgresql.conf` + **Postgres restart** (cannot be satisfied by app migrations alone).
- **Database-level step (operator, once):** connect to `flow_finance_ai` as superuser or role with `CREATE` on database:
  ```sql
  SELECT extname, extversion FROM pg_extension WHERE extname = 'timescaledb';
  -- if empty:
  CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
  ```
- **Failure mode:** backend startup migration error (`extension "timescaledb" is not available` or preload error) — **fail-fast, no partial schema**; `/health` never succeeds until fixed.
- **Operator runbook (execute):** preflight block before `compose up`: (1) create DB/user/grants, (2) run extension check SQL above, (3) only then start `flow-finance-ai`. Document that Firefly DB on same container does **not** imply TimescaleDB on `flow_finance_ai`.
- **Alternatives considered:**
  - *Skip extension in migration 001 for external mode* — rejected (breaks US-0002–US-0009 hypertables; violates released architecture)
  - *Separate TimescaleDB container in finance_goblin* — rejected (AC-1 forbids new `postgres` service in external profile)

### 2. Compose profile guard: `bundled-firefly` vs doc-only

- **Verified (Compose v2.29):** external-only merge lists exactly `flow-finance-ai`, `grafana`; **no** `firefly-iii`/`postgres`.
- **Verified risk:** `--profile minimal --profile external` still starts `firefly-iii` (profiles are union, not exclusive per compose-spec).
- **Recommendation:** **`bundled-firefly` profile split** (architecture/execute):
  - Move `firefly-iii` from `[minimal, standard, full]` → `[bundled-firefly, standard, full]` only.
  - Greenfield dev: `docker compose --profile minimal --profile bundled-firefly up`.
  - Omniflow: `COMPOSE_FILE=docker-compose.yml:docker-compose.external.yml` + `COMPOSE_PROFILES=external` only.
  - Update US-0001 user guide / runbook start commands accordingly.
- **Doc-only guard alone** — insufficient; operators will combine profiles. Runbook must still warn: **never** combine `external` with `minimal`/`standard`/`full`/`bundled-firefly`.
- **CI guard (required):** assert service lists in CI (see §7); optionally fail if `minimal+external` includes `firefly-iii`.

### 3. Env-parameterized Traefik labels (`TRAEFIK_HOST`, `TRAEFIK_MIDDLEWARE`)

- Docker Compose interpolates `${VAR}` in **labels** from shell/`.env` before Traefik reads them (Compose parser, not Traefik container env).
- **Recommended overlay pattern:**
  ```yaml
  - traefik.http.routers.financegnome.rule=Host(`${TRAEFIK_HOST:-financegnome.omniflow.cc}`)
  - traefik.http.routers.financegnome.middlewares=${TRAEFIK_MIDDLEWARE:-auth}
  ```
- **Required vs default:** use **defaults**, not `${VAR:?}` — omniflow production host is stable; overrides support staging without YAML edits. Document in `.env.example`; add commented `TRAEFIK_HOST` / `TRAEFIK_MIDDLEWARE` placeholders.
- Router/service name `financegnome` stays fixed (collision guard vs host `firefly` router per R-0052).
- **`ports: !reset []` on `flow-finance-ai`:** valid on Compose ≥2.24 (compose-spec merge); verified merged config has no published ports on app service.

### 4. Grafana exposure on omniflow

- **Default: internal-only** (discovery MVP) — align with observability hardening pattern: no public Traefik Host rule; reach Grafana via `docker exec`, SSH tunnel, or Traefik-only on `traefik` network without router labels.
- **Gap in partial impl:** external overlay joins `grafana` to `traefik` but **does not** `!reset` Grafana host port — merged config still publishes `${GRAFANA_PORT:-3000}`. Execute should add `grafana.ports: !reset []` in external overlay (optional `expose: ["3000"]` only).
- **Optional public host (operator opt-in):** second router e.g. `grafana-financegnome.omniflow.cc` with same `auth` middleware — gated by `${GRAFANA_TRAEFIK_HOST:-}` (empty = no labels). Do not expose by default (Grafana admin surface + weak base defaults).
- **Alternatives considered:**
  - *Public Grafana behind same Host path* — rejected (path routing conflicts with SPA API)
  - *Always public with auth* — deferred; Traefik basic-auth + Grafana login is double gate but increases attack surface vs internal-only

### 5. OIDC redirect URIs for `https://financegnome.omniflow.cc`

- SPA uses `react-oidc-context` / `oidc-client-ts` (`frontend/src/auth/oidc.ts`): `redirect_uri` defaults to `${window.location.origin}/callback` when `VITE_OIDC_REDIRECT_URI` unset at build — **works for omniflow public URL without rebuild** if IdP allows dynamic origin registration.
- **Operator IdP registration (document-only, out of scope to automate):**
  - Redirect URI: `https://financegnome.omniflow.cc/callback`
  - Post-logout redirect: `https://financegnome.omniflow.cc/`
  - CORS/web origin: `https://financegnome.omniflow.cc`
- **Env updates when OIDC enabled (not required for AC-6 smoke with `AUTH_DEV_BYPASS=true`):**
  - `VITE_OIDC_AUTHORITY`, `VITE_OIDC_CLIENT_ID` (build-time if set; rebuild image if changed)
  - `OIDC_ISSUER_URL`, `OIDC_AUDIENCE=flow-finance-ai` (backend JWT validation)
  - Optional explicit `VITE_OIDC_REDIRECT_URI=https://financegnome.omniflow.cc/callback` when IdP requires exact string match and build-time pinning is preferred
- **Validation script:** optional preflight in execute (`scripts/check-oidc-env.sh`) — warn if `OIDC_ISSUER_URL` set but redirect vars missing; **not a CI gate** unless OIDC profile used.
- **Traefik basic-auth vs OIDC:** orthogonal — host `auth` middleware protects edge; OIDC protects app session when configured.

### 6. Operator smoke test template (AC-6)

Record on Debian host after `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d`:

| Step | Command / check | Pass criterion |
|------|-----------------|----------------|
| Preflight TimescaleDB | `psql -h postgres -U finance -d flow_finance_ai -c "SELECT extversion FROM pg_extension WHERE extname='timescaledb';"` (from host or `docker run --rm --network traefik postgres:16 psql ...`) | Non-null version |
| Firefly reachability | `docker run --rm --network traefik curlimages/curl:latest -sf http://firefly:8080/api/v1/about` | HTTP 200 (PAT not required for about) |
| PAT configured | `docker compose ... exec flow-finance-ai printenv FIREFLY_PERSONAL_ACCESS_TOKEN \| grep -v '^$'` | Non-empty |
| Backend health | `docker run --rm --network traefik curlimages/curl:latest -sf http://flow-finance-ai:8080/health` | `{"status":"ok"}` or project health JSON |
| Traefik TLS + route | `curl -sfI https://financegnome.omniflow.cc/health` (with basic-auth `-u user:pass`) | HTTP 200, valid cert |
| Auth middleware | `curl -sfI https://financegnome.omniflow.cc/` without credentials | HTTP 401 |
| No duplicate Firefly | `docker compose ... ps --services` with external profile only | No `firefly-iii` container in project |
| Migrations applied | backend logs: `migration ... applied` or health OK after fresh DB | No migration panic |

Use placeholder env in CI docs only — **never commit operator credentials**.

### 7. Compose CI / config test patterns

- **Existing pattern (S0001–S0009 UAT):** `docker compose --profile minimal config --services` with placeholder secrets (`DATABASE_PASSWORD`, `FIREFLY_APP_KEY`, `FIREFLY_DB_PASSWORD`, `AUTHENTIK_SECRET_KEY`).
- **Add for US-0010:**
  ```bash
  export DATABASE_PASSWORD=ci FIREFLY_APP_KEY=base64:32RandomCharactersMinimumRequired== \
         FIREFLY_DB_PASSWORD=ci AUTHENTIK_SECRET_KEY=ci
  services=$(docker compose -f docker-compose.yml -f docker-compose.external.yml \
    --profile external config --services | sort)
  test "$services" = "$(printf '%s\n' 'flow-finance-ai' 'grafana' | sort)"
  # regression: minimal unchanged after bundled-firefly split
  docker compose --profile minimal --profile bundled-firefly config --services
  # guard: minimal+external must NOT include firefly-iii once split enforced (or fail CI loudly)
  ```
- Wire into `tests/run-tests.sh` or `scripts/compose-config-check.sh`; no live Docker up required for config gate.
- CI workflow (`.github/workflows/ci.yml`) reads `TEST_COMMAND` from runbook — extend runbook, not duplicate workflow logic (DEC precedent).

**Alternatives considered (cross-cutting):**
- *Single monolithic compose with env-conditional profiles* — rejected at discovery (overlay keeps Traefik labels out of dev runs)
- *host.docker.internal for postgres/firefly on traefik network* — rejected on Linux (R-0052)

**Risks:**
- Host `postgres:latest` without TimescaleDB packages blocks all releases on omniflow until operator installs extension
- Profile combination (`minimal+external`) starts duplicate Firefly until `bundled-firefly` split + CI guard land
- Grafana host port leak on external overlay until `!reset` added
- OIDC misconfiguration silent when `AUTH_DEV_BYPASS=true` — smoke must document auth-off vs auth-on paths
- `!reset` requires Compose ≥2.24 — document minimum in operator runbook

**Linked:** US-0010, US-0001, R-0004, R-0005, R-0052  
**Confidence:** high  
**Status:** current

---

## R-0054 — Unified financegnome analytics shell (Grafana embed vs React port)

**Date:** 2026-06-02  
**Question:** How should US-0011 deliver one web UI for all charts including existing Grafana dashboards on omniflow?

**Findings:**

- **Current state:** Product charts use React + ECharts + REST API; six Grafana JSON dashboards under `grafana/provisioning/dashboards/` query Postgres directly; only WealthPage links out via `VITE_GRAFANA_URL` (new tab).
- **Grafana embed patterns:** (1) iframe to `/d/{uid}/{slug}` with kiosk/TV or anonymous auth; (2) backend reverse-proxy under same origin as financegnome (avoids third-party cookie/CSP issues with Traefik basic auth); (3) Grafana HTTP API + panel PNG — poor UX for interactive dashboards.
- **Auth on omniflow:** Traefik `auth` middleware protects financegnome; internal Grafana has no public route by default (DEC-0056). Same-origin proxy from `flow-finance-ai` to `http://grafana:3000` lets the browser send one auth context; iframe to internal URL without proxy fails from user browser.
- **Full React port:** Duplicates US-0002–US-0009 Grafana investment; high effort; better as phased follow-up per dashboard.
- **Future charts:** Document **React-first** for new features; Grafana embed only for ops/SQL-heavy panels until migrated.

**Alternatives considered:**

- *Require `GRAFANA_TRAEFIK_HOST` public URL + iframe* — rejected as default (extra host, double auth friction).
- *Deprecate Grafana in US-0011* — rejected (scope explosion).

**Recommendation for discovery/architecture:** **Option A + C** — same-origin proxy + `/analytics/*` routes embedding all six dashboards; future charts via React component pattern; keep Grafana container as query engine.

**Linked:** US-0011, US-0010, DEC-0012, DEC-0056, R-0052, R-0053, R-0056  
**Confidence:** medium  
**Status:** current

---

## R-0055 — Auto-provision `flow_finance_ai` database on external Postgres startup

**Date:** 2026-06-02  
**Question:** How should US-0012 create the application database automatically on first start without violating US-0010 external-Postgres contract?

**Findings:**

- PostgreSQL has no connection to a non-existent database; bootstrap must use maintenance DB (`postgres` or `template1`) with credentials that can `CREATE DATABASE`.
- Idempotent pattern: `SELECT 1 FROM pg_database WHERE datname = $1` → `CREATE DATABASE` only when absent (portable across PG versions).
- App role on shared homelab `postgres` often **lacks** `CREATEDB` — recommend optional **`DATABASE_BOOTSTRAP_URL`** (admin connection string, env-only, never committed) separate from runtime `DATABASE_*`.
- After DB create, run `CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE` on app DB when extension files exist; migration 001 also calls `CREATE EXTENSION` — align error messages.
- **TimescaleDB host install** remains operator responsibility (R-0053 §1); auto-provision does not install server packages.
- Compose init container (option B) duplicates connection config; in-app bootstrap (option A) matches single `flow-finance-ai` container on external profile.

**Alternatives considered:**

- *Require manual SQL forever* — current US-0010; rejected per operator request.
- *Auto-create DATABASE_USER* — expands scope; defer unless discovery confirms need.

**Recommendation:** In-app pre-migration bootstrap (A) + optional `DATABASE_BOOTSTRAP_URL`; fail-closed with structured reason codes; integration test with ephemeral Postgres.

**Linked:** US-0012, US-0010, R-0053, R-0004  
**Confidence:** high  
**Status:** current

---

## R-0056 — US-0011 Grafana embed proxy (auth, CSP, subpath, WebSocket, Traefik)

**Date:** 2026-06-02  
**Question:** How should US-0011 resolve discovery open questions for same-origin Grafana embed via `flow-finance-ai` at `/analytics/grafana/` on omniflow (Traefik `auth`, optional OIDC)?

**Sources:**
- [Grafana auth proxy (v11.4)](https://grafana.com/docs/grafana/v11.4/setup-grafana/configure-security/configure-authentication/auth-proxy/)
- [Grafana security overview — anonymous access implications](https://grafana.com/docs/grafana/latest/setup-grafana/configure-security/)
- [Run Grafana behind a reverse proxy](https://grafana.com/tutorials/run-grafana-behind-a-proxy/)
- [Grafana Live / WebSocket behind proxy](https://grafana.com/docs/grafana/latest/setup-grafana/set-up-grafana-live/)
- [Grafana `serve_from_sub_path` redirect behavior (PR #66724)](https://github.com/grafana/grafana/pull/66724)
- [axum-reverse-proxy WebSocket support](https://docs.rs/axum-reverse-proxy/latest/axum_reverse_proxy/)
- R-0054, R-0053, DEC-0056; repo: `backend/src/lib.rs` (`build_router`), `backend/src/auth/mod.rs` (`require_auth`), `docker-compose.yml` (Grafana 11.0.0)

### 1. Grafana anonymous auth vs auth-proxy vs service token

| Mode | Mechanism | Fit for US-0011 |
|------|-----------|-----------------|
| **Anonymous viewer** | `GF_AUTH_ANONYMOUS_ENABLED=true`, `GF_AUTH_ANONYMOUS_ORG_ROLE=Viewer`, `GF_SECURITY_ALLOW_EMBEDDING=true` | **Recommended MVP** — Grafana reachable only on Docker `traefik` network + same-origin proxy; edge Traefik `auth` (+ optional SPA OIDC) is the trust boundary |
| **Auth proxy** | Reverse proxy injects `X-WEBAUTH-USER` (or configured header); Grafana `[auth.proxy] enabled=true`; strip inbound `Authorization` to prevent spoofing | **Deferred** — requires mapping financegnome OIDC subject → proxy header on every proxied request; `enable_login_token` helps after first `/login` but adds complexity vs anonymous behind closed network |
| **Service account / API token in iframe URL** | Token visible in browser network tab and `src` | **Rejected** — leakage risk; tokens are for API/PNG export, not interactive kiosk GUI |

**Anonymous tradeoffs (Grafana docs):** anyone who can reach Grafana gets Viewer API access (list dashboards, run queries). Mitigated when Grafana has **no public route** (DEC-0056) and proxy is same-origin behind Traefik basic auth. Anonymous org must match provisioned dashboards (default org 1).

**Auth-proxy tradeoffs:** per-user Grafana identity and audit; requires trusted proxy to set headers and **never** expose Grafana directly. Headers-only auth fails for static assets unless `enable_login_token=true` sets a session cookie after first validated request.

**Recommendation:** Anonymous Viewer + `GF_USERS_ALLOW_SIGN_UP=false` (existing) for execute; escalate to auth-proxy only if QA finds insufficient Viewer isolation (DEC-0057 decision gate).

### 2. CSP / X-Frame-Options for reverse-proxy embed

- Grafana 11 sets **`X-Frame-Options: deny`** unless **`GF_SECURITY_ALLOW_EMBEDDING=true`** ([security settings](https://grafana.com/docs/grafana/latest/setup-grafana/configure-security/)).
- **Same-origin iframe** (`/analytics/portfolio` → iframe `src=/analytics/grafana/d/...`): parent and child share origin — SPA CSP needs **`frame-src 'self'`** (or omit restrictive `frame-src`); no third-party Grafana host in default build.
- **Proxied responses:** backend should **remove or replace** upstream `X-Frame-Options: deny` and narrow any Grafana `Content-Security-Policy` `frame-ancestors` that block `'self'`. Do **not** rely on Traefik to strip Grafana headers when Grafana is internal-only (no Traefik router on Grafana).
- **Cross-origin embed (rejected default):** would require `frame-ancestors` on Grafana Traefik middleware + `GF_SECURITY_COOKIE_SAMESITE=none` + HTTPS — unnecessary when proxy is same-origin (R-0054).
- **OIDC login in iframe:** not required — Grafana runs anonymous behind proxy; SPA OIDC protects shell routes, not iframe sub-resources individually.

**Sample headers (execute targets):**

| Layer | Header | Value |
|-------|--------|-------|
| SPA (optional hardening) | `Content-Security-Policy` | `frame-src 'self';` … |
| Proxied Grafana response | `X-Frame-Options` | omit or `SAMEORIGIN` |
| Proxied Grafana response | `Content-Security-Policy` | omit `frame-ancestors 'none'` from upstream |

### 3. `GF_SERVER_ROOT_URL` / `GF_SERVER_SERVE_FROM_SUB_PATH`

Two valid reverse-proxy patterns ([Grafana proxy tutorial](https://grafana.com/tutorials/run-grafana-behind-a-proxy/)):

| Pattern | Proxy behavior | Grafana config |
|---------|----------------|----------------|
| **A — Prefix strip (recommended)** | `/analytics/grafana/*` → strip prefix → `http://grafana:3000/*` | `serve_from_sub_path=false` (default); optional `GF_SERVER_ROOT_URL=https://financegnome.omniflow.cc/analytics/grafana/` for generated links |
| **B — Subpath serve** | Forward full path; no rewrite | `GF_SERVER_ROOT_URL=.../analytics/grafana/`, `GF_SERVER_SERVE_FROM_SUB_PATH=true` |

**Findings:**
- Pattern **A** matches discovery proxy contract; avoids Grafana 10+ redirect loops when proxy rewrite and `serve_from_sub_path=true` are both enabled ([issue #70110](https://github.com/grafana/grafana/issues/70110), [PR #66724](https://github.com/grafana/grafana/pull/66724)).
- Pattern **B** is for proxies that **do not** rewrite URLs; increases breakage risk for `/api/live`, static assets, and provisioned dashboard links.
- **`GF_SERVER_ROOT_URL`** should reflect the **browser-visible** URL (public origin + `/analytics/grafana/`) if set; mis-set `root_url` breaks OAuth callbacks (not used in anonymous MVP) and some panel links.

**Recommendation:** Pattern **A** — root upstream proxy with prefix strip; **do not** enable `GF_SERVER_SERVE_FROM_SUB_PATH` for MVP.

### 4. WebSocket through Axum reverse proxy (Grafana Live)

- Grafana Live multiplexes panel subscriptions over **one WebSocket per browser tab** at **`/api/live/`** (same HTTP port) ([Grafana Live docs](https://grafana.com/docs/grafana/latest/setup-grafana/set-up-grafana-live/)).
- Reverse proxy must use **HTTP/1.1**, forward **`Upgrade`** and **`Connection`** headers, and proxy **`/api/live/`** (and prefixed equivalent **`/analytics/grafana/api/live/`** after strip → `/api/live/` upstream).
- Nginx reference pattern uses `$connection_upgrade` map; Axum needs explicit upgrade handling — **`axum-reverse-proxy`** crate documents automatic WebSocket upgrade and bidirectional forwarding ([docs.rs](https://docs.rs/axum-reverse-proxy/latest/axum_reverse_proxy/)); current `flow-finance-ai` has no reverse-proxy dependency (`backend/Cargo.toml`).
- **Alternative:** hyper/tower manual upgrade handler — more control, more code.
- Corporate proxies may strip upgrade headers; same-origin HTTPS through Traefik reduces this vs cross-origin embed.

**Recommendation:** Implement proxy with WebSocket upgrade support; QA smoke must confirm live panel refresh (not just static PNG). Mount proxy **before** SPA fallback in `build_router`; route **outside** `/api/v1` JWT middleware.

### 5. Traefik basic auth + same-origin iframe

- Omniflow uses Traefik **`auth`** basic-auth middleware on **`financegnome.omniflow.cc`** router (R-0053 §3); applies to all paths on that host including future **`/analytics/grafana/`**.
- **Same host + same origin:** browser stores basic-auth credentials per origin; iframe navigations to `/analytics/grafana/...` reuse the parent tab's auth context — **no second login** for same-origin embed (contrast: separate subdomains require separate auth realms per Traefik community guidance).
- **OIDC vs basic auth:** orthogonal layers — Traefik basic auth at edge; SPA OIDC via `react-oidc-context` for API `/api/v1/*`; Grafana proxy should **not** require Bearer JWT (iframe cannot attach SPA Authorization header to sub-resource loads automatically).
- **Risk:** nested auth prompts if iframe pointed at a **different host** (e.g. public `GRAFANA_TRAEFIK_HOST`) — reinforces R-0054 rejection of public Grafana iframe default.

**Recommendation:** Single host UX — all embed traffic via `flow-finance-ai` proxy on financegnome origin; keep Grafana internal-only (DEC-0056).

### Discovery open questions — resolution

| Question | Resolution |
|----------|------------|
| Anonymous vs auth-proxy with OIDC/Traefik? | **Anonymous Viewer** behind internal Grafana + same-origin proxy + Traefik `auth`; auth-proxy deferred |
| CSP / X-Frame-Options? | `GF_SECURITY_ALLOW_EMBEDDING=true`; proxy strips deny headers; SPA `frame-src 'self'` |
| Subpath serve? | **Prefix strip**, `serve_from_sub_path=false`; optional `GF_SERVER_ROOT_URL` for link correctness |
| WebSocket live refresh? | Forward upgrade on `/api/live/` through proxy; verify in QA |
| Theme/kiosk? | **`?kiosk=tv`** hides chrome (discovery); shell theme match is cosmetic — execute smoke only |
| Traefik basic auth + iframe? | Same-origin embed reuses edge basic auth; no Grafana public host |

**Alternatives considered:**
- *Public `GRAFANA_TRAEFIK_HOST` + cross-origin iframe* — rejected (double auth, CSP/cookie complexity)
- *Full React port* — out of scope (R-0054)
- *JWT-protected Grafana proxy under `/api/v1`* — rejected (iframe cannot send Bearer on asset loads)

**Recommendation for architecture:** Accept R-0054 proxy contract with R-0056 refinements — aligns with **DEC-0057** draft elements (anonymous Grafana env, prefix strip, framing rewrite, proxy outside JWT stack).

**Risks:**
- Anonymous Grafana + misconfigured edge exposes Viewer API to anyone passing Traefik auth
- WebSocket proxy gaps cause stale panels without obvious errors
- Broken asset links if `GF_SERVER_ROOT_URL` wrong after execute
- Auth-proxy escalation if Viewer role too permissive for org data

**Linked:** US-0011, US-0010, DEC-0056, DEC-0057, R-0054, R-0053, R-0052  
**Confidence:** high  
**Status:** current

---

## R-0057 — Firefly III API docs discovery (post-BUG-0001)

**Date:** 2026-06-04  
**Topic:** Operator-reported Firefly sync failure on omniflow — API contract refresh for connector triage  
**Query:** Firefly III OpenAPI / REST authentication and sync endpoints for read-only connector  
**Sources:**
- [Firefly III API documentation (OpenAPI)](https://api-docs.firefly-iii.org/)
- [R-0001](docs/engineering/research.md#r-0001--firefly-iii-rest-api-integration-baseline)
- [R-0002](docs/engineering/research.md#r-0002--firefly-iii-pagination--incremental-sync-strategy)

**Findings:**
- Official docs confirm **`/api/v1/`** JSON REST surface; PAT auth via `Authorization: Bearer` (aligns with US-0001 connector).
- Sync triage on external profile should verify in-network `FIREFLY_BASE_URL` (e.g. `http://firefly:8080`) resolves on Docker `traefik` network and PAT is non-empty — without reading operator secrets.
- Pagination/date-filter patterns unchanged from R-0002; sync failures may be connectivity, auth, or misrouted frontend `/api/v1/sync/*` (404) rather than API version drift.
- Discovery should capture Firefly HTTP status from backend sync logs and distinguish connector errors vs SPA/Traefik 404.

**Recommendation:** Use OpenAPI docs for endpoint checklist during BUG-0002 sub-defect C discovery; no standalone `/research` phase unless connector returns unexpected schema/version errors.

**Linked:** BUG-0002, US-0001, US-0010, R-0001, R-0002  
**Confidence:** high  
**Status:** current

---

## R-0058 — Bitunix futures API auth vs connector implementation

**Date:** 2026-06-05  
**Topic:** BUG-0003 sub-defect G — Bitunix test 400 and auth/URL mismatch between shipped connector and official futures API  
**Query:** Bitunix REST primary domain, required auth headers, signing contract vs `BitunixConnector` spot implementation  
**Sources:**
- [Bitunix futures API introduction](https://www.bitunix.com/api-docs/futures/common/introduction.html) — primary domain `https://fapi.bitunix.com`; headers `api-key`, `nonce` (32 chars), `timestamp` (ms), `sign`; `Content-Type: application/json`
- [Bitunix signature (futures)](https://www.bitunix.com/api-docs/futures/common/sign.html) — canonical signing for private REST
- [R-0032](docs/engineering/research.md#r-0032--exchange-connector-rest-patterns-binance-bybit-bitunix) — prior spot-first baseline (`openapi.bitunix.com`, query `sign`)
- `backend/src/exchanges/bitunix.rs`, `backend/src/exchanges/service.rs`, `backend/config/default.toml`

**Findings:**
- **Official futures contract (2026 docs):** Private REST uses **`https://fapi.bitunix.com`** with **header-based** authentication (`api-key`, `nonce`, `timestamp`, `sign`) on all requests; POST bodies JSON (`Content-Type: application/json`).
- **Shipped connector (US-0007 / R-0032):** Spot MVP uses **`https://openapi.bitunix.com`** (default `spot_base_url`), builds signed GET with query string `timestamp` + **`sign` query parameter** (double SHA256 per `bitunix_sign`), header `api-key` only — aligns with spot user API docs, **not** futures header contract.
- **BUG-0003 G — two failure modes:**
  1. **Registry:** `ExchangeService::new` pushes `BitunixConnector` only when `config.bitunix.enabled` (TOML), not `effective_enabled()` — credentials + Q0008 DB mirror may still leave runtime map without `bitunix` → API **400** `unknown exchange: bitunix` before HTTP call.
  2. **Auth/URL:** Even with connector registered, spot signing against `openapi.bitunix.com` may fail if operator keys are futures-scoped or futures endpoints required for test — discovery should capture HTTP status/body from test path and compare against futures sign spec.
- **Recommendation for discovery:**
  - **G1:** Register connectors using `effective_enabled()` (parity with `mirror_enabled_at_startup` / settings view).
  - **G2:** If test still fails after G1, spike futures header-auth client against `fapi.bitunix.com` per futures sign doc; keep spot path for balance sync unless product scope expands.
- **Alternatives considered:**
  - *Futures-only rewrite day one* — defer unless spot test proves keys invalid on spot host.
  - *CCXT* — still rejected (R-0032).
- **H linkage:** Grafana SQL failures may resolve when `DATABASE_HOST=postgres` (sub-defect F) — do not conflate with Bitunix auth until F verified fixed.

**Linked:** BUG-0003 (G1 DONE Q0009), BUG-0005, US-0007, R-0032, Q0008 E1  
**Confidence:** high (docs); medium (operator key scope on omniflow)  
**Status:** current — G1 (`effective_enabled` registry) fulfilled by BUG-0003/Q0009; G2 futures header-auth fulfilled by BUG-0005/Q0012 (DEC-0062)

---

## R-0059 — Exchange multi-product sync scope (Bitunix futures)

**Date:** 2026-06-05  
**Topic:** BUG-0005 — operator requirement that exchange sync include futures/margin accounts, not spot-only  
**Query:** Gap between US-0007 spot-first MVP and operator wealth accuracy needs; Bitunix vs Binance/Bybit connector coverage  
**Sources:**
- [R-0032](docs/engineering/research.md#r-0032--exchange-connector-rest-patterns-binance-bybit-bitunix) — spot-first Bitunix baseline, futures deferred behind `enabled_futures`
- [R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation) — futures auth on `fapi.bitunix.com`
- [Bitunix futures API introduction](https://www.bitunix.com/api-docs/futures/common/introduction.html)
- `backend/src/exchanges/bitunix.rs`, `backend/src/exchanges/binance.rs`, `backend/src/exchanges/bybit.rs`

**Findings:**
- **Operator intent:** Wealth and portfolio must reflect **total exchange exposure** including futures/margin wallets — spot-only sync is insufficient when primary crypto holdings sit in derivatives accounts.
- **Bitunix (primary gap):** `sync_balances` → spot only; `enabled_futures=false` in `default.toml`; `sync_positions`/`sync_funding` return `Ok(vec![])` stubs.
- **Binance (partial):** `sync_balances` spot; `sync_positions` attempts `fapi/v2/account` — further discovery for completeness.
- **Bybit:** Unified wallet via `/v5/account/wallet-balance` may already include derivatives balances but labels all holdings `product_type: "spot"` — labeling/position split may need refinement.
- **Recommendation:** Discovery implements Bitunix futures client per R-0058; enable multi-product ingestion; aggregate in wealth snapshot; keep read-only keys only.

**Linked:** BUG-0005, US-0007, R-0032, R-0058  
**Confidence:** high  
**Status:** current — fulfilled by BUG-0005/Q0012 (DEC-0062, DEC-0063, DEC-0064); omniflow verify-work rows M/N/O PASS 2026-06-05

**Discovery confirmation (2026-06-05, BUG-0005):**
- **M confirmed:** `sync_balances` spot-only; all holdings `product_type: "spot"`.
- **N confirmed:** `enabled_futures=false` default; `sync_positions`/`sync_funding` empty stubs; spot query-sign ≠ futures header-sign (`bitunix_sign` query param vs `SHA256(nonce+timestamp+api-key+queryParams+body)` digest per [sign doc](https://www.bitunix.com/api-docs/futures/common/sign.html)).
- **O confirmed:** wealth/portfolio consume connector holdings without product_type filter — spot-only input under-reports; live omniflow bitunix `holdings_count: 0`, `enabled_futures: false`, spot test OK.
- **Endpoint map frozen:** account `GET /api/v1/futures/account?marginCoin=USDT`; positions `GET /api/v1/futures/position/get_pending_positions` on `fapi.bitunix.com`.
- **Next:** architecture for auth client split + `enabled_futures` default policy.

---

## R-0060 — AI get_transactions empty aggregates vs mirror sync

**Date:** 2026-06-05  
**Topic:** BUG-0006 — AI Chat `get_transactions` tool appears to see no expenses despite 922 synced mirror transactions  
**Query:** Data path from Firefly sync → mirror `transactions` → `TransactionsService::aggregates` → AI tool → LLM interpretation  
**Sources:**
- Operator report (German AI response, `get_transactions` audit ~23:30:13)
- `backend/src/firefly/mod.rs` `sync_transactions`
- `backend/src/db/mod.rs` `upsert_transaction` (no `category_id` column in INSERT)
- `backend/src/transactions/repository.rs` `aggregates_by_category`
- `backend/src/ai/tools/transactions.rs`, [DEC-0032](decisions/DEC-0032.md) aggregate-only privacy mode

**Findings:**
- **Confirmed code gap:** Mirror ingest never sets `category_id` on `transactions` despite schema column — category aggregates join `categories` on NULL keys.
- **Aggregate semantics:** Outflow computed as `SUM` where `amount < 0`; if Firefly stores positive amounts with type metadata only in JSON payload, outflow sums may be zero while `COUNT(*)` > 0.
- **Tool contract:** `TransactionAggregates` lacks top-level period totals; LLM may interpret empty-looking category arrays as "no transactions".
- **Discovery probes:** Compare sync entity count vs `COUNT(*)` for AI query window; sample payload for category + amount fields; audit log tool args (`period_start`, `period_end`).

**Linked:** BUG-0006, US-0006, US-0001, BUG-0004  
**Confidence:** high (category_id gap); medium (date window / amount sign — needs runtime probe)  
**Status:** current

---

## R-0061 — Post-sync analytics pipeline empty data paths

**Date:** 2026-06-05  
**Topic:** BUG-0004 — Stuck exchange sync runs, empty subscriptions UX, Grafana UNION SQL, zero forecast/wealth despite 922 transactions  
**Query:** Root causes for sub-defects I/J/K/L in post-sync pipeline on US-0010 external profile  
**Sources:**
- Code: `backend/src/sync/mod.rs`, `backend/src/recurrence/group.rs`, `grafana/provisioning/dashboards/analytics/portfolio.json`, `backend/src/firefly/mod.rs`, `backend/src/wealth/repository.rs`
- Live probes: `financegnome.omniflow.cc` public curl (sync/runs, subscriptions, wealth, forecast, Grafana ds/query)
- [R-0001](research.md#r-0001--firefly-iii-rest-api-integration-baseline) Firefly account/transaction JSON conventions

**Findings:**
- **I — sync lifecycle:** `RunMode::ExchangesOnly` never persists terminal `sync_runs` status; in-memory mutex clears but DB accumulates `running` rows for `manual_exchanges` / `scheduled_exchanges`.
- **J — subscriptions:** Detection runs only after Full Firefly ingest; `by_payee()` keys on transaction `description` only; live deploy shows 11 **pending** patterns, 0 **confirmed** (operator “empty” likely pre–Full sync or confirmed-only UX).
- **K — Grafana SQL:** PostgreSQL rejects `SELECT … ORDER BY … LIMIT 1 UNION ALL SELECT …` without subquery wrap in portfolio allocation panel.
- **L — forecast/wealth:** Firefly `current_balance` is typically a **string** in JSON; `.as_f64()` → NULL balances → wealth `balance >= 0` filter excludes all asset accounts → zero snapshots and flat forecast series despite populated `forecast_balance_daily` rows.
- **Recommendation:** Architecture freeze I1/K1/L1/L2/J1 contracts; sprint after BUG-0006 ingest fixes if transaction sign/date still affects subscription expense filter.

**Linked:** BUG-0004, US-0002, US-0003, US-0005, US-0011, BUG-0006  
**Confidence:** high  
**Status:** current

---

## R-0062 — Firefly account balance mirror vs forecast/wealth inputs

**Date:** 2026-06-05  
**Topic:** BUG-0010 — Wrong forecast starting balances, zero wealth, ML posture on US-0010 external profile  
**Query:** Firefly `current_balance` API semantics; mirror ingest beyond DEC-0060 parse; negative asset account wealth exclusion; ML disabled metadata gap  
**Sources:**
- Code: `backend/src/firefly/mod.rs`, `backend/src/wealth/repository.rs`, `backend/src/forecast/service.rs`, `backend/src/sync/mod.rs`, `backend/src/api/forecast.rs`, `frontend/src/pages/ForecastPage.tsx`
- Live probes: `financegnome.omniflow.cc` (forecast long-term -25365.78, wealth total 0, meta ML null)
- [R-0001](research.md#r-0001--firefly-iii-rest-api-integration-baseline), [R-0061](research.md#r-0061--post-sync-analytics-pipeline-empty-data-paths)
- [Firefly III Accounts API](https://www.mintlify.com/firefly-iii/firefly-iii/api/accounts) — `current_balance` read-only string; `opening_balance` for initial state only
- [Firefly III finances FAQ](https://docs.firefly-iii.org/references/faq/firefly-iii/finances/) — balance changes only via transactions

**Findings:**
- **AA — forecast math OK, inputs wrong:** Acct 114 mirror `balance = -3395.75`; DEC-0007 projection from negative start + ~6029 EUR/mo outflows → -25365.78 end balance. Not a projection bug.
- **DEC-0060 residual:** Parse works (non-NULL floats present). Zero balances on 115/116 may reflect Firefly source **or** stale account sync — Full Firefly sync required to refresh mirror (`sync_accounts` runs on Full path only).
- **Firefly canonical field:** `attributes.current_balance` (string, read-only) is the sole authoritative balance; **rejected** recompute-from-transactions mirror (cross-cutting, duplicates Firefly ledger).
- **Negative asset accounts:** Valid Firefly state (overdraft/ccAsset). `load_asset_accounts` `COALESCE(balance,0) >= 0` **excludes** acct 114 from wealth — primary checking account invisible.
- **AB snapshots:** Writer runs; totals zero because included accounts sum to 0 — not a snapshot skip.
- **AC ML posture:** `forecast_ml.enabled=false` (DEC-0049) → sync skips ML block entirely → no `record_skip_on_baseline` → meta `ml_skipped_reason: null`. UI treats `!mlAvailable` as "ML skipped" with default reason — **misleading** when ML never configured.
- **Recommendation:** AA1 add structured sync diagnostics + trust `current_balance`; AB1 include negative assets with `is_overdrawn` flag (DEC-0065); AC1 persist/derive `sidecar_disabled` when ML off (DEC-0066); operator Full sync gate before verify.

**Linked:** BUG-0010, BUG-0004, DEC-0060, DEC-0049, DEC-0025, DEC-0065, DEC-0066, US-0013  
**Confidence:** high  
**Status:** current — **fulfilled** by BUG-0010/Q0013 (2026-06-05)

---

## R-0063 — BUG-0012 forecast monthly bucket component attribution

**Date:** 2026-06-05  
**Topic:** BUG-0012 — Income/Fixed monthly buckets always zero; component-level category→bucket attribution  
**Query:** DEC-0007 hybrid projection monthly decomposition; recurring vs rolling residual bucket assignment; Firefly category_id → TOML `[forecast.category_buckets]` alignment; same-day mixed inflow/outflow; subscription category inheritance; test strategy  
**Sources:**
- Code: `backend/src/forecast/project.rs`, `categories.rs`, `recurring.rs`, `rolling.rs`, `types.rs`, `service.rs`, `repository.rs`; `backend/src/recurrence/detect.rs` (`category_ids` on groups); `backend/src/subscriptions/classify.rs` (standing-order category patterns); `backend/config/default.toml`
- Discovery: `handoffs/archive/po-to-tl-pack-20260605-b.md` (discovery-20260605-bug0012)
- [R-0006](research.md#r-0006--rule-based-personal-finance-forecast-algorithms-mvp-baseline) — layered deterministic + rolling residual + category decomposition intent
- [DEC-0007](../../decisions/DEC-0007.md) — hybrid rule-based forecast; monthly buckets via category config
- [cashflow-app decomposition](https://github.com/NikolasMarkou/cashflow-app) — deterministic base vs residual split; per-component NECF attribution (Layer 0 / aggregation / decomposition modules)
- [FinToolSuite monthly money flow](https://www.fintoolsuite.com/en/tools/money-insights/monthly-money-flow-calculator/) — fixed vs variable essential split by recurrence pattern

**Findings:**

### Root cause (confirmed; not mirror ingest)

| Gap | Effect |
|-----|--------|
| `categorize_delta` uses **net daily `delta` sign** only | Income only when `delta >= 0`; most funded-account days net-negative → **Income 0** (AG) |
| `map_category(None, …)` for all `delta < 0` | Recurring rent/utilities due-days bucket as **Variable** → **Fixed 0** (AH) |
| `category_names` loaded in `service.rs` but **unused** | DEC-0007 TOML map never applied in projection |
| `RecurringPattern` lacks `category_id` | `detect_patterns` drops `RecurrenceGroup.category_ids` already collected in `recurrence/detect.rs` |

**Read path OK:** `GET /api/v1/forecast/monthly` + `ForecastPage` Monthly tab display only — fix is projection write path.

### DEC-0007 intent vs implementation

R-0006 / DEC-0007 specify: *"monthly forecast = sum projected recurring by bucket + rolling residual by bucket"*. Current code sums **one net delta per day into one bucket** — violates decomposition intent while daily **balance path** (sum of components) remains correct.

**Recommendation:** **Component-level monthly attribution** — preserve daily balance math (`delta = rolling.daily_rate + Σ recurring_due`); replace single `categorize_delta(delta)` with per-component bucket assignment for `monthly_map` only.

| Component | Bucket rule (recommended) | Rationale |
|-----------|---------------------------|-----------|
| `rolling.daily_rate` | **Variable** (negative → `variable_costs`; positive misc → `variable_costs` or architecture gate) | DEC-0007 "variable residual layer" for non-recurring spend |
| Each recurring due amount | `resolve_bucket(pattern.category_id, category_names, config)` | Uses existing `map_category` + TOML |
| Transfers | Excluded (unchanged) | DEC-0007 |

**Same-day mixed flows (salary + rent):** Component split attributes +salary recurring to Income and −rent recurring to Fixed on the same calendar day without net-sign collapse; balance still nets correctly.

**Alternatives considered:**

| Option | Verdict |
|--------|---------|
| Fix sign check only (`delta >= 0` → Income) | **Rejected** — still one bucket per day; Fixed stays 0 |
| Re-bucket net delta using "dominant" category of day's txs | **Rejected** — fragile; loses recurring semantics |
| Full transaction replay for monthly totals | **Rejected** — duplicates projection layers; performance + drift risk |
| **Component attribution (above)** | **Recommended** — minimal diff; aligns R-0006 §4 |

### Category resolution chain

```
category_id (mirror tx / RecurrenceGroup.category_ids)
  → category_names: HashMap<firefly_id, name>  (repository.category_name_map)
  → map_category(name, config)  (lowercase trim + TOML lookup)
  → Bucket
```

**TOML alignment (`default.toml`):**

```toml
[forecast.category_buckets]
salary = "income"
payroll = "income"
rent = "fixed"
mortgage = "fixed"
insurance = "fixed"
utilities = "fixed"
```

- `map_category` matches on **lowercased category name**, not Firefly id.
- Operator categories in German or custom labels (e.g. `Gehalt`, `Miete Nebenkosten`) **do not match** until operator adds keys to TOML — **not a BUG-0012 code bug** but acceptance risk on omniflow if names diverge.
- **BUG-0012 scope:** wire resolution path; **architecture** should decide operator doc vs default TOML expansion vs alias table (defer fuzzy/AI to **US-0015**).
- **Note:** `subscriptions.standing_order_category_patterns` tests `category_id` string `.contains(pattern)` — Firefly ids are typically numeric; pattern match often ineffective. Forecast fix should use **name map**, not duplicate subscription classify logic.

### RecurringPattern category carry

`RecurrenceGroup` already exposes `category_ids: Vec<Option<String>>` per payee group.

**Recommended propagation:**

1. Add `category_id: Option<String>` to `RecurringPattern`.
2. In `detect_patterns`, set from **mode** (most frequent non-null `category_id` in group) — tie-break: latest tx.
3. `apply_subscription_override`: when pushing `ConfirmedRecurring`, lookup category from mirror txs matching `payee_key` (ConfirmedRecurring has no category field today — runtime lookup sufficient for MVP; schema extension optional architecture gate).
4. On due-day in `project_account`, bucket recurring amount via resolved category — **not** description-only.

### Subscription / confirmed recurring inheritance

US-0003 confirmed patterns override heuristics via `apply_subscription_override` but drop category. **Inherit category** from:

1. Pattern `category_id` after detect, or  
2. Latest mirror tx with matching normalized payee_key when override replaces heuristic pattern.

Standing-order classification (`classify_kind`) is **subscription-engine concern** — forecast should consume **category_id on pattern**, not re-classify kind.

### Performance

- Daily loop: 730 iterations × O(recurring patterns) — unchanged.
- Added work: O(1) HashMap lookups per component — negligible vs existing recompute.
- No per-day full transaction scan required if `category_id` carried on pattern.

### Regression / compatibility risks

| Risk | Mitigation |
|------|------------|
| **Variable totals shrink** when recurring fixed moves to Fixed | Intended DEC-0007 behavior; add unit test asserting rent → Fixed not Variable |
| **Daily balance / milestones / horizons unchanged** | Only change `monthly_map` accumulation — do not alter `balance += delta` |
| **`free_cashflow` drift** | Recompute from component sums each day: `income - fixed - variable` (already done) |
| **Unmapped categories → Variable** | Existing `map_category` default; document operator TOML for omniflow |
| **DEC-0007 baseline authority** | Monthly buckets remain `model_kind=baseline`; ML overlay (US-0013) unchanged |
| **Plans/alerts consuming monthly series** | Bucket semantics become correct — verify no consumers assumed old all-Variable outflow |

### Test strategy (research → architecture → execute)

| Level | Scope |
|-------|-------|
| **Unit** (`project.rs`) | Salary (−category income) + rent (−category fixed) monthly history → first forecast month `income > 0`, `fixed_costs > 0` |
| **Unit** | Same-day salary due + rent due → both buckets non-zero; balance path unchanged |
| **Unit** | Discretionary recurring coffee → Variable; rejected fingerprint excluded |
| **Unit** | `map_category` + `category_names` integration helper |
| **Unit** | Subscription override with payee category lookup |
| **Integration** (`forecast_integration.rs`) | Optional: DB fixture with `categories` + `transactions.category_id` post-BUG-0006 → recompute → monthly API |
| **Regression** | Existing transfer exclusion, sparse `low_confidence`, subscription override balance tests stay green |

**Out of scope tests:** ML buckets, AI mapping (US-0015), operator omniflow name localization (verify-work after deploy).

### Architecture triad note

`docs/engineering/architecture.md` is **4624 lines** (> `ARCH_HOT_MAX_LINES=3000`) — triad **blocked on architecture oversize** (pre-existing). Research lands here (**R-0063**); `/architecture` should add § BUG-0012 after rollover or compact pack — do not append unbounded to hot architecture surface.

**Linked:** BUG-0012, BUG-0006, DEC-0007, DEC-0067, R-0006, R-0013, US-0015, US-0013  
**Confidence:** high  
**Status:** current — **fulfilled** by BUG-0012/Q0014 (DEC-0067, 2026-06-06)

---

## R-0064 — BUG-0009 Grafana panel emptiness vs cross-account overview gap

**Date:** 2026-06-06  
**Topic:** BUG-0009 — Operator sees empty Grafana panels despite 922+ synced transactions; no cross-account value overview in analytics  
**Query:** Variable default strategy; portfolio breakdown SQL fix; cross-account overview panel design; ML empty-state policy on baseline-only omniflow profile  
**Sources:**
- Live probes: `financegnome.omniflow.cc` public curl (Grafana health, ds/query, dashboard API, wealth/forecast meta) — discovery 2026-06-06
- Code: `grafana/provisioning/dashboards/analytics/{cashflow,portfolio,forecast-horizons}.json`; `backend/src/forecast/repository.rs` (`list_asset_accounts`); `frontend/src/pages/ForecastPage.tsx`
- Prior: [R-0061](research.md#r-0061--post-sync-analytics-pipeline-empty-data-paths), [R-0062](research.md#r-0062--firefly-account-balance-mirror-vs-forecastwealth-inputs), [R-0055](research.md#r-0055--grafana-dashboard-5-ml-extensions-dec-0055) (DEC-0055 ML panels)
- Web: [Grafana community — preselected variable from DB](https://community.grafana.com/t/preselected-value-of-variable-from-database/29721) (first query row when `current` empty); [Grafana JSON model — templating `current`](https://grafana.com/docs/grafana/latest/visualizations/dashboards/build-dashboards/view-dashboard-json-model/); [Grafana dashboard groupings — show/hide rules](https://grafana.com/docs/grafana/next/visualizations/dashboards/build-dashboards/create-dashboard/dashboard-groupings/) (Grafana 11 dynamic dashboards; not required for MVP)

### 1. Discovery recap (confirmed root causes)

| Sub | Root cause | Live evidence |
|-----|------------|---------------|
| **Y1** | `$account_id` variable query `ORDER BY name` → first row acct **116** (Cash wallet, flat zero forecast); acct **114** (Giro **-3395.75**) non-empty when selected | ds/query acct 116: 731 daily rows all 0; acct 114: negative non-zero |
| **Y2** | `forecast-horizons` ML panels hard-bind `model_kind='ml_enhanced'` — **0** computations on omniflow (DEC-0049) | ml_enhanced count **0**; baseline panels OK at correct account |
| **Y3** | Datasource/UNION regression **ruled out** | BUG-0003 H + BUG-0004 K pass; portfolio UNION pie **200** |
| **Z1** | Portfolio breakdown SQL: global `LIMIT 1` on cross-join truncates `jsonb_array_elements` to **1 of 3** rows | Broken → 1 row; subquery fix → 3 rows |
| **Z2** | No cross-account overview panel in analytics provisioning | React `/wealth` has breakdown + link to `/analytics/portfolio` but outside iframe shell |

**Not transport/SQL regression:** ds/query **200** for cashflow, portfolio totals, subscriptions, budgets, platform-health.

### 2. Variable default strategy (Y1)

**Current provisioning** (`cashflow.json`, `forecast-horizons.json`):

```sql
SELECT firefly_id AS __value, name AS __text
FROM accounts WHERE type = 'asset' ORDER BY name
```

- No `current` block in JSON → Grafana selects **first query result** on load ([Grafana forum R-0064 ref](https://community.grafana.com/t/preselected-value-of-variable-from-database/29721)).
- Matches React `ForecastPage`: `selectedAccount = accountId || accountsQuery.data?.[0]?.id` where API `list_asset_accounts` also `ORDER BY name` — **same alphabetical trap** on both surfaces.

**Alternatives evaluated:**

| Option | Mechanism | Pros | Cons | Verdict |
|--------|-----------|------|------|---------|
| **A — Max ABS mirror balance** | Variable query joins `accounts` + latest `balance`; `ORDER BY ABS(balance) DESC NULLS LAST, name` | Picks funded primary account (114 on omniflow); no backend change; works in provisioned JSON | Zero-balance-only deploy still defaults alphabetically; tie-break arbitrary | **Recommended for architecture** |
| **B — First non-zero forecast** | Subquery on `forecast_balance_daily` latest baseline computation; pick account with `MAX(ABS(balance))` on latest day | Aligns default with panel data source | Heavier query; fails before first recompute; duplicates computation lookup | Strong alternate if mirror balance stale |
| **C — Hardcoded `current` in JSON** | `"current": {"value": "114", "text": "Giro"}` | Immediate fix for one operator | Breaks other deployments; anti-pattern for provisioning | **Reject** |
| **D — React localStorage sync** | SPA passes `?var-account_id=` into iframe URL | UX parity with Forecast page selection | Requires embed URL contract change (US-0011); Grafana kiosk iframes don't share React state today | Defer to epic; not MVP bug fix |
| **E — Backend `/forecast/accounts` reorder** | Change API sort to match Grafana query | Single source for React + future API consumers | Does **not** fix Grafana embed alone; scope creep into React | Optional follow-up; not sufficient alone |

**Recommendation for architecture (Y1):**

1. Change `$account_id` variable query on **both** account-scoped dashboards to:

```sql
SELECT a.firefly_id AS __value, a.name AS __text
FROM accounts a
WHERE a.type = 'asset'
ORDER BY ABS(COALESCE(a.balance, 0)) DESC, a.name ASC
```

2. **Omit** `current` in provisioning JSON (or set `"current": {"text": "", "value": ""}`) so Grafana always picks first row from refreshed query — never bake operator-specific IDs.
3. Set `refresh: 1` (on dashboard load) — already present.
4. **Risk:** All-zero balances → falls back to alphabetical (same as today). Accept for MVP; document in panel description.
5. **Risk:** Negative overdrawn Giro is valid "non-empty" per DEC-0065 — ABS sort correctly prioritizes |-3395.75| over 0.
6. **Test:** Provisioning snapshot or SQL fixture asserting variable query order for fixture accounts; verify-work smoke with default load (no manual variable change).

### 3. Portfolio breakdown SQL fix (Z1)

**Broken panel** (`portfolio.json` id 5 — "Account breakdown (latest snapshot)"):

```sql
SELECT elem->>'name' AS name, ...
FROM net_worth_snapshots, jsonb_array_elements(payload->'accounts') AS elem
ORDER BY snapshot_date DESC LIMIT 1
```

PostgreSQL applies `LIMIT 1` to the **entire result set** after cross-join — one arbitrary account row from latest snapshot, not "latest snapshot's full account list".

**Recommended fix (subquery isolate snapshot, then unnest):**

```sql
SELECT
  elem->>'name' AS name,
  elem->>'account_role' AS role,
  elem->>'currency' AS currency,
  (elem->>'balance')::float AS balance
FROM (
  SELECT payload
  FROM net_worth_snapshots
  ORDER BY snapshot_date DESC
  LIMIT 1
) latest
CROSS JOIN LATERAL jsonb_array_elements(latest.payload->'accounts') AS elem
ORDER BY ABS((elem->>'balance')::float) DESC
```

- **Alternative rejected:** `DISTINCT ON (snapshot_date)` — unnecessary; subquery + LATERAL is clearer and matches discovery probe.
- **Risk:** Empty `payload->'accounts'` → empty table (valid empty-state post-sync).
- **Risk:** Mixed-currency accounts without FX — balances shown in native currency (existing portfolio stat row already warns via mixed_currency flag).
- **Test:** SQL fixture with 3-account snapshot JSON; assert row count = 3; regression guard against global LIMIT on cross-join.

**Note:** Fixing Z1 alone partially satisfies Z (operator sees all accounts on portfolio dashboard) but AC Z asks for **cross-account value overview** — Z2 still required.

### 4. Cross-account overview panel design (Z2)

**Acceptance Z:** "Operator has **cross-account value overview** in analytics (Grafana summary panel/table **or documented equivalent** via embedded wealth link showing per-account totals)."

**Alternatives evaluated:**

| Option | Placement | Pros | Cons | Verdict |
|--------|-----------|------|------|---------|
| **A — Portfolio stat row + table** | Top of `portfolio.json`: stat `total_eur`, table all accounts from latest snapshot | Single analytics route operator already uses; reuses `net_worth_snapshots`; satisfies AC without React change | Portfolio-only (not on cashflow/forecast) | **Recommended primary** |
| **B — Duplicate overview on every dashboard** | Same table panel copied to cashflow/forecast-horizons | Visible everywhere | Provisioning duplication; drift risk | **Reject** — maintenance |
| **C — New "Overview" analytics route** | Seventh dashboard + sidebar link | Clean landing page | US-0011 scope expansion; sidebar change | Defer unless PO requires landing |
| **D — React `/wealth` link only** | `AnalyticsEmbedPage` banner linking out | Zero Grafana work | **Fails AC Z** as primary — acceptance requires Grafana panel **or documented equivalent inside analytics shell**; link alone is supplementary per discovery | Supplementary only (Z3 docs) |
| **E — Text panel + deep link** | Grafana text panel with markdown link to `/wealth` | Quick | Does not show per-account totals in iframe | **Reject** as sole Z fix |

**Recommended panel set (Z2) on `portfolio.json`:**

1. **Stat row (existing):** `total_eur`, `account_count`, mixed-currency warning — already present; verify labels visible above fold in kiosk embed.
2. **New or upgraded table:** "All accounts (latest snapshot)" — use Z1 fixed SQL; columns: name, role, currency, balance, optional `% of Firefly subtotal` computed in SQL or transformation.
3. **Optional text panel:** "Detailed wealth analysis → `/wealth`" as supplementary (Z3), not substitute.

**SQL for overview table** — same as Z1 fix; optionally add:

```sql
-- pct column (when firefly subtotal non-zero)
ROUND(100.0 * (elem->>'balance')::float / NULLIF(
  (SELECT COALESCE(firefly_value_eur, total_eur) FROM net_worth_snapshots ORDER BY snapshot_date DESC LIMIT 1),
  0
), 1) AS pct_of_firefly
```

- **Risk:** `%` meaningless when `total_eur = 0` with mixed zero accounts — hide column or show `—`.
- **Risk:** Kiosk embed vertical space — place overview table at `y=0` or immediately below stat row; move performance charts down.

### 5. ML empty-state policy (Y2)

**Scope boundary:** US-0013 owns ML **enablement** on omniflow; BUG-0009 closes **honest empty-state** when `ml_enhanced` absent (DEC-0049, DEC-0066 precedent on React side).

**Affected panels** (`forecast-horizons.json`): ML balance series (A/B/C bands), ML vs baseline stat compare, ML portfolio weekly stats, seasonal_detected stat — all query `model_kind='ml_enhanced'`.

**Alternatives evaluated:**

| Option | Mechanism | Pros | Cons | Verdict |
|--------|-----------|------|------|---------|
| **A — Dashboard description only** | Already: `"description": "... ML panels empty when no ml_enhanced computation."` | Zero panel work | Operator still sees blank charts — reported as "empty panels" | **Insufficient alone** |
| **B — Text panel banner** | Row above ML section: "Baseline only — ML overlay requires US-0013 / full profile" | Clear, provisioning-only | Takes grid space | **Recommended MVP** |
| **C — Conditional stat replacing ML panels** | Single stat: `SELECT COUNT(*) FROM forecast_computations WHERE model_kind='ml_enhanced' AND status='success'` → display "ML unavailable (0 runs)" | Data-driven message | Extra panel; ML section still shows empty charts below | Combine with B |
| **D — Hide ML panels (show/hide rules)** | Grafana 11 dynamic dashboard rules on query result | Clean layout | Feature-flagged; repeating-panel bugs ([grafana#108340](https://github.com/grafana/grafana/issues/108340)); overkill for static provisioning | **Reject for MVP** |
| **E — Collapse ML row** | Grafana row panel wrapping ML charts, collapsed by default | Reduces clutter | Row headers visible in kiosk | Optional polish |

**Recommendation for architecture (Y2):**

1. Add **text panel** at top of ML section with copy aligned to DEC-0066 / ForecastPage: *"ML forecast not enabled on this deployment. Baseline DEC-0007 forecast is authoritative. Enable via US-0013."*
2. Set ML time-series panels `fieldConfig.defaults.noValue` → `"ML unavailable"` (cosmetic; series still empty).
3. **Do not** change `$forecast_variant` default (stays `baseline`).
4. **Do not** merge US-0013 sidecar work into BUG-0009.

### 6. Architecture decision gates (carry forward)

| Gate | Question for `/architecture` | Research lean |
|------|------------------------------|---------------|
| **DEC-xxxx Y1** | Canonical `$account_id` default query — ABS(balance) vs non-zero forecast? | **ABS(balance) DESC** |
| **DEC-xxxx Z2** | Is portfolio-only overview sufficient for AC Z, or require seventh landing dashboard? | **Portfolio-only** + supplementary `/wealth` link in Z3 docs |
| **DEC-xxxx Y2** | Banner-only vs hide ML panels when count=0? | **Banner + noValue text** |
| **AC Z equivalence** | Does fixed breakdown table + stat row satisfy Z without React embed change? | **Yes** — `/wealth` link documented supplementary in verify-work |
| **React parity** | Reorder `/forecast/accounts` to match Grafana default? | Optional post-bug consistency task — out of BUG-0009 execute unless architecture bundles |

### 7. Recommended fix order (execute preview)

| Order | Task | Rationale |
|-------|------|-----------|
| 1 | **Z1** — breakdown SQL subquery | Correctness bug; unblocks Z2 table |
| 2 | **Z2** — overview table + stat visibility | AC Z |
| 3 | **Y1** — variable query ORDER BY ABS(balance) | AC Y primary |
| 4 | **Y2** — ML banner + noValue | AC Y secondary |
| 5 | **Y3/Z3** — verify-work omniflow | Acceptance closure |

**Tests (research recommendation):** SQL fixtures for breakdown + overview queries; JSON provisioning test for `$account_id` query string; operator smoke: load `/analytics/cashflow` without changing variable → non-flat series.

### Risks summary

1. **All-zero deploy** — variable default still alphabetical; acceptable edge case.
2. **ABS(balance) picks wrong account** when multiple funded accounts — tie-break by name; rare for household MVP.
3. **Overview on portfolio only** — operator opening cashflow first may miss overview until visiting portfolio (mitigate: sidebar label "Portfolio (overview)").
4. **ML panels still visually empty** below banner — acceptable until US-0013; banner sets expectation.
5. **Provisioning `current` accidentally saved** during manual Grafana edit — document "do not save variable defaults" in execute runbook.

**Linked:** BUG-0009, BUG-0004, BUG-0010, US-0011, US-0013, DEC-0009, DEC-0049, DEC-0055, DEC-0057, DEC-0065, DEC-0066, R-0061, R-0062  
**Confidence:** high  
**Status:** current — research complete 2026-06-06; ready for `/architecture`

---

## R-0065 — BUG-0007 AI merchant/category discovery tool contracts vs RAG

**Date:** 2026-06-07  
**Topic:** BUG-0007 — AI Chat fails merchant/category discovery despite mirror data (sub-defects S/T/U + note V)  
**Query:** Fix-option analysis for category resolution, subscription enumeration, cross-signal fusion, orchestrator/audit improvements, payee aggregates, RAG boundary — within six-tool registry + DEC-0032 privacy defaults  
**Sources:**
- Live probes: `financegnome.omniflow.cc` — `GET /api/v1/{settings,sync/entities,subscriptions,ai/audit}`; Grafana `POST /analytics/grafana/api/ds/query` (`FlowFinancePostgreSQL`) — discovery 2026-06-07
- Code: `backend/src/ai/tools/{subscriptions,transactions}.rs`, `backend/src/ai/orchestrator.rs`, `backend/src/ai/privacy.rs`, `backend/src/ai/audit/repository.rs`, `backend/src/subscriptions/service.rs`, `backend/src/transactions/{repository,service}.rs`, `backend/src/db/mod.rs` (`categories` table)
- Prior: [R-0060](research.md#r-0060--ai-get_transactions-empty-aggregates-vs-mirror-sync) (BUG-0006 — DONE), [R-0031](research.md#r-0031--six-tool-registry-mapping--simulate_plan-read-only-contract), [R-0041](research.md#r-0041--local-llm-tool-calling-degraded-behavior), intake `handoffs/intake_evidence/intake-20260605-ai-merchant-category-discovery.json`
- Web: [OpenAI function calling](https://developers.openai.com/api/docs/guides/function-calling) — enum constraints, opaque-ID companion resolution, ≤20 tools guidance; [OpenAI Structured Outputs](https://help.openai.com/en/articles/8555517-function-calling-in-the-openai-api) — `strict: true` schema enforcement; [Ollama tool calling](https://docs.ollama.com/capabilities/tool-calling) — six-tool local budget per R-0041; privacy-by-design aggregate-first patterns (Zemtik proxy, Spendify hybrid categorization — research analogues only)

### 1. Discovery verdicts (sub-defects)

| Sub | Verdict | Primary root cause |
|-----|---------|-------------------|
| **S** | CONFIRMED | Mirror + tool return merchant `display_name` values; LLM fails to enumerate + mis-parameterizes follow-up `get_subscriptions` calls |
| **T** | SPLIT | (T-a) Amazon Jan–Oct 2023: **true empty period** (mirror starts 2025-06-05). (T-b) Strom/Amazon in mirror window: **keyword passed as `category_id`** + no merchant/description search dimension |
| **U** | CONFIRMED | Aggregate-only contract + `redact_counterparties` + system prompt bias → no cross-field fusion without user-supplied merchant names |
| **V** | NOTE | No RAG in codebase; enhancement path is tool/orchestrator-first unless operator opts into raw transactions |

### 2. Live runtime evidence (omniflow)

| Probe | Result |
|-------|--------|
| Transactions | 922 rows; dates **2025-06-05 … 2026-05-22**; **0** in 2023-01…2023-10 |
| Categories | 75 synced rows in `categories`; `Shopping - Amazon` (id **47**) 28 tx / 1079.35 €; `Wohnen - Stromkosten` (id **146**) 4 tx / 465.53 €; `Hobby & Freizeit - Streaming` (id **18**) 20 tx / 350.51 € |
| Subscriptions | 12 patterns — 3 confirmed (Netflix, YouTube, Mitgliedsbeitrag), 6 pending (Apple, Cursor, …), Strom standing_order **rejected** |
| Privacy settings | `allow_raw_transactions=false`, `redact_counterparties=true` |
| AI audit (2026-06-05) | `get_transactions` ok with `category_id: "amazon"` / `"Strom"`; `get_subscriptions` ok `{}` + errors `kind: Counterparty-*` |
| Description keyword | `transactions.description ILIKE '%amazon%'` → **0 rows** — Amazon signal is **category name**, not description text |

### 3. Tool contract gaps (code-confirmed)

**`get_transactions`:**

- Parameters: `period_start`, `period_end`, optional `category_id` (Firefly string ID), `group_by: category|month`.
- **Missing:** category **name** search, mirror period bounds in response, explicit search-attempt metadata for empty states.
- `category_id` SQL filter is exact match on `transactions.category_id` — passing `"Strom"` returns empty buckets while category **146** holds data.
- `aggregates_by_category` already joins `categories.name` — data exists server-side; LLM lacks a resolution path.

**`get_subscriptions`:**

- Returns `display_name`, `status`, `kind`, `current_amount`, `confidence_pct`, `interval_days` — sufficient for **S** if LLM enumerates `patterns[].display_name`.
- **Omits `payee_key`** vs REST (`SubscriptionService::list_patterns` strips it) — not blocking **S** because `display_name` is human-readable and **not** in PrivacyLayer sensitive-field list.
- Schema gap: `kind` property lacks enum (`subscription|standing_order` per migration 003); LLM passes privacy hashes as `status`/`kind` filters.

**Privacy layer (DEC-0032):**

- Blocks `raw_rows` when `allow_raw_transactions=false`.
- `redact_counterparties=true` replaces `description`/`payee`/`counterparty` with `Counterparty-{hash8}` — **payee-level aggregates under privacy defaults are not merchant-readable**.

**Orchestrator:**

- `SYSTEM_PROMPT`: "Prefer aggregates when raw transactions are disabled" — no instruction to enumerate subscription names, resolve category keywords, or cite mirror bounds on empty periods.
- Audit: `result_rows` column exists (`migration 006`) but orchestrator always inserts `None` — operator cannot distinguish empty mirror vs mis-parameterized tool vs LLM ignore.

### 4. Fix options (expanded)

#### Constraint: six-tool registry preserved

BUG-0007 acceptance footer requires **six-tool registry preserved** (same as BUG-0006 / US-0008 AC4). Adding a seventh `get_categories` tool would violate acceptance unless architecture emits an explicit DEC waiver. Research **rejects a net-new tool** for MVP; category resolution must extend **`get_transactions`** (or enrich an existing tool's contract) within the frozen registry count.

Per [OpenAI function-calling guidance](https://developers.openai.com/api/docs/guides/function-calling): use **enums** for finite parameter sets, document opaque IDs in parameter descriptions, and provide a **companion resolution path** (server-side keyword → ID) rather than expecting the model to guess Firefly IDs.

| ID | Option | Mechanism | Pros | Cons | Risks | Dependencies | Lean |
|----|--------|-----------|------|------|-------|--------------|------|
| **A′** | **Category search on `get_transactions`** | New optional param `category_search` (keyword, ILIKE on `categories.name`); server resolves matching `{firefly_id, name}` and aggregates; mutual exclusion with raw `category_id` or precedence rule | Resolves Strom→146, Amazon→47 without raw rows; **stays at six tools**; reuses synced `categories` table | SQL + schema change; need cap on matches (≤10) | Over-broad keyword returns many categories; ambiguous German synonyms (Strom vs Stromkosten) | `TransactionsRepository`, `GetTransactionsTool`, categories sync (US-0001) | **Baseline — adopt** |
| **A** | Separate category catalog tool | New `get_categories` with keyword filter | Clean separation; model-friendly catalog | **Violates six-tool AC** unless DEC waiver; extra tool round | Local model tool-selection noise (R-0041) | New tool + registry migration CHECK | **Reject MVP** |
| **B** | Payee/`group_by: merchant` aggregates | GROUP BY normalized payee/description | Direct merchant spend | Under `redact_counterparties`, payee keys become `Counterparty-*` — **not human-readable**; normalization hard | False confidence in hashed labels; privacy review | New SQL path + PrivacyLayer review | **Defer** — category search covers Amazon/Strom acceptance probes |
| **C** | `allow_raw_transactions` opt-in | Operator enables capped `raw_rows` | Description search possible | Redaction still hashes counterparties; privacy regression; not default | Operator enables without understanding redaction limits | DEC-0032 Settings display only (read-only MVP) | **Supplementary only** — document in architecture, not BUG-0007 default path |
| **D** | RAG over mirror text | Embed descriptions/categories offline | Natural-language merchant match | No existing infra; sync freshness; privacy surface; epic scope | Stale embeddings; PII in index | Vector store, ingest pipeline, US-0015 overlap | **Defer** — note V only |
| **E** | Orchestrator + audit improvements | Extended `SYSTEM_PROMPT`; populate `audit.result_rows`; richer OpenAI parameter `description`s; optional tool-result `search_attempted` hints | Low diff; fixes **S** enumeration gap; improves operator debug | Insufficient alone for **T** keyword resolution | Prompt drift on local models; over-long system prompt | `orchestrator.rs`, `AuditInsert` | **Baseline — combine with A′** |
| **F** | Enrich `get_subscriptions` schema | Tighten `kind`/`status` enums; tool description instructs listing all `display_name`; optional `merchant_names[]` summary field in response | Fixes **S** + malformed Counterparty-* filter errors | Does not resolve category keyword **T** alone | Standing-order vs subscription confusion in totals | `GetSubscriptionsTool`, `SubscriptionService::list_patterns` | **Baseline — adopt with E** |

#### Recommended fix path (for `/architecture`)

**Primary bundle: A′ + E + F** — server-side category keyword resolution, subscription schema/orchestrator guidance, audit row counts. Satisfies acceptance **S/T/U** under default privacy without RAG or a seventh tool.

**Secondary (architecture gate):** evaluate whether `get_transactions` response should include **`mirror_date_bounds`** `{ min, max }` on every call (or once per session via tool hint) so **T-a** (Amazon 2023) returns explicit empty-state evidence.

**Explicitly deferred:** **B** (payee aggregates under redaction), **D** (RAG), **C** (raw opt-in) except architecture documentation.

#### Cross-signal fusion path (sub-defect U)

Under A′+E+F, fusion without user-supplied merchant names:

1. User asks about streaming → `get_subscriptions` `{ kind: "subscription" }` → enumerate `display_name` + amounts.
2. User asks about Strom → `get_transactions` `{ category_search: "strom", period_* }` → server matches `Wohnen - Stromkosten` (id 146) + returns aggregates.
3. User asks about Amazon spend → `category_search: "amazon"` → id 47 aggregates; if period outside mirror bounds, response includes bounds + `period_status: no_rows_in_period`.
4. Optional second hop: after category match, LLM cites category name + amount + subscription overlap when relevant — no raw description required.

This satisfies **U** within aggregate-only mode because fusion uses **category names** (not redacted payee hashes) and **subscription display_name** (not sensitive per PrivacyLayer).

#### BUG-0008 coordination (do not merge scope)

| Shared surface | BUG-0007 (this bug) | BUG-0008 |
|----------------|---------------------|----------|
| Subscription patterns DB | AI tool JSON + orchestration | Alert unread count vs `/subscriptions` list |
| `display_name` / detection | Enumerate in chat | Detection recall + alert entity |
| Code touch | `ai/tools/subscriptions.rs`, orchestrator | `alerts/`, subscriptions UI, alert filters |

**Rule:** BUG-0007 may enrich AI tool payloads and prompts; must **not** change alert-count semantics, list-tab filters, or detection thresholds (BUG-0008 discovery scope). Shared `SubscriptionService::list_patterns` changes must be **additive JSON fields** only — no behavior change for REST list/alert consumers.

### 5. Orchestrator improvements (option E — detail)

| Change | Rationale | Risk |
|--------|-----------|------|
| System prompt: "When listing subscriptions, enumerate every `display_name` from tool results" | Closes **S** | Local models may still skip — mitigated by F schema text |
| System prompt: "Use `category_search` for utility/merchant keywords; never pass keywords as `category_id`" | Closes **T-b** | Requires A′ implemented |
| System prompt: "On empty period, state mirror date bounds if tool returns them" | Closes **T-a** evidence requirement | — |
| Populate `audit.result_rows` with pattern count / aggregate bucket count | Operator debug per discovery gap | Low — column already exists |
| Enrich parameter `description` on `category_id`: "Firefly numeric/string ID — use category_search for keywords" | OpenAI best practice for opaque IDs | Token cost negligible |
| Reject/warn when tool args contain `Counterparty-` prefix in enum fields | Prevent repeat of audit 2026-06-05 errors | Optional server-side validation in tool execute |

**Alternative considered:** `tool_choice: required` on subscription-list prompts — **rejected** for local providers (Ollama omits `tool_choice` — R-0038/R-0041).

### 6. Payee aggregates evaluation (option B — detail)

**Verdict: defer for BUG-0007 MVP.**

- `TransactionsRepository` has no payee GROUP BY today; `raw_rows_capped` includes description but blocked when `allow_raw_transactions=false`.
- Omniflow probe: Amazon merchant signal lives in **category name** (`Shopping - Amazon`), not `description` — payee aggregates would not improve Amazon acceptance case.
- Under DEC-0032 defaults, any payee aggregate would expose `Counterparty-{hash8}` labels — unusable for operator-facing merchant names and likely to **increase** LLM confusion (audit already shows Counterparty-* passed as enums).

**Revisit only if:** architecture adds an allowlisted merchant dimension (e.g. category-linked merchant tokens) or operator opts into raw rows (**C**).

### 7. RAG vs tools (note V)

No change from discovery: **no RAG path in codebase**. Tool-orchestrator enhancement (A′+E+F) meets acceptance without vector infra. RAG remains a future epic (coordinate US-0015 bucket mapping — different surface: forecast projection, not chat discovery).

### 8. Acceptance mapping

| Row | Research fix coverage |
|-----|----------------------|
| **S** | F + E — enumerate `display_name`; tighten enums |
| **T** | A′ + mirror bounds — Strom/Amazon via `category_search`; 2023 explicit empty-state with period bounds |
| **U** | A′ + F + multi-tool orchestration — category + subscription fusion without user merchant names |
| **V** | Document A′+E+F in architecture; D deferred |

**Regression footer:** `allow_raw_transactions=false` default unchanged; six-tool count unchanged; OIDC smoke deferred to verify-work (precedent Q0010).

### 9. Architecture decision gates (carry forward)

| Gate | Question for `/architecture` |
|------|------------------------------|
| DEC-???? | Approve A′ param contract (`category_search` vs `category_name_query` naming, match cap, precedence vs `category_id`) |
| DEC-???? | Mirror bounds in tool response vs orchestrator-only prompt |
| DEC-???? | Server-side enum guard for `Counterparty-*` in subscription filters |
| DEC-???? | `get_subscriptions` response shape additions (e.g. `merchant_names`, `patterns_count`) |
| Six-tool waiver | **Not recommended** — A′ avoids seventh tool |

### 10. Research readiness

| Gate | Status |
|------|--------|
| Fix options + risks + dependencies | PASS — §4–§6 |
| Category catalog / orchestrator / payee research | PASS — A′/E/F baseline; B deferred |
| Six-tool + DEC-0032 constraints | PASS — no default privacy regression |
| BUG-0008 isolation | PASS — §4 coordinate table |
| BUG-0006 overlap | CLEAR — intelligence layer only |
| Web references | PASS — OpenAI function calling, privacy aggregate-first analogues |
| Next phase | **`/architecture`** — emit DEC-xxxx for A′+E+F bundle |

**Linked:** BUG-0007, BUG-0006, BUG-0004, BUG-0008, US-0006, US-0015, DEC-0032, DEC-0035, R-0060, R-0031, R-0041  
**Confidence:** high  
**Status:** fulfilled — BUG-0007 closed via DEC-0069 / Q0017 (2026-06-07); retain for traceability; BUG-0008 coordinate table still valid

---

## R-0066 — Root README split layout and living-doc maintenance

**Date:** 2026-06-07  
**Topic:** US-0016 — missing root `README.md`; dual-audience entry; keep current as US/BUG close  
**Query:** DEC-0059 split layout, `validate_doc_profile` contract, maintenance at release/refresh-context vs per-commit automation  
**Sources:**
- Code: `scripts/doc_profile_lib.py`, `scripts/validate_doc_profile.py`, `docs/developer/README.md`
- Normative: **DEC-0059**, runbook § documentation profile validation (**US-0077**)
- Practice: [GitHub README best practices](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes) — keep root concise; link deeper docs; update on meaningful releases

### Findings

| Topic | Recommendation |
|-------|----------------|
| **Split layout** | Root README = user channel + `## Contributing` pointer only; `docs/developer/README.md` holds `DEV_*` H2 sections — already implemented in `doc_profile_lib.py` |
| **Content depth** | Purpose/Quickstart: product + compose profiles from `.env.example` comments; Examples: sync + analytics routes; Related docs: user-guides, runbook, architecture index |
| **Living updates** | Manual curated **Product status** bullet list at **release** + **refresh-context** when backlog item closes — avoids noisy per-commit churn; `validate_doc_profile` at release gate |
| **Template parity** | Create `template/README.md` when installer template tree ships; until then `--no-template-parity` or add minimal template stub in same sprint |
| **Anti-patterns** | Duplicating full dev workflow in root; embedding secrets; full backlog dump in README |

**Linked:** US-0016, US-0077, DEC-0059, US-0032, US-0031  
**Confidence:** high  
**Status:** fulfilled — US-0016 released S0013 via **DEC-0070** (2026-06-08); retain for traceability

---

## R-0067 — US-0016 root README research (template parity, Product status, maintenance hooks)

**Date:** 2026-06-08  
**Topic:** US-0016 discovery open questions — template parity posture, Product status placement, release/refresh-context maintenance binding  
**Query:** Stub `template/README.md` vs `--no-template-parity`; `###` vs `## Product status`; exact checklist hook wording for living-doc promise  
**Sources:**
- Code: `scripts/doc_profile_lib.py` (`count_profile_root_h2s`, template parity block), `scripts/validate_doc_profile.py`, `scripts/check_intake_template_parity.py` (installer-owned paths — separate gate)
- Repo state: `template/` tree **absent** (zero files); root `README.md` **absent** (blocks validator today)
- Normative: `docs/product/acceptance.md` § US-0016 (AC-5 maintenance cadence; AC-6 conditional on `template/` existence); runbook § documentation profile validation
- Practice: [GitHub About READMEs](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes) — root concise, link deeper docs; [opensource.guide best practices](https://github.com/github/opensource.guide/blob/main/_articles/best-practices.md) — vision/status in README or linked doc; update on meaningful releases, not every commit
- Prior: **R-0066** (split layout baseline)

### Findings

#### 1. Template parity (`template/` absent)

| Option | Outcome | Verdict |
|--------|---------|---------|
| Default `validate_doc_profile.py --repo .` | Sets `template_root=<repo>/template`; fails `[DOC_TEMPLATE_PARITY_FAIL] template/README.md missing` once root README exists | **Fail** until tree ships |
| Stub `template/README.md` only | Parity also requires `template/docs/developer/README.md` with matching DEV H2 presence for `(both, balanced)` — partial stub still fails | **Reject** — half-stub adds drift without satisfying validator |
| `--no-template-parity` | Sets `template_root=None`; skips parity block entirely (runbook already documents for fixture trees) | **Recommend** until full installer `template/` tree ships |
| Full `template/` mirror | Satisfies AC-6 when tree exists; aligns with `check_intake_template_parity.py` installer-owned paths | **Defer** to installer/template delivery — out of US-0016 execute scope |

**Recommendation:** CI and local release gate use `python scripts/validate_doc_profile.py --repo . --no-template-parity` while `template/` is absent. Drop the flag (use default parity) only when `template/README.md` **and** `template/docs/developer/README.md` land in the same change set. AC-6 remains satisfied vacuously until then ("when `template/` tree exists").

**Risks:** (1) Flag left on after template ships → silent parity gap. Mitigate: runbook note + architecture DEC gate to flip default when template tree merges. (2) Operator runs validator without flag locally → confusing `DOC_TEMPLATE_PARITY_FAIL`. Mitigate: document both commands in runbook § README maintenance (architecture).

#### 2. Product status placement

Validator budget (`count_profile_root_h2s`) counts **only** required `USER_*` H2 titles for the active profile — neither `## Contributing` nor extra H2s such as `## Product status` increment the budget counter (`doc_profile_lib.py`). For `(both, balanced)` the counter tops out at 5 required user H2s against budget 8; discovery's "H2 budget consumption" concern for a dedicated `## Product status` is **not enforced by the validator**.

Scannability still favors a capped subsection over a new top-level H2 (GitHub + opensource.guide: keep root README minimal; status bullets, not backlog dump).

| Placement | Pros | Cons | Verdict |
|-----------|------|------|---------|
| `### Product status` under `## Purpose` | Operators see product + recent closures first; no extra TOC H2; satisfies AC-5 "or equivalent" | Slightly couples status to Purpose prose | **Recommend** |
| `###` under `## Related documentation` | Near doc links | Semantically wrong (status ≠ link index) | **Reject** |
| Dedicated `## Product status` H2 | Visible in TOC | Adds noise; no validator benefit over H3 | **Reject** |

**Content contract:** reverse-chronological bullets `{US-xxxx\|BUG-xxxx} — {one-line outcome}`; cap **8** entries (drop oldest); link `docs/product/backlog.md` for full history; never duplicate acceptance tables or secrets.

#### 3. Maintenance binding (release + refresh-context)

Living-doc updates bind to **phase boundaries**, not per-commit automation (consistent with R-0066; rejects dokku/LLM auto-README patterns for this repo).

**Release (`/release`) — add after backlog reconciliation (≈ step 10), before runbook readiness (≈ step 14):**

> When any **US** or **BUG** in the target sprint transitions to **DONE** / **CLOSED**, append one bullet to root `README.md` **`### Product status`** (under `## Purpose`) in the form `{id} — {one-line outcome}`; trim to the **8** most recent entries. Before finalizing release readiness, run `python scripts/validate_doc_profile.py --repo . --no-template-parity` (drop `--no-template-parity` only after `template/` tree exists); non-zero exit → fail closed with remediation pointing to runbook § README maintenance.

**Refresh-context (`/refresh-context`) — add after backlog status reconciliation:**

> When release or sprint artifacts closed a **US** or **BUG** since the prior refresh, verify root `README.md` **`### Product status`** includes the closed id(s); update if missing. When README or doc-profile surfaces were touched, run `python scripts/validate_doc_profile.py --repo . --no-template-parity`.

**Developer shard (`docs/developer/README.md` § Workflow or Quality gates):**

> After `/release` or `/refresh-context` closes backlog items, curators/release agents update root README Product status per runbook § README maintenance; contributors run `validate_doc_profile` when editing README surfaces.

**Runbook (new subsection under § documentation profile validation — architecture execute):** title **`README maintenance (US-0016)`**; embed the three hooks above; note `--no-template-parity` posture until `template/` exists.

**Linked:** US-0016, US-0077, R-0066, DEC-0059 (doc profile split layout — distinct from Firefly ingest DEC-0059 record)  
**Confidence:** high  
**Status:** fulfilled — formalized as **DEC-0070**; US-0016 released S0013 (2026-06-08); US-0017 expansion per [R-0078](research.md#r-0078--us-0017-readme-omniflow-smoke-templates-h3-layout-validate_doc_profile-gates); retain for traceability

---

## R-0068 — BUG-0008 subscription alert dedup, unread count contract, orphan lifecycle

**Date:** 2026-06-08  
**Topic:** BUG-0008 sub-defect **W** — alert accumulation vs list mismatch; unread-count API; header bell scope; orphan alerts on pattern lifecycle  
**Query:** Alert dedup contract (`pattern_id+type` vs fingerprint lifecycle vs mark-read on confirm); reconciled unread-count API vs banner semantics; US-0005 bell inclusion; stale alert cleanup  
**Sources:**
- Code: `backend/src/subscriptions/detection.rs` (`insert_alert` every sync per group), `repository.rs` (bare INSERT, no ON CONFLICT), `frontend/src/pages/SubscriptionsPage.tsx` (banner = `alerts.length`), `frontend/src/components/AlertBell.tsx` (badge = unified count only)
- Live probe (2026-06-08, omniflow public API): 83 unread `new_detection` alerts, 6 pending patterns, 12 total patterns; unified `/api/v1/alerts/unread-count` = 0
- Prior: [R-0011](docs/engineering/research.md#r-0011--subscription-price-change-detection--alert-thresholds), [R-0012](docs/engineering/research.md#r-0012--subscription-persistence-schema-candidates-confirmed-rejections-events), [R-0023](docs/engineering/research.md#r-0023--alert-persistence-deduplication--lifecycle-acknowledge--dismiss) (unified-alert fingerprint pattern), [R-0065 § BUG-0008 coordinate](docs/engineering/research.md#bug-0008-coordination-do-not-merge-scope)
- Web: [Error-tracking alert dedup LLD](https://www.techinterview.org/post/3233469724/lld-error-tracking/) — fingerprint + lifecycle (alert on first occurrence / regression only); [PostgreSQL unique indexes](https://www.postgresql.org/docs/current/indexes-unique.html) — partial unique for active episodes; [Elysiate ON CONFLICT upsert](https://www.elysiate.com/blog/upserts-from-csv-on-conflict-patterns-that-scale) — dedupe source batch + arbiter index

### 1. Discovery open questions — resolution (W surface)

| # | Question | Resolution |
|---|----------|------------|
| **1** | Alert dedup contract | **Lifecycle fingerprint dedup** (not per-`sync_run_id`). One unread alert episode per `(pattern_id, alert_type)` until read or pattern terminal state. Reject bare INSERT every sync; reject `(pattern_id, alert_type, sync_run_id)` (still spams across runs). |
| **2** | Unread count API | Add **`GET /api/v1/subscriptions/alerts/unread-count`** returning structured contract (§2). Banner and toast consume this endpoint — not raw list length. |
| **3** | Header bell scope | **Keep US-0005-only badge** per R-0011/R-0023 boundary. Subscription unread stays on `/subscriptions` banner + popover link (`AlertBell` already shows "View subscription alerts (N)" when open). Combined badge is optional stretch — reject for BUG-0008 MVP (scope + coordinate table). |
| **6** | Orphan/stale alerts | Auto **mark-read** unread `new_detection` alerts when pattern **confirmed**, **rejected**, or **inactive**. One-time backfill dedupes historical rows (§4). |

### 1.2 Recommended dedup mechanism

Align with R-0023 unified-alert pattern adapted for `subscription_alerts`:

**Fingerprints:**

| `alert_type` | Fingerprint | Re-fire rule |
|--------------|-------------|--------------|
| `new_detection` | `sub_alert:new_detection:{pattern_id}` | After mark-read, only if pattern returns to `pending` with materially changed fingerprint (new pattern row) |
| `price_change` | `sub_alert:price_change:{pattern_id}:{direction}:{round(new_amount,2)}` | New episode after prior price alert marked read AND amount changes again beyond R-0011 thresholds |
| `interval_change` | `sub_alert:interval_change:{pattern_id}:{interval_days}` | Same lifecycle as price_change |

**Schema (new migration):**

```sql
ALTER TABLE subscription_alerts ADD COLUMN fingerprint TEXT;
-- backfill then SET NOT NULL in same migration after dedupe script

CREATE UNIQUE INDEX subscription_alerts_unread_fingerprint
  ON subscription_alerts (fingerprint)
  WHERE read_at IS NULL;
```

**Insert contract (`insert_alert` → `upsert_alert`):**

```sql
INSERT INTO subscription_alerts (pattern_id, alert_type, title, body, sync_run_id, fingerprint)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (fingerprint) WHERE read_at IS NULL
DO UPDATE SET body = EXCLUDED.body, sync_run_id = EXCLUDED.sync_run_id, created_at = NOW();
```

Requires PostgreSQL ≥15 partial unique index for `ON CONFLICT` arbiter — repo already targets modern Postgres/TimescaleDB.

**Detection pipeline change:** call `upsert_alert` only when pattern upsert returns **new pending** OR confidence tier **increased** — skip alert when pending pattern unchanged on routine sync (matches R-0011 "alert on new detection" intent).

**Alternatives rejected:**

| Option | Why rejected |
|--------|--------------|
| Per-`sync_run_id` dedup | 83 alerts from ~14 sync runs — still 6× pending mismatch |
| Mark-read-only (no dedup index) | Does not fix historical accumulation; race on concurrent sync |
| Merge into unified `alerts` table | R-0023 boundary; different UX surfaces |

### 2. Unread count API contract (acceptance **W**)

Acceptance requires **reconciled semantics**, not raw alert rows = list rows. Different entities:

- **List rows** = `subscription_patterns` filtered by tab (`pending`, `confirmed`+`standing_order`, `all`)
- **Actionable unread** = unread `new_detection` alerts whose `pattern_id` references a **`pending`** pattern

**Recommended response:**

```json
GET /api/v1/subscriptions/alerts/unread-count

{
  "unread_total": 2,
  "unread_new_detection": 2,
  "unread_price_change": 0,
  "pending_patterns": 6,
  "reconciled": true,
  "reconciliation_note": "unread_new_detection counts pending patterns with unread new_detection alerts; price_change alerts are informational"
}
```

**UI contract:**

| Surface | Display | Source |
|---------|---------|--------|
| `/subscriptions` banner | `{unread_new_detection} unread alert(s)` when >0; subtitle when `unread_new_detection != pending_patterns`: "N pending patterns to review" | unread-count API |
| Toast after sync | Fire only when `unread_new_detection` increases vs sessionStorage | unread-count API |
| Tab badges | `pending_patterns` count on Pending tab | existing list query or unread-count |

**`reconciled: true`** when `unread_new_detection <= pending_patterns` AND every unread `new_detection` alert joins to a pending pattern (SQL LEFT JOIN guard). After dedup + orphan cleanup, expect **`unread_new_detection == pending_patterns`** for steady state.

**Reject:** deriving banner count from `GET .../alerts?unread=true` list length without dedup — preserves W failure mode.

### 3. Header bell scope (question 3)

| Option | Verdict |
|--------|---------|
| Include subscription unread in badge total | **Defer** — crosses R-0023/US-0005 boundary; needs DEC + combined UX spec |
| US-0005-only badge + popover subscription link | **Keep** — already implemented; operator W symptom is subscriptions-page banner (83), not bell (0) |
| Remove popover subscription link | **Reject** — useful cross-nav |

Document in architecture: subscription alert trust metric is **`/subscriptions` banner + unread-count API**, not header bell.

### 4. Orphan/stale alert lifecycle (question 6)

| Event | Action |
|-------|--------|
| `POST .../confirm` | `UPDATE subscription_alerts SET read_at = NOW() WHERE pattern_id = $1 AND alert_type = 'new_detection' AND read_at IS NULL` |
| `POST .../reject` | Same + pattern status `rejected` stops future `new_detection` (fingerprint in rejections) |
| `mark_inactive` | Mark-read all unread alerts for pattern |
| One-time migration | Group by fingerprint; keep newest unread per `(pattern_id, alert_type)`; mark-read duplicates |

Orphan case today: 83 alerts for 6 pending — ~77 are duplicate sync inserts for same patterns.

### 5. Risks

1. **X before W** — lowering detection thresholds without dedup re-amplifies alert spam (discovery risk #1); enforce W bundle first in sprint plan.
2. **Partial unique + NULL fingerprint backfill** — migration must backfill before NOT NULL + index.
3. **Price-change dedup too aggressive** — if operator marks read then price changes again legitimately, new fingerprint must differ by amount bucket.
4. **`list_patterns` REST regression** — additive JSON only per R-0065 coordinate; unread-count is new route, not filter change.
5. **Acceptance static numbers** — operator 33 vs 11 and live 83 vs 6 are **snapshots**; acceptance tests reconciled semantics, not fixed counts.

### 6. Architecture decision gates

| Gate | Question for `/architecture` |
|------|------------------------------|
| DEC-???? | Approve fingerprint formula + partial unique migration |
| DEC-???? | unread-count response schema + `reconciled` computation |
| DEC-???? | Upsert-only-on-new-pending vs every-sync upsert (touch `created_at`) |
| DEC-???? | Backfill script scope for operator DB (83 rows) |

**Linked:** BUG-0008, US-0003, US-0005, R-0011, R-0012, R-0013, R-0023, R-0065, R-0069  
**Confidence:** high  
**Status:** fulfilled — formalized as **DEC-0071**; BUG-0008 released Q0018 (2026-06-08); retain for traceability

---

## R-0069 — BUG-0008 detection recall levers & AI path boundary

**Date:** 2026-06-08  
**Topic:** BUG-0008 sub-defect **X** — under-detection from 922+ transactions; recall option matrix; AI-assisted detection scope  
**Query:** Threshold tuning, payee normalization, category-aware grouping, standing-order split, in-pipeline vs async AI enrichment  
**Sources:**
- Code: `backend/src/recurrence/{group,detect,normalize}.rs`, `backend/src/subscriptions/detection.rs` (`min_emit_confidence: 60` hardcoded), `backend/config/default.toml` (`detection_window_days = 365`)
- Live probe: 12 patterns (3 confirmed, 6 pending, 3 rejected); pending payees include long SEPA descriptor strings; BUG-0007 categories (`Shopping - Amazon` id 47, 28 tx) not forming patterns
- Prior: [R-0009](docs/engineering/research.md#r-0009--subscription-detection-engine-patterns--confidence-scoring), [R-0010](docs/engineering/research.md#r-0010--dauerauftrag-standing-order-vs-subscription-classification), [R-0065](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag) (AI chat vs detection boundary)
- Web: [GiGurra/subscription-detector](https://github.com/GiGurra/subscription-detector) (35% amount tolerance default vs our 5–15%); [Spade recurring guide](https://docs.spade.com/reference/recurring-transaction-guide) (merchant identity normalization)

### 1. Discovery open questions — resolution (X surface)

| # | Question | Resolution |
|---|----------|------------|
| **4** | Recall levers | **Phased bundle** (§2) — Phase 1 normalization + window; Phase 2 category-aware grouping + optional confidence tuning. Standing-order split already shipped (R-0010). |
| **5** | AI-assisted detection | **Out of sync mutex for MVP.** Document deferred path: optional async enrichment job post-detection (not BUG-0007 orchestrator). Acceptance **X** footer satisfied by rule improvements in execute; AI noted as future gate only. |

### 2. Recall option matrix

| Lever | Mechanism | Recall gain | False-positive risk | Phase | Verdict |
|-------|-----------|-------------|---------------------|-------|---------|
| **Payee normalization** | Extend `payee_key()`: strip SEPA `SVWZ+`/reference tokens, card suffixes, collapse legal-entity suffixes (`GmbH`, `AB`); prefer `counterparty_name` when description matches `SVWZ|UEBERWEISUNG|Lastschrift` regex | **High** — reduces SEPA fragmentation | Low | **1** | **Adopt** |
| **Description vs counterparty priority** | For bank-transfer-shaped descriptions, use `counterparty_name` **before** full memo string (invert DEC-0061 priority for transfer type) | **High** — merges Netflix/Apple under merchant name | Medium — may merge distinct memos same counterparty | **1** | **Adopt with transfer-type guard** |
| **Detection window** | `detection_window_days` 365 → **730** (config-only) | Medium — annual subs need 2+ cycles | Low | **1** | **Adopt** |
| **Category-aware grouping** | When ≥70% txs in payee group share one `category_id`, add secondary grouping key `cat:{category_id}` for txs with weak payee keys | **High** — Amazon/Strom via Firefly categories (BUG-0007 probe) | Medium — category mis-tags merge wrong merchants | **2** | **Adopt** |
| **min_emit_confidence** | 60 → 55 or tiered emit at 55 with UI "low confidence" badge | Medium | **High** — backlog originally set 60% floor | **2** | **Gate** — only after W dedup + operator FP review |
| **Amount tolerance** | Widen low-tier to ±20% (config) | Low–medium | Medium | **2** | **Defer** — try normalization first |
| **min occurrences** | 3 → 2 for monthly+ with 4+ months span | Medium | High | **3** | **Reject MVP** — Spade ≥3 months precedent |
| **Standing-order reclassification** | Tune R-0010 CV thresholds | Low for subscription recall | Low | **3** | **Defer** |

**Recommended execute bundle (X):** Phase 1 payee normalization + transfer-type counterparty priority + 730-day window. Phase 2 category-aware grouping. Measure via acceptance probe: pending+confirmed subscription-kind patterns **> 12** without `unread_new_detection > pending_patterns`.

### 2.1 Code-confirmed gaps

- `RecurrenceGroup.category_ids` populated in `detect.rs` but **not used** in grouping — category signal wasted.
- `min_emit_confidence: 60` **hardcoded** in `detection.rs` — not wired to TOML (config gap).
- `extract_payee_source` prefers description — SEPA memos become payee keys (live pending rows).

### 3. AI-assisted detection (question 5)

| Path | Verdict |
|------|---------|
| In-pipeline sync mutex (LLM classify groups) | **Reject** — latency, privacy (DEC-0032), non-determinism in mutex |
| BUG-0007 orchestrator feeding detection | **Reject** — coordinate table; chat tools ≠ detection service |
| Async post-detection enrichment job | **Defer** — optional architecture spike: suggest category/payee merges for operator review queue |
| RAG over transactions | **Defer** — R-0065 note V; no infra |

**Acceptance **X** footer:** document Phase 1–2 rule improvements in architecture/release notes; state AI path deferred with optional async gate in DEC.

### 4. Sequencing dependency (R-0068)

**W dedup must ship before or with Phase 1 recall.** Each new pending pattern currently adds another undeduped alert per sync — recall work without W fix worsens operator trust.

### 5. Risks

1. **Over-merge** — counterparty priority merges distinct subscriptions same biller (e.g. multiple Apple services).
2. **Category false joins** — shared "Shopping" category merges unrelated merchants.
3. **Regression on forecast** — recurrence core shared with forecast (DEC-0013); normalization changes affect both — require integration tests.
4. **Coordinate table** — `SubscriptionService::list_patterns` filter behavior unchanged; detection internals only.

### 6. Architecture decision gates

| Gate | Question for `/architecture` |
|------|------------------------------|
| DEC-???? | Phase 1 normalization rules + transfer-type detection |
| DEC-???? | Category-aware grouping threshold (70% same category) |
| DEC-???? | Wire `min_emit_confidence` to TOML vs keep 60 hard floor |
| DEC-???? | AI async enrichment — in BUG-0008 scope or new US |

**Linked:** BUG-0008, US-0003, BUG-0007, R-0009, R-0010, R-0013, R-0065, R-0068  
**Confidence:** high  
**Status:** fulfilled — formalized as **DEC-0072** Phase 1; BUG-0008 released Q0018 (2026-06-08); Phase 2 category grouping gated; retain for traceability

---

## R-0070 — BUG-0011 planning mode compare delta, empty-state API, first-run UX

**Date:** 2026-06-08  
**Topic:** BUG-0011 sub-defects **AD/AE/AF** — compare metric semantics, plan-vs-actual empty contract, first-run onboarding, add-adjustment wiring  
**Query:** Resolve six discovery open questions: compare delta definition; empty-plan zero semantics; plan-vs-actual API shape when no active plan; first-run create/activate matrix; add-adjustment UX; OIDC/Grafana regression scope  
**Sources:**
- Code: `backend/src/plan/repository.rs` (`version_metrics` sums `planned_net`), `backend/src/plan/service.rs` (`project_adjustments_in_memory` identical bug), `backend/src/plan/project.rs` (`planned_net = baseline_net + overlay_delta`), `backend/src/plan/overlay.rs` (`build_overlay_deltas`), `backend/src/api/plans.rs` (`plan_error_status` 404 on `NoActivePlan`; `risk_score` 200 `no_score`), `frontend/src/pages/PlanningPage.tsx` (empty state Leasing-only; no add form; `pvaQuery` no error branch)
- Prior: [R-0015](docs/engineering/research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline) (overlay model), [R-0016](docs/engineering/research.md#r-0016--plan-scenario-versioning-immutable-snapshots-vs-editable-drafts) (compare metrics: "Monthly delta sum (net recurring impact €/month)"), [R-0017](docs/engineering/research.md#r-0017--plan-vs-ist-daily-computation--aggregation-grain), [R-0020](docs/engineering/research.md#r-0020--grafana-dashboard-3-budgets-planistdeviation-provisioning) (Grafana uses stored `planned_net`, not compare endpoint)
- User guide: `docs/user-guides/US-0004.md` (custom lines manual add; explicit Set active)
- Web: [Model Reef — scenario override layer + comparison pack](https://modelreef.io/resources/articles/scenario-analysis/scenario-analysis-build-real-time-scenario-planning-models-without-spreadsheet-sprawl) (baseline locked; scenarios = override layer; compare shows delta vs base); [Logica — scenario comparison variance columns](https://help.logica.cloud/en/articles/5160561-comparing-scenarios-with-views) (base vs scenario delta, not scenario total labeled as delta)

### 1. Discovery open questions — resolution

| # | Question | Resolution |
|---|----------|------------|
| **1** | Compare delta contract | **Overlay-only monthly delta** — sum daily values from `build_overlay_deltas(adjustments, …)` for current calendar month through today (same sign convention as R-0015). Reject summing `plan_daily_cashflow.planned_net` (baseline + overlay). Reject per-version delta vs sibling version (compare table already lists each version side-by-side). **Projected month-end balance** stays full scenario total (`planned_balance` at horizon end-of-month) — not an overlay metric; relabel/help text optional in execute. |
| **2** | Empty-plan zero semantics | When `adjustments.is_empty()` (Custom template or zero lines): **`monthly_delta_sum` MUST format `0.00`** regardless of baseline forecast magnitude. **`projected_month_end_balance`** = baseline month-end balance from projection (may be negative — that reflects household forecast, not a "delta"); acceptance **AE** illogical aggregate targets mislabeled delta column primarily. Optional execute footnote under compare table: "Monthly delta = scenario adjustments only; projected balance includes baseline forecast." |
| **3** | plan-vs-actual empty API | **HTTP 200 tagged `{ status: "no_active_plan", reason: "no_active_plan" }`** — mirror `GET /api/v1/plans/risk-score` `RiskScoreApiResponse::NoScore` pattern. Reject raw **404** (breaks tab; acceptance **AF**). Reject **200 + `rows: []` alone** (ambiguous vs active plan with sparse data). Reject **auto-activate on create** (violates US-0004 explicit Set active; use guided UX instead). |
| **4** | First-run onboarding | **Empty state (`plans.length === 0`)**: show compact template card grid (reuse `TEMPLATES` including **Custom**) plus name field and primary **Create empty plan** (`POST { name, template: "custom" }`) — satisfies **AD** "Start empty and add lines" entry path. Keep **Create from Leasing template** as secondary quick path. **Do not auto-activate** globally; after first create show inline banner: "Set active to enable Plan vs Actual and Grafana Dashboard 3." Optional pragmatic shortcut (architecture gate): auto-activate only when `plans.length === 1` after create — **defer**; prefer explicit Set active + AF 200 guided state. |
| **5** | Add-adjustment UX | **Inline add form** above adjustments table (not modal — savings_mode modal stays exception). Required fields per R-0015: `direction`, `amount`, `frequency` (default `monthly`), `target_type` (default `household`), optional `label`, `effective_from` (default today). Wire **`POST /api/v1/plans/{id}/versions/{vid}/adjustments`** + **`PATCH …/adjustments/{aid}`** for edit; invalidate `plan-version`, `plan-compare`, `plans` queries; backend already `spawn_recompute` on add. Custom template **Apply** on existing empty plan: toast "Custom plan ready — add lines below" (no silent no-op). |
| **6** | Regression scope | **OIDC deploy smoke** for `/planning` all three tabs mandatory (acceptance **AF** tail). **Grafana Dashboard 3 (`budgets`) unchanged** for AE — panels read `plan_daily_cashflow.planned_net` (full scenario series, correct for Plan/Ist/Abweichung per R-0020); compare fix is **`/compare` endpoint + React Compare tab only**. No panel SQL review required; note in release notes that compare table semantics align with R-0016 after BUG-0011. |

### 2. Compare metric formula (acceptance **AE** — recommend **DEC-0073**)

**Root cause:** `version_metrics` and `project_adjustments_in_memory` sum `planned_net` (= baseline + overlay), producing operator **-127489.44** on zero-adjustment plans.

**Fix (single helper, two call sites):**

```text
monthly_delta_sum(version) =
  SUM( overlay_delta(d) for d in [month_start .. min(today, month_end)] )
  where overlay_delta = build_overlay_deltas(adjustments, confirmed_subs, month_start, month_end)[d]

projected_month_end_balance(version) =
  planned_balance at overlay_horizon_end(month_start) from full project_plan_series
  (unchanged — includes baseline)
```

**Impact on non-empty plans:**

| Template | Before (bug) | After (correct) |
|----------|--------------|-----------------|
| Custom / Current, 0 lines | ~full forecast net (e.g. -127489) | **0.00** delta |
| Leasing (+€300/mo outflow) | baseline + leasing total | **~-300/mo** overlay delta |
| Savings mode (removals + cut) | baseline-dominated sum | **net overlay** (removed sub amounts + cut) |

**Migration note:** Compare numbers shift for all plans — intentional alignment with R-0016; document in BUG-0011 release notes (not a DB migration).

**Alternatives rejected:**

| Option | Why rejected |
|--------|--------------|
| Delta vs Current (Ist) baseline version row | Equivalent math when overlay additive; extra baseline version fetch without UX gain |
| Rename column only (keep wrong sum) | Does not satisfy acceptance zero/neutral deltas |
| Store overlay series separately in hypertable | Scope creep; compute overlay sum at read time from adjustments |

### 3. plan-vs-actual empty-state contract (acceptance **AF** — recommend **DEC-0074**)

Extend response with tagged enum (Axum `#[serde(tag = "status")]`):

```json
// Active plan exists — unchanged shape
{
  "status": "ok",
  "month": "2026-06",
  "reporting_currency": "EUR",
  "plan_stale": false,
  "actuals_stale": false,
  "rows": [ … ]
}

// No active plan — HTTP 200 (not 404)
{
  "status": "no_active_plan",
  "reason": "no_active_plan"
}
```

**Frontend:** `pvaQuery` with `retry: false`; when `status === "no_active_plan"`, render guided card (link to Set active / create plan) — mirror risk-score badge handling.

**Breaking change:** Clients expecting 404 must migrate; document in API changelog + user guide § Plan vs Actual.

**Alternatives rejected:**

| Option | Why rejected |
|--------|--------------|
| 404 + frontend-only fix | Acceptance **AF** requires API 200; raw 404 fails AC |
| Auto-activate first plan | Hides explicit active-plan semantics; breaks multi-plan operators |
| 200 + empty rows only | Indistinguishable from active plan with no computed rows |

### 4. AD bundle (execute scope — no separate DEC)

| Gap | Execute fix |
|-----|-------------|
| Empty state Leasing-only | Add **Create empty plan** + template cards when `plans.length === 0` |
| Custom Apply silent | Success toast + scroll/focus add form |
| No POST wiring | Inline add/edit form → existing `add_adjustment` / `update_adjustment` routes |
| First-run path | `POST template=custom` creates plan with `PlanTemplate::Custom` defaults `[]` then shows editable table |

Bound to **US-0014** for holistic polish (tooltips, wizard) — out of BUG-0011 defect scope per discovery.

### 5. Risks (carry to architecture)

1. **Compare number shift** — non-empty plans show smaller monthly delta (overlay-only); release note required.
2. **DEC ID coordination** — `docs/engineering/runbook.md` forward-references **DEC-0073** for US-0090 caveman compression; formalize BUG-0011 **AE/AF** as DEC-0073/DEC-0074 first in `decisions/` or renumber US-0090 at architecture.
3. **Negative projected balance on empty overlay** — baseline forecast may still show negative month-end; mitigate with compare help text, not zeroing balance.
4. **Tagged PVA response** — TypeScript `PlanVsActual` union type + any API consumers must handle `no_active_plan`.

### 6. Recommended architecture decisions

| ID | Scope | Bundle |
|----|-------|--------|
| **DEC-0073** | **AE** | Overlay-only `monthly_delta_sum` via `build_overlay_deltas`; `projected_month_end_balance` unchanged; shared helper in `repository.rs` + `service.rs` |
| **DEC-0074** | **AF** | `plan-vs-actual` 200 tagged `no_active_plan`; frontend guided empty state; optional `PlanVsActualResponse` union |

**Sequencing:** AE backend metric (DEC-0073) before or with AF API (DEC-0074); AD frontend can parallel once API contracts frozen.

**Linked:** BUG-0011, US-0004, US-0014 (deferred epic), R-0015, R-0016, R-0017, R-0020  
**Confidence:** high  
**Status:** fulfilled — released Q0019 2026-06-08; formalized as **DEC-0073** (AE) and **DEC-0074** (AF); US-0090 caveman forward-ref renumbered to **DEC-0075**; retain for traceability

---

## R-0071 — US-0013 production ML enablement on omniflow external profile

**Date:** 2026-06-08  
**Topic:** US-0013 re-intake — external profile ML sidecar wiring, sync gates, acceptance decomposition, sprint-plan slice boundaries  
**Query:** Close AC3 gap from BUG-0010; enable US-0009 ML on `financegnome.omniflow.cc` without new model research  
**Sources:**
- Code: `docker-compose.yml` (`stats-forecast` `profiles: [full]`), `docker-compose.external.yml` (no sidecar today), `backend/config/default.toml` `[forecast_ml]`, `backend/src/sync/mod.rs`, `backend/src/forecast_ml/`, `frontend/src/pages/ForecastPage.tsx`
- Prior: [R-0043](research.md#r-0043--self-hosted-statistical-forecasting-for-us-0009-discovery), [R-0044](research.md#r-0044--statsforecast-sidecar-compose-footprint-and-latency), [R-0062](research.md#r-0062--firefly-account-balance-mirror-vs-forecastwealth-inputs), [R-0034](research.md#r-0034--fx-conversion-for-crypto--eur-reporting-currency)
- Architecture: `docs/engineering/architecture.md` § US-0009 (DEC-0049–DEC-0055)
- Intake evidence: `handoffs/intake_evidence/intake-20260608-us0013.json`

**Findings:**

### Root cause (confirmed — infra/feature-completion, not projection bug)

| Gap | Effect |
|-----|--------|
| `stats-forecast` on `full` profile only | External omniflow (`--profile external`) never starts sidecar |
| `[forecast_ml] enabled=false` default (DEC-0049) | Sync skips ML phase; no `ml_enhanced` rows |
| BUG-0010 DONE | Baseline AA/AB/AC satisfied; AC3 explicitly deferred to US-0013 |

### Recommended enablement path (discovery/architecture input)

1. **Compose S1:** Add `stats-forecast` to external overlay with `profiles: [external]` (or shared service on both `full`+`external`); keep `STATS_FORECAST_PORT=8091` on omniflow host.
2. **Config S1:** Document `FORECAST_ML_ENABLED=true` + `STATS_FORECAST_URL=http://stats-forecast:8090` in `.env.example` omniflow block; operator opt-in preserves DEC-0049 default-off elsewhere.
3. **Sync S2:** Existing DEC-0052 phase — verify sidecar health gate + `record_skip_on_baseline` on failure; no sync abort.
4. **UI/Grafana S3:** US-0009 Compare + bands already implemented — wire verification on external profile; Grafana ML panels bind `$forecast_variant=ml_enhanced` (DEC-0055).
5. **Ops S4:** Runbook section: Full sync prerequisite, `min_monthly_points` gate (default 12), health probe, degraded troubleshooting.

### Sprint-plan decomposition (4 vertical slices)

| Slice | Boundary | Independent value |
|-------|----------|-------------------|
| **US-0013-S1** | Compose + env + config merge | Operator can start sidecar on external profile |
| **US-0013-S2** | Sync phase + API `variant=ml_enhanced` | Post-sync ML rows persist |
| **US-0013-S3** | React Compare + wealth overlay + Grafana panels | Operator sees ML in product UI |
| **US-0013-S4** | Runbook + CI mock sidecar test | Ops repeatability without prod secrets |

**Alternatives rejected:**

| Option | Why rejected |
|--------|--------------|
| Merge into BUG-0010 | BUG closed Q0013; AC3 explicitly epic-scoped |
| New US-0017..0020 backlog IDs now | Breaks epic traceability; sprint-plan owns slice IDs |
| Embedded Rust ML (augurs) | R-0044 rejected for US-0009; sidecar path shipped |

**Risks:**

- Short mirror history → SeasonalNaive fallback or skip (DEC-0051) — document in runbook
- Sidecar memory on shared omniflow host — monitor; StatsForecast footprint bounded per R-0044
- FX incomplete crypto (R-0034) → low-confidence banner, not block ML cashflow overlay

**Linked:** US-0013, US-0009, US-0010, BUG-0010, DEC-0049, DEC-0052, DEC-0055, DEC-0066, R-0034, R-0043, R-0044, R-0062  
**Confidence:** high  
**Status:** current — intake synthesis 2026-06-08

### Discovery open questions — research resolution (2026-06-08)

**Web refs:** [Docker Compose profiles merge](https://github.com/docker/compose/pull/7930) (profile arrays additive across files); [Compose networking](https://docs.docker.com/compose/how-tos/networking/) (service-name DNS on shared networks); prior [R-0053 §2](research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci) (external-only service-list CI); [R-0044](research.md#r-0044--statsforecast-sidecar-vs-rust-augurs-execution-model) (sidecar footprint, numba JIT); [R-0045](research.md#r-0045--seasonal-model-selection-autoets-mstl-fallback) (≥12 mo gate); code: `docker-compose.yml`, `docker-compose.external.yml`, `scripts/compose-config-check.sh`, `backend/src/forecast_ml/service.rs`, `backend/src/forecast_ml/sidecar.rs`.

| Topic | Question | Resolution (per research) |
|-------|----------|---------------------------|
| **Profile union** | `profiles: [full, external]` on base vs external-only overlay — avoid duplicate sidecar on `full+external` | **Overlay merge additive profiles** — keep single `stats-forecast` service in base (`profiles: [full]`); external overlay adds `profiles: [external]` + network/port overrides. Compose merges profile arrays → `[full, external]` on one service definition → **one container** when either profile active; no duplicate service name blocks. Reject separate full-only + external-only service definitions (invalid duplicate keys). `full+external` union starts one sidecar; `external` alone starts one sidecar. |
| **Network** | Traefik-only sidecar vs dual-network backend | **Traefik-only co-attachment** on external merge — mirror `flow-finance-ai`/`grafana` pattern: overlay sets `networks: [traefik]` on `stats-forecast`; backend reaches `http://stats-forecast:8090` via embedded DNS on shared network. Dual-network (default + traefik) unnecessary when backend is traefik-only; adds default `_default` network noise. Host port `${STATS_FORECAST_PORT:-8091}:8090` optional for operator curl; internal DNS uses container port 8090. |
| **Sidecar SLO** | Health probe timing on shared omniflow host (R-0044) | **Runtime health gate, not compose `depends_on`** — backend `health_ok()` GET `/health` before ML phase (60s HTTP timeout per `sidecar_timeout_secs`); compose healthcheck `start_period: 30s`, `interval: 30s`, `retries: 3` advisory only. First sync after cold start may record `sidecar_unavailable` skip if sidecar still warming — acceptable per DEC-0052. Numba JIT first forecast ~5s warm (R-0044); document operator smoke: `docker compose … ps` healthy + in-container `curl stats-forecast:8090/health` before expecting ML rows. |
| **Min history** | Production mirror ≥12 monthly points post–Full sync or skip path (DEC-0051) | **No production gate override** — retain `min_monthly_points = 12` default (`default.toml`, `ForecastMlService::recompute` max across asset accounts). Omniflow with successful Full sync should satisfy gate (BUG-0010 confirmed 731 daily forecast rows). If `<12` monthly net-cashflow points → `InsufficientHistory` → `record_skip_on_baseline` with `insufficient_history`; SeasonalNaive ladder (DEC-0051) applies only when gate passes. Runbook: run Full sync before ML enablement smoke. |
| **CI scope** | Compose `config --services` assert vs wiremock integration only | **Both layers** — extend `scripts/compose-config-check.sh`: external-only merge must list `flow-finance-ai`, `grafana`, **`stats-forecast`** (3 services); assert `stats-forecast` joins `traefik` network; retain `cargo test --test forecast_ml_integration` in `tests/run-tests.sh` for sidecar mock + skip-metadata path. Wiremock-only insufficient — leaves US-0010 AC-1 regression gap if overlay sidecar block regresses. |

### Recommended architecture decisions (DEC-0076 area)

| Element | Recommendation | Alternative rejected |
|---------|----------------|----------------------|
| Compose contract | Overlay merge on existing `stats-forecast`; additive `external` profile; traefik network + optional host port remap | Duplicate service name in overlay; `full`-only forever |
| Env opt-in | `FORECAST_ML_ENABLED=true` + `STATS_FORECAST_URL=http://stats-forecast:8090` on `flow-finance-ai` in external merge; DEC-0049 default-off elsewhere | TOML-only enable without env documentation |
| Failure semantics | Unchanged DEC-0052/0066 — health/history/sidecar errors → skip metadata, sync continues | Fail sync on ML error |
| CI guard | Update compose-config-check expected service set + traefik network assert | Wiremock integration only |

**Risks (carry to architecture):**

1. **Cold-start race** — first sync after `compose up` may skip ML until sidecar healthcheck green; runbook must document retry via manual sync
2. **Profile combination** — `minimal+external` or `full+external` on dev host still valid for sidecar; `minimal+external` must not resurrect `firefly-iii` (existing DEC-0056 guard unchanged)
3. **Host memory** — StatsForecast ~80–120 MB RSS on shared omniflow host (R-0044); monitor alongside postgres/grafana
4. **CI drift** — forgetting to update `compose-config-check.sh` when overlay lands silently regresses AC-1 pattern

**Linked (extended):** R-0053, DEC-0056, DEC-0076 (formalized 2026-06-08 architecture)  
**Confidence:** high  
**Status:** fulfilled — released S0014 2026-06-08 (`0.14.0-us0013`); formalized as **DEC-0076**; architecture § US-0013; retain for traceability

---

## R-0072 — US-0014 planning UX epic gap beyond BUG-0011

**Date:** 2026-06-08  
**Topic:** US-0014 re-intake — holistic planning UX polish after BUG-0011 AD/AE/AF release  
**Query:** What UX gaps remain after Q0019 functional fixes; sprint-plan slice boundaries; acceptance decomposition  
**Sources:**
- Prior: [R-0070](docs/engineering/research.md#r-0070--bug-0011-planning-mode-compare-delta-empty-state-api-first-run-ux) (fulfilled Q0019), [R-0015](docs/engineering/research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline), [R-0016](docs/engineering/research.md#r-0016--plan-scenario-versioning-immutable-snapshots-vs-editable-drafts)
- Decisions: **DEC-0073** (overlay-only compare), **DEC-0074** (PVA 200 `no_active_plan`)
- Release: Q0019 shipped AD inline add form, AE overlay delta, AF tagged PVA response
- Web: [Nielsen Norman — empty states](https://www.nngroup.com/articles/empty-state-interface-design/) (actionable empty states with primary CTA); [Material Design — onboarding patterns](https://m3.material.io/foundations/interaction/gestures) (progressive disclosure for first-run flows)

### 1. Gap analysis (post-BUG-0011)

| UX surface | BUG-0011 scope | US-0014 epic scope |
|------------|----------------|-------------------|
| First-run empty state | Wired add form + empty create path (AD) | Template grid + **Create empty plan** primary CTA; all templates visible |
| Compare tab | Correct **0.00** overlay delta (AE/DEC-0073) | Contextual help copy; projected balance footnote |
| Plan vs Actual | 200 `no_active_plan` + basic card (AF/DEC-0074) | Polished guided card with Set active / Scenarios links |
| Templates | Functional apply paths | Discoverability from empty state + existing-plan UI |
| Set active | Implicit via US-0004 | Persistent banner/cue after first create |
| Errors | API paths work | Operator-visible toasts on mutation failure |

**Conclusion:** BUG-0011 closed **broken** behavior; US-0014 closes **confusing** first-visit experience. No backend metric or API contract changes expected unless discovery finds regression.

### 2. Recommended sprint-plan slices

| Slice | Scope | Acceptance rows |
|-------|-------|-----------------|
| **US-0014-S1** | First-run onboarding + templates + Set-active banner | AC-1, AC-5, AC-6 |
| **US-0014-S2** | Add-lines polish + error surfaces | AC-2, AC-7 |
| **US-0014-S3** | Compare help + PVA guided polish + OIDC smoke | AC-3, AC-4, AC-8 |

### 3. Risks (carry to discovery)

1. **Overlap with shipped AD** — discovery must audit Q0019 `PlanningPage.tsx` delta vs AC-1/AC-2 to avoid duplicate execute work
2. **Negative projected balance** — baseline forecast may still show negative month-end on empty overlay; mitigate with help text only (DEC-0073 frozen)
3. **USER_GUIDE_MODE=1** — `docs/user-guides/US-0014.md` required at release; coordinate with US-0032 contract

### 4. Discovery open questions — research resolution (2026-06-08)

Research (`research-20260608-us0014`, orchestrator `auto-20260608-us0014-001`) resolves all four discovery carry-forward items. Detailed analysis: [R-0073](docs/engineering/research.md#r-0073--us-0014-planning-mutation-error-toast-patterns). Recommended decision: **DEC-0077** (planning mutation feedback contract).

| Question | Resolution |
|----------|------------|
| **Error UX** — page-local helper vs per-mutation inline vs global MutationCache? | **Page-local `showPlanningFeedback` helper** — reuse existing green card pattern in `PlanningPage.tsx` / `SubscriptionsPage.tsx`; success `#ecfdf5` / error `#fef2f2`; single active slot. Reject global `MutationCache` + toast library (no dep today; scope creep). |
| **Invalidation** — invalidate `plan-vs-actual` immediately or wait for recompute badge? | **Immediate** on adjustment CRUD, activate, and createPlan success — PVA tab may be open; `plan_stale` badge is advisory not a gate. Extend existing `plan-version` / `plan-compare` invalidation. |
| **Confirmation scope** — toast on every template create vs primary CTA only? | **Required** on createPlan (empty + template), applyTemplate (all templates — extend Q0019 Custom-only path), addAdjustment, activate. **Optional** on update/delete adjustment to avoid edit noise. |
| **User guide** — `docs/user-guides/US-0014.md` section scope? | **Incremental over US-0004** — Purpose (first-run polish); First visit (template grid + Create empty plan); Set active + Grafana Dashboard 3 (`budgets`); Compare overlay-only delta footnote (DEC-0073); brief Troubleshooting (visible mutation errors per DEC-0077). File created at **architecture**; content validated at execute S3. |

**Sprint slice adjustment (post-discovery):** S2 primary (AC-7 + AC-2 invalidation); S1 mostly verify + banner/toasts (AC-5/AC-6); S3 verify shipped AC-3/AC-4 + user guide + OIDC smoke (AC-8).

**Linked:** US-0014, BUG-0011, US-0004, DEC-0073, DEC-0074, DEC-0077, R-0070, R-0073  
**Confidence:** high  
**Status:** fulfilled — released S0015 2026-06-08 (`0.15.0-us0014`); formalized as **DEC-0077**; architecture § US-0014; retain for traceability

---

## R-0073 — US-0014 planning mutation error/toast patterns

**Date:** 2026-06-08  
**Topic:** Planning mutation feedback — page-local helper vs global MutationCache; invalidation timing; confirmation scope  
**Query:** Resolve discovery open questions for AC-7 error surfaces and AC-2/AC-5 feedback before DEC-0077  
**Sources:**
- Prior: [R-0072](docs/engineering/research.md#r-0072--us-0014-planning-ux-epic-gap-beyond-bug-0011), Q0019 `PlanningPage.tsx` audit
- Codebase: `frontend/src/pages/PlanningPage.tsx` (inline green card toast); `frontend/src/pages/SubscriptionsPage.tsx` (same pattern); `frontend/src/lib/api.ts` (`Error` with response body text)
- Web: [TanStack Query mutations guide](https://tanstack.com/query/v5/docs/framework/react/guides/mutations) (`onError` per mutation); [Atomic Object — MutationCache toasts](https://spin.atomicobject.com/toast-notifications-tanstack-query/) (global cache pattern); [GitHub #3441](https://github.com/TanStack/query/discussions/3441) (`meta` opt-out for global handlers)

### 1. Error UX pattern

| Option | Fit | Verdict |
|--------|-----|---------|
| **Page-local `showPlanningFeedback` helper** | Matches existing inline card; 7 mutations on one page; no toast library in repo | **Recommend** |
| Global `MutationCache` + toast lib | Consistent cross-app; requires new dependency + QueryClient refactor | **Reject for US-0014** — scope creep |
| Per-mutation inline only | Works but duplicates `apiFetch` error parsing | **Reject** — extract helper |

**Conclusion:** Single page-local helper with success/error variants; one active message slot (replace on new feedback).

### 2. Invalidation timing

| Option | Verdict |
|--------|---------|
| Invalidate `plan-vs-actual` immediately on adjustment success | **Recommend** — PVA tab may be open in another session; stale rows confuse operators |
| Wait for recompute / `plan_stale` badge clear | **Reject** — adds latency; badge is advisory not gate |

### 3. Confirmation scope

| Path | Toast |
|------|-------|
| `createPlanMutation` (empty + template) | **Required** — AC-5 visible confirmation |
| `applyTemplateMutation` (all templates) | **Required** — extend Q0019 Custom-only path |
| `addAdjustmentMutation` | **Required** — AC-2 success feedback |
| `updateAdjustmentMutation` / `deleteAdjustmentMutation` | **Optional** — avoid noise on frequent edits |

### 4. Error message extraction

`apiFetch` throws `new Error(text || resp.statusText)` for non-2xx. Helper should:

- Truncate body to 240 chars (JSON error payloads)
- Prepend mutation-specific label when body empty
- Never log-only — AC-7 requires operator-visible surface

### 5. User guide scope (`USER_GUIDE_MODE=1`)

US-0014 guide is **delta documentation** over [US-0004](../user-guides/US-0004.md) — not a full planning rewrite.

| Section | Content | Acceptance tie-in |
|---------|---------|-------------------|
| **Purpose** | First-visit polish + visible mutation feedback shipped in US-0014 | Prerequisite row |
| **First visit** | Template card grid, name field, **Create empty plan**, **Create from {template}** | AC-1, AC-5 |
| **Set active** | Yellow banner after create; Plan vs Actual + **Grafana Dashboard 3** (`budgets`) require active plan | AC-6, DEC-0024 |
| **Compare semantics** | Overlay-only monthly delta vs projected balance footnote | AC-3, DEC-0073 |
| **Troubleshooting** | Failed planning actions show red feedback card (not console-only) | AC-7, DEC-0077 |
| **Related documentation** | Link US-0004 for templates, versioning, API examples | DEC-0059 split |

**Timing:** Create `docs/user-guides/US-0014.md` at **architecture** (match US-0013 precedent); execute S3 validates against shipped UI; release gate when `USER_GUIDE_MODE=1`.

**Linked:** US-0014, DEC-0077, R-0072, BUG-0011, DEC-0073, DEC-0074, DEC-0059, US-0032  
**Confidence:** high  
**Status:** fulfilled — released S0015 2026-06-08 (`0.15.0-us0014`); formalized as **DEC-0077** (2026-06-08); extends R-0072 §4; architecture § US-0014; retain for traceability

---

## R-0074 — US-0015 AI forecast bucket mapping (rule+LLM cascade, privacy)

**Date:** 2026-06-06  
**Topic:** US-0015 — AI-assisted income/fixed/variable bucket assignment when Firefly categories missing or ambiguous  
**Query:** Privacy-first transaction categorization patterns; rule-before-LLM cascades; fixed/variable/income bucket decomposition for forecast projection (not chat)  
**Sources:**
- Prior: [R-0060](docs/engineering/research.md#r-0060--bug-0007-ai-merchant-category-discovery-fix-options) (BUG-0007 tool path; six-tool registry; category_search); DEC-0007 category bucket map; DEC-0032 privacy defaults
- Web: [NumbyAI](https://github.com/RoXsaita/NumbyAI-Public) (rule engine → LLM batch for ambiguous rows); [Spendify](https://github.com/drake69/spendify) (4-step cascade: user rules → regex → LLM → fallback + PII sanitization); [finn-tracker](https://github.com/RachithP/finn-tracker) (200+ static rules + learned merchant overrides); [Finima](https://github.com/pacphi/finima) (local Ollama categorization on-device)

### 1. Problem framing

Operators expect `/forecast` **Monthly** tab Income/Fixed/Variable cards to reflect real spending after **BUG-0012** config baseline (Q0014). Remaining gap: mirror rows with **missing Firefly categories**, **custom German labels** outside TOML map, or **ambiguous merchant-only signals** still collapse to Variable or silent mis-bucketing. US-0015 closes AI enrichment on the **projection path** — not chat enumeration (BUG-0007 DONE / DEC-0069).

### 2. Industry pattern (privacy-first cascades)

Open-source personal-finance tools converge on a **deterministic-first, LLM-second** pipeline:

| Stage | Mechanism | US-0015 analog |
|-------|-----------|----------------|
| 1 | Config / user rules | DEC-0007 `[forecast.category_buckets]` + mirror `category_id` (**AC-1**) |
| 2 | Regex / merchant learned rules | Optional operator TOML aliases (discovery scope) |
| 3 | LLM batch for ambiguous rows | New inference module with confidence score (**AC-2**) |
| 4 | Safe fallback | Variable bucket + operator-visible low-confidence flag |

**Privacy:** Local-first tools (NumbyAI, Finima, finn-tracker) keep inference on-device; remote LLM tools (Spendify) **sanitize PII before payload**. Flow Finance AI already enforces aggregate-first via DEC-0032 — US-0015 must **not** widen raw row access by default.

### 3. Recommended MVP path (for `/discovery` → `/architecture`)

**Primary: config map → feature extraction → LLM bucket proposal → projection merge**

1. **Feature extraction (privacy-safe):** category name (if present), normalized merchant token from description (hashed/redacted per DEC-0032), amount sign/magnitude bucket, recurring pattern `display_name`, subscription class — **no raw IBAN/counterparty** under default TOML.
2. **LLM call:** Batch ambiguous rows; structured output `{ bucket: income|fixed|variable, confidence: 0..1, rationale_code }`; reuse US-0008 provider abstraction (OpenAI/Ollama).
3. **Projection merge:** `categorize_delta` applies config first; AI only on map miss; low confidence (&lt; threshold TBD in architecture) → Variable + audit flag.
4. **Operator visibility:** API `bucket_source` + UI **AI-mapped** badge when any AI contribution in month totals (**AC-4/AC-5**).
5. **Audit:** Persist assignment decisions in existing AI audit table pattern (**AC-6**).

**Explicitly deferred:** RAG/vector index (R-0060 §7); seventh chat tool; Firefly write-back; US-0013 ML overlay changes.

### 4. Overlap with BUG-0007 / DEC-0069

| Surface | BUG-0007 (chat) | US-0015 (forecast) |
|---------|-----------------|---------------------|
| Trigger | User natural-language question | Projection recompute on mirror sync |
| Output | Tool JSON to orchestrator | Bucket assignment in `project.rs` loop |
| Category resolution | `category_search` on `get_transactions` | Config map + AI inference module |
| Shared code risk | `categories` mirror, normalization helpers | **Additive module** — do not change six-tool contracts |

**Rule:** Share **category name normalization** and **merchant tokenization** utilities where possible; **do not** route forecast bucketing through chat tools.

### 5. Architecture decision gates (resolved at research 2026-06-06)

| Gate | Research resolution | Formalize at architecture |
|------|---------------------|---------------------------|
| Confidence threshold | **0.75 default** (`ai_bucket_min_confidence` TOML); below → Variable + audit | **DEC-0078** |
| Invalidation | Inline recompute each forecast pass; config-hash bust; no cross-run DB cache MVP | **DEC-0078** |
| Feature allowlist | [R-0075](research.md#r-0075--us-0015-forecast-bucket-privacy-feature-allowlist) | **DEC-0078** + DEC-0032 extension |
| API provenance | Per-month `bucket_sources` map + top-level `ai_mapped` boolean | **DEC-0078** |
| Provider default | Reuse US-0008 `build_provider()`; rule-only when provider absent | **DEC-0078** |
| Merchant aliases TOML | **Defer post-MVP**; category_buckets sufficient for S1 | Note in architecture |

### 6. Acceptance mapping

| Row | Research coverage |
|-----|-----------------|
| Prerequisite BUG-0012 | Config baseline shipped Q0014 — AI extends only |
| AC-1 | Stage 1 config precedence |
| AC-2 | Stage 3 LLM + confidence |
| AC-3 | [R-0075](research.md#r-0075--us-0015-forecast-bucket-privacy-feature-allowlist) |
| AC-4/AC-5 | §7 API provenance |
| AC-6 | Audit persistence |
| AC-7 | Surface isolation + OIDC smoke |

### 7. Discovery open questions — research resolution (2026-06-06)

**Web refs:** [SpendSight hybrid cascade](https://github.com/Zenoguy/SpendSight_) (regex → heuristic → MiniLM → LLM batch; ~76% resolved pre-LLM); [transaction-classifier](https://github.com/Maaz-Zaidi/transaction-classifier) (rules 0.98 confidence short-circuit); [Expense Sorted rules-first](https://www.expensesorted.com/blog/advanced-bank-transaction-categorization-beyond-llms) (60–80% rule coverage; 0.9 auto-apply threshold); [Microsoft agent governance provenance model](https://microsoft.github.io/agent-governance-toolkit/compliance/data-provenance-model/) (source + transformation metadata on derived outputs); code: `backend/src/forecast/project.rs`, `backend/src/api/forecast.rs` (`MonthlyPointResponse`), `backend/src/ai/privacy.rs`, `backend/config/default.toml` `[forecast.category_buckets]`, DEC-0010 recompute trigger, DEC-0014 subscription confidence tiers.

| # | Question | Resolution (per research) |
|---|----------|---------------------------|
| 1 | **Confidence threshold** | **Default 0.75** (`ai_bucket_min_confidence` in `[forecast]` TOML, overridable). Industry cascades use **0.80–0.90** for deterministic rules and **0.75–0.85** for LLM auto-apply ([SpendSight](https://github.com/Zenoguy/SpendSight_), [Expense Sorted](https://www.expensesorted.com/blog/advanced-bank-transaction-categorization-beyond-llms)). Align with household subscription emit floor **60%** (DEC-0014) but use **higher bar for bucket mutation** because mis-bucketing affects monthly cards. **≥ threshold** → apply AI bucket to projection; **&lt; threshold** → Variable + `low_confidence` audit row. Reject 0.90 default (too many Variable fallbacks on German merchant strings). |
| 2 | **Invalidation strategy** | **No persistent assignment cache for MVP.** Each forecast recompute (DEC-0010 post-sync) re-runs cascade on current mirror slice. **Config bust:** hash `[forecast.category_buckets]` (+ future `merchant_aliases`) at inference start; mismatch vs last computation metadata → discard any in-memory memo. **Data bust:** inference input = non-transfer mirror rows for account since rolling window start (reuse rolling.rs lookback). Reject cross-computation Postgres cache table (stale rows after sync edits; operator distrust). Optional stage-2: memoize within single `project_account` call keyed by `payee_fingerprint + category_id + sign` only. |
| 3 | **Privacy allowlist** | See **[R-0075](research.md#r-0075--us-0015-forecast-bucket-privacy-feature-allowlist)** — extend `PrivacyLayer` with `prepare_bucket_features()`; never send raw rows under default TOML. |
| 4 | **API `bucket_source` shape** | Extend `MonthlyPointResponse` with **`bucket_sources: { income, fixed_costs, variable_costs }`** each `config \| ai \| default` (dominant contributor per bucket for that month) plus **`ai_mapped: bool`** when any bucket includes AI-assigned mass. Reject per-daily-point provenance (AC-4 targets monthly cards only). Mixed bucket: label = highest-precedence source present (`config` &gt; `ai` &gt; `default`); `ai_mapped` still true if AI contributed any amount. Frontend badge binds `ai_mapped` (seasonal callout pattern in `ForecastPage.tsx`). |
| 5 | **Provider default** | **Rule-only cascade when LLM unavailable** — reuse **`build_provider()`** (DEC-0043/0044); same provider/mode as chat. No `forecast_ai_*` env split. Batch inference module calls provider only for ambiguous rows after config miss. Provider down → skip LLM stage, Variable fallback, audit `provider_unavailable`. **Ollama/local preferred** for privacy-sensitive operators when configured; OpenAI path unchanged. |
| 6 | **Optional TOML merchant aliases** | **Out of MVP** — stage-2 between config map and LLM per §2 cascade. Existing `[forecast.category_buckets]` + German keys in `default.toml` cover BUG-0012 baseline. Post-MVP: `[forecast.merchant_aliases]` regex→bucket table; architecture documents extension point only. Reject MVP aliases (overlap Firefly categories; operator double-maintenance). |

### 8. Recommended architecture decisions (DEC-0078 area)

| Element | Recommendation | Alternative rejected |
|---------|----------------|----------------------|
| Cascade | DEC-0007 config → LLM batch (ambiguous only) → Variable | Chat tools / RAG index |
| Threshold | 0.75 default TOML | 0.90 (over-fallback); 0.60 (under-safe) |
| Invalidation | Inline per recompute + config hash | Persistent assignment table |
| Privacy | R-0075 feature rows via PrivacyLayer | Widen `allow_raw_transactions` default |
| API | `bucket_sources` + `ai_mapped` on monthly point | Per-tx API provenance |
| Provider | US-0008 shared provider | Separate forecast provider env |
| Aliases | Defer post-MVP | MVP regex table |

**Risks (carry to architecture):**

1. **Threshold calibration** — German merchant-only rows may cluster below 0.75; monitor audit `low_confidence` rate in QA
2. **Rolling residual** — `variable_residual` daily rate stays Variable unless architecture adds aggregate AI split (discovery gap); document as known limitation or stage-2
3. **Provider cost** — batch only ambiguous rows (typically &lt;25% per SpendSight); cap batch size 100/call
4. **Privacy regression** — forecast path must not bypass `PrivacyLayer`; code review gate on S1
5. **Mixed `bucket_sources`** — dominant-source label may hide partial AI; `ai_mapped` boolean is authoritative for badge

**Linked (extended):** R-0075, DEC-0078 (recommended 2026-06-06)  
**Confidence:** high  
**Status:** fulfilled — research 2026-06-06; formalize **DEC-0078** at `/architecture`; retain for traceability

---

## R-0075 — US-0015 forecast bucket privacy feature allowlist

**Date:** 2026-06-06  
**Topic:** AC-3 — privacy-safe feature extraction for batch bucket inference under DEC-0032 defaults  
**Query:** Which merchant/category/amount fields may leave the host for LLM bucket proposal when `allow_raw_transactions=false`  
**Sources:**
- Prior: [R-0028](research.md#r-0028--privacy-layer-middleware-allow_raw_transactions-redaction-semantics) (DEC-0032), [R-0074](research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy) §7
- Code: `backend/src/ai/privacy.rs` (`hash_counterparty`, `is_sensitive_field`, subscription label preserve), `backend/src/forecast/categories.rs`
- Web: [Spendify PII sanitization pattern](https://github.com/drake69/spendify) (sanitize before remote LLM); [Finima on-device Ollama](https://github.com/pacphi/finima) (local inference avoids egress)

### 1. Placement

**New method on existing `PrivacyLayer`** — not a parallel scrubber:

```rust
pub struct BucketFeatureRow {
    pub feature_id: String,           // stable hash for audit, not reversible merchant
    pub category_name: Option<String>,
    pub merchant_token: String,       // hashed when redact_counterparties=true
    pub amount_sign: i8,              // -1 | 0 | 1
    pub magnitude_band: String,       // e.g. "0-50" | "50-200" | "200+"
    pub recurring_label: Option<String>, // subscription display_name when present
    pub pattern_class: Option<String>,    // standing_order | subscription | discretionary
}
```

`prepare_bucket_features(rows: &[MirrorRow]) -> Vec<BucketFeatureRow>` invoked by forecast AI module **before** provider HTTP call. Audit logs `feature_id` + bucket proposal — not raw description.

### 2. Allowlist under `allow_raw_transactions=false` (default)

| Field | Sent to model | Treatment |
|-------|---------------|-----------|
| `category_name` | Yes (if present) | Lowercase trim; empty → omit |
| `merchant_token` | Yes | `hash_counterparty(normalized_payee_or_description)` per DEC-0032 — stable correlate without plaintext |
| `amount_sign` | Yes | Sign only (+ inflow / − outflow) |
| `magnitude_band` | Yes | Log band buckets; **no exact EUR** |
| `recurring_label` | Yes (when from detection) | Same preserve rule as `get_subscriptions` `display_name` |
| `pattern_class` | Yes | Enum string from recurrence/subscription context |
| `description` / `payee` / `counterparty` | **No** | Never in payload under default |
| `iban` / account ids | **No** | Never |
| Exact `amount` | **No** | Band only |

**Batch payload shape (LLM input):**

```json
{
  "task": "forecast_bucket_assignment",
  "features": [
    {
      "feature_id": "bf_a1b2c3d4",
      "category_name": null,
      "merchant_token": "Counterparty-7f3a9b2c",
      "amount_sign": -1,
      "magnitude_band": "50-200",
      "recurring_label": null,
      "pattern_class": "discretionary"
    }
  ]
}
```

Structured output: `{ "assignments": [{ "feature_id", "bucket", "confidence", "rationale_code" }] }`.

### 3. Opt-in `allow_raw_transactions=true`

- Permit **normalized description** (IBAN stripped) for up to **50** ambiguous rows per batch
- Still apply `redact_counterparties` hashing unless operator disables
- Document in user guide as elevated privacy risk; forecast path respects same TOML as chat

### 4. Local provider path

When `ai.provider=ollama` or `openai_compatible` with local base URL, **same allowlist** — consistency over "local = raw OK". Reduces accidental omniflow misconfig leaking descriptions to LAN LLM logs.

### 5. Alternatives considered

| Alternative | Verdict |
|-------------|---------|
| Send category aggregates only (no per-row features) | **Rejected** — insufficient for uncategorized merchant disambiguation (AC-2) |
| Reuse `get_transactions` tool output | **Rejected** — chat contract; aggregates lack per-row bucket assignment |
| New seventh chat tool | **Rejected** — DEC-0069 / AC-7 isolation |

**Linked:** US-0015, DEC-0032, DEC-0069, R-0074, R-0060, DEC-0078  
**Confidence:** high  
**Status:** fulfilled — research 2026-06-06; implemented US-0015/S0016/DEC-0078 (`PrivacyLayer::prepare_bucket_features`); retain for traceability

---

## R-0076 — Omniflow analytics regression hypotheses (post-US-0015)

**Date:** 2026-06-06  
**Topic:** BUG-0013 intake — operator report of Grafana zeros, crypto €0, budgets MTD implausible sums, ds/query Failed to fetch on `financegnome.omniflow.cc` after US-0015 release  
**Query:** Likely root-cause clusters for post-release analytics regression vs stale deploy / missing recompute  
**Sources:**
- Prior: [R-0056](research.md#r-0056--grafana-embed-root-url-and-same-origin-asset-proxy), BUG-0009 discovery (account variable default), BUG-0010 (balance NULL → starting_balance 0), BUG-0005 (futures ingest), US-0015 release notes S0016
- Operator report 2026-06-06 (verbatim themes)
- Grafana community: annotation/query `Failed to fetch` often indicates datasource proxy misconfig or CORS/auth on `ds/query` path

### 1. Deploy-and-recompute gate (highest probability first)

| Check | If skipped | Symptom cluster |
|-------|------------|-----------------|
| **BACKEND_FRONTEND_DEPLOY** post S0016 | Old backend/frontend on omniflow | Forecast 0 €, bucket mapping not active, stale panels |
| **Full Firefly sync** | Mirror stale | Empty subscriptions price changes, zero balances |
| **Forecast recompute** post sync | `forecast_balance_daily` flat 0 | AI cashflow + forecast-horizons emptiness |
| **Exchange sync phase** | No crypto positions | AK/AN crypto €0 despite keys configured |

Discovery **must** record terminal sync run + recompute timestamps before attributing code regression.

### 2. Symptom → subsystem map

| Sub-defect | Primary subsystem | Secondary |
|------------|-------------------|-----------|
| AI | `forecast_balance_daily`, Grafana `$account_id` default | BUG-0009 Y1 regression |
| AJ | `subscription_price_changes` detection + subscriptions dashboard SQL | — |
| AK/AN | `exchange` sync, `wealth` FX pricing, `net_worth_snapshots` | BUG-0005 futures path |
| AL | budgets dashboard SQL, plan currency join | active plan `test v1` scale |
| AM | `/analytics/grafana` reverse proxy, datasource UID, Traefik auth | BUG-0003 H pattern |

### 3. Regression vs configuration

- **Not reopen BUG-0009/0010** without proof prior fix reverted — operator cluster may be **deploy smoke gap** alone.
- **ML unavailable** on seasonal panel is **expected** when US-0013 not enabled — baseline panels must still be non-zero (AI sub-defect).
- **MetaMask `contentscript.js` warnings** — browser extension noise; exclude from defect scope.

### 4. README coupling (US-0017)

Living-doc expansion should document the deploy+sync+recompute prerequisite in Troubleshooting to reduce false regression reports — orthogonal to BUG-0013 code fixes.

**Linked:** BUG-0013, US-0017, intake `intake-20260606-omniflow-regression-readme`  
**Confidence:** medium (pending discovery probes on live omniflow)  
**Status:** fulfilled — Q0020 released via DEC-0079 (AL1) + DEC-0080 (AN1); retain for traceability

### 5. Discovery verdicts (2026-06-08 — `discovery-20260608-bug0013`)

| Sub-defect | Verdict | Research implication |
|------------|---------|---------------------|
| AI | Refuted (ops/stale) | No code sprint; operator re-smoke acct **114** post-recompute |
| AJ | Refuted (expected empty) | Optional empty-state copy only |
| AK/AN | **Confirmed** | Linear futures EUR valuation gap — extend §6 |
| AL | **Confirmed** | MTD SQL upper bound — §7 |
| AM | Not reproduced (curl 200) | Browser/embed hypothesis — **[R-0077](research.md#r-0077--bug-0013-grafana-embed-failed-to-fetch-annotation-runner)** |
| AN | **Confirmed** | Same root as AK |

**Isolation vs US-0015:** Multi-factor cluster — **not** a single US-0015 regression. Execute scope: **AL1**, **AN1**, **V1**.

### 6. Linear futures EUR valuation options (AK/AN — extends BUG-0005 / DEC-0064)

**Context (discovery + code audit):** Omniflow holds **7** Bitunix `product_type=linear` rows (`INJUSDT`, `SOLUSDT`, …) with `market_value_eur` **NULL** → portfolio **Crypto value €0**, `holdings_top` empty. Sync succeeds; gap is **valuation phase**, not ingest. `recompute_pnl` passes **`ExchangePriceBook::default()`** (empty) — tickers never loaded per [R-0034](research.md#r-0034--fx-conversion-for-crypto--eur-reporting-currency). `holding_value_eur` calls `fx.to_eur(qty, asset)` where `asset` is full symbol (`INJUSDT`) → `FxError::Unpriced`.

**DEC-0064 contract (still binding):** Futures **wallet** (`product_type=futures`) priced in wealth subtotal; **linear positions** keep `market_value_usd=None` in subtotal to avoid double-count with wallet equity. Positions contribute via **unrealized PnL** per [R-0033](research.md#r-0033--portfolio-pnl-methodology-realized-unrealized-total-return).

**Bitunix payload fields** ([Get Pending Positions](https://www.bitunix.com/api-docs/futures/position/get_pending_positions.html), [Get Single Account](https://www.bitunix.com/api-docs/futures/account/get_single_account.html)):

| Field | Source | Use |
|-------|--------|-----|
| `unrealizedPNL` | Position row | Primary derivatives unrealized (USDT-denominated) |
| `entryValue` | Position row | Position notional / entry exposure proxy |
| `margin` | Position row | Locked collateral per position |
| `available` + `margin` + `frozen` | Account array | Wallet equity components |
| `crossUnrealizedPNL` / `isolationUnrealizedPNL` | Account row | Account-level unrealized aggregate |

**Code audit gap:** `parse_futures_wallet` expects `data.account` object; Bitunix returns `data: [{ marginCoin, available, … }]`. Array shape may prevent wallet row creation — explains **positions-only** holdings with **€0** crypto when collateral sits in account API not position rows.

| Option | Mechanism | Fits DEC-0064 | Risks |
|--------|-----------|---------------|-------|
| **A — Wallet equity fix (recommended tier 1)** | Fix array parse; `product_type=futures`, `market_value_usd=equity` for USDT; `fx.usd_to_eur` in `recompute_pnl` | **Yes** — wallet in subtotal | Alt margin coins deferred; cross-margin equity semantics |
| **B — Exchange unrealized → EUR (recommended tier 1)** | Map `unrealized_pnl` from position payload at upsert; convert USDT→EUR in `recompute_pnl`; sum into `unrealized_eur` / snapshot | **Yes** — notional excluded; PnL surfaced | Does not alone fix **crypto value** stat if wallet missing |
| **C — Symbol→base + ticker price book (tier 2)** | Strip `USDT` suffix (`INJUSDT`→`INJ`); populate `ExchangePriceBook` from exchange tickers during recompute; spot-style `qty × mark` | **Partial** — spot pattern only | Linear **qty** is contracts not base units; wrong for futures without contract size |
| **D — Persist `entryValue` notional (tier 2)** | Parse `entryValue` as USD exposure; store `exposure_usd`; EUR for **display column** separate from `market_value_eur` | **Needs DEC amend** — exposure vs subtotal | Double-count if wallet equity also includes margin; mark vs entry stale |
| **E — Full notional in `market_value_eur` (rejected)** | `markPrice × qty` or `entryValue` in wealth subtotal | **No** — violates DEC-0064 | Double-count with wallet equity |

**Provisional recommendation (architecture):** **A + B** as AN1 MVP — fix wallet parse + price USDT wallet; convert position `unrealizedPNL` to `unrealized_pnl_eur`. Extend **crypto_value_eur** snapshot to `sum(market_value_eur)` (wallet) **or** document panel uses wallet-only per DEC-0064. If acceptance AK requires non-zero **crypto** when positions-only, add **D** as `exposure_eur` display field (Grafana panel / API) without merging into wealth subtotal — **decision gate** for DEC-0064 narrow amendment.

**Secondary gap:** Populate `ExchangePriceBook` during recompute for **spot** holdings (R-0034 intent); orthogonal but same code path.

**Sources:** [R-0033](research.md#r-0033--portfolio-pnl-methodology-realized-unrealized-total-return), [R-0034](research.md#r-0034--fx-conversion-for-crypto--eur-reporting-currency), [R-0059](research.md#r-0059--exchange-multi-product-sync-scope-bitunix-futures), DEC-0064; `backend/src/exchanges/bitunix.rs`, `backend/src/portfolio/service.rs`, `backend/src/portfolio/pnl.rs`, `backend/src/fx/service.rs`

### 7. Budgets MTD SQL (AL — confirmed)

**Root cause:** `budgets.json` panel id **5** — `planned` CTE filters `pdc.ts >= date_trunc('month', CURRENT_DATE)` **without** `<= CURRENT_DATE`. Dashboard `time.to` includes `now+30d`; plan horizon sums **730** future days → **−€150K** MTD.

**Fix (AL1):** Add `AND pdc.ts::date <= CURRENT_DATE` to planned CTE (and deviation joins if mirrored). Optional footnote when active plan starts mid-month (plan starts **2026-06-07**, today **2026-06-06** → correct capped MTD **0**).

**Alternatives:** Cap via `$__timeFilter` on MTD stat — rejected (stat panels lack reliable time filter for scalar SUM). Separate MTD materialized view — rejected (over-engineered for provisioning-only fix).

**Linked:** `grafana/provisioning/dashboards/analytics/budgets.json` panel 5

**Updated:** BUG-0013 discovery 2026-06-08  
**Confidence:** high (AL, AK/AN); medium (pricing option selection pending architecture)  
**Status:** fulfilled — Q0020/DEC-0079/DEC-0080 shipped; AM waived per R-0077; retain for traceability

---

## R-0077 — BUG-0013 Grafana embed Failed to fetch / annotation runner (AM)

**Date:** 2026-06-08  
**Topic:** BUG-0013 sub-defect **AM** — browser `handleAnnotationQueryRunnerError TypeError: Failed to fetch` on analytics embed; curl reproduces **200** on `ds/query` + `/api/annotations`  
**Query:** WebSocket, subpath, CORS, and Grafana 11 client-side failure modes when HTTP transport passes  
**Sources:**
- [R-0056](research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik) — prefix strip, WS `/api/live/`, same-origin proxy (DEC-0057)
- [Grafana annotations cancelled requests #85292](https://github.com/grafana/grafana/issues/85292) — `status: -1`, `Request was aborted`; logged as 500 server-side
- [Grafana community — DashboardQueryRunner failed to fetch](https://community.grafana.com/t/dashboardqueryrunner-failed-failed-to-fetch/89118) — `handleAnnotationQueryRunnerError` + default **Annotations & Alerts** source
- [Grafana proxy subpath #16135](https://github.com/grafana/grafana/issues/16135) — `root_url` must match browser URL
- Discovery probes 2026-06-08: `POST /analytics/grafana/api/ds/query` **200**; `GET …/api/annotations` **200**; `<base href="/analytics/grafana/">` present; raw WS without upgrade **400** (expected)
- Repo: `backend/src/analytics/proxy.rs`, `docker-compose.yml` (`GF_SERVER_ROOT_URL`), `grafana/provisioning/dashboards/analytics/budgets.json` (built-in dashboard annotation only)

### 1. Ruled out (discovery + R-0056)

| Hypothesis | Verdict | Evidence |
|------------|---------|----------|
| **CORS cross-origin** | **Unlikely** | Same-origin `/analytics/grafana/*` proxy; no third-party Grafana host |
| **ds/query transport failure** | **Refuted** | curl **200** with anonymous Viewer behind edge |
| **Traefik auth blocking API** | **Unlikely** | curl without browser still hits same paths; basic auth reused same-origin in iframe |
| **Missing proxy** | **Refuted** | US-0011 shipped; assets **200** under prefix |

### 2. Plausible causes (ranked)

| Rank | Cause | Mechanism | Fix direction |
|------|-------|-----------|---------------|
| **1** | **Annotation request cancellation** | Grafana 11.3+ aborts in-flight `GET api/annotations` when dashboard refreshes (30s on budgets) or panel re-queries; browser surfaces `Failed to fetch` / `NetworkError` though server may log `context canceled` ([#85292](https://github.com/grafana/grafana/issues/85292)) | **AM-defer:** treat as cosmetic unless panels fail to load; optional disable built-in **Annotations & Alerts** on dashboards with `annotations.list: []` (5/6 already empty; **budgets** has built-in dashboard annotation) |
| **2** | **WebSocket `/api/live/` proxy gap** | Live refresh uses WS; HTTP ds/query succeeds while live channel stale — errors may surface in console without blocking static SQL panels ([R-0056 §4](research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik)) | QA browser smoke: DevTools WS to `/analytics/grafana/api/live/` **101**; verify `proxy.rs` upgrade path |
| **3** | **`GF_SERVER_ROOT_URL` drift** | Mis-set root_url breaks relative fetches in iframe ([Grafana #16135](https://github.com/grafana/grafana/issues/16135)) | Omniflow default set in compose; operator override audit only |
| **4** | **Extension / client noise** | MetaMask `contentscript.js` — **out of scope** per acceptance | Exclude from defect |

### 3. Provisional recommendation (architecture)

- **Do not reopen DEC-0057** without authenticated browser HAR showing **non-200** on `ds/query` or annotations.
- **AM1 execute (if needed):** (a) remove or disable built-in dashboard annotation on `budgets.json`; (b) add QA checklist item — browser WS **101** on live route; (c) suppress/log-level only for cancelled annotation fetches if Grafana upstream documents fix.
- **Acceptance AM:** Pass when panels render data and curl **200** holds; console-only annotation abort **waived** unless operator proves panel breakage.

**Alternatives considered:**
- *Proxy HTML rewrite for subpath* — rejected unless `GF_SERVER_ROOT_URL` smoke fails (BUG-0001 B1 already shipped)
- *Public Grafana host* — rejected (R-0054 / DEC-0056)
- *CORS headers on proxy* — rejected (same-origin; CORS not applicable)

**Risks:**
- Chasing AM may distract from confirmed **AL1** + **AN1** fixes
- WS proxy failures silent until operator opens DevTools
- Disabling annotations removes low-value "No active plan" built-in marker on budgets dashboard

**Linked:** BUG-0013, US-0011, DEC-0057, R-0056, R-0076 §5  
**Confidence:** medium (HTTP refuted; browser failure mode inferred from Grafana issues + community reports)  
**Status:** fulfilled — AM waived at architecture; pass-with-prerequisites at Q0020 release; console-only annotation abort out of scope; retain for traceability

---

## R-0078 — US-0017 README omniflow smoke templates, H3 layout, validate_doc_profile gates

**Date:** 2026-06-09  
**Topic:** US-0017 — root README Examples/Troubleshooting expansion; per-segment Product status maintenance  
**Query:** Omniflow `curl` template contract; budget-safe H3 placement; `validate_doc_profile` gates for balanced profile; maintenance hook wording for release segment  
**Sources:**
- Repo: `README.md`, `scripts/doc_profile_lib.py`, `scripts/validate_doc_profile.py`, `docs/developer/README.md`, `docs/engineering/runbook.md` (§ README maintenance, § Omniflow AC-6, §23 BUG-0013)
- UAT: `sprints/quick/Q0020/uat.md` — operator gates + live probe rows AL–REG
- Prior: [R-0066](research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance), [R-0067](research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks), [R-0053 §6](research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci), [R-0077](research.md#r-0077--bug-0013-grafana-embed-failed-to-fetch--annotation-runner-am)
- Normative: **DEC-0070**, **DEC-0059** (doc audience profile)

### 1. H3 layout contract (extends R-0067)

`count_profile_root_h2s` (`doc_profile_lib.py`) counts **only** required `USER_*` H2 titles for the active profile — **H3 subsections are not budgeted**. For `(both, balanced)` the required user H2 set is Purpose, Quickstart, Limitations, Examples, Related documentation (5 titles); `## Contributing` is a pointer and does not increment the budget counter.

| Placement | Pros | Cons | Verdict |
|-----------|------|------|---------|
| `### Omniflow smoke (external profile)` under `## Examples` | Omniflow curls beside localhost samples; no new TOC H2 | Slightly lengthens Examples | **Recommend** |
| `### Troubleshooting` under `## Limitations` | Satisfies AC "Limitations or budget-safe Troubleshooting subsection"; keeps ML-unavailable context adjacent | Troubleshooting nested under Limitations semantically | **Recommend** (balanced profile does not require `USER_TROUBLESHOOTING` as root H2 — only `technical-deep` adds `## Troubleshooting`) |
| Dedicated `## Troubleshooting` H2 | Visible TOC entry | Consumes H2 budget; redundant with Limitations for balanced profile | **Reject** |
| Expand Limitations bullets only (no H3) | Minimal diff | Harder to scan symptom table; AC asks for subsection | **Reject** |

**Architecture carry:** formalize as DEC-0070 extension — no new root H2; two H3 additions only.

### 2. Omniflow curl template (canonical for README `### Omniflow smoke`)

**Host:** `https://financegnome.omniflow.cc` (override via `TRAEFIK_HOST` in operator `.env` — document placeholder, not hardcode alternate hosts in README body beyond default).

**Edge auth:** Traefik `auth` basic-auth on public routes — use placeholder `-u '<basic-auth-user>:<pass>'` per runbook § Omniflow AC-6 (consistent with R-0053 §6). **Never** commit operator credentials. API routes (`/api/v1/*`) additionally require OIDC session or `AUTH_DEV_BYPASS=true` on external profile — note in README one-liner; full matrix stays in runbook.

**Copy-paste block (README Examples H3):**

```bash
OMNI=https://financegnome.omniflow.cc
AUTH='-u <basic-auth-user>:<pass>'   # Traefik edge only — replace placeholders

# Health
curl -sf "$OMNI/health" $AUTH

# Sync status + entity counts
curl -s "$OMNI/api/v1/sync/status" | jq .
curl -s "$OMNI/api/v1/sync/entities" | jq .

# Manual Full sync (requires Firefly PAT + running backend)
curl -s -X POST "$OMNI/api/v1/sync/trigger" \
  -H 'Content-Type: application/json' \
  -d '{"mode":"full"}' | jq .

# Forecast recompute signal (after Full sync from SPA or trigger above)
curl -s "$OMNI/api/v1/forecast/meta" | jq '.last_computed_at, .computation_id'

# Exchange / crypto sanity
curl -s "$OMNI/api/v1/wealth" | jq '.crypto.subtotal_eur, .total_eur'

# Grafana embed proxy health
curl -s -o /dev/null -w '%{http_code}\n' "$OMNI/analytics/grafana/api/health" $AUTH
```

**Six SPA analytics routes** (table already in README — extend with full URLs or keep slug column + `OMNI` prefix note):

| Route | Smoke focus |
|-------|-------------|
| `/analytics/cashflow` | Baseline balances acct **114** (not **116**) |
| `/analytics/subscriptions` | Price-changes panel or documented empty-state |
| `/analytics/budgets` | MTD planned/actual plausible post-DEC-0079 |
| `/analytics/portfolio` | Crypto stat non-zero post-DEC-0080 |
| `/analytics/forecast-horizons` | Baseline + optional ML banner |
| `/analytics/platform-health` | Stack health |

**Operator gates** (link runbook §23; one-liner in README H3): **BACKEND_FRONTEND_DEPLOY** → **GRAFANA_PROVISIONING_RELOAD** → **FULL_FIREFLY_SYNC** + forecast recompute before attributing flat **0 €** panels to code defects (Q0020 uat.md).

**Anti-pattern (R-0066):** do not duplicate full runbook §23 table in README — keep essentials + deep link.

### 3. Troubleshooting H3 content contract

Place under `## Limitations` → `### Troubleshooting`. Lead with gate sequence, then symptom table.

| Symptom | Likely cause | Operator action |
|---------|--------------|-----------------|
| All analytics panels flat **0 €** after deploy | Stale image / gates skipped | BACKEND_FRONTEND_DEPLOY → GRAFANA_PROVISIONING_RELOAD → FULL_FIREFLY_SYNC + recompute |
| Budgets MTD **−€150K** planned, **€0** actual | Pre-DEC-0079 MTD SQL artifact | Deploy DEC-0079 build + Grafana reload; runbook §23 Row AL |
| Crypto **€0** in wealth/portfolio | Pre-DEC-0080 pricing or exchanges-only sync | Deploy DEC-0080 build + Full sync + exchange sync; `crypto.subtotal_eur` probe |
| Forecast **0 €** on default panels | Wrong `$account_id` or no recompute | Full sync; verify acct **114** (not **116**); BUG-0013 **AI** ops verdict |
| **ML unavailable** banner on forecast-horizons | ML overlay off (US-0013 / DEC-0049) | **Expected** — baseline statistical forecast still applies; not data-missing |
| Grafana **Failed to fetch** (browser console) | Embed annotation cancel / WS edge (R-0077) | curl ds/query **200**; Traefik session; do not Save dashboard overrides |

**Distinction (AC-2):** empty Grafana SQL panels after gates = data/deploy defect; **ML unavailable** banner = honest degraded mode when sidecar disabled.

### 4. validate_doc_profile gates

**Runtime proof (2026-06-09 research):** `python3 scripts/validate_doc_profile.py --repo . --no-template-parity` → `[DOC_PROFILE_VALIDATE_OK]` exit **0**.

| Check | Pre-execute (today) | Post-execute requirement |
|-------|-------------------|--------------------------|
| Profile | `(both, balanced)` from scratchpad defaults | Unchanged |
| Root H2 budget | 5 required user H2s counted; 6 physical H2s incl. Contributing; budget **8** | No new root H2; H3 additions allowed |
| `--no-template-parity` | **Required** (`template/` absent per R-0067) | Unchanged until template tree ships |
| `DEV_*` in root | Absent | Must remain absent (DEC-0059 split) |
| Optional crosslink weak | `USER_GUIDE_MODE=1` — user-guides mention present | Preserve |

**Execute gate:** non-zero `validate_doc_profile` → fail closed; remediation → runbook § README maintenance.

### 5. Maintenance per-segment wording (extends R-0067 §3)

US-0017 tightens hooks so **each** closed US/BUG in the **release segment** (sprint or quick task batch) gets a Product status bullet — not implied by vague "closes backlog items".

**Recommended release hook delta** (runbook § README maintenance + `docs/developer/README.md` Quality gates):

> For **each** US or BUG that transitions to **DONE** / **CLOSED** in the **current release segment** (target sprint, quick task, or paired intake batch), append one bullet to root `README.md` **`### Product status`** …

**Recommended refresh-context hook delta:**

> When the **release segment** or sprint artifacts closed **one or more** US/BUG ids since the prior refresh, verify **each** closed id appears in Product status; update missing bullets before completing refresh.

**Product status at execute:** post-Q0020 refresh already lists US-0015, BUG-0013, US-0013–0016 — AC-3 is **verify-only** at execute unless segment closes additional work.

### Alternatives considered

- *Dedicated `## Troubleshooting` H2* — rejected (balanced profile + H2 budget; R-0067 scannability precedent)
- *Auto-generated README on commit* — rejected (R-0066 / DEC-0070 phase-boundary cadence)
- *Full runbook §23 inline in README* — rejected (noise; link instead)

### Risks

1. **Dual auth confusion** — operators mix Traefik basic-auth with OIDC API session; mitigate with explicit "edge vs API" note in H3
2. **Placeholder hygiene** — angle-bracket placeholders must not resemble real credentials in examples
3. **Segment definition drift** — "release segment" needs architecture one-liner (sprint id or quick task id scope)
4. **README length creep** — symptom table + curls approach H2 scannability limit; cap prose, link runbook

**Linked:** US-0017, US-0016, DEC-0070, DEC-0059, R-0066, R-0067, R-0053, R-0077, BUG-0013, Q0020  
**Confidence:** high  
**Status:** fulfilled — US-0017 released Q0021 via **DEC-0070** extension (2026-06-09); R-0066/R-0067 retained for traceability

---

## R-0079 — BUG-0014 post-rebuild omniflow (ML sidecar, crypto display, Grafana, planning)

**Date:** 2026-06-07  
**Topic:** Operator post-rebuild smoke on `financegnome.omniflow.cc` — ML banner, crypto €0, Grafana cashflow zeros, planning delete gap  
**Query:** Intake synthesis for **BUG-0014** after operator rebuilt `flow-finance-ai` + `grafana` with `FORECAST_ML_ENABLED=true`  
**Sources:**
- Live probes (2026-06-07): `GET /api/v1/forecast/meta`, `GET /api/v1/wealth`, `docker ps` (no `stats-forecast`)
- Operator report + cashflow screenshot; env vars: `BITUNIX_ENABLED_FUTURES=true`, `STATS_FORECAST_URL`, Grafana embed paths
- Code: `docker-compose.external.yml` (`stats-forecast` `profiles: [external]`), `frontend/src/pages/PlanningPage.tsx` (no plan delete), `frontend/src/pages/WealthPage.tsx` (FX banners)
- Prior: [R-0071](research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile), [R-0076](research.md#r-0076--omniflow-analytics-regression-hypotheses-post-us-0015), [R-0034](research.md#r-0034--fx-conversion-for-crypto--eur-reporting-currency), **DEC-0080**, **DEC-0076**

**Findings:**

| Sub | Verdict | Root cause (intake) |
|-----|---------|---------------------|
| **AO** | **CONFIRMED (ops)** | `FORECAST_ML_ENABLED=true` in app container; **`stats-forecast` not running** → `ml_skipped_reason: sidecar_unavailable`. Rebuild command scoped to two services only (**AT**). |
| **AT** | **CONFIRMED (ops)** | External overlay defines sidecar; operator `compose up` did not include `stats-forecast`. Runbook/README gate should list **three** services when ML on. |
| **AP** | **CONFIRMED (display/pricing)** | Wealth API: `holdings_count: 7`, `crypto.subtotal_eur: 0`, `pnl.unrealized_eur: 411.74`, `holdings_top: []` — unrealized priced but subtotal/cards empty; **DEC-0080** wallet equity may not surface in subtotal UI. |
| **AQ** | **LIKELY (product gap)** | Operator wants **native currency** + point-in-time EUR via CEX/public FX; tier-2 price book deferred in DEC-0080; portfolio forecast `fx_incomplete_warning` path separate from wealth `fx_incomplete`. |
| **AR** | **LIKELY (ops/stale)** | Screenshot shows flat **0** balances; prior **BUG-0013 AI** refuted when acct **114** + recompute — re-verify `$account_id`, Full sync, panel SQL post-Q0020. |
| **AS** | **CONFIRMED (UI gap)** | `DELETE /api/v1/plans/:id` in backend; **no** delete-plan in React; `target_type` enum **household/subscription/account** by design — UX/doc gap post-US-0014. |

**Recommended discovery order:** AT/AO (start sidecar + Full sync) → AP/AQ (holdings DB + pricing) → AR (Grafana variable + SQL) → AS (plan delete UI spike).

**Alternatives rejected:** Reopen **BUG-0013** — closed Q0020 with operator gates; residual symptoms are new cluster. Merge into **US-0013** — epic DONE; defect is deployment/operator scope + display residuals.

### 5. Discovery verdicts (2026-06-09 — `discovery-20260607-bug0014`)

Code audit + intake probes; no live host access in discovery subagent.

| ID | Verdict | Evidence |
|----|---------|----------|
| **AO** | **CONFIRMED (ops)** | `stats-forecast` absent; `ml_skipped_reason: sidecar_unavailable` with `FORECAST_ML_ENABLED=true`; Grafana `forecast-horizons.json` static US-0013 banner misleading when env opts in |
| **AT** | **CONFIRMED (ops)** | `docker-compose.external.yml` lines 54–59 define sidecar `profiles: [external]`; operator rebuild two-service only |
| **AP** | **CONFIRMED (code residual)** | `wealth/service.rs` subtotal = `sum(market_value_eur)`; probe unrealized **411.74** vs subtotal **0**; `holdings_top` filters priced rows only — wallet row or deploy gap |
| **AQ** | **CONFIRMED (product gap)** | `unpriced_assets` hardcoded `[]` in wealth breakdown; `portfolio_forecast.fx_incomplete_warning` separate path; crypto holdings table empty |
| **AR** | **LIKELY (ops/stale)** | `cashflow.json` SQL unchanged; BUG-0013 AI refuted acct **114**; June 2028 screenshot = wrong range/account hypothesis |
| **AS** | **CONFIRMED (UI gap)** | `PlanningPage.tsx` adjustment delete only; backend `delete_plan` wired |

**Research carry-forward:** AP1 wallet row SQL probe; AP2 subtotal vs unrealized display; AQ1 all-holdings native+EUR; AS1 active-plan delete guard; AO1 dynamic ML banner; AR1 API vs Grafana divergence test.

**Linked:** BUG-0014, BUG-0013, US-0013, US-0014, DEC-0076, DEC-0080, R-0034, R-0071  
**Confidence:** high (AO/AT/AP/AQ/AS); medium (AR — pending operator re-smoke)  
**Status:** fulfilled — BUG-0014 released Q0022 via **DEC-0081**, **DEC-0082**, **DEC-0083** (2026-06-07); retain for traceability

### 6. Research phase (2026-06-09 — `research-20260607-bug0014`)

Code audit of wealth aggregation, PnL recompute, planning UI, Grafana dashboards, and forecast meta paths. No host `.env` / `.env_prod` read. Web: Grafana native text panels are static markdown only; dynamic ML status requires a Postgres query variable or combined static copy (Grafana docs, community workaround #82993).

#### AP — crypto subtotal €0 vs unrealized €411.74 (`holdings_top` empty)

**Trace (DEC-0080 / DEC-0064):**

| Layer | Contract | Live symptom |
|-------|----------|--------------|
| `bitunix.rs` | `parse_futures_wallet` → `product_type=futures`, USDT equity → `market_value_usd` | Q0020 code present in repo |
| `pnl.rs` `compute_hybrid_pnl` | Futures: `fx.to_eur(quantity, asset)` → `update_holding_eur(..., Some(mv), ...)`; linear: skip subtotal, convert `unrealizedPNL` → EUR | Probe: `unrealized_eur=411.74` ⇒ recompute path ran for linear |
| `wealth/service.rs` | `subtotal_eur = sum(market_value_eur)`; `holdings_top` = top-5 priced rows only; `holdings_count` = **all** rows | Subtotal **0** + count **7** ⇒ **no priced rows** (wallet `market_value_eur` still NULL) |

**Root-cause ranking:**

1. **Deploy gate (most likely)** — Operator rebuilt 2026-06-07; Q0020 verify-work listed **BACKEND_FRONTEND_DEPLOY PENDING** (`sprints/quick/Q0020/uat.md`). Host may still run pre-DEC-0080 image or stale DB without futures wallet row.
2. **Ops gate** — Exchanges-only sync without Full sync / PnL recompute after deploy leaves `market_value_eur` NULL even on Q0020 image.
3. **Code residual (if AP1 fails)** — Unlikely new `bitunix.rs` bug given unit tests; wealth layer has no fallback to `portfolio.latest().crypto_value_eur` when holdings sum is zero but unrealized &gt; 0.

**AP1 verify gate (mandatory before AP2 code):**

```sql
-- Wallet row must exist after Full sync + exchange recompute on Q0020+ image
SELECT product_type, asset, quantity, market_value_eur, unrealized_pnl_eur
FROM exchange_holdings WHERE exchange_id = 'bitunix' ORDER BY product_type, asset;
```

| AP1 outcome | Next |
|-------------|------|
| No `futures` row or all `market_value_eur` NULL after deploy + Full sync | Ops: redeploy Q0020 + Full sync; re-probe |
| `futures` row priced, API still `subtotal_eur=0` | **AP2** — wealth aggregation bug (`wealth/service.rs` lines 126–158) |
| Priced wallet, subtotal &gt; 0 | AP closed — was deploy/stale |

**AP2 recommended fix surface (architecture):** Keep DEC-0064/DEC-0080 subtotal rules. Optional hardening: if `sum(market_value_eur)==0` but `pnl.crypto_value_eur>0`, use portfolio snapshot for subtotal (defensive). Clarify exchange card `holdings_count` — split wallet vs open contracts or annotate “N positions (M priced)”.

**Risks:** Fixing count-only UX without wallet deploy still confuses operator; merging unrealized into subtotal violates DEC-0064.

#### AQ — native currency + EUR display; `unpriced_assets` wiring

**Gap analysis:**

| Surface | Current | Gap |
|---------|---------|-----|
| `wealth/service.rs` | `unpriced_assets` always `[]`; `fx_incomplete` only when list non-empty | Never surfaces `pnl.unpriced_assets` from `compute_hybrid_pnl` |
| `portfolio_forecast.rs` | `fx_incomplete_warning` passed from `extended.fx_incomplete` | Always false when wealth list empty — crypto tab portfolio banner may disagree with operator expectation |
| `WealthPage.tsx` holdings table | Renders `holdings_top` only (priced rows) | 7 holdings, empty table; no native qty + EUR pairs for linear |
| `ExchangePriceBook` | `default()` empty in `recompute_pnl` | Spot alts remain tier-2 deferred per DEC-0080 |

**Display contract (recommended for architecture — candidate DEC):**

- **All holdings** in API (`holdings` or extend `holdings_top` → `holdings_all`, capped e.g. 50): `asset`, `quantity`, `product_type`, `value_eur: Option<f64>`, `unrealized_pnl_eur`, `native_unit` (margin coin for futures wallet; symbol for linear).
- **Linear rows:** show native contract qty + symbol + unrealized EUR; **do not** add notional to subtotal (DEC-0064).
- **FX banner:** single gate — `fx_incomplete = pnl.fx_incomplete || !unpriced_assets.is_empty()`; list assets from PnL breakdown; portfolio-forecast warning uses same flag.
- **Tier-2 pricing:** defer `ExchangePriceBook` population to post-BUG-0014 unless architecture accepts scope creep.

**AQ fix surface:** `backend/src/wealth/service.rs` (wire PnL `unpriced_assets` + optional `holdings_all`); `backend/src/wealth/types.rs`; `frontend/src/pages/WealthPage.tsx` (table columns, unified banner); `frontend/src/lib/api.ts` types.

**Risks:** Showing unpriced linear with “—” EUR may still feel broken without copy; expanding enum/pricing scope delays sprint.

#### AR — Grafana cashflow zeros verify gate

**Code unchanged:** `cashflow.json` panels query `forecast_balance_daily` / `forecast_cashflow_monthly` with `$account_id` and latest `forecast_computations.status='success'`. Variable query sorts accounts by `ABS(balance)` DESC — acct **114** should be default when funded.

**BUG-0013 AI refuted** when acct **114** + Full sync + recompute (baseline non-zero). Operator screenshot (June **2028** window) likely **time-range / account mismatch**, not SQL regression: default range is `now-30d` → `now+6M`; scrolling to 2028 exceeds forecast horizon → empty or zero series.

**AR verify gate (operator — mandatory before AR1 code):**

| Step | Pass criterion |
|------|----------------|
| 1 | Three-service external `compose up` + Full Firefly sync + forecast recompute |
| 2 | Grafana cashflow: `$account_id` = **114** (name matches funded Giro); not acct **116** or empty |
| 3 | Reset time range to **Last 30 days → +6 months** (dashboard default) |
| 4 | `GET /api/v1/forecast/daily?account_id=114` — `balances` non-zero for current month |
| 5 | `GET /api/v1/forecast/meta` — `computation_id` set, status success |
| 6 | Panel SQL (panel id 1) with same `account_id` + computation_id returns rows |

**AR1 code trigger:** Step 4 passes (API non-zero) **and** step 6 returns zero — Grafana variable/computation_id mismatch only.

**Risks:** Premature SQL edit duplicates BUG-0013; unfunded acct 116 looks like “missing data”.

#### AS — plan delete UI; target_type scope

**Delete API:** `DELETE /api/v1/plans/:id` in `backend/src/api/plans.rs` — no active-plan guard; `plans` → `plan_versions` → adjustments **ON DELETE CASCADE** (`004_plans.sql`).

**UI gap:** `PlanningPage.tsx` has `deleteAdjustmentMutation` only; no `deletePlanMutation`, no confirmation dialog.

**AS1 recommended fix:** Mirror adjustment delete pattern — `useMutation` → `DELETE /api/v1/plans/${id}`; confirm modal with plan name; disable when sole plan or require picking another active first; on success invalidate `plans`, clear `selectedPlanId` if deleted.

**Active-plan delete:** Architecture must decide: (a) block delete of `is_active` plan with 409 + copy, or (b) allow cascade leaving `no_active_plan` (PVA already handles per DEC-0074). **Recommend (a)** — align with single-global-active contract (DEC-0024).

**AS2 target_type:** UI offers `household | subscription | account` but DB enum is `household | subscription | category | custom_label | allocation_target` — **`account` is invalid** (insert would fail). Templates use `category` / `allocation_target` internally. **Recommend:** help copy explaining household = all accounts, subscription = payee patterns; remove or replace `account` option with `category` (no enum expansion unless new DEC). Enum expansion deferred to architecture.

**Risks:** Deleting active plan breaks Grafana Dashboard 3 until another activated; no frontend delete precedent for confirm UX.

#### AO — Grafana ML banner when env opts in but sidecar down

**React (correct):** `ForecastPage.tsx` distinguishes `sidecar_disabled` (ML off) vs `sidecar_unavailable` / other skip reasons via `ml_skipped_reason` from `GET /api/v1/forecast/meta` (DEC-0066).

**Grafana (misleading):** `forecast-horizons.json` panel id **13** static markdown: *"ML forecast not enabled on this deployment… Enable via US-0013"* — shows even when `FORECAST_ML_ENABLED=true` and metadata says `sidecar_unavailable`.

**AO1 options:**

| Option | Mechanism | Fit |
|--------|-----------|-----|
| **A (recommended MVP)** | Replace static copy with **dual-scenario** markdown: (1) ML not configured — set env; (2) ML configured but sidecar unreachable — start `stats-forecast` per DEC-0076 | No new plugins; provisioning-only |
| **B** | Hidden Postgres variable: `SELECT metadata->>'ml_skipped_reason' FROM forecast_computations WHERE model_kind='baseline' ORDER BY computed_at DESC LIMIT 1`; text panel `${ml_skip_reason}` | Dynamic; matches API truth |
| **C** | Business Text plugin | Rejected — not in current Compose image |

**Risks:** Option A still not runtime-accurate; Option B shows stale reason until recompute.

#### Ops bundle (AO/AT — unchanged)

Per R-0071 / DEC-0076: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d flow-finance-ai grafana stats-forecast`; then Full sync + recompute before attributing AP/AQ/AR to code.

**Architecture decisions (accepted at architecture, shipped Q0022):** **DEC-0081** (AQ holdings+FX), **DEC-0082** (AS1 active delete 409), **DEC-0083** (AS2 target_type UI). AP2/AR1 remain conditional on operator gates.

**Linked (extended):** DEC-0064, DEC-0066, DEC-0074, DEC-0076, DEC-0080, DEC-0081, DEC-0082, DEC-0083, R-0076 §6  
**Confidence:** high (AP/AQ/AS/AO code paths); medium (AR — verify gate sufficient)  
**Status:** fulfilled — BUG-0014 released Q0022 (2026-06-07); operator smoke AO–AT pass-with-prerequisites

---

## R-0080 — Category analytics, goal planning & subscription tags intake

**Date:** 2026-06-07  
**Topic:** Operator feature intake — category filters/trends, goal plans with AI savings, subscription manual discovery + tags  
**Query:** Decompose into multiple user stories; research patterns and dependencies  
**Sources:**
- Operator request (intake `intake-20260607-category-planning-subscriptions`)
- Code: `backend/src/transactions/repository.rs` (`aggregates_by_category`), `backend/src/plan/`, `frontend/src/pages/PlanningPage.tsx`, `frontend/src/pages/SubscriptionsPage.tsx`
- Prior: [R-0015](research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline), [R-0009](research.md#r-0009--subscription-detection-engine-patterns--confidence-scoring), [R-0074](research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy), US-0014, US-0015, BUG-0006 category ingest

**Findings:**

### Decomposition (accepted — 3 stories)

| Story | Operator value | Primary surfaces |
|-------|----------------|------------------|
| **US-0018** | Category filters + monthly expense trends/charts | Forecast, wealth, Grafana, shared filter API |
| **US-0019** | Goal plans (10k/5mo), per-plan stats, AI savings picks | `/planning`, plan engine, AI tools |
| **US-0020** | Manual sub search, majority category, custom tags | `/subscriptions`, product DB tags |

**Sequence:** US-0018 first (category contract) → US-0019 (richest what-ifs) ∥ US-0020 (independent).

### Category analytics (US-0018)

- Mirror already supports `aggregates_by_category` by month — extend API + UI rather than new ingest
- Grafana: add `$category` variable pattern per R-0008 dashboard-as-code; reuse Firefly category list query
- Finanzguru-like "where do I save" = month-over-month delta per category — chart in React (ECharts) + optional Grafana panel
- Forecast/planning filter: pass `category_id` into existing projection queries where DEC-0007 buckets map to categories (architecture must define join)

### Goal planning (US-0019)

- US-0004 plan engine today: template adjustments by household/subscription/account — **extend** with `category` target type + goal metadata (`target_balance_eur`, `target_date`)
- Per-plan stats: scope `plan_computations` / compare API to `plan_id` — avoid aggregating all plans (operator pain point)
- AI savings: new tool or extend `get_transactions` + category catalog — propose ranked reducible categories with aggregate amounts; operator opt-in lines (DEC-0032)
- Risk: goal feasibility math must respect forecast baseline account selection (acct 114 vs 116 pattern from BUG-0013)

### Subscriptions (US-0020)

- US-0003 auto-detection remains; add **explorer** query over mirror: `GROUP BY payee, account, approximate_interval`
- Majority category: `MODE() WITHIN GROUP (ORDER BY category_id)` on linked txs — exclude single-outlier miscategorization per operator note
- Tags: new table `subscription_tags` + join — **not** Firefly tags; filter in list API
- Search dimensions: account_id, `ILIKE` on description/payee, `HAVING COUNT(*) >= 3` for recurrence heuristic

### Alternatives rejected

| Option | Why |
|--------|-----|
| Single US-0018 mega-epic | Operator asked for multiple US; independent release value |
| Firefly tag sync | Violates read-only contract |
| Auto-apply AI savings | Operator must select suggestions |

**Linked:** US-0018, US-0019, US-0020, US-0004, US-0003, US-0015, R-0015, R-0009, R-0074  
**Confidence:** medium-high (intake synthesis; discovery validates SQL + UX)  
**Status:** fulfilled — all portions shipped (US-0018 S0017, US-0019 S0018, US-0020 S0019 via [R-0085](research.md#r-0085--us-0020-subscription-discover-majority-category--operator-tags)/DEC-0098..0103); retain for traceability

---

## R-0081 — BUG-0015 confirmed subscription reconfirm after rebuild

**Date:** 2026-06-07  
**Topic:** Operator defect — confirmed subscriptions reappear as pending with Confirm/Reject after container rebuild on omniflow external profile  
**Query:** Intake synthesis for fingerprint stability, postgres persistence across rebuild, and detection/alert pipeline hypotheses  
**Sources:**
- Operator report (intake `intake-20260607-subscription-reconfirm`) — Cursor 95% €17.18/mo, Apple 60% €9.99/mo re-prompted post-rebuild
- Code: `backend/src/subscriptions/repository.rs` (`upsert_pending_pattern` L144–147 preserves `confirmed`/`rejected` on same fingerprint); `detection.rs` (`confirmed_fps` skip L42–43); `compute_fingerprint(payee_key, interval_days, median_amount)`
- Prior: [R-0009](research.md#r-0009--subscription-detection-engine-patterns--confidence-scoring), [R-0012](research.md#r-0012--subscription-persistence-schema-candidates-confirmed-rejections-events), [R-0068](research.md#r-0068--bug-0008-subscription-alert-dedup-unread-count-contract-orphan-lifecycle), **BUG-0008**, **BUG-0014** rebuild context

**Findings:**

### Symptom contract (BUG-0015)

Operator trust requires **confirm-once** semantics: a subscription confirmed via US-0003 must survive app container rebuild when postgres persists on external volume. Re-prompting Cursor/Apple after rebuild is a **data-integrity / UX regression** distinct from BUG-0008 alert-count inflation (though AW hypothesis may overlap).

### Hypothesis ranking (discovery order)

| Priority | Hypothesis | Mechanism | Discovery probe |
|----------|------------|-----------|-----------------|
| 1 | **Fingerprint drift (AU)** | Payee normalization (`description` vs counterparty), median amount drift, or interval change → new fingerprint → fresh `pending` INSERT; status preservation only applies on fingerprint match | `SELECT fingerprint, status, payee_key, current_amount, interval_days FROM subscription_patterns WHERE display_name ILIKE '%cursor%' OR payee_key ILIKE '%apple%'` pre/post rebuild |
| 2 | **DB ephemeral (AV)** | Compose rebuild recreated empty postgres or wrong `DATABASE_URL` target | Row count + `confirmed` patterns zero after rebuild without operator action |
| 3 | **Alert/UI desync (AW)** | Pattern `confirmed` but unread `subscription_alerts` or UI reads alert-driven pending state | Compare pattern status vs `subscription_alerts` for same `pattern_id` |
| 4 | **Detection re-run (AX)** | Full sync post-rebuild re-groups transactions; `confirmed_fps` loaded but new groups bypass skip | Trace detection log; verify `load_confirmed_fingerprints` set at run start |

### Code anchors

- **Same-fingerprint preserve:** `upsert_pending_pattern` `ON CONFLICT` keeps `confirmed`/`rejected` status — does **not** merge across fingerprint changes.
- **Skip on detect:** `detection.rs` skips groups when `confirmed_fps.contains(&fingerprint)` — ineffective if fingerprint changed.
- **Rebuild ops:** US-0010 external profile expects postgres on persistent volume; app-only rebuild (**BUG-0014** pattern) should not wipe `subscription_patterns`.

### Recommended discovery sequence

1. Operator gate: confirm Cursor + Apple → note fingerprints → rebuild app containers only → check DB before Full sync.
2. Full sync → re-check fingerprints and statuses.
3. If drift confirmed: evaluate merchant-level dedup or stable fingerprint (payee identity without amount in hash) — architecture decision, not intake closure.
4. If DB empty: ops/runbook — volume mount and `docker compose` service set.

### Alternatives rejected at intake

| Option | Why |
|--------|-----|
| Reopen BUG-0008 | DONE; alert-count symptom ≠ confirm persistence after rebuild |
| Merge BUG-0014 | DONE cluster; subscriptions independent |
| Immediate fingerprint redesign | Needs discovery evidence first |

**Linked:** BUG-0015, US-0003, BUG-0008, BUG-0014, R-0009, R-0012, R-0068, R-0082  
**Confidence:** high (discovery verdicts + code audit + prior-art matrix)  
**Status:** fulfilled — shipped Q0023 via DEC-0084/0085/0086; retain for traceability

### Discovery verdict addendum (2026-06-07)

| ID | Verdict | Implication for fix |
|----|---------|---------------------|
| **H1** fingerprint drift | **LIKELY PRIMARY** | Fix must survive `payee_key` / `median_amount` / `interval_days` recomputation across sync |
| **H2** DB ephemeral | **UNLIKELY sole** | Ops SQL gate still required pre-execute; not a code substitute |
| **H3** alert/UI desync | **REFUTED primary** | AW noise is **secondary** — new `pattern_id` from H1 bypasses DEC-0071 dedup |
| **H4** detection re-run | **Subsumed by H1** | Skip logic works when fingerprint stable; no separate pipeline change |

**Operator gate (pre-execute):** `SELECT status, COUNT(*) FROM subscription_patterns GROUP BY status` immediately after app rebuild **before** Full sync; probe Cursor/Apple rows for duplicate fingerprints on different `payee_key` values.

### Root-cause mechanisms (code-confirmed)

1. **Fingerprint is brittle:** `compute_fingerprint(payee_key, interval_days, median_amount)` — any of three inputs changing yields a new 16-hex hash (`detect.rs` L45–49). `subscription_patterns.fingerprint` is **UNIQUE**; new hash → new `pending` INSERT. ON CONFLICT status preservation (`repository.rs` L144–147) never fires.
2. **Payee key still varies for card merchants:** DEC-0072 normalization strips SEPA noise and legal suffixes but **does not** collapse comma-separated card descriptors (`CURSOR, AI POWERED IDE, CURSOR.COM` vs `CURSOR.COM`) or shared billing roots (`APPLE.COM/BILL` vs `ITUNES.COM`). `display_name` comes from **last** transaction description (`detect.rs` L104–107) while `payee_key` comes from normalized source string — descriptor variance across months changes grouping key.
3. **Median amount drift:** Recent-6-tx median (`detect.rs` L95–97) can shift ±€0.01 after new billings or FX rounding → new fingerprint even when payee stable.
4. **Confirmed skip is exact-match only:** `load_confirmed_fingerprints` + `confirmed_fps.contains(&fingerprint)` (`detection.rs` L42–44) — no payee-level inheritance. **BUG-0008** DEC-0072 recall changes increased grouping consistency but did not add confirm propagation.
5. **Alert dedup is per-pattern_id (DEC-0071):** `sub_alert:new_detection:{pattern_id}` — drift creates new row → new unread alert. Confirm already marks alerts read for **that** `pattern_id` (`confirm_pattern` → `mark_read_unread_alerts_for_pattern`); does not block re-detection under new id.

### Fix option matrix

| Option | Mechanism | Surfaces | Solves AU/AV | Solves AW | Effort | Risks |
|--------|-----------|----------|--------------|-----------|--------|-------|
| **A. Card-merchant `payee_key` normalization** | Extend `payee_key()`: left-prefix before `,` / `*`; collapse `*.com/bill` / `itunes.com` / domain tails to merchant root (see [R-0082](research.md#r-0082--card-billing-descriptor-normalization-for-subscription-identity)) | `recurrence/normalize.rs`, tests | **Partial** — reduces drift at source | Indirect | Low | Over-merge distinct products same biller (multiple Apple subs); forecast/recurrence shared (DEC-0013) |
| **B. Stable fingerprint (drop amount)** | `compute_fingerprint(payee_key, interval_days)` only | `recurrence/detect.rs`, migration backfill optional | **Partial** — price/FX drift only | Indirect | Low–med | Same payee + interval, different amounts (tier changes) collapse; breaks price-change tracking identity |
| **C. Payee-level confirm inheritance** | Load `confirmed`/`rejected` by `payee_key` (+ interval tolerance); **skip** emit when matched; on upsert **merge into** existing confirmed row (update amounts/dates/fingerprint) instead of new pending | `repository.rs`, `detection.rs`, `service.rs` | **Yes** — confirm-once semantics | **Yes** — no new pending → no new alert | Med | Multiple subs same merchant need `payee_key + interval_days` composite; stale confirmed row if merchant truly new |
| **D. Detection skip by `payee_key` only** | Subset of C: skip without merge — confirmed payee never re-emits pending | `detection.rs`, `repository.rs` | **Yes** for re-prompt | **Yes** | Low–med | Confirmed row drifts from live amount/interval until manual refresh; orphan old fingerprint rows |
| **E. Alert dedup on confirm / by payee** | Extend DEC-0071 fingerprint to `sub_alert:new_detection:payee:{payee_key}` or mark-read by payee on confirm | `repository.rs`, migration | **No** — pending cards still show | **Partial** — banner only | Low | Violates DEC-0071 contract; masks AU failure |
| **F. Merchant identity table** | `merchant_id` canonical map; operator or rules-assigned | New migration, admin API | **Yes** | **Yes** | High | Over-engineered for two-operator merchants; deferred |

**BUG-0008 prior art (coordinate — do not reopen):**

| Prior | Relevance to BUG-0015 |
|-------|----------------------|
| **DEC-0071** (R-0068) | Alert dedup + confirm mark-read — **necessary but insufficient** when H1 creates new `pattern_id` |
| **DEC-0072** (R-0069) | Payee normalization + transfer guard — **reduced** SEPA drift; card descriptor variance remains |
| **R-0065 coordinate table** | Detection internals only; `list_patterns` additive JSON OK; no alert-count semantic regression |

### Recommendation (architecture gate)

**Adopt two-layer bundle (primary):**

1. **Layer 1 — A (normalization):** Card-billing descriptor rules in `payee_key()` per [R-0082](research.md#r-0082--card-billing-descriptor-normalization-for-subscription-identity). Low-risk complement to DEC-0072; reduces false splits for Cursor/Apple class descriptors.
2. **Layer 2 — C (payee + interval inheritance):** Detection skip **and** upsert merge keyed on `(payee_key, interval_days)` with ±3-day interval tolerance for monthly cadence. On match to **confirmed**: refresh `current_amount`, `last_seen_at`, `confidence_pct`, optionally rotate `fingerprint` in-place (single row). On match to **rejected**: skip (extend rejection load beyond fingerprint-only). Reject **B** as sole fix (price collisions). Reject **E** as primary (AW without AU). Reject **F** for MVP.

**Alternative (if Layer 1 tests show sufficient stability):** **D only** — skip without merge — acceptable for `/quick` if operator accepts stale amount on confirmed card until price-change pipeline updates.

**Explicit non-goals:** Reopen BUG-0008; postgres volume runbook (H2 ops gate only); UI changes (H3 refuted).

### Architecture decision gates

| Gate | Question |
|------|------------|
| DEC-???? | Payee+interval confirm inheritance vs normalization-only |
| DEC-???? | Interval tolerance (±3 days monthly) and multi-subscription-per-merchant policy |
| DEC-???? | In-place `fingerprint` rotation on confirmed merge vs keep legacy fingerprint |
| DEC-???? | Index `(payee_key, status)` for detection lookup |

### Execute task surfaces (for sprint-plan)

| Task | Surface | Depends |
|------|---------|---------|
| **AU1** | `recurrence/normalize.rs` — card descriptor rules + unit tests | — |
| **AU2** | `subscriptions/repository.rs` — `load_confirmed_payee_intervals`, merge upsert | AU1 |
| **AU3** | `subscriptions/detection.rs` — skip + merge call path; rejection by payee | AU2 |
| **AU4** | `subscriptions/detection.rs` — `mark_stale_inactive` active map by payee+interval | AU2 |
| **V1** | verify-work: confirm Cursor/Apple → rebuild app → Full sync → AU/AV/AW | deploy |

**Ops gate:** operator SQL probe documented in UAT before V1.

---

## R-0082 — Card billing descriptor normalization for subscription identity

**Date:** 2026-06-07  
**Topic:** Web + industry patterns for stabilizing card/PSP billing descriptors in recurrence grouping (BUG-0015 Layer 1)  
**Query:** How do payment processors and enrichment APIs treat variable billing descriptors for recurring merchant identity?  
**Sources:**
- [Ntropy Recurrence API](https://docs.ntropy.com/enrichment/recurrence) — ML grouping by counterparty + periodicity; tolerates amount variance and gaps
- [Visa compelling evidence FAQs](https://usa.visa.com/content/dam/VCOM/regional/na/us/support-legal/documents/evolution-of-compelling-evidence-merchant-faqs-mar2023.pdf) — descriptor consistency: keep **leftmost** characters stable; dynamic suffixes on the **right**
- [Recurly payment descriptors](https://docs.recurly.com/recurly-subscriptions/docs/payment-descriptors) — `DBA*Plan Name` pattern; prefix stable, suffix variable
- Code gap: `payee_key()` handles SEPA (DEC-0072) but not comma-separated card memos or `*.com/bill` roots

**Findings:**

| Pattern | Industry practice | Proposed rule for `payee_key()` |
|---------|-------------------|--------------------------------|
| Dynamic suffix | Right-side variable (`AcmeInc*Gold Plan`) | Take token before `*` if present |
| Multi-field memo | `MERCHANT, PRODUCT, DOMAIN` comma lists | Take **leftmost** segment before `,`; normalize domain |
| Billing aggregator | `APPLE.COM/BILL`, `ITUNES.COM`, `APPLE.COM/BILL ITUNES` | Map known roots to `apple` (configurable alias list or heuristic) |
| Domain tail | `CURSOR.COM`, `cursor.com` | Strip `.com` / `/bill` tails for known SaaS merchants |

**Risks:** Alias list maintenance; EU descriptor charset; over-merge per R-0069 §5. Mitigation: inheritance Layer 2 (R-0081 §C) catches residual drift.

**Linked:** BUG-0015, R-0081, DEC-0072, DEC-0013  
**Confidence:** medium (heuristic; validate against operator Cursor/Apple rows)  
**Status:** fulfilled — shipped DEC-0084 (AU1); retain for traceability

---

## R-0083 — US-0018 category filters, expense-series API & trend analytics

**Date:** 2026-06-08  
**Topic:** Technical research for shared category filter contract, monthly per-category expense series, React trend chart, and Grafana `$category` wiring  
**Query:** Resolve discovery open questions for US-0018 AC-1..AC-6: monthly SQL shape, catalog API, forecast filter depth, planning compare semantics, Grafana panels, chart default, performance on 24-month window  
**Sources:**
- [PostgreSQL `generate_series` gap-fill](https://stackoverflow.com/questions/65307015/how-to-fill-the-time-gap-after-grouping-date-record-for-months-in-postgres) — zero-filled month spine
- [PostgreSQL time-series gap-fill pattern](https://viprasol.com/blog/postgres-time-series-data/) — `date_trunc` + left join
- [Grafana PostgreSQL template variables](https://grafana.com/docs/grafana/latest/datasources/postgres/template-variables/) — `__value`/`__text`, `__searchFilter`
- [Grafana variables — Include All / custom all value](https://grafana.com/docs/grafana/latest/dashboards/variables/add-template-variables/) — empty-string “All” vs regex
- [Highcharts — line vs bar for time data](https://www.highcharts.com/blog/best-practices/line-chart-vs-bar-chart-choosing-the-right-one-for-your-objectives-and-data/) — trend vs discrete comparison
- Code: `backend/src/transactions/repository.rs` (`aggregates_by_category`, `aggregates_by_month`, `search_categories_by_name`), `backend/src/transactions/service.rs` (`label_uncategorized_categories`, `MIN_CATEGORY_SEARCH_LEN=2`), `backend/src/api/mod.rs` (no category routes), `backend/src/forecast/categories.rs` (`resolve_bucket` via `category_id`→name→DEC-0007 map), `grafana/provisioning/dashboards/analytics/{cashflow,budgets}.json` (`$account_id` only), `frontend/src/components/forecast/MonthlyChart.tsx` (ECharts bar), `frontend/src/pages/ForecastPage.tsx` (account `<select>` pattern)
- Prior: [R-0080](research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake), [R-0008](research.md#r-0008--grafana-dashboard-as-code-for-analytics-dashboards), [R-0060](research.md#r-0060--transaction-aggregate-contract-bug-0006), US-0011 embed shell

**Findings:**

### 1. Monthly per-category expense series (AC-2, AC-5)

**Correction vs R-0080:** mirror has **period** `aggregates_by_category`, not monthly-by-category. `aggregates_by_month` is household-only (no `category_id` dimension). New repository method required.

**Recommended SQL (single category or uncategorized sentinel):**

```sql
-- Params: $start, $end (inclusive dates), $category_id (TEXT or NULL for uncategorized-only)
WITH month_spine AS (
  SELECT generate_series(
    date_trunc('month', $start::date),
    date_trunc('month', $end::date),
    '1 month'::interval
  ) AS month_start
)
SELECT
  to_char(m.month_start, 'YYYY-MM') AS month,
  COALESCE(SUM(CASE WHEN t.amount::float8 < 0 THEN ABS(t.amount::float8) ELSE 0 END), 0) AS outflow_eur,
  COALESCE(SUM(CASE WHEN t.amount::float8 > 0 THEN t.amount::float8 ELSE 0 END), 0) AS inflow_eur,
  COUNT(t.*)::bigint AS transaction_count
FROM month_spine m
LEFT JOIN transactions t
  ON date_trunc('month', t.date) = m.month_start
 AND t.date >= $start AND t.date <= $end
 AND (
   ($category_id IS NOT NULL AND t.category_id = $category_id)
   OR ($category_id IS NULL AND t.category_id IS NULL)
 )
GROUP BY m.month_start
ORDER BY m.month_start;
```

- **Window:** default 12 months, max 24 — compute `$end = CURRENT_DATE`, `$start = date_trunc('month', $end) - (months-1) * interval '1 month'` in service layer (not data min/max) so AC-3 labels always show full spine with explicit €0 months.
- **Uncategorized (AC-5):** dedicated request `category_id=__uncategorized__` (or omit + `bucket=uncategorized`) maps to `t.category_id IS NULL`; response includes `category_label: "Uncategorized"` and `uncategorized: true`. Reuse `label_uncategorized_categories` naming — never return silent zeros without bucket metadata.
- **EUR reporting:** mirror amounts are already EUR-normalized at ingest (BUG-0006); mixed native currency note belongs in API meta/tooltip only when `payload` currency ≠ EUR (defer footnote to architecture if rare).
- **Alternatives rejected:**
  - *Materialized view / Timescale continuous aggregate* — overkill for ≤24-point single-category series on ~1k rows; adds refresh orchestration.
  - *Gap-fill only on months with data* — fails AC-3 month-label contract when category idle in a month.

**API shape (research draft):**

| Endpoint | Contract |
|----------|----------|
| `GET /api/v1/categories` | Full mirror catalog `{id, name}` sorted by name; optional `?q=` with `MIN_CATEGORY_SEARCH_LEN=2` reuse; cap 200 rows; `truncated` flag if over cap |
| `GET /api/v1/categories/expense-series` | Query: `category_id` (required), `months` (default 12, max 24), optional `end` (default today); returns `{category_id, category_name, months[], summary{mom_delta_pct, best_month, worst_month}, meta{period_start, period_end, uncategorized}}` |

`summary` fields satisfy AC-4 (MoM on last two **non-empty** months or last two spine months — architecture must pick; recommend **last two calendar months in window** with zero allowed).

**Risks:** `date_trunc('month', date)` prevents index-only `idx_transactions_date` use when filtering `category_id`; acceptable at MVP scale — see §7.

### 2. Category catalog (AC-1)

- **Full list MVP** — typical Firefly households have &lt;100 categories; paginate only if `COUNT(*) > 200`.
- Reuse `search_categories_by_name` for `?q=` but raise `CATEGORY_SEARCH_LIMIT` for public catalog (AI tool keeps 10-cap path).
- **React `CategoryFilter`:** clone `ForecastPage` account `<select>` for ≤20 categories; switch to searchable combobox (native `<datalist>` or lightweight filter input) above 20 — no new dependency required.
- Sentinel options: `All categories` (value `""`) for toolbar surfaces; trend chart requires explicit pick (disable chart until selected).

### 3. Grafana `$category` (AC-1)

Extend existing `$account_id` pattern per [R-0008](research.md#r-0008--grafana-dashboard-as-code-for-analytics-dashboards):

```json
{
  "name": "category",
  "type": "query",
  "datasource": { "type": "postgres", "uid": "FlowFinancePostgreSQL" },
  "query": "SELECT '' AS __value, 'All categories' AS __text UNION ALL SELECT c.firefly_id AS __value, COALESCE(c.name, c.firefly_id) AS __text FROM categories c ORDER BY 2",
  "refresh": 1,
  "sort": 1
}
```

**Panel filter SQL:** `AND ('${category}' = '' OR t.category_id = '${category}')` on `transactions t` joins.

| Dashboard | Panel action |
|-----------|--------------|
| **cashflow** | New time-series/bar panel: monthly category outflow from mirror (`date_trunc('month', t.date)`, sum abs negative amounts), respects `$category` + existing `$account_id` optional later |
| **budgets** | Extend **Ist** / deviation CTEs: when `$category` set, filter `actual` leg `AND t.category_id = '${category}'`; planned leg unchanged (plan engine is household-level) — document “All categories” = current behavior |

**Empty variable state:** default `''` (All) — panels match pre-US-0018 household view; no broken queries.

**MVP: no SPA↔iframe sync** — independent filters per discovery; avoids auth/query-param coupling on US-0011 embed.

**Risks:** Grafana single-select only in MVP; SQL injection mitigated by variable type query (values from DB ids). Category deleted in Firefly but stale in mirror until sync — panel shows empty series (acceptable).

### 4. React trend chart & performance insight (AC-3, AC-4)

- **Default: bar chart** — aligns with existing `MonthlyChart` (stacked bars), Finanzguru-like discrete “Jan €300” labels, and PO discovery “bar default”. Optional line toggle is **stretch** (not required for AC-3).
- Web guidance ([Highcharts best practices](https://www.highcharts.com/blog/best-practices/line-chart-vs-bar-chart-choosing-the-right-one-for-your-objectives-and-data/)): line emphasizes slope; bars emphasize month-to-month magnitude — AC-3/AC-4 wording favors **bar + stat callouts**.
- **Primary home:** `/forecast` monthly tab above/below `MonthlyChart` per discovery; component exported for `/wealth` subsection embed.
- **Empty state:** no rows in period for category → “No categorized spending in this period” + link to Firefly (read-only); uncategorized bucket with zero txs still shows spine with zeros if explicitly selected.
- **MoM / best / worst:** compute server-side in `expense-series` `summary` to keep clients thin; chart annotation optional.

### 5. Forecast filter depth (AC-1, decision gate)

**Recommendation (display-only MVP):** `category_id` on forecast monthly **does not** re-run `forecast_cashflow_monthly` or fork DEC-0007 projection.

| Surface | With category selected |
|---------|------------------------|
| Stat cards (Income/Fixed/Variable/Free) | **Unchanged** — household forecast |
| `MonthlyChart` | **Unchanged** — household buckets |
| New `CategoryTrendChart` | **Filtered actuals** from `expense-series` only |
| Optional decomposition table row filter | Architecture choice — P2 |

Full category-scoped forecast re-projection requires mapping `category_id` → bucket via `resolve_bucket` + re-aggregating recurring patterns — **US-0019 / follow-on**, not US-0018 blocker.

**Alternative rejected:** Block US-0018 on forecast engine category fork — unnecessary for AC-1..AC-5.

### 6. Planning compare & wealth (AC-1)

| Surface | MVP behavior |
|---------|--------------|
| **Planning compare toolbar** | `CategoryFilter` stores selection in page state; **compare API unchanged** — filter applies to adjacent **category trend** widget or PVA actuals preview only. Plan versions with `target_type=category` adjustments remain visible in compare table. |
| **Wealth overview** | New “Category spending” subsection: period totals via `expense-series` or one-shot `aggregates_by_category` for selected month range + link “View trend” → `/forecast` with query `?category_id=` (optional deep link; not required AC). |

**Alternative rejected:** Recompute `build_compare_metrics` per category in US-0018 — scope creep into plan engine.

### 7. Performance & indexes (24-month window)

- Existing indexes: `idx_transactions_date` only; **no** `category_id` index (`001_initial.sql`).
- Estimate: 24-month filter on ~900 rows with `category_id = $1` → sequential scan on date range is **&lt;10 ms** on typical mirror; no migration required for MVP.
- **Gate:** if `EXPLAIN` &gt;50 ms on operator mirror, add `CREATE INDEX idx_transactions_category_date ON transactions (category_id, date)` in execute — optional task.
- Timescale `time_bucket` unnecessary — calendar month spine via `date_trunc` matches existing `aggregates_by_month`.

### 8. Regression & privacy (AC-6)

- New endpoints are **aggregate-only** (same privacy posture as AI `aggregates_by_category`).
- US-0015 `bucket_sources` / AI mapping **unchanged** — category filter does not alter projection path in MVP.
- OIDC external profile smoke: category routes behind same JWT/Traefik stack as forecast — no new public surface.

### Decision gates (carry to `/architecture`)

| Gate | Research recommendation | Alternative |
|------|----------------------|-------------|
| Multi-category overlay | **Defer** — single series (AC-3) | ≤3 series same chart |
| Trend chart type | **Bar default**; line toggle stretch | Line default |
| Grafana ↔ SPA sync | **Independent** | iframe `category_id` query param |
| Forecast filter depth | **Actuals-only side panel**; household forecast unchanged | Full category forecast fork |
| Uncategorized sentinel | **`__uncategorized__` query token** | Separate `/uncategorized` route |
| Planning compare filter | **UI-scoped actuals widget**; compare API unchanged | Server-side compare recompute |
| Category index migration | **Defer** unless explain fails | Ship index in US-0018 |

**Linked:** US-0018, R-0080, R-0008, R-0060, DEC-0007, DEC-0032, US-0011, US-0015, BUG-0006  
**Confidence:** high (code audit + SQL/Grafana patterns); medium on planning-compare UX nuance  
**Status:** fulfilled — released S0017 via **DEC-0087**..**DEC-0090** (`0.18.0-us0018`, 2026-06-09); retain for traceability

---

## R-0084 — US-0019 goal plans, per-plan stats, category overlay & AI savings

**Date:** 2026-06-09  
**Topic:** Technical research for US-0019 AC-1..AC-6 — `goal_balance` template, target-date projection, category-scoped overlay, deterministic savings ranking, account-scoped balance  
**Query:** Resolve discovery open questions: goal schema migration; balance at arbitrary `target_date` from `plan_computations`; yearly rollup grain; `build_overlay_deltas` category scoping; AI category savings under `allow_raw_transactions=false`; default asset account for goal progress  
**Sources:**
- [PostgreSQL recursive CTE budget projection](https://gist.github.com/codingthat/7b1e29ddfb878696468eca177dff01b4) — month-forward balance carry
- [Financial plan on PostgreSQL — cumulative window](https://widefix.com/blog/financial-plan-on-postgresql/) — `sum(profit) OVER (ORDER BY month)` pattern
- [Savings goal PMT formula](https://calc.mintloop.dev/finance/savings-goal-calculator) — required monthly contribution back-solve
- [Wealthfolio Save-Up Planner](https://wealthfolio.app/docs/guide/goals/) — target date + required monthly contribution bisection
- [Yodlee Spending by Category insight](https://developer.yodlee.com/resources/yodlee/insights-details/docs/spending-by-category-all-accounts) — top-N categories by debit total (threshold param)
- [Pareto / 80-20 discretionary focus](https://www.wichitawealth.com/post/the-pareto-principle-cash-flow-unlocking-smarter-financial-strategies)
- Code: `backend/migrations/004_plans.sql`, `backend/src/plan/{overlay.rs,project.rs,service.rs,repository.rs,types.rs,templates.rs}`, `backend/src/forecast/service.rs` (`aggregate_daily_balances`), `backend/src/transactions/repository.rs` (`aggregates_by_category`, `expense_series_by_month`), `backend/src/api/plans.rs` (`savings_suggestions` payee-only), `frontend/src/pages/PlanningPage.tsx`
- Prior: [R-0080](research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake), [R-0015](research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline), [R-0018](research.md#r-0018--plan-persistence-schema-plans-versions-adjustments-daily-snapshots), [R-0083](research.md#r-0083--us-0018-category-filters-expense-series-api--trend-analytics), **DEC-0073**, **DEC-0087**..**DEC-0089**, **DEC-0032**

**Findings:**

### 1. Goal schema & template (AC-1)

**Current gap:** `plan_template` enum has no `goal_balance`; `PlanRow` has no goal columns; `templates.rs` has no goal preset.

**Recommendation — plan-level columns + enum value:**

| Column | Type | Notes |
|--------|------|-------|
| `target_balance_eur` | `NUMERIC(18,2) NULL` | Required when `template = 'goal_balance'` |
| `target_date` | `DATE NULL` | Must be ≥ today on create; editable on latest unfrozen version context |
| `goal_account_id` | `TEXT NULL` | Firefly `accounts.firefly_id`; optional — see §6 |

Migration: `ALTER TYPE plan_template ADD VALUE 'goal_balance'` + `ALTER TABLE plans ADD COLUMN …` (nullable for backward compat).

**Alternatives:**

| Option | Verdict |
|--------|---------|
| JSON `metadata` blob on `plans` | **Reject** — weak typing; harder API validation |
| Goal columns on `plan_versions` | **Defer** — per-version goal drift rare; PO wants plan-level metadata editable until frozen |
| Separate `plan_goals` table | **Reject** — over-normalized for 3 fields |

**Risks:** existing plans NULL-safe; enum migration requires migration script ordering; frozen version should expose goal fields read-only in UI only (no DB immutability trigger needed for MVP).

### 2. Target-date balance projection (AC-2)

**Existing asset:** `plan_daily_cashflow` stores `planned_balance` per `(version_id, computation_id, ts)` after recompute (`run_projection` → `bulk_insert_daily`). Horizon = **730 days** (`overlay_horizon_end`).

**Recommended SQL — balance at `target_date`:**

```sql
SELECT pdc.planned_balance::float8
FROM plan_daily_cashflow pdc
JOIN plan_computations pc ON pc.id = pdc.computation_id
WHERE pdc.version_id = $1
  AND pc.status = 'success'
  AND pc.computed_at = (
      SELECT MAX(computed_at) FROM plan_computations
      WHERE version_id = $1 AND status = 'success'
  )
  AND pdc.ts::date <= $2::date          -- target_date
ORDER BY pdc.ts DESC
LIMIT 1;
```

**Beyond horizon:** if `target_date > today + 730d`, return `{ "projected_balance_at_target": null, "beyond_horizon": true }` — do not extrapolate silently.

**Yearly rollup — recommend calendar year:**

```sql
SELECT EXTRACT(YEAR FROM ts)::int AS yr,
       SUM(planned_net::float8) AS net_eur
FROM plan_daily_cashflow
WHERE version_id = $1 AND computation_id = $2
GROUP BY 1 ORDER BY 1;
```

Overlay-only yearly delta (vs baseline) = same query on **synthetic overlay-only series** OR `SUM(overlay_delta)` per year from in-memory recompute — **simpler MVP:** expose **full scenario** `planned_net` yearly sum + separate `overlay_monthly_delta` (DEC-0073) already computed; label clearly in UI.

**Alternatives:** rolling 12m from target horizon — **reject** for Compare strip (calendar year matches US-0018 month labels).

**New API sketch:** `GET /api/v1/plans/{plan_id}/goal-stats?version_id={optional}`

```json
{
  "plan_id": "...",
  "version_id": "...",
  "target_balance_eur": "10000.00",
  "target_date": "2026-11-01",
  "goal_account_id": "114",
  "monthly_delta_vs_baseline": "-120.00",
  "yearly_rollup": [{ "year": 2026, "planned_net_eur": "2400.00" }],
  "projected_balance_at_target": "9850.00",
  "gap_eur": "150.00",
  "on_track": false,
  "beyond_horizon": false,
  "reporting_currency": "EUR"
}
```

**Scope guard (AC-2):** stats keyed by `plan_id` + `version_id` — never aggregate across plans on detail view. Compare tab shows stats for **selected plan** only; PVA remains **active-plan household** per discovery.

**Risks:** stale `plan_computations` if recompute pending; month-end vs arbitrary date lookup — use last daily point ≤ target_date, not interpolation.

### 3. Category overlay in plan engine (AC-3)

**Current gap:** `AdjustmentTarget::Category` exists in DB enum + form (`PlanningPage.tsx` `CategoryFilter`), but `build_overlay_deltas` applies category adjustments **identically to household** — full `signed_amount` on frequency schedule with no `category_id` join (`overlay.rs` L22–41).

**Recommended overlay semantics:**

| Direction | Behavior |
|-----------|----------|
| `remove_outflow` + `category` | Cap each application at **trailing 3-month average monthly outflow** for `target_key` (category Firefly id) from mirror; amount = `min(adj.amount, avg_monthly_outflow)`; spread on existing monthly/weekly schedule |
| `add_outflow` + `category` | Treat as labeled household outflow (category is metadata for compare/PVA only) — **or** reject in API validation; architecture gate |
| `subscription` | unchanged (payee match) |

**Data source:** reuse `expense_series_by_month` (R-0083) for last 3 months → `avg(outflow_eur)`; empty mirror → overlay contributes **0** with UI warning.

**DEC-0007 interaction:** overlay modifies **plan delta layer only** — forecast baseline buckets unchanged (same as R-0015). Category removal does not rewrite Income/Fixed/Variable projection.

**Alternatives:**

| Option | Verdict |
|--------|---------|
| Display-only category lines (no recompute) | **Reject** — fails AC-3 |
| Daily mirror-weighted removal | **Defer** — higher SQL cost; monthly cap sufficient for MVP |
| Fork forecast per category | **Reject** — scope explosion |

**Risks:** fixed-cost categories (rent) appear as reducible — filter suggestions separately (§5); uncategorized spend invisible to category overlay; double-count if cap exceeds actual category spend in future months.

### 4. AI / deterministic savings suggestions (AC-4, AC-5)

**Current:** `GET /api/v1/plans/templates/savings-mode/suggestions` returns **confirmed subscription payees only** (`map_savings_suggestions`).

**Recommended — new aggregate endpoint + modal UX:**

`GET /api/v1/plans/{plan_id}/category-savings-suggestions?months=6&limit=10`

1. `aggregates_by_category(period_start, period_end, None)` — sort by `total_outflow` DESC
2. Filter: `total_outflow / months >= 20` (min €20/mo); exclude categories already in plan adjustments (`target_type=category`, `remove_outflow`)
3. Optional: exclude DEC-0007 **fixed** bucket categories from `resolve_bucket` map (rent, utilities) — architecture gate
4. Return top-N with evidence: `category_id`, `name`, `avg_monthly_outflow_eur`, `transaction_count`, `suggested_reduction_eur` (= 50% of avg or `adj.amount` default)

**Operator flow:** modal mirrors savings-mode checkboxes → `POST` batch `remove_outflow` adjustments — **no silent auto-apply** (AC-4).

**AI path (AC-5):** deterministic ranking is **primary**; optional chat tool `get_category_savings` wrapping same aggregate service. `get_transactions` with `group_by: category` already privacy-safe under `allow_raw_transactions=false` (DEC-0032) — reuse audit patterns from US-0006/US-0015.

**Alternatives:**

| Option | Verdict |
|--------|---------|
| LLM-only ranking | **Reject** for MVP — non-deterministic; privacy review burden |
| Chat-only (no REST) | **Reject** — Scenarios modal needs REST |

**Risks:** suggesting cuts to non-discretionary categories; multi-account mirror includes all asset txs — consistent with household actuals; operator selects before apply — audit log per adjustment create.

### 5. Account default for goal balance (AC-2)

**Current:** `aggregate_daily_balances` **sums all asset accounts** in reporting currency (`forecast/service.rs` L265–288) — household total, not per-account (acct **114** vs **116** ambiguity per BUG-0013).

**Recommendation for `goal_balance` plans:**

| `goal_account_id` | Baseline source |
|-------------------|-----------------|
| Set explicitly | `fetch_daily_series(computation_id, account_id)` + starting balance from that account |
| NULL on create | Default = **asset account with max positive balance** in `reporting_currency` (ORDER BY balance DESC LIMIT 1 among `type='asset'`) |
| Fallback | Household sum (current behavior) + UI banner "Goal uses all accounts — select one account for precision" |

Non-goal templates remain household aggregate (no regression).

**Alternatives:** always household — **reject** for goal UX; require account on create — **reject** (optional field per discovery).

**Risks:** multi-currency accounts excluded by currency filter; goal progress jumps when operator changes `goal_account_id` — require recompute + confirmation.

### 6. Feasibility / gap copy (discovery)

**MVP math (0% interest):**

- `gap_eur = target_balance - projected_balance_at_target`
- `required_monthly_savings = gap_eur / months_remaining` (ceil to cent)
- UI shows gap + required monthly **copy only** — do **not** auto-insert savings adjustment lines (PO recommendation)

**Alternative:** annuity PMT with savings rate — **defer** (Wealthfolio/Monarch pattern; no product savings-rate config today).

### 7. UI surface map (AC-1..AC-6)

| Surface | Research contract |
|---------|-------------------|
| Scenarios template grid | Add **Goal balance** card → `POST { template: "goal_balance", target_balance_eur, target_date, goal_account_id? }` |
| Scenarios summary | Goal stats strip when `template=goal_balance` |
| Compare tab | Per-plan goal stats above version table (not mixed across plans) |
| Add-line form | `CategoryFilter` + `target_type=category` (exists); overlay must affect recompute |
| AI savings action | New modal parallel to savings-mode; checkbox → POST adjustments |
| Regression | US-0014 templates/toasts/PVA guided state; DEC-0089 compare category filter stays actuals-only |

### Decision gates (carry to `/architecture`)

| Gate | Research recommendation | Alternative |
|------|-------------------------|-------------|
| Goal storage | `plans` columns + `goal_balance` enum | JSON blob; per-version columns |
| Stats API | `GET …/goal-stats` per plan+version | Extend `/compare` only |
| Yearly rollup | Calendar year `SUM(planned_net)` | Rolling 12m |
| Category `remove_outflow` | Cap via 3-mo mirror avg outflow | Daily weighted; display-only |
| Category `add_outflow` | Household-labeled (no cap) | API reject |
| Savings ranking | Deterministic top-N aggregates | LLM-only |
| Fixed-category exclusion | Exclude DEC-0007 fixed bucket from suggestions | Show all |
| Account scope | Optional `goal_account_id`; default max-balance asset | Always household |
| Feasibility | Gap + required monthly (0% interest) | PMT + auto-lines |
| PVA scope | Unchanged household active plan | Per-plan PVA |
| AI tool | Optional wrapper; REST primary | Chat-only |

### Risks (summary)

| Risk | Mitigation |
|------|------------|
| `target_date` beyond 730d horizon | `beyond_horizon` flag + UI copy |
| Category overlay over-removal | Cap at historical avg outflow |
| Goal account vs household compare mismatch | Document; goal-stats uses plan account scope |
| Fixed costs in savings list | Filter fixed bucket at ranking |
| Migration enum ordering | Dedicated migration file; test rollback path |
| DEC-0089 regression | Category filter on Compare stays actuals-only |

**Linked:** US-0019, US-0018, US-0014, US-0004, US-0006, R-0080, R-0015, R-0018, R-0083, DEC-0073, DEC-0087, DEC-0088, DEC-0089, DEC-0032, DEC-0007  
**Confidence:** high (code audit + SQL patterns); medium on category overlay cap tuning and fixed-category exclusion policy  
**Status:** fulfilled — released S0018 via **DEC-0091**..**DEC-0097** (`0.19.0-us0019`, 2026-06-09); retain for traceability

---

## R-0085 — US-0020 subscription discover, majority category & operator tags

**Date:** 2026-06-10  
**Topic:** Technical research for US-0020 AC-1..AC-6 — Discover tab explorer, manual confirm, majority display category, operator tag CRUD/assign/filter, optional Grafana `$tag`  
**Query:** Resolve discovery open questions: explorer SQL vs reuse `detect_recurrence_groups`; manual confirm API vs DEC-0085 merge; majority category computation and tie-break; tag schema; list filter composition; Grafana variable pattern; regression boundaries with US-0003/DEC-0084..0086  
**Sources:**
- [PostgreSQL `mode()` aggregate](https://www.postgresql.org/docs/17/functions-aggregate.html) — arbitrary tie-break; requires explicit `GROUP BY` + `RANK` for deterministic policy
- [Stack Overflow — mode tiebreaker](https://stackoverflow.com/questions/66715271/tiebreaker-criterion-of-the-mode-in-postgres) — do not rely on `ORDER BY` inside `mode()` alone
- [SQL Habit — recurring payments with LAG](https://www.sqlhabit.com/blog/how-to-detect-recurring-payments-with-sql) — `LAG` + `HAVING COUNT(*) > 1` explorer anti-pattern reference (diverges from shared recurrence core)
- [PostgreSQL junction / tag schema patterns](https://www.grizzlypeaksoftware.com/library/postgresql-schema-design-patterns-for-web-applications-iqohhdbg) — many-to-many with composite PK + reverse index
- Code: `backend/src/recurrence/detect.rs` (`detect_recurrence_groups`, `RecurrenceGroup.category_ids`), `backend/src/subscriptions/{detection.rs,repository.rs,service.rs}`, `backend/src/api/subscriptions.rs`, `backend/migrations/003_subscriptions.sql`, `frontend/src/pages/SubscriptionsPage.tsx`, `grafana/provisioning/dashboards/analytics/{subscriptions.json,cashflow.json}`
- Prior: [R-0080 § Subscriptions](research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake), [R-0009](research.md#r-0009--subscription-detection-engine-patterns--confidence-scoring), [R-0012](research.md#r-0012--subscription-persistence-schema-candidates-confirmed-rejections-events), **DEC-0084**, **DEC-0085**, **DEC-0086**, **DEC-0087** (category catalog), US-0003, US-0018

**Findings:**

### 1. Explorer query (AC-1)

**Current gap:** No discover API or UI; detection runs only on sync via `DetectionPipeline::run_candidates` (`detection.rs`).

**Recommendation — reuse Rust recurrence core, not ad-hoc SQL:**

| Step | Contract |
|------|----------|
| Load | `load_expense_transactions(window_days)` with **SQL push-down** for `account_id` + `date >= cutoff` (same `SubscriptionsConfig.detection_window_days`, default 365) |
| Group | `detect_recurrence_groups(&txs, &RecurrenceConfig::default())` — preserves **DEC-0084** `payee_key()` normalization, cadence stability, confidence tiers (R-0009) |
| Filter | Post-group: `payee_key` / `display_name` `ILIKE` (case-insensitive substring); **interval bucket** match with **DEC-0086** `interval_matches` (±3d); optional **amount band** on `median_amount` (stretch — defer if over sprint cap) |
| Exclude | Drop if fingerprint ∈ `confirmed_fps` ∪ `rejection_fps`; drop if `(payee_key, interval_days)` ∈ confirmed/rejected payee-interval maps (same loads as `run_detection`) |
| Cap | Sort by `confidence_pct` DESC, `transaction_ids.len()` DESC; **LIMIT 50**; response documents cap in API meta |

**New API sketch:** `GET /api/v1/subscriptions/discover?account_id=&payee=&interval_days=&amount_min=&amount_max=&limit=50`

```json
{
  "candidates": [{
    "payee_key": "netflix",
    "display_name": "Netflix P3E460",
    "interval_days": 30,
    "median_amount": "-12.99",
    "confidence_pct": 95,
    "transaction_count": 8,
    "transaction_ids": ["ff-…"],
    "account_ids": ["114"]
  }],
  "meta": { "limit": 50, "truncated": false, "window_days": 365 }
}
```

**Alternatives:**

| Option | Verdict |
|--------|---------|
| Ad-hoc SQL `GROUP BY payee_key, account_id HAVING COUNT(*) >= 3` | **Reject** — diverges from detection cadence/tolerance; duplicates R-0009 logic; harder to keep DEC-0084/0086 aligned |
| Re-run full detection and filter pending only | **Reject** — misses merchants below auto-emit threshold or not yet synced to pending |
| Separate `/discover` route | **Accept** — keeps list API stable; Discover tab isolated (PO surface map) |

**Interval UI buckets → days (research defaults):**

| UI label | Target `interval_days` | Match |
|----------|------------------------|-------|
| Weekly | 7 | `interval_matches(7, detected)` |
| Biweekly | 14 | `interval_matches(14, detected)` |
| Monthly | 30 | `interval_matches(30, detected)` |
| Quarterly | 90 | `interval_matches(90, detected)` |
| Custom | user int | exact `interval_matches(custom, detected)` |

**Risks:** 365d window + full mirror load without `account_id` filter may scan large tx sets — **require** account filter in UI default or enforce max window when unfiltered; false negatives when &lt;3 txs (by design per R-0009).

### 2. Manual confirm API vs DEC-0085 merge (AC-2, AC-6)

**Current gap:** `POST /api/v1/subscriptions/:id/confirm` only transitions **`pending` → `confirmed`** (`repository.rs` L576 `WHERE status = 'pending'`). Explorer candidates have **no pattern row**.

**Recommendation — new confirm-from-discover endpoint:**

`POST /api/v1/subscriptions/discover/confirm`

```json
{
  "payee_key": "netflix",
  "interval_days": 30,
  "median_amount": -12.99,
  "transaction_ids": ["ff-1", "ff-2", "ff-3"],
  "kind": "subscription"
}
```

**Server flow (single transaction):**

1. Build `RecurrenceGroup` from payload + mirror validation (tx ids exist, same payee_key after normalization).
2. `fingerprint = compute_fingerprint(payee_key, interval_days, median_amount)`.
3. If `(payee_key, interval_days)` matches **confirmed** map → **`merge_confirmed_pattern`** (DEC-0085/0086): refresh row, link txs, **no** `new_detection` alert.
4. Else if matches **rejected** payee-interval → **409** with body explaining prior rejection (operator must clear rejection first — architecture gate).
5. Else **INSERT** `subscription_patterns` as **`confirmed`** directly (skip pending), link txs, set `confirmed_at`, compute `display_category_id` (§3).
6. **Do not** emit `new_detection` alert for manual confirm (operator-initiated).

**Alternatives:**

| Option | Verdict |
|--------|---------|
| Insert pending then call existing confirm | **Reject** — spurious pending card + alert noise; violates AC-2 "without auto-detection-only path" |
| Extend `confirm_pattern` to accept explorer payload | **Reject** — overloads id-based route; breaks REST clarity |
| Payee+interval already confirmed → 409 | **Reject** — conflicts with DEC-0085 merge semantics and BUG-0015 fix |

**Risks:** payload tampering with wrong `transaction_ids` — validate all txs share normalized payee and fall within interval tolerance; merge fingerprint UNIQUE conflict → same fail-safe as detection (log + 409).

### 3. Majority display category (AC-3, AC-5)

**Current gap:** `subscription_patterns` has no `display_category_id`; `RecurrenceGroup.category_ids` collected in detection but unused for display; `classify.rs` uses categories only for standing-order heuristics.

**Recommendation — compute at confirm time; store on pattern:**

| Field | Type | Notes |
|-------|------|-------|
| `display_category_id` | `TEXT NULL REFERENCES categories(firefly_id)` | Firefly category id; NULL when all linked txs uncategorized |

**Algorithm (deterministic — do not use bare `mode()` for tie-break):**

```sql
WITH linked AS (
  SELECT t.category_id, t.date
  FROM subscription_pattern_transactions spt
  JOIN transactions t ON t.firefly_id = spt.transaction_firefly_id
  WHERE spt.pattern_id = $1 AND t.category_id IS NOT NULL
),
ranked AS (
  SELECT category_id,
         COUNT(*) AS cnt,
         MAX(date) AS last_date,
         RANK() OVER (ORDER BY COUNT(*) DESC, MAX(date) DESC) AS rnk
  FROM linked
  GROUP BY category_id
)
SELECT category_id FROM ranked WHERE rnk = 1 LIMIT 1;
```

Rust equivalent on `group.category_ids` + `transaction_dates` at confirm time — same policy.

**Miscategorization (operator 1-of-12 example):** pure mode sufficient — no singleton exclusion rule for MVP. Optional future: when `N >= 6` and top share &lt; 50%, show UI warning — **defer**.

**Recompute on sync:** when `merge_confirmed_pattern` links new txs, **optionally** refresh `display_category_id` — architecture gate; **research default: recompute on merge only** (not every sync tick) to keep confirm-time semantics stable unless new txs arrive.

**Display:** resolve name via `GET /api/v1/categories` (DEC-0087); tooltip: *"Display category is the most common category among linked transactions; ties broken by most recent charge."*

**Alternatives:**

| Option | Verdict |
|--------|---------|
| `mode() WITHIN GROUP (ORDER BY category_id)` | **Reject** for tie-break — PostgreSQL picks arbitrarily ([docs](https://www.postgresql.org/docs/17/functions-aggregate.html)) |
| Operator override column | **Defer** (stretch) — AC-3 satisfied by computed default |
| Write category back to Firefly txs | **Reject** — violates read-only contract (AC-5) |

**Risks:** majority NULL when operator never categorizes in Firefly — show "Uncategorized" badge (US-0018 `__uncategorized__` labeling); category deleted in Firefly but id remains in mirror — JOIN catalog with fallback to raw id.

### 4. Operator tag schema (AC-4, AC-5)

**Recommendation — global operator tags, product DB only:**

```sql
CREATE TABLE operator_tags (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,
    slug       TEXT NOT NULL UNIQUE,  -- lowercase trimmed, spaces → hyphen
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE subscription_pattern_tags (
    pattern_id UUID NOT NULL REFERENCES subscription_patterns(id) ON DELETE CASCADE,
    tag_id     UUID NOT NULL REFERENCES operator_tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (pattern_id, tag_id)
);

CREATE INDEX idx_subscription_pattern_tags_tag ON subscription_pattern_tags(tag_id);
```

| Operation | Contract |
|-----------|----------|
| Create | `POST /api/v1/subscription-tags` `{ "name": "luxus" }` → slug dedupe |
| Rename | `PATCH /api/v1/subscription-tags/:id` |
| Delete | **Hard delete** tag row; `ON DELETE CASCADE` junction — assignments removed; confirmed patterns unaffected |
| Assign | `PUT /api/v1/subscriptions/:id/tags` `{ "tag_ids": ["…"] }` — replace set (idempotent) |
| List filter | `GET /api/v1/subscriptions?tag=luxus` — join `subscription_pattern_tags` + `operator_tags.slug`; composable with `status`, `kind`, future `account_id` |

**Alternatives:**

| Option | Verdict |
|--------|---------|
| Soft-delete tags (`deleted_at`) | **Reject for MVP** — low volume; complicates unique name constraint |
| Per-account tag namespace | **Reject** — PO global operator MVP |
| Firefly tag sync | **Reject** — AC-5 |

**Risks:** tag delete without confirmation — document in UI; slug collision on rename — enforce UNIQUE on slug; list API join cost — acceptable at household scale (&lt;500 patterns).

### 5. List API filter composition (AC-4)

Extend `ListQuery`:

| Param | Behavior |
|-------|----------|
| `status` | unchanged |
| `kind` | unchanged |
| `tag` | slug match; AND with status/kind |
| `account_id` | patterns having ≥1 linked tx on account (subquery on `subscription_pattern_transactions` ⋈ `transactions`) — **optional P2** if sprint tight; not required for Discover (discover has own account filter) |

### 6. Grafana `$tag` variable (stretch)

**Current:** `subscriptions.json` `templating.list` is **empty** — no variables.

**Recommendation (if capacity):** mirror **DEC-0087 / cashflow `$category`** pattern:

```sql
SELECT '' AS __value, 'All tags' AS __text
UNION ALL
SELECT t.slug AS __value, t.name AS __text
FROM operator_tags t
ORDER BY 2
```

Panel filter example:

```sql
… FROM subscription_patterns p
WHERE p.status = 'confirmed'
  AND ('${tag}' = '' OR EXISTS (
    SELECT 1 FROM subscription_pattern_tags spt
    JOIN operator_tags ot ON ot.id = spt.tag_id
    WHERE spt.pattern_id = p.id AND ot.slug = '${tag}'
  ))
```

**Alternatives:** defer all Grafana work — **acceptable** (AC does not require); SPA tag filter independent per DEC-0089 precedent.

**Risks:** provisioning reload operator gate (**GRAFANA_PROVISIONING_RELOAD**); empty tag table → variable still works with "All tags" only.

### 7. UI surface map (AC-1..AC-6)

| Surface | Research contract |
|---------|-------------------|
| Discover tab | New `Tab = "discover"` on `SubscriptionsPage`; search form + results table; reuse confirm modal (kind override) |
| Manual confirm | Calls `discover/confirm`; toast on merge vs create |
| Majority badge | On confirmed rows + detail drawer; `CategoryFilter` read-only display |
| Tag manager | Modal or drawer section; CRUD list |
| Tag chips | Detail drawer multi-select; filter chips on All / Standing tabs |
| Regression | No changes to `DetectionPipeline` skip order; pending tab + alert dedup untouched; OIDC smoke deferred per prior stories |

### Decision gates (carry to `/architecture`)

| Gate | Research recommendation | Alternative |
|------|-------------------------|-------------|
| Explorer engine | Reuse `detect_recurrence_groups` + post-filters | Ad-hoc SQL GROUP BY |
| Discover route | `GET /discover` + `POST /discover/confirm` | Extend pending confirm only |
| Manual confirm state | Direct `confirmed` insert | Pending intermediate |
| DEC-0085 on manual | **Merge** when payee+interval exists | 409 duplicate |
| Rejected payee-interval manual | 409 until operator clears | Silent override |
| Majority algorithm | `COUNT` + `RANK` (cnt DESC, last_date DESC); NULLs excluded | `mode()`; operator override |
| Majority refresh | Recompute on merge when new txs linked | Every sync; confirm-only forever |
| `display_category_id` | Column on `subscription_patterns` | Join table |
| Tag tables | `operator_tags` + `subscription_pattern_tags` | JSON array on pattern |
| Tag delete | Hard delete + CASCADE | Soft delete |
| Tag assign API | `PUT …/tags` replace set | PATCH per tag |
| List `?tag=` | Slug filter on list API | Client-only filter |
| Result cap | 50 | Paginated |
| Amount band filter | Stretch / P2 | Required in AC-1 |
| Grafana `$tag` | Stretch if ≤12 tasks | Defer post-MVP |
| Alert on manual confirm | **No** `new_detection` | Emit alert |

### Risks (summary)

| Risk | Mitigation |
|------|------------|
| Explorer perf on full 365d mirror | Push `account_id` to SQL; cap 50; share detection window config |
| Manual confirm bypasses rejection maps | Enforce payee-interval rejection check (409) |
| DEC-0085 merge + display category drift | Recompute majority on merge |
| `mode()` tie-break ambiguity | Use explicit RANK policy |
| Tag delete surprise | Confirm dialog in tag manager |
| Detection regression | No changes to `run_candidates` ordering; add tests for skip maps |
| Grafana stretch slips sprint | Defer; document in architecture as P2 |

**Linked:** US-0020, US-0003, US-0018, R-0080, R-0009, R-0012, DEC-0084, DEC-0085, DEC-0086, DEC-0087, DEC-0089  
**Confidence:** high (code audit + PostgreSQL/Grafana patterns); medium on majority refresh-on-merge policy and amount-band priority  
**Status:** fulfilled — released S0019 via **DEC-0098**..**DEC-0103** (`0.20.0-us0020`, 2026-06-10); extends [R-0080 § Subscriptions](research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake); retain for traceability

---
