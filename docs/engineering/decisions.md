# Decisions

## Current context pack

- **Latest released story:** US-0020 / S0019 (`0.20.0-us0020`, 2026-06-10) — subscription discover explorer, manual confirm, majority category, operator tags (DEC-0098 discover explorer, DEC-0099 manual confirm, DEC-0100 majority category, DEC-0101 tag schema, DEC-0102 tag assign/filter, DEC-0103 Grafana `$tag` P2); verify-work + release PASS; intake bundle drain complete
- **Prior released story:** US-0019 / S0018 (`0.19.0-us0019`, 2026-06-09) — goal-driven planning with per-plan stats & AI savings (DEC-0091 goal schema, DEC-0092 goal-stats API, DEC-0093 category overlay cap, DEC-0094 savings ranking, DEC-0095 goal account, DEC-0096 PVA scope, DEC-0097 AI tool path); verify-work + release PASS
- **Prior released story:** US-0018 / S0017 (`0.18.0-us0018`, 2026-06-09) — category filters & expense trend analytics (DEC-0087 expense-series API, DEC-0088 filter + bar chart, DEC-0089 cross-surface semantics, DEC-0090 index deferral); verify-work + release PASS
- **Active bug:** none (intake bundle bug queue drain complete)
- **Latest bug fix:** BUG-0021 / Q0029 (`bug0021-q0029`, 2026-06-11) — CategoryFilter static import + wealth account_role COALESCE path (DEC-0110 BK surfaces, DEC-0111 SQL + label map); verify-work + release PASS
- **Prior bug fix:** BUG-0020 / Q0028 (`bug0020-q0028`, 2026-06-11) — subscription list reconcile + display_category backfill (DEC-0109 migration 016, All-tab filter, detection guard); verify-work + release PASS
- **Prior bug fix:** BUG-0019 / Q0027 (`bug0019-q0027`, 2026-06-10) — Grafana provisioning metrics (DEC-0108 account variable default + mirror-count sync panel); verify-work + release PASS
- **Prior bug fix:** BUG-0018 / Q0026 (`bug0018-q0026`, 2026-06-10) — alert evaluation SQL failure (DEC-0107 scarcity JOIN column qualification); verify-work + release PASS
- **Prior bug fix:** BUG-0017 / Q0025 (`bug0017-q0025`, 2026-06-10) — post-sync forecast recompute cluster (DEC-0105 audit CHECK, DEC-0106 FK CASCADE + retention order); verify-work + release PASS
- **Prior bug fix:** BUG-0016 / Q0024 (`bug0016-q0024`, 2026-06-09) — SPA deep-link HTTP 404 (DEC-0104 Axum SPA fallback, DEC-0057 route order); verify-work + release PASS
- **Prior bug fix:** BUG-0015 / Q0023 (`bug0015-q0023`, 2026-06-07) — subscription confirm persistence after rebuild (DEC-0084 card payee_key, DEC-0085 payee+interval inheritance, DEC-0086 ±3d tolerance); verify-work + release PASS
- **Prior bug fix:** BUG-0014 / Q0022 (`bug0014-q0022`, 2026-06-07) — omniflow post-rebuild cluster (DEC-0081 AQ holdings+FX, DEC-0082 AS1 plan delete guard, DEC-0083 AS2 target_type UI); verify-work + release PASS
- **Prior bug fix:** BUG-0013 / Q0020 (`bug0013-q0020`, 2026-06-09) — budgets MTD cap (DEC-0079 AL1) + Bitunix futures EUR valuation (DEC-0080 AN1); verify-work + release PASS
- **Prior release:** US-0015 / S0016 (`0.16.0-us0015`, 2026-06-06) — AI forecast bucket cascade (config→rule→LLM→Variable, bucket_sources API, AI-mapped badge, DEC-0078); verify-work + release PASS
- **Prior release:** US-0014 / S0015 (`0.15.0-us0014`, 2026-06-08) — planning mutation feedback contract (page-local helper, 7× onError, PVA invalidation, DEC-0077); verify-work + release PASS
- **Prior release:** US-0013 / S0014 (`0.14.0-us0013`, 2026-06-08) — external ML compose contract (`stats-forecast` on external profile, env opt-in, dual CI guard, DEC-0076); verify-work + release PASS
- **Prior release:** US-0016 / S0013 (`0.13.0-us0016`, 2026-06-08) — root README living documentation (`--no-template-parity`, Product status hooks, DEC-0070)
- **Prior release:** US-0012 / S0012 (`0.12.0-us0012`, 2026-06-03) — database bootstrap on first start (`ensure_database`, optional `DATABASE_BOOTSTRAP_URL`, DEC-0058)
- **Prior bug fix:** BUG-0011 / Q0019 (2026-06-08) — planning mode AD/AE/AF (DEC-0073 overlay-only compare, DEC-0074 PVA 200 no_active_plan); verify-work + release PASS
- **Prior bug fix:** BUG-0008 / Q0018 (2026-06-08) — subscription alert dedup + detection recall (DEC-0071 W bundle, DEC-0072 X Phase 1); verify-work + release PASS
- **Prior bug fix:** BUG-0009 / Q0016 (2026-06-06) — Grafana provisioning-only analytics fix (DEC-0068 ABS balance default, portfolio LATERAL breakdown, ML banner); verify-work + release PASS
- **Prior bug fix:** BUG-0012 / Q0014+Q0015 (2026-06-06) — monthly Income/Fixed bucket attribution (DEC-0067 component-level monthly_map); verify-work + release PASS
- **Prior bug fix:** BUG-0010 / Q0013 (2026-06-05) — forecast/wealth/ML posture (DEC-0065 negative wealth, DEC-0066 ML disabled metadata); verify-work + release PASS
- **Prior bug fix:** BUG-0006 / Q0010 (2026-06-05) — Firefly category/date/amount ingest (DEC-0059), aggregate contract (R-0060 fulfilled), verify-work + release PASS
- **Prior bug fix:** BUG-0005 / Q0012 (2026-06-05) — Bitunix dual REST auth (DEC-0062), futures enable policy (DEC-0063), wallet vs position wealth (DEC-0064), verify-work + release PASS
- **Prior bug fix:** BUG-0004 / Q0011 (2026-06-05) — ExchangesOnly sync terminal status (I1), portfolio UNION SQL (K1), Firefly balance parse (DEC-0060), wealth NULL handling (L2), subscription payee fallbacks (DEC-0061); verify-work + release PASS
- **Prior bug fix:** BUG-0003 / Q0009 (2026-06-05) — `DATABASE_HOST=postgres` ops recovery (F1), env guard docs (F2), `effective_enabled` connector registry (G1); verify-work + release PASS; G2 futures auth spike skipped (gated)
- **Prior bug fix:** BUG-0002 / Q0008 (2026-06-05) — Firefly PAT guard, risk-score 200 empty-state, exchange settings; verify-work + release PASS
- **Prior bug fix:** BUG-0001 / Q0007 (2026-06-04) — `DevBypassAuthProvider` + `GF_SERVER_ROOT_URL`; DEC-0057 follow-through
- **Open bug queue:** (empty — intake bundle drain complete)
- **Open epics:** (empty — backlog drain complete for current scope)
- **Active quick task:** none (Q0029 released)
- **Active story:** none (intake bundle backlog drain complete)
- **Open stories:** (empty)
- **Research:** [R-0089](research.md#r-0089--bug-0019-grafana-cashflow-zeros-account_id-default--sync-entity-counts-per-run-cursor) fulfilled by BUG-0019/Q0027/DEC-0108; [R-0088](research.md#r-0088--bug-0018-evaluate_scarcity-ambiguous-balance--alert-eval-pipeline-abort) fulfilled by BUG-0018/Q0026/DEC-0107; [R-0087](research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading) fulfilled by BUG-0017/Q0025/DEC-0105/0106; [R-0086](research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik) fulfilled by BUG-0016/Q0024/DEC-0104; [R-0085](research.md#r-0085--us-0020-subscription-discover-majority-category--operator-tags) fulfilled by US-0020/S0019/DEC-0098..0103; [R-0080](research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake) fulfilled (US-0018/US-0019/US-0020 portions via R-0083/R-0084/R-0085); [R-0084](research.md#r-0084--us-0019-goal-plans-per-plan-stats-category-overlay--ai-savings) fulfilled by US-0019/S0018/DEC-0091..0097; [R-0083](research.md#r-0083--us-0018-category-filters-expense-series-api--trend-analytics) fulfilled by US-0018/S0017/DEC-0087..0090; prior [R-0081](research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild) + [R-0082](research.md#r-0082--card-billing-descriptor-normalization-for-subscription-identity) fulfilled by BUG-0015/Q0023/DEC-0084/0085/0086; [R-0079](research.md#r-0079--bug-0014-post-rebuild-omniflow-ml-sidecar-crypto-display-grafana-planning) fulfilled by BUG-0014/Q0022/DEC-0081/0082/0083
- **Architecture:** **DEC-0110** + **DEC-0111** frozen BUG-0021 (2026-06-11); **DEC-0109** shipped BUG-0020 (2026-06-11); **DEC-0108** shipped BUG-0019 (2026-06-10); **DEC-0107** shipped BUG-0018 (2026-06-10); **DEC-0105** + **DEC-0106** shipped BUG-0017 (2026-06-10); **DEC-0104** shipped BUG-0016 (2026-06-09); **DEC-0098**..**DEC-0103** shipped US-0020 (2026-06-10); **DEC-0091**..**DEC-0097** shipped US-0019 (2026-06-09); **DEC-0087**..**DEC-0090** shipped US-0018 (2026-06-09)
- **Research:** [R-0091](research.md#r-0091--bug-0021-categoryfilter-lazy-load-delay--wealth-role-column-empty) fulfilled by BUG-0021/Q0029/DEC-0110/0111; [R-0090](research.md#r-0090--bug-0020-subscriptions-list-duplicates--uncategorized-display-category) fulfilled by BUG-0020/Q0028/DEC-0109
- **Recommended next:** idle — await new intake (`/intake` or operator story)
- **Backlog drain:** complete — `open_stories_remaining=0`; `bug_queue_remaining=0`

## Compact decision index (bounded summaries)

| ID | Status | Topic | Summary |
|----|--------|-------|---------|
| DEC-0001 | Accepted | OIDC IdP default | Authentik optional `oidc` profile; external IdP via env (R-0003) |
| DEC-0002 | Accepted | Sync watermark | Date-window + 7-day overlap; upsert by Firefly id (R-0002) |
| DEC-0003 | Accepted | External DB startup | Exponential backoff retry, max ~60s (R-0005) |
| DEC-0004 | Accepted | Firefly read-only | GET-only client + audit log + integration test (R-0001) |
| DEC-0005 | Accepted | Schema scope | Relational mirrors US-0001; hypertables US-0002 (R-0004) |
| DEC-0006 | Accepted | API auth pattern | SPA bearer JWT + JWKS; no BFF (R-0003) |
| DEC-0007 | Accepted | Forecast algorithm | Hybrid rule-based: recurring heuristics + 3-month rolling avg (R-0006) |
| DEC-0008 | Accepted | Forecast storage | Precomputed hypertable snapshots on sync (R-0007) |
| DEC-0009 | Accepted | Account scope | Per-account primary; optional aggregate endpoint (R-0006, R-0008) |
| DEC-0010 | Accepted | Recompute trigger | Extends sync mutex; inline after successful ingest (R-0007) |
| DEC-0011 | Accepted | Forecast retention | Keep last 5 successful computations (R-0007) |
| DEC-0012 | Accepted | Grafana scarcity | Static €200 threshold; stable dashboard/datasource uids (R-0008) — **superseded by DEC-0029** |
| DEC-0013 | Accepted | Recurrence core | Extract shared `recurrence` module; forecast wrapper + subscription engine (R-0009) |
| DEC-0014 | Accepted | Confidence tiers | 95/80/60% tiers; emit only ≥60%; min 3 txs (R-0009) |
| DEC-0015 | Accepted | Subscription schema | Single lifecycle `subscription_patterns` + satellites; rejection fingerprints (R-0012) |
| DEC-0016 | Accepted | Dauerauftrag | Rule-based classification + optional config patterns (R-0010) |
| DEC-0017 | Accepted | Price-change threshold | Dual: ≥€1.00 AND ≥5%; configurable in TOML (R-0011) |
| DEC-0018 | Accepted | Sync pipeline order | Inline `subscriptions` phase before forecast in sync mutex (R-0013, extends DEC-0010) |
| DEC-0019 | Accepted | Plan projection model | Delta overlay on latest forecast baseline; template presets (R-0015) |
| DEC-0020 | Accepted | Plan version semantics | Hybrid editable latest; freeze on new version; max 3 versions (R-0016) |
| DEC-0021 | Accepted | Plan-vs-Ist metric | Household daily net cashflow; deviation = actual − planned (R-0017) |
| DEC-0022 | Accepted | Plan persistence | Migration 004 plans/versions/adjustments + plan_daily hypertable (R-0018) |
| DEC-0023 | Accepted | Plan recompute triggers | Plan save async + post-forecast hook; no sync phase; defer /forecast overlay (R-0019) |
| DEC-0024 | Accepted | Active plan & Dashboard 3 | Single global active plan; Grafana uid `budgets`; household MVP (R-0020) |
| DEC-0025 | Accepted | Net worth aggregation | Asset sum; mixed-currency warning; daily snapshots; crypto excluded (R-0021) |
| DEC-0026 | Accepted | Alert evaluation rules | Household scarcity; category-targeted budget drift; plan viability (R-0022) |
| DEC-0027 | Accepted | Alert persistence | Migration 005; fingerprint dedup; acknowledge/dismiss lifecycle (R-0023) |
| DEC-0028 | Accepted | Sync alerts phase | Inline `"alerts"` phase after forecast+plan hook (R-0024) |
| DEC-0029 | Accepted | Threshold centralization | TOML → `alert_config` mirror; Grafana `$scarcity_threshold`; supersedes DEC-0012 hardcode (R-0025) |
| DEC-0030 | Accepted | Unified inbox UI | `/wealth`, `/alerts`, header bell; subscription alerts unchanged (R-0023, R-0026) |
| DEC-0031 | Accepted | AI orchestration | `AiTool` trait registry; async-openai loop; max 5 rounds; services-only `ToolContext` (R-0027) |
| DEC-0032 | Accepted | Privacy layer | Central middleware; Projectplan defaults; aggregates when raw disabled (R-0028) |
| DEC-0033 | Accepted | Chat SSE API | POST `/api/v1/chat/stream`; Bearer JWT; ephemeral client threads (R-0029) |
| DEC-0034 | Accepted | Audit persistence | Migration 006 `ai_tool_audit`; 500 cap + 90-day purge; redacted args only (R-0030) |
| DEC-0035 | Accepted | Tool mapping | Six in-process service tools; `project_ephemeral`; 8 KB result cap (R-0031) |
| DEC-0036 | Accepted | React chat UX | Header Sheet drawer + `/chat` + Settings AI & Privacy + audit table (R-0029) |
| DEC-0037 | Accepted | Exchange connectors | Unified `ExchangeConnector` trait; GET-only; spot+linear; Bitunix spot-first (R-0032) |
| DEC-0038 | Accepted | PnL methodology | Hybrid exchange + avg-cost; wealth analytics not tax (R-0033) |
| DEC-0039 | Accepted | FX conversion | Frankfurter fiat/stablecoin; exchange tickers alts; `fx_incomplete` banner (R-0034) |
| DEC-0040 | Accepted | Exchange secrets | TOML env names + Compose env; Settings read-only (R-0035) |
| DEC-0041 | Accepted | Sync exchanges phase | Inline `"exchanges"` before `"alerts"`; independent interval (R-0036) |
| DEC-0042 | Accepted | Migration 007 schema | Holdings/trades/PnL; extend `net_worth_snapshots`; `allocation_target` (R-0037) |
| DEC-0043 | Accepted | AI provider factory | Unified `OpenAiCompatibleProvider` + `build_provider`; trait-object orchestrator (R-0040) |
| DEC-0044 | Accepted | AI provider modes | `openai \| ollama \| openai_compatible` + `base_url`; restart to switch (R-0038, R-0039) |
| DEC-0045 | Accepted | Local HTTP quirks | Omit `tool_choice` for local; temperature 0.3 default (R-0038, R-0041) |
| DEC-0046 | Accepted | Local tool fallback | Graceful text + SSE warning; optional nudge; no OpenAI fallback (R-0041) |
| DEC-0047 | Accepted | Settings + AI test | Provider status fields; `POST /api/v1/ai/test`; chat badge UX (R-0042) |
| DEC-0048 | Accepted | Audit provider + AC5 | Migration 008 `provider` column; wiremock isolation test (R-0042) |
| DEC-0049 | Accepted | StatsForecast sidecar | Python FastAPI in Compose `full`; `[forecast_ml] enabled=false` default (R-0044) |
| DEC-0050 | Accepted | ML overlay model_kind | Layered overlay; baseline authoritative; paired_baseline_id (R-0049) |
| DEC-0051 | Accepted | Seasonal model ladder | AutoETS 12–23 mo; MSTL ≥24 mo; SeasonalNaive fallback (R-0045) |
| DEC-0052 | Accepted | Sync forecast_ml phase | After baseline + plan hook; ML failure never fails sync (R-0050) |
| DEC-0053 | Accepted | API variant + bands | `variant` query param; compare endpoint; p10/p90 columns (R-0046) |
| DEC-0054 | Accepted | Plan risk score | Deterministic 0–100 index; plan_risk_scores table (R-0048) |
| DEC-0055 | Accepted | Grafana Dashboard 5 ML | `$forecast_variant`; band + seasonal + portfolio + risk panels (R-0051) |
| DEC-0056 | Accepted | Omniflow external deploy | `bundled-firefly` split; overlay contract; env Traefik; Grafana internal default (R-0052, R-0053) |
| DEC-0057 | Accepted | Unified analytics proxy | Same-origin `/analytics/grafana/` proxy; anonymous Grafana; embed env contract (R-0054, DEC-0056) |
| DEC-0058 | Accepted | Database bootstrap on first start | In-app `ensure_database`; optional `DATABASE_BOOTSTRAP_URL`; OWNER create; extension via maintenance creds (R-0055, US-0012) |
| DEC-0059 | Accepted | Firefly mirror amount sign | Normalize signed `amount` at ingest from split `type`; upsert backfill (R-0060, BUG-0006) |
| DEC-0060 | Accepted | Firefly account balance parse | Reuse `parse_split_amount` for `current_balance` string/number at account sync; upsert backfill (R-0061, BUG-0004) |
| DEC-0061 | Accepted | Subscription payee key fallbacks | Payee grouping: description → counterparty_name → destination_name (R-0061, BUG-0004) |
| DEC-0062 | Accepted | Bitunix dual REST auth | Spot query-sign + futures header-sign on separate hosts (R-0058, BUG-0005) |
| DEC-0063 | Accepted | Bitunix futures enable policy | `effective_enabled_futures()` auto-enable when creds present; env opt-out (R-0059, BUG-0005) |
| DEC-0064 | Accepted | Futures wallet vs position wealth | Wallet priced in subtotal; linear positions unrealized-only (DEC-0038, BUG-0005) |
| DEC-0065 | Accepted | Negative asset wealth visibility | Include overdrawn asset accounts with `is_overdrawn`; signed subtotal (R-0062, BUG-0010) |
| DEC-0066 | Accepted | ML disabled metadata | Persist/derive `sidecar_disabled` when forecast_ml off (DEC-0049, BUG-0010) |
| DEC-0067 | Accepted | Monthly bucket attribution | Component-level monthly_map; rolling→Variable; recurring→category_id→map_category (R-0063, BUG-0012) |
| DEC-0068 | Accepted | Grafana analytics provisioning | ABS(balance) variable default; portfolio subquery+LATERAL breakdown; portfolio-only overview; ML banner+noValue (R-0064, BUG-0009) |
| DEC-0069 | Accepted | AI merchant/category discovery | category_search on get_transactions; mirror_date_bounds; subscriptions schema+guard; orchestrator prompt+audit result_rows (R-0065, BUG-0007) |
| DEC-0070 | Accepted | Root README living documentation | `--no-template-parity` until full `template/`; `### Product status` under Purpose; release + refresh-context hooks; US-0017 H3 Examples/Troubleshooting + per-segment maintenance (R-0066, R-0067, R-0078, US-0016, US-0017) |
| DEC-0071 | Accepted | Subscription alert dedup & unread count | Fingerprint partial unique + upsert_alert; GET unread-count API; orphan lifecycle; US-0005-only bell (R-0068, BUG-0008 W) |
| DEC-0072 | Accepted | Subscription detection Phase 1 recall | Payee normalization + transfer counterparty priority + 730-day window; Phase 2 category grouping gated; AI deferred (R-0069, BUG-0008 X) |
| DEC-0073 | Accepted | Compare overlay-only monthly delta | `monthly_delta_sum` via `build_overlay_deltas`; projected balance unchanged; zero-overlay → 0.00 (R-0070, BUG-0011 AE) |
| DEC-0074 | Accepted | Plan-vs-actual 200 no_active_plan | Tagged 200 JSON mirror risk-score; guided PVA UX; no auto-activate (R-0070, BUG-0011 AF) |
| DEC-0076 | Accepted | External ML compose contract | Overlay `stats-forecast` on external profile; traefik network; env opt-in; dual CI guard (R-0071, US-0013) |
| DEC-0077 | Accepted | Planning mutation feedback | Page-local success/error helper; mandatory onError on 7 mutations; PVA invalidation (R-0073, US-0014) |
| DEC-0078 | Accepted | AI forecast bucket cascade | Config→rule→LLM→Variable; 0.75 threshold; R-0075 privacy; bucket_sources API; US-0008 provider (R-0074, US-0015) |
| DEC-0079 | Accepted | Budgets MTD upper date bound | `pdc.ts::date <= CURRENT_DATE` on MTD planned CTE; panel id 5 only (R-0076 §7, BUG-0013 AL) |
| DEC-0080 | Accepted | Bitunix futures EUR valuation | Wallet array parse + USDT equity price; linear unrealizedPNL→EUR; DEC-0064 subtotal rules (R-0076 §6, BUG-0013 AN/AK) |
| DEC-0081 | Accepted | Wealth all-holdings + unified FX | `holdings_all` cap 50; wire PnL `unpriced_assets`; single `fx_incomplete` gate (R-0079 §6, BUG-0014 AQ) |
| DEC-0082 | Accepted | Block active plan delete | HTTP 409 on DELETE when `is_active`; deactivate-first UX (R-0079 §6, BUG-0014 AS1) |
| DEC-0083 | Accepted | Planning target_type UI alignment | Remove invalid `account`; expose DB enum values + help copy (R-0079 §6, BUG-0014 AS2) |
| DEC-0084 | Accepted | Card billing payee_key normalization | Comma/asterisk/domain collapse in `payee_key()` per R-0082; extends DEC-0072 (BUG-0015 AU1) |
| DEC-0085 | Accepted | Payee+interval confirm inheritance | Skip+merge on `(payee_key, interval_days)`; rejection by payee+interval; stale map (BUG-0015 AU2–AU4) |
| DEC-0086 | Accepted | Interval tolerance + fingerprint rotation | ±3d `interval_matches`; in-place fingerprint update on confirmed merge (BUG-0015 AU2–AU4) |
| DEC-0087 | Accepted | Category expense-series API | Month spine SQL; catalog + expense-series; `__uncategorized__` sentinel; server summary (R-0083, US-0018) |
| DEC-0088 | Accepted | CategoryFilter + bar trend chart | Single-select MVP; bar default ECharts; defer multi-overlay (R-0083, US-0018) |
| DEC-0089 | Accepted | Cross-surface filter semantics | Forecast actuals-only panel; planning widget; independent Grafana `$category` (R-0083, US-0018) |
| DEC-0090 | Accepted | Category index deferral | No index in MVP; optional migration if EXPLAIN >50 ms (R-0083 §7, US-0018) |
| DEC-0091 | Accepted | Goal balance plan schema | `goal_balance` enum + plan-level target fields (R-0084 §1, US-0019) |
| DEC-0092 | Accepted | Per-plan goal-stats API | `GET …/goal-stats`; calendar yearly rollup; gap copy; 730d horizon (R-0084 §2, US-0019) |
| DEC-0093 | Accepted | Category overlay cap | remove_outflow capped at 3-mo avg; add household-labeled (R-0084 §3, US-0019) |
| DEC-0094 | Accepted | Deterministic savings suggestions | Aggregate ranking; fixed-bucket exclusion; modal apply (R-0084 §4, US-0019) |
| DEC-0095 | Accepted | Goal account scope | Optional `goal_account_id`; default max-balance asset (R-0084 §5, US-0019) |
| DEC-0096 | Accepted | PVA household scope | Active-plan PVA unchanged; per-plan stats via goal-stats (R-0084, US-0019) |
| DEC-0097 | Accepted | AI savings path | REST primary; optional `get_category_savings` tool P2 (R-0084 §4, US-0019) |
| DEC-0098 | Accepted | Subscription discover explorer | Reuse recurrence core; GET `/discover`; cap 50; amount band P2 (R-0085 §1, US-0020) |
| DEC-0099 | Accepted | Manual confirm-from-discover | POST `/discover/confirm`; direct confirmed; DEC-0085 merge; no alert (R-0085 §2, US-0020) |
| DEC-0100 | Accepted | Display majority category | `display_category_id`; RANK tie-break; recompute on merge (R-0085 §3, US-0020) |
| DEC-0101 | Accepted | Operator tag schema | `operator_tags` + junction; hard delete; global scope (R-0085 §4, US-0020) |
| DEC-0102 | Accepted | Tag assign and list filter | PUT replace set; `?tag=` slug filter (R-0085 §4–5, US-0020) |
| DEC-0103 | Accepted | Grafana subscriptions `$tag` | P2 stretch; DEC-0089 independent pattern (R-0085 §6, US-0020) |
| DEC-0104 | Accepted | Axum SPA index.html fallback | `ServeDir::fallback(ServeFile)` HTTP 200; health→grafana→api→SPA; Traefik pass-through (R-0086, BUG-0016) |
| DEC-0105 | Accepted | ai_tool_audit CHECK extension | DROP+ADD tool_name + result_status; forecast_bucket_assignment + extended statuses (R-0087 §2, BUG-0017 AY/AZ) |
| DEC-0106 | Accepted | paired_baseline_id CASCADE + retention order | ON DELETE CASCADE; ml_enhanced before baseline prune (R-0087 §3, BUG-0017 BA/BC) |
| DEC-0107 | Accepted | Scarcity JOIN column qualification | `fbd.balance` + `fbd.ts` in evaluate_scarcity; forbid a.balance (R-0088 §2, BUG-0018 BE) |
| DEC-0108 | Accepted | Grafana account default + sync counts | `$account_id` sort/current; mirror COUNT panel (R-0089, BUG-0019 BG/BH) |
| DEC-0109 | Accepted | Subscription list reconcile + backfill | Migration 016; All-tab filter; forward pending guard (R-0090, BUG-0020 BI/BJ) |
| DEC-0110 | Accepted | CategoryFilter static import BK | ForecastPage + WealthPage eager import; PlanningPage P2 optional (R-0091, BUG-0021 BK) |
| DEC-0111 | Accepted | Wealth account_role path + labels | COALESCE attributes/root SQL; frontend formatAccountRole map (R-0091, BUG-0021 BL) |

### BUG-0021 architecture (2026-06-11) — released Q0029 2026-06-11

Per [R-0091](research.md#r-0091--bug-0021-categoryfilter-lazy-load-delay--wealth-role-column-empty): **DEC-0110** + **DEC-0111** accepted and shipped (`bug0021-q0029`); operator deploy smoke pass-with-prerequisites.

### BUG-0018 architecture (2026-06-10) — released Q0026 2026-06-10

Per [R-0088](research.md#r-0088--bug-0018-evaluate_scarcity-ambiguous-balance--alert-eval-pipeline-abort): **DEC-0107** accepted and shipped (`bug0018-q0026`); operator sync/alerts smoke pass-with-prerequisites.

### BUG-0017 architecture (2026-06-09) — released Q0025 2026-06-10

### BUG-0016 architecture (2026-06-09) — released Q0024 2026-06-09

Per [R-0086](research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik): **DEC-0104** accepted and shipped (`bug0016-q0024`); Axum-only SPA fallback in `build_router`; extends **DEC-0057** route ordering; supersedes BUG-0009 analytics 404 advisory; operator SPA deep-link smoke pass-with-prerequisites.


### US-0020 architecture (2026-06-10) — released S0019 2026-06-10

Per [R-0085](research.md#r-0085--us-0020-subscription-discover-majority-category--operator-tags): **DEC-0098**..**DEC-0103** accepted and shipped (`0.20.0-us0020`); operator discover/tag smoke pass-with-prerequisites; intake bundle drain complete.

### US-0019 architecture (2026-06-09) — released S0018 2026-06-09

Per [R-0084](research.md#r-0084--us-0019-goal-plans-per-plan-stats-category-overlay--ai-savings): **DEC-0091**..**DEC-0097** accepted and shipped (`0.19.0-us0019`); operator goal-plan smoke pass-with-prerequisites.

### US-0018 architecture (2026-06-08) — released S0017 2026-06-09

Per [R-0083](research.md#r-0083--us-0018-category-filters-expense-series-api--trend-analytics): **DEC-0087**, **DEC-0088**, **DEC-0089**, **DEC-0090** accepted and shipped (`0.18.0-us0018`). T-0185 EXPLAIN probe deferred DEC-0090; operator category-filter smoke pass-with-prerequisites.

### BUG-0015 architecture (2026-06-07) — released Q0023 2026-06-07

Per [R-0081](research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild) + [R-0082](research.md#r-0082--card-billing-descriptor-normalization-for-subscription-identity): **DEC-0084**, **DEC-0085**, **DEC-0086** accepted and shipped (`bug0015-q0023`). V1 operator rebuild smoke pass-with-prerequisites.

### BUG-0014 architecture (2026-06-09) — released Q0022 2026-06-07

Per [R-0079 §6](research.md#r-0079--bug-0014-post-rebuild-omniflow-ml-sidecar-crypto-display-grafana-planning): **DEC-0081**, **DEC-0082**, **DEC-0083** accepted and shipped (`bug0014-q0022`). AO1 extends **DEC-0066** / **DEC-0076** (dual-scenario static banner — no new DEC). AP2/AR1 conditional — deferred pending operator gates (AP1_SQL_PROBE, Full sync).

## DEC-0086 — Interval tolerance and fingerprint rotation on merge (BUG-0015)

**Status:** Accepted  
**Date:** 2026-06-07  
**Work item:** BUG-0015  
**Research:** [R-0081 §C](research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild)  
**Extends:** DEC-0085  
**Architecture:** `docs/engineering/architecture.md` § **BUG-0015**  
**Full record:** [decisions/DEC-0086.md](../../decisions/DEC-0086.md)

### Summary

1. `interval_matches`: ±3 day absolute tolerance on `interval_days` for confirm/reject/stale lookups.
2. On confirmed merge: UPDATE same row `id`; rotate `fingerprint` to newly computed hash; preserve `confirmed_at`.
3. Multi-sub per merchant: composite `(payee_key, interval_days)` — not payee-only.

## DEC-0085 — Payee+interval confirm inheritance (BUG-0015 Layer 2)

**Status:** Accepted  
**Date:** 2026-06-07  
**Work item:** BUG-0015  
**Research:** [R-0081 §C](research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild)  
**Extends:** DEC-0015, DEC-0071, DEC-0072  
**Architecture:** `docs/engineering/architecture.md` § **BUG-0015**  
**Full record:** [decisions/DEC-0085.md](../../decisions/DEC-0085.md)

### Summary

1. Load confirmed/rejected maps keyed by `(payee_key, interval_days)` with DEC-0086 tolerance.
2. Detection: skip+merge into existing confirmed row — no pending INSERT, no `new_detection` alert.
3. `mark_stale_inactive` by payee+interval active set; index `(payee_key, status)`.

## DEC-0084 — Card billing descriptor normalization (BUG-0015 Layer 1)

**Status:** Accepted  
**Date:** 2026-06-07  
**Work item:** BUG-0015  
**Research:** [R-0082](research.md#r-0082--card-billing-descriptor-normalization-for-subscription-identity)  
**Extends:** DEC-0072, DEC-0013  
**Architecture:** `docs/engineering/architecture.md` § **BUG-0015**  
**Full record:** [decisions/DEC-0084.md](../../decisions/DEC-0084.md)

### Summary

1. Extend `payee_key()`: asterisk split, comma left-segment, Apple/ITunes root alias, domain tail strip.
2. Shared `recurrence` module — conservative rules; Layer 2 catches residual drift.
3. Reject normalization-only as sole fix.

## DEC-0080 — Bitunix futures wallet parse and linear unrealized EUR (BUG-0013 AN/AK)

**Status:** Accepted  
**Date:** 2026-06-08  
**Work item:** BUG-0013  
**Research:** [R-0076 §6](research.md#r-0076--omniflow-analytics-regression-hypotheses-post-us-0015)  
**Extends:** DEC-0064, DEC-0038, DEC-0039  
**Architecture:** `docs/engineering/architecture.md` § **BUG-0013**  
**Full record:** [decisions/DEC-0080.md](../../decisions/DEC-0080.md)

### Summary

1. Fix Bitunix `data[]` wallet parse; USDT/USDC futures row with `market_value_usd`.
2. `recompute_pnl`: price futures wallet via fiat stable path; linear rows skip subtotal but convert payload `unrealizedPNL` → `unrealized_pnl_eur` (USDT→EUR).
3. Linear symbols **excluded** from `unpriced_assets` / `fx_incomplete` when exchange unrealized present (DEC-0064).
4. Tier 2 deferred: `ExchangePriceBook`, exposure display, multi-coin margin.
5. AM waived per R-0077 unless HAR shows non-200.

## DEC-0079 — Budgets MTD summary upper date bound (BUG-0013 AL)

**Status:** Accepted  
**Date:** 2026-06-08  
**Work item:** BUG-0013  
**Research:** [R-0076 §7](research.md#r-0076--omniflow-analytics-regression-hypotheses-post-us-0015)  
**Architecture:** `docs/engineering/architecture.md` § **BUG-0013**  
**Full record:** [decisions/DEC-0079.md](../../decisions/DEC-0079.md)

### Summary

1. MTD panel id **5**: add `AND pdc.ts::date <= CURRENT_DATE` to planned CTE.
2. Actual CTE unchanged; deviation = actual − capped planned.
3. Optional footnote when plan horizon starts mid-month.

## DEC-0078 — AI-assisted forecast bucket mapping cascade (US-0015)

**Status:** Accepted  
**Date:** 2026-06-06  
**Work item:** US-0015  
**Research:** [R-0074](research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy), [R-0075](research.md#r-0075--us-0015-forecast-bucket-privacy-feature-allowlist)  
**Architecture:** `docs/engineering/architecture.md` § **US-0015**  
**Full record:** [decisions/DEC-0078.md](../../decisions/DEC-0078.md)

### Summary

1. **Cascade:** DEC-0007 config map first (**AC-1**); rule heuristics; LLM batch on ambiguous recurring rows; Variable fallback (**AC-2**).
2. **Threshold:** `ai_bucket_min_confidence = 0.75` default TOML.
3. **Privacy:** `PrivacyLayer::prepare_bucket_features()` per **R-0075**; `allow_raw_transactions=false` default (**AC-3**).
4. **Invalidation:** Inline per recompute (DEC-0010); config-hash bust; no cross-run DB cache MVP.
5. **API:** `bucket_sources` + `ai_mapped` on `MonthlyPointResponse` (**AC-4/AC-5**).
6. **Provider:** US-0008 `build_provider()`; rule-only when absent.
7. **Audit:** `forecast_bucket_assignment` in `ai_tool_audit` (**AC-6**).
8. **Deferred:** merchant aliases TOML; rolling-residual aggregate AI split (stage-2).

## DEC-0077 — Planning mutation feedback and error surface contract (US-0014)

**Status:** Accepted  
**Date:** 2026-06-08  
**Work item:** US-0014  
**Research:** [R-0073](research.md#r-0073--us-0014-planning-mutation-error-toast-patterns)  
**Architecture:** `docs/engineering/architecture.md` § **US-0014**

### Context

Post-Q0019 discovery: AC-7 gap (no mutation error surfaces); AC-2/AC-5/AC-6 partial feedback. Frontend-only polish; DEC-0073/0074 frozen.

### Decision

1. **Helper:** Page-local `showPlanningFeedback` — success green / error red card; 4s auto-dismiss success; error persists until Dismiss.
2. **Coverage:** Mandatory `onError` on all seven planning mutations (create, activate, apply-template, create-version, add/update/delete adjustment).
3. **Success:** Toasts on create plan, template apply, add adjustment, activate; optional on edit/delete.
4. **Invalidation:** Immediate `plan-vs-actual` invalidation on adjustment CRUD + activate + createPlan.
5. **Banner:** Extend set-active copy for Grafana Dashboard 3 (`budgets`).

Full record: `decisions/DEC-0077.md`

## DEC-0076 — External profile ML sidecar compose contract (US-0013)

**Status:** Accepted  
**Date:** 2026-06-08  
**Work item:** US-0013  
**Research:** [R-0071](research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile)  
**Architecture:** `docs/engineering/architecture.md` § **US-0013**

### Context

US-0009 ML stack is feature-complete; omniflow external profile never starts `stats-forecast`. BUG-0010 deferred AC3 to US-0013.

### Decision

1. **Compose:** Overlay additive `profiles: [external]` on existing `stats-forecast`; `networks: [traefik]`; host port `${STATS_FORECAST_PORT:-8091}:8090`.
2. **Env:** Passthrough `FORECAST_ML_ENABLED` / `STATS_FORECAST_URL` on `flow-finance-ai` in external merge; DEC-0049 default-off preserved.
3. **Failure:** Unchanged DEC-0052/0066 — skip metadata, sync continues.
4. **CI:** External service set includes `stats-forecast`; traefik network assert; retain `forecast_ml_integration`.

Full record: `decisions/DEC-0076.md`

## DEC-0074 — Plan-vs-actual 200 `no_active_plan` (BUG-0011 AF)

**Status:** Accepted  
**Date:** 2026-06-08  
**Work item:** BUG-0011 (sub-defect **AF**)  
**Research:** [R-0070](research.md#r-0070--bug-0011-planning-mode-compare-delta-empty-state-api-first-run-ux)  
**Architecture:** `docs/engineering/architecture.md` § **BUG-0011**

### Context

`plan-vs-actual` returns 404 when no active plan; frontend tab blank. Risk-score already uses 200 `no_score`.

### Decision

1. **API:** HTTP 200 `{ "status": "no_active_plan", "reason": "no_active_plan" }` — not 404.
2. **Frontend:** Guided empty state on PVA tab; `retry: false`.
3. **Policy:** Reject auto-activate on create; explicit Set active retained.

Full record: `decisions/DEC-0074.md`

## DEC-0073 — Compare overlay-only monthly delta (BUG-0011 AE)

**Status:** Accepted  
**Date:** 2026-06-08  
**Work item:** BUG-0011 (sub-defect **AE**)  
**Research:** [R-0070](research.md#r-0070--bug-0011-planning-mode-compare-delta-empty-state-api-first-run-ux), [R-0015](research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline), [R-0016](research.md#r-0016--plan-scenario-versioning-immutable-snapshots-vs-editable-drafts)  
**Architecture:** `docs/engineering/architecture.md` § **BUG-0011**

### Context

Compare sums full `planned_net` (baseline + overlay) — empty plans show illogical negative monthly delta.

### Decision

1. **Metric:** `monthly_delta_sum` = overlay-only sum via `build_overlay_deltas`; empty adjustments → **0.00**.
2. **Balance:** `projected_month_end_balance` unchanged (full scenario).
3. **Scope:** `/compare` + React Compare tab only; Grafana Dashboard 3 unchanged.

**ID note:** US-0090 caveman compression forward-refs renumbered to **DEC-0075**.

Full record: `decisions/DEC-0073.md`

## DEC-0071 — Subscription alert dedup, unread-count contract, orphan lifecycle (BUG-0008 W)

**Status:** Accepted  
**Date:** 2026-06-08  
**Work item:** BUG-0008 (sub-defect **W**)  
**Research:** [R-0068](research.md#r-0068--bug-0008-subscription-alert-dedup-unread-count-contract-orphan-lifecycle)  
**Architecture:** `docs/engineering/architecture.md` § **BUG-0008**

### Context

83 unread `new_detection` alerts vs 6 pending patterns — bare INSERT every sync without fingerprint dedup. Banner uses raw list length.

### Decision

1. **Dedup:** Lifecycle fingerprint + partial unique index + `upsert_alert`; emit only on new pending or tier increase.
2. **API:** `GET /api/v1/subscriptions/alerts/unread-count` with reconciled semantics.
3. **UI:** Banner/toast consume unread-count — not list length; header bell US-0005-only unchanged.
4. **Lifecycle:** Mark-read orphans on confirm/reject/inactive; one-time backfill.

**Prerequisite for:** DEC-0072 (W-before-X mandatory).

Full record: `decisions/DEC-0071.md`

## DEC-0072 — Subscription detection Phase 1 recall (BUG-0008 X)

**Status:** Accepted  
**Date:** 2026-06-08  
**Work item:** BUG-0008 (sub-defect **X**)  
**Research:** [R-0069](research.md#r-0069--bug-0008-detection-recall-levers-ai-path-boundary)  
**Architecture:** `docs/engineering/architecture.md` § **BUG-0008**

### Context

12 patterns from 922+ txs — payee-only grouping fragments SEPA memos; 365-day window too narrow for annual subs.

### Decision

**Phase 1 (execute):** payee normalization + transfer-type counterparty priority + `detection_window_days` 730.  
**Phase 2 (gated):** category-aware grouping (≥70% same category); min_emit tuning blocked until W closed.  
**AI:** deferred — document only; not in sync mutex.

**Sequencing:** DEC-0071 W bundle must land before or with Phase 1 X.

Full record: `decisions/DEC-0072.md`

## DEC-0070 — Root README living documentation contract (US-0016)

**Status:** Accepted  
**Date:** 2026-06-08  
**Story:** US-0016  
**Research:** [R-0067](research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks), [R-0066](research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance)  
**Architecture:** `docs/engineering/architecture.md` § **US-0016**

### Context

Missing root `README.md` blocks `validate_doc_profile`. US-0016 needs a split-layout entry document with living **Product status** curated at phase boundaries. Research [R-0067](research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks) resolved template parity, status placement, and maintenance hook wording.

### Decision

1. **Template parity:** `validate_doc_profile.py --no-template-parity` while `template/` absent; drop flag only when full `template/README.md` + `template/docs/developer/README.md` land together.
2. **Product status:** `### Product status` under `## Purpose`; max **8** reverse-chronological bullets `{id} — outcome`; link backlog for history.
3. **Maintenance:** Release (post-reconciliation) append bullets + validator; refresh-context verify/update + validator when README touched; runbook § **README maintenance (US-0016)**.

**Rejected:** partial template stub; dedicated `## Product status` H2; per-commit auto-update.

### US-0017 layout extension (accepted 2026-06-09 — architecture)

Doc-only expansion under **DEC-0070** split layout — no new root H2; **no DEC-0081** (single-decision extension is simpler and sufficient):

| Surface | Contract |
|---------|----------|
| `## Examples` | Add `### Omniflow smoke (external profile)` — omniflow `curl` block per R-0078 §2; link runbook §23 |
| `## Limitations` | Add `### Troubleshooting` — Q0020 gate sequence + symptom table per R-0078 §3 |
| Maintenance | Release + refresh-context hooks: **each** closed US/BUG in the **release segment** (`Sxxxx`, `Qxxxx`, or paired intake batch) gets a Product status bullet (R-0078 §5) |
| Product status | AC-3 verify-only at execute — post-Q0020 refresh already current |
| Validator | `validate_doc_profile.py --repo . --no-template-parity` must exit **0** after execute |

**Rejected at research/architecture:** dedicated `## Troubleshooting` H2 for `(both, balanced)` profile; separate DEC-0081 record.

**Architecture ref:** `docs/engineering/architecture.md` § **US-0017**

Full record: `decisions/DEC-0070.md`

## DEC-0069 — AI merchant/category discovery tool contracts (BUG-0007)

**Status:** Accepted  
**Date:** 2026-06-07  
**Work item:** BUG-0007  
**Research:** [R-0065](research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag)  
**Architecture:** `docs/engineering/architecture.md` § **BUG-0007**

### Context

Post-BUG-0006, AI Chat fails subscription enumeration (S), category keyword resolution (T-b), and cross-signal fusion (U) despite populated mirror. LLM misuses `category_id` for keywords and `Counterparty-*` privacy hashes as enum filters; orchestrator lacks resolution guidance; audit `result_rows` always NULL.

### Decision

Freeze **A′ + E + F** within six-tool registry — no seventh tool; `allow_raw_transactions=false` default unchanged.

#### A′ — `get_transactions.category_search`

- Optional keyword param; ILIKE on `categories.name`; cap 10 matches; union aggregate filter
- `category_search` precedence over `category_id` when both supplied
- Response: `mirror_date_bounds`, `category_matches[]`, `search_attempted`, truncation flags
- **Rejected:** seventh catalog tool; payee aggregates under redaction; RAG

#### E — Orchestrator + audit

- SYSTEM_PROMPT: enumerate subscription names; use category_search for keywords; cite bounds on empty period; no Counterparty-* enums
- Populate `audit.result_rows` for get_transactions / get_subscriptions
- Enrich OpenAI parameter descriptions for opaque category_id

#### F — `get_subscriptions` schema

- `kind` enum: subscription | standing_order
- Response: `patterns_count`, `merchant_names[]` (additive)
- Server guard: reject Counterparty-* prefix in status/kind
- REST list_patterns behavior unchanged (BUG-0008 isolation)

### Consequences

- Single backend PR spanning transactions repository, two AI tools, orchestrator
- No Firefly re-sync required
- BUG-0008 coordinate: additive AI JSON only

### Risks

| Risk | Mitigation |
|------|------------|
| Broad keyword matches | Cap 10; truncated flag |
| Local model prompt drift | Schema text + LOCAL_TOOL_NUDGE |
| BUG-0008 service regression | Enrichment in AI tool wrapper only |

**Linked:** DEC-0032, DEC-0035, DEC-0034, BUG-0006, BUG-0008 (coordinate), R-0065, R-0041

## DEC-0068 — Grafana analytics provisioning contract (BUG-0009)

**Status:** Accepted  
**Date:** 2026-06-06  
**Work item:** BUG-0009  
**Research:** [R-0064](research.md#r-0064--bug-0009-grafana-panel-emptiness-vs-cross-account-overview-gap)  
**Architecture:** `docs/engineering/architecture.md` § **BUG-0009**

### Context

Post-BUG-0004 omniflow deploy: Grafana ds/query returns **200** with data, but operators perceive empty panels because (1) `$account_id` defaults alphabetically to zero-balance acct 116, (2) portfolio breakdown SQL truncates to 1/3 accounts via erroneous global `LIMIT 1`, (3) no cross-account overview panel in analytics provisioning, (4) ML panels empty on baseline-only profile (DEC-0049) without honest empty-state.

### Decision

Freeze provisioning-only contracts for BUG-0009 execute — no backend changes unless sprint-plan adds optional docs.

#### Y1 — Account variable default

- **Query:** `ORDER BY ABS(COALESCE(a.balance, 0)) DESC, a.name ASC` on `accounts WHERE type = 'asset'`
- **Dashboards:** `cashflow.json`, `forecast-horizons.json`
- **Provisioning:** Omit `current` block — Grafana selects first query row on load — **superseded by DEC-0108 (BUG-0019):** `current` is now required with empty text/value shape (`{"text": "", "value": ""}`); ABS sort and the rest of this contract unchanged
- **Rejected:** Hardcoded `current`; first-non-zero-forecast subquery; React iframe var passthrough (defer)

#### Z1 — Portfolio breakdown SQL

- **Pattern:** Latest snapshot in subquery (`ORDER BY snapshot_date DESC LIMIT 1`) → `CROSS JOIN LATERAL jsonb_array_elements(payload->'accounts')`
- **Rejected:** Global `LIMIT 1` on cross-join; `DISTINCT ON` workaround

#### Z2 — Cross-account overview

- **Placement:** Portfolio dashboard (`uid: portfolio`) only — stat row + all-accounts table using Z1 SQL
- **AC Z:** Stat row + table satisfies acceptance; React `/wealth` link supplementary (Z3 docs), not sole fix
- **Rejected:** Duplicate overview on every dashboard; seventh landing dashboard; Grafana dynamic hide rules

#### Y2 — ML empty-state

- **Mechanism:** Text banner above ML section + `noValue: "ML unavailable"` on ML panels
- **Boundary:** US-0013 owns ML enablement; `$forecast_variant` default stays `baseline`
- **Rejected:** Dynamic panel hide/show (Grafana 11 complexity); merging US-0013 sidecar work

### Consequences

- Execute touches Grafana provisioning JSON + SQL test fixtures only
- Negative overdrawn Giro correctly prioritized via ABS sort (DEC-0065)
- All-zero balance deploy still alphabetical default — documented edge case
- Manual Grafana UI save may bake `current` — runbook warning required at execute

### Risks

| Risk | Mitigation |
|------|------------|
| All-zero balances → alphabetical default | Document; acceptable MVP edge |
| Overview portfolio-only | Sidebar label + Z3 docs; not AC blocker |
| ML charts empty below banner | Expected until US-0013; banner sets copy |
| Accidental `current` bake-in | Execute runbook: reprovision from JSON, don't save variables in UI |

**Linked:** DEC-0009, DEC-0049, DEC-0055, DEC-0057, DEC-0065, DEC-0066, R-0064, BUG-0004, BUG-0010, US-0011, US-0013

## Canonical full records

- `decisions/DEC-0001.md` — OIDC IdP default
- `decisions/DEC-0002.md` — Firefly sync watermark strategy
- `decisions/DEC-0003.md` — External PostgreSQL startup retry policy
- `decisions/DEC-0004.md` — Firefly read-only connector enforcement
- `decisions/DEC-0005.md` — US-0001 relational mirror schema scope
- `decisions/DEC-0006.md` — SPA JWT vs BFF auth pattern
- `decisions/DEC-0007.md` — Hybrid rule-based forecast algorithm
- `decisions/DEC-0008.md` — Precomputed hypertable forecast snapshots
- `decisions/DEC-0009.md` — Per-account forecast scope default
- `decisions/DEC-0010.md` — Sync-post forecast recompute mutex extension
- `decisions/DEC-0011.md` — Forecast computation retention policy
- `decisions/DEC-0012.md` — Grafana scarcity threshold and dashboard uids
- `decisions/DEC-0013.md` — Shared recurrence core extraction
- `decisions/DEC-0014.md` — Subscription confidence tiers
- `decisions/DEC-0015.md` — Subscription persistence schema and rejection semantics
- `decisions/DEC-0016.md` — Dauerauftrag rule-based classification
- `decisions/DEC-0017.md` — Price-change dual threshold defaults
- `decisions/DEC-0018.md` — Sync pipeline subscriptions phase before forecast
- `decisions/DEC-0019.md` — Plan delta overlay on forecast baseline
- `decisions/DEC-0020.md` — Plan version semantics and cap
- `decisions/DEC-0021.md` — Plan-vs-Ist daily net cashflow metric
- `decisions/DEC-0022.md` — Plan persistence migration 004
- `decisions/DEC-0023.md` — Plan recompute triggers
- `decisions/DEC-0024.md` — Active plan and Grafana Dashboard 3
- `decisions/DEC-0025.md` — Net worth aggregation and daily snapshots
- `decisions/DEC-0026.md` — Alert Engine evaluation rules
- `decisions/DEC-0027.md` — Alert persistence and lifecycle
- `decisions/DEC-0028.md` — Sync pipeline alerts phase
- `decisions/DEC-0029.md` — Threshold centralization (supersedes DEC-0012 hardcode)
- `decisions/DEC-0030.md` — Unified inbox UI and Dashboard 4 partial
- `decisions/DEC-0031.md` — AI orchestration and tool registry
- `decisions/DEC-0032.md` — Privacy layer defaults and central redaction
- `decisions/DEC-0033.md` — Chat SSE API and ephemeral sessions
- `decisions/DEC-0034.md` — AI audit log persistence and retention
- `decisions/DEC-0035.md` — Six-tool service mapping and payload limits
- `decisions/DEC-0036.md` — React chat UX (Sheet drawer + /chat)
- `decisions/DEC-0037.md` — Exchange connector trait and read-only scope
- `decisions/DEC-0038.md` — Hybrid portfolio PnL methodology
- `decisions/DEC-0039.md` — FX conversion two-layer model
- `decisions/DEC-0040.md` — Exchange API secret storage pattern
- `decisions/DEC-0041.md` — Sync pipeline exchanges phase
- `decisions/DEC-0042.md` — Migration 007 and snapshot extension
- `decisions/DEC-0043.md` — AI provider factory and unified HTTP client
- `decisions/DEC-0044.md` — Three AI provider modes and TOML schema
- `decisions/DEC-0045.md` — Local provider HTTP request quirks
- `decisions/DEC-0046.md` — Local tool-calling fallback policy
- `decisions/DEC-0047.md` — Settings provider status and test endpoint
- `decisions/DEC-0048.md` — Audit provider column and AC5 wiremock verification
- `decisions/DEC-0049.md` — StatsForecast Python sidecar
- `decisions/DEC-0050.md` — ML overlay model_kind discriminator
- `decisions/DEC-0051.md` — Seasonal model selection ladder
- `decisions/DEC-0052.md` — Sync forecast_ml phase integration
- `decisions/DEC-0053.md` — API variant param and confidence bands
- `decisions/DEC-0054.md` — Deterministic plan risk score
- `decisions/DEC-0055.md` — Grafana Dashboard 5 ML extensions
- `decisions/DEC-0056.md` — Omniflow external deploy (US-0010)
- `decisions/DEC-0057.md` — Unified analytics Grafana proxy (US-0011)
- `decisions/DEC-0058.md` — Database bootstrap on first start (US-0012)
- `decisions/DEC-0059.md` — Firefly mirror amount sign normalization (BUG-0006)
- `decisions/DEC-0060.md` — Firefly account balance parse (BUG-0004)
- `decisions/DEC-0061.md` — Subscription payee key fallbacks (BUG-0004)
- `decisions/DEC-0062.md` — Bitunix dual REST auth (BUG-0005)
- `decisions/DEC-0063.md` — Bitunix futures enable policy (BUG-0005)
- `decisions/DEC-0064.md` — Futures wallet vs position wealth accounting (BUG-0005)
- `decisions/DEC-0065.md` — Negative asset wealth visibility (BUG-0010)
- `decisions/DEC-0066.md` — ML disabled metadata posture (BUG-0010)
- `decisions/DEC-0067.md` — Component-level monthly forecast bucket attribution (BUG-0012)
- `decisions/DEC-0068.md` — Grafana analytics provisioning contract (BUG-0009)
- `decisions/DEC-0069.md` — AI merchant/category discovery tool contracts (BUG-0007)
- `decisions/DEC-0079.md` — Budgets MTD upper date bound (BUG-0013 AL)
- `decisions/DEC-0080.md` — Bitunix futures wallet parse and linear unrealized EUR (BUG-0013 AN/AK)
- `decisions/DEC-0081.md` — Wealth all-holdings display and unified FX incomplete (BUG-0014 AQ)
- `decisions/DEC-0082.md` — Block delete of active plan (BUG-0014 AS1)
- `decisions/DEC-0083.md` — Planning adjustment target_type UI alignment (BUG-0014 AS2)
- `decisions/DEC-0087.md` — Category expense-series API and uncategorized sentinel (US-0018)
- `decisions/DEC-0088.md` — CategoryFilter and bar trend chart UX (US-0018)
- `decisions/DEC-0089.md` — Cross-surface category filter semantics and Grafana independence (US-0018)
- `decisions/DEC-0090.md` — Category query index deferral policy (US-0018)
- `decisions/DEC-0091.md` — Goal balance plan schema and template (US-0019)
- `decisions/DEC-0092.md` — Per-plan goal-stats API and feasibility copy (US-0019)
- `decisions/DEC-0093.md` — Category-scoped plan overlay semantics (US-0019)
- `decisions/DEC-0094.md` — Deterministic category savings suggestions (US-0019)
- `decisions/DEC-0095.md` — Goal account scope for balance projection (US-0019)
- `decisions/DEC-0096.md` — Plan vs Actual household scope unchanged (US-0019)
- `decisions/DEC-0097.md` — AI category savings tool optional REST primary (US-0019)
- `decisions/DEC-0098.md` — Subscription discover explorer API (US-0020)
- `decisions/DEC-0099.md` — Manual confirm-from-discover (US-0020)
- `decisions/DEC-0100.md` — Subscription display majority category (US-0020)
- `decisions/DEC-0101.md` — Operator tag schema (US-0020)
- `decisions/DEC-0102.md` — Subscription tag assign and list filter (US-0020)
- `decisions/DEC-0103.md` — Grafana subscriptions `$tag` variable P2 (US-0020)
- `decisions/DEC-0104.md` — Axum SPA index.html fallback (BUG-0016)
- `decisions/DEC-0105.md` — ai_tool_audit CHECK extension (BUG-0017)
- `decisions/DEC-0106.md` — paired_baseline_id CASCADE + retention order (BUG-0017)
- `decisions/DEC-0107.md` — scarcity JOIN column qualification (BUG-0018)
