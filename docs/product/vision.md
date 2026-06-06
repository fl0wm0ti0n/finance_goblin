# Vision



## Problem



Personal finance tools like Finanzguru offer subscription detection, forecasting, and planning — but they are cloud-hosted and require uploading financial data to third parties. Firefly III provides excellent self-hosted transaction management, yet lacks native forecasting, subscription intelligence, scenario planning, AI assistance, and unified wealth views including crypto.



## Audience



Privacy-conscious individuals and households who already use (or want) Firefly III as their transaction ledger. They are comfortable with self-hosting (Docker), want Finanzguru-like capabilities without vendor lock-in, and need read-only integration that never mutates their canonical Firefly data.



## Value



**Flow Finance AI** extends Firefly III into a complete self-hosted financial operations system:



- Automatic subscription and standing-order detection with user confirmation

- Cashflow forecasting (daily through 24 months)

- Scenario-based financial planning with versioning and plan-vs-actual tracking

- Early-warning alerts (scarcity, budget drift, subscription changes, plan viability)

- Net-worth and portfolio analysis including crypto exchanges

- Privacy-safe AI assistant over structured tools (never direct database access)

- Grafana operational dashboards for cashflow, subscriptions, budgets, portfolio, and forecasts — **surfaced inside the financegnome web UI** (US-0011), not as a separate operator-only site for day-to-day use



Firefly III remains the **sole source of truth** for transactions; Flow Finance AI reads only and produces its own analytics, plans, and projections.



## Look and Feel



Modern, clean React UI built with Tailwind CSS and shadcn/ui components. Data-rich views use Apache ECharts. Grafana-backed SQL dashboards are reachable **inside the same financegnome shell** (embedded or proxied), with new charts added to that shell by default. Subscription confirmations use clear approve/reject cards. Planning views support side-by-side scenario comparison. OIDC-based authentication for secure access.



## UX References



### Finanzguru (feature parity target, not UI clone)



- **US-0001 foundation:** Dashboard-first landing with immediate trust signals — last sync time, connection status, and a read-only badge. Clean, uncluttered operator home; no feature-specific widgets yet.

- **US-0002 forecasting (parity target):** Account-centric cashflow projections with clear horizon pickers — daily (tomorrow, next week, month-end balance), monthly breakdown (income, fixed costs, variable costs, free cashflow), and long-term views (3 / 6 / 12 / 24 months). Charts answer "where will my balance be?" at a glance; stat cards for near-term milestones above the chart. Information-dense but scannable; proactive feel without cloning Finanzguru branding or layout.

- **BUG-0012 monthly bucket UX (discovery 2026-06-05):** On `/forecast` **Monthly** tab, operator expects four scannable stat cards — **Income**, **Fixed**, **Variable**, **Free cashflow** — reflecting the **first forecast month** (`series[0]`) plus a stacked **MonthlyChart** for the full horizon. When fixed, salary/income-category inflows populate **Income** (not `0.00`), rent/utilities/subscription-class outflows populate **Fixed** (not `0.00`), and discretionary spend stays in **Variable**; **Free cashflow** = income − fixed − variable. No new UI chrome required — cards bind directly to `GET /api/v1/forecast/monthly` bucket fields. Parity target remains Finanzguru-style monthly cashflow decomposition without branding clone. AI-assisted bucket inference deferred to **US-0015**; config/DEC-0007 baseline path must work first.

- **US-0003 subscriptions (parity target):** Proactive subscription intelligence — pending detections surfaced as review cards (payee, interval, amount, confidence tier); confirmed list with monthly cost rollup; price-change callouts ("was X, now Y") without cloning Finanzguru branding.

- **US-0004 planning (parity target):** Scenario comparison for life decisions — leasing (+€/month), savings mode (cut subscriptions/spending), house purchase (raise savings rate); side-by-side version compare (v1/v2/v3); daily plan-vs-Ist with planned / actual / deviation; proactive "what if I change X?" density without cloning Finanzguru branding.

- **US-0005 wealth & alerts (parity target):** Aggregated net worth with account breakdown; proactive scarcity, budget-drift, and plan-viability warnings; header notification bell + alert inbox with acknowledge/dismiss — Finanzguru-style early warning without UI clone.

- **US-0006 AI assistant (parity target):** Conversational natural-language Q&A over finances — affordability ("Kann ich mir ein Leasing Auto leisten?"), subscription price changes, budget overrun explanations, savings from cancelling a subscription, top spending categories; proactive **suggested prompt chips** on empty chat; answers grounded in tool results, not raw DB access; privacy-safe feel without cloning Finanzguru branding.

- **US-0007 crypto portfolio (parity target):** Complete net worth including exchange balances; per-exchange holdings and PnL (realized/unrealized/total return); allocation vs target (50/50 ETF/crypto style); proactive "portfolio drift" feel without UI clone.

- **US-0008 local AI (parity target):** Privacy-first conversational Q&A with **zero cloud dependency option** — operator selects OpenAI or a local/self-hosted endpoint; chat and tool audit behave identically; "Local" vs "Cloud" trust signal in chat header without cloning Finanzguru branding.

- **Deferred to later stories:** additional exchanges (Kraken, Coinbase, Bitpanda, OKX); on-chain wallets; ML-enhanced forecasts (US-0009).

- **Parity principle:** Match Finanzguru's information density and proactive feel over time; never clone branding, color palette, or proprietary layouts.



### Firefly III (data model and account structure alignment)



