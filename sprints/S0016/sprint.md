# Sprint S0016

**ID:** S0016  
**Story:** US-0015 — AI-assisted forecast category bucket mapping  
**Status:** PLANNED  
**Created:** 2026-06-06  
**Orchestrator:** `auto-20260606-us0015-001`

## Goal

Deliver **DEC-0078** AI-assisted forecast bucket mapping: `BucketInferenceService` rule→LLM cascade with R-0075 privacy allowlist; `resolve_bucket_with_ai` projection merge preserving DEC-0007 config precedence; monthly API `bucket_sources` + `ai_mapped`; ForecastPage **AI-mapped** badge; `forecast_bucket_assignment` audit trail; publish `docs/user-guides/US-0015.md`; OIDC smoke template in UAT.

## Scope

| Slice | Tasks | Primary files |
|-------|-------|---------------|
| **US-0015-S1** — AI bucket inference | T-0163 … T-0166 | `forecast/bucket_inference.rs`, `ai/privacy.rs`, `config/default.toml` |
| **US-0015-S2** — Projection merge | T-0167 … T-0170 | `forecast/project.rs`, `forecast/categories.rs`, `forecast/types.rs` |
| **US-0015-S3** — API / UI / audit | T-0171 … T-0174 | `api/forecast.rs`, `ForecastPage.tsx`, `ai_tool_audit`, `docs/user-guides/US-0015.md` |

**Out of scope:** RAG/vector index; seventh chat tool; `[forecast.merchant_aliases]` TOML; rolling-residual aggregate AI split (stage-2); US-0013 ML changes.

## Task table

| ID | Title | Slice | Est. | Acceptance |
|----|-------|-------|------|------------|
| T-0163 | BucketInferenceService module (rule→LLM cascade) | S1 | 4h | AC-2 |
| T-0164 | PrivacyLayer::prepare_bucket_features + BucketFeatureRow | S1 | 3h | AC-3 |
| T-0165 | Structured LLM I/O + ai_bucket_min_confidence TOML | S1 | 3h | AC-2 |
| T-0166 | S1 unit tests: privacy, threshold, provider_unavailable | S1 | 2h | AC-2, AC-3 |
| T-0167 | resolve_bucket_with_ai + config precedence guard | S2 | 3h | AC-1 |
| T-0168 | Recurring dues AI on config-map miss | S2 | 4h | AC-1, AC-2 |
| T-0169 | Provenance tracking per monthly accumulation | S2 | 3h | AC-4 |
| T-0170 | S2 integration tests: config never overridden | S2 | 3h | AC-1 |
| T-0171 | MonthlyPointResponse bucket_sources + ai_mapped | S3 | 2h | AC-4 |
| T-0172 | ForecastPage AI-mapped badge | S3 | 2h | AC-5 |
| T-0173 | ai_tool_audit forecast_bucket_assignment persistence | S3 | 2h | AC-6 |
| T-0174 | User guide US-0015 + UAT OIDC smoke template | S3 | 2h | AC-7 |

**Total estimate:** ~32h across 12 tasks.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Threshold calibration (German merchants) | Monitor `low_confidence` audit rate; TOML override | DEC-0078 §2, T-0165 |
| Rolling residual stays Variable | User guide + badge scoped to recurring AI | T-0174 |
| Provider cost on large mirrors | Batch ambiguous rows only; cap 100/call | DEC-0078 §6, T-0163 |
| Privacy regression on forecast path | S1 code review + allowlist unit tests | T-0164, T-0166 |
| Mixed bucket_sources vs badge | `ai_mapped` boolean authoritative | T-0171, T-0172 |
| AC-7 operator gate | OIDC smoke pass-with-prerequisites **BACKEND_FRONTEND_DEPLOY** | T-0174 |

## Definition of Done

- All 12 sprint tasks complete (`T-0163` … `T-0174`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` § US-0015 (7 open AC + prerequisite checked)
- Config-mapped buckets never overridden by AI (AC-1 integration tests pass)
- Privacy allowlist enforced under default TOML (AC-3 unit tests pass)
- Monthly API returns `bucket_sources` + `ai_mapped` when AI contributes (AC-4)
- `docs/user-guides/US-0015.md` published (`USER_GUIDE_MODE=1`)
- Operator gate **BACKEND_FRONTEND_DEPLOY** documented before omniflow OIDC smoke (AC-7)

## Architecture references

- `docs/engineering/architecture.md` § US-0015
- `decisions/DEC-0078.md`
- Research: R-0074, R-0075; frozen DEC-0007, DEC-0032, DEC-0069
- Spec-pack: `docs/engineering/spec-pack/US-0015-{design-concept,crs,technical-specification}.md`
- User guide: `docs/user-guides/US-0015.md`
- Discovery: `handoffs/po_to_tl.md#architecture-20260606-us0015`
- Acceptance: `docs/product/acceptance.md` § US-0015
- Handoff: `handoffs/tl_to_dev.md` (`sprint-plan-20260606-s0016-us0015`)

## Sequencing (frozen)

```text
S1: T-0163 → T-0164 → T-0165 → T-0166
S2: T-0167 → T-0168 → T-0169 → T-0170 (after S1 — inference service required)
S3: T-0171 → T-0172 → T-0173 → T-0174 (after S2 — provenance data required)
Operator: BACKEND_FRONTEND_DEPLOY → verify-work omniflow /forecast Monthly smoke (UAT)
```

## Acceptance coverage map

| Row | Tasks | Notes |
|-----|-------|-------|
| Prerequisite | — | BUG-0012 DONE (pre-checked) |
| AC-1 | T-0167, T-0168, T-0170 | Config precedence guard + integration tests |
| AC-2 | T-0163, T-0165, T-0166, T-0168 | LLM cascade + threshold fallback |
| AC-3 | T-0164, T-0166 | `prepare_bucket_features` allowlist |
| AC-4 | T-0169, T-0171 | Provenance aggregation + API fields |
| AC-5 | T-0172 | AI-mapped badge on Monthly tab |
| AC-6 | T-0173 | `forecast_bucket_assignment` audit rows |
| AC-7 | T-0174 | OIDC smoke UAT template; chat/ML unchanged |

## Split decision

- **Why 12 tasks:** Architecture slices S1(4) + S2(4) + S3(4) = 12; equals `SPRINT_MAX_TASKS=12`.
- **Why not split S0016a/b:** Single epic with S1→S2→S3 dependency chain; incomplete projection if split mid-cascade.
- **OIDC in UAT:** AC-7 captured in T-0174 UAT template (mirrors S0015 pattern) — avoids 13th task requiring auto-split.
- **S1+S2 before S3:** API/UI/audit require provenance from projection merge.

## Next phase

`/plan-verify` in fresh subagent/chat
