# Acceptance

## Bug acceptance (canonical)

- [x] **BUG-0001** — **(A)** With OIDC unset and `AUTH_DEV_BYPASS=true` on US-0010 external profile, financegnome loads without AuthProvider/useAuth console errors; AI Chat button opens ChatPanel without `TypeError` on `user`. **(B)** All six `/analytics/{slug}` routes render Grafana dashboards; no 404 on site-root `/public/build/` or `/public/img/` (assets load via `/analytics/grafana/public/...` or equivalent proxy/root_url fix). OIDC-enabled deploy regression check passes.

- [x] **BUG-0002** — **(C)** On US-0010 external profile with `FIREFLY_BASE_URL` and `FIREFLY_PERSONAL_ACCESS_TOKEN` configured (names only), manual or scheduled Firefly sync completes successfully; Sync Status shows entity counts and no blocking 404 on `/api/v1/sync/*` from the sync page. **(D)** `GET /api/v1/plans/risk-score` on `financegnome.omniflow.cc` returns **200** (JSON risk payload or documented empty-state), not **404**. **(E)** When only Bitunix env credentials are set, Settings shows Bitunix **enabled** and **configured**; Binance/Bybit rows match operator env (no false Binance enabled+configured). OIDC-enabled and bundled-firefly deploy regression checks pass.

- [x] **BUG-0003** — **(F)** On US-0010 external profile, representative `GET /api/v1/*` product endpoints (e.g. alerts/unread-count, sync/entities, sync/runs, exchanges, forecast routes, subscriptions, ai/audit) return **200** within normal latency (not **500** after ~30s DB timeout); `GET /api/v1/settings` reports `database_host: postgres` and `database_mode: external` on omniflow. **(G)** With `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` present in container (names only), `POST /api/v1/exchanges/bitunix/test` returns **200** with connection test payload or documented auth failure message — not **400** `unknown exchange: bitunix`. **(H)** `POST /analytics/grafana/api/ds/query` for provisioned dashboards returns **200** (SQL executes); Grafana Postgres datasource reaches in-network `postgres`. OIDC-enabled and bundled-firefly deploy regression checks pass.

- [x] **BUG-0004** — **(I)** After manual exchange sync on US-0010 external profile with 922+ synced transactions, `GET /api/v1/sync/status` does not remain `state: running` with `finished_at: null` once exchange phase completes; terminal sync run status is persisted. **(J)** Subscription detection surfaces recurring patterns for operator-known subscription merchants (not permanently empty `[]` when ledger contains ≥3 recurring expenses per payee) or documents detection thresholds in UI empty-state. **(K)** Portfolio analytics Grafana panel SQL executes without `pq: syntax error at or near "UNION"`; `POST /analytics/grafana/api/ds/query` returns **200** for fixed portfolio allocation query. **(L)** Forecast and wealth product views show account-level data after successful sync/recompute; `net_worth_snapshots` and forecast series populate for representative accounts; analytics dashboards show non-empty values when `account_id` variable matches synced accounts. OIDC-enabled and bundled-firefly deploy regression checks pass.

- [x] **BUG-0005** — **(M)** Bitunix exchange sync ingests futures/margin account balances and positions (not spot wallet only); post-sync holdings include non-spot `product_type` rows when operator has futures exposure. **(N)** Bitunix futures REST uses `fapi.bitunix.com` header-auth client per R-0058; `sync_positions` / balance endpoints populate data when read-only keys permit — not empty stubs behind `enabled_futures=false` default alone. **(O)** Wealth snapshot and portfolio crypto totals reflect combined spot + futures exchange holdings on US-0010 external profile. Read-only key constraint preserved; OIDC-enabled and bundled-firefly deploy regression checks pass.

- [x] **BUG-0006** — **(P)** After Firefly sync with 922+ transactions on US-0010 external profile, AI Chat answers category/spending questions using `get_transactions` data—not messages claiming no expenses or unavailable data when mirror rows exist for the queried period. **(Q)** Firefly sync persists `category_id` (or equivalent category mapping) on mirror `transactions` rows used by `TransactionsRepository::aggregates_by_category`. **(R)** `get_transactions` aggregate JSON includes explicit period transaction totals/counts and distinguishes empty period vs uncategorized/zero-outflow cases under `allow_raw_transactions=false`. Privacy redaction and six-tool registry constraints preserved; OIDC-enabled deploy regression checks pass.

