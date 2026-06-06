# Plan-verify — Quick Q0012 / BUG-0005

**Sprint:** Q0012 (`/quick`)  
**Bug:** BUG-0005 — Exchange sync multi-product (Bitunix futures)  
**Verified at:** 2026-06-05T15:08:35Z  
**Role:** QA  
**Orchestrator:** `auto-20260605-bug0005-001`  
**Verdict:** **PASS**

## Inputs

| Source | Path |
|--------|------|
| Acceptance | `docs/product/acceptance.md` — BUG-0005 rows **(M)**, **(N)**, **(O)** |
| Tasks | `sprints/quick/Q0012/tasks.md`, `task.json` |
| Sprint plan | `sprints/quick/Q0012/sprint.md`, `sprint.json` |
| Architecture | `docs/engineering/architecture.md` § BUG-0005 |
| Handoff | `handoffs/tl_to_dev.md` (`architecture-20260605-bug0005`, `sprint-plan-20260605-q0012-bug0005`) |
| Decisions | `DEC-0062`, `DEC-0063`, `DEC-0064`; extends `DEC-0037`, `DEC-0038`, `DEC-0041` |
| Research | `R-0058`, `R-0059` |

## Test plan (coverage review)

For each acceptance row, confirm at least one task with explicit done-when checks and feasible deploy/execute order.

| Row | Criterion (abbrev.) | Tasks | Covered |
|-----|---------------------|-------|---------|
| **(M)** | Bitunix sync ingests futures/margin balances and positions; post-sync holdings include non-spot `product_type` when operator has futures exposure | M1, N2, O1 | yes |
| **(N)** | Futures REST uses `fapi.bitunix.com` header-auth per R-0058; sync_positions/balance endpoints populate data when read-only keys permit — not empty stubs behind `enabled_futures=false` default alone | N1, N2, N3, N4, O1 | yes |
| **(O)** | Wealth snapshot and portfolio crypto totals reflect combined spot + futures exchange holdings on US-0010 external profile | M1, N2, O1 | yes |
| Regression | OIDC + bundled-firefly footer; read-only key constraint preserved | post-O1 verify-work | yes (advisory) |

### Architecture contract alignment

| Slice | Contract (frozen) | Task(s) | Aligned |
|-------|-------------------|---------|---------|
| **N1** | `bitunix_futures_sign` + `futures_signed_get`; `futures_base_url` default `https://fapi.bitunix.com` (DEC-0062) | N1 | yes |
| **N3** | `effective_enabled_futures()` auto-enable with credentials; settings expose effective flag (DEC-0063) | N3 | yes |
| **M1** | Futures account `/api/v1/futures/account?marginCoin=USDT` → `product_type: "futures"` | M1 | yes |
| **N2** | `sync_positions` via `get_pending_positions` → `product_type: "linear"`, `market_value_usd: None` (DEC-0064) | N2 | yes |
| **N4** | Dual-path `test_connection` spot + futures sub-status | N4 | yes |
| **O1** | Deploy → exchange sync gate → omniflow probes rows M/N/O | O1 | yes |

**No new DEC.** **DEC-0062** governs N1; **DEC-0063** governs N3; **DEC-0064** governs M1/N2 wealth accounting. Frozen boundaries (no BUG-0006 merge, no `sync_funding`, USDT marginCoin MVP, no Binance/Bybit changes, read-only GET-only) appear in `task.json`, `sprint.md`, and architecture § BUG-0005.

### Task traceability (N1–O1)

| Task | Acceptance hook | Orphan |
|------|-----------------|--------|
| N1 | **(N)** | no |
| N3 | **(N)** | no |
| M1 | **(M)** | no |
| N2 | **(M)**, **(N)** | no |
| N4 | **(N)** | no |
| O1 | **(M)(N)(O)** verify-work | no |

### Dependency review

| Check | Result |
|-------|--------|
| Circular dependencies | none |
| Execution order feasible | yes — N1 → N3 → M1 → N2 → N4 → deploy → manual exchange sync → O1 |
| M1/N2/N4 after N1 + N3 | `depends_on` in task.json |
| O1 gated on deploy + exchange sync | `operator_gates.EXCHANGE_SYNC` in task.json |

### Test coverage review

| Layer | Task | Scope |
|-------|------|-------|
| Unit | N1 | Futures sign fixture; spot regression; mock HTTP headers |
| Unit | N3 | effective_enabled_futures creds/env/TOML matrix; settings API |
| Mock HTTP | M1 | Futures account → `product_type: futures`; spot-only regression |
| Mock HTTP | N2 | Positions → `product_type: linear`, null market value; disabled → empty |
| Unit | N4 | Partial OK messaging; spot fail → ok false |
| Operator | O1 | Settings; bitunix test; holdings; wealth crypto subtotal |
| UAT (post-execute) | verify-work | Rows M/N/O + regression footer |

## Findings

### Gaps

None.

### Orphan tasks

None (6/6 tasks map to rows M, N, or O).

### Advisories (non-blocking)

| ID | Note |
|----|------|
| ADV-1 | Regression footer (OIDC + bundled-firefly) and read-only key constraint — operator verify-work post-O1; no dedicated dev task |
| ADV-2 | Rows **(M)(O)** runtime proof requires operator **manual exchange sync** after deploy before O1 holdings/wealth probes |
| ADV-3 | Acceptance **(O)** mentions portfolio crypto totals — O1 probes `GET /api/v1/wealth` only; architecture DEC-0064 states no `WealthService` change; portfolio totals cascade from same holdings pipeline |
| ADV-4 | DEC-0064 excludes linear positions from wealth subtotal (`market_value_usd: None`) — row **(O)** subtotal driven by spot + futures wallet rows; positions satisfy row **(M)** visibility |
| ADV-5 | USDT `marginCoin` MVP only — multi-coin margin wallets deferred per frozen boundary |
| ADV-6 | `sync_funding` stub remains no-op — out of sprint scope per architecture |
| ADV-7 | BUG-0006 Q0010 is parallel track — frozen boundary forbids merge |

## Verdict

**PASS** — Plan is ready for `/execute`. Machine-readable record: `sprints/quick/Q0012/plan-verify.json`.
