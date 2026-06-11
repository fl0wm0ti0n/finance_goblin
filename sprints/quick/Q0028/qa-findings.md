# QA Findings — Quick Q0028 / BUG-0020

**Work item:** BUG-0020 (defect)  
**Quick task:** Q0028  
**QA phase:** `/qa`  
**Date:** 2026-06-11  
**Orchestrator:** `auto-20260610-bug0019`  
**Decisions:** DEC-0109  
**QA agent:** fresh subagent (`qa-20260611-bug0020-qa-fresh`)

## Verdict

**PASS** — Independent re-run confirms DEC-0109 implementation: migration 016 SQL reconcile order correct; DA2 All-tab filter and DA3 forward pending guard match contract; integration suite **7/7 PASS** with `DATABASE_URL`; regression suites **8/8** + **1/1 PASS**. Live mirror still pre-migration (migration 015 max) — BI/BJ runtime oracles correctly deferred to verify-work after operator deploy gates.

## Scope

**Inputs reviewed:** `handoffs/dev_to_qa.md` (Q0028 top section), `sprints/quick/Q0028/progress.md`, `decisions/DEC-0109.md`, `docs/product/backlog.md` BUG-0020, `backend/migrations/016_bug0020_subscription_list_quality.sql`, `backend/src/subscriptions/detection.rs`, `frontend/src/pages/SubscriptionsPage.tsx`, `backend/tests/bug0020_subscription_list_quality.rs`. Read-only DB probes on `flow_finance_ai` postgres. No host `.env`/secret files read.

## Test plan

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | BUG-0020 integration suite | `DATABASE_URL=… cargo test --test bug0020_subscription_list_quality` | **PASS** — 7/7 (0.16s; not skip) |
| T-2 | Subscription alerts regression | `cargo test --test bug0008_subscription_alerts` | **PASS** — 8/8 |
| T-3 | Subscriptions integration regression | `cargo test --test subscriptions_integration` | **PASS** — 1/1 |
| T-4 | Migration 016 reconcile order | Static SQL review | **PASS** — spt/alerts/tags relink before loser `inactive`/`rejected`; `interval_matches` ±3d via `MAX-MIN <= 3`; loser fingerprint prefixed `merged:` before inactive |
| T-5 | DA2 All-tab contract | `SubscriptionsPage.tsx` L299–304 | **PASS** — All tab excludes `rejected` + `inactive`; Pending/Standing unchanged |
| T-6 | DA3 forward guard | `detection.rs` L63–87 | **PASS** — confirmed payee-interval + `merge_confirmed_pattern` false → `warn` + `continue` (no pending INSERT) |
| T-7 | Blast radius | `git diff --stat` + untracked list | **PASS** — 4 files only (migration + test new; detection.rs + SubscriptionsPage.tsx modified); no `list_patterns`/discover/tag API changes |
| T-8 | Live DB pre-migration state | Read-only `subscription_patterns` probe | **EXPECTED** — YouTube 2× confirmed, Strom 2× pending + 1× rejected; 0/7 confirmed with `display_category_id`; `_sqlx_migrations` max version 15 (016 not applied) |
| T-9 | V1 operator smoke | BACKEND_FRONTEND_DEPLOY + MIGRATION_016_APPLY + FULL_FIREFLY_SYNC | **DEFERRED** — verify-work |

### T-1 output

```
running 7 tests
test bj_backfill_display_category_oracle ... ok
test bi_reconcile_merges_youtube_confirmed_cluster ... ok
test bi_reconcile_collapses_strom_pending_cluster ... ok
test migration_016_is_idempotent ... ok
test da3_skips_pending_when_confirmed_merge_fingerprint_conflicts ... ok
test reg_discover_response_shape_unchanged ... ok
test reg_tag_assign_and_filter_smoke ... ok
test result: ok. 7 passed; 0 failed
```

## Acceptance row status (qa-stage)

| Row | qa-stage evidence | Status |
|-----|-------------------|--------|
| **BI** | Migration reconcile + DA2 filter + DA3 guard + integration tests | **PASS** at qa — live API/UI oracle deferred V1 |
| **BJ** | DEC-0100 RANK backfill in migration 016 + oracle test (netflix/kindle/youtube/hgp/florian) | **PASS** at qa — live category column deferred V1 |

## Non-blocking notes (carry to verify-work)

- Tests without `DATABASE_URL` exit 0 via early return (skip) — QA re-ran with live postgres; operators should set `DATABASE_URL` for integration gates.
- Migration 016 + new test file are **untracked** in working tree (not committed per sprint policy).

## Handoff

- **Next phase:** `/verify-work` (role: qa) — see `handoffs/qa_to_verify_work.md`
- **No return items**

`fresh_context_marker`: qa-20260611-bug0020-qa-fresh  
`runtime_proof_id`: runtime-proof-qa-20260610-bug0020-001  
`phase_boundary`: qa → verify-work
