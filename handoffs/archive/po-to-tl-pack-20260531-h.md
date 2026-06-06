# PO to TL archive pack (2026-05-31)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 4
- Retained units in hot file: 9
- First archived heading: `## discovery-20260602-us0008 — US-0008 local AI provider UX discovery`
- Last archived heading: `## research-20260601-us0007 — US-0007 crypto exchange portfolio technical research`
- Verification tuple (mandatory):
  - archived_body_lines=237
  - retained_body_lines=495

---

## discovery-20260602-us0008 — US-0008 local AI provider UX discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-02  
**Story:** US-0008  
**Next phase:** `/research`

### Summary

Discovery captured design/UX references for local & self-hosted AI provider support (Phase 6). US-0008 extends the US-0006 **AI Tool Layer** with a provider factory behind the existing stub `AiProvider` trait — **no changes** to six tools, PrivacyLayer, orchestrator loop, or chat SSE contract. Delivers three provider modes (`openai`, `ollama`, `openai_compatible`), TOML config extension, Settings **AI & Privacy** provider display + test connection, chat **Local vs Cloud** badge, Compose `full` profile Ollama wiring, and local E2E verification (no outbound OpenAI when local selected). Builds on released US-0006 chat UI, settings API, and R-0027/R-0029 OpenAI-compatible path assumptions.

### UX references captured (see `docs/product/vision.md`)

| Reference | US-0008 application |
|-----------|---------------------|
| **Ollama** | Default Compose `full` service; OpenAI-compatible `/v1/chat/completions` + tools; CLI model pull — no in-app catalog |
| **LM Studio** | Host-run OpenAI-compatible server → `openai_compatible` + configurable `base_url` |
| **LocalAI / vLLM** | Same OpenAI-compatible contract; optional bearer when gateway requires auth |
| **US-0006 chat** | Shared `ChatPanel`; provider swap backend-only; privacy badge + new provider badge |
| **US-0007 exchange Settings** | Test-connection button pattern; read-only TOML display; status badges (R-0035) |
| **Projectplan Phase 6** | Privacy-first Q&A never leaves infrastructure when local provider active |

### Scope refinements (backlog updated)

- **Provider modes:** `openai` (existing path), `ollama` (default internal `http://ollama:11434/v1`), `openai_compatible` (LM Studio / LocalAI / vLLM via `base_url`)
- **Tool layer frozen:** Six tools + PrivacyLayer + audit log semantics unchanged — HTTP client layer only
- **Config:** Extend `[ai]` with `provider`, `base_url`, optional `api_key_env`; restart required to switch
- **Settings:** Provider table + status badge + **Test AI provider** POST endpoint; no secrets rendered
- **Chat UX:** Provider badge (`Local · Ollama` vs `Cloud · OpenAI`); disabled input + Alert when misconfigured
- **Compose:** Document `full` profile Ollama + backend dependency when `provider = "ollama"`
- **Out of scope unchanged:** model fine-tuning, GPU orchestration beyond Compose, in-app model pull UI, runtime config edit, token vault (DEC-0032), ML forecasts (US-0009)

### Discovery decomposition evidence

- Feature/workflow count: provider factory + 3 paths + config + settings test + chat badge + compose docs + E2E local verification (moderate — single story retained)
- Cross-cutting impact: provider module, config/settings API, orchestrator factory, React Settings + ChatPanel, compose operator docs
- Acceptance breadth: unchanged (5 criteria in `docs/product/acceptance.md#US-0008`)
- Risk surface: local tool-calling reliability (R-0027), endpoint variance (`tool_choice`, parallel tools, stream deltas), context window vs 8 KB payloads, SSE auth (R-0029), model-management scope creep

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#us-0008` | Discovery refinements + decomposition evidence present | pass |
| `docs/product/acceptance.md` US-0008 | 5 criteria still valid; no AC change required at discovery | pass |
| `docs/product/vision.md` | US-0008 UX refs + discovery notes appended | pass |
| US-0006 released boundary | Tool layer + privacy unchanged across providers (AC4) | pass |
| `backend/src/ai/provider.rs` | Stub trait exists; extension point confirmed | pass |

