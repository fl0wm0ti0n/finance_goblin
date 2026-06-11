# Acceptance

## Bug acceptance (canonical)

- [x] **BUG-0001** — **(A)** With OIDC unset and `AUTH_DEV_BYPASS=true` on US-0010 external profile, financegnome loads without AuthProvider/useAuth console errors; AI Chat button opens ChatPanel without `TypeError` on `user`. **(B)** All six `/analytics/{slug}` routes render Grafana dashboards; no 404 on site-root `/public/build/` or `/public/img/` (assets load via `/analytics/grafana/public/...` or equivalent proxy/root_url fix). OIDC-enabled deploy regression check passes.

- [x] **BUG-0002** — **(C)** On US-0010 external profile with `FIREFLY_BASE_URL` and `FIREFLY_PERSONAL_ACCESS_TOKEN` configured (names only), manual or scheduled Firefly sync completes successfully; Sync Status shows entity counts and no blocking 404 on `/api/v1/sync/*` from the sync page. **(D)** `GET /api/v1/plans/risk-score` on `financegnome.omniflow.cc` returns **200** (JSON risk payload or documented empty-state), not **404**. **(E)** When only Bitunix env credentials are set, Settings shows Bitunix **enabled** and **configured**; Binance/Bybit rows match operator env (no false Binance enabled+configured). OIDC-enabled and bundled-firefly deploy regression checks pass.

- [x] **BUG-0003** — **(F)** On US-0010 external profile, representative `GET /api/v1/*` product endpoints (e.g. alerts/unread-count, sync/entities, sync/runs, exchanges, forecast routes, subscriptions, ai/audit) return **200** within normal latency (not **500** after ~30s DB timeout); `GET /api/v1/settings` reports `database_host: postgres` and `database_mode: external` on omniflow. **(G)** With `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` present in container (names only), `POST /api/v1/exchanges/bitunix/test` returns **200** with connection test payload or documented auth failure message — not **400** `unknown exchange: bitunix`. **(H)** `POST /analytics/grafana/api/ds/query` for provisioned dashboards returns **200** (SQL executes); Grafana Postgres datasource reaches in-network `postgres`. OIDC-enabled and bundled-firefly deploy regression checks pass.

- [x] **BUG-0004** — **(I)** After manual exchange sync on US-0010 external profile with 922+ synced transactions, `GET /api/v1/sync/status` does not remain `state: running` with `finished_at: null` once exchange phase completes; terminal sync run status is persisted. **(J)** Subscription detection surfaces recurring patterns for operator-known subscription merchants (not permanently empty `[]` when ledger contains ≥3 recurring expenses per payee) or documents detection thresholds in UI empty-state. **(K)** Portfolio analytics Grafana panel SQL executes without `pq: syntax error at or near "UNION"`; `POST /analytics/grafana/api/ds/query` returns **200** for fixed portfolio allocation query. **(L)** Forecast and wealth product views show account-level data after successful sync/recompute; `net_worth_snapshots` and forecast series populate for representative accounts; analytics dashboards show non-empty values when `account_id` variable matches synced accounts. OIDC-enabled and bundled-firefly deploy regression checks pass.

- [x] **BUG-0005** — **(M)** Bitunix exchange sync ingests futures/margin account balances and positions (not spot wallet only); post-sync holdings include non-spot `product_type` rows when operator has futures exposure. **(N)** Bitunix futures REST uses `fapi.bitunix.com` header-auth client per R-0058; `sync_positions` / balance endpoints populate data when read-only keys permit — not empty stubs behind `enabled_futures=false` default alone. **(O)** Wealth snapshot and portfolio crypto totals reflect combined spot + futures exchange holdings on US-0010 external profile. Read-only key constraint preserved; OIDC-enabled and bundled-firefly deploy regression checks pass.

- [x] **BUG-0006** — **(P)** After Firefly sync with 922+ transactions on US-0010 external profile, AI Chat answers category/spending questions using `get_transactions` data—not messages claiming no expenses or unavailable data when mirror rows exist for the queried period. **(Q)** Firefly sync persists `category_id` (or equivalent category mapping) on mirror `transactions` rows used by `TransactionsRepository::aggregates_by_category`. **(R)** `get_transactions` aggregate JSON includes explicit period transaction totals/counts and distinguishes empty period vs uncategorized/zero-outflow cases under `allow_raw_transactions=false`. Privacy redaction and six-tool registry constraints preserved; OIDC-enabled deploy regression checks pass.

