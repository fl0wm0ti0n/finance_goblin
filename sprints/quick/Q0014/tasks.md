# Tasks — Q0014 (BUG-0012)

**Bug:** BUG-0012  
**Task count:** 5 (within `SPRINT_MAX_TASKS=12`)  
**Sprint-plan ref:** `sprint-plan-20260605-q0014-bug0012`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **AH1** | Task **AH1** | `RecurringPattern.category_id` carry from `RecurrenceGroup.category_ids` |
| **AG1** | Task **AG1** | Per-component `monthly_map` accumulation in `project_account` |
| **T1** | Task **T1** | Unit tests salary+rent scenario; same-day mixed; Variable regression |
| **D1** (code) | Task **D1** | Remove/retire `categorize_delta` for monthly bucket path |
| **D1** (runbook) | **V1** prep | Omniflow TOML checklist for German/custom category names |
| **V1** | Task **V1** | verify-work smoke rows AG/AH on omniflow |

## Execute order

```text
AH1 → AG1 → T1 → D1
  → single PR deploy
  → operator manual Full Firefly sync + recompute
  → operator TOML extend if label mismatch
  → V1 verify-work
```

**Parallelism:** T1 and D1 may follow AG1 sequentially; D1 depends on AG1 landing component path.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **(AG)** | AG1, T1, V1 | Income > 0 when salary/income recurring with category_id |
| **(AH)** | AH1, AG1, T1, V1 | Fixed > 0 when rent/utilities recurring with category_id |

| ID | Title | Est. | Status | Acceptance |
|----|-------|------|--------|------------|
| AH1 | RecurringPattern.category_id + recurring bucket path | 2h | done | **(AH)** |
| AG1 | Income from categorized recurring inflows | 2.5h | done | **(AG)**, **(AH)** |
| T1 | Unit tests monthly_map component attribution | 1.5h | done | **(AG)**, **(AH)** |
| D1 | Retire net-delta categorize_delta for monthly buckets | 0.5h | done | **(AG)**, **(AH)** |
| V1 | verify-work + runbook TOML checklist | 1h | prep | **(AG)**, **(AH)** |

---

## AH1 — RecurringPattern.category_id + recurring bucket path

**Status:** done  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0012 **(AH)**

### Description

Add `category_id: Option<String>` to `RecurringPattern`. In `detect_patterns`, set from mode of non-null `RecurrenceGroup.category_ids` (tie-break: latest tx date). In `apply_subscription_override`, inherit `category_id` from replaced heuristic or runtime lookup from mirror tx matching `payee_key`. Enable recurring due-day bucket resolution via `category_id` → `category_names` → `map_category`.

**Files:** `backend/src/forecast/types.rs`, `backend/src/forecast/recurring.rs`, `backend/src/forecast/project.rs`

### Done when

- [x] `RecurringPattern` carries `category_id` from detection
- [x] Subscription override inherits or looks up `category_id`
- [x] Unit: rent recurring with fixed `category_id` resolves to Fixed bucket path
- [x] `cargo test --lib` forecast/recurring PASS

---

## AG1 — Income from categorized recurring inflows

**Status:** done  
**Depends on:** AH1  
**Estimate:** 2.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0012 **(AG)**, **(AH)**

### Description

Replace single `categorize_delta(net_delta)` monthly accumulation with per-component `monthly_map` per DEC-0067. Rolling `daily_rate` → Variable (sign preserved). Each recurring due → `resolve_bucket(pattern.category_id, category_names, config)`. Daily balance path unchanged: `balance += delta`.

**Files:** `backend/src/forecast/project.rs`, `backend/src/forecast/categories.rs`

### Done when

- [x] Rolling residual accumulates to Variable only (positive and negative)
- [x] Salary recurring with income `category_id` → Income bucket
- [x] Rent recurring with fixed `category_id` → Fixed bucket
- [x] Same-day salary + rent → both buckets non-zero; balance unchanged
- [x] `cargo test --lib` project PASS

---

## T1 — Unit tests monthly_map component attribution

**Status:** done  
**Depends on:** AG1  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0012 **(AG)**, **(AH)**

### Description

Frozen test contract from architecture § Test strategy: salary+rent scenario, same-day mixed, Variable regression when fixed moves out of Variable, `map_category` wiring with `category_names`, subscription override category carry.

**Files:** `backend/src/forecast/project.rs` (tests)

### Done when

- [x] Unit: AG — salary recurring → first month `income > 0`
- [x] Unit: AH — rent recurring → `fixed_costs > 0`
- [x] Unit: same-day mixed → both non-zero; balance path unchanged
- [x] Unit: discretionary coffee recurring → Variable; rejected fingerprint excluded
- [x] `cargo test --lib` forecast PASS

---

## D1 — Retire net-delta categorize_delta for monthly buckets

**Status:** done  
**Depends on:** AG1  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0012 **(AG)**, **(AH)**

### Description

Remove or retire `categorize_delta` usage for monthly bucket accumulation. Ensure no dead path passes `None` to `map_category` for negative deltas. Remove `let _ = category_names` dead binding if present. Keep `map_category` unit tests intact.

**Files:** `backend/src/forecast/project.rs`, `backend/src/forecast/service.rs`

### Done when

- [x] No monthly bucket path calls `categorize_delta`
- [x] `category_names` wired through projection (not ignored)
- [x] `cargo test --lib` PASS; no new warnings from dead code

---

## V1 — verify-work + runbook TOML checklist

**Status:** prep (docs done; runtime verify pending deploy)  
**Depends on:** AH1, AG1, T1, D1 deploy + Full Firefly sync + recompute  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0012 **(AG)**, **(AH)**

### Description

Prepare `sprints/quick/Q0014/uat.md` smoke checklist. Add runbook § Omniflow checklist: list mirror category names for income/fixed rows; extend `[forecast.category_buckets]` in operator TOML (lowercase name = key); recompute after config change. After deploy + sync, probe `GET /api/v1/forecast/monthly` and `/forecast` Monthly tab on `financegnome.omniflow.cc`.

**Files:** `sprints/quick/Q0014/uat.md`, `docs/engineering/runbook.md`

### Done when

- [x] Runbook omniflow TOML checklist documented
- [ ] Row **(AG)**: Income non-zero on monthly view when mirror has income categories
- [ ] Row **(AH)**: Fixed non-zero when mirror has fixed-cost categories
- [x] Regression footer documented (OIDC + bundled-firefly)

**Operator gates:** Full Firefly sync + recompute before V1; TOML extend if German/custom labels miss default keys.
