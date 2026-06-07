# Architecture archive pack (2026-06-06)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 14
- First archived heading: `## BUG-0012 — Forecast monthly Income/Fixed buckets always zero`
- Last archived heading: `## BUG-0012 — Forecast monthly Income/Fixed buckets always zero`
- Verification tuple (mandatory):
  - archived_body_lines=164
  - preamble_lines=10
  - retained_body_lines=3000

---

## BUG-0012 — Forecast monthly Income/Fixed buckets always zero

**Status:** architecture complete (2026-06-05)  
**Discovery:** `discovery-20260605-bug0012` in `handoffs/archive/po-to-tl-pack-20260605-b.md`  
**Research:** [R-0063](research.md#r-0063--bug-0012-forecast-monthly-bucket-component-attribution)  
**Decisions:** **DEC-0067** (component-level monthly bucket attribution); extends **DEC-0007** (category→bucket map), **DEC-0013** (recurrence core / `category_ids`)  
**Sprint:** `/quick` recommended (4–6 tasks)  
**Acceptance:** `docs/product/acceptance.md` — BUG-0012 rows **(AG)**, **(AH)**  
**Related:** BUG-0006 DONE (`category_id` ingest); BUG-0010 DONE (balance mirror); **US-0015** OPEN (AI buckets — out of scope); **US-0013** OPEN (ML — unchanged)

### Symptom chain (frozen)

Operator on US-0010 external profile post-BUG-0010: Full Firefly sync + forecast recompute; `/forecast` Monthly tab (or `GET /api/v1/forecast/monthly`) shows **Income: 0** and **Fixed: 0** while mirror holds categorized salary/income and rent/utilities/subscription-class rows.

| Sub | Gap | Effect |
|-----|-----|--------|
| **AG** | `categorize_delta` routes Income only when **net daily `delta >= 0`** | Funded-account days net-negative → Income permanently **0** |
| **AH** | `map_category(None, …)` for all `delta < 0` | Recurring rent/utilities due-days → **Variable** only → Fixed **0** |
| Both | `category_names` loaded in `service.rs` but unused in projection | DEC-0007 TOML `[forecast.category_buckets]` never applied |
| Both | `RecurringPattern` lacks `category_id` | `RecurrenceGroup.category_ids` dropped at detect |

**Read path OK:** API + `ForecastPage` Monthly tab display only — fix is projection **write** path (`project_account` → `forecast_cashflow_monthly`).

`isolation_scope`: artifact + repo source only; no host `.env` / secrets read.

### Fix contract (DEC-0067)

Replace single `categorize_delta(net_delta)` with **per-component monthly accumulation**; preserve daily balance math.

```text
BUG-0012
├── AG1 — Component monthly attribution (P0)
│   └── project.rs: rolling + each recurring due → separate bucket; balance += delta unchanged
├── AH1 — RecurringPattern category carry (P0)
│   └── types.rs + recurring.rs: category_id from RecurrenceGroup.mode; subscription override lookup
├── T1 — Unit tests AG/AH (P0)
│   └── project.rs: salary+rent scenario; same-day mixed; Variable regression
├── D1 — Operator TOML checklist (P1)
│   └── runbook omniflow: extend [forecast.category_buckets] for non-English labels
└── V1 — Operator verify (P1)
    └── verify-work smoke rows AG/AH on financegnome.omniflow.cc
```

**Out of scope:** ML buckets (US-0013); AI category inference (US-0015); fuzzy name matching; transaction-sum balance recompute; frontend changes (cards already bind API fields).

### AG1 — Component monthly attribution (frozen)

**Problem:** One net delta per day collapses same-day salary + rent into a single bucket by sign.

**Contract — daily loop in `project_account`:**

```rust
// Balance path (UNCHANGED)
let delta = rolling.daily_rate + recurring_due_sum;
balance += delta;

// Monthly map (NEW — per component)
accumulate_bucket(&mut monthly_map, month_key, Bucket::Variable, rolling.daily_rate);
for due in recurring_dues_today {
    let bucket = resolve_bucket(due.pattern.category_id, &category_names, config);
    accumulate_bucket(&mut monthly_map, month_key, bucket, due.amount);
}
```

| Component | Bucket rule | Rationale |
|-----------|-------------|-----------|
| `rolling.daily_rate` | **Variable** (sign preserved in amount) | DEC-0007 variable residual layer; positive misc inflow is uncategorized → not Income |
| Recurring due | `category_id` → name → `map_category(name, config)` | DEC-0007 TOML map |
| Unmapped name | **Variable** | Existing `map_category` default |
| Transfers | Excluded | DEC-0007 |

**`free_cashflow`:** recompute daily from component sums: `income - fixed_costs - variable_costs` (existing formula).

**Alternatives rejected:** net-delta sign fix only; dominant-category rebucketing; full tx replay (R-0063).

**Files:** `backend/src/forecast/project.rs`, `backend/src/forecast/categories.rs` (helper `resolve_bucket` if extracted)

**Risks:** Variable total decreases when fixed moves out — intended; regression test required.

### AH1 — RecurringPattern category carry (frozen)

**Problem:** `detect_patterns` drops `RecurrenceGroup.category_ids` already collected in `recurrence/detect.rs`.

**Contract:**

| Step | Rule |
|------|------|
| Schema | `RecurringPattern { …, category_id: Option<String> }` |
| Detect | Mode of non-null `category_ids` in group; tie-break: latest tx `date` |
| Subscription override | Inherit from replaced heuristic `category_id`; else lookup latest mirror tx with matching normalized `payee_key` |
| Projection | Due-day bucket uses `pattern.category_id` — not description or net sign |

**Category resolution chain (frozen):**

```text
category_id (mirror / group mode)
  → category_names: HashMap<firefly_id, name>
  → map_category(lowercase_trim(name), config)
  → Bucket::{Income, Fixed, Variable}
```

**TOML keys match lowercased category name**, not Firefly id (`default.toml`: `salary`, `rent`, …).

**Files:** `backend/src/forecast/types.rs`, `backend/src/forecast/recurring.rs`, `backend/src/forecast/project.rs` (`apply_subscription_override`)

**Risks:** German/custom labels miss default keys — operator TOML extension (D1); not a code defect.

### D1 — Operator TOML alignment doc (frozen)

**Problem:** Omniflow Firefly categories may use non-English names (`Gehalt`, `Miete Nebenkosten`) absent from `default.toml`.

**Contract:** Add runbook § Omniflow checklist:

1. List mirror category **names** for income/fixed rows used in acceptance month.
2. Add matching keys under `[forecast.category_buckets]` in operator TOML (lowercase name = key).
3. Recompute forecast after config change.
4. Re-smoke AG/AH.

**No code change** to `default.toml` in bug scope — operator-owned config on external profile.

**Files:** `docs/engineering/runbook.md` (omniflow section)

### Task map (sprint-plan input)

| Order | Task | Layer | Acceptance |
|-------|------|-------|------------|
| 1 | **AH1** category carry + override lookup | backend forecast/recurring | **AH** (enables fixed routing) |
| 2 | **AG1** component monthly attribution | backend project | **AG**, **AH** |
| 3 | **T1** unit tests | backend tests | **AG**, **AH** |
| 4 | **D1** runbook TOML checklist | docs | operator AG/AH on omniflow |
| 5 | **V1** verify-work | uat | **AG**, **AH** |

**Deploy order:** AH1 + AG1 + T1 in one PR; D1 docs same PR or follow-up; operator Full sync + recompute before V1.

### Test strategy (frozen — maps acceptance AG/AH)

| Check | Type | Pass criteria |
|-------|------|---------------|
| AG — income | Unit (`project.rs`) | Salary recurring with `category_id` → income bucket → first forecast month `income > 0` |
| AH — fixed | Unit | Rent recurring with `category_id` → fixed bucket → `fixed_costs > 0` |
| Mixed same-day | Unit | Salary due + rent due same day → both buckets non-zero; `balance` path unchanged |
| Variable regression | Unit | Discretionary coffee recurring → Variable; rejected fingerprint excluded |
| `map_category` wiring | Unit | `category_names` + TOML key resolution |
| Subscription override | Unit | Confirmed override inherits/lookup `category_id` |
| Integration | Optional post-BUG-0006 | DB fixture → recompute → monthly API |
| V1 | Operator | AG/AH on `financegnome.omniflow.cc` after deploy + TOML if needed |

### Decisions (BUG-0012)

| Topic | Resolution |
|-------|------------|
| Monthly attribution model | **DEC-0067** — component-level; net-delta `categorize_delta` rejected |
| Rolling residual bucket | **Variable** (positive and negative) |
| Unmapped categories | **Variable** default |
| Positive rolling → Income | **Rejected** — Income via categorized recurring only |
| Daily balance / horizons | **Unchanged** |
| AI / fuzzy mapping | **US-0015** — out of scope |

### Next phase

**`/sprint-plan`** — quick sprint 4–6 tasks after architecture.

---