- **Entity vocabulary:** Use Firefly-native terms in UI labels — accounts (asset/expense/revenue/liability), categories, budgets, tags, piggy banks — without renaming or re-hierarchizing synced data.

- **Sync Status page:** Group synced entities by Firefly type with counts, last-updated timestamps, and per-entity sync health (success/warning/error). Mirror Firefly's account-type taxonomy in table columns and filters.

- **Read-only contract:** Persistent "Read-only · Firefly source" indicator in app header; Settings page explains that Flow never writes to Firefly.

- **Navigation alignment:** Sidebar sections map to Firefly concepts (Accounts, Budgets, Categories) as read-only data views; analytics features (Forecast, Subscriptions, Planning) ship as enabled routes per story (US-0002 `/forecast`, US-0003 `/subscriptions`, US-0004 `/planning`); Wealth and AI remain placeholders until their stories ship.

- **Not a Firefly UI clone:** Flow uses shadcn/React patterns, not Firefly's Twig/Alpine layout; alignment is semantic (data model), not visual.



### shadcn/ui dashboard patterns (primary navigation shell)



- **Layout:** `SidebarProvider` + collapsible icon sidebar (`collapsible="icon"`) with `SidebarInset` content area; sidebar persists across routes.

- **Config-driven nav:** Navigation items defined in a config array; map to `SidebarMenu` / `SidebarMenuButton` with `isActive` from router pathname. **US-0002:** Forecast nav item enabled at `/forecast`. **US-0003:** Subscriptions nav item enabled at `/subscriptions`. **US-0004:** Planning nav item enabled at `/planning`. **US-0005:** Wealth nav item enabled at `/wealth`. **US-0006:** AI nav item enabled at `/chat`; header adds **AI assistant** icon button opening the same chat in a `Sheet` drawer for quick Q&A. Header notification bell (unread alert count) links to `/alerts`.

- **Header bar:** `SidebarTrigger`, breadcrumb trail, sync-status pill (green/amber/red), OIDC user menu in `SidebarFooter`.

- **US-0001 pages:**

  - **Home / Dashboard** — placeholder welcome card, sync summary stats, quick links to Sync Status and Settings.

  - **Sync Status** — entity count cards, last sync timestamp, manual "Sync now" trigger, sync history log (table with status badges).

  - **Settings** — Firefly connection (URL, auth method, test connection), database mode indicator (external), sync interval config (read-only display in UI shell; TOML/env is source of truth).

- **Components:** Card, Table, Badge, Button, Alert for status messaging; TanStack Query for API polling and sync state.

- **Responsive:** Desktop fixed sidebar; mobile drawer via same nav config (no duplicated route logic).

- **US-0002 Forecast page:**

  - **Account selector:** Dropdown of synced Firefly asset accounts (default: primary checking / first asset account).

  - **Horizon control:** Segmented tabs — **Daily** | **Monthly** | **Long-term** — with secondary horizon pills where needed (3 / 6 / 12 / 24 months on Long-term).

  - **Daily view:** Stat cards for tomorrow, next week, and month-end projected balance; line chart of projected balance over the current month.

  - **Monthly view:** Grouped or stacked bar chart for income, fixed costs, variable costs, and free cashflow; summary stat row for net free cashflow.

  - **Long-term view:** Area or line chart of projected balance across selected horizon; optional confidence band deferred to US-0009.

  - **Trust signals:** "Last computed" timestamp, link to Sync Status, empty-state when no synced transactions.

  - **Components:** shadcn Card, Select, Tabs or ToggleGroup; TanStack Query for forecast API; Apache ECharts for chart canvas.

- **US-0004 Planning page:**

  - **Active plan selector:** Dropdown of user plans; badge for active plan used in plan-vs-Ist and Grafana Dashboard 3.

  - **Tabs:** **Scenarios** | **Compare** | **Plan vs Actual**.

  - **Scenarios tab:** Template cards (Current/Ist, Leasing, Savings mode, House purchase); adjustment table (amount, frequency, target); **New plan** / **New version** actions.

  - **Compare tab:** Side-by-side **v1 / v2 / v3** stat cards (monthly delta, projected month-end balance); optional ECharts grouped bar.

  - **Plan vs Actual tab:** Daily table (planned, actual, deviation) defaulting to current month; optional ECharts dual-line chart.

  - **Trust signals:** Last sync link; empty states for no plans or no transaction history.

  - **Components:** shadcn Card, Table, Badge, Button, Tabs, Select, Dialog; TanStack Query for plan API.



### Apache ECharts forecast chart patterns (US-0002)



- **Daily balance line:** Time axis with projected balance curve; markers at tomorrow, +7 days, and month-end (`markPoint` or annotation).

- **Monthly breakdown:** Category axis with four series (income, fixed, variable, free cashflow); color palette distinct from Grafana ops theme.

- **Long-term forecast:** Smooth line or area series over months; horizon switch re-fetches without full page reload.

- **Responsive:** `resize` listener on chart container; legend bottom on mobile.

- **Accessibility:** Chart title + summary stat cards carry primary numeric values; chart is supplementary visualization.



### Grafana (Platform Health US-0001; Cashflow & Forecast US-0002)



- **US-0001 (shipped):** Grafana service in Compose minimal profile with PostgreSQL/TimescaleDB datasource; **Platform Health** dashboard (sync/API metrics).

