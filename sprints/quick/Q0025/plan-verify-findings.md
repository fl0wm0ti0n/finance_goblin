# Plan-verify findings — Q0025 / BUG-0017

**Date:** 2026-06-10  
**Role:** QA (`/plan-verify`)  
**Orchestrator:** `intake-20260609-ui-audit`  
**Verdict:** **PASS** (APPROVED for `/execute`)

## Test plan (plan-verify scope)

| Check | Source | Pass criteria |
|-------|--------|---------------|
| Acceptance rows AY–BD | `acceptance.md` BUG-0017 | Each row maps to ≥1 task with verify steps |
| Task completeness | `tasks.md`, `task.json` | 6 tasks; no orphans; dependencies acyclic |
| Architecture alignment | `architecture-pack-20260609.md` § BUG-0017 | AY1, BA1, BA2, BD1, T1, V1 match frozen contracts |
| Decision alignment | `DEC-0105.md`, `DEC-0106.md` | Audit CHECK + FK CASCADE + retention order |
| Sprint artifacts | `sprints/quick/Q0025/` | sprint.json, task.json, tasks.md, sprint.md, uat.md, uat.json present |
| Frozen boundaries | `task.json` | No sync-fail-on-recompute, plan-engine, or threshold scope creep |
| UAT readiness | V1 task spec | uat.md + uat.json placeholder with AY–BD traceability |
| Runtime baseline | pre-execute repo | `cargo test --lib` 213/213; `npm test --run` 9/9 |

## Findings

### Coverage matrix

| Row | Criterion (summary) | Tasks | Covered |
|-----|---------------------|-------|---------|
| **(AY)** | Post-sync `ai_tool_audit` rows for `forecast_bucket_assignment` without tool_name CHECK violation | AY1, V1 | Yes |
| **(AZ)** | `low_confidence` and extended statuses persist without result_status CHECK violation | AY1, V1 | Yes |
| **(BA)** | Recompute succeeds without FK error; meta reflects fresh `computation_id` | BA1, BA2, T1, V1 | Yes |
| **(BB)** | ML-enhanced selectable when gate passes; otherwise honest `ml_skipped_reason` | V1 | Yes |
| **(BC)** | Planning Compare loses **Plan stale** after successful recompute | V1 | Yes |
| **(BD)** | No false **No forecast data yet** when meta has `computation_id` | BD1, V1 | Yes |

### Task → acceptance map

| Task | Acceptance hooks | Decision slice |
|------|------------------|----------------|
| AY1 | **(AY)**, **(AZ)** | DEC-0105: extend `tool_name` + `result_status` CHECK; NOT VALID + VALIDATE |
| BA1 | **(BA)** | DEC-0106: `paired_baseline_id` ON DELETE CASCADE |
| BA2 | **(BA)** | DEC-0106: `enforce_retention` ml_enhanced before baseline |
| BD1 | **(BD)** | `showEmpty = isFetched && !isError && !computation_id` |
| T1 | **(BA)** | Paired baseline+ML prune integration test |
| V1 | **(AY)**–**(BD)** | verify-work: sync logs, meta, month-bucket SQL, planning, forecast nav |

### Dependency review

- **Order:** AY1 → BA1 → BA2 ∥ BD1 → T1 → release → **BACKEND_FRONTEND_DEPLOY** → **FULL_FIREFLY_SYNC** → V1
- **Circular deps:** none
- **Operator gates:** **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** before V1 runtime probes

### DEC-0105 / DEC-0106 contract checklist

| Contract element | Sprint task | Status |
|------------------|-------------|--------|
| DROP+ADD both `ai_tool_audit` CHECK constraints | AY1 | Mapped |
| `forecast_bucket_assignment` in tool_name CHECK | AY1 | Mapped |
| Extended result_status values | AY1 | Mapped |
| No Rust status mapping | frozen_boundaries | Respected |
| `ON DELETE CASCADE` on `paired_baseline_id` | BA1 | Mapped |
| ml_enhanced before baseline retention loop | BA2 | Mapped |
| Paired retention integration test | T1 | Mapped |
| ForecastPage isFetched guard | BD1 | Mapped |
| Month-bucket SQL probe (BB) | V1 | Mapped |
| Planning Compare re-smoke (BC) | V1 | Mapped |
| Sync + meta operator smoke | V1 | Mapped |

### Gaps

**0 gaps** — all six acceptance rows AY–BD have primary task coverage with executable verify steps aligned to DEC-0105, DEC-0106, and architecture § BUG-0017.

### Orphans

**0 orphans** — all six tasks reference BUG-0017 acceptance hooks.

### Advisories (non-blocking)

1. **ADV-1:** BB and BC are ops-only V1 probes — no dedicated code tasks; intentional per architecture.
2. **ADV-2:** AY1/BA1 may be separate or combined `015_*` migration files — preserve sqlx ordering.
3. **ADV-3:** `uat.md` / `uat.json` are PLACEHOLDER — populate at verify-work.
4. **ADV-4:** Pre-ship `\d ai_tool_audit` / `\d forecast_computations` on operator DB if constraint names drift.
5. **ADV-5:** OIDC-enabled deploy regression footer — V1 omniflow smoke; full OIDC session operator-only.
6. **ADV-6:** Sync fail-on-recompute unchanged per R-0050 — deferred product gate.

## Recommendation

**APPROVED** — sprint ready for **`/execute`**. No `handoffs/qa_to_dev.md` fix list required.

**Next phase:** `/execute` (role: dev)
