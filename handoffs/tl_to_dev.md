# TL -> Dev Handoff

## sprint-plan-20260607-q0017-bug0007 — BUG-0007 AI merchant/category discovery

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (V1)  
**Date:** 2026-06-07  
**Work item:** BUG-0007 (defect)  
**Sprint:** **Q0017** (`/quick`)  
**Orchestrator:** `auto-20260607-bug0007-001`  
**Next phase:** `/plan-verify`

### Summary

Quick sprint **Q0017** formalizes architecture **DEC-0069** — A′+E+F intelligence-layer fix for AI merchant/category discovery **(S)**, **(T)**, **(U)**. Seven tasks in dependency order; single backend PR for A1–E2+T1; operator gate before V1.

**Decision:** **DEC-0069**  
**Research:** **R-0065**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0007**  
**Sprint ref:** `sprints/quick/Q0017/sprint.md`, `sprints/quick/Q0017/task.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **A1** | `transactions/repository.rs`, `types.rs` | **(T)**, **(U)** |
| 2 | **A2** | `transactions/service.rs`, `ai/tools/transactions.rs` | **(T)**, **(U)** |
| 3 | **F1** | `ai/tools/subscriptions.rs` | **(S)** |
| 4 | **E1** | `ai/orchestrator.rs` | **(S)**, **(T)**, **(U)** |
| 5 | **E2** | `ai/tools/{transactions,subscriptions}.rs` (schema descriptions) | **(S)**, **(T)** |
| 6 | **T1** | `backend/tests/` or module `#[cfg(test)]` | regression |
| 7 | **V1** | verify-work omniflow AI Chat smoke | **(S)**, **(T)**, **(U)** |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **(S)** | F1, E1, E2, V1 | Chat lists named subscription merchants after cancelable-total question |
| **(T)** | A1, A2, E1, E2, V1 | Strom/Amazon amounts via category_search; 2023 Amazon cites mirror bounds |
| **(U)** | A1, A2, F1, E1, V1 | Multi-tool fusion without user-supplied merchant names |
| Footer | T1, V1 | Six tools; allow_raw_transactions=false default |

### Frozen boundaries

- **Six-tool registry** — no seventh tool; no DEC waiver
- **allow_raw_transactions=false** — default unchanged
- **BUG-0008** — additive AI JSON only; no alert/list/detection changes
- **RAG (V)** — out of scope for MVP
- **Payee aggregates (B)** — deferred
- **BUG-0006** — do not revert ingest fixes
- **No frontend changes** unless sprint-plan adds optional audit UI hint (not required for AC)

### Test contract (S/T/U)

- A1 fixture: ILIKE resolves Strom → 146, Amazon → 47; bounds min/max
- A2 integration: `category_matches`, `mirror_date_bounds`, `search_attempted` in tool JSON
- F1 unit: Counterparty-* rejected; `merchant_names` deduped; kind enum in schema
- E1 unit: audit `result_rows` non-NULL for both tools
- Privacy: six-tool registry; `allow_raw_transactions=false` → no `raw_rows`
- Operator smoke: subscription names; Strom/Amazon amounts; 2023 bounds empty-state

### Operator gates (V1)

1. **BACKEND_DEPLOY** — after deploy, backend image on omniflow before V1 AI Chat probes

### Deploy order

```text
(A1 → A2 → F1 → E1 → E2 → T1) single backend PR → deploy → V1 AI Chat smoke
```

No Firefly re-sync required.

### Artifacts created

- `sprints/quick/Q0017/sprint.json`, `sprint.md`, `tasks.md`, `task.json`, `progress.md`
- `docs/product/backlog.md` — BUG-0007 sprint_id Q0017
- `docs/engineering/state.md` — sprint-plan checkpoint
- `handoffs/resume_brief.md` — next phase plan-verify

### Prior architecture handoff

`architecture-20260607-bug0007` — superseded by this sprint-plan handoff for execute.

---

## architecture-20260607-bug0007 — BUG-0007 AI merchant/category discovery

