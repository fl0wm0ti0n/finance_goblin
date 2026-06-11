# PO to TL archive pack (2026-06-09)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 12
- First archived heading: `## discovery-20260610-bug0018 — BUG-0018 alert evaluation SQL failure (hot pointer)`
- Last archived heading: `## discovery-20260610-bug0018 — BUG-0018 alert evaluation SQL failure (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=54
  - retained_body_lines=500

---

## discovery-20260610-bug0018 — BUG-0018 alert evaluation SQL failure (hot pointer)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-10  
**Bug:** BUG-0018  
**Orchestrator run:** `intake-20260609-ui-audit`  
**Next phase:** `/research`

### Summary

UI audit **UI-003**: post-sync wealth alert evaluation logs PostgreSQL **42702** (`column reference "balance" is ambiguous`); header Alerts bell and `/alerts` show **No active alerts** while sync status stays **success**. Account **114** overdrawn (−3395.75 EUR) — scarcity rule should fire when SQL succeeds.

**Verdict:** **One confirmed backend SQL defect** (**BE**), **one downstream UX symptom** (**BF**). Subscription alerts (`/api/v1/subscriptions/alerts`) run in separate sync phase — not primary root cause.

### Sub-defect verdicts

| AC | Verdict | Root cause | Research task |
|----|---------|------------|---------------|
| **BE** | **CONFIRMED** | `evaluate_scarcity` JOIN `forecast_balance_daily fbd` + `accounts a` uses unqualified `SUM(balance::float8)` | Qualify `fbd.balance`; grep alert SQL for other ambiguous refs |
| **BF** | **DOWNSTREAM CONFIRMED** | `run_post_sync` fails at first evaluator; warn-only in sync; APIs return empty | Re-smoke acct 114 scarcity + bell badge after **BE** fix |

### Operator gates (mandatory before sprint verify)

1. **BACKEND_FRONTEND_DEPLOY** — ship alert SQL fix on localhost:18080 / omniflow
2. **FULL_FIREFLY_SYNC** — `POST /api/v1/sync/trigger`; logs free of `alert evaluation failed` / 42702
3. **Alerts smoke** — `GET /api/v1/alerts?status=active` returns rows when overdrawn; header bell non-empty preview
4. **Subscription regression** — `GET /api/v1/subscriptions/alerts` still dedupes per BUG-0008 / DEC-0071

### Research pointers (for `/research` — extend R-0022 / R-0024)

- Single-line `fbd.balance` qualification vs broader alias refactor
- CI gap: `wealth_alerts_integration` skips without `DATABASE_URL` — recommend mandatory run or static SQL review
- **BF** scope: wealth alerts only vs include subscription unread cross-link in acceptance smoke
- Preserve R-0024 warn-only failure semantics — do not change sync status unless operator requests

### Discovery decomposition evidence

- Workflow count: 1 (sync → alerts phase inspect) — **single bug retained**
- Cross-cutting: `alerts/evaluate.rs`, sync alerts phase, alerts REST, header bell (read-only)
- Acceptance: 2 AC (**BE**, **BF**) unchanged
- Risk: minimal fix must not alter scarcity aggregation semantics or BUG-0008 subscription dedup

### Artifacts updated

- `docs/product/vision.md` § BUG-0018 discovery
- `docs/product/backlog.md#BUG-0018`
- `handoffs/resume_brief.md`
- `docs/engineering/state.md` — discovery checkpoint

**Evidence:** `handoffs/intake_evidence/intake-20260609-alert-evaluation.json`, `handoffs/intake_evidence/ui-audit-20260609-local.json` (UI-003), `backend/src/alerts/evaluate.rs`, `backend/migrations/002_forecast_hypertables.sql`, `backend/migrations/001_initial.sql`, `backend/src/sync/mod.rs`, `backend/tests/wealth_alerts_integration.rs`, [R-0022](docs/engineering/research.md#r-0022--alert-engine-evaluation-rules-scarcity-budget-drift-plan-viability), [R-0024](docs/engineering/research.md#r-0024--post-sync-alert-engine-pipeline--net-worth-snapshot-hook)

---

