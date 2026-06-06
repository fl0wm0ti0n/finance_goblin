# Decisions

## Current context pack

- **Latest released:** US-0012 / S0012 (`0.12.0-us0012`, 2026-06-03) — database bootstrap on first start (`ensure_database`, optional `DATABASE_BOOTSTRAP_URL`, DEC-0058)
- **Latest bug fix:** BUG-0007 / Q0017 (2026-06-07) — AI merchant/category discovery (DEC-0069 A′+E+F, S privacy label exemption); verify-work + release PASS
- **Prior bug fix:** BUG-0009 / Q0016 (2026-06-06) — Grafana provisioning-only analytics fix (DEC-0068 ABS balance default, portfolio LATERAL breakdown, ML banner); verify-work + release PASS
- **Prior bug fix:** BUG-0012 / Q0014+Q0015 (2026-06-06) — monthly Income/Fixed bucket attribution (DEC-0067 component-level monthly_map); verify-work + release PASS
- **Prior bug fix:** BUG-0010 / Q0013 (2026-06-05) — forecast/wealth/ML posture (DEC-0065 negative wealth, DEC-0066 ML disabled metadata); verify-work + release PASS
- **Prior bug fix:** BUG-0006 / Q0010 (2026-06-05) — Firefly category/date/amount ingest (DEC-0059), aggregate contract (R-0060 fulfilled), verify-work + release PASS
- **Prior bug fix:** BUG-0005 / Q0012 (2026-06-05) — Bitunix dual REST auth (DEC-0062), futures enable policy (DEC-0063), wallet vs position wealth (DEC-0064), verify-work + release PASS
- **Prior bug fix:** BUG-0004 / Q0011 (2026-06-05) — ExchangesOnly sync terminal status (I1), portfolio UNION SQL (K1), Firefly balance parse (DEC-0060), wealth NULL handling (L2), subscription payee fallbacks (DEC-0061); verify-work + release PASS
- **Prior bug fix:** BUG-0003 / Q0009 (2026-06-05) — `DATABASE_HOST=postgres` ops recovery (F1), env guard docs (F2), `effective_enabled` connector registry (G1); verify-work + release PASS; G2 futures auth spike skipped (gated)
- **Prior bug fix:** BUG-0002 / Q0008 (2026-06-05) — Firefly PAT guard, risk-score 200 empty-state, exchange settings; verify-work + release PASS
- **Prior bug fix:** BUG-0001 / Q0007 (2026-06-04) — `DevBypassAuthProvider` + `GF_SERVER_ROOT_URL`; DEC-0057 follow-through
- **Open bug queue:** BUG-0008, BUG-0011
- **Open epics:** US-0013 (ML hardening), US-0014 (planning UX), US-0015 (AI bucket mapping)
- **Active bug:** none
- **Recommended next:** `/auto bug-target=BUG-0008` phase=discovery (intake complete; W/X sub-defects)
- **Decision gate:** none open
- **Backlog drain:** `AUTO_BACKLOG_DRAIN=1` — **complete** (12/12 released; defect queue active)

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
- **Provisioning:** Omit `current` block — Grafana selects first query row on load
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