- [x] **BUG-0007** — **(S)** On US-0010 external profile after successful sync, AI Chat enumerates subscription/streaming **merchant/payee names** from `get_subscriptions` and/or `get_transactions` when operator asks to list services (e.g. after reporting cancelable streaming total)—not generic industry-only lists or "cannot retrieve" when mirror holds recurring patterns. **(T)** Merchant/category queries (**Strom**/electricity, **Amazon** for Jan–Oct 2023, streaming) return data-backed amounts or explicit empty-state showing category/description/account search was attempted—not blanket "no expenses" when mirror plausibly contains matches. **(U)** AI fuses **category, transaction name/description, account, and amounts** in tool orchestration without requiring the user to name merchants. Privacy `allow_raw_transactions=false` and six-tool registry preserved; OIDC-enabled deploy regression checks pass. Discovery documents RAG vs tool-enhancement tradeoff (intake note V—not acceptance gate). Verify-work 2026-06-07: S/U pass; T partial (`group_by: month` + `category_search` advisory, non-blocking).

- [ ] **BUG-0008** — **(W)** On US-0010 external profile after sync, subscription-scoped alert unread count reconciles with visible `/subscriptions` list rows (pending + confirmed + standing orders per tab contract)—not **33 alerts vs 11 list rows** or equivalent mismatch without documented dismissed/rejected/dedup semantics in UI. **(X)** Subscription detection surfaces materially more recurring patterns from 922+ synced transactions for operator-known merchants (higher recall than post-BUG-0004 **11 pending** alone) without alert spam; improved rules and/or AI-assisted detection documented if used. OIDC-enabled and bundled-firefly deploy regression checks pass.

- [x] **BUG-0009** — **(Y)** On US-0010 external profile with 922+ synced transactions, representative Grafana analytics dashboards (`POST /analytics/grafana/api/ds/query` for cashflow, portfolio, subscriptions, budgets, forecast) return **200** with **non-empty panel values** when `account_id` variable matches synced Firefly asset accounts—not persistent empty panels post-BUG-0004. **(Z)** Operator has **cross-account value overview** in analytics (Grafana summary panel/table or documented equivalent via embedded wealth link showing per-account totals). Six `/analytics/{slug}` routes operator smoke pass. OIDC-enabled deploy regression checks pass.

- [x] **BUG-0010** — **(AA)** On US-0010 external profile, Forecast UI/API shows **plausible signed balances** for selected account—including 3-month **End balance** not implausible **-25365.78** without explicit deficit scenario; daily/monthly/long-term series populated after recompute. **(AB)** Wealth UI/API shows **non-empty** account breakdown and `total_eur` for synced Firefly asset accounts. **(AC)** ML forecast path runs when US-0009 sidecar/config available on profile, or UI/API **accurately** states degraded baseline-only mode; message **"ML skipped: ML forecast unavailable…"** only when ML truly unavailable and baseline DEC-0007 numbers are correct. OIDC-enabled deploy regression checks pass. Epic ML hardening tracked in **US-0013**.

- [ ] **BUG-0011** — **(AD)** On `/planning` Scenarios, clicking **"Start empty and add lines"** creates an editable empty plan (add-line UX)—not silent no-op. **(AE)** Compare tab with empty/minimal plan shows **zero or neutral deltas**, not illogical aggregates (e.g. **-127489.44** monthly delta, **-4042.41** projected month-end) from missing baseline guards. **(AF)** `GET /api/v1/plans/active/plan-vs-actual` returns **200** with JSON when active plan exists, or **200 documented empty-state/guided UX** when none—not raw **404** breaking Plan vs Actual tab. OIDC-enabled deploy regression checks pass. Intuitive UX epic tracked in **US-0014**.

- [x] **BUG-0012** — **(AG)** On US-0010 external profile after Full Firefly sync and forecast recompute, monthly forecast API/UI shows **non-zero Income** for a funded account when mirror holds income-category inflows (salary, refunds, etc.) in the forecast month—not permanently **Income: 0** while categorized rows exist. **(AH)** Monthly forecast shows **non-zero Fixed** when mirror holds fixed-cost category outflows (rent, utilities, standing orders per DEC-0007 category map)—not **Fixed: 0** with all spend under Variable only. OIDC-enabled deploy regression checks pass. AI-assisted bucket mapping epic tracked in **US-0015**.

## US-0001 — Self-hosted platform foundation & Firefly read-only integration

- [x] Docker Compose starts `flow-finance-ai`, `firefly-iii`, and `grafana` (minimal profile) without embedded PostgreSQL
- [x] External PostgreSQL connection configurable via TOML/env with `database.mode = "external"`
- [x] Rust backend serves health endpoint and authenticated API skeleton
- [x] React UI shell loads with OIDC auth flow wired (login redirect + session)
- [x] Firefly Connector syncs accounts, transactions, categories, budgets, tags, and piggy banks via `/api/v1`
- [x] Sync scheduler runs on configurable interval; manual sync trigger available
- [x] No write operations are issued to Firefly III API (read-only verified in integration test or audit log)
- [x] Dev environment can connect to running local Firefly III instance per operator setup

