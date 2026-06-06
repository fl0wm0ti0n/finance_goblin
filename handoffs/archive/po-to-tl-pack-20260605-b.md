# PO to TL archive pack (2026-06-05)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 4
- Retained units in hot file: 7
- First archived heading: `## discovery-20260605-bug0012 — BUG-0012 forecast monthly bucket discovery`
- Last archived heading: `## intake-20260605-bug0008-0011 — Omniflow production regression batch 2 (4 operator issues)`
- Verification tuple (mandatory):
  - archived_body_lines=300
  - retained_body_lines=485

---

## discovery-20260605-bug0012 — BUG-0012 forecast monthly bucket discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Bug:** BUG-0012  
**Orchestrator:** `auto-20260605-bug0012-001`  
**Next phase:** `/research`

### Summary

Discovery **confirms both sub-defects AG/AH** on post-BUG-0010 omniflow (`financegnome.omniflow.cc`, US-0010 external, 922+ tx). Root cause is **projection bucket assignment**, not missing mirror `category_id` (BUG-0006 DONE) or broken monthly API/UI read path. `category_names` map is loaded at recompute but **never applied**; negative flows always bucket as Variable.

| Sub-defect | Verdict | Root cause (confirmed) |
|------------|---------|------------------------|
| **AG** | CONFIRMED | Income assigned only when **net daily delta ≥ 0**. Salary/recurring inflows not decomposed from rolling outflows — net-negative days dominate; Income stays **0** despite categorized income txs in mirror. |
| **AH** | CONFIRMED | `categorize_delta` → `map_category(None, config)` for all `delta < 0` → **Fixed never populated**. Rent/utilities recurring patterns exist but bucket as Variable. |

### Aggregation path (canonical)

```
run_projection
  → category_name_map() + fetch_transactions_for_account (category_id present)
  → project_account daily loop
      delta = rolling.daily_rate + Σ recurring_due(pattern)
      bucket = categorize_delta(delta, category_names, config)  // category_names ignored
      monthly_map[month][bucket] += |delta|
  → bulk_insert_monthly → forecast_cashflow_monthly
GET /api/v1/forecast/monthly → ForecastPage Monthly tab (series[0] stat cards + MonthlyChart)
```

### Code gaps

| Location | Gap |
|----------|-----|
| `project.rs` `categorize_delta` | `let _ = category_names`; `map_category(None, …)` for negatives |
| `types.rs` `RecurringPattern` | No `category_id` field |
| `recurring.rs` | Pattern detection drops transaction categories |
| `categories.rs` `map_category` | Works with names; never called with mirror categories in projection |
| `default.toml` `[forecast.category_buckets]` | Config present; unused in live assignment |

### UX contract (operator view when fixed)

`/forecast` **Monthly** tab — four stat cards (**Income**, **Fixed**, **Variable**, **Free cashflow**) from first month in API series; stacked chart for horizon. Operator with salary + rent categories should see non-zero Income and Fixed; Variable holds discretionary residual; no UI change required if API buckets correct.

### Fix tasks (for research/architecture)

| ID | Scope | Task |
|----|-------|------|
| AG1 | AG | Decompose daily cashflow into recurring vs rolling components before bucket assignment |
| AG2 | AG | Map income-category inflows via `category_id` → `category_names` → `map_category` |
| AH1 | AH | Map fixed-category recurring outflows to Fixed bucket |
| AH2 | AH | Carry `category_id` on `RecurringPattern` or resolve at due-date from matching txs |
| AH3 | AG/AH | Remove `map_category(None)` path; wire `category_names` |
| REG1 | AG/AH | Tests: salary+rent scenario; Variable regression; free_cashflow math |

### Boundaries

- **BUG-0012 closes:** DEC-0007 config-driven category→bucket projection (AG/AH).
- **US-0015 closes:** AI-assisted mapping when categories ambiguous.
- **US-0013:** ML overlay separate; monthly baseline buckets authoritative for acceptance.
- **BUG-0007:** AI chat surface — coordinate only.

### Intake evidence

- `intake_run_id`: `intake-20260605-forecast-monthly-buckets`
- Bundle: `handoffs/intake_evidence/intake-20260605-forecast-monthly-buckets.json`

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#BUG-0012` | Discovery notes + fix tasks + test hints | pass |
| `docs/product/acceptance.md` | AG/AH unchanged | pass |
| `docs/product/vision.md` | BUG-0012 monthly bucket UX note | pass |
| Code trace | `project.rs`, `categories.rs`, `service.rs`, `ForecastPage` | pass |