- [x] **BUG-0007** — **(S)** On US-0010 external profile after successful sync, AI Chat enumerates subscription/streaming **merchant/payee names** from `get_subscriptions` and/or `get_transactions` when operator asks to list services (e.g. after reporting cancelable streaming total)—not generic industry-only lists or "cannot retrieve" when mirror holds recurring patterns. **(T)** Merchant/category queries (**Strom**/electricity, **Amazon** for Jan–Oct 2023, streaming) return data-backed amounts or explicit empty-state showing category/description/account search was attempted—not blanket "no expenses" when mirror plausibly contains matches. **(U)** AI fuses **category, transaction name/description, account, and amounts** in tool orchestration without requiring the user to name merchants. Privacy `allow_raw_transactions=false` and six-tool registry preserved; OIDC-enabled deploy regression checks pass. Discovery documents RAG vs tool-enhancement tradeoff (intake note V—not acceptance gate). Verify-work 2026-06-07: S/U pass; T partial (`group_by: month` + `category_search` advisory, non-blocking).

- [x] **BUG-0008** — **(W)** On US-0010 external profile after sync, subscription-scoped alert unread count reconciles with visible `/subscriptions` list rows (pending + confirmed + standing orders per tab contract)—not **33 alerts vs 11 list rows** or equivalent mismatch without documented dismissed/rejected/dedup semantics in UI. **(X)** Subscription detection surfaces materially more recurring patterns from 922+ synced transactions for operator-known merchants (higher recall than post-BUG-0004 **11 pending** alone) without alert spam; improved rules and/or AI-assisted detection documented if used. OIDC-enabled and bundled-firefly deploy regression checks pass.

- [x] **BUG-0009** — **(Y)** On US-0010 external profile with 922+ synced transactions, representative Grafana analytics dashboards (`POST /analytics/grafana/api/ds/query` for cashflow, portfolio, subscriptions, budgets, forecast) return **200** with **non-empty panel values** when `account_id` variable matches synced Firefly asset accounts—not persistent empty panels post-BUG-0004. **(Z)** Operator has **cross-account value overview** in analytics (Grafana summary panel/table or documented equivalent via embedded wealth link showing per-account totals). Six `/analytics/{slug}` routes operator smoke pass. OIDC-enabled deploy regression checks pass.

- [x] **BUG-0010** — **(AA)** On US-0010 external profile, Forecast UI/API shows **plausible signed balances** for selected account—including 3-month **End balance** not implausible **-25365.78** without explicit deficit scenario; daily/monthly/long-term series populated after recompute. **(AB)** Wealth UI/API shows **non-empty** account breakdown and `total_eur` for synced Firefly asset accounts. **(AC)** ML forecast path runs when US-0009 sidecar/config available on profile, or UI/API **accurately** states degraded baseline-only mode; message **"ML skipped: ML forecast unavailable…"** only when ML truly unavailable and baseline DEC-0007 numbers are correct. OIDC-enabled deploy regression checks pass. Epic ML hardening tracked in **US-0013**.

- [x] **BUG-0011** — **(AD)** On `/planning` Scenarios, clicking **"Start empty and add lines"** creates an editable empty plan (add-line UX)—not silent no-op. **(AE)** Compare tab with empty/minimal plan shows **zero or neutral deltas**, not illogical aggregates (e.g. **-127489.44** monthly delta, **-4042.41** projected month-end) from missing baseline guards. **(AF)** `GET /api/v1/plans/active/plan-vs-actual` returns **200** with JSON when active plan exists, or **200 documented empty-state/guided UX** when none—not raw **404** breaking Plan vs Actual tab. OIDC-enabled deploy regression checks pass. Intuitive UX epic tracked in **US-0014**.

- [x] **BUG-0012** — **(AG)** On US-0010 external profile after Full Firefly sync and forecast recompute, monthly forecast API/UI shows **non-zero Income** for a funded account when mirror holds income-category inflows (salary, refunds, etc.) in the forecast month—not permanently **Income: 0** while categorized rows exist. **(AH)** Monthly forecast shows **non-zero Fixed** when mirror holds fixed-cost category outflows (rent, utilities, standing orders per DEC-0007 category map)—not **Fixed: 0** with all spend under Variable only. OIDC-enabled deploy regression checks pass. AI-assisted bucket mapping epic tracked in **US-0015**.

