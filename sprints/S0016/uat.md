# UAT — Sprint S0016 / US-0015

**Sprint:** S0016  
**Story:** US-0015 — AI-assisted forecast category bucket mapping  
**Phase:** verify-work complete — runtime smoke pending deploy  
**Status:** PASS (code/test); AC-7 pass-with-prerequisites  
**Plan-verified at:** 2026-06-06T18:00:00Z  
**QA verified at:** 2026-06-06T16:52:00Z  
**Verify-work at:** 2026-06-06T18:55:00Z  
**Orchestrator:** `auto-20260606-us0015-001`  
**Decision:** DEC-0078

## Inputs

- Acceptance: `docs/product/acceptance.md` § US-0015
- Architecture: `docs/engineering/architecture.md` § US-0015, `decisions/DEC-0078.md`
- Research: R-0074, R-0075
- User guide: `docs/user-guides/US-0015.md`
- Sprint tasks: `sprints/S0016/tasks.md`

## Operator gate (pre-runtime smoke)

| Gate | Action |
|------|--------|
| **BACKEND_FRONTEND_DEPLOY** | Deploy S1–S3 backend + frontend on US-0010 external profile (`financegnome.omniflow.cc`) |

## Regression checklists (code audit — verify-work)

### AC-1 config precedence (T-0167, T-0170)

| Check | Contract | Audit |
|-------|----------|-------|
| Config guard | Income/Fixed/explicit Variable never AI-overridden | PASS — `resolve_bucket_with_ai` + `config_mapped_salary_never_uses_ai_assignment` |
| Cascade order | Config map before AI fallback | PASS — `categories.rs` + `project.rs` |

### AC-2 AI inference (T-0163, T-0165, T-0168)

| Check | Contract | Audit |
|-------|----------|-------|
| Threshold | 0.74 → Variable; 0.75 → apply AI bucket | PASS — `threshold_boundary_074/075` tests |
| Provider unavailable | Falls back to Variable | PASS — `provider_unavailable_falls_back` |
| Batch cap | Splits at 100 rows | PASS — `batch_cap_splits_at_100` |

### AC-3 privacy (T-0164, T-0166)

| Check | Contract | Audit |
|-------|----------|-------|
| Default privacy | Raw payee stripped under `allow_raw_transactions=false` | PASS — `prepare_bucket_features_strips_raw_payee_by_default` |
| Opt-in cap | ≤50 rows when raw enabled | PASS — `prepare_bucket_features_opt_in_raw_limited_to_50` |

### AC-4 API provenance (T-0169, T-0171)

| Check | Contract | Audit |
|-------|----------|-------|
| Monthly response | `bucket_sources` per bucket + `ai_mapped` flag | PASS — `MonthlyPointResponse` L94–104 |
| Provenance mass | Tracks config/ai/default per accumulation | PASS — `project.rs` L126–127 |

### AC-5 UI badge (T-0172)

| Check | Contract | Audit |
|-------|----------|-------|
| Badge visible | AI-mapped when `ai_mapped=true` | PASS — `ForecastPage.tsx` L259–274 |
| Tooltip | Documents config precedence + privacy bands | PASS — tooltip copy |

### AC-6 audit trail (T-0173)

| Check | Contract | Audit |
|-------|----------|-------|
| Audit tool name | `forecast_bucket_assignment` in `ai_tool_audit` | PASS — `service.rs` L244 |
| Redaction | No raw merchant in payload | PASS — privacy tests |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | Prerequisite | BUG-0012 AG/AH released (Q0014); DEC-0007 baseline authoritative | **pass** | Q0014 release; acceptance L157 |
| UAT-2 | AC-1 | Config-mapped buckets never overridden by AI | **pass** | T-0167, T-0170; `config_mapped_salary_never_uses_ai_assignment` |
| UAT-3 | AC-2 | AI inference with confidence; low-confidence → Variable | **pass** | T-0163, T-0165, T-0168; bucket_inference tests |
| UAT-4 | AC-3 | Privacy allowlist under `allow_raw_transactions=false` | **pass** | T-0164, T-0166; privacy.rs tests |
| UAT-5 | AC-4 | Monthly API `bucket_sources` + `ai_mapped` | **pass** | T-0169, T-0171; api/forecast.rs |
| UAT-6 | AC-5 | Monthly tab AI-mapped badge when `ai_mapped=true` | **pass** | T-0172; ForecastPage.tsx |
| UAT-7 | AC-6 | `forecast_bucket_assignment` audit rows; no raw merchant | **pass** | T-0173; service.rs + privacy tests |
| UAT-8 | AC-7 | OIDC `/forecast` Monthly smoke; chat/ML regression | **pass_with_prerequisites** | T-0174; pending **BACKEND_FRONTEND_DEPLOY** |

## OIDC smoke checklist (AC-7 — operator post-deploy)

| Step | Action | Expected |
|------|--------|----------|
| 1 | Deploy backend + frontend (**BACKEND_FRONTEND_DEPLOY**) | Services healthy on omniflow |
| 2 | Full Firefly sync + forecast recompute | `GET /api/v1/forecast/meta` shows `computation_id` |
| 3 | Open `/forecast` → **Monthly** tab (OIDC session) | Four stat cards render (Income/Fixed/Variable/Free cashflow) |
| 4 | Select funded account with ambiguous recurring rows | `GET /api/v1/forecast/monthly` includes `bucket_sources` when provenance present |
| 5 | Month with AI mass | **AI-mapped** badge visible; tooltip explains config precedence + privacy |
| 6 | Config-only month | No AI-mapped badge |
| 7 | Regression: AI Chat | Six tools unchanged (BUG-0007) |
| 8 | Regression: Forecast Compare / ML tabs | US-0013 ML overlay unchanged |

**Profile:** US-0010 external (`financegnome.omniflow.cc`)  
**Route:** `/forecast` → Monthly tab  
**User guide:** `docs/user-guides/US-0015.md`

## Results summary

| Metric | Count |
|--------|-------|
| Acceptance criteria (AC-1..AC-7) | 7 |
| Pass (code/test) | 6 (AC-1..AC-6) |
| Pass-with-prerequisites | 1 (AC-7) |
| Fail | 0 |
| Prerequisite checked | 1 (BUG-0012 Q0014) |
| UAT steps total | 8 |
| UAT pass | 7 |
| UAT pass-with-prerequisites | 1 |
| UAT fail | 0 |
| Automated checks | 4/4 pass |

**Verdict:** **PASS** — all acceptance rows satisfied; AC-7 runtime smoke deferred pending operator **BACKEND_FRONTEND_DEPLOY** (US-0014/S0015 precedent).

**Traceability:** prerequisite + AC-1..AC-7 → `docs/product/acceptance.md` § US-0015  
**Next phase:** `/release`
