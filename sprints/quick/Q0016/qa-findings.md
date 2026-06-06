# QA Findings — Quick Q0016 / BUG-0009

**Work item:** BUG-0009 (defect)  
**Quick task:** Q0016  
**QA phase:** `/qa`  
**Date:** 2026-06-06  
**Orchestrator:** `auto-20260606-bug0009-001`  
**Verdict:** **PASS** (ready for `/verify-work`; V1 omniflow smoke deferred until deploy + **GRAFANA_PROVISIONING_RELOAD**)

## Scope

Grafana perceived emptiness **(Y)** and missing cross-account overview **(Z)** per frozen **DEC-0068** (`handoffs/dev_to_qa.md`, `docs/engineering/decisions.md` DEC-0068, `docs/engineering/architecture.md` § BUG-0009):

- **Z1** — Portfolio breakdown: latest-snapshot subquery + `CROSS JOIN LATERAL jsonb_array_elements`
- **Z2** — Cross-account overview table below stat row; supplementary `/wealth` text panel
- **Y1** — `$account_id` ABS(balance) sort on `cashflow.json` + `forecast-horizons.json`; no `current` on `account_id`
- **Y2** — ML status banner (panel id 13) + `noValue: "ML unavailable"` on ML panels 7–10
- **T1** — `backend/tests/grafana_provisioning_bug0009.rs` contract + SQL fixture tests
- **V1** — Omniflow ds/query + six-route smoke — **DEFERRED**

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0016/summary.md`, `sprints/quick/Q0016/uat.md`, `sprints/quick/Q0016/plan-verify.json`, `docs/product/acceptance.md` (BUG-0009 rows Y/Z), `docs/engineering/decisions.md` (DEC-0068), `grafana/provisioning/dashboards/analytics/{portfolio,cashflow,forecast-horizons}.json`, `backend/tests/grafana_provisioning_bug0009.rs`, `tests/run-tests.sh`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Grafana provisioning contract | `cd backend && cargo test --test grafana_provisioning_bug0009` | **PASS** (6/6) |
| T-2 | Z1 LATERAL SQL contract | `portfolio_breakdown_sql_uses_latest_snapshot_lateral` + static review panel 5 | **PASS** |
| T-3 | Z1 rejects global LIMIT 1 bug | Test asserts no erroneous cross-join LIMIT | **PASS** |
| T-4 | Z2 overview title + grid | `portfolio_overview_table_title_and_grid` — title + y=4 | **PASS** |
| T-5 | Z2 supplementary wealth link | Static review panel 12 `/wealth` text | **PASS** |
| T-6 | Y1 ABS(balance) sort | `account_id_variable_uses_abs_balance_sort` + static review | **PASS** |
| T-7 | Y1 no `current` on account_id | Test + static review cashflow + forecast-horizons | **PASS** |
| T-8 | Y2 ML banner | `forecast_horizons_ml_banner_and_no_value` + panel 13 review | **PASS** |
| T-9 | Y2 ML noValue panels 7–10 | Test + static review | **PASS** |
| T-10 | Y2 `$forecast_variant` unchanged | Static review — `current: baseline` preserved | **PASS** |
| T-11 | T1 SQL fixture — 3-account breakdown | `breakdown_query_returns_all_accounts_from_latest_snapshot` | **PASS** |
| T-12 | T1 SQL fixture — ABS sort preference | `account_variable_query_prefers_funded_account_over_zero_wallet` | **PASS** |
| T-13 | Regression — portfolio UNION pie | Static review panel 8 — BUG-0004 K SQL unchanged | **PASS** |
| T-14 | Frozen boundaries | No backend/React; no seventh dashboard; no dynamic hide rules | **PASS** |
| T-15 | Rows Y/Z live smoke | Omniflow deploy + provisioning reload | **DEFERRED** — verify-work (V1) |
| T-16 | Six `/analytics/{slug}` routes | Operator embed smoke | **DEFERRED** — verify-work (V1) |

### Test output (T-1)

```
running 6 tests
test breakdown_query_returns_all_accounts_from_latest_snapshot ... ok
test account_variable_query_prefers_funded_account_over_zero_wallet ... ok
test portfolio_breakdown_sql_uses_latest_snapshot_lateral ... ok
test portfolio_overview_table_title_and_grid ... ok
test forecast_horizons_ml_banner_and_no_value ... ok
test account_id_variable_uses_abs_balance_sort ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Acceptance criteria matrix (BUG-0009)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **(Y)** | Grafana ds/query **200** with non-empty panel values at default `$account_id`; ML honest empty-state | **PASS** (code) / **DEFERRED** (runtime) | Y1: ABS(balance) sort + no `current`; Y2: banner + noValue on panels 7–10; T1: 6/6 tests. Live cashflow/forecast default-load smoke **DEFERRED** until deploy |
| **(Z)** | Cross-account value overview in analytics; portfolio shows all synced asset accounts | **PASS** (code) / **DEFERRED** (runtime) | Z1: LATERAL unnest 3-row fixture; Z2: "All accounts (latest snapshot)" at y=4 below stat row; `total_eur` stat panel 1 at y=0. Live 3-row omniflow probe **DEFERRED** until deploy |
| Regression | Six analytics routes; ds/query portfolio pie UNION **200** | **PASS** (static) / **DEFERRED** (live) | UNION pie SQL unchanged (panel 8); no backend/API changes; full route smoke at verify-work |