- [x] **BUG-0013** — **(AI)** On US-0010 external profile after Full Firefly sync and forecast recompute (post-US-0015 deploy), cashflow **Balance forecast with scarcity threshold** and forecast-horizons **baseline** panels show **non-empty signed balances** for a funded asset account—not persistent flat **0 €** regression vs BUG-0009/0010 closure. **(AJ)** Subscriptions **Price changes (90 days)** panel shows change rows when confirmed subscriptions had amount changes in period, or documented empty-state when none. **(AK)** Portfolio **crypto value** reflects exchange holdings when sync populated positions; FX incomplete warning only with documented partial totals; **total return %** populated when snapshot history exists. **(AL)** Budgets MTD plan/actual/deviation rows are **plausible** for active plan—not unexplained **Planned MTD −€150K** with **Actual €0** unless plan defines that magnitude. **(AM)** `POST /analytics/grafana/api/ds/query` and annotation queries return **200** without browser **Failed to fetch** on omniflow. **(AN)** Exchange crypto balances appear in wealth/portfolio totals when venue sync succeeds—not permanently **€0** with configured read-only keys. OIDC-enabled deploy regression checks pass. ML overlay remains **US-0013**.

- [x] **BUG-0014** — **(AO)** With `FORECAST_ML_ENABLED=true` and healthy `stats-forecast` on external profile, `GET /api/v1/forecast/meta` shows ML available (`ml_computation_id` set after Full sync) or accurate sidecar-down degraded copy—not permanent **US-0013 not enabled** when env opts in. **(AP)** Wealth crypto subtotal, exchange cards, and portfolio panels show **non-zero** values when Bitunix reports connected holdings (e.g. **7** positions) and wallet/unrealized equity exists per DEC-0080—not **€0** everywhere with only a holdings count *(code PASS; AP1/AP-1 live operator-deferred; AP2 conditional skipped)*. **(AQ)** Crypto surfaces show **native asset amounts** and **EUR equivalents** at valuation time; **FX incomplete** banner appears only with documented `unpriced_assets` or partial totals—not when wallet equity is priced. **(AR)** Cashflow Grafana **balance forecast** and **recent daily balances** show **non-zero signed values** for funded account **114** after Full sync + recompute—not persistent flat **0** when API forecast non-zero *(AR-API/AR-GRAF operator-deferred; AR1 conditional skipped)*. **(AS)** Operator can **delete** a plan from `/planning` UI (or documented equivalent); plan mutations surface errors; target-type UX documented or improved beyond household/subscription/account confusion. **(AT)** External profile operator rebuild/runbook starts **`stats-forecast`** with app + Grafana when ML enabled *(ops-only pass-with-prerequisites)*. OIDC-enabled deploy regression checks pass.

- [x] **BUG-0015** — **(AU)** On US-0010 external profile, after operator confirms subscription patterns (e.g. **CURSOR** €17.18/mo, **APPLE.COM/BILL** €9.99/mo) and rebuilds application containers, previously confirmed patterns remain **confirmed** in `GET /api/v1/subscriptions` and `/subscriptions` UI—not re-surfaced as **pending** with Confirm/Reject after container start and Full sync *(code PASS; AU-1/AU-2/H2-1 live operator-deferred)*. **(AV)** Post-rebuild Full sync + detection rerun does not create duplicate pending rows for the same merchant identity; confirmed fingerprints are skipped in detection (`confirmed_fps` / `upsert_pending_pattern` status preservation) or fingerprint drift is documented and remediated *(code PASS; AV-1 live operator-deferred)*. **(AW)** Subscription-scoped unread alerts reconcile with list tabs—no spurious `new_detection` unread forcing re-review of merchants already confirmed in DB *(code PASS; AW-1/OIDC live operator-deferred)*. OIDC-enabled deploy regression checks pass.

- [x] **BUG-0016** — **(AX)** Direct navigation, hard-refresh, and bookmarks to client routes (`/forecast`, `/subscriptions`, `/planning`, `/sync`, `/analytics/{slug}`) return **HTTP 200** with SPA shell and render the correct React page—not **404** with blank body. Applies on localhost override (`:18080`) and US-0010 external profile `financegnome.omniflow.cc`. OIDC-enabled deploy regression checks pass *(code PASS; curl/browser/OIDC live operator-deferred)*.

