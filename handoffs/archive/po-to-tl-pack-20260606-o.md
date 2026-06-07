# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 10
- First archived heading: `## discovery-20260606-us0015 — US-0015 AI forecast bucket mapping discovery (hot pointer)`
- Last archived heading: `## discovery-20260606-us0015 — US-0015 AI forecast bucket mapping discovery`
- Verification tuple (mandatory):
  - archived_body_lines=87
  - retained_body_lines=499

---

## discovery-20260606-us0015 — US-0015 AI forecast bucket mapping discovery (hot pointer)

**From:** PO | **Story:** US-0015 | **Run:** auto-20260606-us0015-001 | **Next:** `/research`  
**Full handoff:** [`po-to-tl-pack-20260606-o.md`](handoffs/archive/po-to-tl-pack-20260606-o.md#discovery-20260606-us0015--us-0015-ai-forecast-bucket-mapping-discovery)  
**Verdict:** config baseline shipped (BUG-0012); AI inference + provenance + badge gaps; rolling residual hardcoded Variable; 8 AC unchanged; 6 open questions → research.  
`triad_hot_surface`: rollover units=3,2 cumulative; archived_body_lines=87; retained_body_lines=499; --check PASS

---

## discovery-20260606-us0015 — US-0015 AI forecast bucket mapping discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-06  
**Story:** US-0015  
**Orchestrator run:** auto-20260606-us0015-001  
**Next phase:** `/research`

### Summary

Discovery confirms **US-0015 is additive AI enrichment** on the forecast projection path after **BUG-0012** config baseline. Code audit: `resolve_bucket` + recurring assignment **shipped**; **rolling residual always Variable**; no AI module, no `bucket_source` on monthly API, no **AI-mapped** badge on Monthly tab. Operator monthly UX (four stat cards + chart) **retained** — trust signal only.

### Partial implementation review

| Area | Status |
|------|--------|
| `categories.rs` / DEC-0007 map | **Done** (Q0014) |
| `project.rs` recurring bucketing | **Done** |
| `project.rs` rolling → Variable | **Gap** — primary S2 hook |
| AI bucket inference | **Missing** (S1) |
| `GET /api/v1/forecast/monthly` provenance | **Missing** (S3) |
| `ForecastPage` Monthly AI badge | **Missing** (S3) |
| Forecast bucket audit | **Missing** (S3) |
| `PrivacyLayer` on projection path | **Not wired** (S1) |

### UX references captured

- **Finanzguru parity** (US-0002): four scannable monthly stat cards — layout unchanged.
- **Seasonal badge** (`ForecastPage.tsx` L259–270): template for **AI-mapped** badge + tooltip above/beside affected cards.
- **R-0074 cascade:** config map → feature extraction → LLM batch → low-confidence Variable + audit (Spendify/NumbyAI pattern).
- **No write-back / no toast:** read-only projection enrichment; badge semantics only.

### Discovery decomposition evidence

- Feature/workflow count: inference + projection merge + API/UI + audit (4 surfaces — **single epic retained**)
- Cross-cutting: `forecast/project.rs`, new AI module, monthly API, `ForecastPage.tsx`
- Acceptance breadth: **8 rows unchanged**
- Risk surface: privacy allowlist, config precedence, mixed bucket_source months, BUG-0007 utility sharing only

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#US-0015` | Discovery findings + partial impl matrix | pass |
| `docs/product/acceptance.md` US-0015 | 8 criteria still valid | pass |
| `backend/src/forecast/project.rs` | Rolling Variable + recurring resolve_bucket documented | pass |
| `frontend/src/pages/ForecastPage.tsx` | Monthly cards present; badge gap documented | pass |
| R-0074 | Cascade + architecture gates linked | pass |

### Open questions (carry to `/research`)

| Topic | Question |
|-------|----------|
| **Confidence threshold** | Default cutoff for low-confidence → Variable |
| **Invalidation** | Full recompute vs incremental AI cache on sync |
| **Feature allowlist** | Fields permitted under `allow_raw_transactions=false` |
| **API shape** | Per-bucket `bucket_source` on series points vs month flags |
| **Provider default** | Ollama vs OpenAI for batch inference (US-0008) |
| **TOML aliases** | Stage-2 merchant/category aliases before LLM — MVP or defer? |

### Recommended next steps

1. `/research` — resolve 6 open questions; extend R-0074 with code-path spike notes
2. `/architecture` — DEC for threshold, allowlist, API contract, invalidation policy
3. `/sprint-plan` — materialize US-0015-S1..S3 tasks after architecture

### Evidence

- Vision: `docs/product/vision.md` (Discovery notes US-0015 2026-06-06)
- Backlog: `docs/product/backlog.md#US-0015` (#### Discovery findings 2026-06-06)
- Intake: `handoffs/intake_evidence/intake-20260606-us0015.json`
- Research: [R-0074](docs/engineering/research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy) (open)
- Acceptance: `docs/product/acceptance.md` § US-0015 (8 rows, unchanged)
- Code: `backend/src/forecast/categories.rs`, `project.rs`, `api/forecast.rs`, `frontend/src/pages/ForecastPage.tsx`

---

