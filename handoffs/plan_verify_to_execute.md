# Plan-verify handoff — BUG-0015 / Q0023

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-07  
**Bug:** BUG-0015  
**Sprint:** Q0023 (`/quick`)  
**Orchestrator:** `auto-20260607-bug0015-001`  
**Verdict:** **PASS**

## Summary

Plan-verify re-run for **Q0023** / **BUG-0015** against `docs/product/acceptance.md` rows **AU**–**AW**, `docs/engineering/architecture.md` § BUG-0015, and **DEC-0084** / **DEC-0085** / **DEC-0086**. Sprint-plan artifacts are materialized under `sprints/quick/Q0023/`. Acceptance **AU**–**AW** trace to **AU1**–**AU4** + **V1** with decision alignment verified. Prior FAIL (race before sprint-plan) superseded. **Execute approved.**

## Verdict

| Check | Result |
|-------|--------|
| Sprint artifacts materialized | **PASS** — sprint.json, task.json, tasks.md, sprint.md, uat.md, uat.json |
| Acceptance AU–AW mapped to sprint tasks | **PASS** — 3/3 rows verified |
| DEC-0084 / DEC-0085 / DEC-0086 in sprint scope | **PASS** — AU1→DEC-0084; AU2–AU4→DEC-0085/0086 |
| V1 e2e verify-work smoke documented | **PASS** — uat.md + uat.json template with operator checklist |
| Operator gates in sprint artifacts | **PASS** — 3 gates in sprint.json, task.json, uat.md |
| Frozen boundaries (no scope creep) | **PASS** |
| Dependency graph valid | **PASS** — AU1→AU2→(AU3∥AU4)→V1 |
| **Execute ready** | **YES** |

## Coverage matrix

### Acceptance rows

| Row | Primary tasks | Decisions | Operator gates | Covered |
|-----|---------------|-----------|----------------|---------|
| **AU** | AU1, AU2, AU3, V1 | DEC-0084, DEC-0085, DEC-0086 | BACKEND_FRONTEND_DEPLOY, POSTGRES_PERSISTENCE_PROBE, FULL_FIREFLY_SYNC | **Yes** |
| **AV** | AU1, AU2, AU3, AU4, V1 | DEC-0084, DEC-0085, DEC-0086 | FULL_FIREFLY_SYNC | **Yes** |
| **AW** | AU3, V1 | DEC-0085 | FULL_FIREFLY_SYNC | **Yes** |

**Verified:** 3/3 acceptance rows · **5/5 tasks** traced · **0 gaps** · **0 orphans**

### Task inventory

| Task | AC rows | Decision | Surface | Priority |
|------|---------|----------|---------|----------|
| **AU1** | AU, AV | DEC-0084 | `recurrence/normalize.rs` + tests | P0 |
| **AU2** | AU, AV | DEC-0085, DEC-0086 | `subscriptions/repository.rs` + index migration | P0 (after AU1) |
| **AU3** | AU, AV, AW | DEC-0085, DEC-0086 | `subscriptions/detection.rs`, `service.rs` | P0 (after AU2) |
| **AU4** | AV | DEC-0085 | `detection.rs` stale map + `service.rs` wire | P0 (after AU2; ∥ AU3) |
| **V1** | AU–AW | — | verify-work rebuild smoke | P0 (after AU1–AU4 + deploy + gates) |

## Resolved gaps (prior FAIL)

| ID | Was | Resolution |
|----|-----|------------|
| **GAP-1** | Sprint-plan not completed | Artifacts at `sprints/quick/Q0023/` |
| **GAP-2** | No acceptance_mapping | AU/AV/AW in sprint.json + task.json + tasks.md |
| **GAP-3** | No tl_to_dev handoff | `sprint-plan-20260607-q0023-bug0015` in tl_to_dev.md |

## Operator gates (before V1)

1. **BACKEND_FRONTEND_DEPLOY** — ship Q0023 backend bundle on omniflow
2. **POSTGRES_PERSISTENCE_PROBE** — H2 SQL on `subscription_patterns` after rebuild, **before** Full sync
3. **FULL_FIREFLY_SYNC** — Full sync + subscription detection phase

## Evidence

- `sprints/quick/Q0023/plan-verify.json`
- `sprints/quick/Q0023/{sprint.json,task.json,tasks.md,uat.md,uat.json}`
- `handoffs/tl_to_dev.md` (`sprint-plan-20260607-q0023-bug0015`)
- `docs/product/acceptance.md` BUG-0015 (AU–AW)
- `docs/engineering/architecture.md` § BUG-0015
- `decisions/DEC-0084.md`, `DEC-0085.md`, `DEC-0086.md`

## Next phase

**`/execute`** — AU1 → AU2 → (AU3 ∥ AU4) → single backend release → operator gates → V1 verify-work template population at `/verify-work`.

No `handoffs/qa_to_dev.md` — no blocking defects.
