# CRS — US-0015

## Purpose

Close the **AI bucket mapping gap** deferred from BUG-0012: operators see Income/Fixed/Variable monthly breakdown that reflects real spending patterns when Firefly categories are missing — with visible provenance and audit under existing privacy constraints.

## Scope

**In scope**

- `backend/src/forecast/bucket_inference.rs` — rule→LLM cascade (DEC-0078)
- `PrivacyLayer::prepare_bucket_features` — R-0075 allowlist
- `project.rs` projection merge with `resolve_bucket_with_ai`
- `MonthlyPointResponse` `bucket_sources` + `ai_mapped`
- `ForecastPage.tsx` AI-mapped badge
- `ai_tool_audit` forecast bucket rows
- `docs/user-guides/US-0015.md`
- OIDC `/forecast` Monthly smoke (AC-7)

**Out of scope**

- Chat tool registry changes (DEC-0069)
- US-0013 ML overlay / `variant=ml_enhanced`
- Firefly category write-back
- Merchant alias TOML table (post-MVP)
- Rolling residual aggregate AI split (stage-2)

## Acceptance criteria ref

See `docs/product/acceptance.md` § US-0015 — 8 rows (1 prerequisite checked + AC-1–AC-7).

## Dependencies

- BUG-0012 DONE (Q0014, DEC-0007)
- US-0008 DONE (`build_provider()`)
- US-0006 DONE (audit pattern, PrivacyLayer)
- US-0010 external profile (AC-7 smoke target)
