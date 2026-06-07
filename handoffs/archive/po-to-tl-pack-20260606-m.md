# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## intake-20260606-us0015 — US-0015 AI forecast bucket mapping (hot pointer)`
- Last archived heading: `## intake-20260606-us0015 — US-0015 AI forecast bucket mapping (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=54
  - retained_body_lines=490

---

## intake-20260606-us0015 — US-0015 AI forecast bucket mapping (hot pointer)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-06  
**Story:** US-0015  
**Orchestrator run:** auto-20260606-us0015-001  
**Next phase:** `/discovery`

### Summary

Re-intake formalizes the **deferred epic** from BUG-0012 intake: when Firefly categories are missing or ambiguous, AI assists income/fixed/variable bucket assignment on the **forecast projection path** so monthly decomposition reflects real spending — not config map gaps or silent Variable absorption. Parent **BUG-0012 DONE** (Q0014); AI **extends** DEC-0007 baseline, does not replace it.

### Scope (bounded)

| In | Out |
|----|-----|
| AI inference (merchant/description/category fusion) with confidence | Firefly write-back / in-app category editing |
| Projection fallback: config map → AI → Variable default | BUG-0007 chat tool registry changes |
| Monthly API `bucket_source` + UI **AI-mapped** badge | US-0013 ML overlay / new forecast models |
| Audit trail per US-0006 / DEC-0032 privacy defaults | RAG / vector index (deferred R-0074) |

### Research pointer

[R-0074](docs/engineering/research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy) — rule+LLM cascade, privacy-first feature extraction, BUG-0007 surface isolation, architecture decision gates.

### Decomposition recommendation

**Single epic** → sprint-plan slices **US-0015-S1** (inference), **S2** (projection merge), **S3** (API/UI badge + audit + user guide).

### Risks for discovery

1. **Privacy regression** — batch inference must respect `allow_raw_transactions=false` default (feature allowlist TBD)
2. **Projection correctness** — AI must not override config-mapped rows; low-confidence fallback behavior needs explicit threshold
3. **BUG-0007 overlap** — share normalization utilities only; do not route forecast through chat tools
4. **Operator trust** — badge semantics when mixed config+AI month totals

### Acceptance traceability

Eight checklist rows in `docs/product/acceptance.md` § US-0015 — 1 prerequisite checked (BUG-0012) + AC-1–AC-7 open.

### Intake evidence

- `intake_run_id`: `intake-20260606-us0015`
- `selected_pack`: `small-intake-pack`
- `writer_id`: `po`
- Evidence bundle: `handoffs/intake_evidence/intake-20260606-us0015.json`
- `prior_intake_ref`: `handoffs/intake_evidence/intake-20260605-forecast-monthly-buckets.json`
- Split: single epic (sprint-plan slices US-0015-S1..S3)

`triad_hot_surface`: rollover units=2,1; --check PASS

---

