# QA Findings — Quick Q0011 / BUG-0004

**Work item:** BUG-0004 (defect)  
**Quick task:** Q0011  
**QA phase:** `/qa`  
**Date:** 2026-06-05  
**Verdict:** **PASS** (ready for deploy → operator Full Firefly sync → `/verify-work` L3)

## Scope

Post-sync pipeline empty analytics per `architecture-20260605-bug0004` (`handoffs/tl_to_dev.md`):

- **I1** — `finish_sync_run` on `RunMode::ExchangesOnly` terminal path
- **K1** — Portfolio pie panel id 8 UNION subquery parentheses wrap
- **L1** — `parse_split_amount` for Firefly `current_balance` (DEC-0060)
- **L2** — `COALESCE(balance, 0) >= 0` in `load_asset_accounts`
- **J1** — `extract_payee_source` priority chain + `by_payee()` (DEC-0061)
- **J2** — Subscriptions empty-state threshold copy + pending-review banner
- **L3** — Omniflow runtime probes (I–L) — **deferred** to verify-work

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0011/summary.md`, `sprints/quick/Q0011/plan-verify.json`, `docs/product/acceptance.md` (BUG-0004 rows I/J/K/L), `docs/engineering/architecture.md` (§ BUG-0004), `decisions/DEC-0060.md`, `decisions/DEC-0061.md`, `backend/src/sync/mod.rs`, `backend/src/firefly/mod.rs`, `backend/src/wealth/repository.rs`, `backend/src/recurrence/group.rs`, `grafana/provisioning/dashboards/analytics/portfolio.json`, `frontend/src/pages/SubscriptionsPage.tsx`, `sprints/quick/Q0011/uat.md`, `sprints/quick/Q0011/progress.md`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Backend unit tests | `cd backend && cargo test --lib` | **PASS** (110/110) |
| T-2 | Frontend unit tests | `cd frontend && npm test` | **PASS** (2/2) |
| T-3 | Frontend production build | `cd frontend && npm run build` | **PASS** |
| T-4 | I1 ExchangesOnly finish | Static review + `sync::tests` | **PASS** |
| T-5 | I1 Full no double-finish | Static review — `finish_sync_run` gated `mode == ExchangesOnly` after exchanges | **PASS** |
| T-6 | K1 portfolio UNION SQL | Static review — parenthesized branches in panel id 8 | **PASS** |
| T-7 | L1 balance parse (DEC-0060) | Static review + `firefly::sync::tests` | **PASS** |
| T-8 | L2 NULL balance filter | Static review + `wealth::repository::tests` | **PASS** |
| T-9 | J1 payee fallbacks (DEC-0061) | Static review + `recurrence::group::tests` (3 tests) | **PASS** |
| T-10 | J2 subscriptions UX | Static review — threshold copy + pending banner logic | **PASS** |
| T-11 | Frozen boundaries | No BUG-0005/0006 merge; no migration backfill; no stuck-row cleanup | **PASS** |
| T-12 | Rows I/J/K/L live smoke | Omniflow deploy + Full Firefly sync + L3 probes | **DEFERRED** — verify-work |

### Environment dependencies (non-blocking)

- **Operator deploy:** Q0011 backend + Grafana + frontend image to omniflow before L3.
- **Full Firefly sync gate:** Account balance backfill via DEC-0002 upsert required before wealth/forecast probes **(L)**.
- **Historical stuck `running` rows:** Forward fix only (I1); out of scope per frozen boundary.

## Acceptance criteria matrix (BUG-0004)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **(I)** | Exchange sync terminal status; `finished_at` set after exchange phase | **PASS** (code) / **DEFERRED** (runtime) | I1: `finish_sync_run` when `mode == ExchangesOnly` after `run_exchanges_and_alerts`; unit tests `exchanges_only_finishes_success_on_ok` / `_failed_on_err`. Live probe **DEFERRED** until deploy |
| **(K)** | Portfolio Grafana SQL no UNION syntax error; ds/query **200** | **PASS** (code) / **DEFERRED** (runtime) | K1: parenthesized `(SELECT … ORDER BY … LIMIT 1) UNION ALL (SELECT …)` in `portfolio.json`. Live ds/query **DEFERRED** until deploy |
| **(L)** | Wealth + forecast account-level data after sync/recompute | **PASS** (code) / **DEFERRED** (runtime) | L1: `parse_split_amount` for `current_balance`; L2: `COALESCE(balance, 0) >= 0`. Full sync backfill gate documented. Live GET wealth/forecast **DEFERRED** until deploy + Full sync |
| **(J)** | Subscription detection surfaces merchants **or** documents thresholds in UI | **PASS** (code) / **DEFERRED** (runtime) | J1: description → counterparty → destination priority; J2: ≥3 txs / ≥60% copy + pending banner + Full-sync-only note. Live UX **DEFERRED** until deploy + Full sync |
| Regression | OIDC-enabled + bundled-firefly deploy checks | **DEFERRED** | plan-verify ADV-1; verify-work uat.md |

**Summary:** I1–J2 **PASS** on static/automated path; full acceptance runtime deferred with `OPERATOR_DEPLOY_PENDING` + `OPERATOR_FULL_FIREFLY_SYNC_PENDING`.

## Architecture compliance

### I1 — Exchange sync terminal status

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| ExchangesOnly success | `finish_sync_run(..., "success", None)` | `sync/mod.rs` L328–336 after `run_exchanges_and_alerts` | PASS |
| ExchangesOnly error | `finish_sync_run(..., "failed", Some(...))` + clear phase/active_run | L337–341 on error path | PASS |
| Full mode guard | No second `finish_sync_run` after exchanges | Full finishes only in Firefly phase (L236–257); post-exchanges block gated `ExchangesOnly` only | PASS |
| Unit tests | Both terminal paths | `exchanges_only_finishes_success_on_ok` / `_failed_on_err` | PASS |

### K1 — Portfolio Grafana UNION SQL

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Panel id 8 | Parenthesized ORDER BY/LIMIT branches | `portfolio.json` L80 — both branches wrapped before `UNION ALL` | PASS |
| Scope | Panel 8 only | No other panel changes in deliverable | PASS |

### L1 — Account balance parse (DEC-0060)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Parse helper | Reuse `parse_split_amount` | `firefly/mod.rs` L261 — not `.as_f64()` only | PASS |
| String + number | Both accepted | `account_current_balance_parses_string_or_number` test | PASS |

### L2 — Wealth NULL balance filter

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| SQL predicate | `COALESCE(balance, 0) >= 0` | `wealth/repository.rs` L36 | PASS |
| Unit contract | SQL fixture asserts COALESCE | `load_asset_accounts_includes_null_balances_via_coalesce` | PASS |

### J1 — Payee key fallbacks (DEC-0061)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Priority chain | description → counterparty_name → destination_name | `extract_payee_source` L21–28 | PASS |
| Grouping | `by_payee()` uses `payee_key(extract_payee_source(...))` | L41–44 | PASS |
| Tests | Counterparty-only, description priority, fallback | 3 tests in `recurrence/group.rs` | PASS |

### J2 — Subscriptions empty-state UX

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Threshold copy | ≥3 txs, ≥60% confidence, Full-sync-only | `SubscriptionsPage.tsx` L197–201 | PASS |
| Pending banner | When filtered tab empty but pending count > 0 | `showPendingBanner` L130–134, banner L179–188 | PASS |

### Frozen boundaries

| Boundary | Verdict |
|----------|---------|
| No merge with BUG-0005/0006 scope | PASS |
| No SQL migration backfill — DEC-0002 upsert on Full sync only | PASS |
| No deploy-time cleanup of historical stuck `sync_runs` | PASS |
| L3 deferred to verify-work after deploy + Full sync | PASS |

## Generated test evidence

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` + `typescript` |
| `generated_test_command` | `cd backend && cargo test --lib && cd ../frontend && npm test && npm run build` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-06-05 — 110/110 lib tests; vitest 2/2; tsc + vite build exit 0 |
| `generated_test_paths_ref` | `backend/src/sync/mod.rs` (I1), `backend/src/firefly/mod.rs` (L1), `backend/src/wealth/repository.rs` (L2), `backend/src/recurrence/group.rs` (J1), `grafana/provisioning/dashboards/analytics/portfolio.json` (K1), `frontend/src/pages/SubscriptionsPage.tsx` (J2) |

