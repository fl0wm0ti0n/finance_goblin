# Sprint Summary — Q0016 (BUG-0009)

**Bug:** BUG-0009  
**Sprint:** Q0016 (`/quick`)  
**Execute date:** 2026-06-06  
**Release date:** 2026-06-06  
**Status:** **RELEASED** — verify-work PASS; release PASS

## Tasks delivered

| ID | Status | Summary |
|----|--------|---------|
| Z1 | done | Portfolio breakdown SQL — latest-snapshot subquery + `LATERAL jsonb_array_elements`; removed global `LIMIT 1` |
| Z2 | done | Cross-account overview table retitled + repositioned below stat row; supplementary `/wealth` text panel |
| Y1 | done | `$account_id` `ORDER BY ABS(COALESCE(balance,0)) DESC, name` on cashflow + forecast-horizons; no `current` block |
| Y2 | done | ML status text banner + `noValue: "ML unavailable"` on ML panels (ids 7–10) |
| T1 | done | `grafana_provisioning_bug0009.rs` — JSON contract + SQL fixtures (3-row breakdown; ABS sort order) |
| V1 | done | Omniflow runtime smoke PASS after Grafana provisioning reload |

## Tests

| Check | Result |
|-------|--------|
| `cargo test --test grafana_provisioning_bug0009` | **PASS** (6/6) |
| SQL fixture: breakdown 3 rows | **PASS** |
| SQL fixture: ABS sort funded before zero wallet | **PASS** |
| Provisioning JSON contract (Y1/Y2/Z1/Z2) | **PASS** |

## Acceptance

| Row | Tasks | Code | Runtime |
|-----|-------|------|---------|
| **(Y)** | Y1, Y2, T1, V1 | **PASS** | **PASS** (acct 114 non-flat; ML banner) |
| **(Z)** | Z1, Z2, T1, V1 | **PASS** | **PASS** (3-row overview; total_eur -3395.75) |

## Files changed (primary)

- `grafana/provisioning/dashboards/analytics/portfolio.json` — Z1, Z2
- `grafana/provisioning/dashboards/analytics/cashflow.json` — Y1
- `grafana/provisioning/dashboards/analytics/forecast-horizons.json` — Y1, Y2
- `backend/tests/grafana_provisioning_bug0009.rs` — T1
- `tests/run-tests.sh` — T1 runner hook
- `sprints/quick/Q0016/uat.md` — V1 checklist
- `docs/engineering/runbook.md` — Grafana `current` save warning (§17 BUG-0009)

## Operator gate (V1)

**GRAFANA_PROVISIONING_RELOAD** — `--force-recreate grafana` executed on omniflow before verify-work smoke.

## Evidence

- Decision: **DEC-0068**
- Handoff: `handoffs/dev_to_qa.md` → `handoffs/verify_work_to_release.md`
- UAT: `sprints/quick/Q0016/uat.md`, `sprints/quick/Q0016/verify-work-findings.md`
- Release: `handoffs/releases/Q0016-release-notes.md`
