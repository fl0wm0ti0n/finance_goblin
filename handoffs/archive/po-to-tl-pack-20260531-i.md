# PO to TL archive pack (2026-05-31)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 8
- First archived heading: `## discovery-20260531-us0004 — US-0004 financial planning UX discovery`
- Last archived heading: `## research-20260531-us0003 — US-0003 subscription detection technical research`
- Verification tuple (mandatory):
  - archived_body_lines=113
  - retained_body_lines=453

---

## discovery-20260531-us0004 — US-0004 financial planning UX discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-05-31  
**Story:** US-0004  
**Next phase:** `/research`

### Summary

Discovery captured design/UX references for financial planning (Phase 3). US-0004 delivers the **Plan Engine** with scenario templates and custom deltas, **plan versioning** (v1/v2/v3 compare), **daily plan-vs-Ist** comparison, React **`/planning`** page, and **Grafana Dashboard 3** (Budgets: plan/ist/deviation). Builds on US-0001 synced Firefly actuals, US-0002 forecast baseline, and US-0003 confirmed subscriptions for savings-mode template picks.

### UX references captured (see `docs/product/vision.md`)

| Reference | US-0004 application |
|-----------|---------------------|
| **Finanzguru** | Scenario templates (leasing, savings mode, house purchase); v1/v2/v3 side-by-side compare; daily plan/ist/deviation; life-decision framing without UI clone |
| **Firefly III** | Ist from synced transactions (read-only); Firefly budget/category vocabulary; no Firefly mutation |
| **shadcn/ui** | Enable Planning nav at `/planning`; Tabs Scenarios / Compare / Plan vs Actual; active plan selector; template cards + adjustment table |
| **Apache ECharts** | Daily planned vs actual dual line; version compare grouped bar; deviation trend with zero line |
| **Grafana Dashboard 3** | Plan, Ist, Abweichung panels for active plan; uid `budgets` |
| **US-0002 forecast** | Current (Ist) scenario aligns with latest forecast computation; deltas adjust projected layer only |
| **US-0003 subscriptions** | Savings-mode template suggests confirmed subscriptions to remove |

### Scope refinements (backlog updated)

- Plan Engine: explicit user deltas on forecast baseline; one active plan drives plan-vs-Ist + Dashboard 3
- Templates: Current (Ist), Leasing (+€/month), Savings mode (remove subs / cut spend), House purchase (raise savings rate)
- Custom adjustments: amount, frequency, target (subscription / category / custom label)
- Versioning: v1/v2/v3 per named plan with Compare tab
- React: `/planning` route; daily plan-vs-Ist table/chart for active plan
- Grafana Dashboard 3 provisioned alongside existing analytics dashboards
- Out of scope unchanged: AI `simulate_plan` (US-0006), crypto allocation scenarios (US-0007), plan viability Alert Engine (US-0005)

### Discovery decomposition evidence