`triad_hot_surface`: pending rollover after prepend (see checkpoint)

### Open questions (carry to research/architecture)

- Recommended Ollama models for reliable six-tool function calling?
- Orchestrator behavior when local model returns no `tool_calls` or malformed arguments?
- Unified `OpenAiCompatibleProvider` vs separate Ollama type?
- Optional `api_key_env` for local (dummy `ollama` key)?
- Settings API response fields for provider status?
- Compose `depends_on: ollama` conditional vs always in `full` profile?
- Audit row provider metadata for traceability?
- AC5 network isolation verification approach?

### Recommended next steps

1. `/research` — Ollama/LM Studio OpenAI-compatible tool-calling compatibility, recommended local models, streaming delta variance, config schema, compose wiring (extends R-0027, R-0029)
2. `/architecture` — Provider factory trait methods, `[ai]` config extension, settings + test endpoint, orchestrator injection, DEC-xxxx for provider enum defaults
3. `/sprint-plan` — S0008 decomposition for US-0008 acceptance criteria

---

## sprint-plan-20260601-s0007 — US-0007 crypto exchange portfolio sprint

**From:** Tech Lead  
**To:** QA (`/plan-verify`) → Dev (`/execute`)  
**Date:** 2026-06-01  
**Story:** US-0007  
**Sprint:** S0007  
**Next phase:** `/plan-verify`

### Summary

Sprint **S0007** created with **12 tasks** (T-0073 … T-0084) covering migration 007, ExchangeConnector (Binance/Bybit/Bitunix), Portfolio Engine hybrid PnL, FxService, sync `"exchanges"` phase, wealth/portfolio REST API, React `/wealth` Crypto tab, Settings read-only exchange section, Sync Status per-exchange rows, `allocation_target` plan template, Grafana Dashboard 4 completion, tests, and operator user guide. No sprint split — at SPRINT_MAX_TASKS=12 threshold.

### Task summary

| ID | Title |
|----|-------|
| T-0073 | SQLx migration 007 exchanges portfolio schema |
| T-0074 | Exchange portfolio config and FxService EUR conversion |
| T-0075 | ExchangeConnector trait HTTP layer and repository |
| T-0076 | Binance Bybit and Bitunix connector implementations |
| T-0077 | Portfolio Engine hybrid PnL and baselines |
| T-0078 | Sync pipeline exchanges phase and ExchangeService |
| T-0079 | Extended wealth portfolio REST API and allocation_target template |
| T-0080 | React /wealth Crypto tab and Overview extension |
| T-0081 | Settings crypto exchanges and Sync Status per-exchange rows |
| T-0082 | Grafana Dashboard 4 completion |
| T-0083 | Exchange portfolio and get_portfolio tests |
| T-0084 | Operator user guide |

### Recommended next steps

1. `/plan-verify` — AC coverage validation
2. `/execute` — T-0073 through T-0084

---

## architecture-20260601-us0007 — US-0007 crypto exchange portfolio architecture

**From:** Tech Lead  
**To:** Dev (via `/execute` after `/plan-verify`)  
**Date:** 2026-06-01  
**Story:** US-0007  
**Next phase:** `/plan-verify` → `/execute`

### Summary

Architecture defined for US-0007 crypto exchange portfolio integration. Unified `ExchangeConnector` trait (Binance/Bybit/Bitunix), Portfolio Engine hybrid PnL, FxService EUR conversion, migration 007 schema, sync `"exchanges"` phase before alerts, extended wealth API + React `/wealth` Crypto tab, Settings read-only exchange status, plan `allocation_target` template, Grafana Dashboard 4 completion. Six decisions (DEC-0037–DEC-0042). Spec-pack 3/3.

### Architecture highlights

