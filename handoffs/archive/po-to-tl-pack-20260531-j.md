# PO to TL archive pack (2026-05-31)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 4
- Retained units in hot file: 8
- First archived heading: `## discovery-20260531-us0003 — US-0003 subscription detection UX discovery`
- Last archived heading: `## architecture-20260531-us0001 — US-0001 platform foundation architecture`
- Verification tuple (mandatory):
  - archived_body_lines=212
  - retained_body_lines=486

---

## discovery-20260531-us0003 — US-0003 subscription detection UX discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-05-31  
**Story:** US-0003  
**Next phase:** `/research`

### Summary

Discovery captured design/UX references for subscription intelligence (Phase 2). US-0003 delivers the Subscription Detection Engine with confidence tiers, confirm/reject workflow, standing-order (Dauerauftrag) separation, price-change detection and subscription-scoped alerts, React `/subscriptions` page, forecast integration for confirmed/rejected patterns, and Grafana Dashboard 2. Builds on US-0001 synced transactions, US-0002 forecast recurring heuristics (DEC-0007), and Grafana provisioning (DEC-0012).

### UX references captured (see `docs/product/vision.md`)

| Reference | US-0003 application |
|-----------|---------------------|
| **Finanzguru** | Pending review cards (payee, interval, amount, confidence); confirm/reject actions; price-change callouts; proactive pending banner without UI clone |
| **Firefly III** | Payee/description from synced transactions; drill-down to filtered transaction list; Flow DB only for confirm/reject state |
| **shadcn/ui** | Enable Subscriptions nav at `/subscriptions`; Tabs (All / Pending / Standing orders); Card confirm/reject; Table + Sheet detail; Badge confidence tiers |
| **Apache ECharts** | Price history line in subscription detail drawer; optional top-N spend bar (defer if tight) |
| **Grafana Dashboard 2** | All subscriptions spend stat, price-change events, new/pending detection counts; uid `subscriptions` |
| **US-0002 forecast** | Confirmed subs override recurring heuristics; rejected excluded from projection (AC-8) |

### Scope refinements (backlog updated)

- Detection Engine: amount, payee, text, regularity, intervals; confidence 95/80/60%; post-sync run
- React: `/subscriptions` route; pending confirm/reject cards; confirmed list; Dauerauftrag tab
- Price changes: amount history, increase/decrease/frequency detection; detail drawer chart
- Subscription-scoped alerts: new detection + price change (in-app; not US-0005 inbox)
- Forecast hook: confirmed override + rejected exclusion for recurring layer
- Grafana Dashboard 2 provisioned alongside existing dashboards
- Out of scope unchanged: plan cancel scenarios (US-0004), full Alert Engine (US-0005), AI tool implementation detail (US-0006)

### Discovery decomposition evidence

- Feature/workflow count: detection + 3 UI workflows (pending, confirmed, standing orders) + price history + alerts + Grafana (moderate-high — single story retained)
- Cross-cutting impact: backend engine, DB schema, React UI, forecast integration, Grafana
- Acceptance breadth: unchanged (8 criteria)
- Risk surface: false positives pre-confirm, Dauerauftrag disambiguation, price-change false alarms, sync mutex latency (DEC-0010), confidence calibration

### Open questions (carry to research/architecture)

- Reuse vs fork `forecast/recurring.rs` `detect_patterns` for subscription candidates — shared module or separate engine with common traits?
- Persistence model: `subscription_candidates` + `subscriptions` + `subscription_price_events` tables vs single polymorphic entity?
- Standing-order heuristic: fixed payee whitelist config vs rule-based (amount stability + category tag)?
- Alert delivery: page-only vs header notification bell with unread count (US-0005 inbox deferred)?
- Price-change threshold: any amount delta vs minimum €/percent change to reduce noise?
- Grafana Dashboard 2: event table vs time-series for price changes; global vs per-account variable?
- Detection run: inline in sync mutex with forecast recompute vs async job queue if latency exceeds threshold?

### Recommended next steps

1. `/research` — subscription detection algorithms (extends R-0006), persistence schema, Dauerauftrag heuristics, Grafana Dashboard 2 as-code (extends R-0008)
2. `/architecture` — Subscription Engine contract, API endpoints, forecast override hook, sync-post detection trigger, DEC-xxxx for confidence thresholds and rejection semantics
3. `/sprint-plan` — S0003 decomposition for US-0003 acceptance criteria

---

## discovery-20260531-us0001 — US-0001 platform foundation UX discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-05-31  
**Story:** US-0001  
**Next phase:** `/research`