**From:** Tech Lead  
**To:** Sprint planner / Dev (`/sprint-plan` → `/execute`)  
**Date:** 2026-06-07  
**Work item:** BUG-0007 (defect)  
**Orchestrator:** `auto-20260607-bug0007-001`  
**Next phase:** `/sprint-plan`

### Summary

Architecture freezes **DEC-0069** — **A′ + E + F** fix bundle within the six-tool registry. Server-side **`category_search`** on `get_transactions` resolves Strom→146 and Amazon→47; **`mirror_date_bounds`** closes T-a empty-period evidence; **`get_subscriptions`** schema enrichment + Counterparty guard fixes S; orchestrator prompt + **`audit.result_rows`** fixes enumeration and operator debug. No seventh tool; no RAG; BUG-0008 coordinate note only (additive AI JSON).

**Decision:** **DEC-0069**  
**Research:** **R-0065**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0007**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0007 rows **(S)**, **(T)**, **(U)**

### Recommended sprint

| Field | Value |
|-------|-------|
| **Type** | `/quick` |
| **ID** | **Q0017** (recommended) |
| **Tasks** | 7 (A1, A2, F1, E1, E2, T1, V1) |
| **Estimate** | ~13.5h total (~12.5h dev + ~1h operator V1) |
| **Split** | No — under `SPRINT_MAX_TASKS=12` |

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **A1** | `transactions/repository.rs`, `types.rs` | **(T)**, **(U)** |
| 2 | **A2** | `transactions/service.rs`, `ai/tools/transactions.rs` | **(T)**, **(U)** |
| 3 | **F1** | `ai/tools/subscriptions.rs` | **(S)** |
| 4 | **E1** | `ai/orchestrator.rs` | **(S)**, **(T)**, **(U)** |
| 5 | **E2** | `ai/tools/{transactions,subscriptions}.rs` (schema descriptions) | **(S)**, **(T)** |
| 6 | **T1** | `backend/tests/` or module `#[cfg(test)]` | regression |
| 7 | **V1** | verify-work omniflow AI Chat smoke | **(S)**, **(T)**, **(U)** |

### Frozen contracts (DEC-0069)

#### A′ — `get_transactions.category_search`

| Slice | Contract |
|-------|----------|
| Param | `category_search` optional string; min 2 chars after trim; ILIKE on `categories.name` |
| Precedence | `category_search` wins when both `category_id` + search supplied |
| Cap | 10 category matches; `category_matches_truncated` when exceeded |
| Response | Always include `mirror_date_bounds { min, max }`; `category_matches[]`, `search_attempted` |
| Privacy | DEC-0032 unchanged; no raw_rows; no description/payee search |

#### F — `get_subscriptions`

| Slice | Contract |
|-------|----------|
| Schema | `kind` enum: `subscription` \| `standing_order` |
| Guard | Reject `Counterparty-*` in status/kind → InvalidArgs |
| Response | Add `patterns_count`, `merchant_names[]` (deduped display_name) |
| REST | **No** `list_patterns` behavior change |

#### E — Orchestrator + audit

| Slice | Contract |
|-------|----------|
| Prompt | Enumerate all display_name; category_search for keywords; cite bounds on empty period |
| Audit | Populate `result_rows` for get_transactions (bucket count) and get_subscriptions (patterns_count) |
| Local | No `tool_choice: required` |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **(S)** | F1, E1, E2, V1 | Chat lists named subscription merchants after cancelable-total question |
| **(T)** | A1, A2, E1, V1 | Strom/Amazon amounts via category_search; 2023 Amazon cites mirror bounds |
| **(U)** | A1, A2, F1, E1, V1 | Multi-tool fusion without user-supplied merchant names |
| Footer | T1, V1 | Six tools; allow_raw_transactions=false default |

### Frozen boundaries

- **Six-tool registry** — no seventh tool; no DEC waiver
- **allow_raw_transactions=false** — default unchanged
- **BUG-0008** — additive AI JSON only; no alert/list/detection changes
- **RAG (V)** — out of scope for MVP
- **Payee aggregates (B)** — deferred
- **BUG-0006** — do not revert ingest fixes
- **No frontend changes** unless sprint-plan adds optional audit UI hint (not required for AC)

