# Technical Specification — US-0015

## Overview

Implement **DEC-0078**: AI bucket inference module with privacy allowlist; projection merge with config precedence; monthly API provenance; ForecastPage badge; audit persistence.

## Components

| Layer | Change |
|-------|--------|
| `backend/src/forecast/bucket_inference.rs` | **New** — `BucketInferenceService`, cascade stages 2–3 |
| `backend/src/ai/privacy.rs` | **Extend** — `prepare_bucket_features`, `BucketFeatureRow` |
| `backend/src/forecast/project.rs` | **Extend** — `resolve_bucket_with_ai`; provenance tracking |
| `backend/src/forecast/categories.rs` | **Extend** — `BucketSource` enum |
| `backend/src/api/forecast.rs` | **Extend** — `bucket_sources`, `ai_mapped` on `MonthlyPointResponse` |
| `backend/config/default.toml` | **Extend** — `ai_bucket_min_confidence = 0.75` |
| `frontend/src/pages/ForecastPage.tsx` | **Extend** — AI-mapped badge (seasonal callout pattern) |
| `docs/user-guides/US-0015.md` | **New** — operator guide at execute |

## Interfaces

### Privacy feature row (R-0075)

```rust
pub struct BucketFeatureRow {
    pub feature_id: String,
    pub category_name: Option<String>,
    pub merchant_token: String,
    pub amount_sign: i8,
    pub magnitude_band: String,
    pub recurring_label: Option<String>,
    pub pattern_class: Option<String>,
}
```

### LLM structured I/O

**Request task:** `forecast_bucket_assignment`

**Response:**

```json
{
  "assignments": [
    {
      "feature_id": "bf_a1b2c3d4",
      "bucket": "fixed",
      "confidence": 0.82,
      "rationale_code": "recurring_utility_pattern"
    }
  ]
}
```

### API extension

```rust
pub struct BucketSources {
    pub income: String,
    pub fixed_costs: String,
    pub variable_costs: String,
}
```

Values: `"config"` | `"ai"` | `"default"`.

### TOML

```toml
[forecast]
ai_bucket_min_confidence = 0.75
```

## Sequencing (sprint-plan input)

1. S1 inference + privacy + unit tests
2. S2 projection merge + AC-1 integration tests
3. S3 API + UI + audit + user guide + OIDC smoke

## Verification

| AC | Test surface |
|----|--------------|
| AC-1 | Integration: config-mapped row never AI-overridden |
| AC-2 | Unit: threshold apply/reject; provider_unavailable |
| AC-3 | Unit: allowlist rejects raw description under default TOML |
| AC-4 | API contract test on monthly response |
| AC-5 | Frontend: badge when `ai_mapped=true` |
| AC-6 | Audit row persisted with `feature_id` only |
| AC-7 | OIDC smoke; chat/ML regression unchanged |