`triad_hot_surface`: check pending post-write (DEC-0054)

### Open questions (carry to `/research`)

| Topic | Question |
|-------|----------|
| **Component attribution** | Assign buckets per recurring event vs net daily delta — DEC-0007 hybrid model intent? |
| **Rolling residual** | Always Variable, or attempt category on non-recurring window txs? |
| **Category name matching** | Firefly operator category names vs TOML `category_buckets` keys — normalization/fallback? |
| **Subscription override** | Confirmed recurring from US-0003 — inherit category from payee txs? |
| **Same-day mixed flows** | Salary inflow + rent outflow same day — split buckets without double-counting balance? |

### Recommended next steps

1. `/research` — component-level bucket attribution spike; category name alignment with Firefly mirror
2. `/architecture` — freeze projection decomposition contract and `RecurringPattern` schema delta
3. `/sprint-plan` — quick sprint after architecture (estimate 4–6 tasks)

---

## intake-20260605-bug0012-forecast-buckets — Forecast monthly Income/Fixed zero + US-0013 ML priority

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Bugs:** BUG-0012 (new)  
**Stories updated:** US-0013 (P0 priority + operator ML external confirm), US-0015 (new epic)  
**Next phase:** `/discovery` on **BUG-0012**

### Summary

Operator report (post-BUG-0010 Q0013 release) on `financegnome.omniflow.cc` (US-0010 external, 922+ tx) decomposed into **one new defect**, **one epic update**, **one new epic** — no new BUG for ML external profile.

| Item | Decision | Rationale |
|------|----------|-----------|
| **ML on external profile** | **Update US-0013 only** (P1→**P0**) | Already tracked as BUG-0010 AC3 → US-0013; operator confirms same gap (`stats-forecast` not on external merge, `sidecar_disabled`) |
| **Income/Fixed always 0** | **New BUG-0012** (P1) | Distinct from BUG-0010 DONE; code gap `categorize_delta` → `map_category(None)` — Fixed never populated; Income often 0 |
| **AI categorization** | **New US-0015** (P2 epic) | Operator wants AI for bucket detection; exceeds quick defect — BUG-0012 closes config/projection path first |

### Overlap decisions

- **US-0013 OPEN** — extend/prioritize, do **not** duplicate with new BUG (operator issue 1)
- **BUG-0010 DONE** — baseline balances fixed; monthly bucket split is post-fix gap
- **BUG-0007 OPEN** — AI chat merchant discovery; coordinate only, do not merge
- **DEC-0007** — category config exists; projection path broken in `backend/src/forecast/project.rs`

### Known code gap (from operator /ask)

```132:142:backend/src/forecast/project.rs
fn categorize_delta(
    delta: f64,
    category_names: &HashMap<String, String>,
    config: &ForecastConfig,
) -> Bucket {
    if delta >= 0.0 {
        return Bucket::Income;
    }
    let _ = category_names;
    map_category(None, config)
}
```

Negative deltas always → Variable; Fixed bucket never receives categorized fixed-cost patterns.

### Acceptance rows

| ID | Bug/Story | Rows |
|----|-----------|------|
| **BUG-0012** | Forecast monthly buckets | **(AG)** Income non-zero when mirror has income categories; **(AH)** Fixed non-zero when mirror has fixed-cost categories |
| **US-0013** | ML external (updated) | Existing criteria + explicit external-profile sidecar criterion |
| **US-0015** | AI forecast buckets (new) | 5 criteria; parent BUG-0012 AG/AH |

### Intake evidence (US-0078)

| Work item | `intake_run_id` | Bundle |
|-----------|-----------------|--------|
| BUG-0012 | `intake-20260605-forecast-monthly-buckets` | `handoffs/intake_evidence/intake-20260605-forecast-monthly-buckets.json` |
| US-0013 reconfirm | _(overlap note in BUG-0012 decomposition)_ | `handoffs/intake_evidence/intake-20260605-forecast-wealth-ml.json` (original) |

- `selected_pack`: `small-intake-pack`
- Validation: `intake_evidence_validate.py` OK; `bug_issue_validate.py --check-acceptance` OK
- `assumptions_confirmed`: `(none)`

