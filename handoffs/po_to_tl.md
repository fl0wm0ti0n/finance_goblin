## Sub-defects confirmed

| AC | Verdict | Key evidence |
|----|---------|--------------|
| **BR** | **NOT confirmed (localhost)**; **OPEN (omniflow)** | Browser: non-active selected → `deleteDisabled=false`, title *Delete this plan*; `planSelector.test.ts` 8/8 |
| **BS** | **CONFIRMED** | Sole active plan: disabled + generic tooltip; no inline guidance for create → activate → delete workflow |

## Root-cause chain (code + live)

| Layer | Finding |
|-------|---------|
| **Selector (Q0031)** | `PlanningPage.tsx` L111–114: `activePlanId = resolveDisplayedPlanId(plans, selectedPlanId)` — dropdown drives displayed plan ✓ |
| **Delete guard** | L490: `activePlanIsSelected = isDeleteDisabled(data, activePlanId)`; L670–675: disabled when globally active plan selected |
| **Sole-plan UX** | Tooltip-only copy assumes another plan exists; sole-plan operator sees permanent gray with no actionable path |
| **API** | `GET /api/v1/plans` — 1 plan initially; 2 after discovery probe create; **DEC-0082** DELETE 409 unchanged |
| **Deployed bundle** | `assets/index-CJ94Af9n.js` includes *Set another plan active* string — Q0031-era frontend on localhost |

## UX / design recommendation (PO)

| Option | Notes | PO preference |
|--------|-------|---------------|
| **A** | Inline hint when sole active plan: *Create another scenario, set it active, then delete this plan* | **Preferred** — satisfies **BS** without backend change |
| **B** | Link/button to create plan from disabled delete row | Higher UI churn; defer unless research prefers |
| **C** | Allow delete active sole plan with auto-deactivate | Violates **DEC-0082**; reject |

## Architecture gates (research carry)

| Gate | Question | PO default |
|------|----------|------------|
| **GATE-COPY-1** | Sole-plan disabled copy placement | Inline text below **Delete plan** row when `plans.length===1 && sole.is_active` |
| **GATE-DEPLOY-1** | Omniflow **BR** verification | Operator **FRONTEND_DEPLOY** then 2-plan smoke on `/planning` |
| **GATE-SCOPE-1** | Backend change? | **Frontend-only** — extends **DEC-0082** UX, not DELETE contract |
| **GATE-TEST-1** | Regression coverage | Extend `planSelector.test.ts` or PlanningPage fixture for sole-plan copy predicate |
| **GATE-DEC-1** | New DEC? | Only if copy policy needs canonical record; default **no new DEC** |

## Acceptance rows (unchanged intent)

- **(BR)** Multi-plan non-active selection enables delete post-**FRONTEND_DEPLOY** — localhost PASS; omniflow TBD
- **(BS)** Sole active plan: disabled + **clear** create→activate→delete guidance — **CONFIRMED gap**

## Research references