### Deploy order

```text
(A1 → A2 → F1 → E1 → E2 → T1) single backend PR → deploy → V1 AI Chat smoke
```

No Firefly re-sync required.

### Operator gates (V1)

1. Probe **S:** ask cancelable streaming total, then "liste mir die dienste auf" — expect named merchants from tool data
2. Probe **T-b:** Strom and Amazon spend in mirror window — expect category_search-backed amounts
3. Probe **T-a:** Amazon Jan–Oct 2023 — expect explicit empty-state with mirror_date_bounds (2025-06-05…)
4. Regression: six-tool count; settings `allow_raw_transactions=false`

### Artifacts created

- `decisions/DEC-0069.md`
- `docs/engineering/architecture.md` § **BUG-0007**
- `docs/engineering/decisions.md` — DEC-0069 index + summary
- `docs/engineering/state.md` — architecture checkpoint
- `handoffs/resume_brief.md` — sprint-plan readiness

**Status:** superseded by `sprint-plan-20260607-q0017-bug0007` above; Q0017 sprint-plan complete 2026-06-07.

### Prior handoffs (reference)

- `research-20260607-bug0007` in `handoffs/po_to_tl.md` — research complete
- `discovery-20260607-bug0007` in `handoffs/po_to_tl.md` — discovery verdicts S/T/U/V

---

## sprint-plan-20260606-q0016-bug0009 — BUG-0009 Grafana empty panels & account overview

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (V1)  
**Date:** 2026-06-06  
**Work item:** BUG-0009 (defect)  
**Sprint:** **Q0016** (`/quick`)  
**Orchestrator:** `auto-20260606-bug0009-001`  
**Next phase:** `/plan-verify`

### Summary

Quick sprint **Q0016** formalizes architecture **DEC-0068** — provisioning-only fix for Grafana perceived emptiness **(Y)** and missing cross-account overview **(Z)**. Six tasks in dependency order; single PR for provisioning JSON + tests (Z1–Y2, T1); operator gate before V1.

