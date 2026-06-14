# Plan-verify handoff — BUG-0024 / Q0033

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-13  
**Bug:** BUG-0024  
**Sprint:** Q0033 (`/quick`)  
**Orchestrator:** `auto-20260613-bug0024`  
**Verdict:** **APPROVED**

## Summary

Plan-verify for **Q0033** / **BUG-0024** against `docs/product/acceptance.md` rows **BR**, **BS**, `docs/engineering/architecture.md` § BUG-0024, **DEC-0082**, and **R-0096**. Sprint-plan artifacts materialized under `sprints/quick/Q0033/`. Both acceptance rows trace to five mandatory P0 tasks (H1, F1, T1, G1, V1) with decision alignment verified; frozen frontend-only blast radius matches architecture. **Execute approved.**

## Verdict

| Check | Result |
|-------|--------|
| Sprint artifacts materialized | **PASS** |
| Acceptance BR, BS mapped to sprint tasks | **PASS** — 2/2 rows |
| DEC-0082 in sprint scope | **PASS** |
| Architecture → tasks 1:1 | **PASS** |
| Operator gates documented | **PASS** — FRONTEND_DEPLOY before V1 |
| Frozen boundaries respected | **PASS** |
| Dependency graph valid | **PASS** |
| Runtime baseline (pre-execute) | **PASS** — npm 24/24; build PASS |
| **Execute ready** | **YES** |

## Coverage matrix

| Row | Primary tasks | Decisions / gates | Covered |
|-----|---------------|-------------------|---------|
| **BS** | H1, F1, T1, G1, V1 | GATE-COPY-1, DEC-0082 | **Yes** |
| **BR** | G1, V1 | GATE-DEPLOY-1, Q0031 regression | **Yes** |

**Verified:** 2/2 acceptance rows · **5/5 mandatory tasks** traced · **0 gaps**

## Execute order (frozen)

`H1` → `F1 ∥ T1` → `G1` → operator **FRONTEND_DEPLOY** → `V1` verify-work.

## Notes for dev (non-blocking)

1. Add `shouldShowSolePlanDeleteHint` + `SOLE_PLAN_DELETE_HINT` to `planSelector.ts` per frozen architecture contract.
2. Render inline hint **immediately below** Delete plan row in `PlanningPage.tsx` when predicate true (L667–687).
3. **Do not** change `resolveDisplayedPlanId` / `isDeleteDisabled` — Q0031 selector must remain intact.
4. Multi-plan active selection: keep existing tooltip; no inline hint when `plans.length >= 2`.
5. T1 table-driven vitest cases per tasks.md; preserve existing 8/8 planSelector cases.
6. uat.md/uat.json placeholders — populate at V1/verify-work.

## Evidence

- `sprints/quick/Q0033/plan-verify.json`
- `sprints/quick/Q0033/plan-verify-findings.md`
- `sprints/quick/Q0033/{sprint.json,task.json,tasks.md,sprint.md,uat.json,uat.md}`
- `docs/product/acceptance.md` BUG-0024 rows BR–BS
- `docs/engineering/architecture.md` § BUG-0024

## Next phase

**`/execute`** in fresh subagent/chat (role: dev).

No `handoffs/qa_to_dev.md` — no blocking defects.

`fresh_context_marker`: plan-verify-20260613-bug0024-qa-fresh  
`runtime_proof_id`: runtime-proof-plan-verify-20260613-bug0024-001  
`phase_boundary`: plan-verify → execute

---

# Plan-verify handoff — BUG-0026 / Q0032

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-13  
**Bug:** BUG-0026  
**Sprint:** Q0032 (`/quick`)  
**Orchestrator:** `auto-20260613-bug0026`  
**Verdict:** **APPROVED**

## Summary

Plan-verify for **Q0032** / **BUG-0026** against `docs/product/acceptance.md` rows **BZ**, **CA**, `docs/engineering/architecture.md` § BUG-0026, **DEC-0089**, and **R-0098**. Sprint-plan artifacts materialized under `sprints/quick/Q0032/`. Both acceptance rows trace to five mandatory P0 tasks (H1, F1, T1, G1, V1) with decision alignment verified; frozen blast radius matches architecture. **Execute approved.**

## Verdict

