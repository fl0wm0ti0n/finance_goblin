# Architecture

## Overview

Flow Finance AI is a self-hosted analytics layer on Firefly III. **US-0001** delivers the deployable platform foundation: Docker Compose stack, external PostgreSQL mirror, read-only Firefly connector, OIDC-protected React UI shell, sync scheduler, and minimal Grafana provisioning. No forecasting, subscription detection, or analytics dashboards in this story.

**Firefly read-only guarantee (explicit):** The Firefly Connector issues **HTTP GET requests only** to Firefly `/api/v1/*` endpoints. No POST, PUT, PATCH, or DELETE calls are permitted. Enforcement is via a typed HTTP client wrapper with method allowlist, integration-test assertion on outbound traffic, and optional audit log of every Firefly request (method, path, timestamp). Firefly remains the sole transaction source of truth; Flow Finance AI never mutates Firefly data (per R-0001, DEC-0004).

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

## US-0017 — README living-doc expansion and troubleshooting (post-US-0016)

**Status:** Architecture complete (2026-06-09)  
**Discovery:** `discovery-20260609-us0017` in `handoffs/archive/po-to-tl-pack-20260606-v.md`  
**Research:** [R-0078](research.md#r-0078--us-0017-readme-omniflow-smoke-templates-h3-layout-validate_doc_profile-gates) (extends [R-0066](research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance), [R-0067](research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks))  
**Decisions:** **DEC-0070** extension (US-0017 layout + per-segment maintenance); **DEC-0059** split layout preserved  
**Sprint:** `/quick` recommended (~5–6 tasks) under `SPRINT_MAX_TASKS` (12)  
**Acceptance:** `docs/product/acceptance.md` § US-0017 (5 rows)  
**Orchestrator:** `auto-20260609-us0017-001`

### Problem

US-0016 shipped the root README split layout and phase-boundary maintenance hooks, but **Examples** remain localhost-centric and **Limitations** lack operator troubleshooting depth. Post-Q0020 omniflow smoke paths (sync, recompute, analytics routes, exchange sanity) and BUG-0013 symptom distinctions live only in runbook §23 and `sprints/quick/Q0020/uat.md`. Maintenance hooks do not yet require **each** closed US/BUG in a **release segment** to receive a Product status bullet.

`isolation_scope`: artifact + repo source only; no host `.env` / secrets read; no application code.

### Architecture contract (DEC-0070 US-0017 extension)

```text
US-0017
├── E1 — Omniflow smoke H3 (P0)
│   └── README.md ## Examples → ### Omniflow smoke (external profile); R-0078 §2 curl block
├── E2 — Troubleshooting H3 (P0)
│   └── README.md ## Limitations → ### Troubleshooting; gate sequence + 6-row symptom table (R-0078 §3)
├── E3 — Product status verify (P0)
│   └── AC-3 verify-only — post-Q0020 refresh already lists US-0015, BUG-0013, US-0013–0016
├── E4 — Developer shard maintenance (P0)
│   └── docs/developer/README.md — per-segment Product status wording (R-0078 §5)
├── E5 — Runbook maintenance delta (P0)
│   └── runbook § README maintenance — per-segment hook wording + release-segment definition
└── E6 — Validator gate (P0)
    └── validate_doc_profile --repo . --no-template-parity exit 0 after all edits
```

**Out of scope:** Analytics code; dedicated `## Troubleshooting` H2; full runbook §23 inline; per-commit README automation; new DEC-xxxx (extends DEC-0070 only).

### E1 — Omniflow smoke H3 (frozen)

| Contract | Value |
|----------|-------|
| Placement | `### Omniflow smoke (external profile)` under `## Examples` |
| Host | `https://financegnome.omniflow.cc` (note `TRAEFIK_HOST` override in operator `.env`) |
| Edge auth | Placeholder `-u '<basic-auth-user>:<pass>'` per runbook § Omniflow AC-6 — **never** commit credentials |
| API auth | One-liner: `/api/v1/*` requires OIDC session or `AUTH_DEV_BYPASS=true`; matrix in runbook |
| Content | R-0078 §2 copy-paste block: health, sync status/entities, `POST /api/v1/sync/trigger` full mode, `GET /api/v1/forecast/meta`, `GET /api/v1/wealth` crypto probe, Grafana embed health |
| Routes | Six `/analytics/{slug}` smoke focuses (extend existing table or `OMNI` prefix note) |
| Gates | One-liner: **BACKEND_FRONTEND_DEPLOY** → **GRAFANA_PROVISIONING_RELOAD** → **FULL_FIREFLY_SYNC** + recompute; link runbook §23 |
| Anti-pattern | Do not duplicate full runbook §23 table in README |

**Alternatives rejected:** dedicated omniflow H2 (H2 budget); localhost-only examples retained (fails AC-1).

### E2 — Troubleshooting H3 (frozen)

| Contract | Value |
|----------|-------|
| Placement | `### Troubleshooting` under `## Limitations` (not a new root H2) |
| Lead | Operator gate sequence (same three gates as E1) |
| Body | 6-row symptom table per R-0078 §3 / Q0020 uat.md |
| Distinction (AC-2) | Empty Grafana SQL panels after gates = data/deploy defect; **ML unavailable** banner = expected degraded mode (US-0013 / DEC-0049) |
| Deep link | runbook §23 for row-level detail; BUG-0013 verdicts cited inline where helpful |

**Alternatives rejected:** dedicated `## Troubleshooting` H2 for `(both, balanced)` profile ([R-0078](research.md#r-0078--us-0017-readme-omniflow-smoke-templates-h3-layout-validate_doc_profile-gates) §1); Limitations bullets only without H3 subsection.

### E3 — Product status (verify-only)

Post-Q0020 `/refresh-context` already lists **US-0015**, **BUG-0013**, and **US-0013–0016** in `### Product status`. Execute verifies AC-3; append bullets only if the release segment closes additional US/BUG ids before story close.

### E4/E5 — Per-segment maintenance (frozen)

**Release segment** = the target sprint id (`Sxxxx`), quick task id (`Qxxxx`), or paired intake batch scope for the current `/release` or `/refresh-context` run.

| Surface | Delta |
|---------|-------|
| Runbook § README maintenance | Release hook: for **each** US/BUG → **DONE**/**CLOSED** in the **current release segment**, append one Product status bullet |
| Runbook § README maintenance | Refresh hook: when segment closed **one or more** US/BUG since prior refresh, verify **each** closed id appears in Product status |
| `docs/developer/README.md` | Quality gates / Workflow — same per-segment wording; pointer to runbook § README maintenance |

Preserves DEC-0070 phase-boundary cadence; tightens vague "closed items" language from US-0016.

### E6 — Validator gate (frozen)

| Check | Requirement |
|-------|-------------|
| Command | `python scripts/validate_doc_profile.py --repo . --no-template-parity` |
| Profile | `(both, balanced)` unchanged |
| Layout | No new root H2; H3 additions allowed; no `DEV_*` in root |
| Execute | Non-zero exit → fail closed; remediation → runbook § README maintenance |

### File touch list (frozen)

| Path | Task | Change |
|------|------|--------|
| `README.md` | E1, E2, E3 | H3 subsections; verify Product status |
| `docs/developer/README.md` | E4 | Per-segment maintenance wording |
| `docs/engineering/runbook.md` | E5 | § README maintenance hook deltas + release-segment definition |

**No touch:** Application source, compose, `template/` tree, analytics dashboards, CI structural changes (validator command unchanged).

### Validation strategy

| Check | Type | Pass criteria |
|-------|------|---------------|
| AC-1 Omniflow Examples | Manual + validator | H3 with external-profile curls, routes, exchange sanity |
| AC-2 Troubleshooting | Manual | H3 with gates, symptom table, ML-off vs data-missing |
| AC-3 Product status | Manual verify | US-0015 + post-US-0016 closures present |
| AC-4 Maintenance hooks | Doc review | Developer README + runbook per-segment wording |
| AC-5 Validator / layout | CI + local | `validate_doc_profile --no-template-parity` exit 0; H2 budget preserved |

### Risks

| Risk | Mitigation |
|------|------------|
| Dual auth confusion (Traefik vs OIDC) | Explicit edge vs API note in E1 H3 ([R-0078](research.md#r-0078--us-0017-readme-omniflow-smoke-templates-h3-layout-validate_doc_profile-gates) §2) |
| Placeholder hygiene | Angle-bracket placeholders only; no credential-like literals |
| Segment definition drift | Architecture + runbook one-liner for release segment scope |
| README length creep | Cap prose; link runbook §23; symptom table not full duplicate |
| Stale Product status | Existing DEC-0070 release/refresh fail-closed validator |

### Decisions (US-0017)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0070 | US-0017 extension | H3 Omniflow smoke + Troubleshooting; per-segment Product status maintenance; validator unchanged |

**No new DEC-xxxx** — layout and maintenance deltas are normative extensions of DEC-0070 ([R-0078](research.md#r-0078--us-0017-readme-omniflow-smoke-templates-h3-layout-validate_doc_profile-gates) architecture carry).

Full record: `decisions/DEC-0070.md` (US-0017 extension subsection)

### Acceptance mapping

| AC | Architecture slice | Verify |
|----|-------------------|--------|
| AC-1 | E1 | Omniflow smoke H3 content review |
| AC-2 | E2 | Troubleshooting H3 + distinction review |
| AC-3 | E3 | Product status verify (append only if new closures) |
| AC-4 | E4, E5 | Developer README + runbook per-segment hooks |
| AC-5 | E6 | `validate_doc_profile --no-template-parity` exit 0 |

### Next phase

`/sprint-plan` or `/quick` — decompose 5 acceptance criteria; expect ~5–6 doc-only tasks (E1–E6). Single quick task under `SPRINT_MAX_TASKS` (12).

---

## BUG-0014 — Post-rebuild omniflow cluster (ML sidecar, crypto display, Grafana, planning)

**Status:** architecture complete (2026-06-09)  
**Discovery:** `discovery-20260607-bug0014` in `handoffs/po_to_tl.md`  
**Research:** [R-0079 §6](research.md#r-0079--bug-0014-post-rebuild-omniflow-ml-sidecar-crypto-display-grafana-planning)  
**Decisions:** **DEC-0081** (AQ holdings + FX); **DEC-0082** (AS active-plan delete guard); **DEC-0083** (AS target_type UI); extends **DEC-0064**, **DEC-0066**, **DEC-0076**, **DEC-0080**; **no DEC-0064 amend**  
**Sprint:** `/quick` **Q0022** (recommended)  
**Acceptance:** `docs/product/acceptance.md` rows **AO**–**AT** (execute: **AO1**, **AP2** conditional, **AQ1**/**AQ2**, **AS1**/**AS2**, **V1**; **AO**/**AT**/**AR** ops/verify-only)  
**Related:** BUG-0013 DONE (DEC-0080); US-0013 DONE (DEC-0076); US-0014 DONE (delete plan out of epic scope)

### Symptom chain (frozen)

Post-rebuild operator cluster on `financegnome.omniflow.cc` decomposes to **three ops gates**, **three code/UX gaps**, and **one verify-only item** — not a single regression.

| Sub | Verdict | Root cause | Execute |
|-----|---------|------------|---------|
| **AO** | CONFIRMED (ops) | `stats-forecast` not running → `sidecar_unavailable`; Grafana static US-0013 banner misleading | Ops gate; **AO1** copy |
| **AT** | CONFIRMED (ops) | Rebuild two-service only; sidecar on external profile per DEC-0076 | Runbook smoke — **no code task** |
| **AP** | CONFIRMED (code residual, gated) | Subtotal sums `market_value_eur` only; wallet row NULL or pre-Q0020 image | **AP1** gate; **AP2** if priced but subtotal 0 |
| **AQ** | CONFIRMED (product gap) | `unpriced_assets` never wired; table priced-only | **AQ1** + **AQ2** (DEC-0081) |
| **AR** | LIKELY (ops/stale) | Time range / account mismatch; BUG-0013 AI refuted acct **114** | **V1** verify; **AR1** only if API≠Grafana |
| **AS** | CONFIRMED (UI gap) | No plan delete UI; invalid `account` target_type | **AS1** (DEC-0082) + **AS2** (DEC-0083) |

**Live probe (2026-06-07):** `ml_skipped_reason: sidecar_unavailable`; `crypto.subtotal_eur: 0`, `holdings_count: 7`, `unrealized_eur: 411.74`, `holdings_top: []`; no `stats-forecast` container.

`isolation_scope`: artifact + repo source reads; no host `.env` / `.env_prod` secrets read.

### Operator gates (mandatory before execute / V1)

1. **BACKEND_FRONTEND_DEPLOY** — Q0020 image (`DEC-0080`) on rebuilt host.
2. **Three-service compose** — `flow-finance-ai`, `grafana`, `stats-forecast` per **DEC-0076** when `FORECAST_ML_ENABLED=true`.
3. **Full Firefly sync** — not exchanges-only.
4. **Forecast recompute** — baseline on acct **114**.
5. **AP1 SQL probe** — futures wallet row priced before **AP2** code (see below).

### Fix slices

```text
BUG-0014
├── Ops (no execute — runbook / operator)
│   ├── AO — start stats-forecast + Full sync + recompute
│   └── AT — three-service external compose smoke (DEC-0076)
├── AO1 — Grafana ML banner dual-scenario copy (extends DEC-0066 / DEC-0076)
├── AP — deploy gate then optional hardening
│   ├── AP1 — verify Q0020 deploy + SQL wallet row (gate)
│   └── AP2 — wealth/service.rs defensive subtotal (conditional)
├── AQ — DEC-0081
│   ├── AQ1 — backend holdings_all + unpriced_assets + fx_incomplete
│   └── AQ2 — WealthPage table + unified FX banner
├── AS — DEC-0082 + DEC-0083
│   ├── AS1 — delete plan UI + active-plan 409 guard
│   └── AS2 — target_type select + help copy
├── AR — verify-only in V1; AR1 conditional
└── V1 — verify-work omniflow smoke (AO–AT acceptance rows)
```

**Deploy order:** Single release bundle AO1 + AQ + AS + conditional AP2; operator gates before V1.

**Out of scope:** Tier-2 `ExchangePriceBook`; Grafana dynamic Postgres variable for ML banner (option B deferred); `target_type` enum expansion; AR1 unless verify gate fails.

### AP — Crypto subtotal €0 (deploy gate + conditional AP2)

#### AP1 verify gate (mandatory before AP2)

```sql
SELECT product_type, asset, quantity, market_value_eur, unrealized_pnl_eur
FROM exchange_holdings WHERE exchange_id = 'bitunix' ORDER BY product_type, asset;
```

| AP1 outcome | Next |
|-------------|------|
| No `futures` row or all `market_value_eur` NULL after deploy + Full sync | **Ops only** — redeploy Q0020 + Full sync; **no AP2** |
| `futures` row priced, API `subtotal_eur > 0` | **AP closed** — was deploy/stale |
| `futures` row priced, API `subtotal_eur = 0` | **AP2** execute |

#### AP2 contract (conditional — frozen)

Preserve **DEC-0064** / **DEC-0080** subtotal rules — **do not** merge linear unrealized into `crypto.subtotal_eur`.

| Hardening | Contract |
|-----------|----------|
| Defensive subtotal | If `sum(market_value_eur) == 0` **and** `pnl.crypto_value_eur > 0`, use portfolio snapshot `crypto_value_eur` for subtotal display |
| Exchange card copy | Annotate `holdings_count` — e.g. "N holdings (M priced)" when counts diverge |

**Files:** `backend/src/wealth/service.rs` only.

**Alternatives rejected:** Linear notional in subtotal (DEC-0064 violation); re-open AN1 in `bitunix.rs` when AP1 shows NULL wallet (ops/deploy issue).

**Risks:** Defensive snapshot masks stale holdings table — log when fallback used; AP1 must still pass for acceptance.

### AQ — All holdings + unified FX (DEC-0081)

See `decisions/DEC-0081.md`. Summary:

- **`holdings_all`** capped at 50 with native qty, `value_eur`, `unrealized_pnl_eur`, `native_unit`
- **`fx_incomplete`** = PnL flag OR non-empty `unpriced_assets`
- Linear rows visible with unrealized EUR; excluded from subtotal (DEC-0064)

**Files:** `backend/src/wealth/{service,types}.rs`, `frontend/src/pages/WealthPage.tsx`, `frontend/src/lib/api.ts`

**Risks:** Empty EUR column for tier-2 alts — banner + copy; defer price book.

### AS — Plan delete + target_type (DEC-0082, DEC-0083)

**AS1:** `DELETE` blocked for `is_active` plan (409); React delete with confirm modal; DEC-0077 error surface.

**AS2:** Remove invalid `account` option; expose `category`, `custom_label`, `allocation_target` with help copy.

**Files:** `backend/src/api/plans.rs` or `plan/service.rs`, `frontend/src/pages/PlanningPage.tsx`

**Risks:** Single-plan operator must create second plan before delete — document in modal.

### AO1 — Grafana ML banner (extends DEC-0066)

**Option A (MVP — frozen):** Replace `forecast-horizons.json` panel id **13** static markdown with **dual-scenario** copy:

1. ML not configured — set `FORECAST_ML_ENABLED` + US-0013 runbook link
2. ML configured but sidecar unreachable — start `stats-forecast` per DEC-0076 three-service compose

**Reject option B** (Postgres `ml_skipped_reason` variable) this sprint — stale until recompute; provisioning complexity.

**Files:** `grafana/provisioning/dashboards/analytics/forecast-horizons.json` panel **13** only.

React `ForecastPage.tsx` already distinguishes `sidecar_disabled` vs `sidecar_unavailable` (DEC-0066) — no change.

### AR — Cashflow Grafana verify gate

**No execute unless AR1 trigger.** Operator steps (embedded in V1):

| Step | Pass criterion |
|------|----------------|
| 1 | Three-service compose + Full sync + recompute |
| 2 | `$account_id` = **114** (funded Giro) |
| 3 | Time range `now-30d` → `now+6M` (dashboard default) |
| 4 | `GET /api/v1/forecast/daily?account_id=114` — non-zero balances |
| 5 | Panel SQL (cashflow id 1) returns rows for same computation |

**AR1 trigger:** Step 4 passes **and** step 5 zero — Grafana variable/computation mismatch only.

**Risks:** June 2028 screenshot is out-of-horizon — not SQL regression; premature AR1 duplicates BUG-0013.

### Task table (sprint-plan input)

| ID | Sub | Task | Files | Priority | Gate |
|----|-----|------|-------|----------|------|
| **AO1** | AO | Dual-scenario ML banner panel 13 | `forecast-horizons.json` | P0 | — |
| **AP2** | AP | Defensive subtotal + count annotation | `wealth/service.rs` | P0 | **AP1** priced + subtotal 0 |
| **AQ1** | AQ | `holdings_all` + wire `unpriced_assets` / `fx_incomplete` | `wealth/service.rs`, `types.rs` | P0 | — |
| **AQ2** | AQ | Holdings table + unified FX banner | `WealthPage.tsx`, `api.ts` | P0 | after AQ1 |
| **AS1** | AS | Delete plan UI + active 409 guard | `plans.rs`/`service.rs`, `PlanningPage.tsx` | P0 | — |
| **AS2** | AS | Target-type select + help copy | `PlanningPage.tsx` | P1 | — |
| **AR1** | AR | Cashflow Grafana variable fix | `cashflow.json` | P2 | API≠Grafana only |
| **V1** | all | verify-work smoke post deploy + gates | acceptance AO–AT | P0 | all execute |

**Waived / ops-only:** AO runtime (start sidecar), AT (runbook), AP1 (operator SQL probe — prerequisite not sprint task), AR default path.

**Count:** 6 mandatory execute (AO1, AQ1, AQ2, AS1, V1) + 1 conditional (AP2) + 1 optional (AS2 P1) + 1 deferred (AR1) → **7–8 tasks** ≤ `SPRINT_MAX_TASKS` (12) → **`/quick` Q0022**.

### Codebase map (BUG-0014 slice)

| Path | Role | Touch |
|------|------|-------|
| `grafana/.../forecast-horizons.json` | ML static banner | AO1 |
| `backend/src/wealth/service.rs` | Subtotal + holdings_all | AP2, AQ1 |
| `backend/src/wealth/types.rs` | API types | AQ1 |
| `frontend/src/pages/WealthPage.tsx` | Crypto table + banner | AQ2 |
| `frontend/src/pages/PlanningPage.tsx` | Delete plan + target_type | AS1, AS2 |
| `backend/src/api/plans.rs` | Active delete guard | AS1 |
| `grafana/.../cashflow.json` | Cashflow panels | AR1 only |

### Decisions (BUG-0014)

| ID | Sub | Contract |
|----|-----|----------|
| **DEC-0081** | AQ | `holdings_all` + unified `fx_incomplete` from PnL; DEC-0064 subtotal preserved |
| **DEC-0082** | AS1 | 409 on active plan delete; React confirm + DEC-0077 errors |
| **DEC-0083** | AS2 | Remove invalid `account`; expose DB enum values + help copy |

**No new DEC for:** AO1 (extends DEC-0066/DEC-0076); AP2 (extends DEC-0080); AR (verify gate).

### Risks

| Risk | Mitigation |
|------|------------|
| AP2 without AP1 masks deploy gap | AP1 mandatory gate; log defensive fallback |
| AQ tier-2 alts still unpriced | DEC-0081 banner + list; defer price book |
| Delete active plan confusion | DEC-0082 409 + disabled UI |
| AR false positive code sprint | AR1 only on API≠Grafana; default V1 verify |
| Sidecar cold start | DEC-0076 runbook re-sync after healthcheck |

### Next phase

`/sprint-plan` — materialize `sprints/quick/Q0022/` from task table; then `/plan-verify` → `/execute`.

# BUG-0015 — Confirmed subscriptions reappear as pending after rebuild

**Status:** architecture complete (2026-06-07)  
**Discovery:** `discovery-20260607-bug0015` in `handoffs/archive/po-to-tl-pack-20260607-h.md`  
**Research:** [R-0081](research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild), [R-0082](research.md#r-0082--card-billing-descriptor-normalization-for-subscription-identity)  
**Decisions:** **DEC-0084** (card `payee_key` normalization); **DEC-0085** (payee+interval confirm inheritance); **DEC-0086** (interval tolerance + fingerprint rotation); extends **DEC-0071**, **DEC-0072**, **DEC-0013**, **DEC-0015**; **no DEC-0071 amend**  
**Sprint:** `/quick` **Q0023** (recommended)  
**Acceptance:** `docs/product/acceptance.md` rows **AU**–**AW** (execute: **AU1**–**AU4**, **V1**)  
**Related:** BUG-0008 DONE (DEC-0071/0072); BUG-0014 DONE (rebuild context); US-0003 (confirm-once contract)

### Symptom chain (frozen)

Operator confirms Cursor (€17.18/mo) and Apple (€9.99/mo) subscriptions on omniflow; after app container rebuild + Full sync, the same merchants reappear as **pending** with Confirm/Reject. Discovery locks **H1 fingerprint drift** as primary; H2 ops gate; H3 refuted primary; H4 subsumed.

| Hypothesis | Verdict | Mechanism | Fix layer |
|------------|---------|-----------|-----------|
| **H1** fingerprint drift | **LIKELY PRIMARY** | `compute_fingerprint(payee_key, interval_days, median_amount)` — any input change → new hash → fresh `pending` INSERT; ON CONFLICT preserve only on same fingerprint | Layer 1 + 2 |
| **H2** DB ephemeral | **UNLIKELY sole** | External postgres persists; merchant-specific re-prompt ≠ total wipe | Ops gate only |
| **H3** alert/UI desync | **REFUTED primary** | Pending cards require `status=pending` in DB; alerts secondary | Satisfied by AU2–AU3 |
| **H4** detection re-run | **Subsumed by H1** | `confirmed_fps` skip works when fingerprint stable | No separate fix |

`isolation_scope`: artifact + repo source reads; no host `.env` / `.env_prod` secrets read.

### Operator gates (mandatory before V1)

1. **BACKEND_FRONTEND_DEPLOY** — ship Q0023 image before rebuild smoke.
2. **Postgres persistence probe** — immediately after app rebuild, **before** Full sync:

```sql
SELECT status, COUNT(*) FROM subscription_patterns GROUP BY status;
SELECT fingerprint, status, payee_key, interval_days, current_amount
FROM subscription_patterns
WHERE payee_key ILIKE '%cursor%' OR payee_key ILIKE '%apple%'
ORDER BY updated_at DESC;
```

| Outcome | Action |
|---------|--------|
| Zero `confirmed` rows after rebuild (no operator action) | **Ops** — volume/DB target issue (H2); do not execute V1 until resolved |
| `confirmed` rows present; duplicate fingerprints or drifted `payee_key` after Full sync | **Execute** AU1–AU4 path |
| Single confirmed row per merchant; still pending in UI | Reopen discovery — unlikely per H3 refutation |

3. **Full Firefly sync** — not exchanges-only; allow subscription detection phase to complete.

### Two-layer fix bundle (frozen)

```text
BUG-0015
├── Layer 1 — DEC-0084 (reduce drift at source)
│   └── AU1 — card billing descriptor rules in payee_key()
├── Layer 2 — DEC-0085 + DEC-0086 (confirm-once semantics)
│   ├── AU2 — load_confirmed/rejected payee+interval maps; merge upsert
│   ├── AU3 — detection skip + merge path; rejection by payee+interval
│   └── AU4 — mark_stale_inactive keyed by payee+interval (not fingerprint-only)
└── V1 — verify-work: confirm → rebuild app → Full sync → AU/AV/AW smoke
```

**Deploy order:** Single release bundle AU1–AU4; operator gates before V1.

**Out of scope:** Reopen BUG-0008; alert-only dedup as primary fix (option E rejected); merchant identity table (option F); postgres volume runbook beyond H2 gate; UI changes (H3 refuted); drop amount from fingerprint globally (option B rejected as sole fix).

### AU1 — Card billing `payee_key` normalization (DEC-0084)

See `decisions/DEC-0084.md`. Summary:

| Rule | Contract |
|------|----------|
| Asterisk split | Token before `*` when present (`DBA*Plan` → `dba`) |
| Comma memo | Leftmost segment before `,` (case-insensitive trim) |
| Billing roots | Collapse `apple.com/bill`, `itunes.com`, `apple.com/bill itunes` → `apple` |
| Domain tail | Known SaaS: strip `.com`, `/bill` suffix after left-segment extract |

**Files:** `backend/src/recurrence/normalize.rs`, unit tests in same module.

**Shared surface:** `recurrence` module used by forecast + subscriptions (**DEC-0013**) — rules must be conservative; Layer 2 catches residual drift.

**Risks:** Over-merge distinct products under same biller (e.g. two Apple subs) — mitigated by **DEC-0086** payee+interval composite, not payee-only.

### AU2 — Repository payee+interval maps and merge upsert (DEC-0085, DEC-0086)

#### New repository contracts (frozen)

| Function | Returns | Purpose |
|----------|---------|---------|
| `load_confirmed_payee_intervals()` | `HashMap<(payee_key, interval_days), ConfirmedRow>` | Detection skip + merge target lookup |
| `load_rejected_payee_intervals()` | `HashSet<(payee_key, interval_days)>` | Extend rejection beyond fingerprint-only |
| `merge_confirmed_pattern(existing_id, group, fingerprint, …)` | `PendingUpsertOutcome` | In-place refresh of confirmed row |
| `interval_matches(stored, detected)` | `bool` | **DEC-0086** ±3 day tolerance |

**Index:** Add migration `CREATE INDEX IF NOT EXISTS idx_subscription_patterns_payee_status ON subscription_patterns (payee_key, status)` — supports lookup without full table scan.

#### Merge upsert contract (confirmed match)

When `(normalized_payee_key, interval_matches)` hits a **confirmed** row:

1. **UPDATE** existing row by `id` — refresh `current_amount`, `confidence_pct`, `display_name`, `last_seen_at`, `detection_run_id`.
2. **Rotate fingerprint in-place** (**DEC-0086**) — set `fingerprint = newly_computed` on same row; preserve `status = confirmed`, `confirmed_at`.
3. **Do not** INSERT new `pending` row; **do not** emit `new_detection` alert.
4. Re-link transactions to existing `pattern_id`.

When match hits **rejected** row: **skip** emit (same as fingerprint rejection today).

When no match: fall through to existing `upsert_pending_pattern` (pending path unchanged).

**Orphan cleanup (best-effort):** If pre-fix duplicate `pending` row exists for same merchant with different fingerprint, AU3 skip+merge prevents new alerts; optional one-time inactive mark deferred — not blocking V1.

**Files:** `backend/src/subscriptions/repository.rs`, new migration for index.

**Alternatives rejected:**

| Alternative | Why |
|-------------|-----|
| Skip without merge (option D) | Stale amount/interval on confirmed card until manual refresh — acceptable fallback only if Layer 1 tests sufficient; **not** primary |
| Alert dedup by payee (option E) | Masks AU failure; violates confirm-once |
| Drop amount from fingerprint globally (option B) | Tier-change collisions; breaks price-change identity |

### AU3 — Detection skip + merge path (DEC-0085)

**`run_candidates` contract change:**

```text
for each group:
  fingerprint = compute_fingerprint(payee_key, interval_days, median_amount)
  if fingerprint in rejections or confirmed_fps: continue  // retain exact-match fast path
  if (payee_key, interval_days) in rejected_payee_intervals: continue
  if match confirmed_payee_intervals via interval_matches:
    merge_confirmed_pattern(...); continue  // no pending, no alert
  else:
    upsert_pending_pattern(...)  // existing pending path
```

**`service.rs`:** Load `confirmed_payee_intervals` and `rejected_payee_intervals` at run start; pass to pipeline alongside existing fingerprint sets.

**Files:** `backend/src/subscriptions/detection.rs`, `backend/src/subscriptions/service.rs`.

**Risks:** Order of checks — confirmed merge before pending upsert prevents duplicate INSERT.

### AU4 — Stale inactive by payee+interval (DEC-0085)

Today `mark_stale_inactive` compares `active_fps` fingerprint set only — drifted fingerprint marks confirmed row stale incorrectly.

**Contract:** Build `active_payee_intervals` from current detection groups (normalized `payee_key` + `interval_days`). For each confirmed pattern, if `(payee_key, interval_matches(interval_days))` **not** in active set **and** gap > `2 × interval_days`, call `mark_inactive`.

Wire `mark_stale_inactive` into `run_detection` after candidates (currently defined but unwired).

**Files:** `backend/src/subscriptions/detection.rs`, `service.rs`.

### V1 — verify-work rebuild smoke

| Step | Acceptance |
|------|------------|
| Confirm Cursor + Apple on omniflow | **AU** baseline |
| Rebuild `flow-finance-ai` (+ grafana if operator scope) only | postgres untouched |
| Run H2 SQL probe before Full sync | ops gate |
| Full sync + detection | **AV** — no duplicate pending; confirmed status preserved |
| Check `/subscriptions` tabs + unread subscription alerts | **AW** — no spurious `new_detection` for confirmed merchants |
| OIDC regression | per acceptance AW footnote |

**Evidence template:** `sprints/quick/Q0023/uat.md` (materialize at sprint-plan).

### Task table (sprint-plan input)

| Task | AC | Surface | Priority | Depends |
|------|-----|---------|----------|---------|
| **AU1** | AU, AV | `recurrence/normalize.rs` + tests | P0 | — |
| **AU2** | AU, AV | `subscriptions/repository.rs` + migration index | P0 | AU1 |
| **AU3** | AU, AV, AW | `subscriptions/detection.rs`, `service.rs` | P0 | AU2 |
| **AU4** | AV | `detection.rs` stale map + `service.rs` wire | P0 | AU2 |
| **V1** | AU–AW | verify-work rebuild smoke | P0 | AU1–AU4 + deploy |

**Count:** 5 tasks ≤ `SPRINT_MAX_TASKS` (12) → **`/quick` Q0023**; no split.

### Codebase map (BUG-0015 slice)

| Path | Role | Touch |
|------|------|-------|
| `backend/src/recurrence/normalize.rs` | Shared payee identity | AU1 |
| `backend/src/subscriptions/repository.rs` | Pattern persistence | AU2 |
| `backend/src/subscriptions/detection.rs` | Detection pipeline | AU3, AU4 |
| `backend/src/subscriptions/service.rs` | Orchestration | AU3, AU4 |
| `backend/migrations/` | Index on payee+status | AU2 |

### Decisions (BUG-0015)

| ID | Layer | Contract |
|----|-------|----------|
| **DEC-0084** | 1 | Card billing descriptor normalization in `payee_key()` per R-0082 |
| **DEC-0085** | 2 | Payee+interval confirm/reject inheritance: skip + merge upsert; stale map by payee+interval |
| **DEC-0086** | 2 | ±3-day `interval_matches`; in-place fingerprint rotation on confirmed merge; multi-sub = payee+interval composite |

**No new DEC for:** V1 (verify gate); H2 ops probe (operator only).

### Risks

| Risk | Mitigation |
|------|------------|
| Over-merge Apple/Cursor descriptors (AU1) | Payee+interval composite (DEC-0086); operator V1 on known merchants |
| Two subs same merchant different intervals | `interval_days` in composite key — both persist |
| Fingerprint UNIQUE violation on rotate | UPDATE same row by `id`; transaction boundary |
| Layer 1 changes forecast grouping | Conservative rules; monitor; DEC-0013 shared module tests |
| Pre-fix orphan pending rows | Skip+merge stops new alerts; optional cleanup deferred |
| H2 false negative | Mandatory SQL probe in UAT before Full sync |

### Next phase

`/sprint-plan` — materialize `sprints/quick/Q0023/` from task table; then `/plan-verify` → `/execute`.

# US-0018 — Category filters & expense trend analytics

**Status:** Architecture complete (2026-06-08)  
**Discovery:** `discovery-20260608-us0018` in `handoffs/archive/po-to-tl-pack-20260608.md`  
**Research:** [R-0083](research.md#r-0083--us-0018-category-filters-expense-series-api--trend-analytics), [R-0080](research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake)  
**Decisions:** **DEC-0087** (expense-series API + uncategorized sentinel); **DEC-0088** (CategoryFilter + bar trend chart); **DEC-0089** (surface filter semantics + Grafana independence); **DEC-0090** (index deferral policy)  
**Depends on:** BUG-0006 DONE (`category_id` ingest), US-0011 (analytics embed), US-0015 (bucket mapping — AC-6 regression guard), DEC-0007 (forecast engine unchanged in MVP)  
**Sprint:** **S0017** recommended (or single sprint ≤12 tasks)  
**Acceptance:** `docs/product/acceptance.md` § US-0018 (AC-1..AC-6)  
**Spec-pack:** `docs/engineering/spec-pack/US-0018-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**User guide:** `docs/user-guides/US-0018.md` (`USER_GUIDE_MODE=1`; execute publishes content)

### Problem

Operators need **category-scoped visibility** across product surfaces and Grafana — monthly expense trends with explicit uncategorized handling — without forking the DEC-0007 forecast engine or plan compare recompute. Mirror ingest and period `aggregates_by_category` exist (BUG-0006); **monthly per-category series**, public REST routes, shared React filter, trend chart, and Grafana `$category` are missing.

| AC | Discovery verdict | Architecture slice |
|----|-------------------|-------------------|
| AC-1 Category filter contract | **Gap** | S2 + S3 + S4 |
| AC-2 Monthly series API | **Gap** | **S1 primary** |
| AC-3 Trend chart UI | **Gap** | S2 primary |
| AC-4 Performance insight | **Gap** | S1 (server `summary`) + S2 |
| AC-5 Mirror fidelity | Partial (ingest done) | S1 (`__uncategorized__`) |
| AC-6 Regression | Verify | S5 smoke |

`isolation_scope`: artifact + repo source reads; no host `.env` / secrets read.

### Research gates resolved

| Gate | Decision | Alternative rejected |
|------|----------|---------------------|
| Multi-category overlay | **Defer** — single series (DEC-0088) | ≤3 overlay series on one chart |
| Trend chart type | **Bar default** (DEC-0088) | Line default; line toggle = stretch P2 |
| Grafana ↔ SPA sync | **Independent filters** (DEC-0089) | iframe `category_id` URL sync |
| Forecast filter depth | **Display-only actuals panel**; household forecast unchanged (DEC-0089) | Full category forecast re-projection |
| Uncategorized sentinel | **`__uncategorized__` query token** (DEC-0087) | Separate `/uncategorized` route |
| Planning compare filter | **UI-scoped actuals widget**; compare API unchanged (DEC-0089) | Server-side `build_compare_metrics` per category |
| Category index | **Defer** unless `EXPLAIN` >50 ms (DEC-0090) | Ship `idx_transactions_category_date` in US-0018 |

### System context

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Browser — ForecastPage / PlanningPage / WealthPage                         │
│    CategoryFilter (shared) ──▶ category_id state (single-select MVP)        │
│    CategoryTrendChart (bar, ECharts) ◀── expense-series summary (MoM)       │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ GET /api/v1/categories
                                │ GET /api/v1/categories/expense-series
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum)                                                     │
│    CategoriesService + TransactionsRepository::expense_series_by_month (NEW)│
│    month_spine + LEFT JOIN transactions (R-0083 §1)                         │
│    label_uncategorized_categories naming reuse                              │
│                                                                             │
│  Forecast monthly API ──▶ UNCHANGED projection (DEC-0007 / US-0015)         │
│  Planning compare API ──▶ UNCHANGED (DEC-0019 overlay)                      │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ mirror `transactions` + `categories`
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  Grafana cashflow + budgets (US-0011 embed)                                 │
│    $category variable (independent of SPA state)                            │
│    Panel SQL: ('${category}' = '' OR t.category_id = '${category}')         │
└─────────────────────────────────────────────────────────────────────────────┘
```

**AC-6 boundary:** No changes to US-0015 `bucket_sources` / AI inference path; no Firefly write-back; aggregate-only new REST (DEC-0032 posture).

### Architecture contract

```text
US-0018
├── S1 — Backend category APIs (P0)
│   ├── T1 — TransactionsRepository::expense_series_by_month (month spine SQL)
│   ├── T2 — GET /api/v1/categories (catalog + optional ?q=)
│   ├── T3 — GET /api/v1/categories/expense-series + summary (AC-2, AC-4, AC-5)
│   └── T4 — Unit/integration tests: spine zeros, __uncategorized__, 24-month cap
├── S2 — Shared React filter + trend chart (P0)
│   ├── T1 — components/category/CategoryFilter.tsx (single-select; All sentinel)
│   ├── T2 — components/category/CategoryTrendChart.tsx (bar default, ECharts)
│   ├── T3 — ForecastPage monthly tab integration (primary home)
│   └── T4 — MoM / best / worst callouts from API summary (AC-4)
├── S3 — Planning + wealth surfaces (P1)
│   ├── T1 — PlanningPage compare toolbar + CategoryTrendChart widget (actuals only)
│   └── T2 — WealthPage category spending subsection + filter
├── S4 — Grafana $category (P1)
│   ├── T1 — cashflow.json: $category variable + monthly category outflow panel
│   └── T2 — budgets.json: $category on Ist/deviation actual leg
├── S5 — Regression + docs (P1)
│   ├── T1 — OIDC external profile smoke (AC-6)
│   ├── T2 — docs/user-guides/US-0018.md
│   └── T3 — Optional EXPLAIN probe task (DEC-0090 gate)
└── V1 — verify-work omniflow smoke
```

**Out of scope:** Multi-category chart overlay; Grafana↔SPA bidirectional sync; category-scoped forecast re-projection (US-0019); materialized views; Firefly category editing.

### S1 — Expense-series API (DEC-0087)

#### Endpoints

| Method | Path | Contract |
|--------|------|----------|
| `GET` | `/api/v1/categories` | `{ categories: [{id, name}], truncated?: bool }` sorted by name; optional `?q=` with `MIN_CATEGORY_SEARCH_LEN=2`; cap **200** rows |
| `GET` | `/api/v1/categories/expense-series` | Query: `category_id` (**required**), `months` (default **12**, max **24**), optional `end` (default today) |

#### `category_id` values

| Value | SQL filter | Response |
|-------|------------|----------|
| Firefly mirror id | `t.category_id = $1` | `category_name` from `categories` |
| `__uncategorized__` | `t.category_id IS NULL` | `uncategorized: true`, `category_label: "Uncategorized"` |
| (invalid / unknown id) | — | **404** with documented empty-state guidance |

**Never** return a month series without bucket metadata when uncategorized is selected (AC-5).

#### Monthly spine SQL (frozen)

Per [R-0083 §1](research.md#r-0083--us-0018-category-filters-expense-series-api--trend-analytics): `generate_series` month spine + `LEFT JOIN transactions` on `date_trunc('month', t.date)`; outflow = `ABS(amount)` where `amount < 0`; inflow where `amount > 0`; explicit **€0** months in window.

Window: `$end = end param or today`; `$start = date_trunc('month', $end) - (months-1) * 1 month` — **not** data min/max (AC-3 labels).

#### `summary` object (AC-4)

Computed server-side on full spine:

| Field | Rule |
|-------|------|
| `mom_delta_pct` | Last two **calendar months in window** (zeros allowed) |
| `best_month` | Month with max `outflow_eur` |
| `worst_month` | Month with min `outflow_eur` among months with any activity, or min outflow if all zero |

#### Module placement

| Type | Path |
|------|------|
| `expense_series_by_month` | `backend/src/transactions/repository.rs` |
| `CategoriesService` | `backend/src/transactions/service.rs` (or new `categories/service.rs`) |
| Routes | `backend/src/api/categories.rs` (new), wire in `api/mod.rs` |

**Risks:** `date_trunc` on `date` may not use `idx_transactions_date` alone when filtering `category_id` — see DEC-0090.

### S2 — CategoryFilter & CategoryTrendChart (DEC-0088)

#### CategoryFilter contract

| Prop / behavior | Contract |
|-----------------|----------|
| Selection | **Single category** MVP; multi-select deferred |
| Sentinel `""` | **All categories** — toolbar surfaces only; hides trend chart or shows household hint |
| Data source | `GET /api/v1/categories`; combobox/search when `categories.length > 20` |
| Uncategorized | Explicit option value `__uncategorized__` in dropdown |
| Pattern | Clone `ForecastPage` account `<select>` for ≤20; native filter input above |

#### CategoryTrendChart contract

| Aspect | Choice |
|--------|--------|
| Chart type | **Bar** default (aligns with `MonthlyChart.tsx`, Finanzguru discrete months) |
| Library | ECharts (existing stack) |
| Data | `expense-series` `months[]` — `month` label + `outflow_eur` |
| Empty state | "No categorized spending in this period" when `transaction_count=0` for all months |
| Disabled state | No `category_id` selected → prompt to pick category |
| Line toggle | **Stretch P2** — not required for AC-3 |

#### Primary placement

`/forecast` **Monthly** tab: filter above stat cards; trend chart below `MonthlyChart` (household buckets unchanged per DEC-0089).

**Files:** `frontend/src/components/category/{CategoryFilter,CategoryTrendChart}.tsx`, `frontend/src/lib/api.ts`, `ForecastPage.tsx`

### S3 — Cross-surface semantics (DEC-0089)

| Surface | With category selected | Unchanged |
|---------|------------------------|-----------|
| **Forecast monthly** | `CategoryTrendChart` shows **filtered actuals** from expense-series | Income/Fixed/Variable stat cards + `MonthlyChart` remain **household** forecast (DEC-0007) |
| **Planning compare** | Toolbar `CategoryFilter` + adjacent **CategoryTrendChart** (actuals preview) | `GET` compare metrics / version table — **no** `category_id` query param |
| **Wealth overview** | New "Category spending" subsection: period total + trend link | Net worth / crypto totals household-level |
| **Grafana cashflow/budgets** | `$category` variable filters panel SQL | **Independent** of SPA selection — no iframe query sync |

**Planning rationale:** `build_compare_metrics` category fork is US-0019 scope; compare tab already surfaces `target_type=category` plan lines in table.

**Grafana `$category` variable (frozen):**

```json
{
  "name": "category",
  "type": "query",
  "query": "SELECT '' AS __value, 'All categories' AS __text UNION ALL SELECT c.firefly_id, COALESCE(c.name, c.firefly_id) FROM categories c ORDER BY 2"
}
```

Panel filter: `AND ('${category}' = '' OR t.category_id = '${category}')`

| Dashboard | Panel action |
|-----------|--------------|
| **cashflow** | New panel: monthly category outflow (`date_trunc` + sum abs negative) |
| **budgets** | Extend Ist/deviation **actual** CTE with category filter; planned leg household-only |

Default `category=''` preserves pre-US-0018 behavior.

### S4 — Performance policy (DEC-0090)

- **MVP:** no migration; sequential scan acceptable for ~900 rows × 24 months × single `category_id`
- **Gate task (optional):** `EXPLAIN ANALYZE` on operator mirror during execute; if **>50 ms**, add `CREATE INDEX idx_transactions_category_date ON transactions (category_id, date)` as follow-on migration task
- **Reject for MVP:** Timescale continuous aggregate; materialized monthly rollup table

### Task table (sprint-plan input)

| ID | Slice | Task | Files | Priority |
|----|-------|------|-------|----------|
| **C1** | S1 | `expense_series_by_month` repository + tests | `transactions/repository.rs` | P0 |
| **C2** | S1 | Categories routes + service | `api/categories.rs`, `api/mod.rs`, `service.rs` | P0 |
| **C3** | S2 | `CategoryFilter` + `CategoryTrendChart` | `components/category/*`, `api.ts` | P0 |
| **C4** | S2 | Forecast monthly integration | `ForecastPage.tsx` | P0 |
| **C5** | S3 | Planning compare widget | `PlanningPage.tsx` | P1 |
| **C6** | S3 | Wealth category subsection | `WealthPage.tsx` | P1 |
| **G1** | S4 | cashflow `$category` + panel | `cashflow.json` | P1 |
| **G2** | S4 | budgets `$category` + Ist filter | `budgets.json` | P1 |
| **D1** | S5 | User guide US-0018 | `docs/user-guides/US-0018.md` | P1 |
| **V1** | S5 | OIDC smoke + AC-1..AC-6 | `uat.md` | P0 |
| **P1** | S5 | EXPLAIN probe (conditional index) | migration optional | P2 |

**Count:** 10 mandatory/primary (C1–C4, G1, G2, V1) + 2 P1 (C5, C6, D1) + 1 conditional (P1) → **≤12** under `SPRINT_MAX_TASKS` → **single sprint S0017** (no split).

**Deploy order:** C1→C2→C3→C4 (vertical slice) ∥ G1/G2 after C2; C5/C6 after C3; V1 last.

### Codebase map (US-0018 slice)

| Path | Role | Touch |
|------|------|-------|
| `backend/src/transactions/repository.rs` | Monthly spine SQL | C1 |
| `backend/src/api/categories.rs` | REST routes | C2 |
| `backend/src/api/mod.rs` | Route registration | C2 |
| `frontend/src/components/category/*` | Filter + chart | C3 |
| `frontend/src/pages/ForecastPage.tsx` | Primary home | C4 |
| `frontend/src/pages/PlanningPage.tsx` | Compare widget | C5 |
| `frontend/src/pages/WealthPage.tsx` | Category subsection | C6 |
| `grafana/.../cashflow.json` | `$category` + panel | G1 |
| `grafana/.../budgets.json` | Ist category filter | G2 |

### Decisions (US-0018)

| ID | Topic | Contract |
|----|-------|----------|
| **DEC-0087** | Expense-series API | Month spine SQL; catalog + expense-series endpoints; `__uncategorized__` sentinel; server `summary` |
| **DEC-0088** | Filter + chart UX | Single-select MVP; bar default trend chart; defer multi-overlay |
| **DEC-0089** | Surface semantics | Forecast actuals-only side panel; planning widget; independent Grafana `$category` |
| **DEC-0090** | Index policy | Defer `category_id` index unless EXPLAIN >50 ms |

### Risks

| Risk | Mitigation |
|------|------------|
| Operators expect category filter to change forecast buckets | Copy/tooltip: household forecast unchanged; trend shows actuals (DEC-0089) |
| Planning compare confusion (filter vs plan lines) | Widget labeled "Actual spending trend"; compare table unchanged |
| Grafana vs SPA category mismatch | Document independent filters in user guide |
| Stale category id post-Firefly delete | Empty series + 404 on unknown id |
| 24-month query slow on large mirrors | DEC-0090 EXPLAIN gate; optional index task P1 |
| US-0015 regression | No `project.rs` / bucket_inference changes in US-0018 |

### Next phase

`/sprint-plan` — materialize **S0017** from task table; then `/plan-verify` → `/execute`.

# US-0019 — Goal-driven planning with per-plan stats & AI savings suggestions

**Status:** Architecture complete (2026-06-09)  
**Discovery:** `discovery-20260609-us0019` in `handoffs/archive/po-to-tl-pack-20260608-d.md`  
**Research:** [R-0084](research.md#r-0084--us-0019-goal-plans-per-plan-stats-category-overlay--ai-savings), [R-0080](research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake)  
**Decisions:** **DEC-0091** (goal schema + template); **DEC-0092** (goal-stats API + yearly rollup + feasibility copy); **DEC-0093** (category overlay cap); **DEC-0094** (deterministic savings suggestions); **DEC-0095** (goal account scope); **DEC-0096** (PVA household scope unchanged); **DEC-0097** (REST primary; optional AI tool)  
**Depends on:** US-0018 DONE (**DEC-0087** catalog + expense-series, **DEC-0088** `CategoryFilter`, **DEC-0089** compare actuals-only), US-0014 DONE (templates/onboarding), US-0006 (AI audit), DEC-0073 (overlay compare delta), DEC-0007 (forecast baseline unchanged)  
**Sprint:** **S0018** recommended (single sprint ≤12 tasks)  
**Acceptance:** `docs/product/acceptance.md` § US-0019 (AC-1..AC-6)  
**Spec-pack:** `docs/engineering/spec-pack/US-0019-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**User guide:** `docs/user-guides/US-0019.md` (`USER_GUIDE_MODE=1`; execute publishes content)

### Problem

Operators need **goal-driven what-if planning**: target balance + target date (e.g. €10 000 in 5 months), **statistics scoped to that plan** (not household aggregates on detail), **category-scoped spend cuts** that affect recompute, and **deterministic savings suggestions** the operator explicitly applies — building on US-0018 category APIs without forking DEC-0007 forecast buckets or DEC-0089 compare actuals preview.

| AC | Discovery verdict | Architecture slice |
|----|-------------------|-------------------|
| AC-1 Goal plan type | **Gap** | **S1** primary |
| AC-2 Per-plan statistics | **Gap** | **S2** primary |
| AC-3 Category adjustments | **Partial** (form exists; overlay ignores category) | **S3** primary |
| AC-4 AI savings suggestions | **Gap** | **S4** primary |
| AC-5 Privacy | Verify | S4 + optional S6 |
| AC-6 Regression | Verify | S5 smoke |

`isolation_scope`: artifact + repo source reads; `fresh_context_marker`: `architecture-20260609-us0019-tl-fresh`; no host `.env` / secrets read.

### Research gates resolved (R-0084)

| Gate | Decision | Alternative rejected |
|------|----------|---------------------|
| Goal storage | **DEC-0091** — `plans` columns + `goal_balance` enum | JSON blob; per-version columns; `plan_goals` table |
| Stats API | **DEC-0092** — `GET …/goal-stats` per plan+version | Extend `/compare` only |
| Yearly rollup | **DEC-0092** — calendar year `SUM(planned_net)` | Rolling 12m |
| Category `remove_outflow` | **DEC-0093** — cap at 3-mo mirror avg outflow | Daily weighted; display-only |
| Category `add_outflow` | **DEC-0093** — household-labeled, no cap | API reject |
| Savings ranking | **DEC-0094** — deterministic top-N aggregates | LLM-only |
| Fixed-category exclusion | **DEC-0094** — exclude DEC-0007 fixed bucket | Show all categories |
| Account scope | **DEC-0095** — optional `goal_account_id`; default max-balance asset | Always household |
| Feasibility | **DEC-0092** — gap + required monthly (0% interest); copy only | PMT + auto-lines |
| PVA scope | **DEC-0096** — unchanged household active plan | Per-plan PVA |
| AI tool | **DEC-0097** — REST primary; optional `get_category_savings` P2 | Chat-only |

### System context

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Browser — PlanningPage (Scenarios / Compare / PVA)                       │
│    Goal balance template card + target fields (DEC-0091)                    │
│    GoalStatsStrip ◀── GET …/goal-stats (DEC-0092)                         │
│    CategoryFilter add-line (exists) + CategorySavingsModal (DEC-0094)     │
│    CategoryTrendChart on Compare = actuals only (DEC-0089 — unchanged)    │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ POST /api/v1/plans (goal_balance)
                                │ GET  /api/v1/plans/{id}/goal-stats
                                │ GET  /api/v1/plans/{id}/category-savings-suggestions
                                │ POST adjustments (batch remove_outflow category)
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum) — plan engine                                       │
│    plans migration: target_balance_eur, target_date, goal_account_id        │
│    build_overlay_deltas + category cap via expense_series (DEC-0093)      │
│    goal-stats SQL on plan_daily_cashflow + compare delta (DEC-0092)       │
│    savings ranking: aggregates_by_category + resolve_bucket filter        │
│    goal projection: per-account baseline when goal_account_id set (0095)  │
│                                                                             │
│  Forecast monthly API ──▶ UNCHANGED (DEC-0007 / US-0015)                  │
│  PVA active endpoint ──▶ UNCHANGED household (DEC-0096)                   │
│  Optional: get_category_savings tool wraps same service (DEC-0097 P2)       │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ mirror transactions + categories + plan_daily
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  TimescaleDB — plan_daily_cashflow (730d horizon)                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

**AC-6 boundary:** US-0014 template grid, toasts, PVA guided card unchanged; DEC-0089 compare category filter stays actuals-only; no Firefly write-back; aggregate-only new REST (DEC-0032).

### Architecture contract

```text
US-0019
├── S1 — Goal plan schema + create flow (P0)
│   ├── T1 — Migration: goal_balance enum + plan columns (DEC-0091)
│   ├── T2 — Plan types, validation, templates.rs goal preset
│   ├── T3 — POST /api/v1/plans goal_balance branch + 422 guards
│   └── T4 — PlanningPage Goal balance template card + form fields
├── S2 — Per-plan goal-stats API + UI strip (P0)
│   ├── T1 — goal-stats repository/service (target-date SQL, yearly rollup)
│   ├── T2 — GET /api/v1/plans/{id}/goal-stats route + beyond_horizon
│   ├── T3 — GoalStatsStrip component (Scenarios + Compare)
│   └── T4 — Feasibility copy: gap + required_monthly_savings (no auto-lines)
├── S3 — Category overlay engine (P0)
│   ├── T1 — overlay.rs category remove_outflow cap (3-mo avg via expense_series)
│   ├── T2 — add_outflow category household-labeled path (DEC-0093)
│   ├── T3 — goal_account_id projection fork in project.rs (DEC-0095)
│   └── T4 — Unit tests: cap, zero mirror, account default
├── S4 — Category savings suggestions (P0)
│   ├── T1 — category-savings-suggestions service (DEC-0094 ranking)
│   ├── T2 — GET route + fixed-bucket exclusion
│   ├── T3 — CategorySavingsModal + batch POST adjustments
│   └── T4 — Audit log on adjustment create (AC-5)
├── S5 — Regression + docs (P1)
│   ├── T1 — US-0014 onboarding/templates smoke (AC-6)
│   ├── T2 — DEC-0089 compare actuals widget unchanged
│   └── T3 — docs/user-guides/US-0019.md
├── S6 — Optional AI tool (P2)
│   └── T1 — get_category_savings wraps DEC-0094 service (DEC-0097)
└── V1 — verify-work OIDC external profile smoke
```

**Out of scope:** Per-plan PVA endpoint; PMT/interest feasibility; LLM savings ranking; category-scoped forecast re-projection; Grafana changes; auto-apply savings lines.

### S1 — Goal schema (DEC-0091)

#### Migration

| Column | Type | Validation |
|--------|------|------------|
| `target_balance_eur` | `NUMERIC(18,2) NULL` | Required when `template='goal_balance'` |
| `target_date` | `DATE NULL` | Required; ≥ today on create |
| `goal_account_id` | `TEXT NULL` | Optional; see DEC-0095 |

`ALTER TYPE plan_template ADD VALUE 'goal_balance'` in dedicated migration after existing values.

#### Create API

```json
POST /api/v1/plans
{
  "name": "Emergency fund",
  "template": "goal_balance",
  "target_balance_eur": "10000.00",
  "target_date": "2026-11-01",
  "goal_account_id": "114"
}
```

**Files:** `backend/migrations/`, `plan/types.rs`, `plan/repository.rs`, `plan/templates.rs`, `api/plans.rs`, `PlanningPage.tsx`

### S2 — Goal-stats API (DEC-0092)

#### Endpoint

`GET /api/v1/plans/{plan_id}/goal-stats?version_id={optional}`

**404** when `template != 'goal_balance'`.

#### Frozen response (excerpt)

```json
{
  "plan_id": "…",
  "version_id": "…",
  "target_balance_eur": "10000.00",
  "target_date": "2026-11-01",
  "goal_account_id": "114",
  "monthly_delta_vs_baseline": "-120.00",
  "yearly_rollup": [{ "year": 2026, "planned_net_eur": "2400.00" }],
  "projected_balance_at_target": "9850.00",
  "gap_eur": "150.00",
  "required_monthly_savings_eur": "30.00",
  "on_track": false,
  "beyond_horizon": false,
  "reporting_currency": "EUR",
  "computed_at": "2026-06-09T12:00:00Z"
}
```

#### Target-date SQL (frozen)

Last `planned_balance` from successful computation where `ts::date <= target_date` — no interpolation. If `target_date > today + 730d`: `beyond_horizon: true`, `projected_balance_at_target: null`.

#### UI

- **Scenarios:** stats strip below plan summary when goal template selected
- **Compare:** strip above version table for **selected plan only** — not mixed across plans
- **PVA:** unchanged per DEC-0096

### S3 — Category overlay (DEC-0093)

#### `remove_outflow` + `category`

```
effective = min(adj.amount, avg_outflow_last_3_calendar_months)
```

Source: `expense_series_by_month` (DEC-0087). Empty history → **0** overlay + line warning.

#### `add_outflow` + `category`

Full amount on household schedule; category id stored for display — no cap.

#### Account scope (DEC-0095)

Goal plan recompute uses `goal_account_id` daily series when set; default max-balance asset on create when NULL; household fallback + banner.

**Primary file:** `backend/src/plan/overlay.rs`

### S4 — Savings suggestions (DEC-0094)

#### Endpoint

`GET /api/v1/plans/{plan_id}/category-savings-suggestions?months=6&limit=10`

| Filter | Rule |
|--------|------|
| Min spend | ≥ €20/mo average over window |
| Already in plan | Skip `target_type=category` + `remove_outflow` |
| Fixed bucket | Exclude via DEC-0007 `resolve_bucket` |
| Ranking | `total_outflow` DESC deterministic |
| Reduction hint | 50% of avg monthly outflow |

#### Apply flow

Checkbox modal → batch `POST` adjustments → recompute. **No auto-apply.**

### S5 — PVA + regression (DEC-0096, AC-6)

- `GET /api/v1/plans/active/plan-vs-actual` — **no API changes**
- Compare `CategoryTrendChart` — actuals only (DEC-0089)
- US-0014 template grid + empty-plan flows — regression smoke in V1

### Task table (sprint-plan input)

| ID | Slice | Task | Files | Priority |
|----|-------|------|-------|----------|
| **G1** | S1 | Migration goal_balance + columns | `migrations/`, `plan/types.rs` | P0 |
| **G2** | S1 | Create API + template card | `api/plans.rs`, `PlanningPage.tsx` | P0 |
| **S1** | S2 | goal-stats service + SQL | `plan/service.rs`, `repository.rs` | P0 |
| **S2** | S2 | goal-stats route + GoalStatsStrip | `api/plans.rs`, `components/plan/` | P0 |
| **O1** | S3 | Category remove_outflow cap | `plan/overlay.rs` | P0 |
| **O2** | S3 | goal_account_id projection | `plan/project.rs`, `forecast/service.rs` | P0 |
| **A1** | S4 | category-savings-suggestions service | `plan/savings_service.rs` or `api/plans.rs` | P0 |
| **A2** | S4 | Savings modal + batch apply | `PlanningPage.tsx` | P0 |
| **D1** | S5 | User guide US-0019 | `docs/user-guides/US-0019.md` | P1 |
| **R1** | S5 | US-0014 + DEC-0089 regression tests | `PlanningPage` tests, `uat.md` | P1 |
| **T1** | S6 | Optional get_category_savings tool | `ai/tools/` | P2 |
| **V1** | — | OIDC smoke AC-1..AC-6 | `uat.json` | P0 |

**Count:** 9 mandatory P0 (G1, G2, S1, S2, O1, O2, A1, A2, V1) + 2 P1 (D1, R1) + 1 P2 (T1) → **≤12** under `SPRINT_MAX_TASKS` → **single sprint S0018** (no split).

**Deploy order:** G1→G2→O1→O2 (schema + overlay) ∥ S1→S2 after G1; A1→A2 after O1; V1 last.

### Codebase map (US-0019 slice)

| Path | Role | Touch |
|------|------|-------|
| `backend/migrations/` | Goal columns + enum | G1 |
| `backend/src/plan/overlay.rs` | Category cap | O1 |
| `backend/src/plan/project.rs` | Account-scoped baseline | O2 |
| `backend/src/plan/service.rs` | goal-stats | S1 |
| `backend/src/api/plans.rs` | goal-stats + savings routes | S2, A1 |
| `backend/src/transactions/repository.rs` | expense_series + aggregates reuse | O1, A1 |
| `frontend/src/pages/PlanningPage.tsx` | Template, strip, modal | G2, S2, A2 |
| `frontend/src/lib/api.ts` | New types + fetchers | S2, A1 |

### Decisions (US-0019)

| ID | Topic | Contract |
|----|-------|----------|
| **DEC-0091** | Goal schema | `goal_balance` template; plan-level target fields |
| **DEC-0092** | Goal-stats API | Per plan+version; calendar yearly; gap copy; 730d horizon guard |
| **DEC-0093** | Category overlay | remove cap 3-mo avg; add household-labeled |
| **DEC-0094** | Savings suggestions | Deterministic ranking; fixed exclusion; modal apply |
| **DEC-0095** | Goal account | Optional id; default max-balance asset |
| **DEC-0096** | PVA scope | Household active plan unchanged |
| **DEC-0097** | AI path | REST primary; optional tool P2 |

### Risks

| Risk | Mitigation |
|------|------------|
| `target_date` beyond 730d | `beyond_horizon` flag + UI copy (DEC-0092) |
| Category overlay over-removal | Cap at historical avg (DEC-0093) |
| Fixed costs in savings list | DEC-0007 bucket filter (DEC-0094) |
| Goal account vs compare mismatch | Document in strip + user guide |
| Compare vs PVA confusion | DEC-0096 contextual help |
| Enum migration ordering | Dedicated migration; CI migration test |
| DEC-0089 regression | No compare API category param; widget actuals-only |
| Seven-tool registry (optional T1) | P2 only; REST satisfies AC-4/AC-5 without tool |

### Next phase

`/sprint-plan` — materialize **S0018** from task table; then `/plan-verify` → `/execute`.

`runtime_proof_id`: `runtime-proof-architecture-20260609-us0019-001`

# US-0020 — Subscription manual discovery, majority category & operator tags

**Status:** Architecture complete (2026-06-10)  
**Discovery:** `discovery-20260609-us0020` in `handoffs/archive/po-to-tl-pack-20260608-i.md`  
**Research:** [R-0085](research.md#r-0085--us-0020-subscription-discover-majority-category--operator-tags), [R-0080](research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake)  
**Decisions:** **DEC-0098** (discover explorer API); **DEC-0099** (manual confirm-from-discover); **DEC-0100** (display majority category); **DEC-0101** (operator tag schema); **DEC-0102** (tag assign + list filter); **DEC-0103** (Grafana `$tag` P2 stretch)  
**Depends on:** US-0003 DONE (detection + pending confirm), **DEC-0084**..**DEC-0086** (confirm persistence), US-0018 DONE (**DEC-0087** category catalog), US-0008 (alert dedup — AC-6 regression guard)  
**Sprint:** **S0019** recommended (single sprint ≤12 tasks; Grafana P2 optional)  
**Acceptance:** `docs/product/acceptance.md` § US-0020 (AC-1..AC-6)  
**Spec-pack:** `docs/engineering/spec-pack/US-0020-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**User guide:** `docs/user-guides/US-0020.md` (`USER_GUIDE_MODE=1`; execute publishes content)

### Problem

Operators need **manual subscription discovery** alongside auto-detection: search recurring candidates, confirm without pending-only path, see **majority display category** from linked transactions, and organize confirmed subscriptions with **operator-defined tags** — all product-DB overlay metadata with **no Firefly write-back**. US-0003 tabs + pending confirm exist; explorer API/UI, manual confirm-from-search, `display_category_id`, and tag CRUD/assign/filter are missing.

| AC | Discovery verdict | Architecture slice |
|----|-------------------|-------------------|
| AC-1 Manual search | **Gap** | **S1 primary** |
| AC-2 Operator confirm | **Gap** | **S2 primary** |
| AC-3 Majority category | **Gap** | **S3 primary** |
| AC-4 Operator tags | **Gap** | S4 + S5 |
| AC-5 Storage contract | **Gap** | S3 + S4 (product DB only) |
| AC-6 Regression | Verify | S6 smoke |

`isolation_scope`: artifact + repo source reads; `fresh_context_marker`: `architecture-20260610-us0020-tl-fresh`; no host `.env` / secrets read.

### Research gates resolved (R-0085 — 14 core + 2 stretch)

| Gate | Decision | Alternative rejected |
|------|----------|---------------------|
| Explorer engine | **DEC-0098** — reuse `detect_recurrence_groups` + post-filters | Ad-hoc SQL GROUP BY |
| Discover route | **DEC-0098** — `GET /discover` + `POST /discover/confirm` | Extend pending confirm only |
| Manual confirm state | **DEC-0099** — direct `confirmed` insert | Pending intermediate |
| DEC-0085 on manual | **DEC-0099** — merge when payee+interval exists | 409 duplicate |
| Rejected payee-interval | **DEC-0099** — 409 until operator clears | Silent override |
| Alert on manual confirm | **DEC-0099** — no `new_detection` | Emit alert |
| Majority algorithm | **DEC-0100** — COUNT + RANK (cnt DESC, last_date DESC) | `mode()`; operator override |
| Majority refresh | **DEC-0100** — recompute on merge only | Every sync |
| `display_category_id` | **DEC-0100** — column on `subscription_patterns` | Join table |
| Tag tables | **DEC-0101** — `operator_tags` + junction | JSON array |
| Tag delete | **DEC-0101** — hard delete + CASCADE | Soft delete |
| Tag assign API | **DEC-0102** — `PUT …/tags` replace set | PATCH per tag |
| List `?tag=` | **DEC-0102** — slug filter on list API | Client-only filter |
| Result cap | **DEC-0098** — 50 | Paginated |
| Amount band filter | **DEC-0098** — P2 stretch | Required in AC-1 |
| Grafana `$tag` | **DEC-0103** — P2 if ≤12 tasks | Defer post-MVP (default OK) |

### System context

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Browser — SubscriptionsPage (All / Pending / Standing / Discover)          │
│    Discover tab: search form + candidate table (DEC-0098)                   │
│    Confirm row → POST discover/confirm (DEC-0099)                           │
│    Majority category badge + tooltip (DEC-0100)                               │
│    Tag manager + chips + ?tag= filter chips (DEC-0101/0102)                 │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ GET  /api/v1/subscriptions/discover
                                │ POST /api/v1/subscriptions/discover/confirm
                                │ GET  /api/v1/subscriptions?tag=
                                │ PUT  /api/v1/subscriptions/:id/tags
                                │ CRUD /api/v1/subscription-tags
                                │ GET  /api/v1/categories (display names)
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum) — subscriptions module                              │
│    detect_recurrence_groups reuse (DEC-0084 payee_key, DEC-0086 tolerance)  │
│    merge_confirmed_pattern on manual confirm (DEC-0085)                     │
│    display_category_id compute at confirm + merge (DEC-0100)              │
│    operator_tags + subscription_pattern_tags (DEC-0101)                   │
│                                                                             │
│  DetectionPipeline::run_candidates ──▶ UNCHANGED (AC-6)                   │
│  Pending confirm/reject + alert dedup ──▶ UNCHANGED (US-0008)             │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ mirror transactions + categories
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  PostgreSQL — subscription_patterns + operator_tags (product DB overlay)    │
│  Optional Grafana subscriptions.json $tag variable (DEC-0103 P2)            │
└─────────────────────────────────────────────────────────────────────────────┘
```

**AC-6 boundary:** No changes to `DetectionPipeline` skip order or pending tab flows; US-0008 alert dedup untouched; OIDC external profile smoke in V1; no Firefly write-back.

### Architecture contract

```text
US-0020
├── S1 — Discover explorer API + UI (P0)
│   ├── T1 — Discover service: load txs, detect_recurrence_groups, post-filters (DEC-0098)
│   ├── T2 — GET /api/v1/subscriptions/discover route + meta cap 50
│   └── T3 — Discover tab: account + payee + interval form + results table
├── S2 — Manual confirm-from-discover (P0)
│   ├── T1 — POST discover/confirm: validate, merge/create, no alert (DEC-0099)
│   └── T2 — Discover row confirm + kind override modal; merge vs create toast
├── S3 — Majority display category (P0)
│   ├── T1 — Migration display_category_id + compute helper (DEC-0100)
│   ├── T2 — Recompute on merge_confirmed_pattern
│   └── T3 — Majority badge on confirmed list + detail drawer + tooltip
├── S4 — Operator tags backend (P0)
│   ├── T1 — Migration operator_tags + subscription_pattern_tags (DEC-0101)
│   ├── T2 — Tag CRUD routes
│   └── T3 — PUT assign + list ?tag= + tags on DTO (DEC-0102)
├── S5 — Tag UI (P0)
│   ├── T1 — Tag manager modal (CRUD + delete confirm)
│   └── T2 — Detail drawer chips + All/Standing filter chips
├── S6 — Regression + docs (P1)
│   ├── T1 — US-0003/US-0008 detection + dedup tests (AC-6)
│   ├── T2 — docs/user-guides/US-0020.md
│   └── T3 — Optional amount band on discover (DEC-0098 P2)
├── S7 — Grafana $tag (P2)
│   └── T1 — subscriptions.json templating (DEC-0103)
└── V1 — verify-work OIDC external profile smoke
```

**Out of scope:** Firefly tag/category write-back; operator override of display category (stretch); per-account tags; changes to auto-detection thresholds; paginated discover beyond 50.

### S1 — Discover explorer (DEC-0098)

#### Endpoint

`GET /api/v1/subscriptions/discover?account_id=&payee=&interval_days=&amount_min=&amount_max=&limit=50`

#### Response (frozen excerpt)

```json
{
  "candidates": [{
    "payee_key": "netflix",
    "display_name": "Netflix P3E460",
    "interval_days": 30,
    "median_amount": "-12.99",
    "confidence_pct": 95,
    "transaction_count": 8,
    "transaction_ids": ["ff-…"],
    "account_ids": ["114"]
  }],
  "meta": { "limit": 50, "truncated": false, "window_days": 365 }
}
```

**Files:** `backend/src/subscriptions/discovery.rs`, `api/subscriptions.rs`, `SubscriptionsPage.tsx`

### S2 — Manual confirm (DEC-0099)

`POST /api/v1/subscriptions/discover/confirm` — see DEC-0099 for merge/rejection/alert rules.

**Primary files:** `repository.rs`, `service.rs`, `api/subscriptions.rs`

### S3 — Majority category (DEC-0100)

Column `display_category_id TEXT NULL` on `subscription_patterns`. Compute at confirm and on merge — RANK policy frozen in DEC-0100.

Display via DEC-0087 catalog; NULL → "Uncategorized".

### S4/S5 — Tags (DEC-0101, DEC-0102)

Schema and APIs per DEC-0101/0102. UI: global tag manager; multi-select chips on detail; slug filter chips on list tabs.

### Task table (sprint-plan input)

| ID | Slice | Task | Files | Priority |
|----|-------|------|-------|----------|
| **M1** | S3/S4 | Migration: `display_category_id` + tag tables | `migrations/` | P0 |
| **D1** | S1 | Discover service + GET route | `subscriptions/discovery.rs`, `api/` | P0 |
| **D2** | S1 | Discover tab UI | `SubscriptionsPage.tsx` | P0 |
| **C1** | S2 | POST discover/confirm + merge | `repository.rs`, `service.rs` | P0 |
| **C2** | S3 | Majority compute + merge refresh | `repository.rs` | P0 |
| **C3** | S3 | Majority badge UI | `SubscriptionsPage.tsx` | P0 |
| **T1** | S4 | Tag CRUD API | `api/subscription_tags.rs` | P0 |
| **T2** | S4 | PUT assign + list `?tag=` | `api/subscriptions.rs` | P0 |
| **T3** | S5 | Tag manager + filter chips | `SubscriptionsPage.tsx` | P0 |
| **R1** | S6 | User guide US-0020 | `docs/user-guides/US-0020.md` | P1 |
| **R2** | S6 | US-0003/US-0008 regression tests | `subscriptions/` tests | P1 |
| **G1** | S7 | Grafana `$tag` variable | `subscriptions.json` | P2 |
| **V1** | — | OIDC smoke AC-1..AC-6 | `uat.json` | P0 |

**Count:** 10 mandatory P0 (M1, D1, D2, C1, C2, C3, T1, T2, T3, V1) + 2 P1 (R1, R2) + 1 P2 (G1) → **12** at `SPRINT_MAX_TASKS` with G1 optional → **single sprint S0019** (no split).

**Deploy order:** M1→D1→C1→C2 (schema + discover + confirm) ∥ T1 after M1; D2/C3/T3 UI after APIs; V1 last.

### Codebase map (US-0020 slice)

| Path | Role | Touch |
|------|------|-------|
| `backend/migrations/` | display_category + tags | M1 |
| `backend/src/recurrence/detect.rs` | `detect_recurrence_groups` reuse | D1 |
| `backend/src/subscriptions/detection.rs` | unchanged — regression guard | — |
| `backend/src/subscriptions/discovery.rs` | **New** discover service | D1 |
| `backend/src/subscriptions/repository.rs` | confirm-from-discover, majority, tags | C1, C2, T2 |
| `backend/src/api/subscriptions.rs` | discover + list tag filter | D1, T2 |
| `backend/src/api/subscription_tags.rs` | **New** tag CRUD | T1 |
| `frontend/src/pages/SubscriptionsPage.tsx` | Discover tab, badges, tags | D2, C3, T3 |
| `grafana/.../subscriptions.json` | optional `$tag` | G1 |

### Decisions (US-0020)

| ID | Topic | Contract |
|----|-------|----------|
| **DEC-0098** | Discover explorer | Reuse recurrence core; GET `/discover`; cap 50; amount band P2 |
| **DEC-0099** | Manual confirm | POST `/discover/confirm`; direct confirmed; DEC-0085 merge; 409 rejection; no alert |
| **DEC-0100** | Majority category | `display_category_id`; RANK tie-break; recompute on merge |
| **DEC-0101** | Tag schema | `operator_tags` + junction; hard delete; global scope |
| **DEC-0102** | Tag assign/filter | PUT replace set; `?tag=` slug on list |
| **DEC-0103** | Grafana `$tag` | P2 stretch; DEC-0089 independent pattern |

### Risks

| Risk | Mitigation |
|------|------------|
| Explorer perf on 365d window | Account filter required in UI; cap 50 (DEC-0098) |
| Manual confirm bypasses rejection maps | 409 on rejected payee-interval (DEC-0099) |
| DEC-0085 merge + category drift | Recompute majority on merge (DEC-0100) |
| `mode()` tie ambiguity | Explicit RANK policy (DEC-0100) |
| Tag delete surprise | Confirm dialog (DEC-0101) |
| Detection regression | No `run_candidates` changes; dedicated tests (AC-6) |
| Grafana stretch slips | DEC-0103 P2; SPA filter sufficient |

### Next phase

`/sprint-plan` — materialize **S0019** from task table; then `/plan-verify` → `/execute`.

`runtime_proof_id`: `runtime-proof-architecture-20260610-us0020-001`

`triad_hot_surface`: architecture § US-0020 appended; decisions DEC-0098..DEC-0103 formalized; spec-pack US-0020 created; state checkpoint; post-write `--check` required.

---

# BUG-0023 — Crypto Wealth EUR values missing (live regression)

**Status:** architecture complete (2026-06-12)  
**Discovery:** `discovery-20260612-bug0023` in `handoffs/po_to_tl.md`  
**Research:** [R-0093 §5](research.md#r-0093--bug-0023-crypto-wealth-eur-values-live-regression)  
**Decisions:** extends **DEC-0064**, **DEC-0080**, **DEC-0081**, **DEC-0038**; **GATE-DEC-1 closed — no new DEC** (subtotal contract unchanged)  
**Sprint:** `/quick` recommended (≤10 tasks)  
**Acceptance:** `docs/product/acceptance.md` rows **BO**, **BP**, **BQ**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0023-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**Related:** BUG-0014 DONE (AP/AQ live deferred); BUG-0013 DONE (**DEC-0080**); **R-0079** AP2 rejected for this bug

### Root cause (frozen, R-0093 §5)

Live regression on Bitunix-connected Wealth **Crypto** tab decomposes to a **single upstream wallet-ingest failure** with three downstream display symptoms — not a wealth aggregation rewrite or deploy gap (H4 ruled out).

| Layer | Finding | Symptom |
|-------|---------|---------|
| **Wallet ingest** | `parse_futures_wallet` silent `None` (no log); equity formula omits `crossUnrealizedPNL`/`isolationUnrealizedPNL`; HTTP client ignores JSON `code` | No `product_type=futures` row persisted (**H1**) |
| **Subtotal** | `wealth/service.rs` `subtotal_eur = sum(market_value_eur)` | **€0** when wallet row missing (**BO**) |
| **Linear display** | `pnl.rs` sets `market_value_eur: None` for linear per **DEC-0064** | `holdings_all[].value_eur` NULL (**BP**, **H2**) |
| **Total return** | `portfolio/service.rs` `total_return_pct` None when `crypto_value_eur=0` | **—** despite unrealized EUR (**BQ**, **H5**) |

**Live probe (2026-06-12):** `GET /api/v1/wealth` — `crypto.subtotal_eur=0`, **11** linear rows, all `value_eur=null`, `pnl.unrealized_eur≈376.83`, `pnl.total_return_pct=null`; positions sync succeeds; no futures wallet row.

`isolation_scope`: artifact + repo source reads; no host `.env` / secrets read.

### Architecture gates (frozen)

| Gate | Decision | Rationale | Alternatives rejected |
|------|----------|-----------|----------------------|
| **GATE-BO-1** | Wallet parse hardening in `bitunix.rs` | Equity keys + `code==0` validation + parse-skip logging + OpenAPI wiremock | Re-open wealth aggregation; ops-only redeploy (H4 ruled out) |
| **GATE-BP-1** | **D1** — `entryValue` → display `exposure_eur` (not `market_value_eur`) | Satisfies BP without **DEC-0064** amend; payload already stored | Tier-2 mark-price API (scope); merge notional into subtotal (violates **DEC-0064**) |
| **GATE-AGG-1** | Subtotal = `sum(market_value_eur)` wallet-only | Preserves **DEC-0064** / **DEC-0080** double-count guard | Sum linear notionals into subtotal; **BUG-0014** AP2 defensive snapshot fallback (masks parse bug) |
| **GATE-BQ-1** | Denominator = wallet-priced `crypto_value_eur`; baseline on first priced sync | Resolves automatically when BO priced; unrealized alone must not drive return (**DEC-0038**) | Use unrealized as return numerator; portfolio snapshot subtotal override |
| **GATE-DEC-1** | **No new DEC** | Contracts unchanged — implementation gap under existing decisions | New DEC only if subtotal merges position notional (rejected) |

### Fix slices

```text
BUG-0023
├── BO — wallet ingest + observability (P0)
│   ├── BO1 — parse_futures_wallet: equity keys + unrealized key fix
│   ├── BO2 — JSON code==0 validation + parse-skip warn diagnostics
│   └── BO3 — wiremock + unit tests (Bitunix OpenAPI sample)
├── BP — linear Value EUR display (P1)
│   ├── BP1 — migration exposure_eur + pnl.rs entryValue→EUR persist
│   └── BP2 — wealth/service.rs map holdings_all.value_eur from exposure_eur
├── BQ — downstream verify (P1)
│   └── BQ1 — baseline capture + total_return_pct when wallet priced
├── T1 — integration/regression tests (BO/BP/BQ)
├── G1 — automated gate (cargo test + npm build)
└── V1 — operator verify-work (localhost:18080 + OIDC smoke)
```

**Deploy order:** BO → exchange sync + PnL recompute → BP (can ship same release) → V1.

**Out of scope:** Tier-2 `ExchangePriceBook` / mark-price feed; merge linear notional into subtotal; **BUG-0014** AP2 defensive subtotal; Grafana panel edits; `holdings_count` UX footnote (P2 optional).

### BO — Wallet parse hardening (GATE-BO-1)

#### BO1 — Equity + unrealized key coverage

| Change | Contract |
|--------|----------|
| Equity fallback sum | `available + frozen + margin + crossUnrealizedPNL + isolationUnrealizedPNL` when direct equity keys absent |
| Equity key scan | Retain `accountEquity`, `totalEquity`, `equity`, `balance` first |
| Unrealized keys | Add `crossUnrealizedPNL`, `isolationUnrealizedPNL` alongside existing aliases |
| Persist row | `product_type=futures`, `asset=marginCoin` (default USDT), `quantity=equity`, `market_value_usd=Some(qty)` for USDT |

**Files:** `backend/src/exchanges/bitunix.rs`

#### BO2 — Fail-visible sync path

| Change | Contract |
|--------|----------|
| JSON `code` | Reject body when `code != 0` (or missing on error responses) before parse |
| Parse skip | `warn!` with redacted shape diagnostic (marginCoin present, equity keys tried, derived sum) when `parse_futures_wallet` returns `None` |
| Sync continuation | Positions sync unchanged — wallet failure must be observable in logs |

**Files:** `backend/src/exchanges/bitunix.rs` (HTTP helper if shared)

#### BO3 — Regression tests

| Test | Contract |
|------|----------|
| OpenAPI sample | Official Get Single Account array shape → futures row with non-zero equity |
| Zero-equity skip | Empty `data: []` → warn path, no row |
| `code != 0` | No row persisted; error surfaced |

**Files:** `backend/src/exchanges/bitunix.rs` tests; wiremock fixture per R-0093 web refs

**Risks:** Live payload still differs from OpenAPI — BO2 logging mandatory before operator V1; equity formula may need field alias iteration post-deploy.

### BP — Per-position Value EUR display (GATE-BP-1, D1)

Preserve **DEC-0064**: linear rows **excluded** from `sum(market_value_eur)` subtotal. Populate **display-only** EUR for the Value EUR column.

#### BP1 — Persist `exposure_eur` at recompute

| Step | Contract |
|------|----------|
| Parse | `entryValue` from linear position `payload` (Bitunix pending-positions API) |
| Convert | `fx.to_eur(entryValue, "USDT", price_book)` |
| Persist | New nullable column `exposure_eur` on `exchange_holdings`; **do not** set `market_value_eur` for linear |
| Recompute | Extend `update_holding_eur` (or sibling) in linear branch of `compute_hybrid_pnl` |

**Files:** `backend/migrations/017_bug0023_exposure_eur.sql`, `backend/src/exchanges/repository.rs`, `backend/src/portfolio/pnl.rs`

**Alternative rejected:** Write display value into `market_value_eur` — would inflate subtotal and violate **DEC-0064** / **DEC-0080**.

#### BP2 — Wire through wealth API

| Surface | Contract |
|---------|----------|
| `holdings_all[].value_eur` | `market_value_eur.or(exposure_eur)` per row |
| `holdings_top` | Priced wallet rows only (unchanged) |
| `crypto.subtotal_eur` | `sum(market_value_eur)` only — **no** `exposure_eur` merge |

**Files:** `backend/src/wealth/service.rs` (minimal); `frontend/src/pages/WealthPage.tsx` pass-through only if API shape unchanged

**Acceptance copy alignment:** "EUR equivalent at valuation time" = exchange-reported `entryValue` notional + FX, not external mark feed.

**Risks:** `entryValue` may differ from operator mark-to-market — acceptable per D1; document in release notes if operator questions gap.

### BQ — Total return % (GATE-BQ-1)

No separate execute slice beyond BO + baseline path verification.

| Step | Code | Expected after BO |
|------|------|-------------------|
| `compute_hybrid_pnl` | Futures row priced → `crypto_value_eur > 0` | Non-zero wallet equity |
| Baseline capture | `capture_if_missing` when exchange `sum(market_value_eur) > 0` | First priced sync captures baseline |
| `total_return_pct` | `(crypto_value_eur - baseline) / baseline` when `baseline > 0` | Populated when baseline exists |

**BQ1 (verify task):** Integration test or SQL probe post-fix: wallet row priced → baseline row exists → API `pnl.total_return_pct` non-null with non-zero unrealized.

**Files:** `backend/src/portfolio/service.rs` (verify-only unless baseline bug found); tests

**Rejected:** Drive `total_return_pct` from unrealized alone — violates **DEC-0038** PnL boundary.

### Operator gates (V1)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild with BO+BP fixes.
2. **Exchange sync** — Bitunix Full/exchange sync success.
3. **PnL recompute** — trigger post-sync (existing scheduler path).
4. **AP1 SQL probe** (from **BUG-0014**, still valid):

```sql
SELECT product_type, asset, quantity, market_value_eur, exposure_eur, unrealized_pnl_eur
FROM exchange_holdings WHERE exchange_id = 'bitunix' ORDER BY product_type, asset;
```

| Probe outcome | Pass criterion |
|---------------|----------------|
| `futures` row | `market_value_eur` ≈ operator ~€2000 order of magnitude |
| Linear rows | `exposure_eur` populated; `market_value_eur` NULL |
| API BO | `crypto.subtotal_eur` matches wallet row, not €0 |
| API BP | `holdings_all[].value_eur` non-null for linear with `entryValue` |
| API BQ | `pnl.total_return_pct` non-null when baseline captured |

### Task table (sprint-plan input)

| ID | Sub | Task | Files | Priority |
|----|-----|------|-------|----------|
| **BO1** | BO | Equity + unrealized key parse fix | `bitunix.rs` | P0 |
| **BO2** | BO | `code==0` validation + parse-skip logging | `bitunix.rs` | P0 |
| **BO3** | BO | OpenAPI wiremock + unit tests | `bitunix.rs` tests | P0 |
| **BP1** | BP | `exposure_eur` migration + pnl `entryValue` persist | `migrations/`, `repository.rs`, `pnl.rs` | P1 |
| **BP2** | BP | `holdings_all.value_eur` from `exposure_eur` | `wealth/service.rs` | P1 |
| **BQ1** | BQ | Baseline + total_return integration verify | `portfolio/service.rs`, tests | P1 |
| **T1** | all | Regression tests BO/BP/BQ | `backend/tests/` | P0 |
| **G1** | all | Automated gate | `cargo test`, `npm run build` | P0 |
| **V1** | all | Operator verify-work | acceptance BO–BQ | P0 |

**Count:** 9 mandatory tasks ≤ `SPRINT_MAX_TASKS` (12) → **`/quick`** recommended.

### Codebase map (BUG-0023 slice)

| Path | Role | Touch |
|------|------|-------|
| `backend/src/exchanges/bitunix.rs` | Wallet parse + sync | BO1–BO3 |
| `backend/src/exchanges/repository.rs` | `exposure_eur` persist | BP1 |
| `backend/migrations/017_bug0023_exposure_eur.sql` | Display column | BP1 |
| `backend/src/portfolio/pnl.rs` | Linear display valuation | BP1 |
| `backend/src/portfolio/service.rs` | Baseline + total return | BQ1 verify |
| `backend/src/wealth/service.rs` | Subtotal + holdings_all | BP2 |
| `frontend/src/pages/WealthPage.tsx` | Value EUR column | pass-through |

### Decisions (BUG-0023)

| Topic | Contract | Existing DEC |
|-------|----------|--------------|
| Subtotal source | Wallet `market_value_eur` only | **DEC-0064**, **DEC-0080** |
| Linear in subtotal | Excluded | **DEC-0064** |
| Value EUR column | Display `exposure_eur` from `entryValue` | extends **DEC-0081** holdings surface |
| Total return denominator | Wallet-priced `crypto_value_eur` | **DEC-0038** |
| New DEC | **None** — GATE-DEC-1 closed | — |

### Risks

| Risk | Mitigation |
|------|------------|
| Live wallet shape differs from OpenAPI | BO2 warn diagnostics + operator payload capture (redacted) pre-V1 |
| Equity still zero after key fix | Log derived components; escalate to Bitunix field alias — do not AP2-mask |
| `entryValue` ≠ operator mark | Accept D1 contract; release note |
| Baseline never captured if wallet intermittently NULL | BO must be stable before BQ acceptance |
| Migration on production | Nullable column; backward compatible |

### Next phase

`/sprint-plan` — materialize `/quick` sprint from task table; then `/plan-verify` → `/execute`.

`runtime_proof_id`: `runtime-proof-architecture-20260612-bug0023-001`

`triad_hot_surface`: architecture § BUG-0023 appended; spec-pack BUG-0023 created; GATE-DEC-1 closed without new DEC; state checkpoint; post-write `--check` required.

---

# BUG-0022 — Plan delete selector regression (activePlanId ignores dropdown)

**Status:** architecture complete (2026-06-13)  
**Discovery:** `discovery-20260613-bug0022` in `handoffs/po_to_tl.md`  
**Research:** [R-0094](research.md#r-0094--bug-0022-plan-delete-selector-regression-activeplanid-ignores-dropdown)  
**Decisions:** extends **DEC-0082** frontend selector contract; **GATE-DEC-1 closed — no new DEC**  
**Sprint:** `/quick` **Q0031** recommended (4 tasks)  
**Acceptance:** `docs/product/acceptance.md` rows **BM**, **BN**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0022-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**Related:** BUG-0014 DONE / Q0022 AS1 (**DEC-0082** shipped); post-Q0022 frontend regression only

### Root cause (frozen, R-0094 §1)

Post-**Q0022** AS1 regression: backend **DEC-0082** guard is intact; frontend selector priority is inverted.

| Layer | Finding | Symptom |
|-------|---------|---------|
| **State** | `selectedPlanId` updated on dropdown `onChange` | Operator selection stored |
| **Derived id** | `activePlanId` useMemo: `active?.id ?? selectedPlanId ?? first` | Global `is_active` **always wins** over dropdown |
| **Dropdown** | Controlled `value={activePlanId}` | Reverts to global active when any active plan exists |
| **Delete guard** | `activePlanIsSelected = plan(activePlanId)?.is_active` | Always **true** when global active exists |
| **Delete control** | `disabled={activePlanIsSelected}` | Permanently disabled under BM repro |
| **Backend** | `DELETE` non-active **204**; active **409** `active_plan_delete_forbidden` | BN API path correct; UI masked by BM |

**Live probe (2026-06-13):** `DELETE /api/v1/plans/:id` non-active **204**; active **409** per **DEC-0082**. PVA tab uses `/api/v1/plans/active/plan-vs-actual` — decoupled from dropdown id.

`isolation_scope`: artifact + repo source reads; no host `.env` / secrets read.

### Architecture gates (frozen)

| Gate | Decision | Rationale | Alternatives rejected |
|------|----------|-----------|----------------------|
| **GATE-SEL-1** | **Option A** — invert useMemo priority: `selectedPlanId ?? globalActiveId ?? firstPlanId` | Minimal diff (~1 useMemo); single derived id; React derived-state best practice ([R-0094](research.md#r-0094--bug-0022-plan-delete-selector-regression-activeplanid-ignores-dropdown) web refs) | **B** split `displayedPlanId` + `globalActivePlanId` (more rename churn); **C** useEffect sync (dual source of truth); **D** uncontrolled dropdown |
| **GATE-DEC82-1** | **No backend change** | 409 guard + tooltip logic correct once selector fixed | Backend delete policy change; auto-activate successor |
| **GATE-TEST-1** | Vitest pure helper `resolveDisplayedPlanId` + delete enablement cases | Closes selector test gap; BM/BN guard logic unit-testable | RTL full-page smoke only (lower priority) |
| **GATE-SCOPE-1** | `/quick` — 2–4 tasks; `PlanningPage.tsx` primary | Well under `SPRINT_MAX_TASKS=12`; same track as Q0022 AS1 | Full sprint split unnecessary |
| **GATE-LABEL-1** | Rename dropdown **"Active plan"** → **"Plan"** (P2 defer OK) | Reduces operator confusion when viewing non-active scenario | Keep misleading label (cosmetic only) |
| **GATE-DEC-1** | **No new DEC** | Clarifies **DEC-0082** §2 frontend contract — selector semantics only | New DEC for selector naming |

### Selector contract (frozen)

```text
displayedPlanId = selectedPlanId ?? plans.find(is_active)?.id ?? plans[0]?.id ?? null

deleteDisabled = plans.find(p => p.id === displayedPlanId)?.is_active === true
deleteTooltip   = deleteDisabled
  ? "Set another plan active before deleting the active plan"
  : "Delete this plan"
```

**Implementation note:** May keep identifier `activePlanId` and change useMemo only, or rename to `displayedPlanId` in one pass — execute choice; contract is priority order, not name.

**Downstream consumers (unchanged semantics after fix):**

| Consumer | Contract |
|----------|----------|
| Plan detail / versions / compare / adjustments | Operate on **displayed** plan id |
| Set active button | `activateMutation.mutate(displayedPlanId)` |
| Delete confirm modal | Delete **displayed** plan id |
| PVA tab | `/api/v1/plans/active/plan-vs-actual` — **not** tied to dropdown |
| `deletePlanMutation` success | Invalidate `plans`, `plan-detail`, `plan-vs-actual`; clear `selectedPlanId` if deleted |

### Fix slices

```text
BUG-0022
├── BM — selector priority fix (P0)
│   └── BM1 — invert useMemo in PlanningPage.tsx; verify activePlanIsSelected uses same id
├── T1 — vitest selector + delete enablement (P0)
├── G1 — automated gate (npm test + build) (P0)
├── V1 — verify-work BM/BN on /planning + OIDC smoke (P0)
└── L1 — dropdown label rename "Plan" (P2 optional, GATE-LABEL-1)
```

**Out of scope:** Backend `plans.rs` / `plan/service.rs`; PVA endpoint; sole-plan delete policy (**DEC-0082** §Risks acceptable); Grafana Dashboard 3 overlay; new DEC.

### BM — Selector priority (GATE-SEL-1)

#### BM1 — Invert `activePlanId` useMemo

| Change | Contract |
|--------|----------|
| useMemo priority | `selectedPlanId ?? plans.find(is_active)?.id ?? plans[0]?.id ?? null` |
| Dropdown | Remains controlled: `value={activePlanId}` + `onChange` → `setSelectedPlanId` |
| Delete guard | `activePlanIsSelected = plan(activePlanId)?.is_active` — now reflects **displayed** plan |
| Set-active banner | `!activePlanIsSelected` when viewing non-active — correct after fix |

**Files:** `frontend/src/pages/PlanningPage.tsx` (L110–113 useMemo primary; L489, L643–683 consumers)

**Alternative rejected:** Split `globalActivePlanId` + `displayedPlanId` — clearer naming but ~20 call-site renames for same behavior.

### BN — Active delete guard (GATE-DEC82-1)

No code change expected. Verify after BM1:

| Surface | Expected |
|---------|----------|
| UI — active plan selected in dropdown | Delete **disabled** + tooltip per **DEC-0082** |
| UI — non-active plan selected | Delete **enabled**; confirm modal → **204** |
| API — `DELETE` active plan | **409** `active_plan_delete_forbidden` (unchanged) |
| `planningFeedback.test.ts` | 409 message path unchanged |
| `active_plan_delete_returns_409_with_code` | Backend test unchanged |

### T1 — Vitest coverage (GATE-TEST-1)

Extract pure helper (suggested location: `frontend/src/pages/planSelector.ts` or colocated export from `PlanningPage.tsx`):

```typescript
resolveDisplayedPlanId(plans, selectedPlanId): string | null
isDeleteDisabled(plans, displayedPlanId): boolean
```

| Case | Expected |
|------|----------|
| Selected non-active + global active exists | Displayed = selected; delete **enabled** |
| Selected null + global active exists | Displayed = global active; delete **disabled** |
| Selected null + no global active | Displayed = first plan |
| Empty plans | Displayed = null; delete disabled |
| Displayed plan `is_active === true` | `isDeleteDisabled === true` |

**Files:** new `planSelector.ts` (or equivalent) + `planSelector.test.ts`

### Operator verification (V1)

1. **BACKEND_FRONTEND_DEPLOY** — frontend rebuild only (no migration).
2. Create or use **2+ plans** with one global active.
3. **BM:** Select non-active in dropdown → **Delete plan** enabled → confirm → plan removed; list refreshes.
4. **BN:** Select active plan → delete disabled + tooltip; direct API `DELETE` active → **409**.
5. OIDC-enabled deploy regression per acceptance **BN**.

### Task table (sprint-plan input)

| ID | Sub | Task | Files | Priority |
|----|-----|------|-------|----------|
| **BM1** | BM | Invert selector useMemo priority | `PlanningPage.tsx` | P0 |
| **T1** | BM/BN | Vitest `resolveDisplayedPlanId` + delete enablement | `planSelector.ts`, `planSelector.test.ts` | P0 |
| **G1** | all | Automated gate | `npm test`, `npm run build` | P0 |
| **V1** | all | verify-work `/planning` BM/BN + OIDC | `sprints/quick/Q0031/uat.md` | P0 |
| **L1** | UX | Dropdown label "Active plan" → "Plan" | `PlanningPage.tsx` L641 | P2 optional |

**Count:** 4 mandatory + 1 optional P2 ≤ `SPRINT_MAX_TASKS` (12) → **`/quick` Q0031** recommended.

### Codebase map (BUG-0022 slice)

| Path | Role | Touch |
|------|------|-------|
| `frontend/src/pages/PlanningPage.tsx` | Selector useMemo, dropdown, delete guard | BM1 (+ optional L1) |
| `frontend/src/pages/planSelector.ts` | Pure helper (new) | T1 |
| `frontend/src/pages/planSelector.test.ts` | Vitest cases | T1 |
| `frontend/src/pages/planningFeedback.test.ts` | 409 message regression | verify only |
| `backend/src/api/plans.rs` | Active delete 409 | **no change** |
| `backend/src/plan/service.rs` | `ActivePlanDeleteForbidden` | **no change** |

### Decisions (BUG-0022)

| Topic | Contract | Existing DEC |
|-------|----------|--------------|
| Selector priority | Operator selection wins over global active for viewing/editing/delete | extends **DEC-0082** §2 frontend |
| Active delete guard | UI disabled + API **409** | **DEC-0082** |
| Single global active | Set active on displayed plan | **DEC-0024** |
| PVA tab | Active endpoint, not dropdown | **DEC-0074** |
| New DEC | **None** — GATE-DEC-1 closed | — |

### Risks

| Risk | Mitigation |
|------|------------|
| Stale `selectedPlanId` after external delete | Existing mutation success clears id; plans refetch |
| Name `activePlanId` misleading post-fix | Optional rename or L1 label change |
| Single-plan operator cannot delete | **DEC-0082** §Risks — acceptable; tooltip explains |
| Browser automation empty SPA (discovery) | BM/BN verified via code + API + vitest; operator visual in V1 |

### Next phase

`/sprint-plan` (role: tech-lead) — materialize `/quick` **Q0031** from task table; then `/plan-verify` → `/execute`.

`runtime_proof_id`: `runtime-proof-architecture-20260613-bug0022-001`

`triad_hot_surface`: architecture § BUG-0022 appended; spec-pack BUG-0022 created; six gates frozen; GATE-DEC-1 closed without new DEC; state checkpoint; post-write `--check` required.

---

# US-0021 — Subscription transaction explorer with rich filters

**Status:** Architecture complete (2026-06-13)  
**Discovery:** `discovery-20260613-us0021` in `handoffs/po_to_tl.md`  
**Research:** [R-0092 §5–8](research.md#r-0092--us-0021-subscription-transaction-explorer-vs-recurrence-only-discover)  
**Decisions:** **DEC-0112** (transaction search API); **DEC-0113** (dual-mode Discover UX); **DEC-0114** (hint pass boundary); extends **DEC-0098**, **DEC-0099**, **DEC-0111**  
**Depends on:** US-0020 DONE (discover patterns tab), US-0018 DONE (category catalog), DEC-0111 DONE (`formatAccountRole`), DEC-0085/0099 (confirm path)  
**Sprint:** **S0020** recommended (single sprint ≤12 tasks; P2 optional defer)  
**Acceptance:** `docs/product/acceptance.md` § US-0021 (AC-1..AC-6)  
**Spec-pack:** `docs/engineering/spec-pack/US-0021-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**User guide:** `docs/user-guides/US-0021.md` (`USER_GUIDE_MODE=1`; execute publishes content)

### Problem

US-0020 shipped recurrence-candidate Discover (`detect_recurrence_groups` → `DiscoverCandidate` rows). Operator expectation (intake) is **transaction-first** ledger search with **category**, **Geldbereich**, **date**, and **manual tx-group activate** for expenses auto-detection missed. Scope expansion — not a US-0020 defect.

| AC | Discovery verdict | Architecture slice |
|----|-------------------|-------------------|
| AC-1 Transaction search | **Gap** | **S1** — DEC-0112 search API + paginated tx table |
| AC-2 Rich filters | **Gap** (partial baseline) | **S1/S2** — SQL push-down + filter bar |
| AC-3 Pattern hint | **Gap** | **S1** — DEC-0114 hint pass on filtered subset |
| AC-4 Manual activate | **Partial** | **S2** — multi-select → preview-group → DEC-0099 confirm |
| AC-5 Regression | Verify | **S4** — no detection threshold changes |
| AC-6 OIDC | Deferred qa | **V1** smoke |

`isolation_scope`: artifact + repo source reads; `fresh_context_marker`: `architecture-20260613-us0021-tl-fresh`; no host `.env` / secrets read.

### Research gates resolved (R-0092 — 9 gates)

| Gate | Decision | Alternative rejected |
|------|----------|---------------------|
| **GATE-UX-1** | **DEC-0113** — dual mode: Transactions (default) \| Suggested patterns | Replace recurrence table |
| **GATE-API-1** | **DEC-0112** — `GET /transactions/search` | `/discover?mode=transactions` |
| **GATE-FILTER-1** | SQL push-down + accounts JOIN (DEC-0111 COALESCE) | In-memory post-load filter |
| **GATE-HINT-1** | **DEC-0114** — separate hint pass; `min_emit_confidence: 60`; row metadata only | Lower global detection threshold |
| **GATE-HINT-2** | **P2 defer** — 2-tx weak hints | Required MVP (scorer returns 0 below 60) |
| **GATE-PAGE-1** | Offset **100/page** hard cap; `total_count` + `has_more` | Keyset cursor MVP; 50 cap |
| **GATE-IDX-1** | **P2 defer** — `idx_transactions_account_date` | Blocking MVP |
| **GATE-CONFIRM-1** | Reuse `POST /discover/confirm`; add `POST .../preview-group` | New confirm payload |
| **GATE-DEC-1** | **DEC-0112**, **DEC-0113**, **DEC-0114** | Extend DEC-0098/0099 only |

### System context

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Browser — SubscriptionsPage Discover tab (DEC-0113 dual mode)              │
│    [Transactions | Suggested patterns]  — Transactions DEFAULT              │
│    Transactions: rich filters + paginated tx table + hint badges            │
│    Multi-select → preview-group → confirm modal → POST discover/confirm     │
│    Patterns: existing DEC-0098 candidate table (unchanged)                  │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ GET  /api/v1/subscriptions/transactions/search (DEC-0112)
                                │ POST /api/v1/subscriptions/transactions/preview-group
                                │ POST /api/v1/subscriptions/discover/confirm (DEC-0099)
                                │ GET  /api/v1/subscriptions/discover (DEC-0098 — Patterns tab)
                                │ GET  /api/v1/categories (US-0018)
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  flow-finance-ai (Axum) — subscriptions module                              │
│    search_transactions: SQL filters + pagination (DEC-0112)                   │
│    attach_recurring_hints: detect_recurrence_groups on subset (DEC-0114)     │
│    preview_transaction_group: median + interval for confirm body            │
│    confirm_from_discover + merge (DEC-0085/0099) — UNCHANGED                │
│                                                                             │
│  DetectionPipeline::run_candidates ──▶ UNCHANGED (AC-5)                   │
│  run_discover (Patterns tab) ──▶ UNCHANGED (DEC-0098)                     │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ mirror transactions + categories + accounts.payload
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  PostgreSQL — transactions, categories, accounts (read-only mirror)         │
│  subscription_patterns overlay — confirm writes only (DEC-0099)             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**AC-5 boundary:** No changes to `DetectionPipeline`, `detection.rs` thresholds, or `run_discover` candidate pass. Hint pass is display-only metadata on search responses.

### Architecture contract

```text
US-0021
├── S1 — Transaction search API (P0)
│   ├── T1 — Repository SQL search + COUNT + Geldbereich JOIN (DEC-0112, GATE-FILTER-1)
│   ├── T2 — Search service + hint pass attachment (DEC-0114)
│   └── T3 — GET /transactions/search + POST /preview-group routes
├── S2 — Transactions mode UI (P0)
│   ├── T4 — Dual-mode tab shell; Transactions default (DEC-0113)
│   ├── T5 — Rich filter bar: account, payee, category, Geldbereich, date
│   ├── T6 — Paginated tx table + hint badges + truncated banner
│   └── T7 — Multi-select + preview-group → confirm modal (DEC-0099)
├── S3 — Patterns tab preservation (P0)
│   └── T8 — Extract existing discover UI to Suggested patterns sub-tab (DEC-0098 frozen)
├── S4 — Tests + docs (P0/P1)
│   ├── T9 — Search + hint integration tests
│   ├── T10 — US-0003/US-0008 regression tests (AC-5)
│   └── T11 — docs/user-guides/US-0021.md
├── P2 — Optional stretch
│   ├── amount_min/max filters (DEC-0112 P2)
│   ├── idx_transactions_account_date migration (GATE-IDX-1)
│   └── 2-tx weak hints (GATE-HINT-2)
└── V1 — verify-work OIDC external profile smoke (AC-6)
```

**Out of scope:** Firefly write-back; changes to global auto-detection thresholds; replacing DEC-0098 patterns tab; keyset pagination MVP; all-accounts search without cap.

### S1 — Transaction search (DEC-0112, DEC-0114)

#### `GET /api/v1/subscriptions/transactions/search`

See **DEC-0112** for frozen query params and response shape.

#### Hint attachment (DEC-0114)

After SQL load (≤500 tx hint budget), run `detect_recurrence_groups` with `min_emit_confidence: 60`. Map group membership to row `recurring_hint`. Exclude confirmed/rejected fingerprints (same as DEC-0098).

#### `POST /api/v1/subscriptions/transactions/preview-group`

**Body:** `{ "transaction_ids": ["…"] }` (≥2)  
**Response:** `{ payee_key, interval_days, median_amount, transaction_ids }`  
Feeds existing `DiscoverConfirmBody` → `POST /discover/confirm` (DEC-0099).

**Files:** `backend/src/subscriptions/repository.rs`, `discovery.rs` or `transaction_search.rs`, `api/subscriptions.rs`

### S2 — Transactions mode UI (DEC-0113)

- Segmented control: **Transactions** | **Suggested patterns** (default Transactions)
- Reuse `CategoryFilter` (US-0018), `formatAccountRole` (DEC-0111)
- Table columns: select, date, description, amount, category, Geldbereich, hint badge
- Pagination: page controls; banner when `has_more` or `truncated`

**Primary files:** `frontend/src/pages/SubscriptionsPage.tsx`, `frontend/src/lib/api.ts`

### S3 — Patterns tab (DEC-0098 unchanged)

Move current Discover candidate UI behind **Suggested patterns** sub-mode. No API or confirm contract changes.

### QA operator repro fixture

| Field | Value |
|-------|-------|
| Environment | `localhost:18080` |
| Account | **114** |
| Discover probe | SEPA-Lastschrift grouped candidate — 11 txs, 31d, 95% |
| Tx-search expected | Same payee as individual rows; hint badge when `recurring_hint=true` |

### Task table (S0020 sprint-plan input)

| ID | Slice | Task | Files | Priority |
|----|-------|------|-------|----------|
| **TX1** | S1 | Repository SQL search + COUNT + role JOIN | `repository.rs` | P0 |
| **TX2** | S1 | Search service + hint pass | `discovery.rs` / `transaction_search.rs` | P0 |
| **TX3** | S1 | GET search + POST preview-group routes | `api/subscriptions.rs` | P0 |
| **UI1** | S2 | Dual-mode tab shell (DEC-0113) | `SubscriptionsPage.tsx` | P0 |
| **UI2** | S2 | Rich filter bar | `SubscriptionsPage.tsx` | P0 |
| **UI3** | S2 | Tx table + pagination + hints | `SubscriptionsPage.tsx` | P0 |
| **UI4** | S2 | Multi-select confirm flow | `SubscriptionsPage.tsx` | P0 |
| **PT1** | S3 | Patterns sub-tab extraction | `SubscriptionsPage.tsx` | P0 |
| **T1** | S4 | Search + hint integration tests | `subscriptions/` tests | P0 |
| **T2** | S4 | AC-5 regression tests | `subscriptions/` tests | P1 |
| **R1** | S4 | User guide US-0021 | `docs/user-guides/US-0021.md` | P1 |
| **V1** | — | OIDC smoke AC-1..AC-6 | `uat.json` | P0 |

**Count:** 9 mandatory P0 (TX1–TX3, UI1–UI4, PT1, T1, V1) + 2 P1 (T2, R1) = **11** core + **V1** → **12** at `SPRINT_MAX_TASKS`; P2 items excluded from mandatory count.

**Deploy order:** TX1→TX2→TX3 (backend) ∥ UI1 shell; UI2–UI4 after TX3; PT1 after UI1; T1/T2 after backend; V1 last.

### Codebase map (US-0021 slice)

| Path | Role | Touch |
|------|------|-------|
| `backend/src/subscriptions/repository.rs` | SQL search + filters | TX1 |
| `backend/src/subscriptions/discovery.rs` | hint pass reuse | TX2 |
| `backend/src/api/subscriptions.rs` | new routes | TX3 |
| `backend/src/recurrence/detect.rs` | read-only reuse | — |
| `backend/src/subscriptions/detection.rs` | unchanged — AC-5 | — |
| `frontend/src/pages/SubscriptionsPage.tsx` | dual mode + tx explorer | UI1–UI4, PT1 |
| `frontend/src/lib/api.ts` | search + preview types | TX3 |

### Decisions (US-0021)

| ID | Topic | Contract |
|----|-------|----------|
| **DEC-0112** | Transaction search API | GET `/transactions/search`; SQL push-down; 100/page; preview-group POST |
| **DEC-0113** | Dual-mode UX | Transactions default \| Suggested patterns (DEC-0098 frozen) |
| **DEC-0114** | Hint pass boundary | Separate pass; min 60; row hints only; no auto-emit; 500 tx scan cap |

### Risks

| Risk | Mitigation |
|------|------------|
| Dual-mode UI complexity | Transactions default; shared account/payee state |
| Hint pass perf on wide filters | 500 tx cap; account required |
| AC-3 sub-threshold expectation | Document MVP boundary; GATE-HINT-2 P2 |
| Regression on detection | No `detection.rs` edits; dedicated AC-5 tests |
| Geldbereich JOIN on JSON | DEC-0111 proven path; low account count |
| Sprint over 12 tasks | P2 (amount band, index, weak hints) deferred |

### Next phase

`/sprint-plan` — materialize **S0020** from task table; then `/plan-verify` → `/execute`.

`runtime_proof_id`: `runtime-proof-architecture-20260613-us0021-001`

`triad_hot_surface`: architecture § US-0021 appended; decisions DEC-0112..DEC-0114 formalized; spec-pack US-0021 created; nine gates frozen; state checkpoint; post-write `--check` required.

# BUG-0026 — Forecast monthly Income card vs chart mismatch

**Status:** architecture complete (2026-06-13)  
**Discovery:** `discovery-20260613-bug0026` in `handoffs/po_to_tl.md`  
**Research:** [R-0098](research.md#r-0098--bug-0026-forecast-monthly-income-card-vs-chart-mismatch)  
**Decisions:** extends **US-0002** forecast monthly view + **DEC-0089** card/chart independence; **GATE-DEC-1 closed — no new DEC**  
**Sprint:** `/quick` recommended (3–4 tasks)  
**Acceptance:** `docs/product/acceptance.md` rows **BZ**, **CA**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0026-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**Related:** BUG-0012 DONE / Q0014 (monthly bucket attribution — backend correct); **DEC-0089** (category filter scopes trend chart only)

### Root cause (frozen, R-0098 §1)

| Layer | Finding | Symptom |
|-------|---------|---------|
| **Summary cards** | `ForecastPage.tsx` L148–152: `monthlySummary = series[0]` | Income card **0.00** on account **114** repro |
| **Chart** | `MonthlyChart.tsx` maps **full** `series` to x-axis | Income bars ~**3266** from **2026-07** onward |
| **API** | `GET /api/v1/forecast/monthly` returns ordered points; no `summary_month` hint | `series[0]` = partial June (salary not due in remaining days) |
| **Projection** | `project.rs` recurring income per due date — by design | Partial month income **0.00**; July **3266.16** |
| **Category filter** | **DEC-0089** helper text L278–281 | Cards unchanged by filter — must remain after fix |
| **BUG-0012** | Backend bucket attribution correct | **RULED OUT** — not a regression |

**Live probe (2026-06-13):** `GET http://localhost:18080/api/v1/forecast/monthly?account_id=114` — 25 points; `series[0]` 2026-06 income **0.00**; `series[1]` 2026-07 income **3266.16**.

`isolation_scope`: artifact + repo source reads; no host `.env` / secrets read.

### Architecture gates (frozen)

| Gate | Decision | Rationale | Alternatives rejected |
|------|----------|-----------|----------------------|
| **GATE-MONTH-1** | **Option A** — skip partial zero-income head | When `parseFloat(series[0].income) === 0` and `series.length > 1`, select `series.find(p => parseFloat(p.income) > 0) ?? series[1]`; else `series[0]` | **B** always `series[0]` + footnote (**BZ fails**); **C** chart hover sync (deferred); **D** rolling aggregate (semantic change); **E** backend `summary_month` field (**GATE-SCOPE-1**) |
| **GATE-LABEL-1** | Shared subtitle above card grid | `Forecast for {Month YYYY}` — one label for all four cards | Per-card micro-label (redundant); inline metric suffix (fallback only) |
| **GATE-SCOPE-1** | Frontend-only | No `project.rs` / API contract change; **DEC-0089** cards independent of category filter | Backend summary hint; chart selection coupling |
| **GATE-TEST-1** | Vitest pure helper + partial-month fixture | Pattern: `planSelector.test.ts` / `planSelector.ts` (**BUG-0022** / R-0094) | Playwright E2E (0 spec files; defer operator smoke) |
| **GATE-DEC-1** | **No new DEC** | UI presentation fix; document forecast summary month contract in architecture only | Canonical DEC for month-selection policy |

### Forecast summary month contract (frozen)

Pure helpers in new module `frontend/src/pages/forecastSummaryMonth.ts` (colocated with page, mirrors **BUG-0022** `planSelector.ts` pattern).

```typescript
type ForecastMonthlyPoint = ForecastMonthly["series"][number];

function parseIncome(income: string): number {
  return parseFloat(income); // same as MonthlyChart.tsx
}

function resolveForecastSummaryPoint(
  series: ForecastMonthlyPoint[],
): ForecastMonthlyPoint | null {
  if (series.length === 0) return null;
  if (parseIncome(series[0].income) === 0 && series.length > 1) {
    return series.find((p) => parseIncome(p.income) > 0) ?? series[1];
  }
  return series[0];
}

function formatForecastMonthLabel(monthIso: string): string {
  // Derive from API `month` date slice — not client clock (R-0098 §2 month-end boundary)
  const [year, month] = monthIso.slice(0, 7).split("-").map(Number);
  return new Date(year, month - 1, 1).toLocaleDateString(undefined, {
    month: "long",
    year: "numeric",
  });
}

function formatForecastSummarySubtitle(monthIso: string): string {
  return `Forecast for ${formatForecastMonthLabel(monthIso)}`;
}
```

**Operator mental model:** `/forecast` **Monthly** summary cards answer "what does the next meaningful forecast month look like?" When the current calendar month is partial and salary has not yet fallen in the projection window, defaulting to the **first month with projected income** aligns card Income with the first non-zero Income bar operators see in the chart.

**Edge cases (frozen):**

| Case | `resolveForecastSummaryPoint` | Subtitle |
|------|------------------------------|----------|
| `series[0].income > 0` | `series[0]` | That month |
| All months `income === 0` | `series[0]` | That month — card/chart both zero (**BZ** satisfied) |
| Single-month series | `series[0]` | That month |
| Empty series | `null` — hide card grid (unchanged) |
| Category filter set | Cards use **unfiltered** `monthlyQuery` series | Unchanged per **DEC-0089** |

**Optional P2 (not required for BZ/CA):** Footnote when skip rule fires: *"Current month has no remaining projected income events."*

### Fix slices

```text
BUG-0026
├── BZ — summary month selection + Income parity (P0)
│   └── H1 — forecastSummaryMonth.ts helper module
│   └── F1 — ForecastPage wire: useMemo + subtitle above card grid
├── CA — month label (P0)
│   └── F1 — shared subtitle "Forecast for {Month YYYY}" (GATE-LABEL-1)
├── T1 — vitest helper + partial-month fixture (P0)
├── G1 — automated gate (npm test + build) (P0)
└── V1 — verify-work BZ/CA on /forecast Monthly + OIDC smoke (P0; operator account 114)
```

**Out of scope:** `project.rs`, `backend/src/api/forecast.rs`, `MonthlyChart.tsx`, category filter wiring, chart hover/selection sync, new DEC.

### H1 — Pure helper module (GATE-MONTH-1)

| Export | Contract |
|--------|----------|
| `resolveForecastSummaryPoint(series)` | Returns selected point per frozen algorithm; `null` when empty |
| `formatForecastMonthLabel(monthIso)` | Locale month-year from API `month` ISO date |
| `formatForecastSummarySubtitle(monthIso)` | `Forecast for {Month YYYY}` |

**Files:** `frontend/src/pages/forecastSummaryMonth.ts` (new)

**Alternative rejected:** Inline logic in `useMemo` only — harder to unit-test; violates GATE-TEST-1 precedent.

### F1 — ForecastPage integration (GATE-LABEL-1, GATE-SCOPE-1)

| Change | Contract |
|--------|----------|
| `monthlySummary` useMemo | `resolveForecastSummaryPoint(monthlyQuery.data?.series ?? [])` |
| Subtitle | Render `formatForecastSummarySubtitle(monthlySummary.month)` immediately **above** `.grid` card block (L312–330) |
| Card values | Income / Fixed / Variable / Free cashflow from **same** resolved point |
| Category filter | **Do not** add `categoryId` to `monthlyQuery` key or card data path |
| `MonthlyChart` | Unchanged — still plots full `series` |

**Files:** `frontend/src/pages/ForecastPage.tsx` (L148–152 useMemo; L312–330 card grid + new subtitle element)

**Acceptance trace:**

| Row | Mechanism | Verify |
|-----|-----------|--------|
| **BZ** | Skip partial June → July point; Income card **3266.16** matches chart July Income bar | Vitest partial-month fixture; V1 account **114** |
| **CA** | Subtitle **"Forecast for July 2026"** above four cards | Visual on `/forecast` Monthly; vitest label helper |

### T1 — Vitest coverage (GATE-TEST-1)

**Files:** `frontend/src/pages/forecastSummaryMonth.test.ts` (new)

**Fixture (matches live API repro):**

```typescript
const partialMonthTrap = [
  { month: "2026-06-01", income: "0.00", fixed_costs: "86.02", variable_costs: "2866.57", free_cashflow: "-2952.59" },
  { month: "2026-07-01", income: "3266.16", fixed_costs: "86.02", variable_costs: "2866.57", free_cashflow: "313.57" },
];
```

| Case | Expected |
|------|----------|
| Partial-month trap (`series[0].income=0`, `series[1].income>0`) | Resolve index **1**; income **"3266.16"** |
| `series[0].income > 0` | Resolve index **0** |
| All-zero income (multi-month) | Resolve index **0** |
| Single-month series | Resolve index **0** |
| Empty series | `null` |
| `formatForecastMonthLabel("2026-07-01")` | Contains **July** and **2026** |
| `formatForecastSummarySubtitle("2026-07-01")` | **`Forecast for July 2026`** (locale-stable month name) |

**Regression suite:** `npm test` frontend; no backend test changes.

### Risks

| Risk | Mitigation |
|------|------------|
| Skip rule hides partial-month Fixed/Variable on cards | Acceptable — chart shows full series; subtitle names reference month |
| Legitimate all-zero-income forecast | No skip; labeled zero matches chart bar |
| `parseFloat` on decimal strings | Same pattern as `MonthlyChart.tsx` |
| Category filter accidentally wired to cards | Explicit out-of-scope; do not touch `monthlyQuery` key |
| Timezone month label drift | Derive label from API `month` string, not `new Date()` on client clock |
| i18n month names | Browser default locale; consistent with page `toLocaleString` |

### Next phase

`/sprint-plan` — materialize `/quick` from task tree; then `/plan-verify` → `/execute`.

`runtime_proof_id`: `runtime-proof-architecture-20260613-bug0026-001`

`triad_hot_surface`: architecture § BUG-0026 appended; five gates frozen; GATE-DEC-1 no new DEC; spec-pack BUG-0026 created; state checkpoint; post-write `--check` required.

---

# BUG-0024 — Plan delete still disabled (live post-Q0031)

**Status:** architecture complete (2026-06-13)  
**Discovery:** `discovery-20260613-bug0024` in `handoffs/po_to_tl.md`  
**Research:** [R-0096 §1–9](research.md#r-0096--bug-0024-plan-delete-still-disabled-live-post-q0031)  
**Decisions:** extends **DEC-0082** sole-plan UX presentation; **GATE-DEC-1 closed — no new DEC**  
**Sprint:** `/quick` **Q0033** recommended (5 tasks)  
**Acceptance:** `docs/product/acceptance.md` rows **BR**, **BS**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0024-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**Related:** BUG-0022 DONE / **Q0031** (`bug0022-q0031`, selector fix); **US-0022** (deploy version stamp — separate)

### Root cause (frozen, R-0096 §1)

Post-**Q0031** operator report *immer ausgegraut* decomposes into two independent sub-defects — not a selector regression on current bundle.

| Hypothesis | Verdict | Layer | Finding | Symptom |
|------------|---------|-------|---------|---------|
| **H1** | **CONFIRMED (BS)** | Sole-plan UX | `isDeleteDisabled` true by design; tooltip *Set another plan active…* assumes another plan exists | Permanent gray with no create→activate→delete path |
| **H2** | **LIKELY (BR on omniflow)** | Deploy | **FRONTEND_DEPLOY** deferred; omniflow not probed post-Q0031/Q0032 | Stale bundle would reproduce pre-Q0031 **BM** when operator has 2+ plans |
| **H3** | **RULED OUT (localhost)** | Selector | `resolveDisplayedPlanId` + `isDeleteDisabled` correct; vitest 8/8; 2-plan probe delete enabled | Multi-plan non-active selection works |

**Live probe (2026-06-13):** localhost:18080 — 1 plan → delete disabled; after create second plan + select non-active → `deleteDisabled=false`, title *Delete this plan*; served bundle `assets/index-CJ94Af9n.js` includes Q0031 tooltip string.

**API contract unchanged:** `DELETE /api/v1/plans/:id` returns **409** when `is_active=true` per **DEC-0082** — no backend change.

`isolation_scope`: artifact + repo source reads; no host `.env` / secrets read.

### Architecture gates (frozen)

| Gate | Decision | Rationale | Alternatives rejected |
|------|----------|-----------|----------------------|
| **GATE-COPY-1** | **Option A** — inline hint below **Delete plan** row when `plans.length===1 && plans[0].is_active && activePlanIsSelected` | Satisfies **BS**; keyboard/screen-reader discoverable; matches disabled-control UX guidance ([R-0096 §3](research.md#r-0096--bug-0024-plan-delete-still-disabled-live-post-q0031)) | **B** create-plan CTA from disabled row (higher UI churn); **C** delete active sole plan with auto-deactivate (**DEC-0082** violation) |
| **GATE-DEPLOY-1** | Operator **FRONTEND_DEPLOY** (Q0031/Q0032 bundles) then 2-plan `/planning` smoke on omniflow | **BR** localhost already PASS; omniflow **OPEN** until deploy | Code fix for **BR** on localhost (unnecessary — H3 ruled out) |
| **GATE-SCOPE-1** | Frontend-only | Extends **DEC-0082** deactivate-first UX with sole-plan guidance; DELETE 409 unchanged | Backend delete policy change; create-plan API |
| **GATE-TEST-1** | Vitest pure helper `shouldShowSolePlanDeleteHint` | Precedent: `planSelector.test.ts` / **BUG-0022** / **BUG-0026** helper pattern | Playwright E2E (0 spec files; defer operator smoke) |
| **GATE-DEC-1** | **No new DEC** | Presentation layer on existing guard — architecture documents copy contract only | Canonical DEC for sole-plan copy policy |

### Sole-plan delete hint contract (frozen)

Pure helpers in existing module `frontend/src/pages/planSelector.ts` (mirrors **BUG-0022** / **BUG-0026** colocated-helper pattern).

```typescript
export const SOLE_PLAN_DELETE_HINT =
  "To delete this plan, create another scenario, set it active, then delete this one.";

export function shouldShowSolePlanDeleteHint(
  plans: PlanListItem[] | undefined,
  activePlanIsSelected: boolean,
): boolean {
  if (!plans || plans.length !== 1) {
    return false;
  }
  return plans[0].is_active === true && activePlanIsSelected === true;
}
```

**Copy (frozen):** `SOLE_PLAN_DELETE_HINT` — exact string above; English; matches PO Option A intent.

**Placement (GATE-COPY-1):** Block-level `<p>` **immediately below** the plan-selector control row (dropdown + **Set active** + **Delete plan** buttons) inside the plan card — not tooltip-only, not modal. Reuse existing PlanningPage muted helper style (`fontSize: "0.85rem"`, `color: "#64748b"`, `margin: "0.5rem 0 0"` — same pattern as L549, L760).

**Multi-plan behavior unchanged:** When `plans.length >= 2` and globally active plan selected, keep existing disabled button + `title` tooltip *Set another plan active before deleting the active plan* — no inline hint (another plan already exists).

**Optional P2 (not required for BS):** Retain shortened tooltip on disabled button for hover users; inline hint is the **BS** closure artifact.

### Fix slices

```text
BUG-0024
├── BS — sole-plan inline guidance (P0)
│   └── H1 — shouldShowSolePlanDeleteHint + SOLE_PLAN_DELETE_HINT in planSelector.ts
│   └── F1 — PlanningPage conditional inline hint render
├── T1 — vitest sole-plan predicate cases (P0)
├── G1 — automated gate (npm test + build) (P0)
├── BR — deploy verification (P0; operator gate)
│   └── V1 — post-FRONTEND_DEPLOY omniflow 2-plan delete + sole-plan hint smoke
└── (no backend slice)
```

**Out of scope:** `DELETE /api/v1/plans/:id`, **DEC-0082** 409 contract, create-plan API, selector priority change (**BUG-0022** shipped), Playwright suite, omniflow deploy automation.

### H1 — Pure helper + copy constant (GATE-COPY-1, GATE-SCOPE-1)

| Export | Contract |
|--------|----------|
| `SOLE_PLAN_DELETE_HINT` | Frozen English copy string |
| `shouldShowSolePlanDeleteHint(plans, activePlanIsSelected)` | `true` only when exactly one globally active plan and delete guard active |

**Files:** `frontend/src/pages/planSelector.ts` (extend existing module)

**Alternative rejected:** Inline predicate in JSX only — harder to unit-test; violates GATE-TEST-1 precedent.

### F1 — PlanningPage inline hint wire (GATE-COPY-1)

| Change | Contract |
|--------|----------|
| Import | `shouldShowSolePlanDeleteHint`, `SOLE_PLAN_DELETE_HINT` from `./planSelector` |
| Render | When predicate true, `<p>` with `SOLE_PLAN_DELETE_HINT` below button row inside plan card (L640–687) |
| Delete guard | **Unchanged** — `activePlanIsSelected = isDeleteDisabled(...)`; button stays `disabled` |
| Tooltip | **Unchanged** for multi-plan active selection; sole-plan may show both tooltip + inline hint (P2 OK) |

**Files:** `frontend/src/pages/PlanningPage.tsx` (L640–687 plan card; reuse L549 helper-text styling)

**Acceptance trace:**

| Row | Mechanism | Verify |
|-----|-----------|--------|
| **BS** | Inline hint visible when sole active plan + delete disabled | Vitest predicate; V1 sole-plan `/planning` visual |
| **BR** | No selector change — localhost already PASS | Existing vitest 8/8; V1 post-**FRONTEND_DEPLOY** 2-plan omniflow smoke |

### T1 — Vitest coverage (GATE-TEST-1)

**Files:** `frontend/src/pages/planSelector.test.ts` (extend existing suite)

| Case | Expected |
|------|----------|
| Sole plan active + `activePlanIsSelected=true` | `shouldShowSolePlanDeleteHint` → **true** |
| Sole plan active + `activePlanIsSelected=false` | **false** (impossible UI state; guard) |
| Two plans, active selected + delete disabled | **false** (multi-plan uses tooltip only) |
| Two plans, non-active selected + delete enabled | **false** (no hint when delete enabled) |
| Empty / undefined plans | **false** |
| `SOLE_PLAN_DELETE_HINT` | Non-empty; contains *create another scenario* |

**Regression suite:** `npm test` frontend; existing `resolveDisplayedPlanId` / `isDeleteDisabled` cases must remain green.

### GATE-DEPLOY-1 — Operator verification (V1)

| Surface | **BR** / **BS** expectation | Status |
|---------|----------------------------|--------|
| **localhost:18080** | 2+ plans + non-active selected → delete enabled | **PASS** (2026-06-13 probe) |
| **localhost:18080** | 1 sole active plan → delete disabled + **inline hint** | **OPEN** until execute |
| **financegnome.omniflow.cc** | Same after **FRONTEND_DEPLOY** (Q0031 + Q0032 bundles) | **OPEN** — operator gate deferred |

**Smoke checklist (post-deploy):**

1. Confirm deployed bundle includes Q0031 selector + BUG-0024 hint string (or **US-0022** version stamp when available).
2. `/planning` with **2+** plans: select non-active → **Delete plan** enabled → confirm removes plan (**BR**).
3. `/planning` with **1** sole active plan: delete disabled + **inline hint** visible (**BS**).
4. OIDC-enabled deploy regression checks per acceptance.

**If BR fails post-deploy:** treat as deploy/process gap first (**H2**); only reopen selector code if bundle is current and **H3** reproduces.

### Task table (sprint-plan input)

| ID | Sub | Task | Files | Priority | Row |
|----|-----|------|-------|----------|-----|
| **H1** | BS | `shouldShowSolePlanDeleteHint` + `SOLE_PLAN_DELETE_HINT` | `planSelector.ts` | P0 | **BS** |
| **F1** | BS | PlanningPage inline hint wire | `PlanningPage.tsx` | P0 | **BS** |
| **T1** | BS | Vitest sole-plan predicate cases | `planSelector.test.ts` | P0 | **BS** |
| **G1** | all | Automated gate | `npm test`, `npm run build` | P0 | all |
| **V1** | BR/BS | verify-work `/planning` smoke + OIDC | `sprints/quick/Q0033/uat.md` | P0 | **BR**, **BS** |

**Count:** 5 mandatory ≤ `SPRINT_MAX_TASKS` (12) → **`/quick` Q0033** recommended.

**Deploy order:** H1 → F1 → T1 → G1 → operator **FRONTEND_DEPLOY** (frontend rebuild only; no migration) → V1.

### Codebase map (BUG-0024 slice)

| Path | Role | Touch |
|------|------|-------|
| `frontend/src/pages/planSelector.ts` | Pure helper + copy constant | H1 |
| `frontend/src/pages/planSelector.test.ts` | Vitest predicate cases | T1 |
| `frontend/src/pages/PlanningPage.tsx` | Inline hint render | F1 |
| `frontend/src/pages/planningFeedback.test.ts` | 409 message regression | verify only |
| `backend/src/api/plans.rs` | Active delete 409 | **no change** |
| `backend/src/plan/service.rs` | `ActivePlanDeleteForbidden` | **no change** |

### Decisions (BUG-0024)

| Topic | Contract | Existing DEC |
|-------|----------|--------------|
| Sole-plan delete guard | UI disabled + API **409** | **DEC-0082** |
| Selector priority | Operator selection wins (multi-plan delete) | **DEC-0082** §2 frontend (**BUG-0022** / Q0031) |
| Sole-plan guidance | Inline hint when `plans.length===1` + active selected | extends **DEC-0082** UX — architecture only |
| New DEC | **None** — GATE-DEC-1 closed | — |

### Risks

| Risk | Mitigation |
|------|------------|
| Hint shown when delete unexpectedly enabled | Predicate requires `activePlanIsSelected === true` (same guard as disabled button) |
| Copy clutter on multi-plan active selection | Hint gated on `plans.length === 1` only |
| Omniflow **BR** still fails post-deploy | Verify bundle hash / **US-0022** version stamp; only then hunt selector regression |
| Tooltip-only **BS** closure | Architecture requires inline `<p>`, not `title` alone |
| Reopening **BUG-0022** | Explicit out-of-scope — selector logic verified PASS on localhost |
| i18n | English copy constant; matches page default locale |

### Next phase

`/sprint-plan` (role: tech-lead) — materialize `/quick` **Q0033** from task table; then `/plan-verify` → `/execute`.

`runtime_proof_id`: `runtime-proof-architecture-20260613-bug0024-001`

`triad_hot_surface`: architecture § BUG-0024 appended; five gates frozen; GATE-DEC-1 no new DEC; spec-pack BUG-0024 created; state checkpoint; post-write `--check` required.

---

# BUG-0025 — Firefly category transactions not updating in mirror (Stromkosten)

**Status:** architecture complete (2026-06-13)  
**Discovery:** `discovery-20260613-bug0025` in `handoffs/po_to_tl.md`  
**Research:** [R-0097 §1–9](research.md#r-0097--bug-0025-firefly-category-transactions-not-updating-stromkosten)  
**Decisions:** extends **DEC-0002** with manual-trigger 365-day lookback exception; **GATE-DEC-1 closed — no new DEC**  
**Sprint:** `/quick` recommended (6–7 tasks)  
**Acceptance:** `docs/product/acceptance.md` rows **BW**, **BX**, **BY**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0025-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**Related:** BUG-0006 DONE (category ingest path); **DEC-0088** (Category spending trend); **US-0018**; [R-0089](research.md#r-0089--bug-0019-grafana-cashflow-zeros-account_id-default--sync-entity-counts-per-run-cursor) (overlap semantics)

### Root cause (frozen, R-0097 §1–6)

| Layer | Finding | Symptom |
|-------|---------|---------|
| **Ingest** | `sync_transactions` L373–378: `start = watermark − overlap_days` (default **7**); Firefly `start` filters by **transaction date** | Backdated Strom imports outside window **skipped**; sync reports success |
| **Mirror** | `category_id=146` → **4** rows, all **2026-05-11…13** | `/forecast` Category spending trend shows **one month** with bars |
| **Watermark** | `sync_cursors.transactions.last_successful_sync_at=2026-06-13 11:53:28Z` | Next incremental `start ≈ 2026-06-06` — months **2025-07…2026-04** never fetched |
| **Sync Status UX** | `GET /api/v1/sync/status` `last_run.trigger=scheduled_exchanges` (12:53) while last Full `scheduled` (11:53) | Hero **"Last sync"** implies Firefly synced when only exchanges ran (**BY** partial) |
| **Surface** | `GET /api/v1/categories/expense-series?category_id=146` aggregates mirror by month | **Not** a chart rendering bug (**H3 CONFIRMED**) |
| **BUG-0006** | category_id binding works for in-window rows | **RULED OUT** |

**Live probe (2026-06-13):** localhost:18080 — expense-series category **146**: **4** txs, **2026-05** only (~€465 outflow); mirror **939** txs span **2025-06-05 … 2026-06-11**.

`isolation_scope`: artifact + repo source reads; no host `.env` / secrets read.

### Architecture gates (frozen)

| Gate | Decision | Rationale | Alternatives rejected |
|------|----------|-----------|----------------------|
| **GATE-OVERLAP-1** | **A + B** — document DEC-0002 limits + **manual Full 365d lookback**; scheduled keeps `watermark − overlap_days` | **BW** requires ingest on **Sync now**; **BX** requires transparency; scheduled cost unchanged | **A only** (**BW fails**); **C** global overlap bump (scheduled cost inflation, still misses deep backfill); **D** UX-only (no ingest fix) |
| **GATE-SYNC-UX-1** | Split **`last_firefly_run`** from exchange-only; hero **"Last Firefly sync"** + trigger badge; secondary exchange line when newer | Fixes **BY** summary confusion; additive API | **B** relabel only (still ambiguous); **C** hide exchange from header (loses exchange signal) |
| **GATE-REMED-1** | Runbook SQL cursor reset; admin API **deferred** | Safe `upsert` dedupe; covers **>365d** backfill | Silent skip; undocumented SQL only |
| **GATE-TEST-1** | Rust integration: tx dated before incremental `start` → skip pre-fix, ingest after manual lookback or cursor delete | Deterministic repro per R-0097 §6 | Wiremock-only without mirror assert |
| **GATE-DEC-1** | **Extend DEC-0002** — manual-trigger lookback exception documented here; no new DEC | Same upsert + watermark contract; trigger-specific window only | New DEC unless contracts diverge |

### Sync start contract (frozen — GATE-OVERLAP-1)

Extends **DEC-0002** / [R-0002](research.md#r-0002--firefly-incremental-sync-strategy) / [R-0089](research.md#r-0089--bug-0019-grafana-cashflow-zeros-account_id-default--sync-entity-counts-per-run-cursor):

```text
sync_transactions(client, pool, overlap_days, trigger):
  watermark = sync_cursors.transactions.last_successful_sync_at

  if trigger == "manual":
    start_date = (Utc::now() - 365 days).date()     // MANUAL_LOOKBACK_DAYS = 365
  else if watermark present:
    start_date = (watermark - overlap_days).date()  // scheduled + initial scheduled Full
  else:
    start_date = (Utc::now() - 365 days).date()     // existing cold-start path (unchanged)

  fetch GET /api/v1/transactions?start={start_date}
  upsert by Firefly id; advance watermark on success
```

**Operator mental model:**

| Action | Window | Expectation |
|--------|--------|-------------|
| **Sync now** (`POST /api/v1/sync/trigger` → `trigger=manual`, `RunMode::Full`) | **365 days** by transaction date | Pulls backdated imports ≤1 year — **BW** |
| **Scheduled Full** (`trigger=scheduled`) | **watermark − 7d** (config `sync.overlap_days`) | Incremental catch-up per **DEC-0002** |
| **Exchange-only** (`scheduled_exchanges` / `manual_exchanges`) | No Firefly fetch | Must not update **Last Firefly sync** hero |

**>365d backfill:** runbook cursor reset → next Full uses cold-start **365d** path; document limitation in Sync Status callout (**BX**).

**Sizing (939-tx profile):** manual 365d ≈ **2** Firefly pages @ 500 — +1–3s per manual sync; acceptable (**R-0097 §6**).

**Alternative considered:** `min(watermark − overlap, today − 365)` on manual — **rejected**; full 365d on manual is simpler and matches operator “Sync now = refresh my year” expectation.

### B1 — Manual lookback wiring (GATE-OVERLAP-1)

| Change | Contract |
|--------|----------|
| `firefly/mod.rs` `sync_transactions` | Add `trigger: &str` param; apply frozen start contract |
| `sync/mod.rs` `execute_run` | Pass `_trigger` (rename to `trigger`) into `sync_transactions` |
| `scheduled` / cold-start | Unchanged overlap path |
| `manual` | **365d** lookback constant `MANUAL_LOOKBACK_DAYS: i64 = 365` (module-local; not TOML in v1) |

**Files:** `backend/src/firefly/mod.rs` L368–415; `backend/src/sync/mod.rs` L196–230; callers in tests (`firefly_readonly_test.rs` — pass `"scheduled"` or test trigger).

**Risks:** Large ledgers (>10k txs/year) slow manual sync — monitor `records_synced`; pagination already capped; operator-scale OK.

### B2 — Sync status API split (GATE-SYNC-UX-1)

Extend `SyncStatusResponse` (**additive** — no breaking removal):

```rust
pub struct SyncStatusResponse {
    pub state: String,
    pub phase: Option<String>,
    pub active_run_id: Option<Uuid>,
    pub last_run: Option<SyncRunRow>,           // latest run of any kind (unchanged)
    pub last_firefly_run: Option<SyncRunRow>,   // NEW — latest trigger IN ('manual', 'scheduled')
}
```

| Query | SQL filter |
|-------|------------|
| `latest_run()` | `ORDER BY started_at DESC LIMIT 1` (existing) |
| `latest_firefly_run()` | `WHERE trigger IN ('manual', 'scheduled') ORDER BY started_at DESC LIMIT 1` |

**Files:** `backend/src/sync/mod.rs` (`SyncStatusResponse`, `status()`, new helper); OpenAPI/regenerate if project uses openapi derive on this type (verify at execute).

**Frontend types:** `frontend/src/lib/api.ts` — add `last_firefly_run: SyncRun | null`.

### F1 — Sync Status UI (GATE-SYNC-UX-1, GATE-OVERLAP-1 doc tier)

| Element | Contract |
|---------|------------|
| Hero primary | **"Last Firefly sync:"** ← `last_firefly_run?.finished_at` or **"Never"** |
| Trigger badge | Pill/chip on hero: `manual` → **Manual**; `scheduled` → **Scheduled** (raw enum fallback for unknown) |
| Secondary line | When `last_run` exists **and** `last_run.trigger ∈ {scheduled_exchanges, manual_exchanges}` **and** (`last_firefly_run` absent **or** `last_run.started_at > last_firefly_run.started_at`): show **"Last exchange sync:"** + timestamp |
| Info callout | Below hero card — explain **DEC-0002**: scheduled sync uses 7-day overlap by **transaction date**; backdated bulk imports need **Sync now** (365d) or runbook cursor reset; link `docs/engineering/runbook.md` anchor |
| **Sync now** button | Unchanged — `POST /api/v1/sync/trigger` → Full + manual lookback post-fix |
| History table | Keep raw `trigger` column (**BY** already partial) |

**Files:** `frontend/src/pages/SyncStatusPage.tsx` L88–92 hero block + new callout.

**Optional P1 — F3:** `HomePage.tsx` L45–50 dashboard stat — prefer `last_firefly_run` over `last_run` for **"Last sync"** card label **"Last Firefly sync"** when field present; secondary not required on home.

### D1 — Runbook remediation (GATE-REMED-1, GATE-OVERLAP-1 doc tier)

Add section to `docs/engineering/runbook.md`:

| Topic | Content |
|-------|---------|
| Symptom | Category trend / expense-series missing months after Firefly backdated import |
| Cause | **DEC-0002** — Firefly `start` filters by transaction date; scheduled overlap **7d** |
| Fix ≤365d | **Sync now** on `/sync` (manual Full — 365d lookback post-fix) |
| Fix >365d | `DELETE FROM sync_cursors WHERE entity_type = 'transactions';` then manual Full |
| Safety | Upsert by Firefly `id` — no duplicate rows |

### T1 — Integration repro (GATE-TEST-1)

**Pattern:** extend backend integration test harness (wiremock Firefly or test DB seed).

| Step | Assert |
|------|--------|
| Seed watermark + mirror tx **outside** `watermark − 7d` window | Not ingested on `trigger=scheduled` sync |
| Same fixture + `trigger=manual` sync (post-fix) | Row present in `transactions` mirror |
| Optional | Cursor delete + scheduled Full ingests via 365d cold-start |

**Files:** new or extend `backend/tests/` sync transaction window test.

### Acceptance trace

| Row | Mechanism | Tasks | Verify |
|-----|-----------|-------|--------|
| **BW** | Manual Full **365d** lookback ingests multi-month Stromkosten | B1, T1, G1, V1 | expense-series category **146** shows bars per month with mirror data; operator manual **Sync now** |
| **BX** | Ingest path + Sync Status callout + runbook cursor reset | B1, D1, F1, V1 | No silent skip without explanation; remediation documented |
| **BY** | `last_firefly_run` hero + history `trigger` column; manual = Full | B2, F1, G1, V1 | Hero not exchange-only timestamp; **Sync now** → `manual` in history; OIDC smoke |

### Fix slices

```text
BUG-0025
├── BW — manual 365d ingest (P0)
│   └── B1 — sync_transactions trigger + MANUAL_LOOKBACK_DAYS
├── BX — transparency + remediation docs (P0)
│   └── D1 — runbook § backdated Firefly imports
│   └── F1 — SyncStatusPage DEC-0002 callout + hero UX (partial)
├── BY — status API + UI split (P0)
│   └── B2 — SyncStatusResponse.last_firefly_run
│   └── F1 — SyncStatusPage hero, badge, exchange secondary line
│   └── F3 — HomePage last_firefly_run (P1 optional)
├── T1 — integration backdated-window repro (P0)
├── G1 — cargo test + npm test + build (P0)
└── V1 — verify-work BW/BX/BY + OIDC smoke (P0)
```

**Out of scope:** Global `overlap_days` config change; Firefly Search API; admin cursor-reset API; expense-series SQL changes; CategoryTrendChart frontend; new DEC record.

### Decisions (BUG-0025)

| Topic | Contract | Existing DEC |
|-------|----------|--------------|
| Scheduled incremental window | `watermark − overlap_days` | **DEC-0002** |
| Manual Full window | **365d** by transaction date | extends **DEC-0002** § manual exception (architecture) |
| Upsert + watermark | Unchanged | **DEC-0002** |
| Status semantics | `last_firefly_run` vs `last_run` | architecture only |
| New DEC | **None** — GATE-DEC-1 closed | — |

### Risks

| Risk | Mitigation |
|------|------------|
| Manual 365d slow on very large ledgers | Pagination + upsert dedupe; log `records_synced`; operator-scale ~2 pages OK |
| >365d backfill still skipped after manual | Runbook cursor reset + callout (**BX**) |
| Duplicate rows on cursor reset | **DEC-0002** upsert by Firefly `id` |
| API additive field ignored by old frontend | Co-deploy; field optional in TS |
| Exchange-only run shown as Firefly sync | **GATE-SYNC-UX-1** hero uses `last_firefly_run` only |
| Scheduled cost regression | Manual-only widened window — scheduled path unchanged |
| OIDC deploy regression | **V1** smoke on `/sync` |

### Next phase

`/sprint-plan` (role: tech-lead) — materialize `/quick` from task tree (6–7 tasks, under `SPRINT_MAX_TASKS=12`); then `/plan-verify` → `/execute`.

`runtime_proof_id`: `runtime-proof-architecture-20260613-bug0025-001`

`triad_hot_surface`: architecture § BUG-0025 appended; five gates frozen; GATE-DEC-1 extends DEC-0002 (no new DEC); spec-pack BUG-0025 created; state checkpoint; post-write `--check` required.

---

# US-0022 — Deploy version stamp & stale-frontend detection

**Status:** Architecture complete (2026-06-14)
**Discovery:** `discovery-20260614-us0022` in `handoffs/po_to_tl.md`
**Research:** [R-0095](research.md#r-0095--us-0022-deploy-version-stamp--stale-frontend-detection) §6–§12
**Decisions:** No new DEC (GATE-DEC-1 closed; all gates are implementation-level)
**Depends on:** BUG-0023 Q0030 operator deploy confusion (motivation); existing `AppLayout.tsx`, `health/mod.rs`, `backend/Dockerfile`, `frontend/vite.config.ts`
**Sprint:** Single sprint recommended — ~8-10 tasks under `SPRINT_MAX_TASKS=12`; no split needed
**Acceptance:** `docs/product/acceptance.md` § US-0022 (AC-1..AC-6)
**Spec-pack:** `docs/engineering/spec-pack/US-0022-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)
**User guide:** `docs/user-guides/US-0022.md` (`USER_GUIDE_MODE=1`; execute publishes content)

### Problem

Post-BUG-0023 deploy confusion: operators cannot confirm which release is running without `docker inspect` or behavioral guesswork. SPA has no embedded build id; cached `index.html`/chunks may lag backend after partial deploy. `/health` returns `{status: ok}` only — no build provenance.

| Gap | Impact |
|-----|--------|
| `/health` returns `status: ok` only | Operator cannot distinguish pre/post deploy from browser |
| SPA has no embedded build id | Cached chunks may lag backend after partial deploy |
| Release tags live in `handoffs/releases/*` only | Requires shell/`docker inspect` to verify production |

### System context

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Browser — AppLayout.tsx sidebar-footer                                     │
│    Subtle version stamp (short label)                                       │
│    Hover tooltip: release tag + build id + build timestamp (UTC)            │
│    Stale banner: non-blocking, dismissible, reload CTA                      │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ GET /api/v1/meta/build-info (on mount)
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  Backend — meta/mod.rs                                                      │
│    GET /api/v1/meta/build-info → {build_id, release_tag, build_timestamp}   │
│    option_env!("BUILD_ID").unwrap_or("dev") — compile-time                  │
│    Public route (no auth); allowlist fields only                            │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  Docker — backend/Dockerfile (3-stage)                                      │
│    ARG BUILD_ID / RELEASE_TAG / BUILD_TIMESTAMP (global)                    │
│    Builder: ENV → cargo build (Rust env!())                                 │
│    Frontend: RUN BUILD_ID=$BUILD_ID npm run build (Vite define)             │
│    Runtime: LABEL org.opencontainers.image.*                                │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Architecture approach

#### Backend: `meta` module (B1)

New `backend/src/meta/mod.rs` with `BuildInfoResponse` struct and `build_info()` handler. Uses `option_env!()` with fallback to `"dev"` / `"unknown"` — never breaks local dev build. Public route (no auth) — metadata is non-sensitive. Registered in `backend/src/api/mod.rs` via `.merge(meta::routes())`.

**Security:** Allowlist fields only (`build_id`, `release_tag`, `build_timestamp`). `option_env!()` never echoes `.env` or PAT. No secrets exposure per AC-6.

**Alternative rejected:** Extend `/health` with `build` field. Rejected per GATE-META-1 — Kubernetes liveness probe should remain minimal (`{status: ok}`). Dedicated `/api/v1/meta/build-info` is cleaner separation.

#### Docker: ARG chain (B2)

Three global `ARG` declarations before first `FROM`: `BUILD_ID`, `RELEASE_TAG`, `BUILD_TIMESTAMP`. Each stage re-declares `ARG` to access values. Builder stage converts `ARG` → `ENV` for Rust `env!()` compile-time injection. Frontend stage passes env vars to `npm run build` for Vite `define`. Runtime stage adds OCI `LABEL` instructions for `docker inspect`.

**CI invocation:** `docker build --build-arg BUILD_ID=$(git rev-parse --short HEAD) --build-arg RELEASE_TAG=... --build-arg BUILD_TIMESTAMP=... -f backend/Dockerfile .`

**Cache behavior:** `ARG` values affect build cache — expected. Each build produces fresh binary with new metadata. Place `ARG` declarations late in each stage (after dependency `COPY`) to minimize cache invalidation.

#### Frontend: Vite define injection (F1, F2)

`frontend/vite.config.ts` adds `define` block with `__BUILD_ID__` and `__RELEASE_TAG__` as `JSON.stringify(process.env.BUILD_ID || 'dev')`. TypeScript declarations in `frontend/src/vite-env.d.ts`: `declare const __BUILD_ID__: string;` and `declare const __RELEASE_TAG__: string;`.

**Alternative rejected:** `import.meta.env.VITE_BUILD_ID` with `envPrefix`. Rejected because `VITE_*` vars load from `.env` files at dev time — not suitable for Docker build-time injection. `define` is canonical for CI/Docker.

#### Stale detection: on-mount hook (F4)

`frontend/src/hooks/useStaleDetection.ts` — `useEffect` on mount fetches `/api/v1/meta/build-info` with `cache: 'no-store'`. Compares `__BUILD_ID__` to server `build_id`. Mismatch → `stale=true`. Skips when `__BUILD_ID__ === 'dev'` (local dev). Silent fail on network error (`.catch(() => {})`) — non-blocking.

**Alternative rejected:** Periodic poll (every 60s). Rejected per GATE-STALE-1 — on-mount only. Operator tool (not public-facing); long-lived tabs rare; on-mount sufficient for operator pain (post-deploy confusion). Matches Sentry PR #98031 pattern.

#### UI: AppLayout stamp + stale banner (F3, F5)

| Element | Placement | Behavior |
|---------|-----------|----------|
| Version stamp | `AppLayout` sidebar-footer (lines 78-91) | Short build id or release tag fragment; always visible; subtle |
| Tooltip | On hover/focus of stamp | Release tag + build id + build timestamp (UTC) |
| Stale banner | Top of app (above content) | Non-blocking; "New version available — reload"; dismissible; hidden when `!stale` |

**Placement rationale:** Sidebar footer already has OIDC user name + logout — natural location for operator-only stamp. Subtle by default (low visual noise); hover/focus for details per AC-1/AC-2.

### Acceptance trace

| Row | Mechanism | Tasks | Verify |
|-----|-----------|-------|--------|
| **AC-1** | `AppLayout` sidebar-footer stamp | F3, G1, V1 | Subtle label visible; does not dominate primary UX |
| **AC-2** | Tooltip on hover/focus | F3, V1 | Release tag + build id + build timestamp (UTC) visible |
| **AC-3** | `GET /api/v1/meta/build-info` | B1, T1, G1, V1 | Returns `{build_id, release_tag, build_timestamp}`; no secrets |
| **AC-4** | Vite `define` + Dockerfile `ARG` | F1, F2, B2, G1 | Frontend bundle embeds build id at compile time |
| **AC-5** | `useStaleDetection()` + `StaleBanner` | F4, F5, V1 | Mismatch → banner + reload CTA; match → no banner |
| **AC-6** | `/health` unchanged; OIDC smoke | B1, G1, V1 | Liveness `{status: ok}` unchanged; OIDC deploy pass; no secrets |

### Fix slices

```text
US-0022
├── AC-3 — backend metadata endpoint (P0)
│   └── B1 — meta/mod.rs + api/mod.rs registration
├── AC-4 — Docker build-arg chain (P0)
│   └── B2 — Dockerfile ARG/ENV/LABEL
├── AC-4 — Vite define injection (P0)
│   └── F1 — vite.config.ts define block
│   └── F2 — vite-env.d.ts declarations
├── AC-1/AC-2 — UI stamp (P0)
│   └── F3 — AppLayout sidebar-footer + tooltip
├── AC-5 — stale detection (P0)
│   └── F4 — useStaleDetection hook
│   └── F5 — StaleBanner component
├── T1 — integration test (P0)
├── G1 — cargo test + npm test + build (P0)
└── V1 — verify-work AC-1..AC-6 + OIDC smoke (P0)
```

**Out of scope:** Periodic polling; Service Worker; `/health` changes; release-management UI; Grafana metadata panel.

### Decisions (US-0022)

| Topic | Contract | DEC |
|-------|----------|-----|
| Metadata route | Dedicated `/api/v1/meta/build-info` | GATE-META-1 (architecture) |
| Build id format | Git short sha + release tag + UTC timestamp | GATE-BUILD-1 (architecture) |
| Stale detection | On-mount fetch only | GATE-STALE-1 (architecture) |
| UI placement | `AppLayout` sidebar-footer | GATE-UI-1 (architecture) |
| Value source | `option_env!()` compile-time | architecture |
| Vite injection | `define` block (not `import.meta.env`) | architecture |
| New DEC | **None** — GATE-DEC-1 closed | — |

### Risks

| Risk | Mitigation |
|------|------------|
| Secrets in metadata response | Allowlist fields only; `option_env!()` never echoes `.env` |
| Backend-only deploy (no frontend rebuild) | Stale banner explains "New version available — reload"; expected behavior |
| Traefik/browser cache on meta endpoint | `cache: 'no-store'` header; operator hard refresh hint in tooltip |
| Docker `ARG` scope confusion (not re-declared) | Document pattern; test in CI; `option_env!()` fallback to `"dev"` |
| Local dev without `--build-arg` | `option_env!()` returns `"dev"`; stale detection skips dev mode |
| Compile-time `env!()` breaks local dev | Use `option_env!().unwrap_or("dev")` — never breaks build |
| OIDC deploy regression | **V1** smoke on `/sync` or `/` |

**Overall risk:** Low. All risks have clear mitigations. No blocking risks.

### Next phase

`/sprint-plan` (role: tech-lead) — materialize sprint from task tree (~8-10 tasks, under `SPRINT_MAX_TASKS=12`); then `/plan-verify` → `/execute`.

`runtime_proof_id`: `runtime-proof-architecture-20260614-us0022-001`

`triad_hot_surface`: architecture § US-0022 appended (H1 heading per policy); four gates frozen; GATE-DEC-1 closed (no new DEC); spec-pack US-0022 created; state checkpoint; post-write `--check` required.

