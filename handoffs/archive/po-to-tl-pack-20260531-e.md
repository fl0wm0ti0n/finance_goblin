# PO to TL archive pack (2026-05-31)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 9
- First archived heading: `## architecture-20260531-us0005 — US-0005 wealth & alerts architecture`
- Last archived heading: `## research-20260531-us0005 — US-0005 wealth & alerts technical research`
- Verification tuple (mandatory):
  - archived_body_lines=113
  - retained_body_lines=492

---

## architecture-20260531-us0005 — US-0005 wealth & alerts architecture

**From:** Tech Lead  
**To:** Sprint planning (`/sprint-plan`)  
**Date:** 2026-05-31  
**Story:** US-0005  
**Next phase:** `/sprint-plan`

### Summary

Architecture defined for US-0005 wealth analysis and Alert Engine. Six decisions (DEC-0025–DEC-0030) extend R-0021–R-0026 research. Spec-pack expanded: design-concept, CRS, technical-specification (3/3).

### Architecture highlights

| Area | Decision |
|------|----------|
| **Net worth** | Sum Firefly asset accounts; EUR default; mixed-currency warning; daily `net_worth_snapshots` post-sync; crypto placeholder excluded (DEC-0025, R-0021) |
| **Alert Engine** | Scarcity (household), budget drift (plan category targets), plan viability (month-end overlay); fingerprint dedup (DEC-0026, DEC-0027, R-0022, R-0023) |
| **Migration 005** | `alert_config` singleton, `alerts` table, `net_worth_snapshots` (DEC-0027) |
| **Sync pipeline** | Extend mutex: … → forecast (+ plan hook) → `"alerts"` phase (snapshot + evaluate); inline; non-blocking failure (DEC-0028, R-0024) |
| **Threshold centralization** | TOML `[alerts]` → `alert_config` DB mirror; Dashboard 1 `$scarcity_threshold` variable replaces static €200 (DEC-0029, R-0025; supersedes DEC-0012 hardcode) |
| **React UI** | `/wealth` route; `/alerts` inbox; header bell + Popover preview (DEC-0030) |
| **Grafana Dashboard 4** | uid `portfolio`; total wealth + account breakdown + wealth-over-time; crypto deferred US-0007 (DEC-0030, R-0026) |
| **US-0003 boundary** | `subscription_alerts` unchanged; optional header cross-link only |

### Decisions created

- **DEC-0025** — Net worth aggregation: asset sum, snapshots, mixed-currency MVP
- **DEC-0026** — Alert evaluation rules: household scarcity, category drift, plan viability
- **DEC-0027** — Alert persistence: migration 005, fingerprint dedup, lifecycle
- **DEC-0028** — Sync alerts phase: inline after forecast+plan hook
- **DEC-0029** — Threshold centralization: TOML + alert_config + Grafana variable
- **DEC-0030** — Unified inbox UI: /wealth, /alerts, bell; Dashboard 4 partial

### Spec-pack status

| Artifact | Status |
|----------|--------|
| `US-0005-design-concept.md` | Complete (Summary, Goals, Non-goals, Key decisions) |
| `US-0005-crs.md` | Complete (Purpose, Scope, Acceptance criteria ref, Architecture mapping) |
| `US-0005-technical-specification.md` | Complete (Overview, Components, Interfaces, Non-functional) |

### Risks carried to sprint-plan

1. Mixed-currency headline total without FX — UI disclaimer required
2. Budget drift proration vs one-time plan deltas
3. Plan viability on stale forecast/plan baseline
4. Sync mutex duration with alerts phase (~100–500ms)
5. US-0003 alert boundary — document cross-link
6. Dashboard 1 threshold migration — test Grafana variable in SQL

### Recommended next steps

1. `/sprint-plan` — S0005 task decomposition against 6 acceptance criteria
2. `/plan-verify` — confirm task coverage against acceptance

---

## research-20260531-us0005 — US-0005 wealth & alerts technical research

**From:** Tech Lead  
**To:** Dev (via `/architecture` handoff)  
**Date:** 2026-05-31  
**Story:** US-0005  
**Next phase:** `/architecture`

### Summary

