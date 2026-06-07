# Tasks — Sprint S0016

**Story:** US-0015  
**Task count:** 12 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Slice | Est. | Acceptance refs |
|----|-------|--------|-------|------|-----------------|
| T-0163 | BucketInferenceService module (rule→LLM cascade) | open | S1 | 4h | AC-2 |
| T-0164 | PrivacyLayer::prepare_bucket_features + BucketFeatureRow | open | S1 | 3h | AC-3 |
| T-0165 | Structured LLM I/O + ai_bucket_min_confidence TOML | open | S1 | 3h | AC-2 |
| T-0166 | S1 unit tests: privacy, threshold, provider_unavailable | open | S1 | 2h | AC-2, AC-3 |
| T-0167 | resolve_bucket_with_ai + config precedence guard | open | S2 | 3h | AC-1 |
| T-0168 | Recurring dues AI on config-map miss | open | S2 | 4h | AC-1, AC-2 |
| T-0169 | Provenance tracking per monthly accumulation | open | S2 | 3h | AC-4 |
| T-0170 | S2 integration tests: config never overridden | open | S2 | 3h | AC-1 |
| T-0171 | MonthlyPointResponse bucket_sources + ai_mapped | open | S3 | 2h | AC-4 |
| T-0172 | ForecastPage AI-mapped badge | open | S3 | 2h | AC-5 |
| T-0173 | ai_tool_audit forecast_bucket_assignment persistence | open | S3 | 2h | AC-6 |
| T-0174 | User guide US-0015 + UAT OIDC smoke template | open | S3 | 2h | AC-7 |

---

## T-0163 — BucketInferenceService module (rule→LLM cascade)

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0078  
**Architecture slice:** US-0015-S1  
**Research:** R-0074

### Description

Add `backend/src/forecast/bucket_inference.rs` with `BucketInferenceService`:

| Stage | Mechanism |
|-------|-----------|
| 1 | DEC-0007 `resolve_bucket` — always first (caller guard in S2) |
| 2 | In-module rule heuristics (sign/pattern short-circuit ≥0.98) |
| 3 | LLM batch via `build_provider()` when configured |
| 4 | Variable fallback on low confidence / provider absent |

Types: `BucketAssignment { feature_id, bucket, confidence, source, rationale_code }`, `BucketSource` enum (`Config`, `Ai`, `Default`).

Batch cap: **100** `BucketFeatureRow` per provider call. Config bust: hash `[forecast.category_buckets]` at inference start. Optional in-call memo keyed by `payee_fingerprint + category_id + sign` within single `project_account`.

Wire module in `forecast/mod.rs`.

### Done when

- [ ] `BucketInferenceService` collects ambiguous rows and returns assignments
- [ ] Rule heuristics short-circuit before LLM when confidence ≥0.98
- [ ] Provider absent → skip LLM stage (no panic)
- [ ] Batch capped at 100 rows per call
- [ ] `cargo test` bucket_inference unit smoke passes

---

## T-0164 — PrivacyLayer::prepare_bucket_features + BucketFeatureRow

**Status:** open  
**Depends on:** T-0163  
**Decisions:** DEC-0078, DEC-0032  
**Architecture slice:** US-0015-S1  
**Research:** R-0075

### Description

Add `PrivacyLayer::prepare_bucket_features(rows) -> Vec<BucketFeatureRow>` in `backend/src/ai/privacy.rs`:

| Field | Default TOML | Treatment |
|-------|--------------|-----------|
| `category_name` | Sent if present | Lowercase trim |
| `merchant_token` | Sent | `hash_counterparty(normalized_payee)` |
| `amount_sign` | Sent | −1 / 0 / +1 |
| `magnitude_band` | Sent | `"0-50"` \| `"50-200"` \| `"200+"` |
| `recurring_label` | Sent when available | Subscription display_name preserve |
| `pattern_class` | Sent | `standing_order` \| `subscription` \| `discretionary` |
| Raw description/payee/IBAN/exact amount | **Never** under `allow_raw_transactions=false` | DEC-0032 default |

**Same allowlist for local and cloud providers.** Opt-in `allow_raw_transactions=true`: normalized description for ≤50 rows/batch — document elevated risk in user guide (T-0174).