### Recommended `/auto` target

**`bug-target=BUG-0012`** — P1 forecast monthly decomposition defect (config-path fix before US-0015 AI epic). Queue after P0 bugs unless operator reprioritizes: BUG-0009 → BUG-0012 → BUG-0007/0008/0011. **US-0013** remains epic queue after P0 bug drain or explicit epic `/auto`.

### Triad check (intake phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md` | BUG-0012 + US-0013 update + US-0015 | pass |
| `docs/product/acceptance.md` | AG/AH + US-0013/0015 criteria | pass |
| Intake evidence JSON | distinct `topic_coverage`, validator OK | pass |

`triad_hot_surface`: intake pack prepended; prior discovery-20260605-bug0010 retained below

---

## discovery-20260605-bug0010 — BUG-0010 forecast/wealth/ML discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Bug:** BUG-0010  
**Orchestrator:** `auto-20260605-bug0010-001`  
**Next phase:** `/architecture`

### Summary

Discovery **confirms all three sub-defects AA/AB/AC** on `financegnome.omniflow.cc` (US-0010 external, 922+ tx, post-BUG-0004/0006). Public curl probes (no secrets). Primary failure is **wrong mirror account balances** driving forecast and wealth — not Grafana (BUG-0009) or ML model research.

| Sub-defect | Verdict | Root cause (confirmed) |
|------------|---------|------------------------|
| **AA** | CONFIRMED | Giro acct **114** starts forecast at **-3395.75** → 3mo end **-25365.78**; monthly ~6029 EUR outflows — **DEC-0007 math correct, inputs wrong**. Accts 115/116 flat **0.00**. DEC-0060 parse fixed NULL but balances still 0/negative vs operator expectation. |
| **AB** | CONFIRMED | Wealth `total_eur: 0.0`; 2 accounts (115, 116) at 0; **114 excluded** (`COALESCE(balance,0) >= 0`). Snapshots exist with zero totals — writer OK, data wrong. |
| **AC** | CONFIRMED (BUG vs epic split) | `forecast_ml.enabled=false`; no `stats-forecast` on external profile (DEC-0049). ML phase never runs → `ml_skipped_reason: null`. UI falsely shows **"ML skipped: ML forecast unavailable"** when ML not configured. Full ML on omniflow → **US-0013**. |

### Live probe evidence (2026-06-05)

| Endpoint | Result |
|----------|--------|
| `GET /api/v1/forecast/long-term?account_id=114&horizon=3` | end balance **-25365.78** |
| `GET /api/v1/forecast/daily?account_id=114` | tomorrow **-4042.41**; series starts **-3395.75** |
| `GET /api/v1/forecast/daily?account_id=115` | all **0.00** |
| `GET /api/v1/wealth` | 2 accounts, `total_eur: 0.0`; Giro 114 absent |
| `GET /api/v1/forecast/meta` | `ml_computation_id: null`, `ml_skipped_reason: null`, `low_confidence: true` |
| `GET /api/v1/sync/status` | `scheduled_exchanges` success `2026-06-05T16:28:57Z` |

### Fix tasks (for architecture)

| ID | Scope | Task |
|----|-------|------|
| AA1 | AA | Firefly account balance mirror — beyond DEC-0060 string parse; verify accts 114/115/116 source vs DB |
| AA2 | AA | Full Firefly sync + recompute gate after AA1 |
| AA3 | AA | Warn when asset `starting_balance <= 0` with history (API/UI) |
| AB1 | AB | Include or surface negative-balance asset accounts (114) in wealth |
| AB2 | AB | Zero-total wealth empty-state with operator guidance |
| AB3 | AB | Re-verify snapshots post AA1 |
| AC1 | AC | Meta: `sidecar_disabled` when ML off (sync or meta derive) |
| AC2 | AC | UI: ML not enabled ≠ ML skipped |
| AC3 | → US-0013 | External profile sidecar + `FORECAST_ML_ENABLED` |

### US-0013 boundary

- **BUG-0010 closes:** plausible baseline balances (AA) + honest ML degraded messaging (AC1/AC2) + non-empty wealth with correct totals (AB, via AA1).
- **US-0013 closes:** ML overlay running on omniflow (sidecar, env, health, compare tab).