Web research completed for US-0005 wealth analysis and Alert Engine. Six new entries (R-0021–R-0026) extend R-0008 Grafana provisioning, DEC-0010/DEC-0018 sync mutex, DEC-0023 plan hook, and R-0017 category actuals with net worth aggregation, alert evaluation rules, persistence lifecycle, post-sync pipeline order, threshold centralization, and Dashboard 4 as-code.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Net worth aggregation** | [R-0021](docs/engineering/research.md#r-0021--net-worth-aggregation-firefly-asset-accounts--snapshots) | Sum Firefly `asset` accounts with `include_net_worth=true`; EUR reporting default; mixed-currency warning banner (no FX MVP); daily `net_worth_snapshots` post-sync; crypto placeholder excluded |
| **Alert evaluation rules** | [R-0022](docs/engineering/research.md#r-0022--alert-engine-evaluation-rules-scarcity-budget-drift-plan-viability) | Scarcity: household min projected balance vs threshold; budget drift: active-plan category targets MTD vs +20%; plan viability: active plan month-end balance < 0 or 2 consecutive negative month-ends |
| **Persistence & lifecycle** | [R-0023](docs/engineering/research.md#r-0023--alert-persistence-deduplication--lifecycle-acknowledge--dismiss) | Migration 005 `alerts` table; fingerprint dedup; acknowledge clears unread; dismiss hides until condition clears/re-triggers; subscription_alerts unchanged |
| **Sync integration** | [R-0024](docs/engineering/research.md#r-0024--post-sync-alert-engine-pipeline--net-worth-snapshot-hook) | Extend mutex: sync → subscriptions → forecast (+ plan hook) → net worth snapshot + alerts (`phase: alerts`); inline; failure non-blocking |
| **Threshold centralization** | [R-0025](docs/engineering/research.md#r-0025--alert-threshold-config-centralization--dashboard-1-scarcity-wiring) | TOML `[alerts]` → `alert_config` singleton DB mirror; Dashboard 1 `$scarcity_threshold` query variable replaces static €200 (supersedes DEC-0012 hardcode) |
| **Grafana Dashboard 4** | [R-0026](docs/engineering/research.md#r-0026--grafana-dashboard-4-portfolio--wealth-partial-provisioning) | uid `portfolio`; total wealth stat + account table + wealth-over-time from snapshots; crypto/performance deferred US-0007 |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Alert evaluation: inline mutex vs async? | **Inline after forecast+plan hook** in same sync task; defer queue if combined >~30s (R-0024, DEC-0010) |
| Scarcity scope: per account vs household? | **Household aggregate min projected balance** across asset accounts; per-account visual only on Dashboard 1 (R-0022) |
| Budget drift grain? | **Active-plan category-targeted adjustments only** — MTD actual vs prorated plan target; skip untargeted categories (R-0022, R-0017) |
| Plan viability rule? | **Month-end planned balance < 0** OR **2 consecutive negative month-ends** on active plan overlay (R-0022) |
| Alert dedup/cooldown? | **Fingerprint partial unique index** — update in place while active/acknowledged; resolve when condition clears (R-0023) |
| Dismiss semantics? | **Hide until condition clears or re-triggers** — not permanent suppress (R-0023) |
| Unified inbox vs subscription alerts? | **Separate surfaces** — optional header link to subscription unread count; no migration (R-0023, R-0011) |
| Net worth snapshots? | **Daily post-sync upsert** to `net_worth_snapshots` for trend chart + Dashboard 4 (R-0021) |
| Dashboard 1 threshold migration? | **Grafana `$scarcity_threshold` query variable** from `alert_config` table in panel SQL (R-0025) |
| Multi-currency net worth? | **Sum native with mixed-currency warning banner** — FX conversion deferred (R-0021) |

### Risks surfaced (carry to architecture)

1. **Mixed-currency headline total** — misleading without UI disclaimer (R-0021)
2. **Budget drift proration** — one-time mid-month plan deltas may skew MTD target (R-0022)
3. **Plan viability on stale baseline** — failed forecast leaves prior plan computation (R-0022, R-0019)
4. **Mutex duration** — alert pass adds ~100–500ms; monitor combined pipeline (R-0024)
5. **US-0003 alert boundary** — users may expect single inbox; document cross-link (R-0023)
6. **DEC-0012 supersession** — architecture must record DEC-xxxx for threshold centralization and dismiss semantics (R-0025)

### Recommended next steps

1. `/architecture` — Alert Engine trait/contract, REST API, migration 005 schema (R-0023), sync pipeline `"alerts"` phase (R-0024), net worth service, DEC-xxxx for scarcity scope, dismiss semantics, threshold config
2. `/sprint-plan` — S0005 task decomposition against 6 acceptance criteria
3. Spec-pack expansion for US-0005 (SPEC_PACK_MODE=1)

---

