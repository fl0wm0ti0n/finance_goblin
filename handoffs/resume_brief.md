# Resume Brief

## Current status

- **Phase:** idle — next defect queue item
- **Last closed bug:** BUG-0007 — AI merchant/category discovery fails despite mirror data
- **Bug status:** BUG-0007 **DONE** (Q0017 released 2026-06-07)
- **Last quick sprint:** **Q0017** (DEC-0069 A′+E+F + S privacy fix)
- **Orchestrator run:** none (segment complete)
- **Last completed phase:** refresh-context — triad rollover post-release

## Sprint verdict

**Q0017 released** — AI Chat enumerates named subscription merchants; category_search resolves Strom/Amazon keywords; multi-tool fusion without user-supplied merchant names. T partial advisory (`group_by: month` gap) documented non-blocking.

## Open queue

| Kind | ID | Priority | Summary |
|------|-----|----------|---------|
| Bug | BUG-0008 | P1 | Subscription alerts vs list mismatch & under-detection |
| Bug | BUG-0011 | P1 | Planning mode broken — intake only |
| Epic | US-0013 | P0 | ML hardening / production overlay |
| Epic | US-0014 | P2 | Planning UX holistic improvements |
| Epic | US-0015 | P2 | AI bucket mapping |

## Recommended next auto target

**`/auto bug-target=BUG-0008`**

## Intended resume phase

**discovery** (PO) on BUG-0008 — alert count semantics vs list filters; detection recall gaps

## Resolution metadata

- `resolution_source`: resume_brief
- `resolved_start_phase`: discovery
- `segment_work_item_kind`: bug
- `active_bug_id`: BUG-0008
- `orchestrator_run_id`: _(pending materialization)_
- `intake_status`: complete — `intake-20260605-subscription-alerts-detection`
- `sprint_evidence`: handoffs/intake_evidence/intake-20260605-subscription-alerts-detection.json, docs/product/backlog.md#BUG-0008
- `recommended_next_auto`: bug-target=BUG-0008 phase=discovery
- `blocking_issue`: none
- `coordinate`: BUG-0007 DONE — additive subscription JSON only; do not regress alert/list semantics (R-0065 §4)

## BUG-0008 context (from intake)

- **W:** 33 subscription alerts vs 11 list rows — count mismatch
- **X:** Under-detection — operator expects higher recall from 922+ txs
- **Acceptance:** rows W/X in `docs/product/acceptance.md`
- **Out of scope:** merge with BUG-0007 (AI chat vs UI alert surface)

**Next artifact:** `/auto bug-target=BUG-0008` or `/discovery`
