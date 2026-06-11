# Q0028 progress

**Sprint:** Q0028 (BUG-0020)  
**Status:** EXECUTE COMPLETE (G1 PASS; V1 pending operator deploy)  
**Last updated:** 2026-06-11 (execute, `auto-20260610-bug0019`)  
**Orchestrator:** `auto-20260610-bug0019`

## Task status

| ID | Status | Priority | Notes |
|----|--------|----------|-------|
| DA1 | done | P0 | migration 016 YouTube confirmed merge (idempotent DO block) |
| DB1 | done | P0 | migration 016 DEC-0100 RANK backfill for confirmed patterns |
| DA2 | done | P0 | SubscriptionsPage All-tab excludes rejected/inactive |
| DA3 | done | P0 | detection.rs skip pending on confirmed merge fingerprint conflict |
| T1 | done | P0 | `bug0020_subscription_list_quality.rs` — 7/7 PASS |
| G1 | done | P0 | automated gate PASS (see below) |
| V1 | open | P0 | verify-work — blocked on BACKEND_FRONTEND_DEPLOY + MIGRATION_016_APPLY + FULL_FIREFLY_SYNC |

## G1 automated gate (2026-06-11)

### 1. `cargo test --test bug0020_subscription_list_quality`

```
running 7 tests
test bi_reconcile_merges_youtube_confirmed_cluster ... ok
test bi_reconcile_collapses_strom_pending_cluster ... ok
test bj_backfill_display_category_oracle ... ok
test reg_discover_response_shape_unchanged ... ok
test reg_tag_assign_and_filter_smoke ... ok
test da3_skips_pending_when_confirmed_merge_fingerprint_conflicts ... ok
test migration_016_is_idempotent ... ok

test result: ok. 7 passed; 0 failed; 0 ignored
```

(Requires `DATABASE_URL`; run against TimescaleDB postgres on localhost:5432.)

### 2. Related subscription suites (no regressions)

- `cargo test --test bug0008_subscription_alerts` → **8 passed**
- `cargo test --test subscriptions_integration` → **1 passed**

### 3. Migration 016 idempotency

- Transaction-wrapped `BEGIN`/`COMMIT`; cluster predicates require `COUNT(*) > 1` per status/display_name group with `interval_matches` (±3d).
- `migration_016_is_idempotent` test runs migration SQL twice without error.

### 4. Blast radius (`git diff --stat` frozen scope)

| File | Touch |
|------|-------|
| `backend/migrations/016_bug0020_subscription_list_quality.sql` | **new** |
| `backend/src/subscriptions/detection.rs` | DA3 forward guard |
| `frontend/src/pages/SubscriptionsPage.tsx` | DA2 All-tab filter |
| `backend/tests/bug0020_subscription_list_quality.rs` | **new** T1 |

No changes to `list_patterns` SQL, `compute_display_category_id` algorithm, discover/tags API, or Firefly writes.

## Operator gates (before V1)

1. **BACKEND_FRONTEND_DEPLOY** — ship DA2 + DA3 + migration 016
2. **MIGRATION_016_APPLY** — sqlx migrate or deploy pipeline applies `016_bug0020_subscription_list_quality.sql`
3. **FULL_FIREFLY_SYNC** — detection regression after DA3
