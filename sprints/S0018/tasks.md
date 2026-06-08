# Tasks — Sprint S0018

**Story:** US-0019  
**Task count:** 12 (at SPRINT_MAX_TASKS=12)

| ID | Title | Status | Slice | Est. | Acceptance refs |
|----|-------|--------|-------|------|-----------------|
| T-0186 | Migration `goal_balance` + plan columns | done | S1 | 3h | AC-1 |
| T-0187 | Create API + Goal balance template card | done | S1 | 4h | AC-1 |
| T-0188 | goal-stats service + target-date SQL | done | S2 | 4h | AC-2 |
| T-0189 | goal-stats route + GoalStatsStrip UI | done | S2 | 4h | AC-2 |
| T-0190 | Category `remove_outflow` cap (3-mo avg) | done | S3 | 4h | AC-3 |
| T-0191 | `goal_account_id` projection fork | done | S3 | 3h | AC-3 |
| T-0192 | category-savings-suggestions service + route | done | S4 | 4h | AC-4, AC-5 |
| T-0193 | CategorySavingsModal + batch apply + audit | done | S4 | 4h | AC-4, AC-5 |
| T-0194 | User guide US-0019 | done | S5 | 2h | — |
| T-0195 | US-0014 + DEC-0089 regression tests | done | S5 | 2h | AC-6 |
| T-0196 | Optional `get_category_savings` AI tool | done | S6 | 2h | AC-5 |
| T-0197 | UAT OIDC smoke + AC-1..AC-6 template | done | V1 | 2h | AC-6 |

---

## T-0186 — Migration `goal_balance` + plan columns

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0091  
**Architecture slice:** US-0019-S1  
**Research:** R-0084

### Description

Add dedicated migration:

| Change | Contract |
|--------|----------|
| Enum | `ALTER TYPE plan_template ADD VALUE 'goal_balance'` |
| `target_balance_eur` | `NUMERIC(18,2) NULL` — required when `template='goal_balance'` |
| `target_date` | `DATE NULL` — required; ≥ today on create |
| `goal_account_id` | `TEXT NULL` — optional per DEC-0095 |

Update `backend/src/plan/types.rs` + `repository.rs` for new fields. Validation helpers for goal template.

### Done when

- [ ] Migration applies cleanly on fresh and existing DBs
- [ ] Rust types expose goal fields on plan model
- [ ] `cargo test` migration/types tests green

---

## T-0187 — Create API + Goal balance template card

**Status:** open  
**Depends on:** T-0186  
**Decisions:** DEC-0091, DEC-0095  
**Architecture slice:** US-0019-S1

### Description

**API** — extend `POST /api/v1/plans`:

```json
{
  "name": "Emergency fund",
  "template": "goal_balance",
  "target_balance_eur": "10000.00",
  "target_date": "2026-11-01",
  "goal_account_id": "114"
}
```

422 when goal fields missing or `target_date` < today. Default `goal_account_id` to max-balance asset when omitted (DEC-0095).

**UI** — `PlanningPage.tsx` Scenarios: **Goal balance** template card with fields; plan appears in Scenarios list after create (AC-1). Extend `templates.rs` goal preset.

### Done when

- [ ] Goal plan creates via API with persisted metadata
- [ ] Template card visible in US-0014 template grid context
- [ ] Invalid payloads return 422 with actionable errors
- [ ] `cargo test` plans API tests green

---

## T-0188 — goal-stats service + target-date SQL

**Status:** open  
**Depends on:** T-0186  
**Decisions:** DEC-0092  
**Architecture slice:** US-0019-S2  
**Research:** R-0084

### Description

Implement `plan/service.rs` goal-stats computation:

| Field | Source |
|-------|--------|
| `monthly_delta_vs_baseline` | Compare delta for selected plan version |
| `yearly_rollup` | Calendar year `SUM(planned_net)` from `plan_daily_cashflow` |
| `projected_balance_at_target` | Last `planned_balance` where `ts::date <= target_date` |
| `gap_eur`, `required_monthly_savings_eur`, `on_track` | Feasibility copy (0% interest; no auto-lines) |
| `beyond_horizon` | `true` when `target_date > today + 730d` |

404 path when `template != 'goal_balance'`.

### Done when

