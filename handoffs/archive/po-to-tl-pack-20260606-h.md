# PO→TL handoff archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500` (post-BUG-0009 release refresh-context)
- Source: `handoffs/po_to_tl.md`
- Archived sections: BUG-0009 research + discovery (closed work item)
- Retained hot tail: US-0011 research onward

---

## research-20260606-bug0009 — BUG-0009 Grafana variable default, overview panel, ML empty-state

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-06  
**Work item:** BUG-0009  
**Orchestrator:** `auto-20260606-bug0009-001`  
**Next phase:** `/architecture`

### Summary

Web + code research completed for BUG-0009. Expanded **[R-0064](docs/engineering/research.md#r-0064--bug-0009-grafana-panel-emptiness-vs-cross-account-overview-gap)** with bounded recommendations for all discovery open questions. Primary emptiness (Y) is **`$account_id` alphabetical default to zero-balance acct 116**, not datasource regression. Cross-account gap (Z) needs **SQL subquery fix (Z1)** plus **portfolio overview table (Z2)**. ML panels empty on omniflow is expected (DEC-0049) — close with **banner empty-state (Y2)**, not US-0013 scope. No host `.env` or secrets read.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Variable default (Y1)** | [R-0064 §2](docs/engineering/research.md#r-0064--bug-0009-grafana-panel-emptiness-vs-cross-account-overview-gap) | **`ORDER BY ABS(COALESCE(balance,0)) DESC, name`** on `$account_id` query; omit saved `current` in provisioning JSON |
| **Portfolio SQL (Z1)** | [R-0064 §3](docs/engineering/research.md#r-0064--bug-0009-grafana-panel-emptiness-vs-cross-account-overview-gap) | Latest-snapshot subquery + `LATERAL jsonb_array_elements`; remove global `LIMIT 1` on cross-join |
| **Overview panel (Z2)** | [R-0064 §4](docs/engineering/research.md#r-0064--bug-0009-grafana-panel-emptiness-vs-cross-account-overview-gap) | **Portfolio dashboard** stat row + fixed all-accounts table; `/wealth` link supplementary only (Z3 docs) |
| **ML empty-state (Y2)** | [R-0064 §5](docs/engineering/research.md#r-0064--bug-0009-grafana-panel-emptiness-vs-cross-account-overview-gap) | Text banner + `noValue` on ML panels when `ml_enhanced` count=0; US-0013 owns enablement |
| **React parity** | [R-0064 §2](docs/engineering/research.md#r-0064--bug-0009-grafana-panel-emptiness-vs-cross-account-overview-gap) | Forecast API also `ORDER BY name` — optional follow-up; not required for Grafana AC Y |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Variable default: max ABS(balance) vs non-zero forecast vs React last-selected? | **ABS(balance) DESC** in Grafana variable query — simplest, no backend change; React localStorage sync deferred |
| Overview placement: portfolio-only vs every dashboard vs new landing route? | **Portfolio-only** table + existing stat row; reject seventh dashboard for MVP |
| AC Z: does React `/wealth` link suffice? | **No as sole fix** — Grafana overview panel required; `/wealth` documented supplementary in verify-work |
| ML panel policy: hide vs banner when no `ml_enhanced`? | **Banner + noValue text** — reject dynamic hide rules (Grafana 11 complexity) |

### Risks surfaced (carry to architecture)

1. **All-zero balances** — variable default falls back to alphabetical (same as today)
2. **Overview only on portfolio** — cashflow-first operators may miss until sidebar navigation
3. **ML charts still empty below banner** — acceptable until US-0013; set copy expectation
4. **Manual Grafana save** — accidental `current` variable bake-in breaks provisioning intent

### Recommended next steps

1. `/architecture` — DEC for Y1 variable query, Z1/Z2 portfolio panels, Y2 ML banner contract
2. `/sprint-plan` — Decompose acceptance rows Y/Z (5 tasks Y1–Y3, Z1–Z3 per discovery)
3. `/execute` — provisioning JSON + SQL fixtures only (no backend unless architecture bundles React parity)

---

## discovery-20260606-bug0009 — BUG-0009 Grafana empty panels & account overview

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-06  
**Work item:** BUG-0009 (defect)  
**Orchestrator:** `auto-20260606-bug0009-001`  
**Next phase:** `/research` → `/architecture`

### Summary

Discovery **confirms both sub-defects Y and Z** with public omniflow curl probes (no secrets). **Primary finding:** Grafana is **not** failing at the datasource/SQL transport layer post-BUG-0004 — `POST /analytics/grafana/api/ds/query` returns **200** with data for portfolio, subscriptions, budgets, and platform-health panels. Operator-perceived emptiness on account-scoped dashboards is primarily **`$account_id` defaulting to zero-balance account 116 (Cash wallet)** while funded Giro **114** has non-empty forecast series. **Secondary:** forecast-horizons ML panels query `ml_enhanced` — zero rows on omniflow until **US-0013**. **Defect Z:** portfolio account-breakdown SQL returns **1 of 3** accounts (`LIMIT 1` bug) and **no dedicated cross-account overview panel** exists in analytics provisioning.

### Confirmed root causes

| Sub | Root cause | Fix task |
|-----|------------|----------|
| **Y1** | `$account_id` variable defaults alphabetically to acct **116** with flat zero `forecast_balance_daily`; acct **114** has data when selected | **Y1** |
| **Y2** | ML panels on `forecast-horizons` require `ml_enhanced` computation — none on external profile (DEC-0049) | **Y2** |
| **Y3** | Datasource/UNION regression **ruled out** — BUG-0003 H + BUG-0004 K pass on live host | verify-work only |
| **Z1** | Portfolio breakdown SQL: global `LIMIT 1` on cross-join truncates to single account row | **Z1** |
| **Z2** | No cross-account value overview panel in Grafana analytics dashboards | **Z2** |

### Live probe summary (2026-06-06, financegnome.omniflow.cc)

| Probe | Result |
|-------|--------|
| `GET /health` | **200** `{"status":"ok"}` |
| `GET /api/v1/sync/status` | **200** `success` (manual sync `2026-06-05T21:39:31Z`) |
| `GET /analytics/grafana/api/health` | **200** Grafana 11.0.0, database ok |
| `GET /analytics/grafana/api/datasources` | **200** `FlowFinancePostgreSQL` → `postgres:5432/flow_finance_ai` |
| `POST …/api/ds/query` account variable | **200** — 3 asset accounts: 116, 114, 115 |
| `POST …/api/ds/query` cashflow acct **116** | **200** — 731 daily rows, all balance **0** |
| `POST …/api/ds/query` cashflow acct **114** | **200** — non-zero negative balances |
| `POST …/api/ds/query` portfolio total | **200** — `total_eur` **-3395.75** |
| `POST …/api/ds/query` portfolio UNION pie | **200** — 2 slices (post-BUG-0004 K fix verified) |
| `POST …/api/ds/query` portfolio breakdown (current SQL) | **200** — **1 row** (Cash wallet 0) |
| `POST …/api/ds/query` overview (fixed subquery) | **200** — **3 rows** (0, -3395.75, 0) |
| `POST …/api/ds/query` subscriptions | **200** — 3 confirmed, 6 pending |
| `POST …/api/ds/query` ml_enhanced count | **200** — **0** computations |
| `GET /api/v1/wealth` | **200** — 3 accounts, `total_eur: -3395.75` |
| `GET /analytics/grafana/d/cashflow/cashflow?kiosk=tv` | **200** (embed HTML; assets under `/analytics/grafana/public/build/` **200**) |
| `GET /analytics/cashflow` (SPA route, unauthenticated curl) | **404** — advisory; client-side nav from authenticated shell expected |

### Recommended sprint shape (post-architecture)

| Order | Task | Files (primary) |
|-------|------|-----------------|
| 1 | **Z1** Portfolio breakdown SQL fix | `grafana/provisioning/dashboards/analytics/portfolio.json` |
| 2 | **Z2** Cross-account overview panel | `portfolio.json` (+ optional analytics shell copy) |
| 3 | **Y1** `$account_id` default strategy | `cashflow.json`, `forecast-horizons.json` |
| 4 | **Y2** ML panel empty-state / hide when no `ml_enhanced` | `forecast-horizons.json` |
| 5 | **Y3/Z3** verify-work omniflow | acceptance rows Y/Z |

**Tests:** SQL fixture for portfolio breakdown + overview queries; variable default integration test or provisioning snapshot; operator smoke with default account auto-select.

### Architecture open questions

1. **Variable default strategy:** Prefer max `ABS(balance)` asset account vs first with non-zero forecast vs match React forecast page last-selected account?
2. **Overview placement:** Portfolio-only table vs new stat row on each dashboard vs analytics landing panel?
3. **AC Z equivalence:** Does React `/wealth` link satisfy cross-account overview if Grafana panel added — or must overview be inside iframe?
4. **ML panel policy:** Hide panels vs show "baseline only (US-0013)" banner when `ml_enhanced` absent?

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#BUG-0009` | Discovery table + fix tasks Y1–Z3 | pass (updated) |
| `docs/product/acceptance.md` BUG-0009 | Rows Y/Z still valid | pass — no AC rewrite |
| `docs/product/vision.md` BUG-0009 discovery | Y/Z causes documented | pass (updated) |
| `handoffs/intake_evidence/intake-20260605-grafana-account-overview.json` | small-intake-pack complete | pass |
| [R-0064](docs/engineering/research.md#r-0064--bug-0009-grafana-panel-emptiness-vs-cross-account-overview-gap) | Discovery findings persisted | pass |

`triad_hot_surface`: rollover 1 section (US-0012 discovery) → `handoffs/archive/po-to-tl-pack-20260605-g.md`; BUG-0009 discovery retained hot tail; retained_body_lines=432; prior duplicate pack `handoffs/archive/po-to-tl-pack-20260605-f.md`

### Related (do not merge scope)

- **BUG-0010/0012** — mirror balance fixes DONE; negative Giro balance is valid data source for panels, not empty-data bug
- **US-0013** — ML overlay enablement on omniflow (Y2 root cause for ML panels only)
- **BUG-0001 B** — Grafana assets resolve via `GF_SERVER_ROOT_URL` + `<base href>` — verified **200** on live host

### Recommended next steps

1. `/research` — bounded spike on Grafana variable default patterns + overview panel SQL (extends R-0064)
2. `/architecture` — DEC for variable default, overview panel contract, ML empty-state policy
3. `/sprint-plan` — Decompose acceptance rows Y/Z after architecture

---
