# QA findings — S0019 / US-0020

**Story:** US-0020 — Subscription manual discovery, majority category & operator tags  
**Sprint:** S0019  
**Phase:** `/qa`  
**Orchestrator:** `auto-20260608-us0020-001`  
**Decisions:** DEC-0098, DEC-0099, DEC-0100, DEC-0101, DEC-0102, DEC-0103  
**QA agent:** fresh subagent (`qa-20260610-us0020-qa-fresh`)  
**Date:** 2026-06-10  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — All acceptance criteria AC-1 through AC-6 satisfied via code review and automated tests. **0 blocking findings.** Operator omniflow OIDC smoke (AC-1..AC-6 live) deferred to `/verify-work` pending **BACKEND_FRONTEND_DEPLOY** and **FULL_FIREFLY_SYNC**. Hand off to `/verify-work`.

## Test plan

| # | Check | Method | Result |
|---|-------|--------|--------|
| 1 | AC-1 manual discover search | Code review `discovery.rs`, GET `/discover`, Discover tab UI | **PASS** |
| 2 | AC-2 operator confirm from discover | Code review `confirm_from_discover`, POST `/discover/confirm`, UI modal | **PASS** |
| 3 | AC-3 majority display category | Code review RANK SQL + `majority_category_id`; badge + tooltip UI | **PASS** |
| 4 | AC-4 operator tags CRUD/assign/filter | Code review `subscription_tags.rs`, PUT assign, `?tag=` filter, tag manager UI | **PASS** |
| 5 | AC-5 product DB storage; no Firefly write-back | Migration + repository scope; Firefly module unchanged for tags/category | **PASS** |
| 6 | AC-6 US-0003/US-0008 regression | Code review `DetectionPipeline::run_candidates`; alert dedup tests; no alert on manual confirm | **PASS** (code) |
| 7 | DEC-0098 discover API | GET `/api/v1/subscriptions/discover`; cap 50; account/payee/interval filters | **PASS** |
| 8 | DEC-0099 manual confirm | Direct `confirmed` insert; DEC-0085 merge; no `upsert_alert` | **PASS** |
| 9 | DEC-0100 majority category | `display_category_id` column; compute on confirm + merge | **PASS** |
| 10 | DEC-0101 tag schema | `operator_tags` (`name`/`slug`); PATCH rename | **PASS** |
| 11 | DEC-0102 tag assign/filter | PUT `…/tags` replace set; list `?tag=` slug filter; tags on DTO | **PASS** |
| 12 | DEC-0103 Grafana `$tag` (P2) | `subscriptions.json` variable + filtered panel SQL | **PASS** |
| 13 | `cargo test --lib` | QA re-run (`backend/`) | **PASS** (213/213) |
| 14 | `npm test -- --run` | QA re-run (`frontend/`) | **PASS** (9/9) |
| 15 | User guide | Code review `docs/user-guides/US-0020.md` | **PASS** |
| 16 | Operator OIDC smoke | `sprints/S0019/uat.md` template | **DEFERRED** → verify-work |

## Automated test output

```
$ cd backend && cargo test --lib
test result: ok. 213 passed; 0 failed; 0 ignored
EXIT_CODE=0

$ cd frontend && npm test -- --run
Test Files  4 passed (4)
Tests  9 passed (9)
EXIT_CODE=0
```

**US-0020-targeted tests (representative):** `payee_filter_matches_substring`, `interval_filter_uses_tolerance`, `majority_category_picks_mode_then_latest_date`, `majority_category_tie_breaks_by_latest_date`, `majority_category_all_uncategorized_returns_none`, `is_rejected_payee_interval_uses_tolerance`, `slug_normalization`, `validate_tag_name_rejects_empty`, `manual_discover_confirm_does_not_use_alert_fingerprint_path`, `detection_pipeline_min_emit_confidence_unchanged_at_60`.

## Acceptance criteria results