**Summary:** Z1–Y2 + T1 **PASS** on automated/static path; full acceptance runtime deferred with `OPERATOR_DEPLOY_PENDING` + `GRAFANA_PROVISIONING_RELOAD_PENDING`.

## Architecture compliance (DEC-0068)

### Z1 — Portfolio breakdown SQL

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Latest snapshot subquery | `ORDER BY snapshot_date DESC LIMIT 1` | Panel 5 `rawSql` | PASS |
| LATERAL unnest | `CROSS JOIN LATERAL jsonb_array_elements` | Present in panel 5 | PASS |
| Reject global LIMIT 1 | No LIMIT on cross-join | Test + review confirm absent | PASS |
| ABS sort on rows | `ORDER BY ABS((elem->>'balance')::float) DESC` | Panel 5 SQL | PASS |

### Z2 — Cross-account overview

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Portfolio-only placement | Overview on `uid: portfolio` | `portfolio.json` panel 5 | PASS |
| Below stat row | y=4 immediately under y=0 stat row | `gridPos.y: 4` | PASS |
| Table title | "All accounts (latest snapshot)" | Panel 5 title | PASS |
| Supplementary wealth link | `/wealth` text panel (Z3) | Panel 12 markdown | PASS |
| Rejected seventh dashboard | Not added | No new dashboard file | PASS |

### Y1 — Account variable default

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| ABS(balance) sort | `ORDER BY ABS(COALESCE(a.balance, 0)) DESC, a.name ASC` | cashflow + forecast-horizons templating | PASS |
| Dashboards | cashflow + forecast-horizons | Both updated | PASS |
| Omit `current` on account_id | No saved default | `account_id` var has no `current` key | PASS |
| `$forecast_variant` unchanged | baseline default preserved | `current: baseline` on forecast_variant only | PASS |

### Y2 — ML empty-state

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Text banner | ML not enabled copy (DEC-0066 aligned) | Panel 13 markdown | PASS |
| noValue on ML panels | `"ML unavailable"` on 7–10 | All four panels | PASS |
| No dynamic hide rules | Static panels only | No hide/show rules added | PASS |
| US-0013 boundary | ML enablement out of scope | Banner references US-0013 | PASS |

### Frozen boundaries

| Boundary | Verdict |
|----------|---------|
| Provisioning-only — no backend/React changes | PASS |
| Portfolio UNION pie SQL unchanged (BUG-0004 K) | PASS |
| Six analytics routes unchanged (US-0011) | PASS |
| No Grafana dynamic hide rules | PASS |
| No seventh overview dashboard | PASS |

## Plan-verify alignment

Plan-verify PASS (2026-06-06): 2/2 acceptance rows Y/Z covered; 6/6 tasks mapped; 0 gaps. QA confirms code tasks Z1, Z2, Y1, Y2, T1 implemented per DEC-0068. V1 correctly deferred to verify-work per operator gate in `sprints/quick/Q0016/uat.md`.

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

## Verify-work readiness

| Gate | Status |
|------|--------|
| Code QA (Z1–Y2 + T1) | **READY** |
| Operator deploy Q0016 provisioning | **PENDING** |
| GRAFANA_PROVISIONING_RELOAD | **PENDING** |
| V1 omniflow smoke (Y/Z rows) | **PENDING** — blocked on deploy |

## Next phase

**`/verify-work`** — after operator deploys Z1–Y2 + T1 and runs Grafana provisioning reload:

1. `/analytics/cashflow` — default non-flat forecast series (funded account, not zero wallet)
2. `/analytics/forecast-horizons` — ML banner visible; ML panels show `ML unavailable`
3. `/analytics/portfolio` — 3-row overview table + `total_eur` stat above fold
4. `POST /analytics/grafana/api/ds/query` — cashflow + portfolio panels **200**
5. Six `/analytics/{slug}` routes render in embed
6. Portfolio pie UNION ds/query **200** (BUG-0004 K regression)
7. Do **not** save `$account_id` in Grafana UI (runbook §17)

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
