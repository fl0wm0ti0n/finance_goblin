# Dev → QA handoff — BUG-0021 / Q0029

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-11  
**Bug:** BUG-0021  
**Sprint:** Q0029 (`/quick`)  
**Orchestrator:** `auto-20260611-bug0021`  
**Verdict:** execute **COMPLETE** — ready for `/qa`

## Summary

Implemented DEC-0110 + DEC-0111 for category filter snappy load (**BK**) and wealth Role column (**BL**):

| Task | Status | Deliverable |
|------|--------|-------------|
| EA1 | **done** | `ForecastPage.tsx` — static `CategoryFilter` import; Suspense removed on Monthly tab; `hasForecast` TS6133 removed |
| EA2 | **done** | `WealthPage.tsx` — static `CategoryFilter` import; Suspense removed on Overview card |
| EB1 | **done** | `wealth/repository.rs` — `COALESCE(payload->'attributes'->>'account_role', payload->>'account_role')` |
| EB2 | **done** | `frontend/src/lib/accountRole.ts` + WealthPage Role column `formatAccountRole` |
| EA3 | **done** | `PlanningPage.tsx` — static CategoryFilter parity (P2) |
| T1 | **done** | `bug0021_wealth_account_role.rs` — nested/root/API-shape/categories smoke |
| G1 | **done** | Automated gate PASS (see `sprints/quick/Q0029/progress.md`) |
| V1 | **deferred** | verify-work — operator BACKEND_FRONTEND_DEPLOY + optional SNAPSHOT_UPSERT_OR_SYNC |

## Test results (G1)

- `cargo test --lib` → **213 passed / 0 failed**
- `cargo test --test bug0021_wealth_account_role` → **4 passed / 0 failed** (tests SKIP seed when `DATABASE_URL` unset; QA should re-run with DB for live assertions)
- `npm run build` → **PASS**
- `npm test` → **9 passed / 0 failed**

## Files changed (BUG-0021 blast radius)

- `frontend/src/pages/ForecastPage.tsx`
- `frontend/src/pages/WealthPage.tsx`
- `frontend/src/pages/PlanningPage.tsx`
- `frontend/src/lib/accountRole.ts` (new)
- `backend/src/wealth/repository.rs`
- `backend/tests/bug0021_wealth_account_role.rs` (new)
- `sprints/quick/Q0029/progress.md`

## QA focus

1. Static review: CategoryTrendChart lazy + Suspense unchanged on all three pages.
2. EB1 SQL: COALESCE path in `load_asset_accounts` and test constant (~L133).
3. EB2 label map matches DEC-0111 table; unknown enum → raw string; null → em dash.
4. Re-run `cargo test --test bug0021_wealth_account_role` with `DATABASE_URL` for live nested-payload assertions.
5. V1 remains operator-gated — BK/BL browser + API oracle probes after deploy.

## Next phase

**`/qa`** in fresh subagent/chat (role: qa).

`fresh_context_marker`: execute-20260611-bug0021-dev-fresh  
`runtime_proof_id`: runtime-proof-execute-20260611-bug0021-001  
`phase_boundary`: execute → qa

---

# Dev → QA handoff — BUG-0020 / Q0028

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-11  
**Bug:** BUG-0020  
**Sprint:** Q0028 (`/quick`)  
**Orchestrator:** `auto-20260610-bug0019`  
**Verdict:** execute **COMPLETE** — ready for `/qa`

## Summary

Implemented DEC-0109 two-layer fix for subscription list quality (**BI** duplicates, **BJ** uncategorized display category):

| Task | Status | Deliverable |
|------|--------|-------------|
| DA1 | **done** | Migration 016 YouTube confirmed merge (earliest `confirmed_at` survivor; loser `inactive`; spt/alerts/tags relink) |
| DB1 | **done** | Migration 016 DEC-0100 RANK `display_category_id` backfill (confirmed only) |
| DA2 | **done** | `SubscriptionsPage.tsx` All-tab client filter excludes `rejected` + `inactive` |
| DA3 | **done** | `detection.rs` skip pending INSERT when confirmed merge returns fingerprint conflict (`warn` log) |
| T1 | **done** | `bug0020_subscription_list_quality.rs` — BI/BJ/regression/DA3/idempotency |
| G1 | **done** | Automated gate PASS (see `sprints/quick/Q0028/progress.md`) |
| V1 | **deferred** | verify-work — operator BACKEND_FRONTEND_DEPLOY + MIGRATION_016_APPLY + FULL_FIREFLY_SYNC |

