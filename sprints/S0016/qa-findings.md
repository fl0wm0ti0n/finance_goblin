# QA findings — S0016 / US-0015

**Story:** US-0015 — AI-assisted forecast category bucket mapping  
**Sprint:** S0016  
**Phase:** `/qa`  
**Orchestrator:** `auto-20260606-us0015-001`  
**Decision:** DEC-0078  
**QA agent:** fresh subagent (`qa-20260606-s0016-us0015-fresh`)  
**Date:** 2026-06-06  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — Prerequisite and AC-1 through AC-6 satisfied via automated tests and code audit. AC-7 OIDC `/forecast` Monthly smoke is **pass_with_prerequisites** pending operator **BACKEND_FRONTEND_DEPLOY** (consistent with S0015/US-0014 precedent). Hand off to `/verify-work`.

## Test plan

| # | Check | Method | Result |
|---|-------|--------|--------|
| 1 | Prerequisite BUG-0012 | Intake + Q0014 release evidence | **PASS** (pre-verified) |
| 2 | AC-1 config precedence | `project.rs` `config_mapped_salary_never_uses_ai_assignment`; `resolve_bucket_with_ai` | **PASS** |
| 3 | AC-2 AI cascade + threshold + fallback | `bucket_inference.rs` unit tests (0.74/0.75, provider_unavailable, batch cap) | **PASS** |
| 4 | AC-3 privacy allowlist | `privacy.rs` `prepare_bucket_features_*` tests | **PASS** |
| 5 | AC-4 API `bucket_sources` + `ai_mapped` | `api/forecast.rs` `MonthlyPointResponse`; projection provenance | **PASS** |
| 6 | AC-5 Monthly AI-mapped badge | Code audit `ForecastPage.tsx` L259–274 | **PASS** |
| 7 | AC-6 audit trail | `service.rs` `forecast_bucket_assignment`; no raw merchant in privacy tests | **PASS** |
| 8 | AC-7 OIDC smoke + chat/ML regression | UAT template `sprints/S0016/uat.md`; six-tool registry unchanged | **pass_with_prerequisites** |
| 9 | Backend unit tests | `cargo test --lib` | **PASS** (169/169) |
| 10 | Frontend unit tests | `npm test -- --run` | **PASS** (5/5) |
| 11 | User-visible metadata guard | `scripts/check-user-visible-metadata.py` | **SKIP** — entrypoint absent (repo precedent S0013/Q0018) |
| 12 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |

## Generated baseline test evidence (US-0066)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | rust+node (cargo lib + vitest) |
| `generated_test_command` | `cargo test --lib`; `npm test -- --run` |
| `generated_test_result` | pass |
| `generated_test_output_ref` | this file § Automated test output |
| `generated_test_paths_ref` | `backend/src/forecast/{bucket_inference,project,categories}.rs`, `backend/src/ai/privacy.rs`, `frontend/src/pages/ForecastPage.tsx` |
| `generated_test_reason_code` | — |

## Runtime QA evidence (US-0065)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | N/A — code-level QA scope; runtime deferred to verify-work |
| `runtime_stack_profile` | rust+node |
| `runtime_mode` | local |
| `runtime_health_target` | N/A (BACKEND_FRONTEND_DEPLOY gate for AC-7) |
| `runtime_health_result` | deferred |
| `runtime_log_summary` | N/A |
| `runtime_retry_count` | 0 |
| `runtime_retry_ledger` | [] |
| `runtime_final_verdict` | pass_with_prerequisites |
| `runtime_reason_code` | OPERATOR_GATE_BACKEND_FRONTEND_DEPLOY |
| `runtime_evidence_refs` | `sprints/S0016/uat.md`, `handoffs/dev_to_qa.md` |

**Environment label:** local (REMOTE_EXECUTION=1 config; tests executed on dev host, not omniflow).

## Automated test output