- **US-0002 — Dashboard 1 (Cashflow):** Panels for account balance time series, forecast overlay line, and scarcity threshold markers (visual reference lines only — full Alert Engine in US-0005). Variable: account selector tied to synced Firefly accounts.

- **US-0002 — Dashboard 5 (Forecast horizons):** Row or repeat panels for forecast horizons — **1 / 3 / 6 / 12 months** per Projectplan; optional **24-month** panel aligned with React long-term selector. Stat + time-series combo per horizon.

- **Provisioning:** Dashboard JSON in `provisioning/dashboards/` volume mount; datasource reuse from US-0001; dark theme default.

- **US-0003 — Dashboard 2 (Subscriptions):** Panels for all confirmed subscriptions (count + monthly spend stat), price-change events (time series or event table), and newly detected pending/confirmed counts. Variable: account or global scope tied to synced Firefly expense flows. uid pattern follows DEC-0012 (`subscriptions`).

- **US-0004 — Dashboard 3 (Budgets):** Panels for **Plan**, **Ist**, and **Abweichung** (deviation) tied to active plan; time series or stat+table combo for daily/monthly grain; variable or filter for active plan id. uid `budgets` per DEC-0012 pattern.

- **US-0005 — Dashboard 4 (Portfolio partial):** Total wealth stat (non-crypto Firefly asset accounts), account breakdown table or pie, optional wealth-over-time series; uid `portfolio`. Crypto and performance panels deferred to US-0007.

- **Visual style:** Dark theme; ops-grade time-series and stat panels; consistent with Grafana conventions, not Finanzguru consumer styling.



### OIDC authentication shell



- **Login flow:** Unauthenticated users redirect to OIDC provider; post-login return to intended route or Home.

- **Session UX:** User avatar/name in sidebar footer; logout action clears session and redirects to provider logout when supported.

- **Protected routes:** All app pages except login callback require valid session; API skeleton returns 401 for unauthenticated requests.

- **Settings visibility:** Display configured OIDC issuer and client ID (masked secret) for operator verification; no in-app OIDC provider configuration in US-0001 (Compose/env config).



## Discovery notes (US-0001, 2026-05-31)



Discovery phase captured foundation UX references only. Feature-specific UI (forecasts, subscriptions, planning, AI) remains specified at vision level but implementation-bound to US-0002 through US-0009.



## Discovery notes (US-0002, 2026-05-31)



Discovery captured forecast UX references for React ECharts views and Grafana Dashboards 1 & 5. Forecast nav placeholder from US-0001 becomes live `/forecast` route. Algorithm details (recurring-pattern inference vs rolling averages) deferred to `/research` and `/architecture`. Subscription-driven forecast adjustments remain US-0003; ML enhancements US-0009.



## Discovery notes (US-0003, 2026-05-31)



Discovery captured subscription intelligence UX for React confirm/reject workflow, standing-order separation, price-change surfacing, subscription-scoped alerts, and Grafana Dashboard 2. Subscriptions nav placeholder from US-0001 becomes live `/subscriptions` route. Detection algorithm thresholds and persistence schema deferred to `/research` and `/architecture`. Full Alert Engine inbox (US-0005); plan scenario impact delivered in US-0004.



### Finanzguru (subscription parity target, not UI clone)



- **Pending review cards:** One card per detected pattern — payee name, interval label (weekly/monthly/quarterly), latest amount, confidence badge (95% / 80% / 60%). Primary actions **Confirm** and **Reject** (Projectplan Bestätigen/Ablehnen pattern).

- **Confirmed subscription list:** Table or card grid sorted by monthly cost impact; columns: payee, interval, current amount, last charge date, type badge (Subscription vs Dauerauftrag).

- **Price-change surfacing:** Inline delta on list row and detail view — prior amount → new amount with direction icon (increase/decrease); frequency-change label when cadence shifts.

- **Proactive feel:** Banner or toast on login when new pending detections exist; subscription-scoped alerts (new abo, price change) distinct from US-0005 scarcity/budget inbox.



### Firefly III (transaction grounding)



- **Payee/description display:** Surface normalized Firefly counterparty and description text from synced transactions; link "View transactions" drill-down filtered to pattern payee.

- **No Firefly mutation:** Confirm/reject state stored in Flow DB only; detection reads mirror `transactions` table (US-0001).

- **Standing orders (Dauerauftrag):** Classify high-regularity fixed outflows (rent, insurance, utilities) separately from discretionary subscriptions — separate tab or filter, not a second product surface.



### shadcn/ui subscriptions page patterns (US-0003)



- **Route:** `/subscriptions` replaces disabled nav placeholder.

- **Layout:** Page header with monthly subscription spend stat card; **Tabs** — **All** | **Pending review** | **Standing orders** (Dauerauftrag).

- **Pending review:** Stack of `Card` components with `Badge` confidence tier (95=default/success tone, 80=secondary, 60=outline/warning); `Button` pair Confirm (primary) / Reject (outline).

- **Confirmed list:** `Table` with sortable amount and interval; row click opens `Sheet` or `Dialog` detail with price history.

- **Detail drawer:** Current amount, interval, confidence at confirmation time, linked transaction list (`Table` compact), price history mini-chart (ECharts).

- **Empty states:** "No pending subscriptions" when detection queue clear; CTA to Sync Status when no transaction history.

- **Components:** Card, Table, Badge, Button, Tabs, Sheet/Dialog, Alert for subscription-scoped notifications; TanStack Query for detection API polling after sync.



### Apache ECharts subscription chart patterns (US-0003)