### Done when

- [ ] `BucketFeatureRow` struct defined with allowlisted fields only
- [ ] `prepare_bucket_features` strips raw payee/description under default TOML
- [ ] Opt-in raw path gated and batch-limited to 50
- [ ] Local (Ollama) and cloud paths use identical allowlist

---

## T-0165 — Structured LLM I/O + ai_bucket_min_confidence TOML

**Status:** open  
**Depends on:** T-0163, T-0164  
**Decisions:** DEC-0078  
**Architecture slice:** US-0015-S1  
**Research:** R-0074

### Description

Structured LLM request/response for bucket batch inference:

- Prompt requests JSON array: `{ feature_id, bucket, confidence, rationale_code }` per row
- Parse with strict schema validation; malformed → skip row + audit
- Add to `backend/config/default.toml`:

```toml
[forecast]
ai_bucket_min_confidence = 0.75
```

Wire into `ForecastConfig` / config merge. `confidence >= threshold` → apply AI bucket; below → Variable + `low_confidence` audit reason.

Reuse US-0008 `build_provider()` — no `forecast_ai_*` env split.

### Done when

- [ ] `ai_bucket_min_confidence` in default TOML and config struct
- [ ] LLM I/O parses structured bucket assignments
- [ ] Threshold gate applies 0.75 default (operator-overridable)
- [ ] Provider HTTP error → skip batch + `provider_unavailable` audit path

---

## T-0166 — S1 unit tests: privacy, threshold, provider_unavailable

**Status:** open  
**Depends on:** T-0163, T-0164, T-0165  
**Decisions:** DEC-0078  
**Architecture slice:** US-0015-S1

### Description

Unit tests in `bucket_inference.rs` / `privacy.rs`:

| Case | Assert |
|------|--------|
| Privacy allowlist | No raw description/payee in prepared features under default TOML |
| Threshold | confidence 0.74 → Variable; 0.75 → apply AI bucket |
| Provider unavailable | Mock provider error → Variable + `provider_unavailable` rationale |
| Batch cap | 101 ambiguous rows → two batches max 100 + remainder |

### Done when

- [ ] Privacy allowlist tests pass under `allow_raw_transactions=false`
- [ ] Threshold boundary tests at 0.74 / 0.75
- [ ] Provider-unavailable fallback tested without network
- [ ] `cargo test` S1 module tests green

---

## T-0167 — resolve_bucket_with_ai + config precedence guard

**Status:** open  
**Depends on:** T-0166  
**Decisions:** DEC-0078, DEC-0007  
**Architecture slice:** US-0015-S2

### Description

Add `resolve_bucket_with_ai` in `forecast/categories.rs` or `project.rs`:

```rust
fn resolve_bucket_with_ai(
    category_id: Option<&str>,
    category_names: &HashMap<String, String>,
    config: &ForecastConfig,
    inference_ctx: &BucketInferenceContext,
    ai: Option<&BucketInferenceService>,
) -> (Bucket, BucketSource);
```

| Rule | Contract |
|------|----------|
| AC-1 precedence | Non-Variable from config map → return immediately; AI never consulted |
| Config-mapped Variable | TOML explicit `"variable"` is config source — not AI-eligible |
| Ambiguous | Empty/missing category on recurring due → stages 2–3 |

### Done when

- [ ] Config-mapped Income/Fixed never reach AI path
- [ ] Explicit config Variable not AI-eligible
- [ ] Function returns `(Bucket, BucketSource)` for all paths
- [ ] `variable_residual` daily rate path unchanged (MVP Variable only)

---

## T-0168 — Recurring dues AI on config-map miss

**Status:** open  
**Depends on:** T-0167  
**Decisions:** DEC-0078  
**Architecture slice:** US-0015-S2

### Description

Wire `resolve_bucket_with_ai` into `project_account` recurring due accumulation:

- Replace direct `resolve_bucket` calls on ambiguous recurring rows with `resolve_bucket_with_ai`
- Pass `BucketInferenceContext` with mirror row + pattern metadata
- Instantiate `BucketInferenceService` once per `project_account` (or forecast pass)
- Low confidence → Variable + audit `low_confidence`
- Provider down → Variable + audit `provider_unavailable`

