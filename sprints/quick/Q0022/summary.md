# Q0022 Summary — BUG-0014

**Sprint:** Q0022  
**Bug:** BUG-0014 — Post-rebuild omniflow cluster  
**Orchestrator:** `auto-20260607-bug0014-001`  
**Status:** RELEASED (`bug0014-q0022`)  
**Last updated:** 2026-06-07

## Outcome

Omniflow post-rebuild fixes: Grafana ML copy (AO1), wealth crypto display with unified FX (AQ1/AQ2), planning delete UX + target_type select (AS1/AS2). AP2/AR1 skipped pending operator gates.

## Tasks

| ID | Status | Notes |
|----|--------|-------|
| AO1 | DONE | `forecast-horizons.json` panel 13 dual-scenario ML copy |
| AQ1 | DONE | `holdings_all` cap 50; unified `fx_incomplete` API |
| AQ2 | DONE | WealthPage native qty + EUR table + FX banner |
| AS1 | DONE | DELETE plan UI; active plan 409 guard |
| AS2 | DONE | Five `target_type` options + help copy |
| AP2 | SKIPPED | AP1_SQL_PROBE gate not met at release |
| AR1 | SKIPPED | Full sync + acct 114 probe deferred |
| V1 | OPERATOR | 14-step smoke post-deploy |

## Validation

| Command | Result |
|---------|--------|
| `cargo test --lib` | 177/177 PASS |
| `cargo test --lib plan_delete_api_tests` | 1/1 PASS |
| `cargo test --test grafana_provisioning_bug0009` | 6/6 PASS |
| `npm test -- --run` (frontend) | 6/6 PASS |

## Decisions

- DEC-0081 — holdings_all cap + unified fx_incomplete
- DEC-0082 — block active plan delete (409)
- DEC-0083 — target_type UI alignment

## Evidence

- Release: `handoffs/releases/Q0022-release-notes.md`
- UAT: `sprints/quick/Q0022/uat.json` (4 pass, 8 pass_with_prerequisites, 2 skipped)
- QA: `sprints/quick/Q0022/qa-findings.md` (0 blockers)