- **Price history (detail drawer):** Category or time axis with amount points per billing cycle; `markLine` or annotation for detected price jumps.

- **Overview optional:** Small bar chart of top-N subscriptions by monthly cost on main page (secondary to table; defer if sprint tight).

- **Accessibility:** List/table carries primary amounts; chart supplementary.



### Grafana Dashboard 2 (Subscriptions US-0003)



- **All subscriptions panel:** Stat total monthly spend + table/panel of confirmed subscription count by interval.

- **Price changes panel:** Time series or event list of amount deltas for confirmed subscriptions since last sync.

- **New subscriptions panel:** Count of pending + recently confirmed detections (rolling window).

- **Provisioning:** Dashboard JSON in `provisioning/dashboards/`; datasource uid `FlowFinancePostgreSQL` (DEC-0012); dark theme default.

- **Out of scope:** Dashboard 4 (US-0005/US-0007).



### Finanzguru (planning parity target, not UI clone) — US-0004



- **Scenario templates:** Quick-start cards for **Current (Ist)**, **Leasing** (+€/month recurring), **Savings mode** (remove subscriptions / cut discretionary spend), **House purchase** (increase savings rate) — Projectplan life-decision framing.

- **Custom plan:** Named plan with explicit adjustment lines (amount, frequency, target) beyond templates.

- **Version compare:** Side-by-side **v1 / v2 / v3** columns with monthly impact summary and projected month-end balance per version.

- **Plan vs actual:** Daily **Geplant / Ist / Abweichung** (planned, actual, deviation); deviation direction visually obvious.

- **Proactive feel:** Active plan badge; deviation callout when month drifts (visual only until US-0005 Alert Engine).



### Firefly III (actuals & budget grounding) — US-0004



- **Actuals (Ist):** Daily amounts aggregated from synced Firefly transactions (read-only).

- **Budget vocabulary:** Plan-vs-Ist labels use Firefly-native budget/category terms where mirror data exists.

- **Baseline alignment:** **Current (Ist)** mirrors latest US-0002 forecast baseline; savings-mode picks from US-0003 confirmed subscriptions.

- **No Firefly mutation:** Plans, versions, and deltas stored in Flow DB only.



### Apache ECharts planning chart patterns (US-0004)



- **Plan vs actual (daily):** Dual line or grouped bar (planned vs actual); month-to-date deviation stat cards.

- **Version compare:** Grouped bar per version (v1/v2/v3) for monthly planned cashflow or delta sum.

- **Deviation trend:** Daily deviation line with zero reference `markLine`.

- **Accessibility:** Table carries primary numbers; charts supplementary.



### Grafana Dashboard 3 (Budgets US-0004)



- **Plan panel:** Planned series from active plan snapshot (daily or monthly grain).

- **Ist panel:** Actual aggregated from Firefly mirror for matching period.

- **Abweichung panel:** Deviation (plan − ist) time series or table.

- **Provisioning:** JSON in `grafana/provisioning/dashboards/analytics/`; uid `budgets`; datasource uid `FlowFinancePostgreSQL` (DEC-0012).

- **Out of scope:** Plan viability alert rules (US-0005); Dashboard 4 portfolio (US-0005/US-0007).



## Discovery notes (US-0004, 2026-05-31)



Discovery captured financial planning UX for scenario templates, plan versioning, daily plan-vs-Ist comparison, React `/planning` page, and Grafana Dashboard 3. Planning nav placeholder from US-0001 becomes live `/planning` route. Plan Engine delta model, persistence schema, and Ist aggregation rules deferred to `/research` and `/architecture`. Plan viability alerts (US-0005), AI `simulate_plan` chat (US-0006), and crypto allocation scenarios (US-0007) remain out of scope.



### Finanzguru (wealth & alerts parity target, not UI clone) — US-0005



- **Net worth overview:** Single headline **Gesamtvermögen** (total wealth) with breakdown by account type — giro, savings, Tagesgeld — matching Projectplan Vermögensanalyse; crypto row shown as "not connected" until US-0007.

- **Proactive warnings:** Early scarcity ("Kontostand fällt unter 200 €"), budget drift ("Lebensmittel +20 %"), plan viability ("Leasing Plan nicht tragfähig") — distinct from US-0003 subscription price-change alerts.

- **Alert inbox:** Unified list with severity/type badge, short message, timestamp; **Acknowledge** and **Dismiss** actions; header bell unread count for at-a-glance status.

- **Proactive feel:** Bell badge on login when unread alerts exist; wealth page surfaces net-worth trend and active alert summary strip.



### Firefly III (account grounding for net worth) — US-0005



- **Asset accounts only:** Net worth sums synced Firefly **asset** account balances (checking, savings, cash); liabilities excluded from headline total (optional liability stat deferred).

- **Account vocabulary:** Table columns use Firefly account name and native type/subtype labels from mirror; no reclassification beyond asset/expense grouping for display.

- **Read-only contract:** Balances from latest sync mirror; "Last synced" link to Sync Status; no Firefly mutation.

- **Budget drift grounding:** Category actuals from mirrored transactions vs active-plan category-targeted adjustments (US-0004); Firefly budget entities as secondary reference only.



### shadcn/ui wealth & alerts page patterns (US-0005)



- **Routes:** `/wealth` replaces disabled Wealth nav placeholder; **`/alerts`** dedicated inbox (reachable from header bell).