| Area | Decision |
|------|----------|
| **Exchange connectors** | `ExchangeConnector` trait; GET-only; spot+linear; Bitunix spot-first (DEC-0037, R-0032) |
| **Portfolio Engine** | Hybrid PnL: exchange fields for derivatives; avg-cost spot; total return vs baseline (DEC-0038, R-0033) |
| **FX conversion** | Frankfurter fiat/stablecoin; exchange tickers for alts; `fx_incomplete` banner (DEC-0039, R-0034) |
| **Secret storage** | TOML `*_env` + Compose env; Settings read-only; test-connection only (DEC-0040, R-0035) |
| **Sync pipeline** | `exchanges` phase before `alerts`; independent interval + partial trigger (DEC-0041, R-0036) |
| **Persistence** | Migration 007; extend `net_worth_snapshots`; `allocation_target` plan kind (DEC-0042, R-0037) |
| **React UI** | `/wealth` Overview + Crypto tab; Settings exchange section; Sync Status per-exchange rows |
| **Grafana** | Dashboard 4 completion: crypto stat, allocation pie, performance series (R-0026) |
| **AI boundary** | Extend `get_portfolio` only — no new tools (R-0031) |

### Decisions created

- **DEC-0037** — Exchange connector trait and read-only scope
- **DEC-0038** — Hybrid portfolio PnL methodology
- **DEC-0039** — FX conversion two-layer model
- **DEC-0040** — Exchange API secret storage pattern
- **DEC-0041** — Sync pipeline exchanges phase
- **DEC-0042** — Migration 007 and snapshot extension

### Spec-pack status

| Artifact | Status |
|----------|--------|
| `US-0007-design-concept.md` | Complete (Summary, Goals, Non-goals, Key decisions) |
| `US-0007-crs.md` | Complete (Purpose, Scope, Acceptance criteria ref) |
| `US-0007-technical-specification.md` | Complete (Overview, Components, Interfaces, Non-functional) |

### Triad check (architecture phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#us-0007` | Scope matches architecture in/out | pass |
| `docs/product/acceptance.md` US-0007 | 6 criteria mapped in architecture | pass |
| `docs/engineering/architecture.md#us-0007` | Components cover all AC | pass |
| `docs/engineering/spec-pack/US-0007-*` | 3/3 artifacts present | pass |
| `docs/engineering/decisions.md` | DEC-0037–DEC-0042 indexed | pass |
| Research R-0032–R-0037 | Cited in architecture + decisions | pass |

`triad_hot_surface`: check pass (no rollover)

### Risks carried to sprint-plan

1. Binance per-symbol trade fan-out — sync latency (R-0032, R-0036)
2. Bitunix spot-first vs futures parity — signing test burden (R-0032)
3. Avg-cost inaccuracy on partial first-sync history (R-0033)
4. FX incomplete for illiquid alts — banner UX required (R-0034)
5. Mutex duration growth with exchanges phase — monitor >45s (R-0036, DEC-0018)
6. Secret rotation requires env + restart — operator doc (R-0035)

### Recommended next steps

1. `/sprint-plan` — S0007 decomposition for US-0007 acceptance criteria
2. `/plan-verify` — confirm task coverage against 6 AC

---

## research-20260601-us0007 — US-0007 crypto exchange portfolio technical research

**From:** Tech Lead  
**To:** Dev (via `/architecture` handoff)  
**Date:** 2026-06-01  
**Story:** US-0007  
**Next phase:** `/architecture`

### Summary

