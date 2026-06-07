# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 10
- First archived heading: `## research-20260606-us0015 — US-0015 AI forecast bucket mapping technical research`
- Last archived heading: `## research-20260606-us0015 — US-0015 AI forecast bucket mapping technical research`
- Verification tuple (mandatory):
  - archived_body_lines=51
  - retained_body_lines=499

---

## research-20260606-us0015 — US-0015 AI forecast bucket mapping technical research

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-06  
**Story:** US-0015  
**Orchestrator run:** auto-20260606-us0015-001  
**Next phase:** `/architecture`

### Summary

Web + codebase research completed for US-0015 AI-assisted forecast bucket mapping. Fulfilled **[R-0074](docs/engineering/research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy)** and appended **[R-0075](docs/engineering/research.md#r-0075--us-0015-forecast-bucket-privacy-feature-allowlist)** resolving all six discovery open questions. Recommends **DEC-0078** cascade contract. No host `.env` or secrets read.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Confidence threshold** | [R-0074 §7](docs/engineering/research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy) | **0.75 default** `ai_bucket_min_confidence`; below → Variable + audit |
| **Invalidation** | [R-0074 §7](docs/engineering/research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy) | Inline recompute per DEC-0010; config-hash bust; no DB cache MVP |
| **Privacy allowlist** | [R-0075](docs/engineering/research.md#r-0075--us-0015-forecast-bucket-privacy-feature-allowlist) | `PrivacyLayer::prepare_bucket_features()`; hashed merchant tokens; magnitude bands |
| **API provenance** | [R-0074 §7](docs/engineering/research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy) | `bucket_sources` map + `ai_mapped` on `MonthlyPointResponse` |
| **Provider default** | [R-0074 §7](docs/engineering/research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy) | Reuse US-0008 `build_provider()`; rule-only when absent |
| **Merchant aliases** | [R-0074 §7](docs/engineering/research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy) | **Defer post-MVP**; `category_buckets` sufficient |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Confidence threshold for AI bucket assignment? | **0.75** default TOML; audit low-confidence → Variable |
| Invalidation when config or transactions change? | Recompute each forecast pass; config map hash bust |
| Privacy allowlist for merchant/description to LLM? | **R-0075** — hashed tokens + bands; no raw description default |
| API shape for bucket_source / provenance? | Per-month `bucket_sources` + `ai_mapped` boolean |
| Provider default (rule-only vs LLM cascade)? | Rule-first; LLM via shared US-0008 provider when configured |
| Optional TOML merchant aliases? | **Out of MVP** — architecture documents extension point |

### Risks surfaced (carry to architecture)

1. **Rolling residual** — `variable_residual` hardcoded Variable; stage-2 or document limitation
2. **Threshold calibration** — monitor audit `low_confidence` rate on German merchants
3. **LLM batch cost** — cap ambiguous rows (~100/call); rule path handles majority
4. **Privacy regression** — forecast must use `prepare_bucket_features`, not raw mirror export

### Recommended next steps

1. `/architecture` — formalize **DEC-0078**; update `architecture.md` § US-0015; spec-pack + user guide stub
2. `/sprint-plan` — materialize US-0015-S1..S3 after architecture

`triad_hot_surface`: research handoff prepended; R-0074 fulfilled; R-0075 appended; state governance updated 2026-06-06T16:05:00Z

---

