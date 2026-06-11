# PO to TL archive pack (2026-06-09)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 13
- First archived heading: `## architecture-20260610-bug0018 — BUG-0018 alert eval SQL architecture (hot pointer)`
- Last archived heading: `## architecture-20260610-bug0018 — BUG-0018 alert eval SQL architecture (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=37
  - retained_body_lines=500

---

## architecture-20260610-bug0018 — BUG-0018 alert eval SQL architecture (hot pointer)

**From:** Tech Lead  
**To:** Sprint-plan  
**Date:** 2026-06-10  
**Bug:** BUG-0018  
**Orchestrator run:** `intake-20260609-ui-audit`  
**Next phase:** `/sprint-plan`

### Summary

[R-0088](docs/engineering/research.md#r-0088--bug-0018-evaluate_scarcity-ambiguous-balance--alert-eval-pipeline-abort) 6 gates resolved — **DEC-0107** (`fbd.balance` + `fbd.ts` qualification in `evaluate_scarcity`); R-0024 warn-only preserved; BF wealth-primary + subscription regression gate; no migration; no frontend change.

### Decisions

| ID | Sub | Contract |
|----|-----|----------|
| **DEC-0107** | BE | `SUM(fbd.balance::float8)`, `fbd.ts::date`, `GROUP BY fbd.ts::date`; forbid `a.balance` |

### Execute scope (P0)

| Task | Surface | Gate |
|------|---------|------|
| **BE1** | `alerts/evaluate.rs` | DEC-0107 |
| **T1** | `wealth_alerts_integration.rs` | DATABASE_URL |
| **V1** | verify-work BE/BF | BACKEND_FRONTEND_DEPLOY + FULL_FIREFLY_SYNC |

**Ops-only:** subscription dedup regression probe in V1.

**Evidence:** [R-0088](docs/engineering/research.md#r-0088--bug-0018-evaluate_scarcity-ambiguous-balance--alert-eval-pipeline-abort), `docs/engineering/architecture.md` § BUG-0018, `handoffs/archive/po-to-tl-pack-20260609-j.md`

`triad_hot_surface`: architecture hot; --rollover units=6,1,1 + --check PASS (2026-06-10T00:00:00Z)

**Recommended sprint:** `/quick` **Q0026** (BE1 + T1 + V1; ≤3 tasks)

---

