# Sprint S0018

**ID:** S0018  
**Story:** US-0019 — Goal-driven planning with per-plan stats & AI savings suggestions  
**Status:** PLANNED  
**Created:** 2026-06-09  
**Orchestrator:** `auto-20260608-us0019-001`

## Goal

Deliver **DEC-0091** goal_balance schema + create flow; **DEC-0092** per-plan goal-stats API and UI strip; **DEC-0093** category overlay cap on remove_outflow; **DEC-0095** goal account projection; **DEC-0094** deterministic category savings suggestions with modal apply; **DEC-0096** PVA household scope unchanged; **DEC-0097** REST-primary savings path (optional AI tool P2); publish `docs/user-guides/US-0019.md`; OIDC smoke template in UAT.

## Scope

| Slice | Tasks | Primary files |
|-------|-------|---------------|
| **US-0019-S1** — Goal schema + create | T-0186 … T-0187 | `migrations/`, `plan/types.rs`, `api/plans.rs`, `PlanningPage.tsx` |
| **US-0019-S2** — Goal-stats API + strip | T-0188 … T-0189 | `plan/service.rs`, `api/plans.rs`, `components/plan/` |
| **US-0019-S3** — Category overlay + account scope | T-0190 … T-0191 | `plan/overlay.rs`, `plan/project.rs` |
| **US-0019-S4** — Savings suggestions + apply | T-0192 … T-0193 | `plan/savings_service.rs`, `PlanningPage.tsx` |
| **US-0019-S5** — Docs + regression | T-0194 … T-0195 | `docs/user-guides/US-0019.md`, `PlanningPage` tests |
| **US-0019-S6** — Optional AI tool | T-0196 | `ai/tools/` |
| **V1** — UAT smoke | T-0197 | `uat.md`, `uat.json` |

**Out of scope:** Per-plan PVA endpoint; PMT/interest feasibility; LLM savings ranking; category-scoped forecast re-projection; Grafana changes; auto-apply savings lines; Firefly write-back.

## Task table

| ID | Title | Slice | Est. | Acceptance |
|----|-------|-------|------|------------|
| T-0186 | Migration `goal_balance` + plan columns | S1 | 3h | AC-1 |
| T-0187 | Create API + Goal balance template card | S1 | 4h | AC-1 |
| T-0188 | goal-stats service + target-date SQL | S2 | 4h | AC-2 |
| T-0189 | goal-stats route + GoalStatsStrip UI | S2 | 4h | AC-2 |
| T-0190 | Category `remove_outflow` cap (3-mo avg) | S3 | 4h | AC-3 |
| T-0191 | `goal_account_id` projection fork | S3 | 3h | AC-3 |
| T-0192 | category-savings-suggestions service + route | S4 | 4h | AC-4, AC-5 |
| T-0193 | CategorySavingsModal + batch apply + audit | S4 | 4h | AC-4, AC-5 |
| T-0194 | User guide US-0019 | S5 | 2h | — |
| T-0195 | US-0014 + DEC-0089 regression tests | S5 | 2h | AC-6 |
| T-0196 | Optional `get_category_savings` AI tool | S6 | 2h | AC-5 |
| T-0197 | UAT OIDC smoke + AC-1..AC-6 template | V1 | 2h | AC-6 |

**Total estimate:** ~38h across 12 tasks (T-0196 P2 optional).

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| `target_date` beyond 730d horizon | `beyond_horizon` flag + UI copy | DEC-0092, T-0189 |
| Category overlay over-removal | Cap at 3-mo mirror avg outflow | DEC-0093, T-0190 |
| Fixed costs in savings list | DEC-0007 bucket filter excludes fixed | DEC-0094, T-0192 |
| Goal account vs compare mismatch | Document in strip + user guide | DEC-0095, T-0194 |
| Compare vs PVA confusion | PVA unchanged; contextual help | DEC-0096, T-0195 |
| Enum migration ordering | Dedicated migration; CI migration test | T-0186 |
| US-0014 regression | Template grid + empty-plan flows in R1 | T-0195 |
| DEC-0089 compare widget | Actuals-only CategoryTrendChart unchanged | T-0195 |
| AC-6 operator gate | OIDC smoke pass-with-prerequisites **BACKEND_FRONTEND_DEPLOY** | T-0197 |

