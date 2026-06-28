# Backlog

## Bug issues (canonical)

### BUG-0001 — Omniflow production regressions (auth + Grafana analytics)

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0007, 2026-06-04

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; Traefik `auth` middleware on UI; `AUTH_DEV_BYPASS=true`; OIDC env vars unset (no IdP configured). Operator report 2026-06-03.

**steps_to_reproduce:**

1. **Defect A (auth):** Deploy external profile with OIDC unset (post-Q0005). Open `https://financegnome.omniflow.cc`, pass Traefik basic-auth, open browser console, click header **AI** or **Chat** nav.
2. **Defect B (Grafana):** From same session, open each Analytics route: `/analytics/platform-health`, `/analytics/cashflow`, `/analytics/subscriptions`, `/analytics/budgets`, `/analytics/portfolio`, `/analytics/forecast-horizons`. Inspect Network tab for static asset requests.

**expected:**

- **Defect A:** `financegnome` loads without `AuthProvider` / `useAuth` console errors; AI Chat button opens `ChatPanel` without `TypeError` on `user`.
- **Defect B:** Each `/analytics/{slug}` route renders the Grafana dashboard; no `404` responses for `public/build/` or `public/img/` at site root. Static assets load via `/analytics/grafana/public/...` (or equivalent `root_url`/proxy fix).

**actual:**

- **Defect A:** `AuthProvider` console error `TypeError: Cannot read properties of undefined (reading 'user')` when clicking AI or Chat header nav; `useAuth()` returns `undefined` when OIDC env vars are unset.
- **Defect B:** All six `/analytics/{slug}` routes are `404` at the site root; Grafana's `/public/build/` and `/public/img/` 404 — assets fail to load (broken dashboard chrome).

**evidence_refs:** operator report 2026-06-03; browser console `useAuth` TypeError screenshot; `/analytics/cashflow` 404 screenshot.

**Related work:** **US-0010** external profile; `auth` middleware; Q0005 deployment of external profile.

#### Intake evidence (BUG-0001)

- `intake_run_id`: `intake-20260603-auth-grafana-404`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: OIDC env unset is intentional (no IdP yet); `AUTH_DEV_BYPASS=true` must continue to work; Grafana reverse proxy via `/analytics/grafana` is the fix approach.

**Decomposition:** single-bug with two independent sub-defects (auth + Grafana) — both blocking external profile usability. Fixed together.

**Related work:** **US-0010** external profile deployment; Q0005 external profile first deployment.

---

### BUG-0002 — Omniflow production integration defects (Firefly sync + API 404 + exchange settings)

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0008, manual Full sync Q0009, 2026-06-05

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; Firefly III hosted externally; `FIREFLY_BASE_URL` + `FIREFLY_PERSONAL_ACCESS_TOKEN` configured; Bitunix exchange configured.

**steps_to_reproduce:**

1. With `FIREFLY_BASE_URL` and `FIREFLY_PERSONAL_ACCESS_TOKEN` configured (names only; secrets redacted), run a Full Firefly sync manually from `/sync`.
2. Navigate to `/sync` (or any Analytics route that calls internal sync APIs).
3. Open Settings → Exchanges and view Binance/Bybit rows.
4. `GET /api/v1/plans/risk-score` on omniflow host.

**expected:**

- **Defect C (sync):** Manual Full Firefly sync completes; `GET /api/v1/sync/runs` shows completed run; entity counts non-zero; Sync Status page shows entity counts; no blocking 404 on `/api/v1/sync/*`.
- **Defect D (risk-score):** `GET /api/v1/plans/risk-score` returns **200** with JSON risk payload or documented empty-state (not `404`).
- **Defect E (exchange settings):** When only Bitunix credentials are set in `.env`, Settings shows Bitunix as `enabled` and `configured`; Binance/Bybit rows match operator env (no false Binance `enabled` + `configured`).
- OIDC-enabled and bundled-firefly deploy regression checks pass.

**actual:**

- **Defect C (sync):** Manual Firefly sync returns `500` with `request to Firefly returned error: 404 Not Found`; sync history page shows failed runs; Sync Status page shows blocked 404.
- **Defect D (risk-score):** `GET /analytics/portfolio` returns `404` at site root (Grafana panel unreachable); `GET /api/v1/plans/risk-score` returns `404`.
- **Defect E (exchange settings):** Binance and Bybit rows incorrectly show `enabled` = `true` + `configured` = `true` when only Bitunix `.env` variables are set (e.g. `BINANCE_API_KEY` empty string but key present in schema).

**evidence_refs:** `handoffs/intake_evidence/intake-20260604-firefly-sync-404.json`; `/sync` 404 screenshot; `/settings` Binance false positive screenshot; `GET /api/v1/plans/risk-score` 404 response.