| Check | Result |
|-------|--------|
| Sprint artifacts materialized | **PASS** |
| Acceptance BZ, CA mapped to sprint tasks | **PASS** — 2/2 rows |
| DEC-0089 in sprint scope | **PASS** |
| Architecture → tasks 1:1 | **PASS** |
| Operator gates documented | **PASS** — FRONTEND_DEPLOY before V1 |
| Frozen boundaries respected | **PASS** |
| Dependency graph valid | **PASS** |
| Runtime baseline (pre-execute) | **PASS** — cargo lib 221/221; npm 17/17 |
| **Execute ready** | **YES** |

## Coverage matrix

| Row | Primary tasks | Decisions / gates | Covered |
|-----|---------------|-------------------|---------|
| **BZ** | H1, F1, T1, G1, V1 | GATE-MONTH-1, DEC-0089 | **Yes** |
| **CA** | H1, F1, T1, G1, V1 | GATE-LABEL-1 | **Yes** |

**Verified:** 2/2 acceptance rows · **5/5 mandatory tasks** traced · **0 gaps**

## Execute order (frozen)

`H1` → `F1 ∥ T1` → `G1` → operator **FRONTEND_DEPLOY** → `V1` verify-work.

## Notes for dev (non-blocking)

1. Replace `ForecastPage.tsx` L148–152 `series[0]` with `resolveForecastSummaryPoint`.
2. Render `formatForecastSummarySubtitle` immediately **above** `.grid` card block (L312–330).
3. **Do not** add `categoryId` to `monthlyQuery` key — DEC-0089 cards independent of category filter.
4. Leave `MonthlyChart.tsx` unchanged — full series plot.
5. T1 partialMonthTrap fixture frozen: June income 0.00 → July 3266.16.
6. uat.md/uat.json placeholders — populate at V1/verify-work.

## Evidence

- `sprints/quick/Q0032/plan-verify.json`
- `sprints/quick/Q0032/plan-verify-findings.md`
- `sprints/quick/Q0032/{sprint.json,task.json,tasks.md,sprint.md,uat.json,uat.md}`
- `docs/product/acceptance.md` BUG-0026 rows BZ–CA
- `docs/engineering/architecture.md` § BUG-0026

## Next phase

**`/execute`** in fresh subagent/chat (role: dev).

No `handoffs/qa_to_dev.md` — no blocking defects.

`fresh_context_marker`: plan-verify-20260613-bug0026-qa-fresh  
`runtime_proof_id`: runtime-proof-plan-verify-20260613-bug0026-001  
`phase_boundary`: plan-verify → execute

---

# Plan-verify handoff — BUG-0021 / Q0029

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-11  
**Bug:** BUG-0021  
**Sprint:** Q0029 (`/quick`)  
**Orchestrator:** `auto-20260611-bug0021`  
**Verdict:** **PASS**

## Summary

Plan-verify for **Q0029** / **BUG-0021** against `docs/product/acceptance.md` rows **BK**, **BL**, `docs/engineering/architecture.md` § BUG-0021, **DEC-0110**, **DEC-0111**, and **R-0091**. Sprint-plan artifacts materialized under `sprints/quick/Q0029/`. Both acceptance rows trace to seven mandatory P0 tasks (EA1, EA2, EB1, EB2, T1, G1, V1) with decision alignment verified; frozen blast radius matches architecture. **Execute approved.**

## Verdict

| Check | Result |
|-------|--------|
| Sprint artifacts materialized | **PASS** |
| Acceptance BK, BL mapped to sprint tasks | **PASS** — 2/2 rows |
| DEC-0110 / DEC-0111 in sprint scope | **PASS** |
| Architecture → tasks 1:1 | **PASS** |
| Operator gates documented | **PASS** |
| Frozen boundaries respected | **PASS** |
| Dependency graph valid | **PASS** |
| Runtime baseline (pre-execute) | **PASS** — cargo lib 213/213; npm 9/9 |
| **Execute ready** | **YES** |

## Coverage matrix

| Row | Primary tasks | Decisions | Covered |
|-----|---------------|-----------|---------|
| **BK** | EA1, EA2, T1, G1, V1 | DEC-0110 | **Yes** |
| **BL** | EB1, EB2, T1, G1, V1 | DEC-0111 | **Yes** |