## US-0002 — Cashflow forecasting & Grafana analytics (MVP)

- [x] Daily forecast shows tomorrow, next week, and month-end projected balance per account
- [x] Monthly forecast shows income, fixed costs, variable costs, and free cashflow
- [x] Long-term forecasts available for 3, 6, 12, and 24 months
- [x] Forecast data persisted in TimescaleDB hypertables
- [x] React UI displays forecast charts (ECharts) for selected account and horizon
- [x] Grafana Dashboard 1 (Cashflow: balance, forecast, scarcity markers) provisioned
- [x] Grafana Dashboard 5 (Forecast horizons) provisioned
- [x] Forecasts recompute after Firefly sync completes

## US-0003 — Subscription detection, price changes & alerts

- [x] Detection engine identifies recurring patterns with confidence score (95/80/60% tiers)
- [x] User receives notification for new detected subscription with confirm/reject actions
- [x] Confirmed subscriptions appear in subscription list with interval and amount
- [x] Standing-order (Dauerauftrag) patterns detected separately from discretionary subscriptions
- [x] Price increase/decrease detected when amount changes for confirmed subscription
- [x] Alert fired on new subscription detection and on price change
- [x] Grafana Dashboard 2 (Subscriptions, price changes, new) provisioned
- [x] Rejected patterns do not appear in subscription forecasts or alerts

## US-0004 — Financial planning, scenarios & plan-vs-actual

- [x] User can create named plan with scenario adjustments (e.g. +300 €/month leasing)
- [x] Built-in scenario templates: current (Ist), leasing, savings mode, house purchase
- [x] Plan versions (v1, v2, v3) can be created and compared side-by-side
- [x] Daily plan-vs-Ist comparison shows planned, actual, and deviation amounts
- [x] Grafana Dashboard 3 (Budget plan/ist/deviation) reflects active plan
- [x] Scenario changes do not modify Firefly transaction data

## US-0005 — Wealth analysis, budget drift & scarcity alerts

- [x] Net worth view aggregates Firefly-linked accounts (giro, savings, etc.)
- [x] Scarcity alert fires when projected balance falls below configurable threshold (e.g. 200 €)
- [x] Budget drift alert fires when category spending exceeds plan by configurable % (e.g. +20%)
- [x] Plan viability alert fires when active scenario becomes infeasible per forecast
- [x] Alert inbox in React UI lists active alerts with acknowledge/dismiss
- [x] Grafana Dashboard 4 shows total wealth (non-crypto until US-0007)

## US-0006 — AI financial assistant with privacy-safe tool layer

- [x] Chat UI accepts natural-language questions about finances
- [x] AI invokes only registered tools (`get_transactions`, `get_subscriptions`, `get_forecast`, `get_budget_status`, `get_portfolio`, `simulate_plan`) — no direct DB access
- [x] OpenAI provider configurable via environment/config
- [x] Privacy settings: `allow_raw_transactions`, `redact_iban`, `redact_counterparties` honored in tool responses
- [x] Example queries work: affordability check, subscription price changes, budget overrun explanation, savings from cancelling subscription, top spending categories
- [x] Tool call audit log available for operator review

## US-0007 — Crypto exchange portfolio integration

- [x] Connectors for Binance, Bybit, and Bitunix import balances, positions, trades, transfers, funding, and PnL
- [x] Portfolio Engine calculates realized gains (closed positions), unrealized gains (open positions), and total return
- [x] Crypto holdings included in net worth view (extends US-0005)
- [x] Portfolio allocation scenarios supported (e.g. 50% ETF / 50% crypto)
- [x] Grafana Dashboard 4 shows crypto slice and portfolio performance
- [x] API keys stored in self-hosted secrets/config, not transmitted externally

## US-0008 — Local & self-hosted AI provider support

- [x] Provider selector supports OpenAI and local OpenAI-compatible endpoints
- [x] Ollama integration works when Ollama service is running (full Docker Compose profile)
- [x] LM Studio / LocalAI / vLLM work via OpenAI-compatible base URL configuration
- [x] US-0006 tool layer and privacy settings unchanged across providers
- [x] Chat functionality verified end-to-end with local provider (no external API call when local selected)

## US-0009 — Advanced forecasting with ML & risk assessment