**Related work:** **BUG-0001** prior auth/Grafana defects; [R-0057](docs/engineering/research.md#r-0057) (Firefly PAT contract); exchange settings schema `enabled: true` default.

#### Intake evidence (BUG-0002)

- `intake_run_id`: `intake-20260604-firefly-sync-404`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Firefly PAT is configured but sync URL is wrong (404 vs 401); Binance/Bybit env vars unset but key-in-schema marks them enabled; OIDC middleware passes through.

**Decomposition:** single-bug cluster — three independent defects found during first Full sync + settings review on omniflow. Fixed together.

---

### BUG-0003 — Omniflow production API 500 cascade, Bitunix test, Grafana SQL

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0010, 2026-06-06 — all three defects fixed

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; Traefik `auth` middleware; Postgres `15` on omniflow host; Grafana via `/analytics` reverse proxy; `BITUNIX_API_KEY` + `BITUNIX_API_SECRET` configured (names only).

**steps_to_reproduce:**

1. With fresh external profile deployment (post-BUG-0002 fix), open `/sync` and trigger a Full Firefly sync. Wait for scheduled run or trigger manual sync.
2. `GET /api/v1/alerts/unread-count`, `GET /api/v1/sync/entities`, `GET /api/v1/sync/runs`, `GET /api/v1/exchanges`, `GET /api/v1/forecast/monthly?account_id=N`, `GET /api/v1/subscriptions/recurring`, `GET /api/v1/ai/audit` (representative product endpoints).
3. Open `/sync` (Sync Status page); inspect `GET /api/v1/settings` response body for `database_host` and `database_mode`.
4. With only Bitunix env credentials set (Binance/Bybit unset), open Settings → Exchanges and observe Bitunix row status.
5. Open Grafana Analytics → Cashflow and Portfolio dashboards; inspect panel SQL and data source responses.
6. `POST /api/v1/exchanges/bitunix/test` with valid Bitunix keys in env.

**expected:**

- **Defect F (API cascade):** Representative product endpoints return **200** within normal latency (not 500 after ~30s DB timeout); `settings` shows `database_host: postgres` and `database_mode: external`.
- **Defect G (Bitunix test):** `POST /api/v1/exchanges/bitunix/test` returns **200** with connection test payload or documented auth failure (not `400 unknown exchange: bitunix`).
- **Defect H (Grafana SQL):** `POST /analytics/grafana/api/ds/query` for provisioned dashboards returns **200** (SQL executes); Grafana Postgres datasource queries reach in-network `postgres`.
- OIDC-enabled and bundled-firefly deploy regression checks pass.

**actual:**

- **Defect F (API cascade):** After initial sync attempt (`500 Firefly 404`), every representative `GET /api/v1/*` product endpoint returns **500** after ~30s timeout (`pq: connection refused` / db pool exhaustion cascading from blocked sync). Sync Status page stuck on `running` or `failed`. `GET /api/v1/settings` shows `database_host: unknown`.
- **Defect G (Bitunix test):** `POST /api/v1/exchanges/bitunix/test` returns **400** `{"error": "unknown exchange: bitunix"}` despite `BITUNIX_API_KEY` + `BITUNIX_API_SECRET` present in `.env`.
- **Defect H (Grafana SQL):** `POST /analytics/grafana/api/ds/query` for Cashflow and Portfolio panels returns **500** `pq: syntax error at or near "UNION"` (panel SQL invalid); Grafana Postgres datasource shows red "unreachable" in datasource config when accessed via `/analytics` proxy.

**evidence_refs:** API 500 trace logs; `POST /api/v1/exchanges/bitunix/test` response body; Grafana panel SQL error screenshot; `GET /api/v1/settings` response; network tab `/analytics/grafana/api/ds/query` 500.

**Related work:** **BUG-0002** prior sync 404; **US-0007** Bitunix exchange; Grafana provisioning; [R-0057](docs/engineering/research.md#r-0057).

#### Intake evidence (BUG-0003)

- `intake_run_id`: `intake-20260606-api-cascade-bitunix-grafana`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: API 500 cascade caused by sync failure exhausting db pool; Bitunix connector exists but is not wired to `/exchange/test` route; Grafana UNION syntax from stale dashboard JSON; database connection string uses DNS `postgres` not IP.

**Decomposition:** single-bug cluster — three independent omniflow surface defects. Fixed together.

---

### BUG-0004 — Post-sync pipeline empty analytics (stuck exchange sync, subscriptions, Grafana SQL)

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0011, 2026-06-09 — post-sync pipeline fixed; Q0012 verify-work added subscription detection improvements and portfolio SQL validation

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; **922** transactions synced from live Firefly mirror; Grafana via `/analytics` reverse proxy; Bitunix exchange configured.

**steps_to_reproduce:**

1. Run a Full Firefly sync on omniflow with 922+ transactions in mirror.
2. Wait for scheduled exchange sync to complete; check `GET /api/v1/sync/status`.
3. Open `/subscriptions` (Pending tab) and observe detected patterns.
4. Open Analytics → Portfolio dashboard and observe panel SQL / data responses.
5. Open `/forecast` and `/wealth` pages and observe values for account 114 (Raiffeisenbank Giro).

**expected:**

- **Defect I (exchange sync):** After exchange sync runs, `GET /api/v1/sync/status` shows `state: completed` (not `running` with `finished_at: null`).
- **Defect J (subscription detection):** `/subscriptions` Pending tab surfaces recurring patterns for known merchants (e.g. streaming services) — not permanently empty `[]`.
- **Defect K (portfolio SQL):** Grafana Portfolio dashboard panel SQL returns **200** (no `pq: syntax error at or near "UNION"`).
- **Defect L (forecast/wealth data):** `/forecast` and `/wealth` show populated data for synced accounts (e.g. account 114 with net worth; forecast series not all zero).

**actual:**

- **Defect I (exchange sync):** After Firefly sync completes, exchange sync runs indefinitely; `sync_status` returns `state: running` with `finished_at: null`; exchange phase never reaches terminal state.
- **Defect J (subscription detection):** `/subscriptions` Pending tab shows `[]` or only **11 pending** patterns — does not surface operator-known recurring merchants (streaming, Amazon, etc.) despite 922+ transactions in mirror.
- **Defect K (portfolio SQL):** Portfolio analytics panel SQL fails with `pq: syntax error at or near "UNION"`; `POST /analytics/grafana/api/ds/query` returns **500** for portfolio allocation query.
- **Defect L (forecast/wealth data):** `/forecast` and `/wealth` show empty values for all synced accounts; `net_worth_snapshots` rows absent; forecast series all zero.

**evidence_refs:** `sync_status` stuck-running response; `/subscriptions` empty Pending tab screenshot; Grafana portfolio panel SQL error screenshot; `GET /api/v1/sync/status` running response body.

**Related work:** **BUG-0001**/**BUG-0002**/**BUG-0003** resolved; exchange sync pipeline; subscription detection engine; Grafana provisioning; **US-0003** detection; **US-0007** exchange sync.

#### Intake evidence (BUG-0004)

- `intake_run_id`: `intake-20260606-post-sync-pipeline`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: exchange sync completes Firefly phase but never reaches terminal state; subscription detection threshold too strict for operator's merchant set; portfolio SQL UNION syntax from stale provisioning JSON; forecast/wealth require explicit recompute after sync.

**Decomposition:** single-bug cluster — post-sync pipeline touches sync status, subscription engine, portfolio SQL, and recompute orchestration. Fixed together.

---

### BUG-0005 — Exchange sync spot-only (Bitunix futures / multi-product accounts)

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0015, 2026-06-20 — multi-product exchange sync + wealth crypto subtotal verified

**environment:** US-0010 external Compose profile; Bitunix connected with futures + spot exposure; read-only API keys; Grafana `/analytics` proxy; omniflow Traefik deployment.

**steps_to_reproduce:**

1. Connect Bitunix exchange (read-only API keys) with operator account containing **both** futures and spot balances.
2. Run manual Full sync (Firefly + exchange).
3. Inspect `GET /api/v1/wealth` → `holdings` / `holdings_count` / `crypto` values; open `/wealth` Crypto tab.
4. `GET /api/v1/exchanges/bitunix/positions` (or equivalent) and observe `product_type` per row.

**expected:**

- **Defect M (futures ingest):** `sync_positions` ingests futures and margin balances/positions (not just spot wallet); `GET /api/v1/holdings` includes rows with `product_type` in (`linear`, `inverse`, `margin`) alongside `spot`.
- **Defect N (Bitunix API path):** Bitunix futures REST client uses `fapi.bitunix.com` (or equivalent futures endpoint) with header-auth per R-0058; `sync_positions` / balance endpoints return non-empty when keys are valid.
- **Defect O (wealth crypto total):** `/wealth` **Crypto** tab + `GET /api/v1/wealth` **`crypto.subtotal_eur`** reflects combined spot + futures exchange holdings on US-0010 external profile.
- Read-only key constraint preserved; OIDC-enabled and bundled-firefly deploy regression checks pass.

**actual:**

- **Defect M (futures ingest):** Post-sync `holdings` only includes `spot` rows; `product_type = linear` / `inverse` / `margin` absent; operator futures portfolio (~11 linear positions, ~€2000 equivalent) not reflected.
- **Defect N (Bitunix API path):** Bitunix connector issues requests to `api.bitunix.com` only (spot); `fapi.bitunix.com` futures endpoint never called.
- **Defect O (wealth crypto total):** `crypto.subtotal_eur = 0.0` on `/wealth` Crypto tab despite Bitunix reporting **7 open positions** and **connected accounts**.

**evidence_refs:** `GET /api/v1/wealth` response body; Bitunix connector source (`bitunix.rs`) L220-260; sync logs showing `product_type=spot` only.

**Related work:** **US-0007** crypto portfolio; **BUG-0002**/**BUG-0003** prior exchange defects; [R-0058](docs/engineering/research.md#r-0058--bitunix-futures-rest-api-v1).

#### Intake evidence (BUG-0005)

- `intake_run_id`: `intake-20260618-bitunix-futures-sync`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Read-only Bitunix keys can read futures balances; futures REST endpoint `fapi.bitunix.com` is the canonical path per R-0058; wealth aggregation sums all `product_type` rows; no Firefly write-back required.

**Decomposition:** single-bug — Bitunix spot-only sync requires futures endpoint integration; wealth aggregation bug is a downstream effect. Fixed together as exchange sync extension.

---

### BUG-0006 — AI get_transactions sees no expenses despite synced transactions

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0016, 2026-06-21 — AI transaction aggregate + category_id persist verified

**environment:** US-0010 external Compose profile; Firefly **922** transactions synced; `allow_raw_transactions=false`; AI Chat queries category/spending questions.

**steps_to_reproduce:**

1. Run Full Firefly sync on omniflow (922+ transactions).
2. Open AI Chat and ask: *"Wieviel habe ich im Mai für Strom ausgegeben?"*, *"Zeige mir meine letzten 10 Ausgaben"*, *"Was sind meine Top-Kategorien diesen Monat?"*.
3. Inspect `ai_tool_audit` log for tool-call traces; `GET /api/v1/ai/audit` for audit rows.
4. Inspect `transactions` mirror table: `SELECT COUNT(*), category_id FROM transactions GROUP BY category_id`.

**expected:**

- **Defect P (AI chat):** AI answers with data-backed amounts or explicit empty-state ("Keine Ausgaben im Mai für Strom gefunden"); not "Ich kann keine Ausgaben sehen" / "Keine Daten verfügbar" when mirror holds matching rows.
- **Defect Q (category_id persist):** Firefly sync persists `category_id` (or equivalent) on mirrored `transactions` rows used by `TransactionsRepository::aggregates_by_category`.
- **Defect R (aggregate payload):** `get_transactions` aggregate JSON includes explicit period transaction totals/counts; distinguishes empty-period vs uncategorized vs zero-outflow cases under `allow_raw_transactions=false`.
- Privacy redaction and six-tool registry constraints preserved; OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect P (AI chat):** AI Chat responds *"Es scheint, dass es keine Ausgaben für Strom im Mai gibt"* / *"Keine Daten verfügbar"* despite `transactions` mirror holding May 2026 Stromkosten rows (category_id populated).
- **Defect Q (category_id persist):** `SELECT COUNT(*) FROM transactions WHERE category_id IS NOT NULL` returns **0**; Firefly sync persist code path drops `category_id` attribute (maps only `amount`, `date`, `description`, `source_name`, `destination_name`).
- **Defect R (aggregate payload):** `get_transactions` aggregate returns zero totals and no period count breakdown; no documented distinction between empty-period and uncategorized cases.

**evidence_refs:** AI Chat transcript screenshot; `ai_tool_audit` tool-call trace; `SELECT COUNT(*)` query output; `TransactionsRepository::aggregates_by_category` signature; `get_transactions` aggregate response body.

**Related work:** **US-0006** AI assistant; **US-0018** category filter; [R-0059](docs/engineering/research.md#r-0059--firefly-category-id-mapping-contract).

#### Intake evidence (BUG-0006)

- `intake_run_id`: `intake-20260618-ai-chat-empty-transactions`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `category_id` attribute dropped during Firefly sync insert; mirror aggregate query relies on `category_id`; AI tool `get_transactions` uses `aggregates_by_category`; fix must preserve privacy defaults (`allow_raw_transactions=false`).

**Decomposition:** single-bug — Firefly sync category_id drop + AI aggregate response contract. Fixed together.

---

### BUG-0007 — AI merchant/category discovery fails despite mirror data

Status: DONE
Priority: P1

**closure_note:** verify-work 2026-06-21 PASS for S/U; T partial (`group_by: month` + `category_search` advisory, non-blocking)

**environment:** US-0010 external Compose profile; Firefly 922 transactions synced; `allow_raw_transactions=false`; AI Chat queries specific merchants / categories.

**steps_to_reproduce:**

1. After successful sync, open AI Chat and ask: *"Liste alle meine Streaming-Dienste auf"*, *"Wieviel habe ich für Strom im Januar bis Oktober 2023 ausgegeben?"*, *"Zeige Amazon-Ausgaben"*.
2. Observe AI response; check `ai_tool_audit` tool-call traces.
3. Compare AI response vs `SELECT * FROM transactions WHERE description ILIKE '%amazon%'` and equivalent category/description queries against mirror.

**expected:**

- **Defect S (subscription listing):** AI enumerates **subscription/streaming merchant payee names** from `get_subscriptions` and/or `get_transactions` when asked (e.g. *"Netflix €9.99, Disney+ €7.99"*) — not generic industry list (*"Streaming-Anbieter sind Netflix, Disney+..."*) or *"cannot retrieve"*.
- **Defect T (merchant/category search):** Merchant/category queries (Strom Jan–Oct 2023, Amazon, streaming) return **data-backed amounts** or explicit empty-state showing that search was attempted — not blanket *"Keine Ausgaben gefunden"*.
- **Defect U (tool orchestration):** AI fuses **category, transaction name/description, account, and amounts** in tool orchestration without requiring the user to name merchants explicitly.
- Privacy `allow_raw_transactions=false` and six-tool registry preserved; OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect S (subscription listing):** AI responds *"Hier sind typische Streaming-Dienste: Netflix, Disney+, Amazon Prime, Apple TV+"* (generic industry list) — no actual operator merchant names.
- **Defect T (merchant/category search):** Queries like *"Strom Jan–Oct 2023"* or *"Amazon Ausgaben"* return *"Keine Ausgaben gefunden"* despite mirror holding matching rows (e.g. SEPA Dauerauftrag Stadtwerke, Amazon Marketplace transactions).
- **Defect U (tool orchestration):** AI requires user to name merchant explicitly; does not attempt fuzzy description/category/account search.

**evidence_refs:** AI Chat transcript screenshots; `ai_tool_audit` tool-call trace; mirror `SELECT` count for Amazon/Strom transactions; `get_subscriptions` list response.

**Related work:** **BUG-0006** category_id ingest; **US-0006** AI assistant; [R-0059](docs/engineering/research.md#r-0059); [R-0060](docs/engineering/research.md#r-0060--ai-merchant-fuzzy-search-tools).

#### Intake evidence (BUG-0007)

- `intake_run_id`: `intake-20260618-ai-merchant-discovery`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Mirror `description`/`category_id`/`source_name` fields contain searchable merchant signals; AI needs a tool-side fuzzy `category_search`/`merchant_search` parameter; privacy allows aggregate signals only; `allow_raw_transactions=false` must be preserved; **T**: `group_by: month` aggregate for period-range queries.

**Decomposition:** single-bug — AI tool-side search parameters required for merchant/discovery queries (not Firefly sync defect). Fixed at AI tool orchestration layer.

**Discovery note V:** RAG / Firefly-side full-text indexing discussed; tool-enhancement chosen (DEC-0021) — not stored as acceptance gate.

---

### BUG-0008 — Subscription alerts vs list mismatch & under-detection

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0017, 2026-06-22 — alert/list reconciliation + detection recall improved; Q0018 verify-work confirmed standing-order alert handling and subscription price-change detection fidelity

**environment:** US-0010 external Compose profile; 922 transactions synced; Subscriptions page `/subscriptions`, header Alerts panel; AI Chat streaming queries.

**steps_to_reproduce:**

1. With 922 synced transactions, open `/subscriptions` **Pending** tab and count rows.
2. Open header **Alerts** panel (bell icon) and count `new_detection` / unread alerts.
3. Compare alert count vs tab count — document any discrepancy.
4. Trigger a re-run of subscription detection (e.g. via `/sync` Full sync or detection re-evaluation path).
5. Confirm/detect a known recurring merchant (e.g. streaming service) and observe subsequent detection runs.

**expected:**

- **Defect W (alert/list reconciliation):** Subscription-scoped alert unread count reconciles with `/subscriptions` Pending + Confirmed + Standing orders tab row counts — no unexplained mismatch (e.g. **33 alerts vs 11 list rows**).
- **Defect X (detection recall):** Subscription detection surfaces materially more recurring patterns from 922+ synced transactions for operator-known merchants — higher recall than post-BUG-0004 **11 pending** alone.
- OIDC-enabled and bundled-firefly deploy regression checks pass.

**actual:**

- **Defect W (alert/list reconciliation):** Header shows **33 unread alerts** while `/subscriptions` Pending tab shows only **11 rows**; dismissed/rejected/dedup semantics not documented in UI or API.
- **Defect X (detection recall):** Detection surfaces only **11 pending** patterns from 922+ transactions — under-recall vs operator-known merchants (streaming, shopping subscriptions not detected).

**evidence_refs:** Header Alerts panel badge count screenshot; `/subscriptions` Pending tab screenshot (11 rows); `GET /api/v1/subscriptions/alerts` response body; `GET /api/v1/subscriptions?status=pending` response body.

**Related work:** **US-0003** subscription detection; **BUG-0004** post-sync pipeline; detection engine `detect_recurrence_groups`.

#### Intake evidence (BUG-0008)

- `intake_run_id`: `intake-20260622-subscription-alert-mismatch`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Alert/list discrepancy caused by dismissed/rejected alerts counted in header but filtered in list; detection recall improvement requires tuning `detect_recurrence_groups` thresholds and/or AI-assisted detection pass; standing-order patterns detected separately (AC-4 US-0003).

**Decomposition:** single-bug — alert/list reconciliation + detection recall improvements; both required for operator trust.

---

### BUG-0009 — Grafana empty panels & missing account value overview

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0019, 2026-06-23 — Grafana panels populated; operator cross-account overview surfaced; BUG-0013 extended coverage for cashflow/crypto/budget anomalies

**environment:** US-0010 external Compose profile; 922 transactions synced; Grafana via `/analytics` proxy (six routes); `account_id` variable on dashboards; Bitunix exchange connected.

**steps_to_reproduce:**

1. After Full sync + recompute, open each Analytics route: `/analytics/platform-health`, `/analytics/cashflow`, `/analytics/subscriptions`, `/analytics/budgets`, `/analytics/portfolio`, `/analytics/forecast-horizons`.
2. Set `account_id` variable to a synced Firefly asset account (e.g. account 114).
3. Inspect `POST /analytics/grafana/api/ds/query` responses for each dashboard.
4. Look for a cross-account value overview in Analytics or Wealth embedded panel.

**expected:**

- **Defect Y (panel values):** Representative Grafana dashboards return **200** with **non-empty panel values** when `account_id` variable matches synced Firefly asset accounts — not persistent empty panels post-BUG-0004.
- **Defect Z (account overview):** Operator has **cross-account value overview** in analytics (Grafana summary panel/table or documented equivalent via embedded wealth link showing per-account totals).
- Six `/analytics/{slug}` routes operator smoke pass.

**actual:**

- **Defect Y (panel values):** Cashflow, Portfolio, Budgets, Forecast dashboards show **empty panels** despite populated mirror and successful recompute; `POST /analytics/grafana/api/ds/query` returns `{results: [{frames: []}]}` when `account_id` set to synced asset account.
- **Defect Z (account overview):** No single place shows per-account net worth / balance / exposure overview — operator must click between dashboards to piece together cross-account picture.

**evidence_refs:** Screenshot of empty Grafana Cashflow panel with `account_id=114`; `POST /analytics/grafana/api/ds/query` response body; six-route smoke test output.

**Related work:** **BUG-0002**/**BUG-0003** prior Grafana defects; **BUG-0013** related cashflow/budget anomalies; **US-0005** wealth; Grafana provisioning.

#### Intake evidence (BUG-0009)

- `intake_run_id`: `intake-20260623-grafana-empty-panels`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Grafana dashboard SQL filters by `account_id` variable; variable mapping must bind to Firefly asset account IDs in mirror; empty frames indicate either SQL returns zero rows (variable not bound) or query returns filtered data; cross-account summary requires a new summary panel or an embedded `/wealth` link.

**Decomposition:** single-bug cluster — Grafana panel data path + cross-account overview gap. Fixed together.

---

### BUG-0010 — Forecast & Wealth empty/wrong numbers; ML skipped

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0020, 2026-06-23 — signed balances / ML / wealth totals verified; BUG-0014 extended hardening for post-rebuild omniflow cluster

**environment:** US-0010 external Compose profile; Full sync complete; `/forecast` and `/wealth` pages; ML forecast sidecar `stats-forecast` (US-0013).

**steps_to_reproduce:**

1. Open `/forecast` on omniflow (account 114 Raiffeisenbank Giro); observe 3-month **End balance** and daily/monthly/long-term series.
2. Open `/wealth` and observe `total_eur`, account breakdown, crypto values.
3. `GET /api/v1/forecast/meta` and observe ML forecast availability.

**expected:**

- **Defect AA (forecast balance):** Forecast UI shows **plausible signed balances** for selected funded account — including 3-month **End balance** (not implausible **-25365.78** without explicit deficit scenario configured).
- **Defect AB (wealth totals):** `/wealth` shows **non-empty** account breakdown and `total_eur` for synced Firefly asset accounts.
- **Defect AC (ML forecast):** ML forecast path runs when sidecar config available on profile; if ML skipped, UI/API states degraded baseline-only mode accurately — not generic *"ML skipped: ML forecast unavailable..."* when ML is truly unavailable.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect AA (forecast balance):** `/forecast` shows 3-month **End balance: -25365.78 €** for funded account with ~€2000 positive balance — implausible deficit number without explicit deficit scenario.
- **Defect AB (wealth totals):** `/wealth` shows `total_eur: 0.0` and empty account breakdown for synced Firefly asset accounts.
- **Defect AC (ML forecast):** `GET /api/v1/forecast/meta` returns `ml_skipped_reason: "ML forecast unavailable (US-0013 not enabled)"` — ambiguous whether ML is truly unavailable or misconfigured.

**evidence_refs:** `/forecast` screenshot showing -25365.78; `/wealth` screenshot showing €0 totals; `GET /api/v1/forecast/meta` response body; `stats-forecast` sidecar logs.

**Related work:** **BUG-0009** prior Grafana defects; **US-0002** forecast; **US-0005** wealth; **US-0013** ML hardening; `DEC-0007` forecast projection baseline.

#### Intake evidence (BUG-0010)

- `intake_run_id`: `intake-20260623-forecast-wealth-numbers`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Negative balance is a projection bug (likely missing income bucket); wealth total_eur aggregation missing account balance join; ML skip message ambiguous; `stats-forecast` sidecar health check determines ML availability.

**Decomposition:** single-bug cluster — forecast + wealth numbers share the income/balance projection path; ML skip message needs clarity. Fixed together.

---

### BUG-0011 — Planning mode broken (empty plan, compare sums, plan-vs-actual 404)

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0021, 2026-06-24 — planning mode fixed; BUG-0024 extended plan-delete surface fix

**environment:** US-0010 external Compose profile; `/planning` Scenarios + Compare + Plan vs Actual tabs; operator 2026-06-24.

**steps_to_reproduce:**

1. Open `/planning` with no plans created.
2. Click **Start empty and add lines** or equivalent empty-plan affordance; attempt to add an adjustment line.
3. Open **Compare** tab with an empty or minimal plan; observe monthly delta and projected balance.
4. Open **Plan vs Actual** tab and observe rendering; watch Network tab for `GET /api/v1/plans/active/plan-vs-actual`.

**expected:**

- **Defect AD (empty plan UX):** Creating an empty plan gives the operator an editable empty plan with add-lines UX — not a silent no-op.
- **Defect AE (compare deltas):** Compare tab with empty/minimal plan shows **zero or neutral deltas** — not illogical aggregates (e.g. **-127489.44** monthly delta, **-4042.41** projected month-end).
- **Defect AF (plan-vs-actual):** `GET /api/v1/plans/active/plan-vs-actual` returns **200** with JSON when an active plan exists, or a **200 documented empty-state/guided UX** when none — not raw **404** breaking the Plan vs Actual tab.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect AD (empty plan UX):** Clicking "Start empty and add lines" does nothing — no plan is created; page stays on template grid; no error message.
- **Defect AE (compare deltas):** Compare tab with minimal plan shows **-127489.44** monthly delta and **-4042.41** projected month-end (illogical aggregates from missing baseline guard).
- **Defect AF (plan-vs-actual):** Plan vs Actual tab is blank/broken; Network tab shows `GET /api/v1/plans/active/plan-vs-actual` returning raw **404** when no active plan exists.

**evidence_refs:** `/planning` screenshots for all three defects; Network tab 404 for plan-vs-actual; `GET /api/v1/plans/active/plan-vs-actual` 404 response body.

**Related work:** **US-0004** planning scenarios; plan-vs-actual contract; `decide_monthly_delta` projection function.

#### Intake evidence (BUG-0011)

- `intake_run_id`: `intake-20260624-planning-mode-broken`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Empty-plan UX requires a new `create_plan` mutation with empty-lines path; compare delta baseline guard missing for first-month projection; plan-vs-actual API requires explicit empty-state response instead of 404.

**Decomposition:** single-bug — three planning surface defects sharing the same plan-lifecycle API contract. Fixed together.

---

### BUG-0012 — Forecast monthly Income/Fixed buckets always zero

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0021, 2026-06-24 — income/fixed bucket assignments verified; BUG-0015 extended subscription persistence after rebuild

**environment:** US-0010 external Compose profile; Full sync complete + recompute; `/forecast` Monthly tab; account 114 with income-category inflows + fixed-cost outflows.

**steps_to_reproduce:**

1. After Full sync + recompute, open `/forecast` Monthly tab (account 114).
2. Observe summary cards: Income, Fixed, Variable, Free cashflow.
3. `GET /api/v1/forecast/monthly?account_id=114` and inspect `income`, `fixed`, `variable` values.

**expected:**

- **Defect AG (income):** Monthly forecast shows **non-zero Income** when mirror holds income-category inflows (salary, refunds, etc.) in the forecast month — not permanently **Income: 0** while categorized rows exist.
- **Defect AH (fixed):** Monthly forecast shows **non-zero Fixed** when mirror holds fixed-cost category outflows (rent, utilities, standing orders per DEC-0007 category map) — not **Fixed: 0** with all spend under Variable only.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect AG (income):** `GET /api/v1/forecast/monthly` returns `income: 0.0` for months where mirror holds income-category transactions.
- **Defect AH (fixed):** `fixed: 0.0` for months where mirror holds fixed-cost category outflows (rent, Strom, standing orders); all spend categorized as Variable.

**evidence_refs:** `/forecast` Monthly tab screenshot (Income 0, Fixed 0); `GET /api/v1/forecast/monthly?account_id=114` response body; mirror `transactions` rows with income/fixed `category_id`.

**Related work:** **US-0018** category filters; **DEC-0007** forecast category→bucket mapping; [R-0061](docs/engineering/research.md#r-0061--forecast-category-bucket-mapping).

#### Intake evidence (BUG-0012)

- `intake_run_id`: `intake-20260624-forecast-buckets-zero`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Bucket mapping `income`/`fixed` requires mirror `category_id` join to DEC-0007 category→bucket config; current path falls through to Variable default; fix required at projection layer.

**Decomposition:** single-bug — bucket mapping requires DEC-0007 category→bucket join. Fixed at projection layer.

---

### BUG-0013 — Omniflow analytics regression cluster (Grafana zeros, crypto, budgets)

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0022, 2026-06-24 — cashflow/crypto/budget panels populate; exchange balances populate; BUG-0014 extended the cluster with ML sidecar, planning, and crypto display fixes

**environment:** US-0010 external Compose profile; 922 transactions synced; Grafana via `/analytics` (six routes); Bitunix exchange connected with holdings; active plan exists.

**steps_to_reproduce:**

1. After Full sync + recompute (post-BUG-0009 fix), open `/analytics/cashflow` and set `account_id=114`.
2. Observe **balance forecast** and **recent daily balances** panels.
3. Open `/subscriptions` and inspect **Price changes (90 days)** panel.
4. Open `/analytics/portfolio` and observe **crypto value**, **total return %**, **FX incomplete** banner.
5. Open `/analytics/budgets` and observe MTD planned/actual/deviation rows.
6. Open `/analytics` routes and observe Grafana API responses.

**expected:**

- **Defect AI (cashflow):** Cashflow Grafana **balance forecast** and **recent daily balances** show **non-empty signed balances** for funded asset account — not persistent flat **0 €** regression vs BUG-0009 closure.
- **Defect AJ (subscriptions price changes):** Subscriptions **Price changes (90 days)** panel shows change rows when confirmed subscriptions had amount changes in period, or a documented empty-state when none.
- **Defect AK (portfolio crypto):** Portfolio **crypto value** reflects exchange holdings when sync populated positions; **FX incomplete** banner only with documented `unpriced_assets` or partial totals; **total return %** populated when snapshot history exists.
- **Defect AL (budgets MTD):** Budgets MTD plan/actual/deviation rows are **plausible** for active plan — not unexplained **Planned MTD −€150K** with **Actual €0** unless plan defines that magnitude.
- **Defect AM (Grafana fetch):** `POST /analytics/grafana/api/ds/query` and annotation queries return **200** without browser **Failed to fetch** on omniflow.
- **Defect AN (exchange crypto):** Exchange crypto balances appear in wealth/portfolio totals when venue sync succeeds — not permanently **€0** with configured read-only keys.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect AI (cashflow):** Cashflow Grafana panels show flat **0** for balance forecast and recent daily balances despite populated mirror and non-zero API forecast.
- **Defect AJ (subscriptions price changes):** Price changes (90 days) panel is empty even though operator confirmed subscriptions with amount changes in period.
- **Defect AK (portfolio crypto):** Portfolio shows **crypto value: €0**, **total return %: —**, **FX incomplete** banner visible without documented `unpriced_assets`.
- **Defect AL (budgets MTD):** Budgets MTD shows **Planned MTD −€150K** with **Actual €0** — unexplained magnitude not matching any operator-defined plan.
- **Defect AM (Grafana fetch):** Browser **Failed to fetch** on annotation queries; `POST /analytics/grafana/api/ds/query` returns 500 or empty results for some panels.
- **Defect AN (exchange crypto):** Exchange crypto balances show **€0** in wealth/portfolio totals despite Bitunix connected and reporting 7 positions with unrealized EUR.

**evidence_refs:** Grafana dashboard screenshots (cashflow zero, budgets -€150K, portfolio crypto €0); `POST /analytics/grafana/api/ds/query` responses; exchange sync logs.

**Related work:** **BUG-0009** prior Grafana defects; **BUG-0010** forecast numbers; **US-0007** exchange; **US-0013** ML overlay; Grafana provisioning.

#### Intake evidence (BUG-0013)

- `intake_run_id`: `intake-20260624-analytics-regression-cluster`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Cashflow Grafana uses cached forecast snapshots; snapshot table empty or stale; subscription price-change detection re-runs on sync; portfolio crypto requires exchange position join; budget MTD uses active plan baseline (likely wrong baseline); Grafana annotation endpoint misconfigured on omniflow.

**Decomposition:** single-bug cluster — analytics regression touches six surfaces sharing Grafana data path + sync recompute pipeline. Fixed together.

---

### BUG-0014 — Post-rebuild omniflow cluster (ML sidecar, crypto display, Grafana zeros, planning delete)

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0023, 2026-06-25 — ML sidecar started, budget MTD fixed, Grafana annotations wired, exchange crypto priced; Q0023 verify-work confirmed crypto value + PnL return surface

**environment:** US-0010 external Compose profile; `stats-forecast` sidecar configured; Bitunix connected; Grafana via `/analytics`; rebuild 2026-06-25.

**steps_to_reproduce:**

1. Rebuild + redeploy omniflow (post-BUG-0013 fixes).
2. Confirm `stats-forecast` is started and healthy in compose; run Full sync + ML recompute.
3. `GET /api/v1/forecast/meta` and inspect `ml_computation_id` and `ml_skipped_reason`.
4. Open `/wealth` Crypto tab and observe Bitunix card, holdings **Value EUR**, PnL **Total return %**.
5. Open `/analytics/cashflow` (account 114) and inspect balance forecast panels.
6. Open `/planning` and attempt to delete a non-active plan, then attempt to delete the globally active plan.

**expected:**

- **Defect AO (ML sidecar):** With `FORECAST_ML_ENABLED=true` and healthy sidecar, `GET /api/v1/forecast/meta` shows ML available (`ml_computation_id` set after Full sync) or accurate degraded copy — not permanent **US-0013 not enabled** when env opts in.
- **Defect AP (wealth crypto):** Wealth crypto subtotal, exchange cards, and portfolio panels show **non-zero** values when Bitunix reports connected holdings (e.g. **7** positions) and wallet/unrealized equity exists per DEC-0080 — not **€0** everywhere with only a holdings count.
- **Defect AQ (crypto FX):** Crypto surfaces show **native asset amounts** and **EUR equivalents** at valuation time; **FX incomplete** banner only with documented `unpriced_assets` or partial totals — not when wallet equity is priced.
- **Defect AR (Grafana cashflow):** Cashflow Grafana **balance forecast** and **recent daily balances** show **non-zero signed values** for funded account **114** after Full sync + recompute — not persistent flat **0** when API forecast non-zero.
- **Defect AS (planning delete):** Operator can **delete** a non-active plan from `/planning` UI (or documented equivalent); plan mutations surface errors; target-type UX improved beyond household/subscription/account confusion.
- **Defect AT (stats-forecast start):** External profile operator rebuild/runbook starts **`stats-forecast`** with app + Grafana when ML enabled.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect AO (ML sidecar):** `GET /api/v1/forecast/meta` returns `ml_skipped_reason: "US-0013 not enabled"` despite `FORECAST_ML_ENABLED=true` + `stats-forecast` configured in compose.
- **Defect AP (wealth crypto):** Bitunix card shows **€ -0,00** with **11** open positions; holdings **Value EUR** all **—**; **Total return —**.
- **Defect AQ (crypto FX):** **FX incomplete** banner visible with no `unpriced_assets` documentation; wallet value not priced.
- **Defect AR (Grafana cashflow):** Cashflow Grafana shows flat **0** despite `GET /api/v1/forecast/monthly?account_id=114` returning non-zero series.
- **Defect AS (planning delete):** Plan deletion UX broken — delete button permanently disabled regardless of plan state; target-type confusion in plan mutations.
- **Defect AT (stats-forecast start):** `stats-forecast` sidecar not started by rebuild; requires manual `docker compose up stats-forecast`.

**evidence_refs:** `GET /api/v1/forecast/meta` response; `/wealth` Crypto tab screenshot; Grafana cashflow panel with account 114; `stats-forecast` compose profile config.

**Related work:** **BUG-0010**/**BUG-0013** prior defects; **US-0007** crypto portfolio; **US-0013** ML hardening; **DEC-0080** crypto subtotal contract; **DEC-0082** planning delete.

#### Intake evidence (BUG-0014)

- `intake_run_id`: `intake-20260625-omniflow-post-rebuild`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: ML sidecar compose start requires `external` profile union; crypto value EUR requires wallet + futures wallet row parsed; Grafana cashflow uses stale snapshot cache; planning delete UX requires frontend guard refactor; `stats-forecast` must be added to rebuild runbook.

**Decomposition:** single-bug cluster — post-rebuild omniflow hardening across ML, crypto, Grafana snapshot, planning UX, and runbook. Fixed together.

---

### BUG-0015 — Confirmed subscriptions reappear as pending after rebuild

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0024, 2026-06-25 — confirmed fingerprint skip logic + alert dedup verified; Q0029 verify-work confirmed pattern deduplication and alert reconciliation

**environment:** US-0010 external Compose profile; operator confirmed subscriptions (CURSOR, APPLE.COM/BILL, etc.); rebuild + container recreation 2026-06-25.

**steps_to_reproduce:**

1. Confirm subscription patterns (e.g. **CURSOR** €17.18/mo, **APPLE.COM/BILL** €9.99/mo, **YOUTUBE**).
2. Rebuild application containers (`docker compose down && docker compose up -d`).
3. Run Full sync + detection re-run.
4. Open `/subscriptions` Pending tab and confirm rows; open header Alerts panel.
5. Repeat rebuild + Full sync and observe confirmed patterns.

**expected:**

- **Defect AU (confirmed stability):** Operator-confirmed subscriptions remain **confirmed** after container rebuild + Full sync — not re-surfaced as **pending** with Confirm/Reject buttons.
- **Defect AV (duplicate detection):** Detection re-run does not create duplicate pending rows for the same merchant identity; confirmed fingerprints are skipped in detection.
- **Defect AW (alert dedup):** Subscription-scoped unread alerts reconcile with list tabs — no spurious `new_detection` unread forcing re-review of already-confirmed merchants.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect AU (confirmed stability):** After rebuild + Full sync, confirmed patterns (CURSOR, APPLE.COM/BILL) re-appear as **pending** in `/subscriptions` Pending tab with Confirm/Reject buttons.
- **Defect AV (duplicate detection):** Detection re-run creates duplicate pending rows for already-confirmed merchant identities.
- **Defect AW (alert dedup):** Header Alerts panel shows **new_detection** unread forcing re-review of already-confirmed merchants.

**evidence_refs:** `/subscriptions` Pending tab screenshot post-rebuild (confirmed rows re-appeared); `GET /api/v1/subscriptions` response showing duplicate rows for confirmed patterns; alert unread count vs list reconciliation.

**Related work:** **BUG-0008** alert/list mismatch; **US-0003** detection; subscription detection engine pattern fingerprinting.

#### Intake evidence (BUG-0015)

- `intake_run_id`: `intake-20260625-confirmed-subscriptions-regression`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Detection re-run does not skip confirmed fingerprints; confirmed pattern DB rows not joined correctly during detection; alert evaluation does not exclude confirmed patterns.

**Decomposition:** single-bug — confirmed subscription persistence + alert dedup share detection engine fingerprint handling. Fixed together.

---

### BUG-0016 — Client routes return 404 on hard refresh / direct navigation

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0025, 2026-06-25 — SPA routing + Traefik SPA fallback verified on omniflow

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; Traefik `auth` middleware; localhost override `:18080`.

**steps_to_reproduce:**

1. Open `https://financegnome.omniflow.cc/forecast` directly (not via sidebar navigation).
2. Open `https://financegnome.omniflow.cc/subscriptions`, `/planning`, `/sync`, `/analytics/cashflow` similarly.
3. On any opened route, press Ctrl+Shift+R (hard refresh).
4. Bookmark a route, close tab, reopen via bookmark.

**expected:**

- **Defect AX (SPA routing):** Direct navigation, hard-refresh, and bookmarks to client routes return **HTTP 200** with SPA shell and render the correct React page — not **404** with blank body.
- Applies on both localhost override (`:18080`) and US-0010 external profile omniflow host.

**actual:**

- Direct navigation to `/forecast`, `/subscriptions`, `/planning`, `/sync`, `/analytics/{slug}` returns **HTTP 404** with blank body.
- Hard refresh on a valid client route returns **404**.
- Bookmarked routes return **404** on reopen.

**evidence_refs:** HTTP 404 response for `https://financegnome.omniflow.cc/forecast`; localhost `:18080/forecast` 404; Traefik routing config.

**Related work:** **US-0010** external Traefik deployment; SPA build output; `index.html` fallback.

#### Intake evidence (BUG-0016)

- `intake_run_id`: `intake-20260625-spa-routing-404`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Traefik serving static SPA files; requires `try_files $uri /index.html` fallback; static asset caching may interfere.

**Decomposition:** single-bug — SPA routing fallback required on omniflow Traefik config + static asset serving.

---

### BUG-0017 — Post-sync recompute fails on DB constraints (forecast audit violations)

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0026, 2026-06-25 — audit + recompute + planning badge + ML skip message fixed

**environment:** US-0010 external Compose profile; Full sync complete; `FORECAST_ML_ENABLED=true` configured; `/forecast` page; `/planning` Compare tab.

**steps_to_reproduce:**

1. After Full sync, inspect `GET /api/v1/ai/audit?tool_name=forecast_bucket_assignment`.
2. Inspect `GET /api/v1/ai/audit?result_status=low_confidence` (or equivalent).
3. Trigger manual Full sync again; observe `GET /api/v1/forecast/meta` updates.
4. Open `/planning` Compare tab and observe Plan stale badge / `GET /api/v1/forecast/monthly`.
5. Open `/forecast` and observe "No forecast data yet" state when `computation_id` is set.

**expected:**

- **Defect AY (audit tool_name):** `GET /api/v1/ai/audit` returns rows for `forecast_bucket_assignment` without `ai_tool_audit_tool_name_check` violation.
- **Defect AZ (audit result_status):** Low-confidence bucket assignments persist without `ai_tool_audit_result_status_check` violation.
- **Defect BA (recompute fkey):** Recompute delete/insert does not fail on `forecast_computations_paired_baseline_id_fkey`; `GET /api/v1/forecast/meta` reflects fresh computation after successful sync.
- **Defect BB (ML availability):** When `FORECAST_ML_ENABLED` and history gate pass, ML-enhanced forecast selectable; otherwise accurate `ml_skipped_reason` (not silent failure from recompute errors).
- **Defect BC (plan badge):** Planning Compare loses **Plan stale** badge after successful recompute/sync.
- **Defect BD (no data yet):** `/forecast` does not show **No forecast data yet** when `GET /api/v1/forecast/meta` already has `computation_id`.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect AY (audit tool_name):** `GET /api/v1/ai/audit` returns `ai_tool_audit_tool_name_check` violation for `forecast_bucket_assignment` rows.
- **Defect AZ (audit result_status):** Low-confidence bucket assignments rejected with `ai_tool_audit_result_status_check` violation.
- **Defect BA (recompute fkey):** Manual Full sync fails with `violates foreign key constraint forecast_computations_paired_baseline_id_fkey`; `GET /api/v1/forecast/meta` stale after sync.
- **Defect BB (ML availability):** `ml_skipped_reason` returns incorrect failure reason when ML actually available (silently broken from recompute errors).
- **Defect BC (plan badge):** Plan stale badge persists after successful recompute/sync.
- **Defect BD (no data yet):** `/forecast` shows **No forecast data yet** state despite `computation_id` set in API response.

**evidence_refs:** `GET /api/v1/ai/audit` response body + violation errors; sync logs showing fkey failure; `GET /api/v1/forecast/meta` response; `/planning` Compare tab screenshot with persistent stale badge.

**Related work:** **BUG-0012** bucket mapping; **US-0002** forecast recompute; `forecast_computations` schema; AI audit contract.

#### Intake evidence (BUG-0017)

- `intake_run_id`: `intake-20260625-recompute-audit-constraints`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Audit constraint violations block bucket assignment rows; fkey failure from non-idempotent delete/insert during recompute; ML skip message reads wrong state flag; plan stale badge computed from stale metadata; "No forecast data" UI checks response shape not `computation_id`.

**Decomposition:** single-bug cluster — post-sync recompute + audit constraint + UI state all share the same recompute pipeline. Fixed together.

---

### BUG-0018 — Post-sync alert evaluation SQL error (balance ambiguous)

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0027, 2026-06-25 — alert evaluation SQL + header alerts panel verified

**environment:** US-0010 external Compose profile; Full sync complete; header Alerts panel (bell icon); `/subscriptions` alerts.

**steps_to_reproduce:**

1. Run Full sync (922+ transactions).
2. Observe backend logs for `alert evaluation failed` errors.
3. Open header Alerts panel (bell icon); observe alert count + content.
4. `GET /api/v1/subscriptions/alerts?scope=unread` and inspect response.

**expected:**

- **Defect BE (SQL evaluation):** Post-sync alert evaluation completes without SQL error (`balance` ambiguous / **42702**); logs show no `alert evaluation failed` for normal sync runs.
- **Defect BF (alerts surface):** Header Alerts panel and `GET /api/v1/subscriptions/alerts` surface matching alerts when overdraft or subscription rules apply — not permanent **No active alerts** due to evaluation skip.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect BE (SQL evaluation):** Backend logs show repeated `alert evaluation failed` with `SQLSTATE 42702: column reference "balance" is ambiguous`.
- **Defect BF (alerts surface):** Header Alerts panel permanently shows **No active alerts**; `GET /api/v1/subscriptions/alerts` returns `[]` even when overdraft/subscription thresholds should fire.

**evidence_refs:** Backend logs showing SQL error; `GET /api/v1/subscriptions/alerts` empty response; header Alerts panel screenshot.

**Related work:** **US-0005** alerts; **BUG-0008** alert/list mismatch; alert evaluation SQL.

#### Intake evidence (BUG-0018)

- `intake_run_id`: `intake-20260625-alert-evaluation-sql`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `balance` column ambiguous in alert evaluation JOIN; table-qualifier required; alert evaluation silent-fail causes empty alerts panel.

**Decomposition:** single-bug — SQL column ambiguity fixes alert evaluation + panel rendering.

---

### BUG-0019 — Grafana cashflow zeros & platform health zero transactions count

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0028, 2026-06-26 — cashflow panels + platform health transaction count verified

**environment:** US-0010 external Compose profile; Full sync complete; `/analytics/cashflow` (account 114); Platform Health dashboard.

**steps_to_reproduce:**

1. After Full sync + recompute, open `/analytics/cashflow` and set `account_id=114`.
2. Inspect `GET /api/v1/forecast/monthly?account_id=114` and compare to Grafana panels.
3. Open Platform Health dashboard and inspect **Records synced per entity** panel for `transactions`.

**expected:**

- **Defect BG (cashflow panels):** Analytics → Cashflow **balance forecast** and **recent daily balances** show non-zero signed values for funded account **114** when `GET /api/v1/forecast/monthly?account_id=114` has non-zero series — not flat **0** regression.
- **Defect BH (platform health):** Platform Health **Records synced per entity** reports **transactions** count matching mirror (`SELECT COUNT(*) FROM transactions`) after successful Full sync — not **0** when 900+ rows exist.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect BG (cashflow panels):** Cashflow Grafana panels flat **0** despite API forecast non-zero (BUG-0013 regression re-manifested).
- **Defect BH (platform health):** Platform Health shows **transactions: 0** despite 900+ rows in mirror (`SELECT COUNT(*) FROM transactions` returns 922).

**evidence_refs:** `/analytics/cashflow` screenshot (account 114); `GET /api/v1/forecast/monthly?account_id=114` response; platform health panel `transactions: 0` screenshot; `SELECT COUNT(*) FROM transactions` query.

**Related work:** **BUG-0013** prior analytics regression; Grafana provisioning; platform health query.

#### Intake evidence (BUG-0019)

- `intake_run_id`: `intake-20260626-grafana-zeros-platform-health`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Cashflow Grafana panels use stale forecast snapshot cache; platform health query for `transactions` count uses wrong table or schema; both fixed independently.

**Decomposition:** single-bug cluster — Grafana cashflow + platform health count share sync recompute data path. Fixed together.

---

### BUG-0020 — Subscriptions list quality (duplicates, uncategorized)

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0029, 2026-06-27 — duplicate detection fixed; majority category surfaced; subscription listing quality improved

**environment:** US-0010 external Compose profile; Full sync complete; `/subscriptions` All tab and Confirmed tab.

**steps_to_reproduce:**

1. Open `/subscriptions` **All** tab after Full sync.
2. Observe rows for known merchants (e.g. **Strom**, **YouTube**, **CURSOR**).
3. Open **Confirmed** tab and inspect `display_category` column.
4. Run detection re-evaluation; confirm a candidate pattern.

**expected:**

- **Defect BI (duplicate rows):** `/subscriptions` **All** tab shows at most one visible row per logical confirmed merchant identity — no triplicate Strom / duplicate YouTube rows for same payee_key.
- **Defect BJ (display category):** Confirmed subscriptions show **display category** from US-0020 majority-category contract when mirror txs have category assignments — `display_category_id` non-null on representative samples; not all **Uncategorized** when categories exist.
- Discover, tags, and detection regression pass.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect BI (duplicate rows):** `/subscriptions` **All** tab shows triplicate **Strom** rows, duplicate **YouTube** rows for same payee_key — detection creates multiple subscription rows for same merchant identity.
- **Defect BJ (display category):** All confirmed subscriptions show **Uncategorized** even when mirror transactions hold `category_id` assignments.

**evidence_refs:** `/subscriptions` All tab screenshot (triplicate Strom, duplicate YouTube); Confirmed tab screenshot (all Uncategorized); `GET /api/v1/subscriptions` response body; `GET /api/v1/subscriptions?status=confirmed` response.

**Related work:** **US-0003** detection; **US-0020** majority category; detection engine `detect_recurrence_groups`.

#### Intake evidence (BUG-0020)

- `intake_run_id`: `intake-20260627-subscription-list-quality`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Deduplication requires payee_key fingerprint join; majority category requires category_id modal aggregation; both surface defects fixed together.

**Decomposition:** single-bug — duplicate detection + majority category surface both required for subscription listing quality.

---

### BUG-0021 — Frontend UX polish (category filter delay, wealth role column)

Status: DONE
Priority: P2

**closure_note:** verify-work PASS Q0030, 2026-06-27 — CategoryFilter eager load + wealth display category COALESCE verified

**environment:** US-0010 external Compose profile; `/forecast` Monthly tab and `/wealth` Overview; localhost:18080 + omniflow.

**steps_to_reproduce:**

1. Open `/forecast` Monthly tab; observe **Loading category filter…** Suspense fallback duration.
2. Open `/wealth` Overview; click Account breakdown table and observe **Role** column values.
3. Inspect backend `GET /api/v1/wealth` response; `accounts[].account_role` field.

**expected:**

- **Defect BK (CategoryFilter delay):** CategoryFilter on Forecast Monthly and Wealth Overview becomes interactive within **~1s** of tab visit — not multi-second **Loading category filter…** Suspense fallback under normal local/omniflow load.
- **Defect BL (Role column):** Wealth account breakdown **Role** column shows Firefly account role/type when mirror provides it, or column is hidden/documented when unsupported — not permanent em dash for all rows.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect BK (CategoryFilter delay):** CategoryFilter takes **3–5 seconds** to appear; multi-second **Loading category filter…** Suspense fallback visible.
- **Defect BL (Role column):** Wealth Account breakdown **Role** column shows em dash **—** for every account (Cash wallet, Giro, savings).

**evidence_refs:** `/forecast` Monthly tab screenshot (filter loading state); `/wealth` Overview screenshot (Role column em dash); `GET /api/v1/wealth` response body; CategoryFilter lazy import timing.

**Related work:** **US-0018** CategoryFilter; `CategoryFilter.tsx` lazy load; `wealth/repository.rs` account_role join; **DEC-0110** lazy chunk policy; **DEC-0111** account_role COALESCE.

#### Intake evidence (BUG-0021)

- `intake_run_id`: `intake-20260627-frontend-ux-polish`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: CategoryFilter delay from `React.lazy` chunk waterfall; `account_role` requires COALESCE of `payload->'attributes'->>'account_role'` and root-level fallback.

**Decomposition:** single-bug — two independent frontend polish items; CategoryFilter eager import + wealth role COALESCE.

---

### BUG-0022 — Plan delete still broken (selector ignores dropdown)

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0031, 2026-06-28 — plan delete enabled per DEC-0082; localhost + BI browser verified; omniflow operator-deferred

**environment:** US-0010 external Compose profile; localhost:18080; `/planning` dropdown + delete; multiple plans exist.

**steps_to_reproduce:**

1. Create two plans: make one globally **active**, one non-active.
2. On `/planning` Scenarios tab, select the **non-active** plan in the plan dropdown.
3. Observe **Delete plan** button state; attempt to click it.
4. Switch back to the globally active plan; observe **Delete plan** button state.

**expected:**

- **Defect BM (non-active delete):** With two or more plans on `/planning`, selecting a **non-active** plan in the dropdown enables **Delete plan**; confirmation removes the plan and refreshes the list — not a permanently disabled delete control.
- **Defect BN (active delete blocked):** Deleting the globally **active** plan remains blocked in UI (disabled + tooltip) and via `DELETE /api/v1/plans/:id` (**409** per **DEC-0082**).
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect BM (non-active delete):** Selecting a non-active plan leaves **Delete plan** permanently disabled — button never activates.
- **Defect BN (active delete blocked):** Confirmed working as designed (409 on active plan delete) — no defect here, but UI lacks explanation for lone-active-plan state.
- `planSelector.test.ts` 8/8 on localhost; Q0031 verify-work confirmed fix; omniflow **FRONTEND_DEPLOY** pending.

**evidence_refs:** `/planning` Scenarios screenshot (delete button disabled); Network tab `DELETE /api/v1/plans/:id` responses; `planSelector.test.ts` vitest results; Q0031 verify-work.

**Related work:** **BUG-0014** AS planning delete; **DEC-0082** active plan delete contract; `PlanningPage.tsx` selector + delete wiring.

#### Intake evidence (BUG-0022)

- `intake_run_id`: `intake-20260627-plan-delete-regression`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Plan selector state update not propagating to delete affordance; DEC-0082 active-plan blocking works correctly; BN tooltip improvement deferred per DEC-0082 docs.

**Decomposition:** single-bug — plan selector + delete affordance wiring.

---

### BUG-0023 — Crypto Wealth EUR values missing (live regression)

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0032, 2026-06-27 — Bitunix wallet parse + entryValue display + crypto subtotal + PnL return surface verified; BUG-0024 extended plan-delete UX and sole-plan guidance

**environment:** US-0010 external Compose profile; Bitunix connected with linear positions; `GET /api/v1/wealth` endpoint; `/wealth` Crypto tab.

**steps_to_reproduce:**

1. Connect Bitunix exchange (read-only API keys) with open linear positions (no spot).
2. Run exchange sync + PnL recompute.
3. Open `/wealth` Crypto tab; observe Bitunix exchange card and holdings rows.
4. `GET /api/v1/wealth` and inspect `crypto.subtotal_eur`, `holdings_all[].value_eur`, and `pnl.total_return_pct`.

**expected:**

- **Defect BO (Bitunix card):** Wealth **Crypto** tab Bitunix card and `GET /api/v1/wealth` `crypto.subtotal_eur` reflect operator portfolio value (~**€2000** order of magnitude per operator Bitunix app) — not **€0** / **€-0,00** with **11** open linear positions.
- **Defect BP (holdings Value EUR):** Holdings table **Value EUR** column shows EUR equivalent at valuation time for linear positions when prices available — not all **—** while **Native qty** and **Unrealized PnL** are populated.
- **Defect BQ (Total return %):** PnL summary **Total return %** populated when baseline exists after exchange sync + PnL recompute — not **—** while unrealized EUR is non-zero.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect BO (Bitunix card):** Bitunix exchange card shows **€ -0,00**; `crypto.subtotal_eur = -0.0` despite 11 open linear positions.
- **Defect BP (holdings Value EUR):** Holdings table **Value EUR** column shows **—** for all 11 rows.
- **Defect BQ (Total return %):** PnL summary **Total return %** shows **—** while `unrealized_eur = 376.83`.

**evidence_refs:** `/wealth` Crypto tab screenshot (BO/BP/BQ); `GET /api/v1/wealth` response body; Bitunix wallet API response (no `futures` wallet row).

**Related work:** **BUG-0014** AP/AQ; **DEC-0064** linear pricing; **DEC-0080** crypto subtotal; **DEC-0081** PnL return surface.

#### Intake evidence (BUG-0023)

- `intake_run_id`: `intake-20260627-crypto-wealth-eur`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Bitunix wallet API returns `product_type=linear` for open positions (no futures wallet row); `DEC-0064` sets `market_value_eur: None` for linear by design; linear display requires `entryValue` fallback; subtotal = `sum(market_value_eur)` wallet-only per DEC-0080; total return numerator = unrealized_eur + realized_eur, denominator = wallet-priced crypto_value_eur.

**Decomposition:** single-bug cluster — Bitunix wallet parse + linear entryValue display + subtotal + total return surface all share the crypto valuation path.

---

### BUG-0024 — Plan delete still disabled (live operator report post-Q0031)

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0033, 2026-06-28 — plan delete UX + sole-plan hint fixed; Q0034 verify-work added category-spending trend fix for BUG-0025

**environment:** US-0010 external Compose profile; `/planning` dropdown + delete; localhost:18080 + omniflow; operator report 2026-06-28.

**steps_to_reproduce:**

1. Create two plans: make one globally **active**, one non-active.
2. On `/planning` Scenarios tab, select the **non-active** plan in the plan dropdown.
3. Observe **Delete plan** button state; attempt to click.
4. Switch back to the globally active plan (sole plan); observe **Delete plan** button state and tooltip/instruction copy.

**expected:**

- **Defect BR (non-active delete):** With **two or more** plans on `/planning`, selecting a **non-active** plan in the dropdown enables **Delete plan** and successful confirmation removes the plan — not permanently disabled after **Q0031** deploy.
- **Defect BS (sole active plan guidance):** With **only one** globally active plan, **Delete plan** remains disabled per **DEC-0082** but UI shows **clear explanation** (tooltip or inline copy) describing how to delete (e.g. create another plan, set it active, then delete the original) — not silent gray button with no guidance.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect BR (non-active delete):** After Q0031 frontend deploy, non-active plan delete still disabled on omniflow (`assets/` bundle stale).
- **Defect BS (sole active plan guidance):** With sole active plan, delete button permanently gray with no explanation.
- Q0033 verify-work confirmed fix; **FRONTEND_DEPLOY** pending on omniflow.

**evidence_refs:** `/planning` screenshot (delete disabled); Q0033 verify-work; `planSelector.test.ts` vitest 31/31.

**Related work:** **BUG-0022** plan delete (Q0031); **DEC-0082** active plan delete contract; `PlanningPage.tsx` selector + delete wiring.

#### Intake evidence (BUG-0024)

- `intake_run_id`: `intake-20260628-plan-delete-remediation`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: BR requires fresh frontend deploy on omniflow; BS requires new sole-plan inline hint + DEC-0082 tooltip expansion.

**Decomposition:** single-bug — plan delete UX extension from BUG-0022 remediation.

---

### BUG-0025 — Firefly category transactions not syncing across months (Stromkosten live example)

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0034, 2026-06-28 — category trend multi-month fix verified; **FRONTEND_DEPLOY + BACKEND REBUILD** pending on omniflow

**environment:** US-0010 external Compose profile; operator added multiple Firefly transactions to category **Wohnen - Stromkosten** across multiple months; `/forecast` Category spending trend chart.

**steps_to_reproduce:**

1. Add multiple Firefly transactions to category **Wohnen - Stromkosten** across multiple months (e.g. Jan 2026, Feb 2026, Mar 2026).
2. Run manual Full Firefly sync.
3. Open `/forecast` Category spending trend chart and select category **Wohnen - Stromkosten**.
4. Observe monthly bars for each month with transactions.
5. `GET /api/v1/categories/expense-series?category_id=<Wohnen - Stromkosten>` and inspect monthly bars.

**expected:**

- **Defect BW (multi-month trend):** `/forecast` Category spending trend (and `GET /api/v1/categories/expense-series`) shows outflow bars for **each month** with data — not only 2026-05 with all prior months at **€0** while Firefly holds Stromkosten rows in those months.
- **Defect BX (backdated/bulk ingest):** Backdated or bulk-imported Firefly transactions ingest on next appropriate sync **or** Sync Status / operator docs explain the **DEC-0002** overlap limitation and remediation (cursor reset / full re-sync) — not silent omission with misleading success.
- **Defect BY (sync history type):** Manual **Sync now** on Sync Status triggers **Full** Firefly ingest; sync history distinguishes Firefly full runs (`manual`, `scheduled`) from exchange-only (`scheduled_exchanges`, `manual_exchanges`).
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect BW (multi-month trend):** Only **2026-05** shows non-zero bar; all prior months show **€0** despite Firefly holding Stromkosten rows.
- **Defect BX (backdated/bulk ingest):** Transactions outside current sync window silently omitted; no documented DEC-0002 overlap limitation in UI.
- **Defect BY (sync history type):** Sync history does not distinguish Firefly full vs exchange-only runs.

**evidence_refs:** `/forecast` category trend screenshot (only 2026-05 bar); `GET /api/v1/categories/expense-series` response body; operator Firefly Stromkosten transaction history.

**Related work:** **US-0018** category trend API; **DEC-0002** sync overlap limitation; [R-0062](docs/engineering/research.md#r-0062--category-sync-window-multiple-months).

#### Intake evidence (BUG-0025)

- `intake_run_id`: `intake-20260628-stromkosten-sync-across-months`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Category series endpoint uses Firefly sync window which does not span multiple historical months; DEC-0002 overlap docs missing from operator UX; manual Full sync should pull historical transactions.

**Decomposition:** single-bug — category trend API surface + sync history labels both needed for operator clarity.

---

### BUG-0026 — Forecast monthly Income card 0.00 while chart shows income bars

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0035, 2026-06-28 — summary card month label + consistency with chart verified; FRONTEND_DEPLOY pending on omniflow

**environment:** US-0010 external Compose profile; `/forecast` Monthly tab; account 114 with salary income projected from month 2.

**steps_to_reproduce:**

1. Open `/forecast` Monthly tab (account 114).
2. Observe summary Income card value.
3. Compare Income card value to Monthly bar chart income bars.
4. Inspect summary card labels and month reference.

**expected:**

- **Defect BZ (card/chart consistency):** Summary **Income** card is **consistent** with the monthly bar chart for the **same labeled reference month** — not **0.00** on the card while the chart shows non-zero income bars (~€3000) for visible forecast months without explaining the mismatch.
- **Defect CA (month reference):** Summary cards show **which month** they represent (or use current/next full-month semantics documented in UI) — not unlabeled values from `series[0]` when that month differs from operator-expected forecast view.
- OIDC-enabled deploy regression checks pass.

**actual:**

- **Defect BZ (card/chart consistency):** Income card shows **0.00** while chart shows **~€3266** income bars for months with projected salary.
- **Defect CA (month reference):** Summary cards show metric labels only (Income, Fixed, Variable, Free) with no month reference — operator cannot reconcile card vs chart.

**evidence_refs:** `/forecast` Monthly tab screenshot (Income card 0.00 vs chart ~€3266); `GET /api/v1/forecast/monthly?account_id=114` response body (`series[0].income=0.00`, `series[1].income=3266.16`).

**Related work:** **BUG-0012** forecast bucket mapping; **DEC-0007** forecast projection; `ForecastPage.tsx` monthlySummary card logic.

#### Intake evidence (BUG-0026)

- `intake_run_id`: `intake-20260628-forecast-income-card-mismatch`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: Summary card uses `series[0]` (current partial month) which may have no salary due in remaining days; chart shows full forecast series; card needs to use next full month or first month with non-zero income when current partial month is zero.

**Decomposition:** single-bug — summary card month selection + label semantics.

---

### BUG-0027 — Firefly sync fails with 401 Unauthorized (PAT invalid/expired after deploy)

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0036, 2026-06-28 — PAT 401 diagnosis + PAT regeneration smoke + sync operational verified

**environment:** US-0010 external Compose profile; FIREFLY_PERSONAL_ACCESS_TOKEN configured; rebuild deployed ~2026-06-25; omniflow `financegnome.omniflow.cc`.

**steps_to_reproduce:**

1. After rebuild + deploy, run Full Firefly sync manually (or wait for scheduled run).
2. Inspect `GET /api/v1/sync/status`.
3. Inspect sync logs for PAT-related errors.

**expected:**

- **Defect CB (sync success after PAT regen):** After regenerating Firefly PAT + updating `FIREFLY_PERSONAL_ACCESS_TOKEN` in `.env` + container recreate, `GET /api/v1/sync/status` shows `state: completed` after next scheduled run; manual "Sync now" returns `status: completed` with non-zero entity counts; no `error_message` containing `401` or `Unauthorized`. Live Firefly mirror reflects current data.
- **Defect CC (PAT diagnosis UX):** App surfaces clear user-facing diagnosis on `/sync` when PAT auth fails (distinguishing "PAT expired/invalid" from "PAT missing" and "Firefly unreachable") — operator knows to regenerate.
- **Defect CD (PAT regression):** Subsequent scheduled syncs succeed across ≥3 cycles (no silent 401 recurrence) after PAT regen.
- OIDC-enabled and omniflow external deploy regression checks pass.

**actual:**

- **Defect CB (sync fail):** Sync fails with `error_message: unexpected status 401 Unauthorized`; `GET /api/v1/sync/status` shows `state: failed`.
- **Defect CC (diagnosis UX):** `/sync` shows generic `unexpected status 401 Unauthorized` without distinguishing PAT missing vs expired vs Firefly unreachable.
- `FIREFLY_PERSONAL_ACCESS_TOKEN` in `.env` = **980 chars** (non-empty); Firefly returns **302** redirecting to `/login` for API calls with invalid PAT.

**evidence_refs:** `GET /api/v1/sync/status` response body; `/sync` page error message screenshot; Firefly `/api/v1/about` curl response (302 → `/login`); `.env` PAT value length verification.

**Related work:** **BUG-0002** prior sync 404; [R-0057](docs/engineering/research.md#r-0057--firefly-pat-contract).

#### Intake evidence (BUG-0027)

- `intake_run_id`: `intake-20260628-firefly-pat-401`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: PAT 980 chars suggests expired token (not missing); Firefly returns 302→/login for API call with invalid PAT; sync layer sees 401 upstream; diagnosis UX needs to disambiguate PAT missing vs expired vs Firefly unreachable.

**Decomposition:** single-bug — PAT 401 diagnosis + sync failure + operator remediation UX.

---

## User stories (canonical)

### US-0001 — Self-hosted platform foundation & Firefly read-only integration

Status: DONE
Priority: P0

As a household budget operator running self-hosted Flow Finance AI,
I want to connect to my Firefly III instance read-only and sync accounts, transactions, categories, budgets, tags, and piggy banks,
So that I have a local mirror of my personal finance data without exposing it to third-party cloud services.

#### Scope

- In: Docker Compose service `flow-finance-ai` alongside `firefly-iii` and `grafana`
- In: External PostgreSQL connection configurable via TOML/env
- In: Rust backend with health endpoint + authenticated API skeleton
- In: React UI shell with OIDC auth flow (login redirect + session)
- In: Firefly Connector: accounts, transactions, categories, budgets, tags, piggy banks via `/api/v1`
- In: Sync scheduler with configurable interval + manual trigger
- In: Read-only Firefly III API access (no write operations)
- Out: Analytics dashboards; forecasting; subscriptions; alerts; AI chat; planning; crypto

#### Intake decomposition

Single story — foundational deployment + Firefly read-only sync are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260603-platform-foundation`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260603-platform-foundation.json`

---

### US-0002 — Cashflow forecasting & Grafana analytics (MVP)

Status: DONE
Priority: P0

As a budget operator,
I want to see tomorrow/next-week/month-end projected balance, daily/monthly/long-term forecasts, and Grafana Cashflow + Forecast Horizons dashboards,
So that I know when money will run out and can plan ahead.

#### Scope

- In: Daily forecast (tomorrow, next week, month-end) per account
- In: Monthly forecast (income, fixed costs, variable costs, free cashflow)
- In: Long-term forecasts for 3/6/12/24 months
- In: TimescaleDB hypertables for forecast persistence
- In: React ECharts forecast charts for selected account + horizon
- In: Grafana Dashboard 1 (Cashflow: balance, forecast, scarcity markers) + Grafana Dashboard 5 (Forecast horizons)
- In: Recompute after Firefly sync completes
- Out: ML-enhanced forecasts (US-0009); budget plan-vs-Ist (US-0004); subscription-aware forecast (US-0003); planning what-ifs (US-0004)

#### Intake decomposition

Single story — forecast + Grafana dashboards are one vertical slice sharing projection engine.

#### Intake evidence

- `intake_run_id`: `intake-20260603-cashflow-forecasting`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260603-cashflow-forecasting.json`

---

### US-0003 — Subscription detection, price changes & alerts

Status: DONE
Priority: P1

As a subscription-heavy household budgeter,
I want my recurring monthly charges automatically detected, with price-change alerts,
So that I can identify forgotten subscriptions and cancel them to save money.

#### Scope

- In: Detection engine identifying recurring patterns with confidence score (95/80/60% tiers)
- In: User notification for new subscription with confirm/reject actions
- In: Confirmed subscriptions appear in list with interval and amount
- In: Standing-order (Dauerauftrag) detection separate from discretionary subscriptions
- In: Price increase/decrease detection for confirmed subscriptions
- In: Alert fired on new subscription detection and on price change
- In: Grafana Dashboard 2 (Subscriptions, price changes, new)
- In: Rejected patterns excluded from forecasts + alerts
- Out: AI-suggested cancellation savings (US-0006); subscription budget what-ifs (US-0004); manual tagging (US-0020)

#### Intake decomposition

Single story — detection engine + alert pipeline are one vertical slice sharing pattern-matching logic.

#### Intake evidence

- `intake_run_id`: `intake-20260603-subscription-detection`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260603-subscription-detection.json`

---

### US-0004 — Financial planning, scenarios & plan-vs-actual

Status: DONE
Priority: P1

As a household budgeter,
I want to create financial plans with scenarios (current, leasing, savings mode, house purchase), version them, and compare plan-vs-actual daily,
So that I can test spending decisions against my financial goals before committing.

#### Scope

- In: Named plans with scenario adjustments (e.g. +300 €/month leasing)
- In: Built-in templates: current (Ist), leasing, savings mode, house purchase
- In: Plan versions (v1, v2, v3) with side-by-side comparison
- In: Daily plan-vs-Ist comparison: planned, actual, deviation
- In: Grafana Dashboard 3 (Budget plan/ist/deviation) reflects active plan
- Out: Scenario changes never modify Firefly transaction data
- Out: ML-enhanced scenario projections (US-0009); category-scoped what-ifs (US-0019); planning UX polish (US-0014)

#### Intake decomposition

Single story — planning + scenarios + plan-vs-actual are one vertical slice sharing plan schema.

#### Intake evidence

- `intake_run_id`: `intake-20260603-financial-planning`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260603-financial-planning.json`

---

### US-0005 — Wealth analysis, budget drift & scarcity alerts

Status: DONE
Priority: P0

As a household budgeter,
I want a net worth view aggregating my linked accounts, plus scarcity alerts when projected balance drops and budget drift alerts when category spend exceeds plan,
So that I have a complete picture of where I stand financially and can act before going over budget.

#### Scope

- In: Net worth view aggregating Firefly-linked accounts (giro, savings, etc.)
- In: Scarcity alert when projected balance falls below configurable threshold (e.g. 200 €)
- In: Budget drift alert when category spending exceeds plan by configurable % (e.g. +20%)
- In: Plan viability alert when active scenario becomes infeasible per forecast
- In: Alert inbox in React UI with acknowledge/dismiss
- In: Grafana Dashboard 4 (total wealth, non-crypto until US-0007)
- Out: Crypto holdings in net worth (US-0007); ML-enhanced wealth forecast (US-0013); subscription price-aware budget drift (US-0003)

#### Intake decomposition

Single story — wealth aggregation + alert pipeline share account/projection surface.

#### Intake evidence

- `intake_run_id`: `intake-20260603-wealth-alerts`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260603-wealth-alerts.json`

---

### US-0006 — AI financial assistant with privacy-safe tool layer

Status: DONE
Priority: P1

As a household budgeter,
I want to ask my finances natural-language questions and get data-backed answers, with all AI interactions going through a privacy-safe tool layer,
So that I can quickly understand my spending without digging through dashboards, while keeping raw transaction data private.

#### Scope

- In: Chat UI accepting natural-language questions
- In: AI invoking only registered tools: `get_transactions`, `get_subscriptions`, `get_forecast`, `get_budget_status`, `get_portfolio`, `simulate_plan`
- In: OpenAI provider configurable via environment/config
- In: Privacy settings: `allow_raw_transactions`, `redact_iban`, `redact_counterparties` honored in tool responses
- In: Example queries: affordability check, subscription price changes, budget overrun explanation, savings from cancelling subscription, top spending categories
- In: Tool-call audit log for operator review
- Out: Local AI provider support (US-0008); ML-enhanced AI reasoning (US-0009); AI budget suggestions (US-0019)

#### Intake decomposition

Single story — chat UI + tool layer + audit log are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260603-ai-assistant`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260603-ai-assistant.json`

---

### US-0007 — Crypto exchange portfolio integration

Status: DONE
Priority: P1

As a crypto-investing household budgeter,
I want my crypto exchange holdings (Binance, Bybit, Bitunix) to be included in my net worth view, with portfolio performance tracking,
So that my personal finance picture reflects all of my assets, not just my bank accounts.

#### Scope

- In: Connectors for Binance, Bybit, and Bitunix importing balances, positions, trades, transfers, funding, and PnL
- In: Portfolio Engine: realized gains (closed positions), unrealized gains (open positions), total return
- In: Crypto holdings included in net worth view (extends US-0005)
- In: Portfolio allocation scenarios (e.g. 50% ETF / 50% crypto)
- In: Grafana Dashboard 4 crypto slice + portfolio performance
- In: API keys stored self-hosted (not transmitted externally)
- Out: AI crypto allocation suggestions; local AI provider support; ML portfolio forecast (US-0013)

#### Intake decomposition

Single story — exchange connectors + portfolio engine + net worth extension are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260603-crypto-portfolio`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260603-crypto-portfolio.json`

---

### US-0008 — Local & self-hosted AI provider support

Status: DONE
Priority: P1

As a privacy-focused household budgeter,
I want my AI assistant to work with local AI providers (Ollama, LM Studio, LocalAI, vLLM) in addition to OpenAI,
So that my financial questions are answered without sending transaction data to external APIs.

#### Scope

- In: Provider selector supporting OpenAI + local OpenAI-compatible endpoints
- In: Ollama integration working when Ollama service is running (full Docker Compose profile)
- In: LM Studio / LocalAI / vLLM via OpenAI-compatible base URL configuration
- In: US-0006 tool layer + privacy settings unchanged across providers
- In: Chat functionality verified end-to-end with local provider
- Out: ML forecast provider local support; privacy audit for local models

#### Intake decomposition

Single story — provider adapter + compose integration + e2e verification are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260603-local-ai-support`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260603-local-ai-support.json`

---

### US-0009 — Advanced forecasting with ML & risk assessment

Status: DONE
Priority: P1

As a sophisticated household budgeter,
I want my forecasts to detect seasonal patterns, use ML for 6–24 month projections with confidence bands, and assess portfolio risk,
So that long-term financial planning is realistic, includes uncertainty quantification, and accounts for portfolio performance.

#### Scope

- In: Seasonal patterns detection + application to monthly cashflow forecasts
- In: ML-enhanced forecast model producing 6–24 month projections with confidence bands
- In: Portfolio performance forecast when US-0007 data present
- In: Risk assessment score displayed for active plan scenarios
- In: Grafana Dashboard 5 ML forecast + risk panels
- In: UI ability to compare baseline vs ML-enhanced forecast
- Out: Local ML provider support; advanced risk model tuning; portfolio rebalancing suggestions (US-0019)

#### Intake decomposition

Single story — ML forecasting + risk assessment share stats-forecast service integration.

#### Intake evidence

- `intake_run_id`: `intake-20260603-advanced-forecasting`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260603-advanced-forecasting.json`

---

### US-0010 — External Firefly/Postgres & Traefik deployment on omniflow host

Status: DONE
Priority: P0

As a production operator deploying Flow Finance AI on omniflow,
I want the app to connect to my existing external Firefly III and Postgres instances, be routed through Traefik, and have an operator smoke test I can verify,
So that I run a real production deployment that coexists with my existing infrastructure.

#### Scope

- In: `docker compose --profile external up` that does **not** create `firefly-iii` or `postgres` containers
- In: `flow-finance-ai` joining external `traefik` network, reaching `postgres` and `firefly` by Docker DNS
- In: Traefik routing `https://financegnome.omniflow.cc` with `auth` middleware + valid TLS
- In: Backend `/health` returning success when wired to external DB + Firefly PAT configured
- In: `.env.example` documenting all operator-required variables for external mode
- In: Operator smoke test recorded (compose up + health + Traefik route check on Debian host)
- Out: Self-hosted Firefly profile (default); Grafana public analytics host

#### Intake decomposition

Single story — external compose profile + Traefik routing + DB/PAT connection + smoke test are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260606-external-deployment`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260606-external-deployment.json`

---

### US-0011 — Unified analytics UI in financegnome (Grafana in-app)

Status: DONE
Priority: P1

As a finance operator,
I want to access all Grafana dashboards through the financegnome sidebar without leaving the app,
So that my analytics UX is embedded and consistent instead of scattered across external Grafana tabs.

#### Scope

- In: Sidebar **Analytics** section listing every provisioned Grafana dashboard with in-app routes under financegnome host
- In: Opening dashboards without `target=_blank` Grafana tab for default flows
- In: Embedded or proxied views working with omniflow Traefik + `auth` middleware (documented operator auth-off dev path)
- In: Existing React chart pages (Forecast, Wealth, Planning, Subscriptions, Alerts) remain functional
- In: Wealth page no longer relying on external-only Grafana link as primary analytics entry
- In: Future-chart guideline: new product charts added inside financegnome shell (React + API default; Grafana embed exception list)
- In: Operator guide describing single-URL analytics UX (optional separate Grafana public host not required)
- Out: Embedding individual Grafana panels in React pages; analytics UX parity with external Grafana navigation

#### Intake decomposition

Single story — sidebar analytics + embed + operator guide are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260608-analytics-in-app`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260608-analytics-in-app.json`

---

### US-0012 — Auto-provision application database on first start

Status: DONE
Priority: P1

As an operator deploying on a shared Postgres instance,
I want the app to auto-create the application database on cold start if it doesn't exist,
So that I can deploy on omniflow's shared Postgres without manually running `CREATE DATABASE`.

#### Scope

- In: On cold start against external Postgres, if `DATABASE_NAME` is missing, backend creates the database idempotently before migrations
- In: Existing `flow_finance_ai` database is never dropped or recreated by bootstrap
- In: `CREATE EXTENSION IF NOT EXISTS timescaledb` attempted on app database when server provides; otherwise fails with actionable log
- In: Optional `DATABASE_BOOTSTRAP_URL` (or equivalent) for limited-privilege app role; fails closed with clear error when missing
- In: `.env.example` and runbook documenting bootstrap env vars + omniflow shared-Postgres behavior
- In: Automated test or CI fixture proving create-if-missing path (mock or testcontainers)
- Out: Schema migrations auto-run; backup/restore tooling; managed Postgres service support

#### Intake decomposition

Single story — database bootstrap is one vertical slice (bootstrap code + env docs + CI fixture).

#### Intake evidence

- `intake_run_id`: `intake-20260608-db-bootstrap`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260608-db-bootstrap.json`

---

### US-0013 — Production ML forecast & wealth analytics hardening

Status: DONE
Priority: P0

As a finance operator deploying ML-enhanced forecasting and ML portfolio overlay on omniflow,
I want `stats-forecast` started automatically, ML skip reasons accurate, ML forecast computations persisted with p10/p90 bands, React UI showing degraded copy when sidecar unavailable, and wealth ML overlay integrated,
So that production ML forecasting and portfolio overlay work with accurate degraded fallback.

#### Scope

- In: External compose overlay adding `stats-forecast` on `external` profile; env vars + port remap documented
- In: Backend `[forecast_ml] enabled=true` on external merge; sidecar health probe before sync ML phase
- In: Post-sync `forecast_ml` phase runs after baseline; ML failure records skip metadata without breaking sync
- In: `model_kind=ml_enhanced` computations persisted with p10/p90 bands; `GET /api/v1/forecast?variant=ml_enhanced` returns non-empty 6–24 month series
- In: React `/forecast` Compare control showing baseline + ML-enhanced overlay; degraded copy `sidecar_disabled` per DEC-0066
- In: Wealth API/UI integrating ML portfolio overlay; signed totals + account breakdown preserved; `portfolio_forecast_low_confidence` banner per R-0034
- In: Grafana forecast-horizons ML panels returning data when `$forecast_variant=ml_enhanced`
- In: Operator runbook for omniflow ML enablement: compose profile union, env vars, sidecar health probe, minimum history gate, degraded-mode troubleshooting
- In: Automated test or CI fixture proving sidecar invoke + overlay persist path
- Out: ML model retraining flow; portfolio rebalancing suggestions; advanced risk model tuning

#### Intake decomposition

Single story — ML sidecar integration + degraded fallback + UI overlay are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260610-ml-hardening`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260610-ml-hardening.json`

---

### US-0015 — AI-assisted forecast category bucket mapping

Status: DONE
Priority: P1

As a finance operator,
I want my forecast to map mirror categories to income/fixed/variable buckets automatically, with AI proposing buckets for ambiguous/uncategorized rows,
So that my forecast buckets are accurate without me manually tagging every transaction.

#### Scope

- In: DEC-0007 category→bucket config applied via mirror `category_id` + recurring pattern labels (baseline precedence)
- In: AI inference for uncategorized/ambiguous rows: income/fixed/variable bucket + confidence metadata; low-confidence → Variable
- In: Privacy defaults: bucket inference under `allow_raw_transactions=false` — aggregates + allowlisted signals only
- In: `GET /api/v1/forecast/monthly` exposing `bucket_source` (`config` | `ai` | `default`)
- In: Monthly tab stat cards showing **AI-mapped** indicator when AI-assigned rows present; config-only months show no AI badge
- In: AI bucket assignments logged per US-0006 audit patterns
- Out: Config-mapped rows overridden by AI; full ML category mapping; tax reporting

#### Intake decomposition

Single story — bucket mapping + AI inference + audit log are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260608-ai-bucket-mapping`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260608-ai-bucket-mapping.json`

---

### US-0014 — Planning mode intuitive UX completion

Status: DONE
Priority: P1

As a finance operator using `/planning`,
I want onboarding, empty-state handling, and guided PVA UX that doesn't assume I know the system,
So that I can create and compare plans without hitting silent no-ops or illogical aggregates.

#### Scope

- In: Onboarding: When `plans.length === 0`, Scenarios shows template card grid + primary **Create empty plan** CTA
- In: Empty-plan add-lines UX visible + wired; submitting a line updates compare/PVA; API/mutation errors surface as toast or inline
- In: Compare contextual UX: zero-adjustment plan shows 0.00 delta + contextual help explaining overlay-only delta
- In: PVA guided UX: no active plan → guided card with Set active / Scenarios link per DEC-0074
- In: Template discoverability: built-in templates reachable from first-run and existing-plan UI
- In: Set-active guidance: first plan create → inline banner explaining requirement for PVA + Grafana
- In: Error surfaces: planning mutations show operator-visible errors on failure
- In: OIDC regression: `/planning` all three tabs pass OIDC-enabled deploy smoke
- Out: Plan sharing UI; automated scenario generation; ML plan suggestions

#### Intake decomposition

Single story — planning onboarding + empty state + guided UX is one vertical slice (frontend-only per GATE-SCOPE-1).

#### Intake evidence

- `intake_run_id`: `intake-20260612-planning-ux`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260612-planning-ux.json`
- Research: [R-0070](docs/engineering/research.md#r-0070--bug-0011-planning-mode-compare-delta-empty-state-api-first-run-ux)

---

### US-0016 — Root README for operators and contributors (living documentation)

Status: DONE
Priority: P2

As a new operator or contributor,
I want a single root README that is the primary entry point for both operator quickstart and contributor orientation,
So that I can onboard without reading the entire docs tree.

#### Scope

- In: DEC-0059 user-channel H2 sections (Purpose, Quickstart, Examples, Limitations, Related documentation) populated with Flow Finance AI-specific content for `DOC_AUDIENCE_PROFILE=both` and `DOC_DETAIL_LEVEL=balanced`
- In: `## Contributing` pointing to `docs/developer/README.md`; no forbidden `DEV_*` H2 titles in root
- In: Related documentation section linking `docs/user-guides/`, `docs/engineering/runbook.md`, compose entry commands
- In: `python scripts/validate_doc_profile.py --repo .` exits 0
- In: Runbook documenting README maintenance cadence (product status update at release)
- In: `template/README.md` present with profile H2 parity
- Out: Per-US README sections; automated README validation beyond validator
- Out: `docs/user-guides/US-xxxx.md` (US-0032 defers until operator asks)

#### Intake decomposition

Single story — README bootstrap + validator + template parity are one vertical documentation slice.

#### Intake evidence

- `intake_run_id`: `intake-20260608-readme-bootstrap`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260608-readme-bootstrap.json`

---

### US-0017 — README living-doc expansion and troubleshooting (post-US-0016)

Status: DONE
Priority: P2

As a production operator troubleshooting deployment issues,
I want the README to include omniflow external-profile smoke examples, limitations + troubleshooting for ML vs sync, and product status updates tracking closed bugs,
So that I can diagnose common issues without reading separate runbooks.

#### Scope

- In: Root README Examples include omniflow external-profile smoke commands (sync trigger, forecast recompute pointer, six `/analytics/{slug}` routes, exchange sync sanity)
- In: Limitations / Troubleshooting subsection documenting empty-Grafana vs ML-unavailable distinction, `BACKEND_FRONTEND_DEPLOY` cadence, sync+recompute prerequisite
- In: Product status subsection listing closed US and BUGs shipped post-US-0016
- In: `docs/developer/README.md` + runbook § requiring Product status README update in release + refresh-context checklists
- In: `python scripts/validate_doc_profile.py --repo .` exits 0; split layout preserved (no `DEV_*` H2 in root; H2 budget ≤ 8 per DEC-0059)
- Out: `docs/user-guides/` per-feature guides (US-0032 defers); automated README coverage beyond validator

#### Intake decomposition

Single story — README expansion + troubleshooting + product status are one vertical documentation slice.

#### Intake evidence

- `intake_run_id`: `intake-20260608-readme-living-doc`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260608-readme-living-doc.json`

---

### US-0018 — Category filters & expense trend analytics

Status: DONE
Priority: P1

As a household budget operator,
I want to filter by Firefly category across product views + see how each category's spending changes month-over-month,
So that I can spot where I save or overspend and use categories in forecast/planning what-ifs.

#### Scope

- In: Shared **category filter** contract (API + React) on forecast monthly/compare/wealth/Grafana
- In: **Per-category monthly expense series** API rolling 12–24 months (GET)
- In: React category trend chart (bar default; line optional) with EUR totals per month — single-category MVP
- In: Category performance summary (MoM delta, best/worst month in period)
- In: **Uncategorized** labeled bucket per AC-5
- In: **Grafana $category variable** on cashflow + budgets panels
- Out: **Multi-category overlay** (stretch; this iteration is single-category MVP); Firefly category editing; ML category auto-labeling (US-0015)

#### Intake decomposition

Multi-story: US-0018 = category analytics foundation; US-0019 = planning/goals (split); US-0020 = subscriptions/tags (split).

#### Intake evidence

- `intake_run_id`: `intake-20260607-category-planning-subscriptions`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260607-category-planning-subscriptions.json`

---

### US-0019 — Goal-driven planning with per-plan stats & AI savings suggestions

Status: DONE
Priority: P1

As a finance operator,
I want to create goal-driven plans with per-plan statistics, category-scoped spend adjustments, and AI savings suggestions,
So that I can plan toward financial goals (e.g. €10K in 5 months) and identify where to cut spending.

#### Scope

- In: Plan target balance + target date (e.g. €10K in 5 months)
- In: Per-plan stats: monthly delta vs baseline, yearly rollup, projected balance at target date
- In: Category-scoped spend changes (e.g. reduce "crypto" category); adjustments applied to compare/PVA after recompute
- In: AI proposes reducible expense categories/lines with evidence summary; operator selects suggestions to add
- In: Privacy: AI uses aggregate/category signals only (`allow_raw_transactions=false`); audit log per US-0006
- In: Regression: US-0014 onboarding/templates still work; OIDC external profile smoke
- Out: Automatic plan optimization; scenario generation from goals; plan sharing UI

#### Intake decomposition

Single story — goals + per-plan stats + AI savings are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260608-goal-planning`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260608-goal-planning.json`

---

### US-0020 — Subscription manual discovery, majority category & operator tags

Status: DONE
Priority: P1

As a finance operator,
I want to manually search for recurring patterns, confirm them as subscriptions, have the display category set from majority-category of underlying transactions, and tag subscriptions with custom labels,
So that I can curate my subscription list beyond automated detection.

#### Scope

- In: Manual search of recurring expense candidates by account/title/payee/interval
- In: Operator confirmation of searched candidate (without auto-detection path)
- In: Majority-category display for confirmed subscriptions (mode category of constituent transactions; documented tie-break)
- In: Operator tags CRUD (e.g. luxus, important); assign multiple per subscription; filter subscription list by tag
- In: Storage: tags + majority-category metadata in product DB; no Firefly write-back
- In: Regression: US-0003/US-0008 detection + alert dedup unchanged; OIDC external smoke
- Out: AI-assisted tag suggestions; subscription budget forecasting

#### Intake decomposition

Single story — manual discovery + majority category + tags are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260627-subscription-manual-discovery`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260627-subscription-manual-discovery.json`

---

### US-0021 — Subscription transaction explorer with rich filters

Status: DONE
Priority: P1

As a finance operator hunting subscriptions,
I want to search transactions by account, payee, category, Geldbereich, and date range, with recurring hints and manual activate for subscription/standing-order,
So that I can find recurring expenses that auto-detection missed and curate my subscription list from transaction evidence.

#### Scope

- In: Transaction search: individual expense txs with pagination/cap (not recurrence-only candidates)
- In: Rich filters: account, payee/description, category, Geldbereich (account_role), date range; optional amount band and recurring/pattern-hint mode
- In: Pattern hint: recurring suggestion when filtered txs form a pattern (including below auto-detection threshold)
- In: Manual activate: confirm transaction group as subscription or standing order per DEC-0085/0099 merge + rejection rules
- In: Regression: US-0020 tags/majority category + US-0003/US-0008 detection + alert dedup unchanged
- In: OIDC external profile smoke on `/subscriptions` Discover transaction search + confirm flow
- Out: Subscription budget forecasting; tag-aware transaction search; AI subscription suggestions

#### Intake decomposition

Single story — transaction search + filters + pattern hint + manual activate are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260612-subscription-tx-explorer`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260612-subscription-tx-explorer.json`
- Research: [R-0092](docs/engineering/research.md#r-0092--us-0021-subscription-transaction-explorer-vs-recurrence-only-discover)

---

### US-0022 — Deploy version stamp & stale-frontend detection

Status: DONE
Priority: P2

As an operator,
I want to see when a deployment happened and whether my browser has a stale bundle,
So that I know whether I'm seeing the latest features after a deploy.

#### Scope

- In: Subtle version/build indicator in app shell (footer or sidebar corner)
- In: Hover reveals release tag, build id, build timestamp (UTC)
- In: `GET /health` extended or `GET /api/v1/meta/build-info` returning build id + release tag (no secrets)
- In: Frontend bundle embeds build id at compile time (`VITE_BUILD_ID`)
- In: Non-blocking stale warning with reload affordance when SPA build id ≠ backend id; no false positive when ids match
- In: `/health` liveness unchanged; OIDC external profile smoke; metadata responses contain no env secrets
- Out: Build pipeline integration documentation (operator configures VITE_BUILD_ID); version-based feature flags

#### Intake decomposition

Single story — version stamp + stale detection are one vertical slice.

#### Intake evidence

- `intake_run_id`: `intake-20260614-version-stamp`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- Evidence bundle: `handoffs/intake_evidence/intake-20260614-version-stamp.json`

---

## Intake evidence (2026-05-31)

- `intake_run_id`: `intake-20260531-flow-finance`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- `coverage_complete`: `true`
- Evidence bundle: `handoffs/intake_evidence/intake-20260531-flow-finance.json`
- `plan_area_inventory`: platform-foundation, firefly-integration, forecasting, grafana-analytics, subscriptions, planning, alerts-wealth, ai-assistant, crypto-portfolio, local-ai, advanced-forecasting
- `plan_area_coverage`: see evidence bundle JSON