**Verified:** 2/2 acceptance rows · **7/7 mandatory tasks** traced · **0 gaps**

## Execute order (frozen)

`EA1 ∥ EA2 ∥ EB1` → `EB2` (WealthPage — coordinate with EA2) → `EA3` (optional) → `T1` → `G1` → operator **BACKEND_FRONTEND_DEPLOY** → operator **SNAPSHOT_UPSERT_OR_SYNC** (optional) → `V1` verify-work.

## Notes for dev (non-blocking)

1. Remove lazy CategoryFilter + Suspense on BK surfaces only — **CategoryTrendChart** lazy boundary must remain.
2. EB1 SQL: `COALESCE(payload->'attributes'->>'account_role', payload->>'account_role')` — align test constant ~L133.
3. EB2 label map per DEC-0111 — API returns raw enum; UI maps to human labels.
4. Fix `hasForecast` TS6133 in EA1 if still unused (docker build blocker from BUG-0020 release).
5. WealthPage EA2 + EB2 same file — single commit recommended.
6. Do not touch `CategoryFilter.tsx`, `firefly/mod.rs`, `portfolio.json`, or migrations.
7. uat.md/uat.json placeholders — populate at V1/verify-work.

## Evidence

- `handoffs/plan_verify_report.md` (Q0029 section)
- `sprints/quick/Q0029/{sprint.json,task.json,tasks.md,sprint.md,uat.json,uat.md}`
- `handoffs/tl_to_dev.md` sprint-plan-20260611-q0029-bug0021
- `docs/product/acceptance.md` BUG-0021 rows BK–BL
- `docs/engineering/architecture.md` § BUG-0021
- `decisions/DEC-0110.md`, `decisions/DEC-0111.md`

## Next phase

**`/execute`** in fresh subagent/chat (role: dev).

No `handoffs/qa_to_dev.md` — no blocking defects.

`fresh_context_marker`: plan-verify-20260611-bug0021-qa-fresh  
`runtime_proof_id`: runtime-proof-plan-verify-20260611-bug0021-001  
`phase_boundary`: plan-verify → execute

---

# Plan-verify handoff — BUG-0020 / Q0028

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-11  
**Bug:** BUG-0020  
**Sprint:** Q0028 (`/quick`)  
**Orchestrator:** `auto-20260610-bug0019`  
**Verdict:** **PASS**

## Summary

Plan-verify for **Q0028** / **BUG-0020** against `docs/product/acceptance.md` rows **BI**, **BJ**, `docs/engineering/architecture.md` § BUG-0020, **DEC-0109**, and **R-0090**. Sprint-plan artifacts materialized under `sprints/quick/Q0028/`. Both acceptance rows trace to seven P0 tasks (DA1, DB1, DA2, DA3, T1, G1, V1) with decision alignment verified; frozen blast radius matches architecture. **Execute approved.**

## Verdict

| Check | Result |
|-------|--------|
| Sprint artifacts materialized | **PASS** |
| Acceptance BI, BJ mapped to sprint tasks | **PASS** — 2/2 rows |
| DEC-0109 in sprint scope | **PASS** |
| Architecture → tasks 1:1 | **PASS** |
| Operator gates documented | **PASS** |
| Frozen boundaries respected | **PASS** |
| Dependency graph valid | **PASS** |
| **Execute ready** | **YES** |

## Coverage matrix

| Row | Primary tasks | Decisions | Covered |
|-----|---------------|-----------|---------|
| **BI** | DA1, DA2, DA3, T1, G1, V1 | DEC-0109, DEC-0085, DEC-0086 | **Yes** |
| **BJ** | DB1, T1, G1, V1 | DEC-0109, DEC-0100 | **Yes** |

**Verified:** 2/2 acceptance rows · **7/7 tasks** traced · **0 gaps**

## Execute order (frozen)

`DA1 → DB1` (single migration 016 file) `∥ DA2 ∥ DA3` → `T1` → `G1` → operator **BACKEND_FRONTEND_DEPLOY** + **MIGRATION_016_APPLY** → operator **FULL_FIREFLY_SYNC** → `V1` verify-work.

## Notes for dev (non-blocking)