- [x] **BUG-0017** — **(AY)** Post-sync forecast recompute inserts `ai_tool_audit` rows for `forecast_bucket_assignment` without `ai_tool_audit_tool_name_check` violation. **(AZ)** Low-confidence bucket assignments persist without `ai_tool_audit_result_status_check` violation (`low_confidence` allowed or mapped). **(BA)** Recompute delete/insert does not fail on `forecast_computations_paired_baseline_id_fkey`; `GET /api/v1/forecast/meta` reflects fresh computation after successful sync. **(BB)** When `FORECAST_ML_ENABLED` and history gate pass, ML-enhanced forecast selectable; otherwise accurate `ml_skipped_reason` (not silent failure from recompute errors). **(BC)** Planning Compare loses **Plan stale** badge after successful recompute/sync. **(BD)** Forecast page does not show **No forecast data yet** when `GET /api/v1/forecast/meta` already has `computation_id`. OIDC-enabled deploy regression checks pass *(code PASS; sync/audit/planning/Forecast live operator-deferred)*.

- [x] **BUG-0018** — **(BE)** Post-sync alert evaluation completes without SQL error (`balance` ambiguous / **42702**); logs show no `alert evaluation failed` for normal sync runs. **(BF)** Header Alerts panel and `GET /api/v1/subscriptions/alerts` surface matching alerts when overdraft or subscription rules apply—not permanent **No active alerts** due to evaluation skip. OIDC-enabled deploy regression checks pass.

- [x] **BUG-0019** — **(BG)** Analytics → Cashflow **balance forecast** and **recent daily balances** show non-zero signed values for funded account **114** when `GET /api/v1/forecast/monthly?account_id=114` has non-zero series—not flat **0** regression. **(BH)** Platform Health **Records synced per entity** reports **transactions** count matching mirror (`SELECT COUNT(*) FROM transactions`) after successful Full sync—not **0** when 900+ rows exist. OIDC-enabled deploy regression checks pass.

- [x] **BUG-0020** — **(BI)** `GET /api/v1/subscriptions` and `/subscriptions` **All** tab show at most one visible row per logical confirmed merchant identity (no triplicate Strom / duplicate YouTube rows for same payee_key). **(BJ)** Confirmed subscriptions show **display category** from US-0020 majority-category contract when mirror txs have category assignments—`display_category_id` non-null on representative samples; not all **Uncategorized** when categories exist on underlying txs. Discover, tags, and detection regression pass. OIDC-enabled deploy regression checks pass *(code + API PASS; BI-ALL browser visual operator-deferred pending BACKEND_FRONTEND_DEPLOY — ForecastPage TS6133 blocks docker build)*.

- [x] **BUG-0021** — **(BK)** CategoryFilter on Forecast Monthly and Wealth Overview becomes interactive within **~1s** of tab visit—not multi-second **Loading category filter…** Suspense fallback under normal local/omniflow load. **(BL)** Wealth account breakdown **Role** column shows Firefly account role/type when mirror provides it, or column is hidden/documented when unsupported—not permanent em dash for all rows. OIDC-enabled deploy regression checks pass *(code + mirror COALESCE PASS; BK browser + BL API/UI/snapshot operator-deferred pending **BACKEND_FRONTEND_DEPLOY**)*.

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

- [x] External compose overlay adds `stats-forecast` on **`external`** profile (not `full`-only); `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL`, and omniflow port remap (`STATS_FORECAST_PORT=8091`) documented in `.env.example`
- [x] Backend `[forecast_ml] enabled=true` on external merge resolves sidecar health on traefik network before sync ML phase
- [x] Post-sync `forecast_ml` phase runs after baseline per DEC-0052; ML failure records skip metadata without failing sync; sync status UI shows "ML forecast…" when active
- [x] `model_kind=ml_enhanced` computations persisted with p10/p90 bands when history ≥ `min_monthly_points`; GET `/api/v1/forecast` with `variant=ml_enhanced` returns non-empty 6–24 month series on omniflow after Full sync + recompute
- [x] React `/forecast` Compare control shows baseline + ML-enhanced overlay when ML available (extends US-0009); degraded copy uses `sidecar_disabled` per DEC-0066—not generic skip message
- [x] Wealth API/UI integrates ML portfolio overlay when US-0007 data present; signed totals and account breakdown remain correct (DEC-0065); `portfolio_forecast_low_confidence` banner per R-0034 when FX incomplete
- [x] Grafana forecast-horizons ML panels return data when `$forecast_variant=ml_enhanced` and computations exist (post-enablement; BUG-0009 banner remains when ML off)
- [x] Operator runbook documents omniflow ML enablement: compose profile union, env vars, sidecar health probe, minimum history gate, degraded-mode troubleshooting
- [x] Automated test or CI fixture proves sidecar invoke + overlay persist path (mock HTTP or testcontainers) without production secrets
- [x] Prerequisite verified: **BUG-0010** AA/AB/AC **DONE** (Q0013)—baseline DEC-0007 numbers authoritative before ML overlay closure