- [ ] Service returns frozen JSON shape from architecture
- [ ] Target-date SQL matches DEC-0092 (no interpolation)
- [ ] `beyond_horizon` guard tested
- [ ] `cargo test` goal-stats unit tests green

---

## T-0189 — goal-stats route + GoalStatsStrip UI

**Status:** open  
**Depends on:** T-0188, T-0187  
**Decisions:** DEC-0092, DEC-0096  
**Architecture slice:** US-0019-S2

### Description

**API** — `GET /api/v1/plans/{plan_id}/goal-stats?version_id={optional}`

**UI** — `GoalStatsStrip` component:

- **Scenarios:** below plan summary when goal template selected
- **Compare:** above version table for **selected plan only** — not mixed across plans
- **PVA:** unchanged (DEC-0096)

Extend `frontend/src/lib/api.ts` with goal-stats fetcher + types.

### Done when

- [ ] Route registered; 404 for non-goal plans
- [ ] Strip renders monthly delta, yearly rollup, projected balance at target
- [ ] Feasibility copy (gap + required monthly) visible when off-track
- [ ] `npm test` component smoke passes if applicable

---

## T-0190 — Category `remove_outflow` cap (3-mo avg)

**Status:** open  
**Depends on:** T-0186  
**Decisions:** DEC-0093, DEC-0087  
**Architecture slice:** US-0019-S3

### Description

Extend `backend/src/plan/overlay.rs`:

```
effective_remove = min(adj.amount, avg_outflow_last_3_calendar_months)
```

Source: `expense_series_by_month` (DEC-0087). Empty history → **0** overlay + line warning.

`add_outflow` + `category`: full amount on household schedule; category id stored for display — no cap (DEC-0093).

Unit tests: cap applied, zero mirror, over-request clamped.

### Done when

- [ ] Category remove_outflow affects recompute after cap
- [ ] add_outflow category path household-labeled
- [ ] `cargo test` overlay category tests green

---

## T-0191 — `goal_account_id` projection fork

**Status:** open  
**Depends on:** T-0186, T-0190  
**Decisions:** DEC-0095  
**Architecture slice:** US-0019-S3

### Description

Fork `plan/project.rs` (and forecast baseline read if needed):

- When `goal_account_id` set → per-account daily series for goal plan recompute
- When NULL on create → default max-balance asset (wired in T-0187)
- Household fallback + banner when account invalid

### Done when

- [ ] Goal plan recompute scoped to goal account when set
- [ ] Default account selection tested
- [ ] Compare/PVA reflect overlay after recompute (AC-3)
- [ ] `cargo test` projection tests green

---

## T-0192 — category-savings-suggestions service + route

**Status:** open  
**Depends on:** T-0190  
**Decisions:** DEC-0094, DEC-0007, DEC-0032  
**Architecture slice:** US-0019-S4

### Description

`GET /api/v1/plans/{plan_id}/category-savings-suggestions?months=6&limit=10`

| Filter | Rule |
|--------|------|
| Min spend | ≥ €20/mo average |
| Already in plan | Skip existing `target_type=category` + `remove_outflow` |
| Fixed bucket | Exclude via DEC-0007 `resolve_bucket` |
| Ranking | `total_outflow` DESC deterministic |
| Reduction hint | 50% of avg monthly outflow |

Response: aggregate/category signals only — no raw transaction rows (AC-5).

### Done when

- [ ] Route returns ranked suggestions with evidence summary
- [ ] Fixed-bucket categories excluded
- [ ] `cargo test` savings ranking tests green

---

## T-0193 — CategorySavingsModal + batch apply + audit

**Status:** open  
**Depends on:** T-0192, T-0187  
**Decisions:** DEC-0094, DEC-0097  
**Architecture slice:** US-0019-S4

### Description

`PlanningPage.tsx` — **CategorySavingsModal**:

- Checkbox select suggestions
- Batch `POST` adjustments → trigger recompute
- **No auto-apply** — operator must confirm (AC-4)
- Audit log entry per US-0006 patterns on adjustment create (AC-5)

Reuse subscription savings modal patterns where applicable.

### Done when

- [ ] Modal opens from goal plan context; lists suggestions from API
- [ ] Selected lines materialize as plan adjustments after confirm
- [ ] Audit log records adoption
- [ ] `npm test` modal flow smoke passes if applicable

---

## T-0194 — User guide US-0019