1. Reconcile before backfill in migration 016 — single transaction-wrapped file per DEC-0109.
2. `interval_matches` (DEC-0086) required on all merge clusters and DA3 guard paths.
3. Survivor selection frozen: YouTube = earliest `confirmed_at`; Strom pending = highest `last_seen_at`.
4. Do not touch `list_patterns` SQL shape, `compute_display_category_id` algorithm, or discover/tags API.
5. uat.md/uat.json placeholders — populate at V1/verify-work.
6. G1 blast-radius check: migration, detection.rs, SubscriptionsPage.tsx, test file only.

## Evidence

- `handoffs/plan_verify_report.md` (Q0028 section)
- `sprints/quick/Q0028/{sprint.json,task.json,tasks.md,sprint.md,uat.json,uat.md}`
- `handoffs/tl_to_dev.md` sprint-plan-20260611-q0028-bug0020
- `docs/product/acceptance.md` BUG-0020 rows BI–BJ
- `docs/engineering/architecture.md` § BUG-0020
- `decisions/DEC-0109.md`

## Next phase

**`/execute`** in fresh subagent/chat (role: dev).

No `handoffs/qa_to_dev.md` — no blocking defects.

`fresh_context_marker`: plan-verify-20260611-bug0020-qa-fresh  
`runtime_proof_id`: runtime-proof-plan-verify-20260610-bug0020-001  
`phase_boundary`: plan-verify → execute

---

# Plan-verify handoff — BUG-0019 / Q0027

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-10  
**Bug:** BUG-0019  
**Sprint:** Q0027 (`/quick`)  
**Orchestrator:** `auto-20260610-bug0019`  
**Verdict:** **PASS**

## Summary

Plan-verify for **Q0027** / **BUG-0019** against `docs/product/acceptance.md` rows **BG**, **BH**, `docs/engineering/architecture.md` § BUG-0019, **DEC-0108**, and **R-0089**. Sprint-plan artifacts materialized under `sprints/quick/Q0027/`. Both acceptance rows trace to the six P0 tasks (CA1, CA2, CA3, CB1, G1, V1) with decision alignment verified; provisioning-only blast radius confirmed. **Execute approved.**

## Verdict

| Check | Result |
|-------|--------|
| Sprint artifacts materialized | **PASS** — sprint.json, task.json, tasks.md, sprint.md, progress.md, uat.json, uat.md |
| Acceptance BG, BH mapped to sprint tasks | **PASS** — 2/2 rows verified |
| DEC-0108 in sprint scope (CA variable + subquery, CB mirror COUNT, version bumps) | **PASS** |
| Architecture CA-1/CA-2/CA-3/CB-1/static/runtime → tasks | **PASS** — 1:1 mapping |
| Operator gates documented | **PASS** — GRAFANA_PROVISIONING_RELOAD + Full sync + 0-new-tx incremental rerun before/within V1 |
| Frozen boundaries (no scope creep) | **PASS** — no `upsert_cursor`/sync semantics/`AnalyticsEmbedPage.tsx`/migration edits; no hardcoded 114; CA-B deferred |
| Dependency graph valid | **PASS** — `(CA1 → CA2) ∥ CA3 ∥ CB1 → G1 → reload → V1` acyclic |
| **Execute ready** | **YES** |

## Coverage matrix

| Row | Primary tasks | Decisions | Covered |
|-----|---------------|-----------|---------|
| **BG** | CA1, CA2, CA3, G1, V1 | DEC-0108 | **Yes** — default account 114; panels 1–2 non-zero vs `GET /api/v1/forecast/monthly?account_id=114`; kiosk embed **and** direct Grafana |
| **BH** | CB1, G1, V1 | DEC-0108 | **Yes** — panel 2 `transactions` = mirror COUNT (922) after Full sync **and** after 0-new-tx incremental rerun |

**Verified:** 2/2 acceptance rows · **6/6 tasks** traced · **0 gaps**

## Execute order (frozen)

`(CA1 → CA2) ∥ CA3 ∥ CB1` → `G1` static guard PASS → operator **GRAFANA_PROVISIONING_RELOAD** (`docker compose restart grafana`) → `V1` verify-work (embed + direct Grafana; Full sync + 0-new-tx incremental rerun; OIDC omniflow re-check).

## Notes for dev (non-blocking)

