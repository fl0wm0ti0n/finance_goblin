# UAT — Q0013 (BUG-0010)

**Status:** VERIFY-WORK PASS — ready for `/release`  
**Acceptance:** `docs/product/acceptance.md` — BUG-0010 rows **(AA)**, **(AB)**, **(AC)** (checked — release 2026-06-05)  
**Orchestrator:** `auto-20260605-bug0010-001`  
**Plan-verify:** PASS (`sprints/quick/Q0013/plan-verify.json`, 2026-06-06)  
**QA:** PASS (`sprints/quick/Q0013/qa-findings.md`, 131/131 + vitest 2/2, 2026-06-05)  
**Verify-work:** PASS (`sprints/quick/Q0013/verify-work-findings.md`, 2026-06-05)

## Acceptance ↔ task traceability

Canonical source: `docs/product/acceptance.md` BUG-0010 row.

| Row | Task(s) | QA (code) | Verify-work |
|-----|---------|-----------|-------------|
| **(AA)** | AA1, AA3, V1 | **PASS** | **PASS** |
| **(AB)** | AB1, AB2, V1 | **PASS** | **PASS** |
| **(AC)** | AC1, AC2, V1 | **PASS** | **PASS** |
| Regression | post-V1 | **DEFERRED** | **PARTIAL** (OIDC browser deferred) |

**Discovery gates folded into V1:** AA2 (Full Firefly sync before probes); AB3 (snapshot totals via `wealth/history`).

## QA step results (2026-06-05)

| Step | Description | Result |
|------|-------------|--------|
| Q-1 | Backend `cargo test --lib` | pass (131/131) |
| Q-2 | Frontend `npm test` | pass (2/2) |
| Q-3 | Frontend `npm run build` | pass |
| Q-4 | AA1 balance_ingest logs + parse regression | pass |
| Q-5 | AB1 negative wealth + is_overdrawn + signed subtotal | pass |
| Q-6 | AC1 sidecar_disabled persist + meta derive | pass |
| Q-7 | AA3 balance_warnings meta + UI banner | pass |
| Q-8 | AB2 zero-total callout | pass |
| Q-9 | AC2 ML three-state UI copy | pass |
| Q-10 | Frozen boundaries (no tx recompute, ML default off) | pass |

## Verify-work probes (2026-06-05)

Sync run **`3e44fbfb`** manual success 2026-06-05T16:55:41Z.

| Step | Endpoint / action | Pass criteria | Result |
|------|-------------------|---------------|--------|
| V-1 | `cargo test --lib` | PASS | **pass** (131/131) |
| V-2 | `npm test` | PASS | **pass** (2/2) |
| V-3 | `npm run build` | PASS | **pass** |
| AA-1 | `GET /api/v1/forecast/long-term?account_id=114&horizon=3` | End balance plausible OR meta/UI negative-start warning | **pass** — start -3395.75, end -23590.16 + warning |
| AA-2 | `GET /api/v1/forecast/meta` | `balance_warnings` when negative start + history | **pass** — acct 114 `negative_starting_balance` |
| AB-1 | `GET /api/v1/wealth` | Acct 114 (Giro) present; `is_overdrawn` if negative; honest `total_eur` | **pass** — 3 accounts, total -3395.75 |
| AB-2 | `GET /api/v1/wealth/history?days=30` | Snapshots reflect post-sync totals | **pass** — 2026-06-05 -3395.75 |
| AC-1 | `GET /api/v1/forecast/meta` | `ml_skipped_reason: sidecar_disabled` when ML off | **pass** |
| AC-2 | `/forecast` UI explain panel | "Not enabled" copy — not "ML skipped" on null | **pass** (code path; browser deferred) |
| REG-1 | OIDC deploy regression | Per acceptance footer | **deferred** |
| REG-2 | Bundled-firefly deploy regression | Per acceptance footer | **deferred** |

## Release gate

Proceed to **`/release`** — verify-work PASS.