```
$ cd backend && cargo test --lib
running 169 tests
test forecast::bucket_inference::tests::threshold_boundary_074_falls_back_to_variable ... ok
test forecast::bucket_inference::tests::threshold_boundary_075_applies_ai_bucket ... ok
test forecast::bucket_inference::tests::provider_unavailable_falls_back ... ok
test forecast::bucket_inference::tests::batch_cap_splits_at_100 ... ok
test ai::privacy::tests::prepare_bucket_features_strips_raw_payee_by_default ... ok
test forecast::project::tests::config_mapped_salary_never_uses_ai_assignment ... ok
test forecast::project::tests::ai_assignment_applies_on_ambiguous_recurring ... ok
test ai::registry::tests::registry_has_six_tools_matching_migration ... ok
test result: ok. 169 passed; 0 failed
EXIT_CODE=0

$ cd frontend && npm test -- --run
 ✓ src/pages/planningFeedback.test.ts (3 tests)
 ✓ src/components/chat/ChatPanel.test.tsx (2 tests)
 Test Files  2 passed (2)
      Tests  5 passed (5)
EXIT_CODE=0
```

## Acceptance criteria results

| AC | Requirement | Result | Evidence |
|----|-------------|--------|----------|
| Prerequisite | BUG-0012 AG/AH released (Q0014); DEC-0007 baseline authoritative | **PASS** | `docs/product/acceptance.md` L157; Q0014 release |
| AC-1 | Config-mapped buckets never overridden by AI | **PASS** | `categories.rs` `resolve_bucket_with_ai`; `project.rs` `config_mapped_salary_never_uses_ai_assignment` asserts `bucket_sources.income == "config"` and `!ai_mapped` despite fake AI Variable assignment |
| AC-2 | AI inference with confidence; low-confidence → Variable | **PASS** | `bucket_inference.rs` threshold 0.74→Variable, 0.75→apply; `provider_unavailable` fallback; `ai_bucket_min_confidence = 0.75` in `default.toml` |
| AC-3 | Privacy allowlist under `allow_raw_transactions=false` | **PASS** | `privacy.rs` `prepare_bucket_features_strips_raw_payee_by_default`, `prepare_bucket_features_opt_in_raw_limited_to_50` |
| AC-4 | Monthly API exposes `bucket_sources` + `ai_mapped` | **PASS** | `api/forecast.rs` `MonthlyPointResponse` L94–104; wired from projection `project.rs` L126–127 |
| AC-5 | Monthly tab **AI-mapped** badge when `ai_mapped=true` | **PASS** | `ForecastPage.tsx` L259–274: badge + tooltip documents config precedence, privacy bands, rolling residual Variable |
| AC-6 | `forecast_bucket_assignment` audit rows; no raw merchant | **PASS** | `service.rs` L244 `tool_name: "forecast_bucket_assignment"`; privacy tests confirm stripped payee |
| AC-7 | OIDC `/forecast` Monthly smoke; chat/ML unchanged | **pass_with_prerequisites** | `sprints/S0016/uat.md` checklist; six-tool registry test unchanged; runtime deferred pending **BACKEND_FRONTEND_DEPLOY** |

## DEC-0078 contract verification

| Element | Contract | Result |
|---------|----------|--------|
| Cascade precedence | config → rule → LLM → Variable | **PASS** |
| Config guard | Income/Fixed/explicit Variable never AI-overridden | **PASS** |
| Threshold | `ai_bucket_min_confidence = 0.75` default | **PASS** |
| Privacy allowlist | `prepare_bucket_features` identical local/cloud | **PASS** |
| API provenance | `bucket_sources` + `ai_mapped` on monthly only | **PASS** |
| Provider reuse | `build_provider()` — no forecast_ai_* split | **PASS** |
| Audit | `forecast_bucket_assignment` redacted payload | **PASS** |
| Rolling residual | Variable only in MVP | **PASS** (`project.rs` rolling path unchanged) |
| Chat/ML isolation | No six-tool or ML overlay changes | **PASS** |

## Findings summary

| ID | Severity | Finding | Blocking US-0015 |
|----|----------|---------|------------------|
| — | — | No findings | — |

**Blocking findings:** 0  
**Critical findings:** 0

## Operator gate (non-blocking for QA)

Runtime omniflow forecast smoke (UAT steps 1–8 in `sprints/S0016/uat.md`) requires operator **BACKEND_FRONTEND_DEPLOY**: deploy S1–S3 backend + frontend on US-0010 external profile (`financegnome.omniflow.cc`), Full Firefly sync + recompute, smoke `/forecast` Monthly tab, verify AI-mapped badge and chat/ML regression. Deferred to `/verify-work`.

## Handoff

**Next phase:** `/verify-work` in fresh subagent/chat  
**Stop reason:** QA_PASS — US-0015 prerequisite + AC-1 through AC-6 verified; AC-7 pass-with-prerequisites; no `handoffs/qa_to_dev.md` required