1. **`current` shape** — mirror the existing `forecast_variant` `current` structure in `forecast-horizons.json` exactly (wrong shape → variable shows "None"); G1 only asserts non-null.
2. **CA2 assertion** — `grep -c` → 3 assumes one target per panel; assert per-panel if a panel has multiple targets.
3. **G1 version-bump diff** — compare vs `HEAD~` only if edits are committed; otherwise diff vs `HEAD`/working tree.
4. **Rollback** — `git revert` of the three JSON files + Grafana restart (DEC-0108); no schema/data state.
5. **Out of scope** — `ml_enhanced` stuck-`running` cleanup (PO flag, separate bug); embed-forwarded `var-account_id` (CA-B deferred); fresh-install arbitrary-default runbook note may ride along with release notes.
6. uat.md/uat.json placeholders — populate at V1/verify-work.

## Evidence

- `handoffs/plan_verify_report.md` (Q0027 section)
- `sprints/quick/Q0027/{sprint.json,task.json,tasks.md,sprint.md,uat.json,uat.md}`
- `handoffs/tl_to_dev.md` sprint-plan-20260610-q0027-bug0019
- `docs/product/acceptance.md` BUG-0019 rows BG–BH
- `docs/engineering/architecture.md` § BUG-0019
- `decisions/DEC-0108.md`
- `docs/engineering/research.md` R-0089

## Next phase

**`/execute`** in fresh subagent/chat (role: dev).

No `handoffs/qa_to_dev.md` — no blocking defects.

`fresh_context_marker`: plan-verify-20260610-bug0019-qa-fresh  
`runtime_proof_id`: runtime-proof-plan-verify-20260610-bug0019-001  
`phase_boundary`: plan-verify → execute

---

# Plan-verify handoff — BUG-0018 / Q0026

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-10  
**Bug:** BUG-0018  
**Sprint:** Q0026 (`/quick`)  
**Orchestrator:** `intake-20260609-ui-audit`  
**Verdict:** **PASS**

## Summary

Plan-verify for **Q0026** / **BUG-0018** against `docs/product/acceptance.md` rows **BE**, **BF**, `docs/engineering/architecture-archive/architecture-pack-20260609-a.md` § BUG-0018, and **DEC-0107**. Sprint-plan artifacts materialized under `sprints/quick/Q0026/`. Two acceptance rows trace to tasks BE1, T1, V1 with decision alignment verified. **Execute approved.** Supersedes prior BLOCKED plan-verify (sprint-plan race).

## Verdict

| Check | Result |
|-------|--------|
| Sprint artifacts materialized | **PASS** — sprint.json, task.json, tasks.md, sprint.md, uat.json, uat.md |
| Acceptance BE, BF mapped to sprint tasks | **PASS** — 2/2 rows verified |
| DEC-0107 in sprint scope | **PASS** |
| Architecture BE1/T1/V1 → tasks | **PASS** — 1:1 mapping |
| Operator gates documented | **PASS** — BACKEND_FRONTEND_DEPLOY + FULL_FIREFLY_SYNC before V1 |
| Frozen boundaries (no scope creep) | **PASS** — no migration, frontend, sync-fail-on-alert, or sibling evaluator changes |
| Dependency graph valid | **PASS** |
| Runtime baseline (pre-execute) | **PASS** — cargo lib 213/213; npm 9/9 |
| **Execute ready** | **YES** |

## Coverage matrix

| Row | Primary tasks | Decisions | Covered |
|-----|---------------|-----------|---------|
| **BE** | BE1, T1, V1 | DEC-0107 | **Yes** |
| **BF** | BE1, V1 | DEC-0107 | **Yes** |

**Verified:** 2/2 acceptance rows · **3/3 tasks** traced · **0 gaps**

## Operator gates (before V1 runtime smoke)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild `flow-finance-ai` with BE1 `evaluate_scarcity` SQL fix
2. **FULL_FIREFLY_SYNC** — Full Firefly sync; alerts phase must complete without SQL error

## Execute order (frozen)

BE1 → T1 → single backend release → BACKEND_FRONTEND_DEPLOY → FULL_FIREFLY_SYNC → V1 verify-work

## Advisories (non-blocking)