### Intake evidence

- `intake_run_id`: `intake-20260605-forecast-wealth-ml`
- Bundle: `handoffs/intake_evidence/intake-20260605-forecast-wealth-ml.json`

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#BUG-0010` | Discovery notes + fix tasks + US-0013 boundary | pass |
| `docs/product/acceptance.md` | AA/AB/AC unchanged | pass |
| Live probes | -25365.78, wealth zero, ML null meta | pass |

`triad_hot_surface`: rollover intake pack → `handoffs/archive/po-to-tl-pack-20260605-a.md`; BUG-0010 discovery prepended

### Recommended next steps

1. `/architecture` — freeze AA1 balance ingest contract, AB1 visibility, AC1/AC2 ML posture
2. `/sprint-plan` — quick sprint Q0013+ (estimate 6–8 tasks after arch)
3. Defer AC3 to US-0013 epic queue

---

## intake-20260605-bug0008-0011 — Omniflow production regression batch 2 (4 operator issues)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Bugs:** BUG-0008, BUG-0009, BUG-0010, BUG-0011 (+ US-0013, US-0014 epics)  
**Next phase:** `/discovery` on **BUG-0010** (recommended `/auto` P0 target)

### Summary

Operator report on `financegnome.omniflow.cc` (US-0010 external profile, **922+ transactions** synced, post-BUG-0004/0006 fixes) decomposed into **four bugs** and **two deferred US epics**. Overlap with **BUG-0004** (DONE — partial subscriptions/Grafana/forecast fixes) and **BUG-0007** (OPEN — AI chat enumeration) explicitly **related, not merged**.

| Bug | Priority | Sub-defects | Overlap decision |
|-----|----------|-------------|------------------|
| **BUG-0008** | P1 | W (33 alerts vs 11 list), X (under-detection) | Extends BUG-0004 J — new alert/list mismatch; coordinate BUG-0007 only |
| **BUG-0009** | P0 | Y (Grafana empty), Z (no account overview) | Post-BUG-0004 K/L regression/partial — separate Grafana surface |
| **BUG-0010** | P0 | AA (-25365.78 forecast), AB (wealth empty), AC (ML skipped) | Post-BUG-0004 L; epic → **US-0013** |
| **BUG-0011** | P1 | AD (empty plan no-op), AE (compare sums), AF (plan-vs-actual 404) | Supersedes BUG-0004 404 empty-state note; epic → **US-0014** |

### Intake evidence (US-0078)

| Bug | `intake_run_id` | Evidence bundle |
|-----|-----------------|-----------------|
| BUG-0008 | `intake-20260605-subscription-alerts-detection` | `handoffs/intake_evidence/intake-20260605-subscription-alerts-detection.json` |
| BUG-0009 | `intake-20260605-grafana-account-overview` | `handoffs/intake_evidence/intake-20260605-grafana-account-overview.json` |
| BUG-0010 | `intake-20260605-forecast-wealth-ml` | `handoffs/intake_evidence/intake-20260605-forecast-wealth-ml.json` |
| BUG-0011 | `intake-20260605-planning-mode-broken` | `handoffs/intake_evidence/intake-20260605-planning-mode-broken.json` |

All bundles: `small-intake-pack`, validation OK, `assumptions_confirmed: (none)`.

### Decomposition rationale

- **Split axis:** product surface (subscriptions alerts, Grafana embed, React forecast/wealth, planning UX)
- **Why not one BUG:** independent acceptance rows, different code paths, parallel `/auto` bug-queue candidates
- **Why US-0013/0014:** operator "implement fully" for ML forecast and intuitive planning exceeds quick defect scope

### Recommended `/auto` target

**BUG-0010** (P0 — wrong forecast **-25365.78** + empty wealth blocks core product value). Queue order suggestion: BUG-0010 → BUG-0009 → BUG-0008 → BUG-0011 → BUG-0007 (existing OPEN).

### Triad check (intake phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md` | BUG-0008–0011 + US-0013/0014 blocks | pass |
| `docs/product/acceptance.md` | W–AF sub-rows + US-0013/0014 criteria | pass |
| Intake evidence JSON | 4 bundles, distinct `topic_coverage` | pass |

`triad_hot_surface`: check pass (intake at hot tail; archive pack `handoffs/archive/po-to-tl-pack-20260605-a.md`)

---

