# Design Concept — BUG-0018

## Summary

BUG-0018 fixes a PostgreSQL **42702** ambiguous-column defect in `evaluate_scarcity` that aborts the entire post-sync wealth alert evaluation pass. A one-line SQL qualification (**DEC-0107**) restores projected-path household scarcity semantics per R-0022. Downstream **BF** (empty alerts inbox and header bell) resolves without frontend changes when **BE** is fixed.

## Goals

- **BE:** Post-sync alert evaluation completes without SQL error; logs free of `alert evaluation failed` / 42702
- **BF:** `GET /api/v1/alerts?status=active` and header Alerts panel surface matching wealth alerts when scarcity rules apply — not permanent "No active alerts" from evaluation skip
- Preserve R-0024 warn-only sync semantics (alert eval failure does not fail sync)
- Preserve BUG-0008 subscription alert dedup (regression gate only)
- OIDC-enabled deploy regression checks pass

## Non-goals

- Sync phase fail-on-alert-error (deferred — keep R-0024)
- Frontend error/loading state when eval fails (defer)
- Subscription alert SQL changes (separate sync phase per R-0068)
- CI TimescaleDB service container (optional stretch per R-0088)
- Broader `evaluate.rs` alias refactor (siblings already unambiguous)

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0107 | Qualify `fbd.balance` + `fbd.ts` | Minimal fix; preserves R-0022 projected aggregate |
| Sync semantics | R-0024 warn-only unchanged | Research verdict — no DEC |
| BF scope | Wealth alerts primary; subscription regression gate | R-0088 §5 |
| Sprint shape | `/quick` ≤3 tasks (BE1, T1, V1) | Single-file backend fix + integration proof |

**Spec-pack traceability:** `docs/engineering/spec-pack/BUG-0018-crs.md`, `docs/engineering/spec-pack/BUG-0018-technical-specification.md`