1. Pre-execute `evaluate_scarcity` still unqualified — BE1 is the execute delta per DEC-0107.
2. `wealth_alerts_integration` skips without `DATABASE_URL` — T1 runbook note; V1 covers live path.
3. uat.md/uat.json PLACEHOLDER — populate at verify-work.
4. New alerts after fix (budget/plan evaluators) — expected; document in release notes.
5. OIDC regression footer is operator verify-work scope (V1 OIDC-1).
6. BF subscription dedup is V1 regression gate only — separate path per BUG-0008.

## Evidence

- `sprints/quick/Q0026/plan-verify.json`
- `sprints/quick/Q0026/plan-verify-findings.md`
- `sprints/quick/Q0026/{sprint.json,task.json,tasks.md,sprint.md,uat.json,uat.md}`
- `handoffs/tl_to_dev.md` sprint-plan-20260610-q0026-bug0018
- `docs/product/acceptance.md` BUG-0018 rows BE–BF
- `docs/engineering/architecture-archive/architecture-pack-20260609-a.md` § BUG-0018
- `decisions/DEC-0107.md`

## Next phase

**`/execute`** in fresh subagent/chat (role: dev).

No `handoffs/qa_to_dev.md` — no blocking defects.

`fresh_context_marker`: plan-verify-20260610-bug0018-qa-fresh-rerun  
`runtime_proof_id`: runtime-proof-plan-verify-20260610-bug0018-002  
`phase_boundary`: plan-verify → execute

---

# Plan-verify handoff — BUG-0017 / Q0025

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-10  
**Bug:** BUG-0017  
**Sprint:** Q0025 (`/quick`)  
**Orchestrator:** `intake-20260609-ui-audit`  
**Verdict:** **PASS**

## Summary

Plan-verify for **Q0025** / **BUG-0017** against `docs/product/acceptance.md` rows **AY**–**BD**, `docs/engineering/architecture-archive/architecture-pack-20260609.md` § BUG-0017, and **DEC-0105** / **DEC-0106**. Sprint-plan artifacts materialized under `sprints/quick/Q0025/`. Six acceptance rows trace to tasks AY1, BA1, BA2, BD1, T1, V1 with decision alignment verified. **Execute approved.**

## Verdict

| Check | Result |
|-------|--------|
| Sprint artifacts materialized | **PASS** — sprint.json, task.json, tasks.md, sprint.md, uat.json, uat.md |
| Acceptance AY–BD mapped to sprint tasks | **PASS** — 6/6 rows verified |
| DEC-0105 / DEC-0106 in sprint scope | **PASS** |
| Architecture AY1/BA1/BA2/BD1/T1/V1 → tasks | **PASS** — 1:1 mapping |
| Operator gates documented | **PASS** — BACKEND_FRONTEND_DEPLOY + FULL_FIREFLY_SYNC before V1 |
| Frozen boundaries (no scope creep) | **PASS** — no sync-fail-on-recompute, plan-engine, or threshold changes |
| Dependency graph valid | **PASS** |
| Runtime baseline (pre-execute) | **PASS** — cargo lib 213/213; npm 9/9 |
| **Execute ready** | **YES** |

## Coverage matrix

| Row | Primary tasks | Decisions | Covered |
|-----|---------------|-----------|---------|
| **AY** | AY1, V1 | DEC-0105 | **Yes** |
| **AZ** | AY1, V1 | DEC-0105 | **Yes** |
| **BA** | BA1, BA2, T1, V1 | DEC-0106 | **Yes** |
| **BB** | V1 | — | **Yes** |
| **BC** | V1 | — | **Yes** |
| **BD** | BD1, V1 | — | **Yes** |

**Verified:** 6/6 acceptance rows · **6/6 tasks** traced · **0 gaps**

## Operator gates (before V1 runtime smoke)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild `flow-finance-ai` with Q0025 migrations + repository + ForecastPage changes
2. **FULL_FIREFLY_SYNC** — Full Firefly sync + forecast recompute before meta/planning/ML probes

## Execute order (frozen)

AY1 → BA1 → BA2 ∥ BD1 → T1 → single backend+frontend release → BACKEND_FRONTEND_DEPLOY → FULL_FIREFLY_SYNC → V1 verify-work

## Advisories (non-blocking)