| AC | Requirement | Result | Evidence |
|----|-------------|--------|----------|
| AC-1 | Manual search by account, payee, interval; capped results | **PASS** | `run_discover` filters account/payee/interval; `limit.min(50)`; `DiscoverMeta.truncated`; `SubscriptionsPage.tsx` Discover tab with account select, payee input, interval buckets + custom |
| AC-2 | Operator confirm searched candidate without auto-detection-only path | **PASS** | `confirm_from_discover` inserts `status='confirmed'` directly; POST `/discover/confirm`; UI confirm modal; DEC-0085 merge path; 409 on rejected payee-interval |
| AC-3 | Display category = mode category; tie-break documented | **PASS** | `compute_display_category_id` RANK by count then `MAX(date)`; `majority_category_id` unit tests; badge `title` tooltip on list + detail |
| AC-4 | Tag CRUD; multi-assign; filter by tag | **PASS** | `/api/v1/subscription-tags` GET/POST/PATCH/DELETE; PUT `…/tags` replace; `list_patterns(…, tag_slug)`; Manage tags modal + filter chips |
| AC-5 | Product DB storage; no Firefly write-back | **PASS** | `014_us0020_display_category_tags.sql`; `operator_tags` / `subscription_pattern_tags`; no Firefly client changes for tags or display category |
| AC-6 | US-0003/US-0008 regression; OIDC external smoke | **PASS** (code) | `run_candidates` unchanged thresholds; pending confirm/reject routes preserved; manual confirm has no `upsert_alert`; operator live smoke **deferred** |

## Architecture decision alignment

| DEC | Contract | Result | Notes |
|-----|----------|--------|-------|
| DEC-0098 | GET discover explorer | **PASS** | Reuses `detect_recurrence_groups`; skips confirmed/rejected fingerprints and payee-intervals |
| DEC-0099 | Manual confirm direct to confirmed | **PASS** | No `new_detection` alert; merge on duplicate payee+interval |
| DEC-0100 | Majority `display_category_id` | **PASS** | Computed on confirm + merge refresh; UI badge + tooltip |
| DEC-0101 | `operator_tags` schema; `name` field; PATCH rename | **PASS** | Slug normalization in `tags.rs`; unique slug constraint |
| DEC-0102 | PUT assign + `?tag=` filter | **PASS** | Replace-set semantics; slug filter on list |
| DEC-0103 | Grafana `$tag` variable (P2) | **PASS** | T-0210 shipped; tag dropdown from `operator_tags` |

## Findings summary

| ID | Severity | Finding | Blocking |
|----|----------|---------|----------|
| — | — | No findings | — |

**Blocking findings:** 0  
**Critical findings:** 0  
**Advisory (non-blocking):** 0

## Generated baseline test evidence (US-0066)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | rust+node |
| `generated_test_command` | `cargo test --lib`; `npm test -- --run` |
| `generated_test_result` | pass |
| `generated_test_output_ref` | this file § Automated test output |
| `generated_test_paths_ref` | `backend/` lib tests; `frontend/src/**/*.test.ts(x)` |
| `generated_test_reason_code` | — |

## Runtime QA evidence (US-0065)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | n/a (code-review + unit-test QA scope) |
| `runtime_stack_profile` | rust axum + react vitest |
| `runtime_mode` | local |
| `runtime_health_target` | n/a |
| `runtime_health_result` | n/a |
| `runtime_log_summary` | n/a |
| `runtime_retry_count` | 0 |
| `runtime_retry_ledger` | [] |
| `runtime_final_verdict` | pass |
| `runtime_reason_code` | — |
| `runtime_evidence_refs` | `handoffs/dev_to_qa.md`, automated test output above |

## Isolation / proof

| Field | Value |
|-------|-------|
| `fresh_context_marker` | `qa-20260610-us0020-qa-fresh` |
| `runtime_proof_id` | `runtime-proof-qa-20260610-us0020-001` |
| `phase_boundary` | qa → verify-work |
| `isolation_scope` | QA fresh subagent; artifact/handoff reads only; no prior chat history; no host `.env` / operator secrets read; verify-work not started |

## Handoff

**Next phase:** `/verify-work` in fresh subagent/chat (role: qa)  
**Stop reason:** QA_PASS — no `handoffs/qa_to_dev.md` required