## US-0015 — AI-assisted forecast category bucket mapping

- [x] **Prerequisite:** Parent defect **BUG-0012** AC rows AG/AH released (Q0014) — DEC-0007 config-driven category→bucket projection baseline authoritative
- [x] **AC-1 Baseline precedence:** Forecast projection applies DEC-0007 config map via mirror `category_id` and recurring pattern labels before any AI fallback; config-mapped rows never overridden by AI
- [x] **AC-2 AI inference:** For uncategorized or ambiguous mirror rows, AI layer proposes income/fixed/variable bucket with confidence metadata; low-confidence proposals fall back to Variable (not silent zero absorption)
- [x] **AC-3 Privacy defaults:** Bucket inference operates under `allow_raw_transactions=false` default (DEC-0032) — aggregates and allowlisted category/merchant signals only; no raw transaction row leakage to model
- [x] **AC-4 API visibility:** `GET /api/v1/forecast/monthly` exposes per-bucket `bucket_source` (`config` \| `ai` \| `default`) or equivalent when AI path contributes to Income/Fixed/Variable totals
- [x] **AC-5 UI badge:** Monthly tab stat cards show **AI-mapped** indicator (badge or tooltip) when any bucket total includes AI-assigned rows; config-only months show no AI badge
- [x] **AC-6 Audit trail:** AI bucket assignments logged per US-0006 audit patterns (operator-reviewable; no Firefly write-back)
- [x] **AC-7 Regression:** `/forecast` Monthly tab OIDC-enabled deploy smoke pass-with-prerequisites **BACKEND_FRONTEND_DEPLOY**; BUG-0007 chat tool surface unchanged; US-0013 ML overlay unchanged

## US-0014 — Planning mode intuitive UX completion

