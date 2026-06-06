# Plan-verify — Quick Q0011 / BUG-0004

**Sprint:** Q0011 (`/quick`)  
**Bug:** BUG-0004 — Post-sync pipeline empty analytics  
**Verified at:** 2026-06-05T20:00:00Z  
**Role:** QA  
**Orchestrator:** `auto-20260605-bug0004-001`  
**Verdict:** **PASS**

## Inputs

| Source | Path |
|--------|------|
| Acceptance | `docs/product/acceptance.md` — BUG-0004 rows **(I)**, **(J)**, **(K)**, **(L)** |
| Tasks | `sprints/quick/Q0011/tasks.md`, `task.json` |
| Sprint plan | `sprints/quick/Q0011/sprint.md`, `sprint.json` |
| Architecture | `docs/engineering/architecture.md` § BUG-0004 |
| Handoff | `handoffs/tl_to_dev.md` (`architecture-20260605-bug0004`, `sprint-plan-20260605-q0011-bug0004`) |
| Decisions | `DEC-0060`, `DEC-0061`; extends `DEC-0002`, `DEC-0014`, `DEC-0041` |
| Research | `R-0061` |

## Test plan (coverage review)

For each acceptance row, confirm at least one task with explicit done-when checks and feasible deploy/execute order.

| Row | Criterion (abbrev.) | Tasks | Covered |
|-----|---------------------|-------|---------|
| **(I)** | Exchange sync does not remain `state: running` with `finished_at: null` after exchange phase; terminal status persisted | I1, L3 | yes |
| **(J)** | Subscription detection surfaces recurring merchants (≥3 txs per payee) **or** documents thresholds in UI empty-state | J1, J2, L3 | yes |
| **(K)** | Portfolio Grafana SQL executes without `pq: syntax error at or near "UNION"`; ds/query **200** | K1, L3 | yes |
| **(L)** | Wealth/forecast show account-level data; `net_worth_snapshots` and forecast series populate; analytics non-empty for synced accounts | L1, L2, L3 | yes |
| Regression | OIDC + bundled-firefly footer | post-L3 verify-work | yes (advisory) |

### Architecture contract alignment

| Slice | Contract (frozen) | Task(s) | Aligned |
|-------|-------------------|---------|---------|
| **I** | `finish_sync_run` on `RunMode::ExchangesOnly` success/failed; no double-finish Full | I1 | yes |
| **K** | Portfolio pie panel id 8: parenthesize `ORDER BY … LIMIT 1` branches before `UNION ALL` | K1 | yes |
| **L1** | `parse_split_amount` for Firefly `current_balance` string/number (DEC-0060) | L1 | yes |
| **L2** | `COALESCE(balance, 0) >= 0` in `load_asset_accounts` | L2 | yes |
| **J1** | Payee source priority: description → counterparty_name → destination_name (DEC-0061) | J1 | yes |
| **J2** | Empty-state threshold copy + pending-count banner; Full-sync-only detection note | J2 | yes |
| **L3** | Deploy → Full Firefly sync gate → omniflow probes rows I–L | L3 | yes |

**No new DEC.** **DEC-0060** governs L1; **DEC-0061** governs J1. Frozen boundaries (no BUG-0005/0006 merge, no stuck-row SQL cleanup, no migration backfill, no auto-confirm) appear in `task.json`, `sprint.md`, and architecture § BUG-0004.

### Task traceability (I1–L3)

| Task | Acceptance hook | Orphan |
|------|-----------------|--------|
| I1 | **(I)** | no |
| K1 | **(K)** | no |
| L1 | **(L)** | no |
| L2 | **(L)** | no |
| J1 | **(J)** | no |
| J2 | **(J)** | no |
| L3 | **(I)(J)(K)(L)** verify-work | no |

### Dependency review

| Check | Result |
|-------|--------|
| Circular dependencies | none |
| Execution order feasible | yes — I1 → K1 → L1 → L2 → J1 → J2 → deploy → Full sync → L3 |
| L2 after L1 | `depends_on` in task.json |
| J2 after J1 | `depends_on` in task.json |
| L3 gated on deploy + Full Firefly sync | `operator_gates.FULL_FIREFLY_SYNC` in task.json |

### Test coverage review

| Layer | Task | Scope |
|-------|------|-------|
| Unit | I1 | ExchangesOnly success/error → `finish_sync_run` |
| Fixture/manual | K1 | Portfolio pie SQL; ds/query **200** |
| Unit | L1 | String/number `current_balance` → persisted balance |
| Unit/integration | L2 | NULL balance asset row included |
| Unit | J1 | Counterparty-only fixture → payee group key |
| Component/manual | J2 | Threshold copy; pending banner |
| Operator | L3 | Sync terminal; portfolio ds/query; wealth; forecast; subscriptions UX |
| UAT (post-execute) | verify-work | Rows I/J/K/L + regression footer |

## Findings

### Gaps

None.

### Orphan tasks

None (7/7 tasks map to rows I, J, K, or L).

### Advisories (non-blocking)

| ID | Note |
|----|------|
| ADV-1 | Regression footer (OIDC + bundled-firefly) — operator verify-work post-L3; no dedicated dev task |
| ADV-2 | Rows **(L)** runtime requires operator **Full Firefly sync** after deploy before L3 wealth/forecast probes (DEC-0002 upsert backfill) |
| ADV-3 | Historical stuck `sync_runs` rows not cleaned on deploy — I1 forward fix only per frozen boundary |
| ADV-4 | `net_worth_snapshots` populate implicitly via wealth recompute after L1 backfill — L3 probes wealth API + portfolio pie SQL, not explicit snapshot SQL |
| ADV-5 | Acceptance **(L)** mentions all analytics dashboards with `account_id` variable — plan fixes root NULL-balance cascade; only portfolio pie explicitly probed in K1/L3 |
| ADV-6 | BUG-0006 Q0010 amount sign (DEC-0059) may improve J1 expense filter after separate deploy — coordinate, not blocking |
| ADV-7 | Row **(J)** detection runs on Full sync only — J2 documents; exchange-only sync does not trigger detection |

## Verdict

**PASS** — Plan is ready for `/execute`. Machine-readable record: `sprints/quick/Q0011/plan-verify.json`.
