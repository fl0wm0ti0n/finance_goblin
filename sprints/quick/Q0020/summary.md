# Q0020 Summary — BUG-0013

**Sprint:** Q0020  
**Bug:** BUG-0013  
**Orchestrator:** `auto-20260608-bug0013-001`  
**Status:** **RELEASED** (`bug0013-q0020`, 2026-06-09)  
**Last updated:** 2026-06-09

## Outcome

Released **DEC-0079** (budgets MTD upper date bound) and **DEC-0080** (Bitunix wallet array parse + linear unrealized USDT→EUR). Optional Grafana copy for subscriptions price-changes empty state and portfolio performance % min-snapshot footnote. Acceptance AI–AN checked; operator omniflow smoke pass-with-prerequisites pending deploy gates.

## Tasks

| ID | Status | Notes |
|----|--------|-------|
| AL1 | DONE | Panel 5 `AND pdc.ts::date <= CURRENT_DATE` + mid-month footnote |
| AN1 | DONE | `resolve_futures_account`, `unrealizedPNL` keys, linear valuation in `pnl.rs` |
| AJ1 | DONE | Subscriptions price-changes panel description |
| AK2 | DONE | Portfolio performance % `noValue` + description |
| V1 | DONE | `uat.md` smoke checklist; runtime pass-with-prerequisites |

## Files changed

| Layer | Path |
|-------|------|
| Grafana | `grafana/provisioning/dashboards/analytics/budgets.json` |
| Grafana | `grafana/provisioning/dashboards/analytics/subscriptions.json` |
| Grafana | `grafana/provisioning/dashboards/analytics/portfolio.json` |
| Backend | `backend/src/exchanges/bitunix.rs` |
| Backend | `backend/src/exchanges/repository.rs` |
| Backend | `backend/src/portfolio/pnl.rs` |
| UAT | `sprints/quick/Q0020/uat.md` |

## Tests run

| Command | Result |
|---------|--------|
| `cargo test --lib` | **PASS** (174 tests) |

## Release evidence

- `handoffs/releases/Q0020-release-notes.md`
- `sprints/quick/Q0020/release-findings.md`
- `sprints/quick/Q0020/uat.json` (12 steps: 5 pass, 7 pass_with_prerequisites)
- `decisions/DEC-0079.md`, `decisions/DEC-0080.md`

## Operator gates (post-release smoke)

1. **BACKEND_FRONTEND_DEPLOY** — deploy AL1 + AN1 on omniflow
2. **GRAFANA_PROVISIONING_RELOAD** — reload after AL1/AJ1/AK2 JSON
3. **FULL_FIREFLY_SYNC** — Full sync + forecast recompute

Live omniflow probes AL-1 through REG-1 per `sprints/quick/Q0020/uat.md`.

## Stop reason

**RELEASED** — refresh-context complete; segment terminal.