Web research completed for US-0007 crypto exchange portfolio integration. Six new entries (R-0032–R-0037) extend R-0021/R-0024/R-0026 deferrals with exchange REST patterns (Binance/Bybit/Bitunix), hybrid PnL methodology, EUR FX conversion, env-only secret storage, post-sync pipeline extension, and migration 007 schema.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Exchange APIs** | [R-0032](docs/engineering/research.md#r-0032--exchange-connector-rest-patterns-binance-bybit-bitunix) | Unified `ExchangeConnector` trait; read-only keys mandatory; Binance spot+USD-M, Bybit UNIFIED, Bitunix spot-first; per-symbol trade watermarks; 429 backoff |
| **PnL methodology** | [R-0033](docs/engineering/research.md#r-0033--portfolio-pnl-methodology-realized-unrealized-total-return) | Hybrid: exchange position PnL for derivatives; avg-cost from trades for spot; total return vs first-sync baseline; no tax FIFO |
| **FX conversion** | [R-0034](docs/engineering/research.md#r-0034--fx-conversion-for-crypto--eur-reporting-currency) | Frankfurter ECB daily for USD/stablecoin→EUR; exchange ticker prices for alts; `fx_incomplete` banner for missing pairs |
| **Secret storage** | [R-0035](docs/engineering/research.md#r-0035--exchange-api-secret-storage-self-hosted) | TOML `*_env` names + Compose env vars only; Settings read-only status; test-connection endpoint; no DB vault MVP |
| **Sync pipeline** | [R-0036](docs/engineering/research.md#r-0036--post-sync-exchange-pipeline--scheduler-integration) | Extend DEC-0028: forecast → **`exchanges`** → alerts snapshot; inline mutex; independent exchange interval + manual trigger |
| **Persistence** | [R-0037](docs/engineering/research.md#r-0037--portfolio-persistence-schema--snapshot-payload-extension) | Migration 007 holdings/trades/PnL tables; extend `net_worth_snapshots`; allocation via plan adjustment kind |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Exchange API auth / read-only? | **Read-only keys mandatory**; HMAC per exchange (R-0032); connector audit rejects write endpoints |
| Sync frequency vs rate limits? | **Independent `[exchanges] interval_seconds`** defaulting to Firefly interval; exchange-only trigger for phases 4–5 (R-0036) |
| FX: ticker vs external API? | **Hybrid** — Frankfurter for fiat/stablecoin; exchange prices for crypto; CoinGecko fallback deferred (R-0034) |
| PnL: FIFO vs exchange-reported? | **Hybrid wealth analytics** — exchange fields for derivatives; avg-cost for spot; no tax FIFO (R-0033) |
| Spot vs futures MVP scope? | **Spot + linear/USDT-M** on Binance/Bybit; Bitunix spot-first with futures stub (R-0032) |
| Secret storage pattern? | **TOML metadata + env secrets** mirroring `AiConfig`; Settings display-only (R-0035) |
| Pipeline order? | **`exchanges` before `alerts`** snapshot inline in mutex — extend DEC-0028 (R-0036) |
| Allocation template storage? | **`allocation_target` plan adjustment kind** on existing plan tables (R-0037) |
| Snapshot schema? | **Extend `net_worth_snapshots`** with crypto columns/payload — no separate hypertable (R-0037) |
| Settings runtime credential edit? | **No** — TOML/env + restart; test-connection only (R-0035) |
| `get_portfolio` payload? | Include crypto totals + top-5 holdings within 8 KB cap (R-0037 extends R-0031) |

### Risks surfaced (carry to architecture)

1. **Binance symbol fan-out** — `myTrades` per symbol dominates sync latency (R-0032, R-0036)
2. **Bitunix signing complexity** — double SHA256 + dual domains; spot-first reduces scope (R-0032)
3. **Avg-cost inaccuracy on first sync** — 90-day backfill may miss older lots (R-0033)
4. **FX incomplete for illiquid alts** — exclude from subtotal + banner (R-0034)
5. **Mutex duration growth** — monitor combined pipeline; consider longer exchange interval (R-0036, DEC-0018)
6. **Secret rotation** — requires env update + container restart; document operator flow (R-0035)
7. **PnL reconciliation drift** — prefer exchange cumulative realized when local sum diverges (R-0033)

### Recommended next steps

1. `/architecture` — `ExchangeConnector` trait, Portfolio Engine contract, migration 007 schema (R-0037), post-sync pipeline extension (R-0036), wealth API + FX service, DEC-xxxx for PnL method and FX provider defaults
2. `/sprint-plan` — S0007 decomposition for US-0007 acceptance criteria
3. Spec-pack expansion for US-0007 (SPEC_PACK_MODE=1)

---