- Feature/workflow count: Plan Engine + 4 templates + custom deltas + version compare + daily plan-vs-Ist + React page + Grafana (moderate-high — single story retained)
- Cross-cutting impact: backend engine, DB migration, React UI, forecast/subscription integration, Grafana
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md`)
- Risk surface: delta modeling vs forecast baseline, Ist lag vs sync, v1/v2/v3 compare UX breadth, Dashboard 3 SQL joining plan + actual tables, read-only actuals constraint

### Open questions (carry to research/architecture)

- Plan projection: fork US-0002 forecast computation with delta overlay vs independent Plan Engine projection layer?
- Ist aggregation grain: household free cashflow vs category budget spend vs account balance path?
- Savings mode: auto-populate from confirmed subscriptions vs manual line items only?
- Version semantics: immutable snapshots on "new version" vs editable in-place with history?
- Compare UX: table-first metrics vs chart-first for v1/v2/v3?
- Daily plan-vs-Ist metric: which primary number — free cashflow, total outflow, or category-specific?
- Active plan overlay on `/forecast` in US-0004 or planning-only surface?
- Dashboard 3 MVP: household aggregate vs per-category breakdown panels?
- Recompute trigger: on plan save only vs extend sync mutex after forecast recompute?

### Recommended next steps

1. `/research` — Plan Engine delta model, persistence schema, Ist aggregation rules, Grafana Dashboard 3 as-code (extends R-0008)
2. `/architecture` — Plan Engine contract, REST API, migration 004 schema, active plan selection, forecast baseline hook, DEC-xxxx for template defaults and version limits
3. `/sprint-plan` — S0004 decomposition for US-0004 acceptance criteria

---

## research-20260531-us0003 — US-0003 subscription detection technical research

**From:** Tech Lead  
**To:** Dev (via `/architecture` handoff)  
**Date:** 2026-05-31  
**Story:** US-0003  
**Next phase:** `/architecture`

### Summary

Web research completed for US-0003 subscription intelligence. Six new entries (R-0009–R-0014) extend R-0006 forecast heuristics, R-0008 Grafana provisioning, and DEC-0010 sync mutex with detection algorithms, Dauerauftrag classification, price-change thresholds, persistence schema, post-sync pipeline order, and Dashboard 2 as-code.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Detection patterns** | [R-0009](docs/engineering/research.md#r-0009--subscription-detection-engine-patterns--confidence-scoring) | Extract shared `recurrence` module from `forecast/recurring.rs`; confidence tiers 95/80/60% from occurrence count + cadence + amount tolerance; min 3 txs; fingerprint dedup |
| **Dauerauftrag heuristics** | [R-0010](docs/engineering/research.md#r-0010--dauerauftrag-standing-order-vs-subscription-classification) | Rule-based primary (amount CV <2%, category boost, large fixed outflows) + optional config payee patterns; `kind` enum on single entity |
| **Price change detection** | [R-0011](docs/engineering/research.md#r-0011--subscription-price-change-detection--alert-thresholds) | Dual threshold: ≥€1.00 AND ≥5% delta on confirmed subs only; append-only price events; in-app banner/toast alerts (defer header bell) |
| **Persistence model** | [R-0012](docs/engineering/research.md#r-0012--subscription-persistence-schema-candidates-confirmed-rejections-events) | Single lifecycle `subscription_patterns` table + satellites (transactions link, price_events, rejections, alerts); migration 003 |
| **Sync integration** | [R-0013](docs/engineering/research.md#r-0013--post-sync-subscription-detection-pipeline--forecast-integration) | Extend DEC-0010 mutex: sync → detection (`subscriptions` phase) → forecast recompute; confirmed override + rejected exclusion hook |
| **Grafana Dashboard 2** | [R-0014](docs/engineering/research.md#r-0014--grafana-dashboard-2-subscriptions-provisioning) | uid `subscriptions`; stat panels for spend/pending; table for price changes; time series for detection volume; global scope MVP |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Reuse vs fork `detect_patterns`? | **Extract shared recurrence core** — forecast wrapper + subscription engine with confidence/persistence (R-0009) |
| Persistence: separate tables vs polymorphic? | **Single lifecycle table** `subscription_patterns` with status enum + event/alert satellites (R-0012) |
| Dauerauftrag: whitelist vs rules? | **Rule-based primary** (amount stability, category boost) + optional config payee patterns (R-0010) |
| Alert delivery: page-only vs header bell? | **MVP: `/subscriptions` banner + toast** with `subscription_alerts` table; defer global bell to US-0005 (R-0011) |
| Price-change threshold? | **Dual: ≥€1.00 AND ≥5%** on confirmed subs; configurable in TOML (R-0011) |
| Grafana: event table vs time series? | **Table for price changes** (before/after columns); time series for daily detection counts (R-0014) |
| Detection: inline mutex vs async queue? | **Inline before forecast** in same mutex per DEC-0010; defer Redis queue if combined latency >~30s (R-0013) |

### Risks surfaced (carry to architecture)

1. **Descriptor normalization drift** — bank payee suffixes may split groups; regex strip trailing codes (R-0009)
2. **Dauerauftrag edge cases** — insurance premium adjustments, rent Nebenkosten may misclassify; user kind override on confirm (R-0010)
3. **Promotional pricing false alarms** — promo end triggers price-increase alert; document as known limitation (R-0011)
4. **Sync mutex duration** — detection pass adds O(transactions) work; monitor combined sync+detection+forecast latency (R-0013, DEC-0010)
5. **60% confidence tier false positives** — strong confirm/reject UX required before alerts propagate (R-0009)
6. **Grafana monthly-spend normalization** — SQL must convert weekly/annual intervals to monthly equivalent (R-0014)

### Recommended next steps

1. `/architecture` — Subscription Engine trait/contract, REST API, migration 003 schema (R-0012), sync pipeline order (R-0013), forecast override hook, DEC-xxxx for confidence thresholds, rejection semantics, price-change defaults
2. `/sprint-plan` — S0003 task decomposition against 8 acceptance criteria
3. Spec-pack expansion for US-0003 (SPEC_PACK_MODE=1)

---

