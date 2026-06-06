# UAT — Q0010 (BUG-0006)

**Status:** PASS (verify-work re-run 2, 2026-06-05)  
**Orchestrator:** `auto-20260605-bug0006-002`  
**Acceptance:** `docs/product/acceptance.md` — BUG-0006 rows **(P)**, **(Q)**, **(R)** (checked @ release 2026-06-05)  
**Plan-verify:** PASS (`sprints/quick/Q0010/plan-verify.json`, 2026-06-05)  
**QA:** PASS (`sprints/quick/Q0010/qa-findings.md`, 123/123)  
**Verify-work:** PASS (`sprints/quick/Q0010/verify-work-findings.md`, `uat.json`)

| Row | Task(s) | Verify-work re-run 2 | Notes |
|-----|---------|---------------------|-------|
| **(Q)** | Q1, Q2, Q3 | **PASS** | Sync `2ef16cfe` success; operator SQL 917/922 category_id, 919/922 date, 865/922 negative |
| **(R)** | R1 | **PASS** | Totals/counts on populated months; empty pre-ledger months correct `no_rows` |
| **(P)** | P1 | **PASS** | May 2026, Dec 2025, Jan 2026, Jun 2025 AI aggregates; no false empty when data exists |
| Regression | post-P1 | **PARTIAL** | `allow_raw_transactions=false` PASS; six-tool audit PASS; OIDC deferred |

## UAT step results

| Step | Description | Result |
|------|-------------|--------|
| V-1 | Backend `cargo test --lib` | pass (123/123) |
| V-2 | Frontend `npm test` | pass (2/2) |
| V-3 | Frontend `npm run build` | pass |
| V-4 | Q1 category_id upsert code path | pass |
| V-5 | Q2 ISO date parse code path | pass |
| V-6 | Q3 DEC-0059 amount sign code path | pass |
| V-7 | R1 TransactionAggregates + period_status | pass |
| V-8 | DEC-0032 privacy + six-tool unit tests | pass |
| Q-1 | Sync entities transactions ≥922 | pass |
| Q-2 | Post-reset Firefly sync backfill | pass (`2ef16cfe`) |
| Q-3 | SQL probe category_id / dates / signed amounts | pass (operator attestation) |
| R-1 | Aggregate totals + period_status live | pass |
| P-1 | AI Chat category/spending aggregates | pass |
| REG-1 | `allow_raw_transactions=false` | pass |
| REG-2 | Six-tool registry live | pass |
| REG-3 | OIDC deploy regression | deferred |
| REG-4 | Bundled-firefly deploy regression | deferred |

**Summary:** 15 pass, 2 fail (deferred OIDC + bundled-firefly), 0 partial.

## Acceptance mapping

- **(P)** AI Chat uses `get_transactions` aggregates when mirror rows exist for queried period — evidenced on May 2026, Dec 2025, Jan 2026, Jun 2025.
- **(Q)** Firefly sync persists `category_id`, parsed dates, signed amounts — operator SQL 917/919/865 of 922.
- **(R)** Aggregate JSON totals/counts + empty vs populated distinction under privacy gate — live on omniflow.

See `verify-work-findings.md` and `handoffs/verify_work_to_release.md` for release handoff.
