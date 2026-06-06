# UAT — Q0011 (BUG-0004)

**Status:** Verify-work **PASS** (2026-06-05 re-run) — ready for `/release`  
**Acceptance:** `docs/product/acceptance.md` — BUG-0004 rows **(I)**, **(J)**, **(K)**, **(L)** (checkboxes pending release phase)  
**Plan-verify:** PASS (`sprints/quick/Q0011/plan-verify.json`, `plan-verify.md` — 2026-06-05)  
**QA:** PASS (`sprints/quick/Q0011/qa-findings.md` — 2026-06-05; I1–J2 code validated)  
**Verify-work:** PASS (`sprints/quick/Q0011/verify-work-findings.md`, `uat.json` — 2026-06-05 re-run)  
**Orchestrator:** `auto-20260605-bug0004-002`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Verify-work |
|-----|---------|-----------------------------------|-------------|
| **(I)** | I1 | Exchange sync does not remain `state: running` with `finished_at: null` once exchange phase completes | **PASS** |
| **(J)** | J1, J2 | Subscription detection surfaces recurring merchants (≥3 txs per payee) **or** documents thresholds in UI empty-state | **PASS** (API); J2 UI deferred |
| **(K)** | K1 | Portfolio Grafana SQL executes without `pq: syntax error at or near "UNION"`; ds/query **200** | **PASS** |
| **(L)** | L1, L2, L3 | Forecast/wealth show account-level data; snapshots populate; analytics non-empty for synced `account_id` | **PASS** |
| Regression | post-L3 | OIDC-enabled and bundled-firefly deploy regression checks pass | **DEFERRED** |

## Verify-work step results

| Step | Description | Result |
|------|-------------|--------|
| V-1 | `cargo test --lib` | **PASS** (110/110) |
| V-2 | `npm test` | **PASS** (2/2) |
| V-3 | `npm run build` | **PASS** |
| V-4 | Omniflow reachability | **PASS** |
| I-1 | Row **(I)** exchange terminal status | **PASS** — `fc2a6ab9` manual_exchanges success + `finished_at` |
| J-1 | Row **(J)** subscriptions detection / UX | **PASS** — 11 pending patterns; J2 UI 401 deferred |
| K-1 | Row **(K)** portfolio ds/query | **PASS** — fixed UNION SQL **200** |
| L-1 | Row **(L)** wealth + forecast | **PASS** — 2 accounts; forecast series populated |
| REG-1 | OIDC regression | **DEFERRED** |
| REG-2 | Bundled-firefly regression | **DEFERRED** |

## Results summary

- **Passed:** 9 (local gates + reachability + rows I/J/K/L)
- **Failed:** 0
- **Deferred:** 2 (regression footer)
- **Blocking codes:** (none)
- **Evidence:** `sprints/quick/Q0011/verify-work-findings.md`, `sprints/quick/Q0011/uat.json`

## Operator smoke (completed)

1. Deploy backend + Grafana + frontend image with I1–J2 merged — **done**
2. Manual Full Firefly sync — **done** (`2026-06-05T14:30:14Z`)
3. Manual exchange sync — **done** (`fc2a6ab9` `2026-06-05T14:30:52Z`)
4. Portfolio ds/query — **200** **(K)**
5. Wealth 2 accounts + forecast daily series **(L)**
6. Subscriptions 11 pending **(J)**

## Next phase

**`/release`** — check BUG-0004 acceptance checkbox; operator OIDC/J2 browser smoke optional.
