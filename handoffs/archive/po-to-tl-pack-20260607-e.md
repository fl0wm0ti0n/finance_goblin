# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## sprint-plan-20260610-q0022-bug0014 — BUG-0014 Q0022 quick sprint (hot pointer)`
- Last archived heading: `## sprint-plan-20260610-q0022-bug0014 — BUG-0014 Q0022 quick sprint (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=35
  - retained_body_lines=496

---

## sprint-plan-20260610-q0022-bug0014 — BUG-0014 Q0022 quick sprint (hot pointer)

**From:** Tech Lead  
**To:** Plan-verify / Execute  
**Date:** 2026-06-10  
**Bug:** BUG-0014  
**Sprint:** **Q0022** (`/quick`)  
**Orchestrator run:** `auto-20260607-bug0014-001`  
**Next phase:** `/plan-verify`

### Summary

Materialized **Q0022** — 8 tasks (5 mandatory P0 + AS2 P1 optional + AP2/AR1 conditional); 8/12 under `SPRINT_MAX_TASKS`; no split. Execute bundle: **AO1**, **AQ1**/**AQ2**, **AS1**; conditional **AP2** (AP1 gate); **AS2** P1; **AR1** P2 if API≠Grafana; **V1** verify AO–AT.

### Tasks

| ID | Acceptance | Priority | Gate |
|----|------------|----------|------|
| **AO1** | AO | P0 | — |
| **AQ1** | AQ | P0 | — |
| **AQ2** | AQ | P0 | after AQ1 |
| **AS1** | AS | P0 | — |
| **AS2** | AS | P1 optional | — |
| **AP2** | AP | P0 conditional | AP1 SQL |
| **AR1** | AR | P2 conditional | V1 AR gate |
| **V1** | AO–AT | P0 | operator gates |

### Artifacts

`sprints/quick/Q0022/{sprint.md,sprint.json,tasks.md,task.json,progress.md,uat.md,uat.json}`, `handoffs/tl_to_dev.md` (sprint-plan-20260610-q0022-bug0014)

**Prior:** `architecture-20260609-bug0014` — superseded for execute by this sprint-plan handoff.

---