### Summary

Discovery captured design/UX references for the self-hosted platform foundation. US-0001 delivers a deployable stack, OIDC-protected UI shell, Firefly read-only connector, and operator Grafana provisioning — not feature analytics UI.

### UX references captured (see `docs/product/vision.md`)

| Reference | US-0001 application |
|-----------|---------------------|
| **Finanzguru** | Dashboard-first landing with trust signals (sync status, last updated, read-only badge); feature widgets deferred |
| **Firefly III** | Firefly-native entity vocabulary; Sync Status grouped by account type; semantic alignment, not UI clone |
| **shadcn/ui** | SidebarProvider + collapsible icon sidebar; config-driven nav; Home, Sync Status, Settings pages; disabled nav placeholders for future features |
| **Grafana** | Minimal profile provisioning + datasource; optional Platform Health dashboard (sync/API metrics only); Dashboards 1–5 deferred to US-0002+ |
| **OIDC** | Redirect login, session in sidebar footer, protected routes; provider config via Compose/env |

### Scope refinements (backlog updated)

- UI shell pages: Home/Dashboard placeholder, Sync Status, Settings
- Nav placeholders (disabled + "Coming soon"): Forecast, Subscriptions, Planning, Wealth, AI
- Grafana: service + datasource in minimal profile; Platform Health ops dashboard optional; no analytics dashboard content
- Compose profiles: minimal (3 services), standard (+redis), full (+ollama)
- Firefly connector auth: PAT preferred (R-0001), OAuth2 alternative
- Persistent read-only indicator in app header

### Discovery decomposition evidence

- Feature/workflow count: 3 UI pages + auth shell + sync status + Grafana provisioning (low breadth, foundation-only)
- Cross-cutting impact: platform layer only; no downstream story UI
- Acceptance breadth: unchanged (8 criteria in `docs/product/acceptance.md`)
- Risk surface: OIDC provider choice, sync volume/pagination (delegated to `/research`)

### Open questions (carry to research/architecture)

- OIDC provider choice for dev (Keycloak, Authentik, or Firefly-adjacent)?
- Redis in standard profile from day one, or defer until caching need proven?
- Platform Health Grafana dashboard: ship in US-0001 or defer to US-0002 with first analytics dashboard?

### Recommended next steps

1. `/research` — Firefly API pagination/cursor strategy (extends R-0001), OIDC provider evaluation, TimescaleDB schema boundaries
2. `/architecture` — connector contract, DB schema, Compose profile wiring, OIDC integration pattern

---

## research-20260531-us0001 — US-0001 platform foundation technical research

**From:** Tech Lead  
**To:** Dev (via `/architecture` handoff)  
**Date:** 2026-05-31  
**Story:** US-0001  
**Next phase:** `/architecture`

### Summary

