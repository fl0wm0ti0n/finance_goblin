# Architecture archive pack (2026-06-09)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 10
- First archived heading: `# BUG-0018 — Alert evaluation SQL failure (balance ambiguous)`
- Last archived heading: `# BUG-0018 — Alert evaluation SQL failure (balance ambiguous)`
- Verification tuple (mandatory):
  - archived_body_lines=156
  - preamble_lines=10
  - retained_body_lines=2930

---

# BUG-0018 — Alert evaluation SQL failure (balance ambiguous)

**Status:** architecture complete (2026-06-10)  
**Discovery:** `discovery-20260610-bug0018` in `handoffs/archive/po-to-tl-pack-20260609-j.md`  
**Research:** [R-0088](research.md#r-0088--bug-0018-evaluate_scarcity-ambiguous-balance--alert-eval-pipeline-abort)  
**Decisions:** **DEC-0107** (scarcity JOIN column qualification); extends **DEC-0026**, **DEC-0028**; **no new DEC** for R-0024 warn-only sync semantics  
**Sprint:** `/quick` **Q0026** (PLANNED — BE1 + T1 + V1; ≤3 tasks)  
**Acceptance:** `docs/product/acceptance.md` rows **BE**, **BF**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0018-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**Related:** UI-003, BUG-0008 (subscription dedup regression gate only), US-0005, R-0022, R-0024, R-0068

### Root-cause chain (frozen)

Post-sync alerts phase per [R-0024](research.md#r-0024--post-sync-alert-engine-pipeline--net-worth-snapshot-hook):

```text
forecast success → phase "alerts" → wealth snapshot upsert → AlertService::run_post_sync
  → evaluate_scarcity (FIRST) → evaluate_budget_drift → evaluate_plan_viability → upsert/resolve
```

| AC | Defect | Mechanism | Symptom |
|----|--------|-----------|---------|
| **BE** | Unqualified `balance` in JOIN | `evaluate_scarcity` L23–30: `fbd` + `accounts` JOIN + `SUM(balance::float8)` — both tables define `balance` | PostgreSQL **42702**; log `alert evaluation failed` |
| **BF** | First-evaluator abort | `service.rs` L72 `evaluate_scarcity(...).await?` propagates error; entire wealth pass skipped; R-0024 warn-only preserves sync **success** | `GET /api/v1/alerts?status=active` → `[]`; header bell "No active alerts" despite overdrawn account **114** |

Intake live probe: sync run `9ee95e6b-c6bd-4f4e-9b8c-4c068bf718cf`, ui-audit **UI-003**.

`isolation_scope`: artifact + repo source reads; no host `.env` / `.env_prod` secrets read.

### Architecture contract

```text
BUG-0018
├── BE1 — Qualify fbd.balance + fbd.ts in evaluate_scarcity (P0)
│   └── DEC-0107: projected-path household aggregate unchanged
├── T1 — wealth_alerts_integration regression gate (P0)
│   └── Existing test exercises exact JOIN path; PASS when DATABASE_URL set
└── V1 — verify-work operator smoke (P0)
    └── BE log clean; BF wealth inbox + header bell; subscription dedup regression
```

**Deploy:** Single backend change in `evaluate.rs`; rebuild `flow-finance-ai` image. **No migration.** **No frontend change** — BF resolves when **BE** fixed.

### DEC-0107 — Scarcity SQL qualification

**Defective query** (`evaluate.rs` L21–32):

```sql
SELECT ts::date AS day, SUM(balance::float8) AS balance
FROM forecast_balance_daily fbd
JOIN accounts a ON a.firefly_id = fbd.account_id
...
GROUP BY ts::date
```

**Fixed query** (frozen contract):

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

**Semantic requirement ([R-0022](research.md#r-0022--alert-engine-evaluation-rules-scarcity-budget-drift-plan-viability)):** household aggregate = sum of **projected** `forecast_balance_daily.balance` per day across included asset accounts. JOIN filters which accounts contribute; aggregation must read **`fbd.balance` only**.

**Sibling evaluator audit** ([R-0088 §2](research.md#r-0088--bug-0018-evaluate_scarcity-ambiguous-balance--alert-eval-pipeline-abort)):

| Function | JOIN with `balance`? | Verdict |
|----------|---------------------|---------|
| `evaluate_scarcity` | Yes (`fbd` + `accounts`) | **Fix required (BE1)** |
| `evaluate_budget_drift` | No | OK |
| `evaluate_plan_viability` | No (`planned_balance AS bal`) | OK |
| `current_balance` scalar | No (single-table) | OK |

**Rejected:** `a.balance` qualification; subquery filter; broader alias refactor; sync fail-on-alert-error.

### R-0024 failure semantics (preserve — no DEC)

Per [R-0024](research.md#r-0024--post-sync-alert-engine-pipeline--net-worth-snapshot-hook) and `sync/mod.rs` L413–414:

- Alert eval failure → **`warn!`**, sync run remains **`success`** if ingest+forecast succeeded.
- Last alert state preserved (no mass resolve on error).
- Optional Sync Status UI surfacing — **defer**.

**Downstream note:** budget_drift and plan_viability evaluators never run while scarcity SQL fails — fixing **BE** may surface additional alerts post-sync (expected, not regression).

### BF acceptance scope

| Proof target | Scope | Gate |
|--------------|-------|------|
| **BF primary** | Wealth inbox: `GET /api/v1/alerts?status=active` returns rows when scarcity rule matches; header bell non-empty preview | **Mandatory** V1 after **BE1** + **FULL_FIREFLY_SYNC** |
| **Subscription regression** | `GET /api/v1/subscriptions/alerts` dedup per BUG-0008 / DEC-0071 | **Operator gate** in V1 — confirm no regression; not primary fix validation |
| **Frontend change** | Error/loading when eval fails | **Defer** — BF resolves when **BE** fixed |

### CI / test gap

| Layer | Catches BUG-0018? |
|-------|-------------------|
| `cargo test --lib` | **No** — unit tests mock logic only |
| `wealth_alerts_integration` | **Yes** — gated on `DATABASE_URL` |
| Default CI (`tests/run-tests.sh`) | **No** — skips without DB |

**T1 contract:** existing `wealth_snapshot_and_scarcity_alert_on_post_sync` must PASS post-fix. V1 mandates operator run when DB available. Optional stretch: CI TimescaleDB service container ([R-0088 §3](research.md#r-0088--bug-0018-evaluate_scarcity-ambiguous-balance--alert-eval-pipeline-abort)) — defer unless sprint capacity.

### Operator smoke matrix (V1)

1. **BACKEND_FRONTEND_DEPLOY** — ship alert SQL fix
2. **FULL_FIREFLY_SYNC** — `POST /api/v1/sync/trigger`; logs free of `alert evaluation failed` / 42702
3. **BE proof** — alerts phase completes without SQL error
4. **BF proof** — `GET /api/v1/alerts?status=active` returns rows when household scarcity rule matches (account **114** overdrawn fixture)
5. **Header bell** — non-empty active preview (not permanent "No active alerts" from eval skip)
6. **Subscription regression** — `GET /api/v1/subscriptions/alerts` still dedupes per BUG-0008
7. **Integration** — `DATABASE_URL=… cargo test --test wealth_alerts_integration` PASS

### Alternatives rejected

| Alternative | Why rejected |
|-------------|--------------|
| Qualify `accounts.balance` | Wrong semantics — Ist mirror, not projected path (R-0022) |
| Subquery `account_id IN (SELECT …)` | Larger diff; equivalent semantics |
| Broader `evaluate.rs` alias refactor | Siblings already unambiguous |
| Sync fail on alert SQL error | Violates R-0024 unless product overrides |
| Frontend error state for eval failure | BF resolves with **BE**; defer |
| `cargo sqlx prepare --check` | Runtime strings; integration test simpler |

### Risks

| Risk | Mitigation |
|------|------------|
| Wrong column qualified (`a.balance`) | Code review + T1 integration test |
| CI still skips integration | V1 operator gate + runbook note |
| New alerts after fix (budget/plan run) | Expected behavior — document |
| BUG-0008 subscription dedup regression | Separate code path; V1 regression gate only |

### Decisions

| ID | Topic | Summary |
|----|-------|---------|
| **DEC-0107** | Scarcity SQL qualification | `fbd.balance` + `fbd.ts` in JOIN aggregate; forbid `a.balance`; no migration |

### Sprint recommendation

| Field | Value |
|-------|-------|
| **Type** | `/quick` **Q0026** |
| **Tasks** | 3 (BE1, T1, V1) |
| **Split** | No — under `SPRINT_MAX_TASKS=12` and `/quick` ≤3 guidance |

---