## Definition of Done

- All 11 mandatory sprint tasks complete (`T-0186` … `T-0195`, `T-0197`; T-0196 conditional P2)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` § US-0019 (AC-1..AC-6)
- Goal plan create persists `target_balance_eur`, `target_date`, optional `goal_account_id` (AC-1)
- `GET …/goal-stats` returns per-plan monthly delta, yearly rollup, projected balance at target (AC-2)
- Category `remove_outflow` adjustments affect recompute with cap; `add_outflow` household-labeled (AC-3)
- Savings modal proposes ranked categories; operator checkbox apply only — no auto-apply (AC-4)
- Aggregate-only savings path; audit log on adjustment create (AC-5)
- US-0014 onboarding/templates + OIDC smoke template (AC-6)
- `docs/user-guides/US-0019.md` published (`USER_GUIDE_MODE=1`)

## Architecture references

- `docs/engineering/architecture.md` § US-0019
- `decisions/DEC-0091.md` through `DEC-0097.md`
- Research: R-0084, R-0080; frozen DEC-0087, DEC-0089, DEC-0007, DEC-0032
- Spec-pack: `docs/engineering/spec-pack/US-0019-{design-concept,crs,technical-specification}.md`
- User guide: `docs/user-guides/US-0019.md`
- Discovery: `handoffs/po_to_tl.md#discovery-20260609-us0019`
- Acceptance: `docs/product/acceptance.md` § US-0019
- Handoff: `handoffs/tl_to_dev.md` (`sprint-plan-20260609-s0018-us0019`)

## Sequencing (frozen)

```text
S1: T-0186 → T-0187
S2: T-0188 → T-0189 (after T-0186; UI after T-0187)
S3: T-0190 → T-0191 (after T-0186; ∥ S2 after migration)
S4: T-0192 → T-0193 (after T-0190 + T-0187)
S5: T-0194 → T-0195 (after S2–S4)
S6: T-0196 optional after T-0192
V1: T-0197 after T-0195
Operator: BACKEND_FRONTEND_DEPLOY → verify-work omniflow smoke (UAT)
```

## Acceptance coverage map

| Row | Tasks | Notes |
|-----|-------|-------|
| AC-1 | T-0186, T-0187, T-0197 | Goal balance template; target fields persist; Scenarios list |
| AC-2 | T-0188, T-0189, T-0197 | Per-plan stats strip; not household aggregates on detail |
| AC-3 | T-0190, T-0191, T-0197 | Category overlay affects compare/PVA after recompute |
| AC-4 | T-0192, T-0193, T-0197 | Deterministic suggestions; operator select to apply |
| AC-5 | T-0192, T-0193, T-0196, T-0197 | Aggregates only; audit log; optional AI tool wraps same service |
| AC-6 | T-0194, T-0195, T-0197 | US-0014 templates; OIDC smoke; read-only Firefly |

## Split decision

- **Why 12 tasks:** Architecture G1–G2 + S1–S2 + O1–O2 + A1–A2 + D1 + R1 + V1 + optional T1 = 12 = `SPRINT_MAX_TASKS` 12.
- **Why not split S0018a/b:** Single vertical slice; overlay (S3) gates savings (S4); stats (S2) parallelizes after schema.
- **P2 optional:** T-0196 (`get_category_savings` tool) does not block MVP — REST primary per DEC-0097.
- **User guide in D1:** `USER_GUIDE_MODE=1` — separate task avoids bundling with V1.

## Next phase

`/plan-verify` in fresh subagent/chat (role: qa)