Web research completed for US-0001 platform foundation. Four new entries (R-0002–R-0005) extend R-0001 Firefly baseline with sync strategy, OIDC provider selection, Rust/DB stack patterns, and Compose external-PostgreSQL wiring.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|-------------------------------|
| **Firefly sync** | [R-0002](docs/engineering/research.md#r-0002--firefly-iii-pagination--incremental-sync-strategy) | Page-based pull with `limit=500`; incremental via date-window + 7-day overlap; upsert by Firefly `id`; no native cursor API |
| **OIDC** | [R-0003](docs/engineering/research.md#r-0003--self-hosted-oidc-provider-options--reactaxum-integration) | Authentik preferred for self-hosted dev (optional `oidc` Compose profile); React `react-oidc-context`; Axum JWT validation via JWKS |
| **Backend/DB** | [R-0004](docs/engineering/research.md#r-0004--rust-axum--sqlx--timescaledb-foundation-patterns) | SQLx migrations at startup; relational mirror tables in US-0001; enable TimescaleDB extension early; hypertables deferred to US-0002 |
| **Compose** | [R-0005](docs/engineering/research.md#r-0005--docker-compose-multi-service-with-external-postgresql) | No embedded PostgreSQL; startup retry for external DB; `extra_hosts` for Linux dev; healthchecks on in-compose services only |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| OIDC provider for dev? | **Authentik** as optional Compose profile; external IdP URL also supported via env (R-0003) |
| Redis in standard profile from day one? | **Defer** — no caching need proven in US-0001; standard profile adds redis container but app need not depend on it until US-0002+ (R-0004/R-0005) |
| Platform Health Grafana dashboard? | **Ship optional in US-0001** — minimal provisioning pattern established in R-0005; dashboard JSON can be thin (sync/API metrics only) |

### Risks surfaced (carry to architecture)

1. **External TimescaleDB prerequisite** — operator must install TimescaleDB extension on external PostgreSQL; not verifiable via Compose alone (R-0004, R-0005)
2. **Firefly `updated_at` unreliable** — do not depend on Search API `updated_at_after` as sole incremental signal; use date-window overlap strategy (R-0002)
3. **Large transaction history** — initial sync may take minutes; need progress reporting in Sync Status UI and configurable scheduler interval (R-0002)
4. **IdP operational overhead** — Authentik adds 2–4 containers; keep out of minimal profile, document external IdP as alternative (R-0003)
5. **Linux dev DB reachability** — requires `host.docker.internal:host-gateway` or explicit host IP in database config (R-0005)

### Recommended next steps

1. `/architecture` — Firefly connector contract (sync state machine per R-0002), DB schema (relational mirrors per R-0004), Compose profiles (minimal/standard/oidc per R-0005), OIDC integration (React + Axum per R-0003)
2. Record architecture decisions as DEC-xxxx for OIDC provider default, sync watermark strategy, and external DB retry policy
3. `/sprint-plan` — US-0001 tasks after architecture sign-off

---

## architecture-20260531-us0001 — US-0001 platform foundation architecture

**From:** Tech Lead  
**To:** Sprint planning (`/sprint-plan`)  
**Date:** 2026-05-31  
**Story:** US-0001  
**Next phase:** `/sprint-plan`

### Summary

Architecture defined for deployable platform foundation with explicit **Firefly read-only guarantee** (GET-only HTTP client, audit log, integration test per DEC-0004). Six decisions recorded (DEC-0001–DEC-0006). Spec-pack expanded: design-concept, CRS, technical-specification.

### Architecture highlights

| Area | Decision |
|------|----------|
| **Firefly Connector** | GET-only, PAT auth, `limit=500` pagination, 7-day overlap watermark (DEC-0002, R-0002) |
| **Mirror schema** | Relational tables for 6 entity types + sync metadata; TimescaleDB extension enabled; hypertables deferred to US-0002 (DEC-0005, R-0004) |
| **Sync scheduler** | Tokio cron; manual `POST /api/v1/sync/trigger`; mutex prevents overlap |
| **Compose profiles** | minimal (3 services), standard (+redis), full (+ollama), oidc (+Authentik) — no embedded PostgreSQL (R-0005) |
| **External DB** | Startup retry ~60s exponential backoff (DEC-0003); `host.docker.internal:host-gateway` on Linux |
| **UI shell** | shadcn sidebar: Home, Sync Status, Settings; disabled nav placeholders; OIDC via react-oidc-context |
| **OIDC** | Authentik optional `oidc` profile; external IdP via env (DEC-0001, R-0003); SPA JWT + JWKS validation (DEC-0006) |
| **Grafana** | Datasource provisioning + optional Platform Health dashboard; analytics dashboards deferred |
| **API boundary** | Browser → Axum (JWT); Axum → Firefly (GET-only, server-side); no direct browser→Firefly |

### Decisions created

- **DEC-0001** — OIDC IdP default: Authentik optional profile
- **DEC-0002** — Sync watermark: date-window + 7-day overlap
- **DEC-0003** — External DB startup retry policy
- **DEC-0004** — Firefly read-only enforcement (GET-only + audit)
- **DEC-0005** — Relational mirrors in US-0001; hypertables in US-0002
- **DEC-0006** — SPA bearer JWT auth; no BFF

### Spec-pack status

| Artifact | Status |
|----------|--------|
| `US-0001-design-concept.md` | Complete (Summary, Goals, Non-goals, Key decisions) |
| `US-0001-crs.md` | Complete (Purpose, Scope, Acceptance criteria ref) |
| `US-0001-technical-specification.md` | Complete (Overview, Components, Interfaces, Non-functional) |

### Risks carried to sprint-plan

1. Operator TimescaleDB extension prerequisite on external PostgreSQL
2. Large initial sync duration — progress UI required
3. Authentik optional but documented; callback URL sensitivity
4. Linux external DB reachability via host-gateway

### Recommended next steps

1. ~~`/sprint-plan`~~ — completed 2026-05-31 (see sprint-plan section below)
2. `/plan-verify` — confirm task coverage against acceptance
3. US-0002 deferred to sprint S0002 (S0001 at max task threshold)

---