**Frozen:** `accumulate_bucket(entry, Bucket::Variable, rolling.daily_rate)` unchanged for rolling residual.

### Done when

- [ ] Recurring dues with config miss invoke AI cascade
- [ ] Config hits bypass AI (AC-1)
- [ ] Rolling residual still Variable only
- [ ] No silent zero absorption for ambiguous recurring rows

---

## T-0169 — Provenance tracking per monthly accumulation

**Status:** open  
**Depends on:** T-0168  
**Decisions:** DEC-0078  
**Architecture slice:** US-0015-S2

### Description

During monthly loop, track per-bucket mass by `BucketSource`:

- Accumulate mass per `(bucket, source)` per month
- Dominant label per bucket: precedence `config` > `ai` > `default`
- `ai_mapped = true` when any AI-assigned mass > 0 in month (badge authoritative)

Persist provenance on monthly point struct for API serialization in T-0171.

### Done when

- [ ] Per-month per-bucket source mass tracked during projection
- [ ] Dominant source computed with correct precedence
- [ ] `ai_mapped` flag set when any AI mass > 0
- [ ] Mixed-source months: `ai_mapped` true even if dominant label is config

---

## T-0170 — S2 integration tests: config never overridden

**Status:** open  
**Depends on:** T-0169  
**Decisions:** DEC-0078, DEC-0007  
**Architecture slice:** US-0015-S2

### Description

Integration/unit tests in `project.rs` or `forecast_integration`:

| Case | Assert |
|------|--------|
| Salary category in TOML map | Income bucket; source=config; AI not called |
| Rent category in TOML map | Fixed bucket; source=config |
| Uncategorized recurring due | AI path invoked when provider mocked |
| Config + ambiguous same month | Config mass dominates label; ai_mapped reflects AI mass if any |

Extend or add `cargo test --test forecast_integration` case if seeded mirror available.

### Done when

- [ ] Config-mapped categories never overridden in `monthly_map`
- [ ] AI mock invoked only on ambiguous rows
- [ ] `ai_mapped` true when AI mass present alongside config mass
- [ ] `cargo test` projection tests green

---

## T-0171 — MonthlyPointResponse bucket_sources + ai_mapped

**Status:** open  
**Depends on:** T-0170  
**Decisions:** DEC-0078  
**Architecture slice:** US-0015-S3

### Description

Extend `backend/src/api/forecast.rs`:

```rust
pub struct BucketSources {
    pub income: String,        // "config" | "ai" | "default"
    pub fixed_costs: String,
    pub variable_costs: String,
}

pub struct MonthlyPointResponse {
    // existing fields unchanged
    pub bucket_sources: Option<BucketSources>,
    pub ai_mapped: bool,
}
```

Wire from projection provenance (T-0169). Reject per-daily-point provenance — monthly cards only.

### Done when

- [ ] `GET /api/v1/forecast/monthly` returns `bucket_sources` when data available
- [ ] `ai_mapped` boolean present on monthly points
- [ ] Existing monthly fields unchanged (backward compatible)
- [ ] OpenAPI/serde skips null `bucket_sources` on legacy rows if needed

---

## T-0172 — ForecastPage AI-mapped badge

**Status:** open  
**Depends on:** T-0171  
**Decisions:** DEC-0078  
**Architecture slice:** US-0015-S3

### Description

Mirror `ForecastPage.tsx` seasonal callout block (~L259): when `series[0].ai_mapped`, show compact **AI-mapped** badge on Monthly stat cards with tooltip:

- Config precedence explained
- Privacy-safe inference (hashed merchant, amount bands)
- Rolling residual stays Variable in MVP

Config-only months (`ai_mapped=false`): no badge.

### Done when

- [ ] Badge visible when `ai_mapped=true` on first month point
- [ ] No badge when config-only month
- [ ] Tooltip documents DEC-0078 operator expectations
- [ ] Monthly four-card layout unchanged (Finanzguru pattern)

---

## T-0173 — ai_tool_audit forecast_bucket_assignment persistence