- [x] Seasonal patterns detected and applied to monthly cashflow forecasts
- [x] ML-enhanced forecast model produces 6–24 month projections with confidence bands
- [x] Portfolio performance forecast available when US-0007 data present
- [x] Risk assessment score displayed for active plan scenarios
- [x] Grafana Dashboard 5 extended with ML forecast and risk panels
- [x] User can compare baseline (US-0002) vs ML-enhanced forecast in UI

## US-0010 — External Firefly/Postgres & Traefik deployment on omniflow host

- [x] `docker compose --profile external up` (or documented equivalent) does **not** create `firefly-iii` or `postgres` containers
- [x] `flow-finance-ai` joins external `traefik` network and reaches `postgres` and `firefly` by Docker DNS
- [x] Traefik routes `https://financegnome.omniflow.cc` to Flow Finance AI with `auth` middleware and valid TLS
- [x] Backend `/health` returns success when wired to external DB and Firefly PAT is configured
- [x] `.env.example` documents all operator-required variables for external mode
- [x] Operator smoke test recorded (compose up + health + Traefik route check on Debian host)

## US-0011 — Unified analytics UI in financegnome (Grafana in-app)

- [x] Sidebar **Analytics** section lists every provisioned Grafana dashboard (Platform Health + dashboards 1–5) with in-app routes under financegnome host
- [x] User can open each dashboard without leaving financegnome (no `target=_blank` Grafana tab required for default flows)
- [x] Embedded or proxied views work with omniflow Traefik + `auth` middleware (or documented operator auth-off dev path)
- [x] Existing React chart pages (Forecast, Wealth, Planning, Subscriptions, Alerts) remain functional (regression)
- [x] Wealth page no longer relies on external-only Grafana link as the primary portfolio analytics entry
- [x] Future-chart guideline documented: new product charts added inside financegnome shell (React + API default; Grafana embed exception list)
- [x] Operator guide describes single-URL analytics UX; optional separate Grafana public host not required for acceptance

## US-0012 — Auto-provision application database on first start

- [x] On cold start against external Postgres, if `DATABASE_NAME` is missing, backend creates the database idempotently before migrations
- [x] Existing `flow_finance_ai` database is never dropped or recreated by bootstrap
- [x] Bootstrap attempts `CREATE EXTENSION IF NOT EXISTS timescaledb` on the app database when server provides the extension; otherwise fails with actionable log pointing to runbook host install
- [x] When app role lacks `CREATEDB`, optional `DATABASE_BOOTSTRAP_URL` (or documented equivalent) enables bootstrap; missing privilege fails closed with clear error
- [x] `.env.example` and runbook document bootstrap env vars and omniflow shared-Postgres behavior
- [x] Automated test or CI fixture proves create-if-missing path (mock or testcontainers) without requiring operator manual SQL

## US-0013 — Production ML forecast & wealth analytics hardening

- [ ] StatsForecast/ML sidecar reachable and invoked on US-0010 external profile after sync (not `full` Compose profile only; `FORECAST_ML_ENABLED` on omniflow external merge)
- [ ] ML-enhanced forecast produces non-flat 6–24 month projections with confidence bands when data sufficient
- [ ] UI compare baseline vs ML-enhanced forecast works on production profile (extends US-0009)
- [ ] Wealth analytics integrate ML overlay without empty/incorrect totals
- [ ] Operator runbook documents ML deps, degraded-mode, and health checks on omniflow
- [ ] Parent defect **BUG-0010** AC rows AA/AB/AC pass before or with this story closure

## US-0015 — AI-assisted forecast category bucket mapping

- [ ] Forecast projection applies DEC-0007 category→bucket map using mirror `category_id` and recurring pattern labels (baseline path; parent **BUG-0012** AG/AH pass first)
- [ ] AI layer proposes income/fixed/variable bucket for uncategorized or ambiguous mirror rows with confidence metadata under privacy defaults
- [ ] Monthly forecast UI/API surfaces AI-mapped buckets distinctly from config-mapped buckets when AI path used
- [ ] AI bucket mapping reuses US-0006 audit/privacy constraints; no Firefly write-back
- [ ] Parent defect **BUG-0012** AC rows AG/AH pass before or with this story closure

## US-0014 — Planning mode intuitive UX completion

- [ ] First-visit planning onboarding guides scenario creation without broken empty states
- [ ] Empty-plan → add-lines flow is discoverable and completes without silent failures
- [ ] Compare tab shows contextual empty-state copy and sane zero baselines
- [ ] Plan-vs-actual tab guides operator when no active plan (no raw 404 UX)
- [ ] Built-in scenario templates discoverable from Scenarios tab
- [ ] Parent defect **BUG-0011** AC rows AD/AE/AF pass before or with this story closure
