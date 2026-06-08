# Dev → QA handoff — US-0020 / S0019

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-10  
**Story:** US-0020  
**Sprint:** S0019  
**Orchestrator:** `auto-20260608-us0020-001`  
**Verdict:** execute **COMPLETE** — ready for QA

## Summary

S0019 execute delivered discover explorer, manual confirm-from-discover, majority display category, operator tags (CRUD + assign + filter), user guide, regression tests, UAT template, and optional Grafana `$tag` per DEC-0098..DEC-0103.

## Tasks completed

| ID | Title | Evidence |
|----|-------|----------|
| T-0198 | Migration `display_category_id` + tag tables | `backend/migrations/014_us0020_display_category_tags.sql`, `types.rs` |
| T-0199 | Discover service + GET `/discover` | `backend/src/subscriptions/discovery.rs`, `api/subscriptions.rs` |
| T-0200 | Discover tab UI | `frontend/src/pages/SubscriptionsPage.tsx`, `lib/api.ts` |
| T-0201 | POST `discover/confirm` + merge | `repository.rs::confirm_from_discover`, DEC-0085 merge |
| T-0202 | Majority category compute | `repository.rs::compute_display_category_id`, `majority_category_id` |
| T-0203 | Majority badge + tooltip | `SubscriptionsPage.tsx` |
| T-0204 | Tag CRUD API | `api/subscription_tags.rs` (PATCH rename per DEC-0101) |
| T-0205 | PUT tag assign + `?tag=` filter | `api/subscriptions.rs`, `repository.rs` |
| T-0206 | Tag manager + filter chips | `SubscriptionsPage.tsx` |
| T-0207 | User guide | `docs/user-guides/US-0020.md` |
| T-0208 | Regression tests | `detection.rs`, `repository.rs`, `discovery.rs`, `tags.rs` tests |
| T-0209 | UAT template | `sprints/S0019/uat.md`, `uat.json` |
| T-0210 | Grafana `$tag` (P2) | `grafana/.../subscriptions.json` |

## Test results

| Check | Result |
|-------|--------|
| `cargo test --lib` | **213/213 PASS** |
| `npm test -- --run` | **9/9 PASS** |

## Decision alignment notes

- Tag field **`name`** (not `label`) per DEC-0101 at execute
- Tag rename uses **PATCH** per DEC-0101 (not PUT)
- `DetectionPipeline::run_candidates` unchanged (AC-6)
- Manual discover confirm does not call `upsert_alert` (DEC-0099)

## Operator gates (deferred live smoke)

1. **BACKEND_FRONTEND_DEPLOY** — deploy S0019 backend + frontend on US-0010 external profile
2. **FULL_FIREFLY_SYNC** — mirror transactions + categories for discover + majority category

## QA focus

- AC-1..AC-6 trace vs `sprints/S0019/uat.json`
- DEC-0098..DEC-0103 contract review
- Regression: pending confirm/reject, alert dedup, DEC-0085 merge paths
- No Firefly write-back for tags or `display_category_id`

## Artifacts

- `sprints/S0019/{progress.md,summary.md,uat.md,uat.json}`
- `docs/user-guides/US-0020.md`
- `handoffs/plan_verify_to_execute.md`

`fresh_context_marker`: execute-20260610-us0020-dev-fresh  
`runtime_proof_id`: runtime-proof-execute-20260610-us0020-001  
`phase_boundary`: execute → qa

**Next:** `/qa` in fresh subagent/chat (role: qa). Do not begin QA in this subagent.
