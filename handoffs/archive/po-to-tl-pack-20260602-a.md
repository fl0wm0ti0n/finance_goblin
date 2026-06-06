# PO to TL archive pack (2026-06-02-a)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500` (post-prepend US-0011 discovery)
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest contiguous tail): 4
- Retained units in hot file: 5
- First archived heading: `## discovery-20260601-us0009 — US-0009 advanced forecasting UX discovery`
- Last archived heading: `## sprint-plan-20260601-s0009 — US-0009 advanced forecasting sprint decomposition`
- Verification tuple (mandatory):
  - archived_body_lines=250
  - retained_body_lines=221

## discovery-20260601-us0009 — US-0009 advanced forecasting UX discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-01  
**Story:** US-0009  
**Next phase:** `/research`

### Summary

Discovery captured Phase 7 **Advanced Forecasting** design/UX references. US-0009 extends the released US-0002 rule-based Forecast Engine (`DEC-0007`) with an optional **ML-enhanced overlay** (seasonal statistical models, 6–24 month projections with p10–p90 confidence bands), **Baseline vs ML-enhanced** comparison on `/forecast` Long-term tab, portfolio performance outlook when US-0007 exchange snapshots exist, **plan-scenario risk score** (0–100) on `/planning`, and Grafana **Dashboard 5** panel extensions. Baseline computations remain authoritative default; ML pass runs after baseline in sync mutex when history thresholds met. Builds on US-0002 hypertables (R-0007), US-0004 plan overlays (R-0015), US-0007 portfolio snapshots, US-0005 plan viability semantics (R-0022), and Dashboard 5 provisioning (R-0008).

### UX references captured (see `docs/product/vision.md`)

| Reference | US-0009 application |
|-----------|---------------------|
| **US-0002 `/forecast`** | Long-term tab gains confidence band area chart + Baseline \| ML \| Compare segmented control; Monthly tab seasonal callout |
| **US-0004 `/planning`** | Risk score badge on Scenarios; Compare tab risk column per version |
| **US-0007 `/wealth` Crypto** | Portfolio outlook stat row (3/6/12 mo projected EUR) when exchanges connected |
| **Grafana Dashboard 5** | Extend `forecast-horizons.json` — ML/baseline overlay, band panel, seasonal stat, optional portfolio row |
| **Projectplan Phase 7** | ML, saisonale Modelle, Portfolio Prognosen, Risikobewertungen |
| **R-0043 (discovery research)** | Layered baseline + StatsForecast overlay; minimum history gate; deterministic risk index |

### Scope refinements (backlog updated)

- **Layered forecast:** `model_kind=baseline` (DEC-0007) + optional `model_kind=ml_enhanced` per sync; never replace baseline
- **Seasonal:** Detect periods on net-cashflow; metadata + Monthly tab callout
- **ML horizons:** 6/12/24 months with p10–p90 bands on ECharts long-term view
- **Compare UX:** Baseline vs ML-enhanced vs dual-series Compare on Long-term tab
- **Portfolio forecast:** Snapshot-driven EUR projection on Wealth Crypto tab
- **Risk score:** 0–100 on active plan scenario from balance stress + viability-style signals (+ optional crypto volatility proxy)
- **Grafana:** Dashboard 5 extensions only; uid unchanged (DEC-0012)
- **Out of scope added:** external cloud ML APIs, training/MLOps UI, new AI tools, trading/tax (unchanged)

### Discovery decomposition evidence

- Feature/workflow count: seasonal + ML bands + compare UI + portfolio outlook + risk score + Grafana + sync ML pass (high breadth)
- **Split decision:** single story retained — Compare AC and risk score depend on same ML computation IDs and sync ordering; splitting "ML core" vs "portfolio/risk" would block independent acceptance
- Cross-cutting impact: forecast engine, hypertable/schema, React forecast/planning/wealth, Grafana JSON, optional Python stats sidecar (research)
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md#US-0009`)
- Risk surface: sparse history instability, mutex latency, ML/baseline divergence without Compare, FX gaps on portfolio (R-0034), MLOps scope creep

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#us-0009` | Discovery refinements + decomposition evidence + open questions | pass |
| `docs/product/acceptance.md` US-0009 | 6 criteria still valid; map to discovery scope without AC rewrite | pass |
| `docs/product/vision.md` | US-0009 discovery notes + UX refs appended | pass |
| `docs/engineering/research.md` | R-0043 discovery entry linked | pass |
| US-0002 released boundary | Baseline preserved; ML is overlay not replacement | pass |
| US-0008 / AI tools | No new chat tools; `get_forecast` display extension only (research) | pass |

