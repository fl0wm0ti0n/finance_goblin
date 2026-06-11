# Tasks — Q0026 (BUG-0018)

**Bug:** BUG-0018  
**Task count:** 3 (all P0 mandatory; 3/12 under `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260610-q0026-bug0018`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **BE1** | Task **BE1** | DEC-0107 `SUM(fbd.balance::float8)`, `fbd.ts::date`, `GROUP BY fbd.ts::date` |
| **T1** | Task **T1** | Existing `wealth_snapshot_and_scarcity_alert_on_post_sync` must PASS |
| **V1** | Task **V1** | verify-work BE log + BF wealth inbox + subscription dedup regression |

## Execute order

```text
BE1 (evaluate_scarcity SQL qualification)
  → T1 (wealth_alerts_integration regression)
  → single backend release
  → operator: BACKEND_FRONTEND_DEPLOY + FULL_FIREFLY_SYNC
  → V1 verify-work
```

**Parallelism:** None — T1 depends on BE1; V1 blocked on deploy + sync.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **BE** | BE1, T1, V1 | Alerts phase completes without 42702; sync logs free of `alert evaluation failed` |
| **BF** | BE1, V1 | `GET /api/v1/alerts?status=active` returns rows when scarcity matches; header bell non-empty; `GET /api/v1/subscriptions/alerts` dedup regression |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| BE1 | Qualify `fbd.balance` + `fbd.ts` in `evaluate_scarcity` | 1h | done | **BE** | P0 |
| T1 | `wealth_alerts_integration` regression gate | 1h | done | **BE** | P0 |
| V1 | verify-work sync + alerts smoke | 1.5h | open | **BE**, **BF** | P0 |

---

## BE1 — Qualify `fbd.balance` + `fbd.ts` in `evaluate_scarcity`

**Status:** done  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0018 **BE** — **DEC-0107**

### Description

Fix ambiguous column references in `evaluate_scarcity` daily aggregate query per DEC-0107:

```sql
SELECT fbd.ts::date AS day, SUM(fbd.balance::float8) AS balance
FROM forecast_balance_daily fbd
JOIN accounts a ON a.firefly_id = fbd.account_id
WHERE fbd.computation_id = $1
  AND a.type = 'asset'
  AND COALESCE((a.payload->>'include_net_worth')::boolean, true) = true
  AND fbd.ts::date >= $2 AND fbd.ts::date <= $3
GROUP BY fbd.ts::date
ORDER BY day
```

**Forbidden:** unqualified `balance` or `ts`; `a.balance` or `SUM(a.balance::float8)`.

**Files:** `backend/src/alerts/evaluate.rs` — `evaluate_scarcity` only

### Done when

- [x] `fbd.balance` and `fbd.ts` qualified in SELECT, WHERE, GROUP BY
- [x] No migration or sibling evaluator changes
- [x] `cargo build` PASS

---

## T1 — `wealth_alerts_integration` regression gate

**Status:** done  
**Depends on:** BE1  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0018 **BE** — **DEC-0107**

### Description

Ensure existing integration test `wealth_snapshot_and_scarcity_alert_on_post_sync` PASS when `DATABASE_URL` set. Test seeds `forecast_balance_daily` + `accounts` JOIN fixture and exercises `AlertService::run_post_sync` — catches unqualified `balance` regression.

Optional: document `DATABASE_URL=… cargo test --test wealth_alerts_integration` in runbook if not already present.

**Files:** `backend/tests/wealth_alerts_integration.rs`

### Done when

- [x] `cargo test --test wealth_alerts_integration` PASS (3/3; DB path skipped — `DATABASE_URL` unset)
- [x] Test asserts scarcity alert created (`alert_type = 'scarcity'`, `status = 'active'`) when DB available
- [x] No test skip regression introduced

---

## V1 — verify-work sync + alerts smoke

**Status:** open  
**Depends on:** BE1, T1 + deploy  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0018 **BE**, **BF**

### Description

Prepare `sprints/quick/Q0026/uat.md` smoke checklist. After **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**:

1. `POST /api/v1/sync/trigger` — logs free of `alert evaluation failed` / PostgreSQL 42702
2. **BE proof** — alerts phase completes; budget_drift/plan_viability may run
3. `GET /api/v1/alerts?status=active` — returns rows when household scarcity rule matches (account **114** overdrawn fixture)
4. **Header bell** — non-empty active preview (not permanent "No active alerts")
5. **Subscription regression** — `GET /api/v1/subscriptions/alerts` dedupes per BUG-0008 / DEC-0071
6. **Integration** — `DATABASE_URL=… cargo test --test wealth_alerts_integration` PASS

### Done when

- [ ] Rows **BE**, **BF** probed per acceptance.md operator matrix
- [ ] `uat.md` and `uat.json` populated with results
- [ ] Operator gates documented: **BACKEND_FRONTEND_DEPLOY**, **FULL_FIREFLY_SYNC**

**Operator gates:** **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC** before runtime probes.