- **Header bell:** Icon button with `Badge` unread count; `Popover` or `DropdownMenu` preview of latest alerts + "View all" link.

- **Wealth page — Overview tab:** Stat card total net worth; `Table` per-account breakdown (name, type, balance, currency); optional `Alert` strip when active scarcity/plan-viability alerts exist.

- **Alerts page:** `Table` or stacked `Card` list filtered **Active | Acknowledged | Dismissed**; row actions Acknowledge / Dismiss; type `Badge` (Scarcity, Budget drift, Plan viability).

- **Empty states:** "No active alerts" on inbox; "Connect crypto in US-0007" placeholder row on wealth breakdown.

- **Components:** Card, Table, Badge, Button, Tabs, Popover/DropdownMenu, Alert; TanStack Query for alerts and net-worth API polling after sync.



### Apache ECharts wealth chart patterns (US-0005)



- **Account breakdown (Overview):** Horizontal bar or pie by account balance share; distinct palette from forecast charts.

- **Wealth over time (optional MVP):** Line series of total asset sum if balance snapshots available post-sync; defer if sprint tight.

- **Accessibility:** Table carries primary balances; chart supplementary.



### Grafana Dashboard 4 (Portfolio partial US-0005)



- **Total wealth panel:** Stat panel summing non-crypto Firefly asset balances (latest sync).

- **Account breakdown panel:** Table or pie by account type/name.

- **Wealth over time panel (optional):** Time series if snapshot history stored; defer if sprint tight.

- **Provisioning:** JSON in `grafana/provisioning/dashboards/analytics/`; uid `portfolio`; datasource uid `FlowFinancePostgreSQL` (DEC-0012).

- **Out of scope:** Crypto slice and portfolio performance panels (US-0007); Grafana Alertmanager rules (React Alert Engine owns firing).



## Discovery notes (US-0005, 2026-05-31)



Discovery captured wealth analysis UX (net worth aggregation, account breakdown), unified Alert Engine inbox (scarcity, budget drift, plan viability), header notification bell, React `/wealth` and `/alerts` routes, and Grafana Dashboard 4 partial (non-crypto total wealth). Wealth nav placeholder from US-0001 becomes live `/wealth` route. Alert evaluation rules, persistence schema, Dashboard 1 threshold centralization, and budget-drift metric grain deferred to `/research` and `/architecture`. Crypto portfolio slice (US-0007), AI tool wiring (US-0006), and subscription alert migration remain out of scope.



### Finanzguru (AI assistant parity target, not UI clone) — US-0006



- **Conversational Q&A:** Natural-language questions about affordability, subscriptions, budgets, savings scenarios, and spending categories — Projectplan example queries as acceptance anchors.

- **Grounded answers:** Responses cite structured tool outputs (forecast paths, subscription lists, plan deltas) — not free-form database narration; user sees a concise answer with optional "Sources" expansion.

- **Proactive starters:** Empty chat shows **suggested prompt chips** mapped to example queries (affordability, price changes, budget overrun, cancel savings, top categories).

- **Privacy-first tone:** Persistent indicator when privacy redaction is active ("Aggregated data only" or similar badge when `allow_raw_transactions=false`).

- **Proactive feel:** Quick access from header AI button without leaving current page; full `/chat` route for longer sessions.



### Firefly III (tool grounding, read-only) — US-0006



- **Tool-only data access:** AI path invokes registered tools that wrap existing Flow APIs and services — transactions, subscriptions, forecast, budget status, portfolio, plan simulation — never PostgreSQL or Firefly directly.

- **No Firefly mutation:** `simulate_plan` and all tools are read-only; plan changes remain on `/planning` only.

- **Semantic labels:** Tool summaries use Firefly-native vocabulary (accounts, categories, payees) where mirror data is surfaced.

- **Privacy redaction:** IBAN and counterparty fields stripped or hashed per TOML before tool JSON reaches the model; raw transaction rows omitted when `allow_raw_transactions=false`.



### shadcn/ui AI chat patterns (US-0006)



- **Routes:** `/chat` replaces disabled AI nav placeholder; shared **`ChatPanel`** component used by full page and header drawer.

- **Header trigger:** Icon button (sparkles or message-square) beside notification bell; opens **`Sheet`** side drawer (~400px desktop) with compact chat; drawer preserves underlying page context.

- **Full page `/chat`:** Same `ChatPanel` with wider layout, message history scroll region, input bar fixed bottom; breadcrumb **AI Assistant**.

- **Message layout:** User bubbles right-aligned; assistant bubbles left-aligned with markdown-lite rendering (bold, lists, tables for category breakdowns).

- **Tool transparency (MVP):** Collapsible **"Tools used"** row under assistant messages listing tool names + timestamp (no raw JSON dump); defer fancy step-by-step UI if sprint tight.

- **Suggested prompts:** Horizontal scroll or wrapped `Button` variant `outline` chips above input on empty thread.

- **Privacy badge:** `Badge` in chat header reflecting active privacy mode (e.g. "Redacted · no raw transactions").

- **Settings extension:** Settings page adds **AI & Privacy** section — display TOML-sourced `allow_raw_transactions`, `redact_iban`, `redact_counterparties` (editable vs read-only deferred to architecture); OpenAI model/base URL display (masked API key).

- **Operator audit:** Settings or Sync Status adjacent **Tool audit log** table — recent invocations with tool name, session/user, redacted arg summary, duration, success/error (operator-only or same OIDC role as Settings).