**Status:** open  
**Depends on:** T-0189, T-0193  
**Decisions:** DEC-0091, DEC-0092, DEC-0093, DEC-0094, DEC-0096  
**Architecture slice:** US-0019-S5

### Description

Publish `docs/user-guides/US-0019.md` (`USER_GUIDE_MODE=1`):

- Goal balance template workflow
- Per-plan stats vs household compare/PVA scope
- Category overlay cap semantics
- Savings modal operator-select flow
- Goal account default + `beyond_horizon` copy

### Done when

- [ ] User guide complete with operator-facing steps
- [ ] Links from README/runbook if required by doc profile

---

## T-0195 — US-0014 + DEC-0089 regression tests

**Status:** open  
**Depends on:** T-0187, T-0189, T-0193  
**Decisions:** DEC-0096, DEC-0089  
**Architecture slice:** US-0019-S5

### Description

Regression coverage for AC-6:

- US-0014 template grid + empty-plan onboarding still works
- Compare `CategoryTrendChart` remains actuals-only (DEC-0089)
- PVA guided card unchanged when no active plan
- No Firefly write-back in plan mutations

Add/extend `PlanningPage` tests and document checklist in `uat.md`.

### Done when

- [ ] Template onboarding smoke tests pass
- [ ] Compare actuals widget unchanged by goal features
- [ ] Regression checklist captured for T-0197

---

## T-0196 — Optional `get_category_savings` AI tool

**Status:** open  
**Depends on:** T-0192  
**Decisions:** DEC-0097  
**Architecture slice:** US-0019-S6  
**Priority:** P2 optional

### Description

Optional `get_category_savings` tool in `ai/tools/` wrapping same service as REST route (DEC-0097). Aggregate-only responses; no auto-apply. Does not block MVP.

### Done when

- [ ] Tool registered in six-tool registry
- [ ] Returns same ranked data as REST endpoint
- [ ] `cargo test` tool tests green

---

## T-0197 — UAT OIDC smoke + AC-1..AC-6 template

**Status:** open  
**Depends on:** T-0194, T-0195  
**Decisions:** DEC-0091..DEC-0097  
**Architecture slice:** V1

### Description

Populate `sprints/S0018/uat.md` and `uat.json`:

- OIDC US-0010 external profile smoke checklist for AC-1..AC-6
- Operator gate **BACKEND_FRONTEND_DEPLOY** documented
- Regression scope: US-0014, US-0015 forecast buckets unchanged, DEC-0089 compare widget

### Done when

- [ ] UAT template lists 6 acceptance rows with probe steps
- [ ] Operator gates documented
- [ ] `uat.json` structured for plan-verify / verify-work

---

## Sequencing

```text
1. T-0186 (migration)
2. T-0187 (create API + card)
3. T-0188 ∥ T-0190 ∥ T-0191 (after migration)
4. T-0189 (after T-0188 + T-0187)
5. T-0192 (after T-0190)
6. T-0193 (after T-0192 + T-0187)
7. T-0194 → T-0195 (after integrations)
8. T-0197 (after T-0195)
9. T-0196 optional after T-0192
```

Dependency graph:

```text
T-0186
 ├── T-0187 ──┬── T-0189
 ├── T-0188 ──┘
 ├── T-0190 ── T-0192 ── T-0193
 └── T-0191
T-0194 → T-0195 → T-0197
T-0196 (optional, after T-0192)
```

## Acceptance map

| Row | Tasks | Verify |
|-----|-------|--------|
| AC-1 | T-0186, T-0187, T-0197 | Goal plan type; target fields; Scenarios list |
| AC-2 | T-0188, T-0189, T-0197 | Per-plan stats strip; not household on detail |
| AC-3 | T-0190, T-0191, T-0197 | Category adjustments affect compare/PVA after recompute |
| AC-4 | T-0192, T-0193, T-0197 | Suggestions + operator select apply |
| AC-5 | T-0192, T-0193, T-0196, T-0197 | Aggregates only; audit log |
| AC-6 | T-0194, T-0195, T-0197 | US-0014 templates; OIDC smoke |

## Notes

- **P2 optional:** T-0196 does not block MVP — REST primary per DEC-0097.
- **PVA scope:** DEC-0096 — no per-plan PVA endpoint; household active plan unchanged.
- **Forecast buckets:** DEC-0007 / US-0015 unchanged — no `project.rs` bucket inference edits.