`triad_hot_surface`: check pass (rollover 4 sections → `handoffs/archive/po-to-tl-pack-20260531-j.md`; US-0009 discovery at tail; retained_body_lines=486, pack_ref=handoffs/archive/po-to-tl-pack-20260531-j.md)

### Open questions (carry to research/architecture)

- StatsForecast Python sidecar vs Rust stats crate vs subprocess?
- Minimum history months before ML pass; `ml_skipped` metadata shape?
- Hypertable schema: `model_kind` discriminator vs separate ML tables?
- Seasonal: MSTL/AutoETS vs simpler month-of-year factors?
- Portfolio series: total EUR snapshots vs per-asset forecasts?
- Risk score weights and thresholds vs reusing alert `plan_viability` only?
- Grafana `$forecast_variant` variable and panel SQL patterns?
- `get_forecast` tool: single payload with both variants or query param?

### Recommended next steps

1. `/research` — Spike StatsForecast sidecar footprint, MSTL on household monthly series, schema extension on R-0007, portfolio snapshot series, deterministic risk index formula (extends R-0006, R-0007, R-0008, R-0022; R-0043)
2. `/architecture` — `model_kind` computation model, sync mutex extension, API contracts for compare/bands/risk, Grafana panel queries, DEC-xxxx for ML enablement defaults
3. `/sprint-plan` — Decompose 6 AC after architecture (expect ≥10 tasks; consider sprint split only if architecture proves separable deploy paths)

---

## research-20260601-us0009 — US-0009 advanced forecasting technical research

**From:** Tech Lead  
**To:** Architecture (`/architecture`)  
**Date:** 2026-06-01  
**Story:** US-0009  
**Next phase:** `/architecture`

### Summary