- **Components:** Sheet, ScrollArea, Button, Badge, Textarea, Card, Collapsible, Table (audit); TanStack Query for chat API + SSE/stream subscription.

- **Loading states:** Typing indicator during model response; disable send while in-flight.



### Apache ECharts in AI context (US-0006)



- **Optional inline charts:** When assistant answer includes time series (forecast path, category spend), render small embedded ECharts sparkline or bar in assistant bubble — secondary to text; defer if sprint tight.

- **Accessibility:** Numeric summary in prose carries primary values; charts supplementary.



### Grafana (US-0006)



- **Out of scope:** No new Grafana dashboard for AI in US-0006; operator observability via in-app tool audit log and backend structured logs.

- **Existing dashboards unchanged:** AI tools read same forecast/subscription/wealth/plan data surfaces as Dashboards 1–4.



## Discovery notes (US-0006, 2026-05-31)



Discovery captured AI assistant UX for privacy-safe conversational Q&A: header **Sheet** drawer + full-page **`/chat`**, six registered tools via OpenAI function calling, Privacy Layer per Projectplan TOML defaults, example-query starter chips, tool transparency row, Settings AI & Privacy section, and operator tool audit log. AI nav placeholder from US-0001 becomes live `/chat` route. Tool registry contract, OpenAI streaming/SSE, audit persistence schema, redaction middleware placement, and `simulate_plan` API mapping deferred to `/research` and `/architecture`. Local/self-hosted providers (US-0008), ML forecasts (US-0009), and Grafana AI panels remain out of scope.



### Finanzguru (crypto portfolio parity target, not UI clone) — US-0007



- **Complete net worth:** Headline **Gesamtvermögen** includes Firefly asset accounts **and** connected exchange crypto balances — replaces US-0005 "not connected" placeholder.

- **Exchange holdings:** Per-exchange breakdown (Binance, Bybit, Bitunix); asset-level table with quantity, market value, unrealized PnL; total crypto slice as % of household wealth.

- **Performance visibility:** Realized gains (closed positions), unrealized gains (open positions), total return since tracking baseline — Projectplan Portfolio Engine outputs surfaced as stat cards.

- **Allocation planning:** Target mix examples (50% ETF / 50% crypto; 70% ETF / 20% crypto / 10% cash); current vs target gap callout — proactive rebalancing hint without trade execution.

- **Proactive feel:** Sync Status shows exchange connection health; wealth Crypto tab highlights largest holdings and PnL direction; no Finanzguru branding clone.



### Exchange connector UX patterns (US-0007)



- **Read-only API keys:** Operator configures exchange credentials in Settings; keys never leave self-hosted backend; test-connection validates before save.

- **Supported start set:** Binance, Bybit, Bitunix (Projectplan Phase 5); additional exchanges deferred.

- **Imported data surfaces:** Balances, positions, trades, transfers, funding, PnL — mapped to unified holdings view; exchange-native labels preserved in detail drill-down.

- **Sync parity with Firefly:** Manual sync trigger on Sync Status; per-exchange last-sync timestamp and entity counts; error badges on connection failure.



### Firefly III (wealth boundary, read-only) — US-0007



- **Dual source model:** Firefly remains **sole transaction ledger** (DEC-0004); exchange data ingested into Flow DB only — no writes to Firefly or exchanges.

- **Combined net worth:** Firefly asset accounts + crypto exchange totals in single headline; subtotals labeled by source (Firefly / Crypto).

- **Planning actuals unchanged:** Plan-vs-Ist still aggregates Firefly transactions; crypto allocation compares wealth breakdown, not Firefly categories.

- **No Firefly crypto accounts required:** Exchange balances do not need Firefly mirror accounts for MVP.



### shadcn/ui wealth & settings patterns (US-0007)



- **Routes:** Extend existing **`/wealth`** (no new nav item); add **Tabs** — **Overview** | **Crypto**.

- **Overview tab (updated):** Combined net worth stat; Firefly subtotal + Crypto subtotal stat cards; account breakdown table (Firefly rows only); optional combined allocation summary bar.

- **Crypto tab:** Per-exchange `Card` grid (connection status, total EUR, last sync); holdings `Table` (exchange, asset, quantity, value EUR, unrealized PnL, 24h change if available); PnL stat row — Realized | Unrealized | Total return.

- **Empty state:** "No exchanges connected" with `Button` link to Settings **Crypto exchanges** section.

- **Settings extension:** **Crypto exchanges** section below Firefly connection — one card per supported exchange (enable toggle, API key/secret masked fields, test connection); read-only sync interval display (TOML source of truth).

- **Sync Status extension:** Additional table section **Exchange sync** — exchange name, status badge, balances/positions/trades counts, last sync, manual sync per exchange or combined "Sync exchanges now".

- **Components:** Card, Table, Badge, Button, Tabs, Alert (connection errors), Dialog for add-exchange flow; TanStack Query for portfolio API polling after sync.



### Apache ECharts crypto chart patterns (US-0007)



- **Allocation pie (Overview or Crypto tab):** Firefly asset classes vs crypto by top-N assets; distinct palette from forecast charts.

- **Portfolio performance line:** Total return or combined net worth over time including crypto slice (from extended snapshots).

- **Holdings bar (optional):** Top holdings by value horizontal bar on Crypto tab; defer if sprint tight.

- **Accessibility:** Table and stat cards carry primary values; charts supplementary.



### Grafana Dashboard 4 (Portfolio completion US-0007)