**Decision:** **DEC-0068**  
**Research:** **R-0064**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0009**  
**Sprint ref:** `sprints/quick/Q0016/sprint.md`, `sprints/quick/Q0016/task.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **Z1** | `grafana/provisioning/dashboards/analytics/portfolio.json` | **(Z)** |
| 2 | **Z2** | `portfolio.json` (grid layout + overview table title) | **(Z)** |
| 3 | **Y1** | `cashflow.json`, `forecast-horizons.json` (`$account_id` query) | **(Y)** |
| 4 | **Y2** | `forecast-horizons.json` (ML banner + noValue) | **(Y)** |
| 5 | **T1** | SQL fixtures + optional provisioning snapshot test | **(Y)(Z)** |
| 6 | **V1** | verify-work omniflow smoke | **(Y)(Z)** |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **(Y)** | Y1, Y2, T1, V1 | Default-load cashflow/forecast non-empty; ML banner honest; ds/query 200 |
| **(Z)** | Z1, Z2, T1, V1 | Portfolio overview shows all synced asset accounts; six `/analytics/{slug}` routes smoke |

### Frozen boundaries

- **No backend changes** — mirror `accounts.balance` already correct (DEC-0060/0065)
- **No React changes** — optional Z3 docs only
- **US-0013** — ML enablement out of scope
- **Seventh overview dashboard** — rejected
- **Grafana dynamic hide rules** — rejected
- **React `/forecast` API reorder** — optional follow-up; not required for AC Y

### Test contract (Y/Z)

- SQL fixture: 3-account snapshot → breakdown query returns **3 rows**
- SQL fixture: variable query ABS sort picks funded account over zero wallet
- Operator smoke: `/analytics/cashflow` default load → non-flat series (acct 114 pattern on omniflow)
- Operator smoke: `/analytics/portfolio` → overview table 3 rows; `total_eur` stat visible
- Regression: six analytics routes + ds/query 200 (BUG-0003 H, BUG-0004 K)

### Operator gates (V1)

1. **GRAFANA_PROVISIONING_RELOAD** — after deploy, reload Grafana provisioning (container restart or poll) before V1 omniflow probes

### Deploy order

```text
(Z1 + Z2 + Y1 + Y2 + T1) single PR → deploy → Grafana provisioning reload → V1
```

### Artifacts created

- `sprints/quick/Q0016/sprint.json`, `sprint.md`, `tasks.md`, `task.json`, `progress.md`
- `docs/product/backlog.md` — BUG-0009 sprint_id Q0016
- `docs/engineering/state.md` — sprint-plan checkpoint
- `handoffs/resume_brief.md` — next phase plan-verify

### Prior architecture handoff

`architecture-20260606-bug0009` — superseded by this sprint-plan handoff for execute.

---

## architecture-20260606-bug0009 — BUG-0009 Grafana empty panels & account overview

**From:** Tech Lead  
**To:** Sprint planner / Dev (`/sprint-plan` → `/execute`)  
**Date:** 2026-06-06  
**Work item:** BUG-0009 (defect)  
**Orchestrator:** `auto-20260606-bug0009-001`  
**Next phase:** `/sprint-plan`

### Summary

Architecture freezes **DEC-0068** — provisioning-only fix for Grafana perceived emptiness (Y) and missing cross-account overview (Z). Primary Y cause is **`$account_id` alphabetical default → acct 116 (zero forecast)**; Z requires **portfolio SQL subquery fix (Z1)** + **overview table on portfolio dashboard (Z2)**. ML panels empty on omniflow is expected (DEC-0049) — close with **banner + noValue (Y2)**, not US-0013. No backend or React code in scope.

**Decision:** **DEC-0068**  
**Research:** **R-0064**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0009**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0009 rows **(Y)**, **(Z)**

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **Z1** | `grafana/provisioning/dashboards/analytics/portfolio.json` | **(Z)** |
| 2 | **Z2** | `portfolio.json` (grid layout + overview table title) | **(Z)** |
| 3 | **Y1** | `cashflow.json`, `forecast-horizons.json` (`$account_id` query) | **(Y)** |
| 4 | **Y2** | `forecast-horizons.json` (ML banner + noValue) | **(Y)** |
| 5 | **T1** | SQL fixtures + optional provisioning snapshot test | **(Y)(Z)** |
| 6 | **V1** | verify-work omniflow smoke | **(Y)(Z)** |

### Frozen contracts (DEC-0068)

| Slice | Contract |
|-------|----------|
| **Y1** | `ORDER BY ABS(COALESCE(balance,0)) DESC, name` on `$account_id`; **omit** `current` in JSON |
| **Z1** | Latest-snapshot subquery + `LATERAL jsonb_array_elements`; remove global `LIMIT 1` on cross-join |
| **Z2** | Overview on **portfolio dashboard only** — stat row + all-accounts table; `/wealth` supplementary (Z3 docs) |
| **Y2** | Text banner + `noValue: "ML unavailable"`; **no** dynamic hide; **no** US-0013 scope |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **(Y)** | Y1, Y2, T1, V1 | Default-load cashflow/forecast non-empty; ML banner honest; ds/query 200 |
| **(Z)** | Z1, Z2, T1, V1 | Portfolio overview shows all synced asset accounts; six `/analytics/{slug}` routes smoke |

**Status:** superseded by `sprint-plan-20260606-q0016-bug0009` above; Q0016 released PASS 2026-06-06.

---
|-----|-------|--------|
| **(Y)** | Y1, Y2, T1, V1 | Default-load cashflow/forecast non-empty; ML banner honest; ds/query 200 |
| **(Z)** | Z1, Z2, T1, V1 | Portfolio overview shows all synced asset accounts; six `/analytics/{slug}` routes smoke |

**Status:** superseded by `sprint-plan-20260606-q0016-bug0009` above; Q0016 released PASS 2026-06-06.

---
