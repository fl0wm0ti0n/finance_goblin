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

**Linked:** US-0003, R-0009, R-0012  
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

**Linked:** US-0003, R-0009, R-0010, R-0011, DEC-0004  
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

**Linked:** US-0003, DEC-0010, DEC-0007, R-0009, R-0012  
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