- **Crypto value panel:** Stat summing latest crypto holdings converted to reporting currency (replaces placeholder text panel).

- **Allocation panel:** Pie or bar — Firefly assets vs crypto total; optional top-5 crypto assets.

- **Portfolio performance panel:** Time series of total return or combined wealth including crypto from extended `net_worth_snapshots`.

- **Account breakdown panel:** Extend latest snapshot payload to include crypto rows alongside Firefly accounts.

- **Provisioning:** Update `portfolio.json` in `grafana/provisioning/dashboards/analytics/`; uid `portfolio` unchanged (DEC-0012).

- **Out of scope:** Per-exchange Grafana variables (defer); Alertmanager rules; tax-lot reporting.



### Planning allocation scenarios (US-0007)



- **Allocation target template:** New built-in scenario on **`/planning`** Scenarios tab — user defines target weights across **ETF/traditional** (Firefly assets), **Crypto**, **Cash** buckets (Projectplan examples: 50/50, 70/20/10).

- **Current vs target:** Read-only comparison card showing current allocation from wealth API vs target weights; gap % per bucket; link from `/wealth` Overview optional.

- **No trade execution:** Scenario is planning-only; no exchange order placement.

- **Plan versioning:** Allocation targets stored as named plan adjustments or dedicated allocation config — persistence detail deferred to `/architecture`.



## Discovery notes (US-0007, 2026-06-01)



Discovery captured crypto exchange portfolio UX: three read-only exchange connectors (Binance, Bybit, Bitunix), Portfolio Engine PnL surfaces, **`/wealth` Crypto tab**, Settings **Crypto exchanges** credentials, Sync Status exchange rows, allocation target planning template, and Grafana Dashboard 4 completion (crypto value + allocation + performance panels). US-0005 crypto placeholder replaced by live data in net worth headline. Exchange API auth, FX conversion, PnL methodology, spot vs futures scope, secret storage, and post-sync pipeline ordering deferred to `/research` and `/architecture`. Additional exchanges, on-chain wallets, tax reporting, and trade execution remain out of scope.



### Self-hosted AI runtimes (US-0008 reference, not product clone)



- **Ollama:** Default Compose `full` profile service on port 11434; OpenAI-compatible `/v1/chat/completions` with tool calling; operator pulls models via CLI (`ollama pull`) — no in-app model catalog MVP.

- **LM Studio:** Desktop operator pattern — local OpenAI-compatible server on configurable host/port; maps to generic `openai_compatible` provider + `base_url`.

- **LocalAI / vLLM:** Same OpenAI-compatible contract — single configurable base URL; optional bearer token when gateway requires auth.

- **Privacy positioning:** Financial Q&A stays on operator infrastructure; outbound OpenAI calls optional fallback when `provider = "openai"` and key configured.



### Provider selection UX patterns (US-0008)



- **Settings extension:** Extend existing **AI & Privacy** section (US-0006) — read-only provider table: active provider kind (`openai` | `ollama` | `openai_compatible`), model name, base URL (local only), API key env name (OpenAI only), max tool rounds; **Provider status** badge (configured / not configured / unreachable) mirroring exchange connection badges.

- **Test connection:** **Test AI provider** button (POST test endpoint) — latency + model reachability; no chat message sent; pattern mirrors exchange test-connection (R-0035).

- **Config source of truth:** TOML `[ai]` + env vars; edit `config.toml` and restart — runtime provider switching deferred (same as US-0006 privacy toggles).

- **Chat header badges:** Existing privacy badge unchanged; add **Provider badge** — e.g. `Local · Ollama` (green) vs `Cloud · OpenAI` (neutral) so operator always knows data path.

- **Empty / misconfigured state:** When selected provider not configured or test fails, chat input disabled with Alert linking to Settings — actionable copy ("Configure `[ai]` provider in config.toml and restart").

- **No secrets in UI:** Mask API key env name only; never render key values; local providers show base URL without tokens.



### US-0006 chat continuity (US-0008)



- **Shared ChatPanel:** No duplicate chat implementations — provider change is backend-only; SSE stream, suggested prompts, tool transparency row, and audit log unchanged.

- **Tool layer frozen:** Six registered tools and PrivacyLayer middleware untouched — provider swap at HTTP client layer only (R-0027 stub trait extension).

- **Audit log:** Tool audit rows include `provider` column or metadata field for operator traceability (which backend answered).



### Docker Compose full profile (US-0008)



- **Ollama service:** Existing `ollama` service under `profiles: [full]` — discovery adds backend `depends_on: ollama` when `[ai] provider = "ollama"` documented in operator guide; default internal URL `http://ollama:11434/v1`.

- **Out of scope:** In-compose GPU device passthrough beyond default Compose; model pull automation; sidecar for LM Studio (host-run, not containerized).



### shadcn/ui Settings patterns (US-0008)



- **Provider card:** Below OpenAI status row — table rows for Provider, Model, Base URL, Status badge, Test connection button.

- **Components:** Reuse Badge, Button, Alert, Table from Settings exchange section; TanStack Query mutation for test endpoint.



## Discovery notes (US-0008, 2026-06-02)



Discovery captured local & self-hosted AI provider UX: three provider modes (`openai`, `ollama`, `openai_compatible`) over unchanged US-0006 tool layer, Settings **AI & Privacy** provider display + test connection, chat **Local vs Cloud** provider badge, Compose `full` profile Ollama wiring, and E2E verification that local selection avoids external API calls. Provider factory design, OpenAI-compatible endpoint variance, tool-calling model requirements, streaming delta compatibility, config schema, and compose dependency graph deferred to `/research` and `/architecture`. Model fine-tuning, GPU orchestration beyond Compose profiles, in-app model management, and runtime config editing remain out of scope.