Web research completed for US-0009 advanced forecasting. Extended discovery entry **R-0043** and added eight focused entries **R-0044–R-0051** covering StatsForecast sidecar execution, seasonal model ladder, confidence bands, portfolio outlook, deterministic risk score, migration 009 schema, sync mutex integration, and Grafana Dashboard 5 extensions.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Execution model** | [R-0044](docs/engineering/research.md#r-0044--statsforecast-sidecar-vs-rust-augurs-execution-model) | Python StatsForecast FastAPI sidecar in Compose `full` profile; `[forecast_ml] enabled = false` default; augurs-in-Rust deferred |
| **Seasonal ML overlay** | [R-0045](docs/engineering/research.md#r-0045--seasonal-model-selection-autoets-mstl-fallback) | AutoETS(`season_length=12`) for 12–23 mo; MSTL when ≥24 mo + strength ≥0.35; integrate monthly deltas onto baseline daily path |
| **Confidence bands** | [R-0046](docs/engineering/research.md#r-0046--ml-confidence-bands-storage-api-and-echarts-mapping) | StatsForecast `level=[90]` → `balance_p10`/`balance_p90` on existing hypertable; `variant` query param on long-term API |
| **Portfolio outlook** | [R-0047](docs/engineering/research.md#r-0047--portfolio-performance-forecast-from-exchange-snapshots) | `portfolio_pnl_snapshots.crypto_value_eur` weekly series; ≥8 points; `forecast_portfolio_weekly` hypertable; Wealth Crypto tab |
| **Plan risk score** | [R-0048](docs/engineering/research.md#r-0048--deterministic-plan-scenario-risk-score-0-100) | Deterministic 0–100 index: 45% balance stress + 40% plan viability + 15% crypto CV; `plan_risk_scores` table |
| **Migration schema** | [R-0049](docs/engineering/research.md#r-0049--migration-009-schema-for-ml-overlay-bands-portfolio-risk) | `model_kind` + `paired_baseline_id` on `forecast_computations`; nullable band columns; retain 5 per kind (DEC-0011) |
| **Sync integration** | [R-0050](docs/engineering/research.md#r-0050--sync-mutex-ml-phase-integration-and-history-gates) | ML sub-phase after baseline + plan hook inside mutex; phase `forecast_ml`; ML failure never fails sync |
| **Grafana Dashboard 5** | [R-0051](docs/engineering/research.md#r-0051--grafana-dashboard-5-ml-extensions-and-forecast_variant) | `$forecast_variant` variable; band + seasonal + portfolio panels; uid `forecast-horizons` unchanged |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| StatsForecast sidecar vs Rust crate vs subprocess? | **Python sidecar** in `full` profile (R-0044); subprocess rejected; augurs deferred |
| Minimum history before ML pass? | **≥12 monthly** net-cashflow points; portfolio **≥8 weekly** (R-0045, R-0047, R-0050) |
| Hypertable schema approach? | **`model_kind` discriminator** + nullable band columns on existing tables (R-0049) — not separate ML hypertables |
| MSTL/AutoETS vs month-of-year rules? | **AutoETS default**; MSTL when ≥24 months; SeasonalNaive fallback (R-0045) |
| Portfolio series source? | **`crypto_value_eur`** from `portfolio_pnl_snapshots` (R-0047) — not per-asset |
| Risk score approach? | **Deterministic weighted index** reusing R-0022 viability semantics (R-0048) — not ML classifier |
| Grafana panel strategy? | **Extend existing JSON** with `$forecast_variant`; new row of panels (R-0051) |
| `get_forecast` tool payload? | Optional **`variant`** query param; default `baseline`; bands in summary when ML (R-0046) |

### Risks surfaced (carry to architecture)

1. **Sidecar optional runtime** — operators on `minimal`/`standard` profiles see baseline-only; ML disabled without error (R-0044, R-0050)
2. **Mutex latency growth** — baseline + ML + exchanges; monitor 30s budget (R-0050, DEC-0010)
3. **Sparse history instability** — MSTL overfit <24 months; WMAPE gate + `low_confidence` UI (R-0045)
4. **Symmetric prediction intervals** — p10/p90 may understate skewed spend tails (R-0046)
5. **FX incomplete crypto** — portfolio forecast with warning banner, not hard skip (R-0047, R-0034)
6. **Baseline authority** — alerts, plan hook, AI default must stay on `model_kind=baseline` (R-0050, DEC-0023)

### Recommended next steps

1. `/architecture` — ML overlay service contract, migration 009, sync sub-phase, API compare/bands/risk endpoints, sidecar Compose service, DEC-xxxx for `[forecast_ml]` defaults and model_kind retention
2. `/sprint-plan` — Decompose 6 AC (expect ≥10 tasks; single sprint unless architecture finds separable path)

---

## architecture-20260601-us0009 — US-0009 advanced forecasting architecture

**From:** Tech Lead  
**To:** Sprint planning (`/sprint-plan`)  
**Date:** 2026-06-01  
**Story:** US-0009  
**Next phase:** `/sprint-plan`

### Summary

Architecture defined for Phase 7 **Advanced Forecasting** over frozen US-0002 baseline (DEC-0007). Seven decisions (DEC-0049–DEC-0055). Spec-pack 3/3. ML overlay via Python StatsForecast sidecar (`full` profile, disabled by default); baseline remains authoritative for alerts, plan hook, and AI defaults.

### Architecture highlights

| Area | Decision |
|------|----------|
| **Sidecar** | Python FastAPI `stats-forecast` in Compose `full`; `[forecast_ml] enabled = false` default (DEC-0049, R-0044) |
| **ML overlay** | `model_kind=ml_enhanced` layered on baseline daily path; `paired_baseline_id` (DEC-0050, R-0043) |
| **Seasonal models** | AutoETS 12–23 mo; MSTL ≥24 mo + strength; SeasonalNaive fallback (DEC-0051, R-0045) |
| **Migration 009** | `model_kind`, band columns, `forecast_portfolio_weekly`, `plan_risk_scores` (R-0049) |
| **Sync phase** | `forecast_ml` after baseline + plan hook; ML failure never fails sync (DEC-0052, R-0050) |
| **API variant** | `variant=baseline\|ml_enhanced` on long-term; `/compare`; extended `/meta` (DEC-0053, R-0046) |
| **React Compare** | Baseline \| ML \| Compare on Long-term tab; ECharts bands; Monthly seasonal callout |
| **Plan risk score** | Deterministic 0–100 weighted index; components JSON (DEC-0054, R-0048) |
| **Portfolio outlook** | `crypto_value_eur` weekly series; Wealth Crypto 3/6/12 mo (R-0047) |
| **Grafana Dashboard 5** | `$forecast_variant`; band + seasonal + portfolio + risk panels; uid unchanged (DEC-0055, R-0051) |
| **Baseline authority** | Alerts, plan hook (DEC-0023), AI `get_forecast` default = baseline |

### Decisions created

- **DEC-0049** — StatsForecast Python sidecar; full profile; disabled by default
- **DEC-0050** — ML overlay `model_kind` discriminator; baseline authoritative
- **DEC-0051** — Seasonal model selection ladder (AutoETS/MSTL/SeasonalNaive)
- **DEC-0052** — Sync `forecast_ml` phase integration and failure semantics
- **DEC-0053** — API `variant` param, compare endpoint, confidence band columns
- **DEC-0054** — Deterministic plan-scenario risk score (0–100)
- **DEC-0055** — Grafana Dashboard 5 ML extensions and `$forecast_variant`

### Spec-pack status

| Artifact | Status |
|----------|--------|
| `US-0009-design-concept.md` | Complete |
| `US-0009-crs.md` | Complete |
| `US-0009-technical-specification.md` | Complete |

### Triad check (architecture phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#us-0009` | Architecture aligns with discovery scope; baseline preserved | pass |
| `docs/product/acceptance.md` US-0009 | 6 AC mapped in architecture + spec-pack | pass |
| `docs/engineering/architecture.md` US-0009 | Sidecar, ML overlay, migration 009, sync, API, UI, risk, Grafana | pass |
| `docs/engineering/decisions.md` | DEC-0049–DEC-0055 indexed + canonical records | pass |
| US-0002 released boundary | Baseline authoritative; ML is overlay not replacement | pass |
| US-0006 / AI tools | No new chat tools; `get_forecast` variant extension only | pass |

`triad_hot_surface`: check pass (architecture section at po_to_tl tail; spec-pack 3/3; state checkpoint updated)

### Risks carried to sprint-plan

1. Sidecar optional runtime — document `full` profile + `enabled=true` in user guide (DEC-0049)
2. Mutex latency — baseline + ML + portfolio <30s budget; log sub-phase ms (DEC-0052)
3. Sparse history — WMAPE gate + low_confidence UI; MSTL only ≥24 mo (DEC-0051)
4. Compare UX mandatory — disable ML/Compare when skipped with reason (AC6)
5. Baseline authority — code review gate on alert/plan computation_id paths (DEC-0050)

### Recommended next steps

1. `/sprint-plan` — decompose 6 AC (expect ≥10 tasks; consider split only if separable deploy paths emerge)
2. `/plan-verify` — confirm task coverage against acceptance after sprint-plan

---

## sprint-plan-20260601-s0009 — US-0009 advanced forecasting sprint decomposition

**From:** Tech Lead  
**To:** QA (`/plan-verify`) → Dev (`/execute`)  
**Date:** 2026-06-01  
**Story:** US-0009  
**Sprint:** S0009  
**Next phase:** `/plan-verify`

### Summary

Created sprint **S0009** with **12 tasks** (T-0097–T-0108) covering all six US-0009 acceptance criteria. No sprint split — exactly at SPRINT_MAX_TASKS=12; discovery retained single story because Compare AC and risk score depend on same ML computation IDs and sync ordering.

### Task summary

| ID | Title |
|----|-------|
| T-0097 | Migration 009 forecast ML schema |
| T-0098 | StatsForecast sidecar and Compose full profile |
| T-0099 | Forecast ML overlay service |
| T-0100 | Sync forecast_ml phase integration |
| T-0101 | Forecast API variant compare and meta |
| T-0102 | React forecast Compare UI with ECharts bands |
| T-0103 | Plan risk score service and Planning UI |
| T-0104 | Portfolio outlook API and Wealth Crypto UI |
| T-0105 | Grafana Dashboard 5 ML extensions |
| T-0106 | Forecast ML unit and integration tests |
| T-0107 | Monthly seasonal callout and get_forecast variant |
| T-0108 | Operator user guide US-0009 |

### Split decision

- **Why 12 tasks:** Maps architecture modules + three UI surfaces + Grafana + tests + user guide — one deployable increment per task.
- **Why not S0009a/b:** ML core without Compare UI fails AC6; portfolio/risk without ML computation IDs blocks independent acceptance.
- **USER_GUIDE_MODE=1:** T-0108 → `docs/user-guides/US-0009.md`.

### Artifacts created

- `sprints/S0009/sprint.md`, `sprint.json`, `tasks.md`, `progress.md`
- `sprints/S0009/uat.md`, `uat.json`, `plan-verify.json` (PENDING)
- `handoffs/tl_to_dev.md`, `docs/engineering/state.md` checkpoint

### Recommended next steps

1. `/plan-verify` — validate AC coverage in `sprints/S0009/plan-verify.json`
2. `/execute` — implement T-0097 through T-0108 in dependency order

---
