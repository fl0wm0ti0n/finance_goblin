# Tasks — Q0028 (BUG-0020)

**Bug:** BUG-0020  
**Task count:** 7 (all P0 mandatory; 7/12 under `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260611-q0028-bug0020`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **DA1** | Task **DA1** | migration 016 YouTube confirmed merge + Strom pending collapse |
| **DB1** | Task **DB1** | migration 016 DEC-0100 RANK backfill (confirmed only) |
| **DA2** | Task **DA2** | SubscriptionsPage All-tab `rejected`/`inactive` exclusion |
| **DA3** | Task **DA3** | detection.rs forward pending guard |
| **T1** | Task **T1** | `bug0020_subscription_list_quality.rs` BI/BJ + regression |
| Static gate | Task **G1** | `cargo test` + migration idempotency smoke |
| BI/BJ runtime | Task **V1** | verify-work after deploy + migration apply |

## Execute order

```text
DA1 → DB1 (single migration file)
  ∥ DA2 ∥ DA3
  → T1
  → G1
  → operator: BACKEND_FRONTEND_DEPLOY + migration 016
  → operator: FULL_FIREFLY_SYNC
  → V1 verify-work
```

**Parallelism:** DA2 (frontend) and DA3 (backend) are disjoint files — may run
parallel to each other after DA1/DB1 design is frozen; T1 blocked on all four
implementation slices.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **BI** | DA1, DA2, DA3, T1, G1, V1 | Confirmed API ≤1 per `payee_key`; All tab no Strom/YouTube dupes; detection no new YouTube confirmed dup |
| **BJ** | DB1, T1, G1, V1 | R-0090 oracle: netflix/kindle→18, youtube→66, hgp→56, florian gabriel→3 |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| DA1 | Migration 016 reconcile (YouTube + Strom) | 3h | open | **BI** | P0 |
| DB1 | Migration 016 display_category backfill | 1.5h | open | **BJ** | P0 |
| DA2 | SubscriptionsPage All-tab filter | 1h | open | **BI** | P0 |
| DA3 | detection.rs forward pending guard | 2h | open | **BI** | P0 |
| T1 | Integration tests BI/BJ + regression | 3h | open | **BI**, **BJ** | P0 |
| G1 | Automated gate | 0.5h | open | **BI**, **BJ** | P0 |
| V1 | verify-work operator smoke | 2h | open | **BI**, **BJ** | P0 |

---

## DA1 — Migration 016 reconcile (YouTube + Strom)

**Status:** open  
**Depends on:** —  
**Estimate:** 3h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0020 **BI** — **DEC-0109** reconcile

### Description

Create `backend/migrations/016_bug0020_subscription_list_quality.sql` (idempotent,
transaction-wrapped) implementing **DEC-0109** reconcile half:

**YouTube cluster** (2 `confirmed`, same `display_name`, `interval_days` 30,
`interval_matches`):

1. Survivor = earliest `confirmed_at`.
2. Relink `subscription_pattern_transactions` loser → survivor (`ON CONFLICT DO NOTHING`, delete loser orphans).
3. Reassign `subscription_alerts.pattern_id` and `subscription_pattern_tags` to survivor.
4. Update survivor `payee_key`, `fingerprint`, `last_seen_at`, `current_amount` from fresher loser data.
5. Mark loser `status = 'inactive'`.

**Strom cluster** (2 `pending`, same `display_name`, `interval_days` 31):

1. Survivor = highest `last_seen_at`.
2. Relink txs; mark loser `status = 'rejected'`.
3. Leave existing `rejected` Strom row unchanged.

**Invariants:** merge only when `interval_matches` (DEC-0086 ±3d); never collapse
distinct cadences; no action on all-`rejected` exact `payee_key` dup groups.

Adapt SQL to probed row ids at execute time (R-0090). Handle `fingerprint` UNIQUE
on survivor update; spt PK conflicts via `ON CONFLICT DO NOTHING`.

**Files:** `backend/migrations/016_bug0020_subscription_list_quality.sql`

### Done when

- [ ] YouTube reconcile: single confirmed survivor per cluster; loser `inactive`
- [ ] Strom reconcile: single pending survivor; loser `rejected`; existing rejected unchanged
- [ ] `interval_matches` gate enforced in merge predicates
- [ ] Migration idempotent (safe re-run)

### Verification

Post-migration probe: `SELECT payee_key, COUNT(*) FROM subscription_patterns WHERE status = 'confirmed' GROUP BY payee_key HAVING COUNT(*) > 1` → 0 rows.

---

## DB1 — Migration 016 confirmed `display_category_id` backfill

**Status:** open  
**Depends on:** DA1 (same migration file; reconcile before backfill)  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0020 **BJ** — **DEC-0109** + **DEC-0100**

### Description

Append to the same migration 016 file (after reconcile block) a batch
`UPDATE subscription_patterns SET display_category_id = …` for `status = 'confirmed'`
using **DEC-0100** RANK SQL (identical semantics to
`repository.rs` `compute_display_category_id`). NULL when no categorized links.

Expected post-backfill oracle (R-0090):

| `payee_key` (survivor) | `display_category_id` |
|------------------------|----------------------|
| netflix | 18 |
| kindle unltd | 18 |
| youtube (merged) | 66 |
| hgp unfall / hgp haushalt | 56 |
| mitgliedsbeitrag - florian gabriel | 3 |

Pending/rejected patterns remain NULL.

**Files:** `backend/migrations/016_bug0020_subscription_list_quality.sql`

### Done when

- [ ] All confirmed patterns with categorized linked txs get non-null `display_category_id`
- [ ] RANK tie-break matches `compute_display_category_id` (count DESC, max date DESC)
- [ ] No list-time recompute introduced

### Verification

`SELECT COUNT(*) FILTER (WHERE display_category_id IS NOT NULL), COUNT(*) FROM subscription_patterns WHERE status = 'confirmed'` — non-zero numerator when links exist.

---

## DA2 — SubscriptionsPage All-tab filter

**Status:** open  
**Depends on:** — (may parallel DA3)  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0020 **BI** — **DEC-0109** All-tab contract

### Description

In `frontend/src/pages/SubscriptionsPage.tsx`, when `tab === "all"`, filter the
`listQuery` response client-side:

```typescript
patterns.filter((p) => p.status !== "rejected" && p.status !== "inactive")
```

**Pending** tab (`?status=pending`) and **Standing** tab unchanged. API
`GET /api/v1/subscriptions` without `status` remains unfiltered.

**Files:** `frontend/src/pages/SubscriptionsPage.tsx`

### Done when

- [ ] All tab shows only `pending` + `confirmed`
- [ ] Pending and Standing tabs unchanged
- [ ] No API query param change for unfiltered list fidelity

### Verification

Manual or component test: All tab with mixed-status fixture hides rejected/inactive rows.

---

## DA3 — detection.rs forward pending guard

**Status:** open  
**Depends on:** — (may parallel DA2)  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0020 **BI** — **DEC-0109** forward guard

### Description

In `backend/src/subscriptions/detection.rs` / `upsert_pending_pattern` path
(`run_candidates`), when no confirmed merge occurred:

- Existing: skip when `(payee_key, interval_days)` matches **rejected** via `interval_matches`.
- **Add:** skip pending INSERT when matches any **confirmed** payee-interval via
  `interval_matches` but `merge_confirmed_pattern` returned `Ok(false)` (fingerprint
  conflict) — log warn.

Extends DEC-0085 rejection load; prevents new Strom-style pending siblings after reconcile.

**Files:** `backend/src/subscriptions/detection.rs`

### Done when

- [ ] Confirmed payee-interval match + failed merge → no new pending row
- [ ] Existing rejected skip path preserved
- [ ] `interval_matches` (DEC-0086) used for tolerance

### Verification

Unit or integration test: simulate fingerprint conflict path → no duplicate pending INSERT.

---

## T1 — Integration tests BI/BJ + regression

**Status:** open  
**Depends on:** DA1, DB1, DA2, DA3  
**Estimate:** 3h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0020 **BI**, **BJ**

### Description

Add `backend/tests/bug0020_subscription_list_quality.rs` covering:

1. **BI** — post-migration fixture or seed: `list_patterns` with `status=confirmed`
   returns ≤1 row per `payee_key`; no duplicate YouTube `display_name` cluster.
2. **BJ** — confirmed patterns return expected `display_category_id` per R-0090 oracle
   (netflix/kindle→18, youtube→66, hgp→56, florian gabriel→3).
3. **Regression** — `GET /api/v1/subscriptions/discover` unchanged shape; tag
   assign/filter smoke; detection forward guard (DA3) no duplicate pending for
   confirmed payee-interval.

Reuse `repository.rs` helpers where practical; optional `reconcile_cluster` test
helper per architecture § BUG-0020.

**Files:** `backend/tests/bug0020_subscription_list_quality.rs`

### Done when

- [ ] BI assertion: no confirmed `payee_key` duplicates after reconcile fixture
- [ ] BJ assertion: oracle category ids on representative payee_keys
- [ ] Discover/tags regression tests pass
- [ ] DA3 guard covered by at least one test case

### Verification

`cargo test --test bug0020_subscription_list_quality` → all PASS.

---

## G1 — Automated gate

**Status:** open  
**Depends on:** T1  
**Estimate:** 0.5h  
**Acceptance hook:** DEC-0109 verification gates — static/automated

### Description

Run and record automated checks in `sprints/quick/Q0028/progress.md`:

1. `cargo test --test bug0020_subscription_list_quality` → PASS.
2. Related subscription test suites unchanged (no regressions in detection/merge tests).
3. Migration 016 SQL parses; idempotency note documented in progress.
4. `git diff --stat` blast radius matches frozen file list (migration, detection.rs,
   SubscriptionsPage.tsx, test file only).

**Files:** `sprints/quick/Q0028/progress.md`

### Done when

- [ ] All automated checks PASS, recorded in progress.md
- [ ] No forbidden paths touched (`list_patterns` dedup, Firefly writes, etc.)

### Verification

Test output pasted in progress.md; diff stat confirms scope.

---

## V1 — verify-work operator smoke

**Status:** open  
**Depends on:** G1 + operator BACKEND_FRONTEND_DEPLOY + migration 016  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0020 **BI**, **BJ**

### Description

Populate `sprints/quick/Q0028/uat.md` and `uat.json` after deploy on
localhost:18080 (and optional omniflow OIDC):

1. **BI-API** — `GET /api/v1/subscriptions?status=confirmed` ≤1 per `payee_key`; no duplicate YouTube.
2. **BI-ALL** — `/subscriptions` **All** tab: no triplicate Strom / duplicate YouTube.
3. **BJ** — API samples: netflix/kindle→18, youtube→66, hgp→56, florian gabriel→3.
4. **REG-DISCOVER** — discover endpoint unchanged.
5. **REG-TAGS** — tag CRUD/filter unchanged.
6. **REG-DETECT** — Full sync: no new confirmed YouTube dup family.
7. **OIDC** — list endpoints smoke on omniflow profile.

Optional pre/post SQL probes per architecture § BUG-0020.

**Files:** `sprints/quick/Q0028/uat.md`, `sprints/quick/Q0028/uat.json`

### Done when

- [ ] Rows **BI**, **BJ** probed per acceptance.md matrix
- [ ] Regression gates documented
- [ ] `uat.md` and `uat.json` populated with results

**Operator gates:** **BACKEND_FRONTEND_DEPLOY**; **MIGRATION_016_APPLY**;
**FULL_FIREFLY_SYNC** for detection regression.
