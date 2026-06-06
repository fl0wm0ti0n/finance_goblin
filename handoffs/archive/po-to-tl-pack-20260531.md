# PO to TL archive pack (2026-05-31)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 9
- First archived heading: `## intake-20260531-flow-finance — Flow Finance AI initial backlog`
- Last archived heading: `## architecture-20260531-us0003 — US-0003 subscription detection architecture`
- Verification tuple (mandatory):
  - archived_body_lines=139
  - retained_body_lines=471

---

## intake-20260531-flow-finance — Flow Finance AI initial backlog

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-05-31  
**Stories:** US-0001 … US-0009  
**Next phase:** `/research` (US-0001)

### Summary

Initial intake from `Projectplan.md` for **Flow Finance AI** — a self-hosted Finanzguru-style layer on Firefly III. Firefly remains read-only transaction source; Flow Finance AI adds forecasting, subscriptions, planning, alerts, wealth/crypto, AI assistant, and Grafana analytics.

**Decomposition decision:** 9 vertical-slice user stories aligned with the 7-phase Projectplan roadmap, splitting MVP into platform (US-0001) + forecasting/Grafana (US-0002) for deliverable increments.

| Story | Title | Priority | Roadmap phase |
|-------|-------|----------|---------------|
| US-0001 | Platform & Firefly integration | P0 | Phase 1 |
| US-0002 | Forecasting & Grafana MVP | P0 | Phase 1 |
| US-0003 | Subscription detection & alerts | P1 | Phase 2 |
| US-0004 | Planning & plan-vs-actual | P1 | Phase 3 |
| US-0005 | Wealth & alert engine | P1 | cross-cutting |
| US-0006 | AI assistant + tool layer | P2 | Phase 4 |
| US-0007 | Crypto portfolio | P2 | Phase 5 |
| US-0008 | Local AI providers | P3 | Phase 6 |
| US-0009 | Advanced ML forecasting | P3 | Phase 7 |

### Split rationale

- **Why split:** Full concept spans 11 plan areas, 7 roadmap phases, multiple engines (connector, forecast, subscription, plan, alert, portfolio, AI), and two UIs (React + Grafana). Single story would block incremental delivery and obscure acceptance boundaries.
- **Split axes:** Roadmap phase vertical slices + separation of platform ingest from analytics features.
- **Why not fewer:** Operator requested full-concept user stories (`lege usersorties an`); merging phases would hide independent user value (e.g. subscriptions usable before AI).
- **Dependency chain:** US-0001 → US-0002 → {US-0003, US-0004, US-0005} → US-0006 → US-0007 → {US-0008, US-0009}.

### Key assumptions (confirmed via Projectplan delegation)

- Self-hosted Docker Compose deployment; external PostgreSQL never embedded
- Rust/Axum backend, React/shadcn frontend, TimescaleDB, Grafana, OpenAI first for AI
- Dev environment has Docker + running Firefly III with testable API (operator note in Projectplan)
- OIDC for UI auth; Personal Access Token or OAuth for Firefly API (per R-0001)

### Risks & unknowns for architecture

1. **Firefly API pagination/volume** — large transaction histories may need incremental sync cursors (R-0001)
2. **TimescaleDB schema design** — forecast vs raw transaction storage boundaries
3. **Subscription detection accuracy** — false positives require strong confirm/reject UX before alerts propagate
4. **AI tool latency** — multi-tool chains for complex questions; token/cost controls needed
5. **Crypto exchange API stability** — rate limits and auth rotation for Binance/Bybit/Bitunix
6. **Grafana provisioning** — dashboard-as-code vs manual; align with Compose profiles

### Intake evidence

- Pack: `first-intake-pack` — all 8 required topics covered via `Projectplan.md` delegation
- Bundle: `handoffs/intake_evidence/intake-20260531-flow-finance.json`
- Plan coverage: 11 plan areas → 9 stories (`coverage_complete=true`)

### Spec-pack (SPEC_PACK_MODE=1)

CRS artifacts created at `docs/engineering/spec-pack/US-xxxx-crs.md` for each story. Design Concept and Technical Specification to be expanded in `/architecture`.

### User guides (USER_GUIDE_MODE=1)

End-user guides at `docs/user-guides/US-xxxx.md` — create during execute when feature is implemented.