**Status:** open  
**Depends on:** T-0171  
**Decisions:** DEC-0078, US-0006  
**Architecture slice:** US-0015-S3

### Description

Persist `forecast_bucket_assignment` rows in `ai_tool_audit` (US-0006 pattern):

| Field | Value |
|-------|-------|
| `tool_name` | `forecast_bucket_assignment` |
| `result_summary` | Redacted: `feature_id`, bucket, confidence, source, `rationale_code` |
| Never log | Raw merchant/description |

Log on each AI assignment and on `low_confidence` / `provider_unavailable` fallback paths.

### Done when

- [ ] Audit rows written on AI bucket assignments
- [ ] Fallback paths audited with rationale codes
- [ ] No raw payee/description in audit payload
- [ ] Operator can query audit table for forecast bucket events

---

## T-0174 — User guide US-0015 + UAT OIDC smoke template

**Status:** open  
**Depends on:** T-0172, T-0173  
**Decisions:** DEC-0078, DEC-0059  
**Architecture slice:** US-0015-S3  
**Research:** R-0074, R-0075

### Description

Publish `docs/user-guides/US-0015.md` from stub (`USER_GUIDE_MODE=1`):

| Section | Content |
|---------|---------|
| Purpose | AI bucket mapping when categories ambiguous |
| Config wins | `[forecast.category_buckets]` never overridden |
| AI-mapped badge | Monthly tab indicator semantics |
| Privacy | Default allowlist; opt-in raw transactions risk |
| Rolling residual | Stays Variable in MVP |
| Troubleshooting | Provider absent → rule-only; low confidence → Variable |

Populate `sprints/S0016/uat.md` OIDC smoke template for AC-7:

| Step | Contract |
|------|----------|
| Profile | US-0010 external |
| Route | `/forecast` Monthly tab |
| Gate | **BACKEND_FRONTEND_DEPLOY** |
| Regression | Chat tools + ML Compare unchanged |

Cross-link `docs/user-guides/US-0002.md` forecast overview.

### Done when

- [ ] User guide stub replaced with operator-facing content
- [ ] MVP rolling-residual limitation documented
- [ ] `uat.md` includes OIDC Monthly smoke checklist
- [ ] AC-7 regression scope documented (no chat/ML changes)

---

## Execution order (recommended)

1. **S1 inference:** T-0163 → T-0164 → T-0165 → T-0166
2. **S2 projection:** T-0167 → T-0168 → T-0169 → T-0170
3. **S3 visibility:** T-0171 → T-0172 → T-0173 → T-0174
4. **Operator:** BACKEND_FRONTEND_DEPLOY → UAT omniflow `/forecast` Monthly smoke

```text
T-0163 → T-0164 → T-0165 → T-0166
  ↓
T-0167 → T-0168 → T-0169 → T-0170
  ↓
T-0171 → T-0172 → T-0173 → T-0174
  ↓
Operator: deploy backend+frontend → UAT OIDC smoke
```

## Acceptance coverage map

| AC | Tasks | Notes |
|----|-------|-------|
| Prerequisite | — | BUG-0012 DONE (checked) |
| AC-1 | T-0167, T-0168, T-0170 | Config precedence + integration tests |
| AC-2 | T-0163, T-0165, T-0166, T-0168 | LLM cascade + threshold fallback |
| AC-3 | T-0164, T-0166 | Privacy allowlist unit tests |
| AC-4 | T-0169, T-0171 | Provenance + API fields |
| AC-5 | T-0172 | AI-mapped badge |
| AC-6 | T-0173 | Audit trail |
| AC-7 | T-0174 | OIDC smoke UAT; chat/ML unchanged |

## Operator gate

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_FRONTEND_DEPLOY** | UAT AC-7 omniflow smoke | Deploy S1–S3 backend + frontend on US-0010 external profile |

## Split decision

- **Why 12 tasks:** Architecture S1(4) + S2(4) + S3(4); equals `SPRINT_MAX_TASKS=12`.
- **OIDC in UAT:** T-0174 captures AC-7 template — avoids 13th task and auto-split.
- **S1 before S2:** Inference service required for projection merge.
- **S2 before S3:** Provenance required for API/UI fields.
