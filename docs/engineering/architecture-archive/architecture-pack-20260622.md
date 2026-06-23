# Architecture archive pack (2026-06-22)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=120`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 13
- First archived heading: `## US-0015 — AI-assisted forecast category bucket mapping`
- Last archived heading: `## US-0015 — AI-assisted forecast category bucket mapping`
- Verification tuple (mandatory):
  - archived_body_lines=263
  - preamble_lines=10
  - retained_body_lines=2843

---

## US-0015 — AI-assisted forecast category bucket mapping

**Status:** Architecture complete (2026-06-06)  
**Discovery:** `discovery-20260606-us0015` in `handoffs/po_to_tl.md` / `handoffs/archive/po-to-tl-pack-20260606-o.md`  
**Research:** [R-0074](research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy), [R-0075](research.md#r-0075--us-0015-forecast-bucket-privacy-feature-allowlist)  
**Decisions:** **DEC-0078** (AI bucket cascade); frozen **DEC-0007** (config map), **DEC-0032** (privacy defaults), **DEC-0069** (chat tool isolation)  
**Depends on:** BUG-0012 DONE (Q0014, DEC-0007 baseline), US-0008 (`build_provider()`), US-0006 (audit pattern)  
**Sprint:** **S0016** recommended — slices US-0015-S1..S3  
**Acceptance:** `docs/product/acceptance.md` § US-0015 (8 rows)  
**Spec-pack:** `docs/engineering/spec-pack/US-0015-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**User guide:** `docs/user-guides/US-0015.md` (`USER_GUIDE_MODE=1`; execute publishes content)

### Problem

BUG-0012 (Q0014) shipped DEC-0007 config-driven `resolve_bucket` for **recurring** pattern dues in `project.rs`. Remaining gaps:

| AC | Discovery verdict | Execute weight |
|----|-------------------|----------------|
| Prerequisite BUG-0012 | Shipped | Verify only |
| AC-1 Baseline precedence | Partial | S2 primary |
| AC-2 AI inference | **Gap** | **S1 primary** |
| AC-3 Privacy defaults | **Gap** | **S1 primary** |
| AC-4 API visibility | **Gap** | S3 |
| AC-5 UI badge | **Gap** | S3 |
| AC-6 Audit trail | **Gap** | S3 |
| AC-7 Regression | Verify | S3 smoke |

**Critical path:** uncategorized mirror rows and config-map misses on recurring dues still collapse to Variable via `map_category` empty-name default. **Rolling residual** (`variable_residual` daily rate) remains hardcoded Variable — documented MVP limitation (stage-2 gate).

`isolation_scope`: artifact + repo source reads; no host `.env` / secrets read.

### System context

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Browser — ForecastPage.tsx Monthly tab                                     │
│    Four stat cards (Income / Fixed / Variable / Free cashflow)              │
│    Seasonal callout pattern → AI-mapped badge when ai_mapped=true (S3)      │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ GET /api/v1/forecast/monthly
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum)                                                     │
│                                                                             │
│  ForecastService::recompute (DEC-0010 post-sync)                            │
│    └─▶ project_account (project.rs)                                         │
│           ├─▶ resolve_bucket (categories.rs) — DEC-0007 config first (AC-1)│
│           ├─▶ BucketInferenceService (NEW S1) — LLM batch on ambiguous rows │
│           │      PrivacyLayer::prepare_bucket_features (R-0075)               │
│           │      build_provider() when configured (US-0008)                   │
│           └─▶ variable_residual → Variable (MVP unchanged)                  │
│                                                                             │
│  GET /api/v1/forecast/monthly ──▶ MonthlyPointResponse + bucket_sources     │
│                                   + ai_mapped (S3)                          │
│  ai_tool_audit ──▶ forecast_bucket_assignment rows (S3, AC-6)               │
└─────────────────────────────────────────────────────────────────────────────┘
```

**AC-7 boundary:** No changes to chat six-tool registry (DEC-0069), US-0013 ML overlay, or Firefly write-back.

### Architecture contract (DEC-0078)

```text
US-0015
├── S1 — AI bucket inference module (P0)
│   ├── T1 — backend/src/forecast/bucket_inference.rs (rule→LLM cascade)
│   ├── T2 — PrivacyLayer::prepare_bucket_features + BucketFeatureRow (R-0075)
│   ├── T3 — Structured LLM I/O + ai_bucket_min_confidence=0.75 TOML
│   └── T4 — Unit tests: privacy allowlist, threshold, provider_unavailable
├── S2 — Projection merge (P0)
│   ├── T1 — resolve_bucket_with_ai wrapper; config precedence guard (AC-1)
│   ├── T2 — Recurring dues: AI only on config-map miss / ambiguous rows
│   ├── T3 — Provenance tracking per monthly accumulation (source enum)
│   └── T4 — Integration tests: config never overridden by AI
└── S3 — API / UI / audit (P1)
    ├── T1 — MonthlyPointResponse bucket_sources + ai_mapped (AC-4)
    ├── T2 — ForecastPage AI-mapped badge (seasonal callout pattern, AC-5)
    ├── T3 — ai_tool_audit forecast_bucket_assignment persistence (AC-6)
    ├── T4 — docs/user-guides/US-0015.md
    └── V1 — OIDC /forecast Monthly smoke (AC-7; BACKEND_FRONTEND_DEPLOY)
```

**Out of scope:** RAG/vector index; seventh chat tool; `[forecast.merchant_aliases]` TOML (post-MVP); rolling-residual aggregate AI split (stage-2); US-0013 ML changes.

### S1 — AI bucket inference (frozen — DEC-0078 §1–3)

#### Cascade stages

| Stage | Trigger | Output |
|-------|---------|--------|
| 1 Config | `category_id` resolves via `category_buckets` | Bucket from DEC-0007; source=`config` |
| 2 Rule heuristics | Config miss; optional in-module sign/pattern rules | Bucket + confidence; short-circuit ≥0.98 |
| 3 LLM batch | Ambiguous rows after stage 2; provider configured | `{ bucket, confidence, rationale_code }` per `feature_id` |
| 4 Fallback | confidence &lt; threshold or provider absent | Variable; source=`default`; audit reason |

**Batch cap:** 100 `BucketFeatureRow` per provider call (R-0074 cost guard).

#### Module placement

| Type | Path | Responsibility |
|------|------|----------------|
| `BucketInferenceService` | `backend/src/forecast/bucket_inference.rs` | Collect ambiguous rows; invoke privacy + provider; return assignments |
| `BucketAssignment` | same | `{ feature_id, bucket, confidence, source, rationale_code }` |
| `BucketSource` | `backend/src/forecast/types.rs` or `categories.rs` | Enum: `Config`, `Ai`, `Default` |

Reuse `build_provider()` from `backend/src/ai/provider.rs` via `AiService` — **no** `forecast_ai_*` env split.

#### TOML contract

```toml
[forecast]
ai_bucket_min_confidence = 0.75   # new; below → Variable + low_confidence audit
```

#### Privacy (R-0075 / AC-3)

`PrivacyLayer::prepare_bucket_features(rows) -> Vec<BucketFeatureRow>` before any HTTP call:

| Field | Default TOML | Treatment |
|-------|--------------|-----------|
| `category_name` | Sent if present | Lowercase trim |
| `merchant_token` | Sent | `hash_counterparty(normalized_payee)` |
| `amount_sign` | Sent | −1 / 0 / +1 only |
| `magnitude_band` | Sent | `"0-50"` \| `"50-200"` \| `"200+"` |
| `recurring_label` | Sent when detection provides | Subscription `display_name` preserve rule |
| `pattern_class` | Sent | `standing_order` \| `subscription` \| `discretionary` |
| Raw description/payee/IBAN/exact amount | **Never** under `allow_raw_transactions=false` | DEC-0032 default |

**Opt-in:** `allow_raw_transactions=true` permits normalized description for ≤50 rows/batch — document elevated risk in user guide.

**Local provider:** Same allowlist for Ollama/openai_compatible — consistency over "local = raw OK" (R-0075 §4).

#### Invalidation (DEC-0078 §4)

- **No cross-run DB assignment cache** for MVP
- Recompute inline each forecast pass (DEC-0010)
- Config bust: hash `[forecast.category_buckets]` at inference start
- Optional in-call memo: `payee_fingerprint + category_id + sign` within single `project_account` invocation only

### S2 — Projection merge (frozen — DEC-0078 §1, §5)

#### `resolve_bucket_with_ai` contract

```rust
fn resolve_bucket_with_ai(
    category_id: Option<&str>,
    category_names: &HashMap<String, String>,
    config: &ForecastConfig,
    inference_ctx: &BucketInferenceContext,  // mirror row + pattern metadata
    ai: Option<&BucketInferenceService>,
) -> (Bucket, BucketSource);
```

| Rule | Contract |
|------|----------|
| AC-1 precedence | If `resolve_bucket` returns non-Variable from config map → **return immediately**; AI never consulted |
| Config-mapped Variable | TOML explicit `"variable"` is config source — not AI-eligible |
| Ambiguous | Empty/missing category name on recurring due → stage 2–3 cascade |
| Threshold | `confidence >= ai_bucket_min_confidence` → apply AI bucket, source=`ai` |
| Low confidence | Variable + `low_confidence` audit |
| Provider down | Skip LLM; Variable + `provider_unavailable` audit |

#### Rolling residual (MVP limitation — decision gate deferred)

`accumulate_bucket(entry, Bucket::Variable, rolling.daily_rate)` **unchanged** in MVP. Rolling aggregate is not per-row disambiguable without stage-2 aggregate split design.

| Option | Verdict |
|--------|---------|
| MVP: keep rolling as Variable | **Accepted** — document in user guide; `ai_mapped` reflects recurring AI only |
| Stage-2: monthly aggregate AI split of residual | **Deferred** — requires new DEC if operator feedback demands |

#### Provenance aggregation

During monthly loop, track per-bucket mass by source. Dominant label per bucket month: precedence `config` &gt; `ai` &gt; `default`. `ai_mapped = true` when any AI-assigned mass &gt; 0 in that month (authoritative for badge per R-0074 §7).

### S3 — API / UI / audit (frozen — DEC-0078 §5–7)

#### API extension (`backend/src/api/forecast.rs`)

```rust
#[derive(Serialize)]
pub struct BucketSources {
    pub income: String,        // "config" | "ai" | "default"
    pub fixed_costs: String,
    pub variable_costs: String,
}

#[derive(Serialize)]
pub struct MonthlyPointResponse {
    // existing fields unchanged
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket_sources: Option<BucketSources>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ai_mapped: bool,
}
```

Reject per-daily-point provenance — AC-4 targets monthly cards only.

#### UI badge (AC-5)

Mirror `ForecastPage.tsx` seasonal callout block (~L259): when `ai_mapped`, show compact **AI-mapped** badge with tooltip explaining config precedence and privacy-safe inference. Config-only months: no badge.

#### Audit (AC-6)

Persist to `ai_tool_audit` (existing US-0006 pattern):

| Column / payload | Value |
|------------------|-------|
| `tool_name` | `forecast_bucket_assignment` |
| `result_summary` | Redacted: `feature_id`, proposed bucket, confidence, source, rationale_code |
| Never log | Raw merchant/description |

### Provider contract (DEC-0078 §6)

| Condition | Behavior |
|-----------|----------|
| `build_provider()` succeeds | LLM stage 3 for ambiguous rows |
| Provider absent / misconfigured | Rule-only stages 1–2; Variable fallback |
| Provider HTTP error | Skip batch; Variable; audit `provider_unavailable` |
| Ollama/local configured | Preferred for privacy-sensitive operators; same allowlist |

### Risks

| Risk | Mitigation |
|------|------------|
| Threshold calibration — German merchants below 0.75 | Monitor `low_confidence` audit rate in QA; TOML override documented |
| Rolling residual stays Variable | User guide + architecture stage-2 gate; `ai_mapped` scoped to recurring AI |
| Provider cost on large mirrors | Batch ambiguous rows only; cap 100/call |
| Privacy regression on forecast path | S1 code review gate; unit tests on allowlist |
| Mixed `bucket_sources` hides partial AI | `ai_mapped` boolean authoritative for badge |
| Config override regression | S2 integration tests — AC-1 guard |

### Decision gates

| Gate | Status | Resolution |
|------|--------|------------|
| DEC-0078 formalization | **Closed** | Accepted at architecture |
| Confidence threshold 0.75 | **Closed** | Default TOML; operator override allowed |
| Merchant aliases TOML | **Deferred** | Post-MVP extension point documented |
| Rolling residual AI split | **Deferred** | MVP keeps Variable; stage-2 if operator feedback |

**No blocking gates** — proceed to `/sprint-plan`.

### Acceptance mapping

| Row | Architecture slice | Verify |
|-----|-------------------|--------|
| Prerequisite | — | BUG-0012 DONE (checked) |
| AC-1 | S2 | Config map never overridden |
| AC-2 | S1, S2 | LLM proposal + threshold fallback |
| AC-3 | S1 | `prepare_bucket_features` allowlist |
| AC-4 | S3 | `bucket_sources` on monthly API |
| AC-5 | S3 | `ai_mapped` badge |
| AC-6 | S3 | `ai_tool_audit` rows |
| AC-7 | S3 | OIDC smoke; chat/ML unchanged |

### Next phase

`/sprint-plan` **S0016** — materialize US-0015-S1..S3 tasks; S1+S2 before S3 API/UI; then `/plan-verify` → `/execute`.

---