### Recommended next steps

1. ~~`/discovery`~~ — completed 2026-05-31 for US-0001 (see discovery section below)
2. ~~`/research` for US-0001~~ — completed (see research section below)
3. ~~`/architecture` for US-0001~~ — completed (see architecture section below)
4. ~~`/sprint-plan` / release US-0001~~ — completed (S0001 released)
5. ~~`/discovery` for US-0002~~ — completed 2026-05-31 (see discovery section below)
6. ~~`/research` for US-0002~~ — completed 2026-05-31 (see research section below)
7. ~~`/architecture` for US-0002~~ — completed 2026-05-31 (see architecture section below)
8. ~~`/sprint-plan`~~ — S0002 for US-0002 (released `0.2.0-us0002`)
9. ~~`/discovery` for US-0003~~ — completed 2026-05-31 (see discovery section below)
10. ~~`/research` for US-0003~~ — completed 2026-05-31 (see research section below)
11. ~~`/architecture` for US-0003~~ — completed 2026-05-31 (see architecture section below)
12. `/sprint-plan` for US-0003

---

## architecture-20260531-us0003 — US-0003 subscription detection architecture

**From:** Tech Lead  
**To:** Sprint planning (`/sprint-plan`)  
**Date:** 2026-05-31  
**Story:** US-0003  
**Next phase:** `/sprint-plan`

### Summary

Architecture defined for subscription intelligence: **shared recurrence core** (DEC-0013), **Subscription Engine** with confidence tiers and Dauerauftrag classification, **migration 003** lifecycle schema (DEC-0015), **sync pipeline extension** with subscriptions phase before forecast (DEC-0018), **forecast override hook** for confirmed/rejected patterns, **subscription REST API**, React **`/subscriptions`**, and **Grafana Dashboard 2** (`uid: subscriptions`). Six decisions recorded (DEC-0013–DEC-0018). Spec-pack expanded: design-concept, CRS, technical-specification.

### Architecture highlights

| Area | Decision |
|------|----------|
| **Recurrence core** | Extract `backend/src/recurrence/` from forecast; shared detect + confidence (DEC-0013, R-0009) |
| **Confidence** | 95/80/60% tiers; emit only ≥60%; min 3 txs (DEC-0014) |
| **Persistence** | Single `subscription_patterns` + satellites; rejection fingerprints (DEC-0015, R-0012) |
| **Dauerauftrag** | Rule-based classify + optional config patterns; user override on confirm (DEC-0016, R-0010) |
| **Price change** | Dual ≥€1.00 AND ≥5% on confirmed only (DEC-0017, R-0011) |
| **Sync pipeline** | sync → subscriptions → forecast in mutex (DEC-0018, R-0013) |
| **Forecast hook** | Confirmed override heuristics; rejected excluded (AC-8) |
| **API** | `/api/v1/subscriptions/*` + alerts endpoints |
| **React** | `/subscriptions` tabs All/Pending/Standing orders; confirm/reject; ECharts price history |
| **Grafana** | Dashboard 2 uid `subscriptions`; stat + table + time series (R-0014) |

### Decisions created

- **DEC-0013** — Shared recurrence core extraction
- **DEC-0014** — Confidence tiers 95/80/60
- **DEC-0015** — Single lifecycle schema + rejection semantics
- **DEC-0016** — Dauerauftrag rule-based classification
- **DEC-0017** — Price-change dual threshold defaults
- **DEC-0018** — Subscriptions phase before forecast in sync mutex

### Spec-pack status

| Artifact | Status |
|----------|--------|
| `US-0003-design-concept.md` | Complete |
| `US-0003-crs.md` | Complete |
| `US-0003-technical-specification.md` | Complete |

### Risks carried to sprint-plan

1. Recurrence refactor must preserve US-0002 forecast behavior
2. 60% confidence tier false positives — confirm/reject UX critical
3. Sync mutex duration grows with detection pass — monitor combined latency
4. Grafana monthly-spend SQL interval normalization
5. Dauerauftrag edge cases — user kind override required

### Recommended next steps

1. `/sprint-plan` — S0003 task decomposition against 8 acceptance criteria
2. `/plan-verify` — confirm AC coverage before execute

---