- [R-0096](docs/engineering/research.md#r-0096--bug-0024-plan-delete-still-disabled-live-post-q0031) — update status to discovery-complete; add localhost 2-plan probe + H verdicts

## Recommended next phase

`/research` — fulfill R-0096; freeze gates; size `/quick` if frontend-only copy fix.

---

# research-20260613-bug0026 — BUG-0026 Forecast monthly Income card vs chart mismatch

**From:** Tech Lead **To:** Dev (via architecture) **Bug:** BUG-0026 **Run:** `auto-20260613-bug0026`
**Date:** 2026-06-13 **Next phase:** `/architecture` (role: tech-lead)

## Research summary

[R-0098 §1–9](docs/engineering/research.md#r-0098--bug-0026-forecast-monthly-income-card-vs-chart-mismatch) fulfilled. Live repro confirmed on account **114** (`series[0]` June income **0.00**, `series[1]` July **3266.16**). Root cause remains frontend **unlabeled `series[0]`** vs full chart series — not backend.

## Frozen gates

| Gate | Research verdict |
|------|------------------|
| **GATE-MONTH-1** | Skip partial head when `series[0].income === 0` and a later month exists → `series.find(income>0) ?? series[1]`; else `series[0]` |
| **GATE-LABEL-1** | Shared subtitle above card grid: **"Forecast for {Month YYYY}"** |
| **GATE-SCOPE-1** | Frontend-only; no API/`project.rs` change; **DEC-0089** cards unchanged by category filter |
| **GATE-TEST-1** | Vitest pure helper + partial-month fixture (pattern: `planSelector.test.ts`); Playwright not required |
| **GATE-DEC-1** | **No new DEC** — architecture documents forecast summary month contract |

## Recommended execute shape

`/quick` — 2–4 tasks: `forecastSummaryMonth.ts` helper, ForecastPage wire + subtitle, vitest, deferred V1 smoke.

## Recommended next phase

`/architecture` — formalize helper contract, acceptance trace BZ/CA, size quick sprint.

---

# discovery-20260613-bug0026 — BUG-0026 Forecast monthly Income card vs chart mismatch

**From:** PO **To:** Tech Lead **Bug:** BUG-0026 **Run:** `auto-20260613-bug0026`
**Date:** 2026-06-13 **Next phase:** `/research` (role: tech-lead)
**Intake evidence:** `handoffs/intake_evidence/intake-20260613-forecast-income-card-mismatch.json` (read-only)

## Discovery summary

Code audit + live API probe confirm intake hypothesis: summary cards bind to **unlabeled `series[0]`** while **MonthlyChart** plots the **full** monthly series. On account **114** (Raiffeisenbank Giro, operator repro), `GET http://localhost:18080/api/v1/forecast/monthly?account_id=114` returns `series[0]` **2026-06-01** with **income 0.00** (fixed 86.02, variable 2866.57, free cashflow -2952.59) and `series[1]` **2026-07-01** with **income 3266.16** — matching operator screenshot and chart bars. **Not** a **BUG-0012** backend bucket regression; projection engine correctly projects salary from month 2 per recurring due dates.

## Sub-defects confirmed

| AC | Verdict | Key evidence |
|----|---------|--------------|
| **BZ** | **CONFIRMED** | Income card **0.00** while chart shows **~3266** Income bars from **2026-07**; live API `series[0].income=0.00`, `series[1].income=3266.16` |
| **CA** | **CONFIRMED** | Cards show metric labels only — no month reference; `ForecastPage.tsx` L148–152 `monthlySummary = series[0]`; operator cannot reconcile card vs chart |

## Root-cause chain (code + live)

| Layer | Finding |
|-------|---------|
| **Summary cards** | `ForecastPage.tsx` L148–152: `monthlySummary = series[0]`; L312–330 render Income/Fixed/Variable/Free cashflow **without month label** |
| **Chart** | `MonthlyChart.tsx` maps **entire** `series` to x-axis months and bar data — correct |
| **API** | `GET /api/v1/forecast/monthly` returns ordered monthly points from `forecast_cashflow_monthly`; no summary-month hint field |
| **Projection** | `project.rs` accumulates recurring income per due date; **current partial month** (June) has **no salary due** in remaining days → income **0.00** by design |
| **Category filter** | **DEC-0089** intact — helper text L278–281 scopes filter to trend chart only; cards unchanged ✓ |
| **BUG-0012** | **RULED OUT** — API non-zero income from month 2; bucket_sources `income: config` on July+ |

## UX / design recommendation (PO)

| Option | Notes | PO preference |
|--------|-------|---------------|
| **A** | Label summary cards with **reference month**; default to **next full month** (first month with projected salary) or **current calendar month** with explicit partial-month copy | **Preferred** — minimal change; fixes trust gap |
| **B** | Sync card values to chart hover/selected month | Higher interaction cost; defer unless operator requests |
| **C** | Rolling 12-month aggregate on cards | Changes metric semantics; not Finanzguru parity |

**Vision tension:** BUG-0012 discovery stated cards = `series[0]`; discovery confirms that semantics misleads when partial month lacks salary. Research should freeze month-selection policy (**GATE-MONTH-1**) before execute.

## Architecture gates (research carry)

| Gate | Question | PO default |
|------|----------|------------|
| **GATE-MONTH-1** | Which month drives summary cards? | **Next full month** or first month with non-zero income when current partial month is zero |
| **GATE-LABEL-1** | Month label placement | Shared subtitle above card grid: "Forecast for **July 2026**" |
| **GATE-SCOPE-1** | Backend change? | **Frontend-only** — no `project.rs` / API contract change unless TL finds gap |
| **GATE-TEST-1** | Regression test | Vitest/Playwright: when `series[0].income=0` and `series[1].income>0`, cards must not show unlabeled 0.00 Income |
| **GATE-DEC-1** | New DEC? | Only if month-selection policy needs canonical record |

## Acceptance rows (unchanged)

- **(BZ)** Income card consistent with chart for same labeled reference month — not 0.00 card vs ~€3000 chart bars
- **(CA)** Cards show which month they represent — not unlabeled `series[0]` when misleading

## Research questions (carry from R-0098)

1. Freeze **GATE-MONTH-1**: next full month vs current month vs first non-zero income month — operator mental model on `/forecast` Monthly
2. Month label UX pattern — subtitle vs per-card micro-label (Finanzguru/shadcn stat-card precedents)
3. Edge cases: all-zero series, single-month series, month-end boundary (June 30 vs July 1)
4. Test fixture: mock monthly response with `series[0].income=0`, `series[1].income>0`

## Related work

**BUG-0012** DONE (**Q0014**); **US-0018** / **DEC-0089**; **US-0002** forecast monthly view; [R-0098](docs/engineering/research.md#r-0098--bug-0026-forecast-monthly-income-card-vs-chart-mismatch)

## Recommended next phase

`/research` — freeze month-selection policy, label UX, frontend-only scope, test strategy.

---

## Hypothesis resolution (final)

| ID | Verdict |
|----|---------|
| H1 | CONFIRMED — no futures wallet row |
| H2 | CONFIRMED — linear NULL by design |
| H3 | CONFIRMED — `entryValue` available in payload |
| H4 | RULED OUT |
| H5 | CONFIRMED — blocked by `crypto_value_eur=0` |

## Architecture gates (mandatory decisions)

| Gate | Question | TL default |
|------|----------|------------|
| **GATE-BO-1** | Wallet parse hardening | Equity keys + `code==0` validation + parse-skip logging + OpenAPI wiremock |
| **GATE-BP-1** | Value EUR source | `entryValue` display-only (D1) vs tier-2 mark-price |
| **GATE-AGG-1** | Subtotal | `sum(market_value_eur)` wallet-only — no linear merge |
| **GATE-BQ-1** | Return denominator | Wallet-priced `crypto_value_eur`; baseline on first priced sync |
| **GATE-DEC-1** | New DEC? | No unless subtotal contract changes |

## Provisional fix stack

1. **P0 BO:** `bitunix.rs` wallet parse + sync observability; `recompute_pnl` prices futures row.
2. **P1 BP:** `pnl.rs` linear `entryValue` → display `value_eur`; wire through `holdings_all`.
3. **P1 BQ:** Resolves when BO priced + baseline captured.
4. **P2 UX:** Optional holdings_count footnote (wallet vs positions).

## Acceptance rows (unchanged)

- **(BO)** Bitunix card + `crypto.subtotal_eur` ~operator portfolio — not €0 with 11 positions
- **(BP)** Value EUR populated when prices available — not all em dash
- **(BQ)** Total return % when baseline exists — not — with non-zero unrealized

## Research artifact

[R-0093 §5](docs/engineering/research.md#r-0093--bug-0023-crypto-wealth-eur-values-live-regression) — extended with research phase findings and web refs.

## Recommended next phase

`/architecture` — freeze GATE decisions, document contracts, size `/quick` or sprint.

---

# discovery-20260612-bug0023 — BUG-0023 Crypto Wealth EUR values missing (live regression)

**From:** PO **To:** Tech Lead **Bug:** BUG-0023 **Run:** `auto-20260612-bug0023`
**Date:** 2026-06-12 **Next phase:** `/research` (role: tech-lead)
**Intake evidence:** `handoffs/intake_evidence/intake-20260612-crypto-eur-values.json` (read-only)

## Discovery summary

Live probe **`GET http://localhost:18080/api/v1/wealth`** (2026-06-12, no secrets) confirms all three intake sub-defects on Bitunix-connected Wealth Crypto tab. Sync timestamp `2026-06-12T21:15:31Z`; **11** linear positions ingested; unrealized PnL EUR path active.

## Sub-defects confirmed

| AC | Verdict | Key evidence |
|----|---------|--------------|
| **BO** | **CONFIRMED** | `crypto.subtotal_eur=-0.0`; `bitunix.subtotal_eur=-0.0`; `holdings_count=11` |
| **BP** | **CONFIRMED** | All **11** `holdings_all[].value_eur=null`; unrealized per-row populated; `holdings_top=[]` |
| **BQ** | **CONFIRMED** | `pnl.unrealized_eur=376.83`; `pnl.total_return_pct=null` |

## Root-cause chain (code + live)

| Layer | Finding |
|-------|---------|
| **Wallet ingest** | API response has **only** `product_type=linear` — **no** `futures` USDT wallet row → **H1 CONFIRMED** |
| **Aggregation** | `wealth/service.rs` subtotal = `sum(market_value_eur)` → **€0** when all linear NULL |
| **Linear pricing** | `portfolio/pnl.rs` L30–54 sets `market_value_eur: None` for linear per **DEC-0064** → **H2 CONFIRMED** |
| **PnL return** | `portfolio/service.rs` L60–64: `total_return_pct` None when `crypto_value_eur=0` → **BQ** |
| **Deploy gap** | **H4 RULED OUT** — recent sync + unrealized EUR computed |

## Acceptance rows (unchanged)

- **(BO)** Bitunix card + `crypto.subtotal_eur` ~operator portfolio — not €0 with 11 positions
- **(BP)** Value EUR column populated when prices available — not all em dash
- **(BQ)** Total return % when baseline exists — not — with non-zero unrealized

## Research questions (carry from R-0093)

1. Wallet equity parse — why no `futures` row despite ~€2000 Bitunix app? (`bitunix.rs` `parse_futures_wallet`, `resolve_futures_account`)
2. Per-position Value EUR — mark-price/notional display field vs **DEC-0064** amend?
3. Subtotal contract — wallet equity only vs exposure sum with double-count guard (**DEC-0080**)
4. Total return denominator — use wallet `crypto_value_eur` when linear excluded from subtotal sum
5. SQL probe: `SELECT product_type, asset, quantity, market_value_eur, unrealized_pnl_eur FROM exchange_holdings WHERE exchange_id='bitunix'`

## Related decisions / bugs

**BUG-0014** AP/AQ (DONE, live deferred); **DEC-0064**, **DEC-0080**, **DEC-0081**; [R-0093](docs/engineering/research.md#r-0093--bug-0023-crypto-wealth-eur-values-live-regression)

## Recommended next phase

`/research` — wallet API shape verification, mark-price options, aggregation/display contract, SQL probe.

---

# intake-20260612-bug0023 — BUG-0023 Crypto Wealth EUR values missing (live regression)

**From:** PO **To:** Tech Lead **Bug:** BUG-0023 **Run:** `intake-20260612-crypto-eur-values`
**Date:** 2026-06-12 **Next phase:** `/discovery` (role: po)
**Intake evidence:** `handoffs/intake_evidence/intake-20260612-crypto-eur-values.json` (`small-intake-pack`, kind=bug)

## Bug summary

Operator screenshot (2026-06-12): Wealth **Crypto** tab — **11** Bitunix `linear` positions with **correct native qty**, but:

- Bitunix exchange card **€ -0,00** (operator Bitunix app ~**€2000** portfolio)
- Holdings **Value EUR** column all **—**
- **Unrealized €378,02** shown; **Total return —**

Residual **live regression** vs **BUG-0014** AP/AQ (code PASS, operator smoke deferred).

## Acceptance rows (canonical)

- **(BO)** Crypto subtotal / exchange card ~operator portfolio — not €0 with 11 positions
- **(BP)** Per-position **Value EUR** populated when prices available — not all em dash
- **(BQ)** **Total return %** when baseline exists — not — with non-zero unrealized

## Scope / risks

| Area | Note |
|------|------|
| Wallet equity | **DEC-0080** — `product_type=futures` wallet row may be missing (`bitunix.rs` array parse) |
| Linear pricing | **DEC-0064** — `market_value_eur` NULL for linear by design; may need `exposure_eur` display field |
| Aggregation | `wealth/service.rs` subtotal = `sum(market_value_eur)` only |
| PnL | `portfolio/service.rs` total_return when `crypto_value_eur` zero |
| UI | `WealthPage.tsx` Value EUR / exchange card |
| Ops | Verify Q0022 deploy + exchange sync + PnL recompute before code attribution |

## Decomposition

**Single bug** — BO/BP/BQ one pricing/display cluster. Reopen **BUG-0014** rejected (DONE; new live evidence).

## Research

[R-0093](docs/engineering/research.md#r-0093--bug-0023-crypto-wealth-eur-values-live-regression)

## Recommended next phase

`/discovery` — live `GET /api/v1/wealth` probe; DB `exchange_holdings` wallet vs linear `market_value_eur`; Bitunix wallet API shape; mark-price availability for Value EUR column.

---

# intake-20260612-us0021 — US-0021 Subscription transaction explorer with rich filters

**From:** PO **To:** Tech Lead **Story:** US-0021 **Run:** `intake-20260612-subscription-tx-explorer`
**Date:** 2026-06-12 **Next phase:** `/discovery` (role: po)
**Intake evidence:** `handoffs/intake_evidence/intake-20260612-subscription-tx-explorer.json` (`small-intake-pack`, kind=story)

## Story summary

Operator report (German, 2026-06-12): subscription search/filter **falsch implementiert** — expects **transaction-first** search for expenses auto-detection missed, with filters for **wiederkehrend**, **account**, **Geldbereich**, **kategorie**, and ability to **mark/activate** found recurring patterns as subscriptions.

**Prior /ask analysis:** US-0020 **AC-1 met** shipped contract (account/payee/interval on recurrence groups via `detect_recurrence_groups`). Gap is **scope expansion**, not a US-0020 defect — new story **US-0021**.

## Acceptance rows (canonical)

- **AC-1** Transaction search — individual expense txs, not recurrence-only candidates
- **AC-2** Rich filters — account, payee, category, Geldbereich, date range (+ optional amount/recurring)
- **AC-3** Pattern hint — recurring suggestion on filtered txs below auto-detection threshold
- **AC-4** Manual activate — confirm tx group as subscription/standing order (DEC-0085/0099)
- **AC-5** Regression — US-0020 tags/majority category + US-0003/US-0008 unchanged
- **AC-6** OIDC external profile smoke

## Scope / risks

| Area | Note |
|------|------|
| Backend | New transaction search API or `/discover?mode=transactions`; reuse `load_expense_transactions` + SQL filters; hint via `detect_recurrence_groups` on subset |
| Frontend | Enhance `SubscriptionsPage.tsx` Discover tab — dual-mode (Transactions \| Patterns) TBD at discovery |
| Geldbereich | Join `accounts.payload` `account_role` per **DEC-0111**; reuse `formatAccountRole` labels |
| Category | **US-0018** `CategoryFilter` / catalog |
| Confirm | Extend or reuse `POST /api/v1/subscriptions/discover/confirm` with explicit `transaction_ids` |
| Risk | Full-window scan — cap/pagination required; account filter default recommended |
| Regression | Do not lower global auto-detection thresholds without documented contract |

## Decomposition

**Single story** — transaction search + filters + manual activate is one subscription-ops vertical slice extending US-0020.

**Alternatives rejected:**

- BUG on US-0020 — AC-1 met narrowly
- Split API vs UI — no independent user value

## Research

[R-0092](docs/engineering/research.md#r-0092--us-0021-subscription-transaction-explorer-vs-recurrence-only-discover) — gap table, reusable components, architecture questions.

## Recommended next phase

`/discovery` — layout (dual-mode vs replace), API shape, hint threshold, Geldbereich join, pagination cap, operator repro on localhost:18080.

---

# intake-20260611-bug0022 — BUG-0022 Plan delete still broken (selector ignores dropdown)

**From:** PO **To:** Tech Lead **Bug:** BUG-0022 **Run:** `intake-20260611-plan-delete-regression`
**Date:** 2026-06-11 **Next phase:** `/discovery` (role: po)
**Intake evidence:** `handoffs/intake_evidence/intake-20260611-plan-delete-regression.json` (`small-intake-pack`, kind=bug)

## Bug summary

Operator report: *Delete plan geht immer noch nicht* — plan delete UX from **BUG-0014 AS1** (**Q0022**, **DEC-0082**) still unusable.

**Hypothesis (code audit, high confidence):** `PlanningPage.tsx` `activePlanId` `useMemo` resolves `plans.find(is_active).id` **before** `selectedPlanId`. When any globally active plan exists, the **Active plan** dropdown cannot switch context; **Delete plan** stays disabled because `activePlanIsSelected` is always true.

## Acceptance rows (canonical)

- **(BM)** Non-active plan selected → delete enabled → plan removed after confirm.
- **(BN)** Active plan → delete disabled in UI + **409** on API (**DEC-0082** preserved).

## Scope / risks

| Area | Note |
|------|------|
| Frontend | `frontend/src/pages/PlanningPage.tsx` L110–113 selector `useMemo`; L643–683 dropdown + delete affordance |
| Backend | `DELETE /api/v1/plans/:id` + **DEC-0082** — **no change expected** |
| Single-plan edge | Only one active plan → delete always disabled today; discovery should decide sole-plan policy |
| Regression | `plan_delete_api_tests`, `planningFeedback.test.ts`, npm frontend suite |

## Decomposition

**Single bug** — not reopening **BUG-0014** (DONE). Split rejected: one selector-state root cause.

## Recommended next phase

`/discovery` — confirm repro on localhost:18080 with 2+ plans; validate `selectedPlanId` vs `activePlanId` wiring; scope sole-plan UX if needed.

---

# discovery-20260611-bug0021 — BUG-0021 Frontend UX polish (category filter delay, wealth role column)

**From:** PO **To:** Tech Lead **Bug:** BUG-0021 **Run:** `auto-20260610-bug0019`
**Date:** 2026-06-11 **Next phase:** `/research` (role: tech-lead)
**Intake evidence (read-only):** `handoffs/intake_evidence/intake-20260609-frontend-ux.json` (`intake-20260609-frontend-ux`, `small-intake-pack`, kind=bug)
**Prior context:** BUG-0020 released Q0028 (`bug0020-q0028`, DEC-0109); bug queue continues; operator deferred ForecastPage TS6133 rebuild from BUG-0020 release notes.

## Bug summary

Post-US-0020 rebuild (localhost:18080), two P3 frontend polish gaps from UI audit 2026-06-09:

- **EA (UI-011):** Forecast → **Monthly** or Wealth → **Overview** — **Loading category filter…** Suspense fallback visible **3–5s** before combobox renders.
- **EB (UI-012):** Wealth → **Overview** → Account breakdown — **Role** column shows **—** for every account (Cash wallet, Giro, savings).

## Acceptance rows (canonical: `docs/product/acceptance.md` BUG-0021)

- **(BK)** CategoryFilter on Forecast Monthly and Wealth Overview becomes interactive within **~1s** of tab visit—not multi-second **Loading category filter…** Suspense fallback under normal local/omniflow load. *(= sub-defect EA)*
- **(BL)** Wealth account breakdown **Role** column shows Firefly account role/type when mirror provides it, or column is hidden/documented when unsupported—not permanent em dash for all rows. OIDC-enabled deploy regression checks pass. *(= sub-defect EB)*

## Reproduction / evidence pointers

| Ref | Surface | Pointer |
|-----|---------|---------|
| UI steps | EA | Forecast → Monthly or Wealth → Overview — `handoffs/intake_evidence/ui-audit-20260609-local.json` **UI-011** |
| UI steps | EB | Wealth → Overview → Account breakdown — **UI-012**, screenshot `wealth-page.png` |
| Lazy import | EA | `frontend/src/pages/ForecastPage.tsx` L20–22 `lazy(() => import("../components/category/CategoryFilter")…)`; monthly tab L276–282 Suspense fallback |
| Lazy import | EA | `frontend/src/pages/WealthPage.tsx` L15–17, L179–181 (overview Category spending card) |
| Component | EA | `frontend/src/components/category/CategoryFilter.tsx` — lightweight select/combobox; `useQuery` → `fetchCategories`; **no ECharts** |
| Separate lazy | EA | `CategoryTrendChart` lazy in same pages (ECharts) — loads **after** category selection, not the filter fallback |
| Out-of-scope twin | EA | `frontend/src/pages/PlanningPage.tsx` L31–32, L854–855 — same lazy CategoryFilter pattern (not in BK acceptance) |
| UI render | EB | `frontend/src/pages/WealthPage.tsx` L220 `{row.account_role ?? "—"}` |
| API type | EB | `frontend/src/lib/api.ts` `account_role?: string \| null` on wealth account row |
| API assembly | EB | `backend/src/wealth/service.rs` L80–102 maps `a.account_role` from repository row |
| SQL extract | EB | `backend/src/wealth/repository.rs` L31 `payload->>'account_role' AS account_role` (also L133 snapshot query) |
| Sync ingest | EB | `backend/src/firefly/mod.rs` L249–287 — stores full Firefly API `item` in `accounts.payload`; logs `attrs["account_role"]` from **attributes** |
| Contrast | EB | `backend/src/plan/repository.rs` L752 uses `t.payload->'attributes'->>'type'` for transactions — nested path pattern |
| Research spec | EB | `docs/engineering/research.md` R-0001 wealth SQL documents same root-level `payload->>'account_role'` |
| Snapshots | EB | `backend/src/wealth/service.rs` L271–283 snapshot `accounts` array inherits null `account_role` from query |
| Grafana | EB | `grafana/provisioning/dashboards/analytics/portfolio.json` — `elem->>'account_role'` from snapshot payload |

## Discovery observations (facts only — root cause is research scope)

### EA — CategoryFilter lazy-load delay

1. **Suspense is chunk-bound:** Fallback text **Loading category filter…** is shown until the `React.lazy` dynamic import resolves—not until `fetchCategories` returns. CategoryFilter itself is small (~96 LOC, react-query + API helper only).
2. **Acceptance surfaces match code:** BK scopes Forecast **Monthly** tab and Wealth **Overview**; both wrap CategoryFilter in `Suspense` with the reported fallback string.
3. **ECharts not in filter chunk:** `CategoryTrendChart` (imports `echarts-for-react`) is a **separate** lazy boundary; it mounts only when `categoryId` is set—unlikely cause of filter delay.
4. **Likely fix axis (research):** Eager/static import of CategoryFilter, route-level prefetch of the chunk, or a non-blocking skeleton—balance against P3 “avoid material bundle regression” (intake constraint).
5. **PlanningPage parity:** Same lazy pattern exists on Planning (out of BK) — research should decide whether fix is page-local or shared component import policy.

### EB — Wealth Role column empty

1. **UI contract is pass-through:** Frontend correctly renders API `account_role` or em dash; no missing column component.
2. **SQL path mismatch (strong hypothesis):** Repository reads `payload->>'account_role'` at JSON **root**, but Firefly sync persists the full API **item** (`upsert_account(…, item)`) where `account_role` lives under `attributes` (sync logs `attrs["account_role"]`).
3. **Filters may mask bug:** `active` / `include_net_worth` use same root-level `payload->>'…'` with `COALESCE(…, true)` — accounts still appear when attributes are nested.
4. **Downstream propagation:** Null `account_role` flows into `GET /api/v1/wealth`, daily snapshot payload, and Grafana portfolio SQL (`elem->>'account_role'`).
5. **Tests blind spot:** Integration seeds often use `payload: '{}'` — would not catch nested-path extraction gap.
6. **Display vs raw enum:** Firefly roles (`defaultAsset`, `cashWalletAsset`, `savingAsset`, etc. per R-0001) may need human labels for BL “useful” column—or hide when truly absent.

## Open questions for research

1. **EA timing proof:** On localhost:18080 / omniflow, measure lazy-chunk load vs categories API—confirm Suspense duration tracks JS import not network.
2. **EA fix choice:** Static import vs `import()` prefetch on route enter vs slimmer suspense UX—which satisfies BK ≤1s without violating bundle constraint?
3. **EA scope:** Include PlanningPage lazy CategoryFilter in fix for consistency?
4. **EB live payload:** Sample `accounts.payload` for Giro / savings / cash wallet—verify `attributes.account_role` present post-sync.
5. **EB fix axis:** SQL `payload->'attributes'->>'account_role'` vs denormalize at sync vs Rust mapping layer—which is canonical and snapshot-safe?
6. **EB display contract:** Map Firefly enums to operator labels, show raw enum, or hide column when null after fix?
7. **EB Grafana:** Does portfolio dashboard role column need the same backfill/re-snapshot pass?
8. **Regression gates:** Smoke BK (combobox ≤1s on tab switch), BL (non-dash role when mirror has metadata), OIDC deploy template per acceptance.

## Done definition (intake)

Operator perceives snappy category filter; Wealth breakdown Role column useful or removed.

## Recommended next phase

`/research` (role: tech-lead) — confirm EA chunk-load hypothesis with timing probes; confirm EB JSON path on live mirror; decide fix axes and regression scope; then architecture.

---

## Prior segment hot pointers (archived)

- discovery-20260610-bug0020 — **DONE** (released `bug0020-q0028`) — BI/BJ subscriptions list · DEC-0109 · R-0090
- refresh-context-20260611-bug0020 — **DONE** — `handoffs/curator_refresh.md` · bug queue advance to BUG-0021
- discovery-20260610-bug0019 — **DONE** (released `bug0019-q0027`) — Grafana CA/CB → BG/BH · DEC-0108 · R-0089

`triad_hot_surface`: prior BUG-0020 discovery body retained in archive refs; BUG-0021 discovery written fresh

---

## Research findings (2026-06-11) — R-0091

**Phase:** research (tech-lead) · **Orchestrator:** `auto-20260611-bug0021`

### EA — CategoryFilter delay (BK)

- **Confirmed:** Suspense fallback is `React.lazy` dynamic import resolution, **not** `fetchCategories` (API 2–5 ms on :18080).
- **Chunk:** `CategoryFilter-B6dgjo-5.js` = 1.5 KB; direct fetch ~1 ms; depends on already-loaded main split — delay is lazy waterfall on tab switch, not network-heavy categories call.
- **Recommended fix axis:** **static import** on ForecastPage + WealthPage (negligible bundle cost); PlanningPage parity optional (out of BK).

### EB — Wealth Role column (BL)

- **Confirmed:** `payload->>'account_role'` null for 3/3 asset accounts; `payload->'attributes'->>'account_role'` populated (`cashWalletAsset`, `defaultAsset`, `savingAsset`); API returns `account_role: null` for all.
- **Root cause:** repository SQL root path vs Firefly full-item sync storage under `attributes`.
- **Recommended fix axis:** SQL `payload->'attributes'->>'account_role'` in `wealth/repository.rs`; optional human label map for BL; snapshots/Grafana heal on next daily upsert.

**Next phase:** `/architecture` (role: tech-lead) — DEC for EA import policy + EB display contract; map to quick sprint.

---

## Architecture findings (2026-06-11) — DEC-0110, DEC-0111

**Phase:** architecture (tech-lead) · **Orchestrator:** `auto-20260611-bug0021`

### EA — CategoryFilter (BK) — DEC-0110

- **Frozen:** static top-level import on `ForecastPage` (Monthly tab) and `WealthPage` (Overview category card); remove Suspense wrapper around CategoryFilter on those surfaces.
- **Unchanged:** `CategoryTrendChart` lazy + Suspense; other chart lazy boundaries.
- **Optional P2:** `PlanningPage` parity (out of BK).

### EB — Wealth Role column (BL) — DEC-0111

- **Frozen:** `COALESCE(payload->'attributes'->>'account_role', payload->>'account_role')` in `wealth/repository.rs` `load_asset_accounts` (+ test SQL constant).
- **Frozen:** frontend `formatAccountRole` human label map on WealthPage Role column; API returns raw enum; unknown → raw string; null → em dash.
- **Propagation:** snapshots + Grafana portfolio panel heal on next daily upsert — no Grafana JSON edit.

**Next phase:** `/sprint-plan` (role: tech-lead) — materialize quick sprint from architecture § BUG-0021 task tree.
