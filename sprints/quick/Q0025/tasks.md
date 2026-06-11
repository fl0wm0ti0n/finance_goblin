# Tasks — Q0025 (BUG-0017)

**Bug:** BUG-0017  
**Task count:** 6 (all P0 mandatory; 6/12 under `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260610-q0025-bug0017`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **AY1** | Task **AY1** | DEC-0105 DROP+ADD both CHECK constraints; `forecast_bucket_assignment` + extended statuses |
| **BA1** | Task **BA1** | DEC-0106 `paired_baseline_id` ON DELETE CASCADE |
| **BA2** | Task **BA2** | DEC-0106 `ml_enhanced` before `baseline` in `enforce_retention` |
| **BD1** | Task **BD1** | `isFetched && !isError && !computation_id` empty guard |
| **T1** | Task **T1** | Paired baseline+ML prune integration test |
| **V1** | Task **V1** | verify-work operator smoke AY–BD |

## Execute order

```text
AY1 (audit CHECK migration)
  → BA1 (FK CASCADE migration)
  → BA2 (retention loop order)
  → BD1 (ForecastPage guard)  [parallel with BA2 after BA1]
  → T1 (retention integration test)
  → single backend+frontend release
  → operator: BACKEND_FRONTEND_DEPLOY + Full sync + recompute
  → V1 verify-work
```

**Parallelism:** BD1 may run in parallel with BA2 after BA1; T1 depends on BA1+BA2; V1 blocked on deploy.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **AY** | AY1, V1 | `ai_tool_audit` accepts `forecast_bucket_assignment`; no tool_name CHECK WARN in sync logs |
| **AZ** | AY1, V1 | `low_confidence` rows persist; no result_status CHECK WARN |
| **BA** | BA1, BA2, T1, V1 | Recompute succeeds; meta fresh `computation_id`, `stale=false` |
| **BB** | V1 | Month-bucket SQL; ML meta honest after BA fix |
| **BC** | V1 | Planning Compare no Plan stale after successful recompute |
| **BD** | BD1, V1 | No false empty flash on Forecast nav when meta has data |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| AY1 | `ai_tool_audit` CHECK migration | 2h | open | **AY**, **AZ** | P0 |
| BA1 | `paired_baseline_id` CASCADE migration | 1.5h | open | **BA** | P0 |
| BA2 | Retention loop order | 1h | open | **BA** | P0 |
| BD1 | ForecastPage `isFetched` empty guard | 1h | open | **BD** | P0 |
| T1 | Retention integration test | 2h | open | **BA** | P0 |
| V1 | verify-work operator smoke | 2h | open | **AY**–**BD** | P0 |

---

## AY1 — `ai_tool_audit` CHECK migration

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0017 **AY**, **AZ** — **DEC-0105**

### Description

Add migration `015_bug0017_ai_audit_forecast.sql` extending `006_ai_audit.sql` CHECK constraints:

| Column | Additions |
|--------|-----------|
| `tool_name` | `forecast_bucket_assignment` |
| `result_status` | `low_confidence`, `provider_unavailable`, `parse_error` |

Pattern: `DROP CONSTRAINT` → `ADD CONSTRAINT … NOT VALID` → `VALIDATE CONSTRAINT`.

**Files:** `backend/migrations/015_bug0017_ai_audit_forecast.sql`

### Done when

- [ ] Both CHECK constraints extended per DEC-0105
- [ ] `sqlx migrate` / `cargo test` migration ordering PASS
- [ ] No Rust insert-path changes required

---

## BA1 — `paired_baseline_id` ON DELETE CASCADE migration

**Status:** open  
**Depends on:** —  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0017 **BA** — **DEC-0106**

### Description

Add migration (separate file or combined with AY1) altering `forecast_computations_paired_baseline_id_fkey`:

```sql
ALTER TABLE forecast_computations
  DROP CONSTRAINT forecast_computations_paired_baseline_id_fkey,
  ADD CONSTRAINT forecast_computations_paired_baseline_id_fkey
    FOREIGN KEY (paired_baseline_id) REFERENCES forecast_computations(id)
    ON DELETE CASCADE NOT VALID;
ALTER TABLE forecast_computations VALIDATE CONSTRAINT forecast_computations_paired_baseline_id_fkey;
```

**Files:** `backend/migrations/015_bug0017_forecast_fk_cascade.sql` (or combined `015_bug0017_*.sql`)

### Done when

- [ ] FK uses `ON DELETE CASCADE`
- [ ] `NOT VALID` + `VALIDATE` pattern applied
- [ ] Migration runs clean on fresh and existing DB

---

## BA2 — Retention loop order

**Status:** open  
**Depends on:** BA1  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0017 **BA** — **DEC-0106**

### Description

Change `enforce_retention()` in `forecast/repository.rs` loop order from `["baseline", "ml_enhanced"]` to `["ml_enhanced", "baseline"]` — defense in depth with CASCADE.

**Files:** `backend/src/forecast/repository.rs`

### Done when

- [ ] Loop processes `ml_enhanced` before `baseline`
- [ ] `cargo test --lib` forecast retention paths PASS

---

## BD1 — ForecastPage `isFetched` empty guard

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0017 **BD**

### Description

Replace `emptyState = !hasForecast` with architecture contract:

```tsx
const showLoading = metaQuery.isPending;
const showEmpty =
  metaQuery.isFetched && !metaQuery.isError && !metaQuery.data?.computation_id;
```

Show skeleton when `showLoading`; empty card when `showEmpty`; preserve chart content when `computation_id` present.

**Files:** `frontend/src/pages/ForecastPage.tsx`

### Done when

- [ ] Pending meta query shows loading skeleton, not empty card
- [ ] Error state does not show false empty
- [ ] `npm test` PASS

---

## T1 — Retention integration test

**Status:** open  
**Depends on:** BA1, BA2  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0017 **BA**

### Description

Add integration or repository test proving paired baseline+`ml_enhanced` rows prune without FK violation when retention exceeds `retention_count`. Seed baseline + ML pair referencing `paired_baseline_id`; invoke `enforce_retention()`; assert no error and stale rows removed.

**Files:** `backend/tests/` or `backend/src/forecast/repository.rs` `#[cfg(test)]`

### Done when

- [ ] Test seeds paired rows and prunes without FK error
- [ ] `cargo test` retention test PASS

---

## V1 — verify-work operator smoke

**Status:** open  
**Depends on:** AY1, BA1, BA2, BD1, T1 + deploy  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0017 **AY**–**BD**

### Description

Prepare `sprints/quick/Q0025/uat.md` smoke checklist. After **BACKEND_FRONTEND_DEPLOY** + Full sync + recompute:

1. `POST /api/v1/sync/trigger` — logs free of audit CHECK WARN and FK WARN
2. `GET /api/v1/forecast/meta` — fresh `computation_id`, `stale=false`
3. `SELECT … FROM ai_tool_audit WHERE tool_name = 'forecast_bucket_assignment'` — rows present
4. **BB** month-bucket SQL probe per R-0087; ML meta honest
5. Planning Compare — no **Plan stale** after successful recompute (**BC**)
6. Forecast nav from Home — no false empty when meta has `computation_id` (**BD**)

### Done when

- [ ] Rows **AY**–**BD** probed per acceptance.md operator matrix
- [ ] `uat.md` and `uat.json` populated with results
- [ ] Operator gates documented: **BACKEND_FRONTEND_DEPLOY**, **FULL_FIREFLY_SYNC**

**Operator gates:** **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** before runtime probes.