## Test results (G1)

- `cargo test --test bug0020_subscription_list_quality` → **7 passed / 0 failed** (with `DATABASE_URL`)
- `cargo test --test bug0008_subscription_alerts` → **8 passed** (regression)
- `cargo test --test subscriptions_integration` → **1 passed** (regression)

## Files changed

- `backend/migrations/016_bug0020_subscription_list_quality.sql` (new)
- `backend/src/subscriptions/detection.rs`
- `frontend/src/pages/SubscriptionsPage.tsx`
- `backend/tests/bug0020_subscription_list_quality.rs` (new)
- `sprints/quick/Q0028/progress.md`

## QA focus

1. Static review: migration reconcile order (relink before inactive/rejected); `interval_matches` ±3d gate on clusters.
2. DA3: confirmed payee-interval + merge `Ok(false)` → no pending row (integration test `da3_skips_pending_when_confirmed_merge_fingerprint_conflicts`).
3. Blast radius: no `list_patterns` / discover / tag API changes.
4. V1 remains operator-gated — do not expect live BI/BJ oracle until deploy + migration apply.

## Next phase

**`/qa`** in fresh subagent/chat (role: qa).

`fresh_context_marker`: execute-20260611-bug0020-dev-fresh  
`runtime_proof_id`: runtime-proof-execute-20260610-bug0020-001  
`phase_boundary`: execute → qa

---

# Dev → QA handoff — BUG-0019 / Q0027 — FIX CYCLE 2 (re-QA)

**From:** Dev (`/execute`, fix cycle 2 of max 5)  
**To:** QA (`/qa`)  
**Date:** 2026-06-10  
**Bug:** BUG-0019 — Sprint: Q0027 — Orchestrator: `auto-20260610-bug0019`  
**Verdict:** fix cycle 2 **COMPLETE** — all `qa_to_dev.md` return items resolved; ready for QA re-run

## Fix summary (QA return items)

QA FAIL driver was a contract conflict, not an implementation defect: test
`account_id_variable_uses_abs_balance_sort` enforced the DEC-0068 "omit saved
`current`" clause, which DEC-0108 intentionally reverses. Resolution:

1. **Test updated** — `backend/tests/grafana_provisioning_bug0009.rs`
   (L144–147 replacement): asserts `current` is **present** with the DEC-0108
   empty shape (`text == ""`, `value == ""` — no saved concrete value) and
   that the `account_id` variable contains no hardcoded `114`. ABS-balance
   sort + no-alphabetical-only assertions kept unchanged; header doc comment
   notes the amendment.
2. **Supersession recorded** — `decisions/DEC-0108.md` header now carries
   `**Supersedes:** DEC-0068 Y1 clause "Omit current block"…` (existing
   `**Supersedes:**` convention per DEC-0104/DEC-0057);
   `docs/engineering/decisions.md` § DEC-0068 Y1 bullet annotated
   superseded-by-DEC-0108. Note: `decisions/DEC-0068.md` does not exist as a
   standalone file; the canonical DEC-0068 record in
   `docs/engineering/decisions.md` was amended instead.
3. **Tests green** — `cargo test --test grafana_provisioning_bug0009`:
   **6 passed / 0 failed / 0 ignored (6/6 PASS)**. Static guard re-run
   (python json): 12/12 PASS — sort:0 + empty `current` shape in both
   dashboards, versions 2/3/2, no literal `114`, platform-health panel 2
   mirror UNION ALL + `LEFT JOIN sync_cursors`, no bare `records_synced`.

**Not touched:** the three dashboard JSONs (QA verified them correct),
backend/frontend code, migrations. Files changed this cycle:
`backend/tests/grafana_provisioning_bug0009.rs`, `decisions/DEC-0108.md`,
`docs/engineering/decisions.md`, `sprints/quick/Q0027/progress.md`, this file.
Nothing committed. Duplicate-UID provisioning warning remains a pre-existing
V1/operator note (QA classification unchanged).

---

# Dev → QA handoff — BUG-0019 / Q0027 (original execute, cycle 1)

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-10  
**Bug:** BUG-0019  
**Sprint:** Q0027 (`/quick`)  
**Orchestrator:** `auto-20260610-bug0019`  
**Verdict:** execute **COMPLETE** — ready for QA (V1 remains verify-work scope)

## Summary