## Discovery notes (US-0009, 2026-06-01)



Discovery captured Phase 7 **Advanced Forecasting** UX: ML-enhanced long-term projections with confidence bands, seasonal pattern surfacing, baseline vs ML comparison on `/forecast`, portfolio performance outlook when US-0007 data exists, plan-scenario **risk score** on `/planning`, and Grafana Dashboard 5 extensions — all over the released US-0002 rule-based baseline (`DEC-0007`) without replacing it. Statistical/ML execution model, schema for ML snapshots, minimum history gates, and risk-score weighting deferred to `/research` and `/architecture`. External cloud ML APIs, training pipelines, trading signals, and tax optimization remain out of scope.



### Long-term forecast & confidence bands (US-0009)



- **Horizon:** Reuse US-0002 long-term pills **6 / 12 / 24 months**; ML series extends to 24 months where mirror history allows.

- **Chart:** ECharts **area band** — central ML line + shaded **p10–p90** interval (amber/sage palette per vision chart conventions); baseline rule-based line overlaid when Compare mode active (berry secondary color).

- **Compare control:** Segmented **Baseline | ML-enhanced | Compare** on Long-term tab; Compare shows both series + end-horizon delta stat card.

- **Trust signals:** Model family badge (e.g. `AutoETS`, `MSTL`), seasonal periods detected, backtest WMAPE or holdout metric, `low_confidence` when history sparse; link to Sync Status.



### Seasonal patterns (US-0009)



- **Monthly view enhancement:** Optional callout when seasonality detected — e.g. "Higher spending typically in Nov–Dec" with strength indicator (weak/moderate/strong).

- **Not in scope:** Replacing US-0003 subscription detection; seasonality applies to aggregate cashflow layers only.



### Portfolio performance forecast (US-0009)



- **Placement:** **`/wealth` Crypto tab** section "Portfolio outlook" when ≥1 exchange connected; stat cards for projected value at 3 / 6 / 12 months.

- **Data source:** US-0007 `portfolio_snapshots` EUR series; empty state when no exchange data.

- **Trust:** Same last-computed timestamp as forecast; FX incomplete warning carries forward (R-0034).



### Plan scenario risk assessment (US-0009)



- **Placement:** Planning **Scenarios** tab — risk score badge on active plan card; **Compare** tab adds risk column per plan version.

- **Score UX:** 0–100 with color bands (low / medium / high); tooltip explains drivers (projected deficit months, balance below scarcity threshold, crypto volatility proxy when applicable).

- **Relationship to US-0005:** Complements `plan_viability` alerts (R-0022) with continuous score — does not replace Alert Engine inbox.



### Grafana Dashboard 5 extensions (US-0009)



- **Extend** existing `forecast-horizons` uid — new row: ML vs baseline balance series, confidence band panel, seasonal-detected stat, optional portfolio forecast stat when snapshots exist.

- **Variables:** Reuse `$account_id`; add `$forecast_variant` (`baseline` | `ml`) for panel queries.

- **Provisioning:** Same R-0008 bind-mount pattern; no new dashboard uid.



### Reference products (UX inspiration, not clones)



- **Finanzguru / consumer apps:** Long-horizon balance outlook with implicit uncertainty — we expose bands explicitly for self-hosted trust.

- **FP&A / Runway-style planning:** Scenario risk framing — risk score on active plan, not enterprise workflow clone.

- **Layered forecasting (R-0043):** Rule baseline + statistical overlay pattern aligns with DEC-0007 preservation.



## Discovery notes (BUG-0009, 2026-06-06)



Grafana analytics on `financegnome.omniflow.cc` are **not blocked by datasource failure** post-BUG-0004 — SQL executes and returns data. Operator-perceived emptiness is primarily **account-variable UX** plus a **missing/broken cross-account overview**.



### Defect Y — panels appear empty



- **Primary cause:** `$account_id` dashboard variable defaults to **first alphabetically** (116 Cash wallet) which has **zero** forecast series; funded Giro **114** has non-empty data when selected manually.

- **Secondary cause:** Forecast-horizons **ML panels** query `ml_enhanced` computations — none on US-0010 external profile until **US-0013**; panels should empty-state rather than look broken.

- **Not the cause:** Postgres datasource (BUG-0003 H pass); portfolio UNION SQL (BUG-0004 K pass); missing `net_worth_snapshots` (latest snapshot has 3 accounts, total **-3395.75 EUR**).



### Defect Z — no cross-account value overview



- **Portfolio account-breakdown panel SQL** returns only **one** account row (`LIMIT 1` on cross-join) instead of all asset accounts from latest snapshot.

- **No dedicated overview panel** listing per-account balances across analytics dashboards; React `/wealth` provides equivalent data outside Grafana embed — insufficient alone for AC Z unless documented as accepted equivalent.



### UX expectations (carry to architecture)



- Analytics dashboards with `$account_id` should default to the **primary funded account** (non-zero balance or forecast), not alphabetical first.

- Portfolio dashboard should expose a **cross-account table or stat row** visible without variable hunting.

- ML-dependent Grafana panels should **hide or label baseline-only mode** when ML sidecar disabled (DEC-0049 / US-0013).