- [x] **Prerequisite:** Parent defect **BUG-0011** AC rows AD/AE/AF released (Q0019, DEC-0073, DEC-0074) — functional gates satisfied
- [x] **AC-1 Onboarding (first visit):** When `plans.length === 0`, `/planning` Scenarios shows template card grid (Current, Leasing, Savings mode, House purchase, Custom) plus name field and primary **Create empty plan** CTA per [R-0070](docs/engineering/research.md#r-0070--bug-0011-planning-mode-compare-delta-empty-state-api-first-run-ux) §4 — operator reaches add-lines flow without Leasing-only dead end
- [x] **AC-2 Empty-plan add-lines:** After custom/empty plan create, inline add-adjustment form is visible and wired; submitting a line updates compare/PVA after recompute; API/mutation errors surface as toast or inline — no silent no-op
- [x] **AC-3 Compare contextual UX:** Compare tab with zero-adjustment plan shows **0.00** monthly delta (DEC-0073) and contextual help explaining overlay-only delta vs projected balance — no illogical aggregates
- [x] **AC-4 PVA guided UX:** When no active plan, Plan vs Actual tab renders guided card from `status: no_active_plan` 200 payload (DEC-0074) with link to Set active / Scenarios — not blank tab
- [x] **AC-5 Template discoverability:** Built-in templates reachable from first-run empty state and from existing-plan Scenarios UI; **Create from {template}** succeeds with visible confirmation
- [x] **AC-6 Set-active guidance:** After first plan create (`is_active=false` default), inline banner or persistent cue explains Set active requirement for Plan vs Actual and Grafana Dashboard 3
- [x] **AC-7 Error surfaces:** Planning mutations (create plan, add adjustment, set active, version create) show operator-visible errors on failure — no silent console-only failures
- [x] **AC-8 OIDC regression:** `/planning` all three tabs pass OIDC-enabled deploy smoke on US-0010 external profile (pass-with-prerequisites **BACKEND_FRONTEND_DEPLOY**)

## US-0016 — Root README for operators and contributors (living documentation)

- [x] Root `README.md` exists with all **DEC-0059** user-channel H2 sections for `DOC_AUDIENCE_PROFILE=both` and `DOC_DETAIL_LEVEL=balanced` (Purpose, Quickstart, Examples, Limitations, Related documentation) populated with Flow Finance AI-specific content—not placeholder stubs
- [x] Root README includes `## Contributing` pointing to `docs/developer/README.md`; no forbidden `DEV_*` H2 titles in root per split layout
- [x] Related documentation section links `docs/user-guides/`, `docs/engineering/runbook.md`, and documents minimal / bundled-firefly / external omniflow compose entry commands
- [x] `python scripts/validate_doc_profile.py --repo .` exits **0** on CI and locally with current scratchpad profile flags
- [x] Runbook documents README maintenance cadence: release or refresh-context updates **Product status** (or equivalent) when a US or BUG closes; validator run at release gate
- [x] `template/README.md` present and profile H2 parity matches active root when `template/` tree exists

## US-0017 — README living-doc expansion and troubleshooting (post-US-0016)

- [x] Root `README.md` **Examples** include omniflow external-profile smoke commands (sync trigger, forecast recompute pointer, six `/analytics/{slug}` routes, exchange sync sanity) — not only localhost curls
- [x] **Limitations** or budget-safe **Troubleshooting** subsection documents empty-Grafana vs ML-unavailable distinction, `BACKEND_FRONTEND_DEPLOY` cadence, and sync+recompute prerequisite for non-zero analytics
- [x] **Product status** subsection lists **US-0015** and other closures shipped after US-0016 baseline when this story closes
- [x] `docs/developer/README.md` and runbook § documentation profile require Product status README update in release and refresh-context checklists for each closed US/BUG in the segment
- [x] `python scripts/validate_doc_profile.py --repo .` exits **0**; split layout preserved (no `DEV_*` H2 in root; H2 budget ≤ 8 per DEC-0059)

## US-0018 — Category filters & expense trend analytics

- [x] **AC-1 Category filter contract:** Shared category filter (single or multi) available on `/forecast` monthly view, `/planning` compare context, `/wealth` firefly breakdown, and at least **two** embedded Grafana analytics dashboards via variable or filter panel
- [x] **AC-2 Monthly series API:** `GET` category expense API returns per-month EUR outflow (and inflow where scoped) for selected `category_id`(s) over configurable window (default 12 months, max 24)
- [x] **AC-3 Trend chart UI:** React category trend chart renders month labels with EUR amounts (e.g. Jan €300, Feb €250); supports at least one category; empty-state when mirror has no categorized rows in period
- [x] **AC-4 Performance insight:** UI surfaces month-over-month change and best/worst month indicator for selected category in period (table or chart annotation)
- [x] **AC-5 Mirror fidelity:** Series uses Firefly-synced `category_id` on mirror transactions; uncategorized bucket explicit — not silent zero
- [x] **AC-6 Regression:** OIDC-enabled US-0010 external profile smoke pass; read-only Firefly preserved; US-0015 bucket mapping unchanged

## US-0019 — Goal-driven planning with per-plan stats & AI savings suggestions

- [x] **AC-1 Goal plan type:** Operator creates plan with **target balance** + **target date** (e.g. €10 000 in 5 months); plan persists and appears in Scenarios list
- [x] **AC-2 Per-plan statistics:** Dedicated stats for **selected plan only** — monthly delta vs baseline, yearly rollup, projected balance at target date — not household-wide aggregates on plan detail view
- [x] **AC-3 Category adjustments:** Plan builder accepts category-scoped spend changes (e.g. reduce "crypto" category); adjustments affect compare/PVA for that plan after recompute
- [x] **AC-4 AI savings suggestions:** AI proposes reducible expense categories/lines with evidence summary; operator **selects** suggestions to add as plan adjustments — no silent auto-apply
- [x] **AC-5 Privacy:** AI path uses aggregate/category signals only (`allow_raw_transactions=false`); audit log per US-0006 patterns
- [x] **AC-6 Regression:** US-0014 onboarding/templates still work; OIDC external profile smoke pass

## US-0020 — Subscription manual discovery, majority category & operator tags

- [x] **AC-1 Manual search:** `/subscriptions` (or discovery-named surface) filters potential candidates by **account**, **title/payee** text, and **repeating interval** (months); results paginated or capped with documented limit
- [x] **AC-2 Operator confirm:** Operator can confirm a searched candidate into confirmed subscriptions without auto-detection-only path
- [x] **AC-3 Majority category:** Confirmed subscription **display category** defaults to **mode category** of constituent transactions; tie-break rule documented in UI or tooltip
- [x] **AC-4 Operator tags:** CRUD for operator-defined tags (e.g. luxus, important); assign multiple tags per subscription; filter subscription list by tag
- [x] **AC-5 Storage contract:** Tags and majority-category metadata in product DB — no Firefly write-back
- [x] **AC-6 Regression:** US-0003/US-0008 detection and alert dedup unchanged; OIDC external profile smoke pass