1. BB/BC are ops-only V1 probes — no dedicated code tasks; intentional per architecture.
2. AY1/BA1 may be separate or combined `015_*` migrations — preserve sqlx ordering.
3. uat.md/uat.json PLACEHOLDER — populate at verify-work.
4. Pre-ship constraint name probe on operator DB if names drift.
5. OIDC regression footer is operator verify-work scope.

## Evidence

- `sprints/quick/Q0025/plan-verify.json`
- `sprints/quick/Q0025/plan-verify-findings.md`
- `sprints/quick/Q0025/{sprint.json,task.json,tasks.md,sprint.md,uat.json,uat.md}`
- `handoffs/tl_to_dev.md` sprint-plan-20260610-q0025-bug0017
- `docs/product/acceptance.md` BUG-0017 rows AY–BD
- `docs/engineering/architecture-archive/architecture-pack-20260609.md` § BUG-0017
- `decisions/DEC-0105.md`, `decisions/DEC-0106.md`

## Next phase

**`/execute`** in fresh subagent/chat (role: dev).

No `handoffs/qa_to_dev.md` — no blocking defects.

`fresh_context_marker`: plan-verify-20260610-bug0017-qa-fresh  
`runtime_proof_id`: runtime-proof-plan-verify-20260610-bug0017-001  
`phase_boundary`: plan-verify → execute

---

# Plan-verify handoff — BUG-0016 / Q0024

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-09  
**Bug:** BUG-0016  
**Sprint:** Q0024 (`/quick`)  
**Orchestrator:** `intake-20260609-ui-audit`  
**Verdict:** **PASS**

## Summary

Plan-verify for **Q0024** / **BUG-0016** against `docs/product/acceptance.md` row **AX**, `docs/engineering/architecture.md` § BUG-0016, and **DEC-0104**. Sprint-plan artifacts materialized under `sprints/quick/Q0024/`. Acceptance row AX traces to tasks AX1, AX2, V1 with decision alignment verified. **Execute approved.**

## Verdict

| Check | Result |
|-------|--------|
| Sprint artifacts materialized | **PASS** — sprint.json, task.json, tasks.md, sprint.md, uat.json, uat.md |
| Acceptance AX mapped to sprint tasks | **PASS** — 1/1 row verified |
| DEC-0104 in sprint scope | **PASS** |
| Architecture AX1/AX2/V1 → tasks | **PASS** — 1:1 mapping |
| Operator gate documented | **PASS** — BACKEND_FRONTEND_DEPLOY before V1 |
| Frozen boundaries (no scope creep) | **PASS** — no Traefik/frontend/callback redirect changes |
| Dependency graph valid | **PASS** |
| **Execute ready** | **YES** |

## Coverage matrix

| Row | Primary tasks | Decisions | Covered |
|-----|---------------|-----------|---------|
| **AX** | AX1, AX2, V1 | DEC-0104, DEC-0057 | **Yes** |

**Verified:** 1/1 acceptance row · **3/3 tasks** traced · **0 gaps**

## Operator gates (before V1 omniflow smoke)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild `flow-finance-ai` with AX1 SPA fallback

## Execute order (frozen)

AX1 → AX2 → single backend release → operator BACKEND_FRONTEND_DEPLOY → V1 verify-work

## Advisories (non-blocking)

1. Grafana proxy integration test may defer to V1 — operator smoke covers `/analytics/grafana/*`.
2. uat.md/uat.json PLACEHOLDER — populate at verify-work.
3. OIDC full deploy regression is operator verify-work footer — V1 includes `/callback` SPA shell probe.

## Evidence

- `sprints/quick/Q0024/plan-verify.json`
- `sprints/quick/Q0024/plan-verify-findings.md`
- `sprints/quick/Q0024/{sprint.json,task.json,tasks.md,sprint.md,uat.json,uat.md}`
- `handoffs/tl_to_dev.md` sprint-plan-20260609-q0024-bug0016
- `docs/product/acceptance.md` BUG-0016 row AX
- `docs/engineering/architecture.md` § BUG-0016
- `decisions/DEC-0104.md`

## Next phase

**`/execute`** in fresh subagent/chat (role: dev).

No `handoffs/qa_to_dev.md` — no blocking defects.

`fresh_context_marker`: plan-verify-20260609-bug0016-qa-fresh  
`runtime_proof_id`: runtime-proof-plan-verify-20260609-bug0016-001  
`phase_boundary`: plan-verify → execute