Q0027 execute delivered the **DEC-0108** provisioning-only Grafana fix in the
frozen order `(CA1→CA2) ∥ CA3 ∥ CB1 → G1`. Exactly **three dashboard JSON
files** edited — no backend, frontend, or migration changes; no hardcoded
account `114`; cursor/`upsert_cursor` semantics untouched. **G1** static guard
all-PASS (python `json` fallback — `jq` not installed). **V1** deferred to
verify-work after operator **GRAFANA_PROVISIONING_RELOAD** + Full sync +
0-new-tx incremental rerun.

## Tasks completed

| ID | Title | Evidence |
|----|-------|----------|
| CA1 | cashflow `$account_id` `sort: 0` + `current` + version 1→2 | `grafana/provisioning/dashboards/analytics/cashflow.json` templating block |
| CA2 | cashflow panels 1–3 latest-success subqueries gain `AND model_kind = 'baseline'` | same file, panels id 1–3 `rawSql` (**4 subqueries** — panel 1 has two targets, A balance + B threshold; asserted per-panel per plan-verify note 2) |
| CA3 | forecast-horizons `$account_id` `sort: 0` + `current` + version 2→3 | `grafana/provisioning/dashboards/analytics/forecast-horizons.json`; no panel `rawSql` changed |
| CB1 | platform-health panel 2 `rawSql` → mirror COUNT(*) UNION ALL over six entities LEFT JOIN `sync_cursors`; version 1→2 | `grafana/provisioning/dashboards/platform-health.json`; `records_synced` column dropped (not relabeled — allowed per DEC-0108) |
| G1 | static guard, results in `sprints/quick/Q0027/progress.md` | all assertion groups **PASS** |
| V1 | verify-work re-provision smoke | **deferred** — blocked on GRAFANA_PROVISIONING_RELOAD + Full sync + 0-new-tx rerun |

## File diffs summary

`git diff --stat` (execute-phase edits only; working tree also carries
pre-existing uncommitted changes from earlier sprints — edits are uncommitted,
diff per plan-verify note 3):

| File | Lines | Change |
|------|-------|--------|
| `analytics/cashflow.json` | 13 (+7/−6) | `sort` 1→0, `current` added, 4× `AND model_kind = 'baseline'`, version 1→2 |
| `analytics/forecast-horizons.json` | 5 (+3/−2) | `sort` 1→0, `current` added, version 2→3 |
| `platform-health.json` | 4 (+2/−2) | panel 2 `rawSql` replaced verbatim per DEC-0108, version 1→2 |

## G1 static guard results (summary)

All PASS — full table in `sprints/quick/Q0027/progress.md`:
`account_id` `sort==0` + `current` non-null in both dashboards; cashflow
panels 1–3 per-panel `model_kind = 'baseline'` (2/2, 1/1, 1/1); platform-health
panel 2 references all six mirror tables + `LEFT JOIN sync_cursors`, no bare
`records_synced`; all three files parse; versions bumped; no `114` literal.

## Implementation notes for QA

1. **`current` shape:** `{ "text": "", "value": "" }` — mirrors the
   `forecast_variant` text/value structure; empty value never matches an
   option, so Grafana with `refresh: 1` resolves to the **first option**
   (highest |balance| per the unchanged SQL `ORDER BY ABS(...) DESC`) on
   dashboard load. This keeps the default deterministic **without hardcoding
   114** (forbidden by DEC-0108).
2. **CA2 count is 4, not 3:** cashflow panel 1 has two targets (balance +
   scarcity threshold), both with the latest-success subquery; both filtered.
   A naive `grep -c` over panels 0–3 returns 4 — expected (plan-verify note 2).
3. **Grafana restart needed** before any runtime check (provisioned dashboards
   load at startup/scan; `version` bump cache-busts).
4. **Local reload sanity check done** (`docker restart finance_goblin-grafana-1`):
   all three files provision-scan with **no parse errors**. However a
   **pre-existing environment issue** surfaced: two provisioning providers
   (`Analytics` and `Flow Finance AI`) scan overlapping paths → duplicate UID
   warnings and `"Not saving new dashboard due to restricted database access"`.
   While duplicates exist, Grafana **refuses to persist re-provisioned
   dashboards**, which can mask the fix at V1. Not introduced by Q0027 (the
   files at HEAD provision the same way) and outside the frozen file boundary —
   flag for operator/V1 (dedupe provider scan paths in Grafana provisioning
   config) and consider a follow-up bug if confirmed.