## Runtime QA evidence (omniflow)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed (operator-owned deploy + Full Firefly sync) |
| `runtime_stack_profile` | `docker-compose` external profile |
| `runtime_mode` | `deferred` |
| `runtime_health_target` | BUG-0004 rows I/J/K/L + regression on omniflow |
| `runtime_health_result` | `deferred` |
| `runtime_final_verdict` | `deferred` (verify-work L3 / operator) |
| `runtime_reason_code` | `OPERATOR_DEPLOY_PENDING`, `OPERATOR_FULL_FIREFLY_SYNC_PENDING` |
| `runtime_evidence_refs` | `handoffs/dev_to_qa.md`; `sprints/quick/Q0011/uat.md`; `docs/engineering/architecture.md` § BUG-0004 |

## Findings

### Blockers

None for proceeding to deploy → Full Firefly sync → `/verify-work` L3.

### Advisories (non-blocking)

1. **Deploy Q0011** image (backend + Grafana + frontend) to omniflow before verify-work.
2. **Operator gate:** Trigger **manual Full Firefly sync** after deploy for L1 balance backfill before wealth/forecast probes.
3. **L3:** Exchange-only sync validates **(I)** only; subscription detection **(J)** requires Full sync per DEC-0018 / J2 copy.
4. **Historical stuck rows:** I1 forward fix only; acceptance probes new runs post-deploy.
5. **Acceptance checkbox:** `docs/product/acceptance.md` BUG-0004 remains unchecked until verify-work L3 passes.

## Verdict

**PASS** — proceed to deploy → operator **Full Firefly sync** → `/verify-work` L3 in a fresh subagent/chat. No dev rework required; do not populate `handoffs/qa_to_dev.md`.