---

# Plan-verify handoff — US-0020 / S0019

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-10  
**Story:** US-0020  
**Sprint:** S0019  
**Orchestrator:** `auto-20260608-us0020-001`  
**Verdict:** **PASS**

## Summary

Plan-verify for **S0019** / **US-0020** against `docs/product/acceptance.md` AC-1..AC-6, `docs/engineering/architecture.md` § US-0020, and **DEC-0098** through **DEC-0103**. Sprint-plan artifacts materialized under `sprints/S0019/`. All six acceptance criteria trace to tasks T-0198..T-0210 with decision alignment verified. **Execute approved.**

## Verdict

| Check | Result |
|-------|--------|
| Sprint artifacts materialized | **PASS** — sprint.json, tasks.md, sprint.md, uat.json, uat.md, progress.md |
| Acceptance AC-1..AC-6 mapped to sprint tasks | **PASS** — 6/6 rows verified |
| DEC-0098 .. DEC-0103 in sprint scope | **PASS** |
| Architecture M1/D1–D2/C1–C3/T1–T3/R1–R2/G1/V1 → T-0198..T-0210 | **PASS** — 1:1 mapping |
| UAT OIDC smoke template task documented | **PASS** — T-0209 |
| Operator gates documented | **PASS** — BACKEND_FRONTEND_DEPLOY, FULL_FIREFLY_SYNC |
| Frozen boundaries (no scope creep) | **PASS** — no DetectionPipeline threshold edits; DEC-0085 merge preserved; no Firefly write-back |
| Dependency graph valid | **PASS** |
| **Execute ready** | **YES** |

## Coverage matrix

| AC | Primary tasks | Decisions | Covered |
|----|---------------|-----------|---------|
| **AC-1** | T-0199, T-0200, T-0209 | DEC-0098 | **Yes** |
| **AC-2** | T-0201, T-0209 | DEC-0099, DEC-0085 | **Yes** |
| **AC-3** | T-0202, T-0203, T-0209 | DEC-0100, DEC-0087 | **Yes** |
| **AC-4** | T-0204, T-0205, T-0206, T-0209 | DEC-0101, DEC-0102 | **Yes** |
| **AC-5** | T-0198, T-0202, T-0204, T-0209 | DEC-0100, DEC-0101 | **Yes** |
| **AC-6** | T-0207, T-0208, T-0209 | DEC-0099, DEC-0084..0086 | **Yes** |

**Verified:** 6/6 acceptance criteria · **12/12 tasks** traced · **0 gaps**

## Operator gates (before AC-1..AC-6 omniflow smoke)

1. **BACKEND_FRONTEND_DEPLOY** — ship S0019 backend + frontend on US-0010 external profile
2. **FULL_FIREFLY_SYNC** — mirror transactions + categories current for discover + majority category

## Execute order (frozen)

T-0198 → T-0199 → (T-0200 ∥ T-0201) → T-0202 → T-0203; T-0198 → T-0204 → T-0205 → T-0206; T-0207 → T-0208 → T-0209; T-0210 optional after T-0205.

## Advisories (non-blocking)

1. T-0207 user guide — no direct AC row; architecture R1 support task.
2. T-0210 Grafana `$tag` P2 optional — SPA `?tag=` sufficient per DEC-0103.
3. Align tag field naming (`name` per DEC-0101 vs `label` in tasks) and PATCH verb at execute.

## Evidence

- `sprints/S0019/plan-verify.json`
- `sprints/S0019/plan-verify-findings.md`
- `sprints/S0019/{sprint.json,tasks.md,sprint.md,uat.json,uat.md}`
- `handoffs/tl_to_dev.md` (sprint-plan pointer)
- `docs/product/acceptance.md` US-0020 (AC-1..AC-6)
- `docs/engineering/architecture.md` § US-0020
- `decisions/DEC-0098.md` through `DEC-0103.md`

## Next phase

**`/execute`** in fresh subagent/chat (role: dev).

No `handoffs/qa_to_dev.md` — no blocking defects.

`fresh_context_marker`: plan-verify-20260610-us0020-qa-fresh  
`runtime_proof_id`: runtime-proof-plan-verify-20260610-us0020-001  
`phase_boundary`: plan-verify → execute