5. **`jq` not installed** — G1 used python3 `json` equivalents; commands
   reproducible from `progress.md`.

## Frozen-boundary compliance

- No edits to `backend/src/db/mod.rs` `upsert_cursor`, `backend/src/firefly/mod.rs`,
  `AnalyticsEmbedPage.tsx`, or migrations.
- forecast-horizons panel SQL unchanged (`$forecast_variant` per R-0051).
- No embed `var-account_id` forwarding (CA-B deferred).
- Nothing committed to git (no convention in handoff requires it).

## Operator gates (before V1)

1. **GRAFANA_PROVISIONING_RELOAD** — `docker compose restart grafana` (done
   once locally as sanity check; operator must still gate V1 on it).
2. **BH proof sequence** — Full sync (transactions = 922) then 0-new-tx
   incremental rerun; panel 2 must hold the mirror count both times.
3. Verify kiosk embed **and** direct Grafana URL; OIDC omniflow re-check.

`fresh_context_marker`: execute-20260610-bug0019-dev-fresh  
`runtime_proof_id`: runtime-proof-execute-20260610-bug0019-001  
`phase_boundary`: execute → qa

---

# Dev → QA handoff — BUG-0018 / Q0026

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-10  
**Bug:** BUG-0018  
**Sprint:** Q0026 (`/quick`)  
**Orchestrator:** `intake-20260609-ui-audit`  
**Verdict:** execute **COMPLETE** — ready for QA

## Summary

Q0026 execute delivered **DEC-0107** scarcity SQL qualification (**BE1**) — qualify `fbd.balance` and `fbd.ts` in `evaluate_scarcity` daily aggregate query. **T1** regression gate verified via `wealth_alerts_integration` (static + module tests PASS; live DB path skipped without `DATABASE_URL`). **V1** operator runtime smoke deferred to verify-work after **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**.

## Tasks completed

| ID | Title | Evidence |
|----|-------|----------|
| BE1 | Qualify `fbd.balance` + `fbd.ts` in `evaluate_scarcity` | `backend/src/alerts/evaluate.rs` L21–31 |
| T1 | `wealth_alerts_integration` regression gate | `backend/tests/wealth_alerts_integration.rs` |
| V1 | verify-work sync + alerts smoke | **deferred** — blocked on deploy + Full sync gates |

## Test results

| Check | Result |
|-------|--------|
| `cargo test --lib` | **213/213 PASS** |
| `cargo test --test wealth_alerts_integration` | **3/3 PASS** (scarcity integration skipped — `DATABASE_URL` unset) |

## Decision alignment notes

- **DEC-0107:** `SELECT fbd.ts::date AS day, SUM(fbd.balance::float8) AS balance`; date bounds and GROUP BY use `fbd.ts::date`; aggregation reads forecast path only — not `a.balance`.
- **R-0024:** Sync warn-only semantics unchanged — no sync fail-on-alert-error.
- **Frozen boundaries:** No migration; no frontend change; no sibling evaluator edits; subscription dedup regression is V1-only per BUG-0008.

## Operator gates (deferred live smoke)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild `flow-finance-ai` with BE1 `evaluate_scarcity` SQL fix
2. **FULL_FIREFLY_SYNC** — trigger sync; alerts phase must complete without 42702 / `alert evaluation failed`

## QA focus

- Acceptance rows **BE**, **BF** trace vs `sprints/quick/Q0026/tasks.md` and `docs/product/acceptance.md`
- **DEC-0107** contract review — qualified columns in SELECT, WHERE, GROUP BY only
- Confirm no unqualified `balance`/`ts` or `a.balance` in `evaluate_scarcity`
- T1: recommend `DATABASE_URL=… cargo test --test wealth_alerts_integration` when DB available
- V1 runtime probes (sync logs, `GET /api/v1/alerts`, header bell, subscription dedup) deferred to verify-work

## Artifacts

- `sprints/quick/Q0026/{progress.md,summary.md,tasks.md,sprint.json}`
- `handoffs/plan_verify_to_execute.md`
- `decisions/DEC-0107.md`

`fresh_context_marker`: execute-20260610-bug0018-dev-fresh  
`runtime_proof_id`: runtime-proof-execute-20260610-bug0018-001  
`phase_boundary`: execute → qa

**Next:** `/qa` in fresh subagent/chat (role: qa). Do not begin QA in this subagent.
