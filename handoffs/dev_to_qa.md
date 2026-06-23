# Dev → QA handoff — operator omniflow deploy 2026-06-16

**From:** Dev (`/execute` — operator deploy)  
**To:** QA (smoke optional)  
**Date:** 2026-06-16  
**Release:** `0.22.0-us0022` (US-0022 / S0021) + pending BUG-0025 (`bug0025-q0034`) code in tree  
**Verdict:** deploy **COMPLETE**

## Dev environment relaunch

| Field | Value |
|-------|-------|
| `dev_auto_launch_profile` | on (manual deploy — `scripts/dev_environment_lib.py` not shipped) |
| `runtime_mode` | docker-host-local / omniflow-external |
| `relaunch_tier` | B (full image rebuild) |
| `relaunch_command` | `bash /workdir/financegoblin/deploy.sh` |
| `relaunch_outcome` | success |
| `retry_count` | 0 |
| `reason_code` | (none) |

## Connect

| Field | Value |
|-------|-------|
| `runtime_mode` | remote (Traefik) |
| `connect_endpoint` | `https://financegnome.omniflow.cc` |
| `health_path` | `/health` |
| `service_id` | `flow-finance-ai` |
| `container_id` | `financegoblin-flow-finance-ai-1` |
| `target_id` | `omniflow-external` |
| `env_refs` | `TRAEFIK_HOST`, `DATABASE_HOST`, `FIREFLY_BASE_URL` |

## Build metadata (live)

```json
{"build_id":"bc3b959","release_tag":"0.22.0-us0022","build_timestamp":"2026-06-16T21:38:57Z"}
```

## Deploy root

- Host: `/workdir/financegoblin` (`.env`, `deploy.sh`, `docker-compose.build.yml`)
- Source: `/workdir/dev_git/finance_goblin`
- Project: `financegoblin` — containers on `traefik` network

## QA smoke (suggested)

1. `https://financegnome.omniflow.cc/health` → 200
2. Sidebar-footer version stamp + tooltip (AC-1/AC-2)
3. Manual **Sync now** on `/sync` — Stromkosten multi-month (BUG-0025 BW)
4. `/forecast` Income card (prior bugs BZ/CA — if frontend bundle current)

---

# Dev → QA handoff — US-0022 / S0021

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-14  
**Story:** US-0022  
**Sprint:** S0021  
**Orchestrator:** `auto-20260613-bug0025`  
**Verdict:** execute **COMPLETE** — ready for `/qa`

## Summary

Implemented US-0022 deploy version stamp & stale-frontend detection — compile-time build provenance oracle with on-mount stale-bundle detection:

| Task | Status | Deliverable |
|------|--------|-------------|
| B1 | **done** | `meta/mod.rs` — `GET /api/v1/meta/build-info` (public, no auth); `option_env!()` fallback |
| B2 | **done** | `Dockerfile` — 3-stage `ARG`/`ENV`/`LABEL` chain |
| F1 | **done** | `vite.config.ts` — `define` block with `__BUILD_ID__` + `__RELEASE_TAG__` |
| F2 | **done** | `vite-env.d.ts` — TypeScript declarations for build constants |
| F3 | **done** | `AppLayout.tsx` — sidebar-footer stamp + hover tooltip + StaleBanner integration |
| F4 | **done** | `useStaleDetection.ts` — on-mount hook (skip dev, silent fail) |
| F5 | **done** | `StaleBanner.tsx` — non-blocking banner with reload CTA |
| T1 | **done** | `meta_test.rs` — 3 integration tests (shape, no-secrets, fallback) |
| G1 | **done** | Automated gate PASS (see below) |
| R1 | **done** | `docs/user-guides/US-0022.md` |
| V1 | **deferred** | verify-work — operator **BACKEND_FRONTEND_DEPLOY** required |

## Test results (G1)

| Suite | Result |
|-------|--------|
| `cargo test --lib` | **221 passed** / 0 failed |
| `cargo test --test meta_test` | **3 passed** / 0 failed |
| `npm test` | **31 passed** / 0 failed |
| `npm run build` | **PASS** (709 modules, 12.90s) |

## Files changed (US-0022 blast radius)

- `backend/src/meta/mod.rs` (new)
- `backend/src/lib.rs` (mod declaration + build_router merge — public route)
- `backend/Dockerfile` (ARG chain)
- `backend/tests/meta_test.rs` (new)
- `frontend/vite.config.ts` (define block)
- `frontend/src/vite-env.d.ts` (declarations)
- `frontend/src/components/AppLayout.tsx` (stamp + tooltip + StaleBanner)
- `frontend/src/hooks/useStaleDetection.ts` (new)
- `frontend/src/components/StaleBanner.tsx` (new)
- `docs/user-guides/US-0022.md` (new)
- `sprints/S0021/{progress.md,summary.md}`

## QA focus

1. **AC-1/AC-2:** Sidebar-footer stamp visible; hover tooltip shows release tag + build id + build timestamp (UTC).
2. **AC-3:** `GET /api/v1/meta/build-info` returns 200 with `{build_id, release_tag, build_timestamp}`; no secrets; public route (no auth).
3. **AC-4:** Frontend bundle embeds `__BUILD_ID__` at compile time (verify via `npm run build` + inspect bundle).
4. **AC-5:** On-mount stale detection: mismatch → banner + reload CTA; match → no banner; dev mode skipped.
5. **AC-6:** `/health` liveness unchanged (`{status: ok}`); OIDC smoke pass; no env secrets in metadata.

## Key implementation notes

- **Meta route is public** — registered in `build_router` before `api_router` (which has auth middleware). Architecture § US-0022 mandates public route.
- **`option_env!()` fallback** — `BUILD_ID` → `"dev"`, `RELEASE_TAG` → `"dev"`, `BUILD_TIMESTAMP` → `"unknown"`. Never breaks local dev.
- **StaleBanner receives `stale` prop** — hook lives in `AppLayout`; banner is pure presentational.
- **On-mount only (GATE-STALE-1)** — no periodic poll. Sufficient for operator use case.

## Operator gates (V1)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild with `--build-arg BUILD_ID=$(git rev-parse --short HEAD) --build-arg RELEASE_TAG=... --build-arg BUILD_TIMESTAMP=...`
2. Verify stamp visible in sidebar footer; hover tooltip shows metadata.
3. Verify stale banner appears when backend build id differs from frontend build id.
4. OIDC smoke on `/sync` or `/`.

---

# Dev → QA handoff — BUG-0025 / Q0034

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-14  
**Bug:** BUG-0025  
**Sprint:** Q0034 (`/quick`)  
**Orchestrator:** `auto-20260613-bug0025`  
**Verdict:** execute **COMPLETE** — ready for `/qa`

## Summary

Implemented BUG-0025 Firefly Stromkosten mirror lag fix — extends **DEC-0002** with manual 365d lookback and Sync Status UX split:

| Task | Status | Deliverable |
|------|--------|-------------|
| B1 | **done** | `firefly/mod.rs` — `sync_transactions(..., trigger)` + `MANUAL_LOOKBACK_DAYS=365` |
| B2 | **done** | `sync/mod.rs` — `SyncStatusResponse.last_firefly_run` + `latest_firefly_run()` |
| F1 | **done** | `SyncStatusPage.tsx` — Last Firefly sync hero, trigger badge, exchange secondary, DEC-0002 callout |
| D1 | **done** | `runbook.md` — § Backdated Firefly imports (`#backdated-firefly-imports`) |
| T1 | **done** | `bug0025_sync_transaction_window.rs` — 3 integration cases |
| G1 | **done** | Automated gate PASS (see `sprints/quick/Q0034/progress.md`) |
| V1 | **deferred** | verify-work — operator **BACKEND_REBUILD** + **FRONTEND_DEPLOY** required |

## Test results (G1)

| Suite | Result |
|-------|--------|
| `cargo test --lib` | **221 passed** / 0 failed |
| `cargo test --test bug0025_sync_transaction_window` | **3 passed** / 0 failed |
| `npm test` | **31 passed** / 0 failed |
| `npm run build` | **PASS** |

## Files changed (BUG-0025 blast radius)

- `backend/src/firefly/mod.rs`
- `backend/src/sync/mod.rs`
- `backend/tests/bug0025_sync_transaction_window.rs` (new)
- `backend/tests/firefly_integration.rs`
- `backend/tests/firefly_readonly_test.rs`
- `frontend/src/lib/api.ts`
- `frontend/src/pages/SyncStatusPage.tsx`
- `docs/engineering/runbook.md`
- `sprints/quick/Q0034/{progress.md,summary.md}`

## QA focus

1. **BW:** `trigger=manual` uses 365d lookback; scheduled path unchanged (`watermark − 7d`); T1 asserts start param + ingest.
2. **BX:** Sync Status DEC-0002 callout visible; runbook anchor `#backdated-firefly-imports` resolves; cursor-reset SQL documented.
3. **BY:** `GET /api/v1/sync/status` returns `last_firefly_run` distinct from exchange-only `last_run`; hero never shows exchange timestamp as Firefly sync; history `trigger` column unchanged.
4. **GATE-DEC-1:** extends **DEC-0002** — no new DEC record.
5. Re-run `cargo test --lib`, `cargo test --test bug0025_sync_transaction_window`, `npm test`, `npm run build` independently.
6. **V1** requires operator **BACKEND_REBUILD** + **FRONTEND_DEPLOY** before verify-work smoke on `/sync` and `/forecast`.

## Blockers

- **V1 deferred** — runtime BW/BX/BY smoke blocked on **BACKEND_REBUILD** + **FRONTEND_DEPLOY**.

---

# Dev → QA handoff — BUG-0024 / Q0033

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-13  
**Bug:** BUG-0024  
**Sprint:** Q0033 (`/quick`)  
**Orchestrator:** `auto-20260613-bug0024`  
**Verdict:** execute **COMPLETE** — ready for `/qa`

## Summary

Implemented BUG-0024 sole-plan inline delete guidance under **DEC-0082** / **GATE-COPY-1**:

| Task | Status | Deliverable |
|------|--------|-------------|
| H1 | **done** | `planSelector.ts` — `shouldShowSolePlanDeleteHint`, `SOLE_PLAN_DELETE_HINT` |
| F1 | **done** | `PlanningPage.tsx` — inline muted hint below **Delete plan** row |
| T1 | **done** | `planSelector.test.ts` — +7 vitest predicate cases (15 total in suite) |
| G1 | **done** | Automated gate PASS (see `sprints/quick/Q0033/progress.md`) |
| V1 | **deferred** | verify-work — operator **FRONTEND_DEPLOY** required |

## Test results (G1)

| Suite | Result |
|-------|--------|
| `npm test` | **31 passed** / 0 failed (+7 sole-plan hint vs plan-verify baseline 24/24) |
| `npm run build` | **PASS** |

## Files changed (BUG-0024 blast radius)

- `frontend/src/pages/planSelector.ts`
- `frontend/src/pages/planSelector.test.ts`
- `frontend/src/pages/PlanningPage.tsx`
- `sprints/quick/Q0033/{progress.md,summary.md,task.json}`

## QA focus

1. **BS:** `shouldShowSolePlanDeleteHint` true when sole active plan + delete disabled; inline copy *To delete this plan, create another scenario, set it active, then delete this one.*
2. **BR regression:** `resolveDisplayedPlanId` / `isDeleteDisabled` unchanged — multi-plan non-active delete still enabled.
3. **BN regression:** Multi-plan active selection — tooltip only, no inline hint.
4. **GATE-SCOPE-1:** No backend `plans.rs` or DELETE handler changes.
5. Re-run `npm test` and `npm run build` independently.
6. **V1** requires operator **FRONTEND_DEPLOY** before verify-work smoke on `/planning`.

## Blockers

- **V1 deferred** — runtime BR/BS smoke blocked on **FRONTEND_DEPLOY** (frontend rebuild only; no migration).

---

# Dev → QA handoff — BUG-0026 / Q0032

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-13  
**Bug:** BUG-0026  
**Sprint:** Q0032 (`/quick`)  
**Orchestrator:** `auto-20260613-bug0026`  
**Verdict:** execute **COMPLETE** — ready for `/qa`

## Summary

Implemented BUG-0026 forecast monthly Income card mismatch fix under **DEC-0089** / **GATE-MONTH-1** / **GATE-LABEL-1**:

| Task | Status | Deliverable |
|------|--------|-------------|
| H1 | **done** | `forecastSummaryMonth.ts` — `resolveForecastSummaryPoint`, `formatForecastMonthLabel`, `formatForecastSummarySubtitle` |
| F1 | **done** | `ForecastPage.tsx` — resolved summary point + subtitle above card grid |
| T1 | **done** | `forecastSummaryMonth.test.ts` — 7 vitest cases (partial-month trap + edge cases) |
| G1 | **done** | Automated gate PASS (see `sprints/quick/Q0032/progress.md`) |
| V1 | **deferred** | verify-work — operator **FRONTEND_DEPLOY** required |

## Test results (G1)

| Suite | Result |
|-------|--------|
| `npm test` | **24 passed** / 0 failed (+7 forecastSummaryMonth vs plan-verify baseline 17/17) |
| `npm run build` | **PASS** |

## Files changed (BUG-0026 blast radius)

- `frontend/src/pages/forecastSummaryMonth.ts` (new)
- `frontend/src/pages/forecastSummaryMonth.test.ts` (new)
- `frontend/src/pages/ForecastPage.tsx`
- `sprints/quick/Q0032/{progress.md,summary.md,task.json,tasks.md}`

## QA focus

1. **BZ:** `resolveForecastSummaryPoint` skips partial zero-income head — partialMonthTrap resolves July **3266.16** (vitest T1).
2. **CA:** Subtitle **"Forecast for July 2026"** rendered above four cards via `formatForecastSummarySubtitle`.
3. **DEC-0089:** Category filter does not alter `monthlyQuery` key or card data path — cards use unfiltered series.
4. **GATE-SCOPE-1:** No `MonthlyChart.tsx`, backend, or API changes.
5. Re-run `npm test` and `npm run build` independently.
6. **V1** requires operator **FRONTEND_DEPLOY** before verify-work smoke on `/forecast` Monthly account **114**.

## Blockers

- **V1 deferred** — runtime BZ/CA smoke blocked on **FRONTEND_DEPLOY** (frontend rebuild only; no migration).

---

# Dev → QA handoff — US-0021 / S0020

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-13  
**Story:** US-0021  
**Sprint:** S0020  
**Orchestrator:** `auto-20260613-us0021`  
**Verdict:** execute **COMPLETE** — ready for `/qa`

## Summary

Implemented US-0021 subscription transaction explorer under **DEC-0112** / **DEC-0113** / **DEC-0114**:

| Task | Status | Deliverable |
|------|--------|-------------|
| TX1 | **done** | `repository.rs` — SQL search + COUNT + Geldbereich JOIN |
| TX2 | **done** | `transaction_search.rs` — search orchestration + hint pass |
| TX3 | **done** | `api/subscriptions.rs` — GET search + POST preview-group |
| UI1 | **done** | Dual-mode Discover shell (Transactions default) |
| UI2 | **done** | Rich filter bar (category, Geldbereich, date, hints toggle) |
| UI3 | **done** | Tx table + pagination + hint badges |
| UI4 | **done** | Multi-select → preview-group → confirm modal |
| PT1 | **done** | Suggested patterns sub-tab (DEC-0098 frozen) |
| T1 | **done** | `us0021_transaction_search.rs` integration tests |
| T2 | **done** | AC-5 `run_discover` regression test |
| R1 | **done** | `docs/user-guides/US-0021.md` |
| V1 | **deferred** | verify-work — operator **BACKEND_FRONTEND_DEPLOY** required |

## Test results

| Suite | Result |
|-------|--------|
| `cargo test --lib` | **221 passed** / 0 failed (+3 vs plan-verify baseline) |
| `cargo test --test us0021_transaction_search` | **6 passed** / 0 failed |
| `npm test` | **17 passed** / 0 failed |
| `npm run build` | **PASS** |

## Files changed (US-0021 blast radius)

- `backend/src/subscriptions/repository.rs`
- `backend/src/subscriptions/transaction_search.rs` (new)
- `backend/src/subscriptions/types.rs`
- `backend/src/subscriptions/mod.rs`
- `backend/src/api/subscriptions.rs`
- `backend/tests/us0021_transaction_search.rs` (new)
- `frontend/src/lib/api.ts`
- `frontend/src/pages/SubscriptionsPage.tsx`
- `docs/user-guides/US-0021.md` (new)
- `sprints/S0020/{progress.md,summary.md,task.json}`

## QA focus

1. **AC-1:** Transactions mode returns individual paginated rows (not candidates-only).
2. **AC-2:** Category, Geldbereich, date filters push to API params.
3. **AC-3:** Hint badges on filtered subset; account 114 SEPA-Lastschrift fixture after deploy.
4. **AC-4:** Multi-select ≥2 → preview-group → confirm modal → `POST /discover/confirm`.
5. **AC-5:** Suggested patterns tab + `run_discover` unchanged (T2 regression).
6. Re-run `cargo test --lib`, `cargo test --test us0021_transaction_search`, `npm test`, `npm run build`.
7. **V1** requires operator **BACKEND_FRONTEND_DEPLOY** before verify-work OIDC smoke.

## Blockers

- **V1 deferred** — runtime AC-1..AC-6 smoke blocked on **BACKEND_FRONTEND_DEPLOY**.

---

# Dev → QA handoff — BUG-0022 / Q0031

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-13  
**Bug:** BUG-0022  
**Sprint:** Q0031 (`/quick`)  
**Orchestrator:** `auto-20260613-bug0022`  
**Verdict:** execute **COMPLETE** — ready for `/qa`

## Summary

Implemented BUG-0022 plan delete selector regression fix under **DEC-0082** / **DEC-0024** / **DEC-0074**:

| Task | Status | Deliverable |
|------|--------|-------------|
| BM1 | **done** | `PlanningPage.tsx` — `resolveDisplayedPlanId` (selectedPlanId ?? globalActive ?? firstPlan) |
| T1 | **done** | `planSelector.ts` + `planSelector.test.ts` — 8 vitest cases for selector + delete enablement |
| G1 | **done** | Automated gate PASS (see `sprints/quick/Q0031/progress.md`) |
| V1 | **deferred** | verify-work — operator **FRONTEND_DEPLOY** required |
| L1 | **skipped** | Optional P2 dropdown label rename |

## Test results (G1)

- `npm test` → **17 passed / 0 failed** (+8 planSelector vs plan-verify baseline 9/9)
- `npm run build` → **PASS**

## Files changed (BUG-0022 blast radius)

- `frontend/src/pages/PlanningPage.tsx`
- `frontend/src/pages/planSelector.ts` (new)
- `frontend/src/pages/planSelector.test.ts` (new)
- `sprints/quick/Q0031/{progress.md,summary.md,task.json}`

## QA focus

1. **BM:** With 2+ plans and one global active, select non-active plan → Delete plan **enabled** → confirm → plan removed; list refreshes.
2. **BN:** Select active plan → delete **disabled** + tooltip *Set another plan active before deleting the active plan*; API DELETE active → **409** `active_plan_delete_forbidden`.
3. Re-run `npm test` and `npm run build` independently.
4. **V1** requires operator **FRONTEND_DEPLOY** before verify-work smoke on `/planning`.

## Blockers

- **V1 deferred** — runtime BM/BN smoke blocked on **FRONTEND_DEPLOY** (frontend rebuild only; no migration).

---

# Dev → QA handoff — BUG-0023 / Q0030

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-12  
**Bug:** BUG-0023  
**Sprint:** Q0030 (`/quick`)  
**Orchestrator:** `auto-20260612-bug0023`  
**Verdict:** execute **COMPLETE** — ready for `/qa`

## Summary

Implemented BUG-0023 crypto Wealth EUR fix under DEC-0064 / DEC-0080 / DEC-0081 / DEC-0038:

| Task | Status | Deliverable |
|------|--------|-------------|
| BO1 | **done** | `bitunix.rs` equity fallback + unrealized key aliases |
| BO2 | **done** | `code==0` validation + parse-skip `warn!` diagnostic |
| BO3 | **done** | OpenAPI wiremock + 5 unit test cases |
| BP1 | **done** | Migration `017_bug0023_exposure_eur.sql` + `pnl.rs` `entryValue` → `exposure_eur` |
| BP2 | **done** | `wealth/service.rs` `value_eur = market_value_eur.or(exposure_eur)`; subtotal wallet-only |
| BQ1 | **done** | `portfolio/service.rs` baseline capture before `total_return_pct` (order fix) |
| T1 | **done** | `bug0023_crypto_wealth_eur.rs` — BO/BP/BQ + regression (4 cases) |
| G1 | **done** | Automated gate PASS (see `sprints/quick/Q0030/progress.md`) |
| V1 | **deferred** | verify-work — operator **BACKEND_DEPLOY** + **EXCHANGE_SYNC** + **PNL_RECOMPUTE** |

## Test results (G1)

- `cargo test --lib` → **218 passed / 0 failed** (+5 vs plan-verify baseline)
- `cargo test --test bug0023_crypto_wealth_eur` → **4 passed / 0 failed** (SKIP when `DATABASE_URL` unset; QA should re-run with DB)
- `npm run build` → **PASS**

## Files changed (BUG-0023 blast radius)

- `backend/migrations/017_bug0023_exposure_eur.sql` (new)
- `backend/src/exchanges/bitunix.rs`
- `backend/src/exchanges/repository.rs`
- `backend/src/portfolio/pnl.rs`
- `backend/src/portfolio/service.rs`
- `backend/src/wealth/service.rs`
- `backend/tests/bug0023_crypto_wealth_eur.rs` (new)
- `backend/tests/exchanges_portfolio_integration.rs`
- `sprints/quick/Q0030/{progress.md,summary.md,task.json}`

## QA focus

1. **BO:** Verify equity fallback sum includes `crossUnrealizedPNL` + `isolationUnrealizedPNL`; `code!=0` rejects wallet row; parse-skip emits structured warn.
2. **BP:** Linear `exposure_eur` populated from `entryValue`; `market_value_eur` NULL for linear; `holdings_all.value_eur` non-null; `crypto.subtotal_eur` wallet-only (DEC-0064).
3. **BQ:** First priced recompute captures baseline and yields non-null `total_return_pct` (baseline order fix).
4. Re-run integration tests with `DATABASE_URL` for live DB assertions.
5. **V1** operator gates: BACKEND_DEPLOY (migration 017) → EXCHANGE_SYNC (Bitunix) → PNL_RECOMPUTE → smoke on localhost:18080.

## Operator prerequisites (V1)

1. **BACKEND_DEPLOY** — rebuild backend; apply migration `017_bug0023_exposure_eur.sql`
2. **EXCHANGE_SYNC** — Bitunix full/exchange sync
3. **PNL_RECOMPUTE** — post-sync recompute

## Next phase

**`/qa`** in fresh subagent/chat (role: qa).

`fresh_context_marker`: execute-20260612-bug0023-dev-fresh  
`runtime_proof_id`: runtime-proof-execute-20260612-bug0023-001  
`phase_boundary`: execute → qa

---

# Dev → QA handoff — BUG-0021 / Q0029

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-11  
**Bug:** BUG-0021  
**Sprint:** Q0029 (`/quick`)  
**Orchestrator:** `auto-20260611-bug0021`  
**Verdict:** execute **COMPLETE** — ready for `/qa`

## Summary

Implemented DEC-0110 + DEC-0111 for category filter snappy load (**BK**) and wealth Role column (**BL**):

| Task | Status | Deliverable |
|------|--------|-------------|
| EA1 | **done** | `ForecastPage.tsx` — static `CategoryFilter` import; Suspense removed on Monthly tab; `hasForecast` TS6133 removed |
| EA2 | **done** | `WealthPage.tsx` — static `CategoryFilter` import; Suspense removed on Overview card |
| EB1 | **done** | `wealth/repository.rs` — `COALESCE(payload->'attributes'->>'account_role', payload->>'account_role')` |
| EB2 | **done** | `frontend/src/lib/accountRole.ts` + WealthPage Role column `formatAccountRole` |
| EA3 | **done** | `PlanningPage.tsx` — static CategoryFilter parity (P2) |
| T1 | **done** | `bug0021_wealth_account_role.rs` — nested/root/API-shape/categories smoke |
| G1 | **done** | Automated gate PASS (see `sprints/quick/Q0029/progress.md`) |
| V1 | **deferred** | verify-work — operator BACKEND_FRONTEND_DEPLOY + optional SNAPSHOT_UPSERT_OR_SYNC |

## Test results (G1)

- `cargo test --lib` → **213 passed / 0 failed**
- `cargo test --test bug0021_wealth_account_role` → **4 passed / 0 failed** (tests SKIP seed when `DATABASE_URL` unset; QA should re-run with DB for live assertions)
- `npm run build` → **PASS**
- `npm test` → **9 passed / 0 failed**

## Files changed (BUG-0021 blast radius)

- `frontend/src/pages/ForecastPage.tsx`
- `frontend/src/pages/WealthPage.tsx`
- `frontend/src/pages/PlanningPage.tsx`
- `frontend/src/lib/accountRole.ts` (new)
- `backend/src/wealth/repository.rs`
- `backend/tests/bug0021_wealth_account_role.rs` (new)
- `sprints/quick/Q0029/progress.md`

## QA focus

1. Static review: CategoryTrendChart lazy + Suspense unchanged on all three pages.
2. EB1 SQL: COALESCE path in `load_asset_accounts` and test constant (~L133).
3. EB2 label map matches DEC-0111 table; unknown enum → raw string; null → em dash.
4. Re-run `cargo test --test bug0021_wealth_account_role` with `DATABASE_URL` for live nested-payload assertions.
5. V1 remains operator-gated — BK/BL browser + API oracle probes after deploy.

## Next phase

**`/qa`** in fresh subagent/chat (role: qa).

`fresh_context_marker`: execute-20260611-bug0021-dev-fresh  
`runtime_proof_id`: runtime-proof-execute-20260611-bug0021-001  
`phase_boundary`: execute → qa

---

# Dev → QA handoff — BUG-0020 / Q0028

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-11  
**Bug:** BUG-0020  
**Sprint:** Q0028 (`/quick`)  
**Orchestrator:** `auto-20260610-bug0019`  
**Verdict:** execute **COMPLETE** — ready for `/qa`

## Summary

Implemented DEC-0109 two-layer fix for subscription list quality (**BI** duplicates, **BJ** uncategorized display category):

| Task | Status | Deliverable |
|------|--------|-------------|
| DA1 | **done** | Migration 016 YouTube confirmed merge (earliest `confirmed_at` survivor; loser `inactive`; spt/alerts/tags relink) |
| DB1 | **done** | Migration 016 DEC-0100 RANK `display_category_id` backfill (confirmed only) |
| DA2 | **done** | `SubscriptionsPage.tsx` All-tab client filter excludes `rejected` + `inactive` |
| DA3 | **done** | `detection.rs` skip pending INSERT when confirmed merge returns fingerprint conflict (`warn` log) |
| T1 | **done** | `bug0020_subscription_list_quality.rs` — BI/BJ/regression/DA3/idempotency |
| G1 | **done** | Automated gate PASS (see `sprints/quick/Q0028/progress.md`) |
| V1 | **deferred** | verify-work — operator BACKEND_FRONTEND_DEPLOY + MIGRATION_016_APPLY + FULL_FIREFLY_SYNC |

## Test results (G1)

- `cargo test --test bug0020_subscription_list_quality` → **7 passed / 0 failed** (with `DATABASE_URL`)
- `cargo test --test bug0008_subscription_alerts` → **8 passed** (regression)
- `cargo test --test subscriptions_integration` → **1 passed** (regression)

## Files changed

- `backend/migrations/016_bug0020_subscription_list_quality.sql` (new)
- `backend/src/subscriptions/detection.rs`
- `frontend/src/pages/SubscriptionsPage.tsx`
- `backend/tests/bug0020_subscription_list_quality.rs` (new)
- `sprints/quick/Q0028/progress.md`

## QA focus

1. Static review: migration reconcile order (relink before inactive/rejected); `interval_matches` ±3d gate on clusters.
2. DA3: confirmed payee-interval + merge `Ok(false)` → no pending row (integration test `da3_skips_pending_when_confirmed_merge_fingerprint_conflicts`).
3. Blast radius: no `list_patterns` / discover / tag API changes.
4. V1 remains operator-gated — do not expect live BI/BJ oracle until deploy + migration apply.

## Next phase

**`/qa`** in fresh subagent/chat (role: qa).

`fresh_context_marker`: execute-20260611-bug0020-dev-fresh  
`runtime_proof_id`: runtime-proof-execute-20260610-bug0020-001  
`phase_boundary`: execute → qa

---

# Dev → QA handoff — BUG-0019 / Q0027 — FIX CYCLE 2 (re-QA)

**From:** Dev (`/execute`, fix cycle 2 of max 5)  
**To:** QA (`/qa`)  
**Date:** 2026-06-10  
**Bug:** BUG-0019 — Sprint: Q0027 — Orchestrator: `auto-20260610-bug0019`  
**Verdict:** fix cycle 2 **COMPLETE** — all `qa_to_dev.md` return items resolved; ready for QA re-run

## Fix summary (QA return items)

QA FAIL driver was a contract conflict, not an implementation defect: test
`account_id_variable_uses_abs_balance_sort` enforced the DEC-0068 "omit saved
`current`" clause, which DEC-0108 intentionally reverses. Resolution:

1. **Test updated** — `backend/tests/grafana_provisioning_bug0009.rs`
   (L144–147 replacement): asserts `current` is **present** with the DEC-0108
   empty shape (`text == ""`, `value == ""` — no saved concrete value) and
   that the `account_id` variable contains no hardcoded `114`. ABS-balance
   sort + no-alphabetical-only assertions kept unchanged; header doc comment
   notes the amendment.
2. **Supersession recorded** — `decisions/DEC-0108.md` header now carries
   `**Supersedes:** DEC-0068 Y1 clause "Omit current block"…` (existing
   `**Supersedes:**` convention per DEC-0104/DEC-0057);
   `docs/engineering/decisions.md` § DEC-0068 Y1 bullet annotated
   superseded-by-DEC-0108. Note: `decisions/DEC-0068.md` does not exist as a
   standalone file; the canonical DEC-0068 record in
   `docs/engineering/decisions.md` was amended instead.
3. **Tests green** — `cargo test --test grafana_provisioning_bug0009`:
   **6 passed / 0 failed / 0 ignored (6/6 PASS)**. Static guard re-run
   (python json): 12/12 PASS — sort:0 + empty `current` shape in both
   dashboards, versions 2/3/2, no literal `114`, platform-health panel 2
   mirror UNION ALL + `LEFT JOIN sync_cursors`, no bare `records_synced`.

**Not touched:** the three dashboard JSONs (QA verified them correct),
backend/frontend code, migrations. Files changed this cycle:
`backend/tests/grafana_provisioning_bug0009.rs`, `decisions/DEC-0108.md`,
`docs/engineering/decisions.md`, `sprints/quick/Q0027/progress.md`, this file.
Nothing committed. Duplicate-UID provisioning warning remains a pre-existing
V1/operator note (QA classification unchanged).

---

# Dev → QA handoff — BUG-0019 / Q0027 (original execute, cycle 1)

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-10  
**Bug:** BUG-0019  
**Sprint:** Q0027 (`/quick`)  
**Orchestrator:** `auto-20260610-bug0019`  
**Verdict:** execute **COMPLETE** — ready for QA (V1 remains verify-work scope)

## Summary

Q0027 execute delivered the **DEC-0108** provisioning-only Grafana fix in the
frozen order `(CA1→CA2) ∥ CA3 ∥ CB1 → G1`. Exactly **three dashboard JSON
files** edited — no backend, frontend, or migration changes; no hardcoded
account `114`; cursor/`upsert_cursor` semantics untouched. **G1** static guard
all-PASS (python `json` fallback — `jq` not installed). **V1** deferred to
verify-work after operator **GRAFANA_PROVISIONING_RELOAD** + Full sync +
0-new-tx incremental rerun.

## Tasks completed

| ID | Title | Evidence |
|----|-------|----------|
| CA1 | cashflow `$account_id` `sort: 0` + `current` + version 1→2 | `grafana/provisioning/dashboards/analytics/cashflow.json` templating block |
| CA2 | cashflow panels 1–3 latest-success subqueries gain `AND model_kind = 'baseline'` | same file, panels id 1–3 `rawSql` (**4 subqueries** — panel 1 has two targets, A balance + B threshold; asserted per-panel per plan-verify note 2) |
| CA3 | forecast-horizons `$account_id` `sort: 0` + `current` + version 2→3 | `grafana/provisioning/dashboards/analytics/forecast-horizons.json`; no panel `rawSql` changed |
| CB1 | platform-health panel 2 `rawSql` → mirror COUNT(*) UNION ALL over six entities LEFT JOIN `sync_cursors`; version 1→2 | `grafana/provisioning/dashboards/platform-health.json`; `records_synced` column dropped (not relabeled — allowed per DEC-0108) |
| G1 | static guard, results in `sprints/quick/Q0027/progress.md` | all assertion groups **PASS** |
| V1 | verify-work re-provision smoke | **deferred** — blocked on GRAFANA_PROVISIONING_RELOAD + Full sync + 0-new-tx rerun |

## File diffs summary

`git diff --stat` (execute-phase edits only; working tree also carries
pre-existing uncommitted changes from earlier sprints — edits are uncommitted,
diff per plan-verify note 3):

| File | Lines | Change |
|------|-------|--------|
| `analytics/cashflow.json` | 13 (+7/−6) | `sort` 1→0, `current` added, 4× `AND model_kind = 'baseline'`, version 1→2 |
| `analytics/forecast-horizons.json` | 5 (+3/−2) | `sort` 1→0, `current` added, version 2→3 |
| `platform-health.json` | 4 (+2/−2) | panel 2 `rawSql` replaced verbatim per DEC-0108, version 1→2 |

## G1 static guard results (summary)

All PASS — full table in `sprints/quick/Q0027/progress.md`:
`account_id` `sort==0` + `current` non-null in both dashboards; cashflow
panels 1–3 per-panel `model_kind = 'baseline'` (2/2, 1/1, 1/1); platform-health
panel 2 references all six mirror tables + `LEFT JOIN sync_cursors`, no bare
`records_synced`; all three files parse; versions bumped; no `114` literal.

## Implementation notes for QA

1. **`current` shape:** `{ "text": "", "value": "" }` — mirrors the
   `forecast_variant` text/value structure; empty value never matches an
   option, so Grafana with `refresh: 1` resolves to the **first option**
   (highest |balance| per the unchanged SQL `ORDER BY ABS(...) DESC`) on
   dashboard load. This keeps the default deterministic **without hardcoding
   114** (forbidden by DEC-0108).
2. **CA2 count is 4, not 3:** cashflow panel 1 has two targets (balance +
   scarcity threshold), both with the latest-success subquery; both filtered.
   A naive `grep -c` over panels 0–3 returns 4 — expected (plan-verify note 2).
3. **Grafana restart needed** before any runtime check (provisioned dashboards
   load at startup/scan; `version` bump cache-busts).
4. **Local reload sanity check done** (`docker restart finance_goblin-grafana-1`):
   all three files provision-scan with **no parse errors**. However a
   **pre-existing environment issue** surfaced: two provisioning providers
   (`Analytics` and `Flow Finance AI`) scan overlapping paths → duplicate UID
   warnings and `"Not saving new dashboard due to restricted database access"`.
   While duplicates exist, Grafana **refuses to persist re-provisioned
   dashboards**, which can mask the fix at V1. Not introduced by Q0027 (the
   files at HEAD provision the same way) and outside the frozen file boundary —
   flag for operator/V1 (dedupe provider scan paths in Grafana provisioning
   config) and consider a follow-up bug if confirmed.
5. **`jq` not installed** — G1 used python3 `json` equivalents; commands
   reproducible from `progress.md`.

## Frozen-boundary compliance

- No edits to `backend/src/db/mod.rs` `upsert_cursor`, `backend/src/firefly/mod.rs`,
  `AnalyticsEmbedPage.tsx`, or migrations.
- forecast-horizons panel SQL unchanged (`$forecast_variant` per R-0051).
- No embed `var-account_id` forwarding (CA-B deferred).
- Nothing committed to git (no convention in handoff requires it).

## Operator gates (before V1)

1. **GRAFANA_PROVISIONING_RELOAD** — `docker compose restart grafana` (done
   once locally as sanity check; operator must still gate V1 on it).
2. **BH proof sequence** — Full sync (transactions = 922) then 0-new-tx
   incremental rerun; panel 2 must hold the mirror count both times.
3. Verify kiosk embed **and** direct Grafana URL; OIDC omniflow re-check.

`fresh_context_marker`: execute-20260610-bug0019-dev-fresh  
`runtime_proof_id`: runtime-proof-execute-20260610-bug0019-001  
`phase_boundary`: execute → qa

---

# Dev → QA handoff — BUG-0018 / Q0026

**From:** Dev (`/execute`)  
**To:** QA (`/qa`)  
**Date:** 2026-06-10  
**Bug:** BUG-0018  
**Sprint:** Q0026 (`/quick`)  
**Orchestrator:** `intake-20260609-ui-audit`  
**Verdict:** execute **COMPLETE** — ready for QA

## Summary

Q0026 execute delivered **DEC-0107** scarcity SQL qualification (**BE1**) — qualify `fbd.balance` and `fbd.ts` in `evaluate_scarcity` daily aggregate query. **T1** regression gate verified via `wealth_alerts_integration` (static + module tests PASS; live DB path skipped without `DATABASE_URL`). **V1** operator runtime smoke deferred to verify-work after **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**.

## Tasks completed

| ID | Title | Evidence |
|----|-------|----------|
| BE1 | Qualify `fbd.balance` + `fbd.ts` in `evaluate_scarcity` | `backend/src/alerts/evaluate.rs` L21–31 |
| T1 | `wealth_alerts_integration` regression gate | `backend/tests/wealth_alerts_integration.rs` |
| V1 | verify-work sync + alerts smoke | **deferred** — blocked on deploy + Full sync gates |

## Test results

| Check | Result |
|-------|--------|
| `cargo test --lib` | **213/213 PASS** |
| `cargo test --test wealth_alerts_integration` | **3/3 PASS** (scarcity integration skipped — `DATABASE_URL` unset) |

## Decision alignment notes

- **DEC-0107:** `SELECT fbd.ts::date AS day, SUM(fbd.balance::float8) AS balance`; date bounds and GROUP BY use `fbd.ts::date`; aggregation reads forecast path only — not `a.balance`.
- **R-0024:** Sync warn-only semantics unchanged — no sync fail-on-alert-error.
- **Frozen boundaries:** No migration; no frontend change; no sibling evaluator edits; subscription dedup regression is V1-only per BUG-0008.

## Operator gates (deferred live smoke)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild `flow-finance-ai` with BE1 `evaluate_scarcity` SQL fix
2. **FULL_FIREFLY_SYNC** — trigger sync; alerts phase must complete without 42702 / `alert evaluation failed`

## QA focus

- Acceptance rows **BE**, **BF** trace vs `sprints/quick/Q0026/tasks.md` and `docs/product/acceptance.md`
- **DEC-0107** contract review — qualified columns in SELECT, WHERE, GROUP BY only
- Confirm no unqualified `balance`/`ts` or `a.balance` in `evaluate_scarcity`
- T1: recommend `DATABASE_URL=… cargo test --test wealth_alerts_integration` when DB available
- V1 runtime probes (sync logs, `GET /api/v1/alerts`, header bell, subscription dedup) deferred to verify-work

## Artifacts

- `sprints/quick/Q0026/{progress.md,summary.md,tasks.md,sprint.json}`
- `handoffs/plan_verify_to_execute.md`
- `decisions/DEC-0107.md`

`fresh_context_marker`: execute-20260610-bug0018-dev-fresh  
`runtime_proof_id`: runtime-proof-execute-20260610-bug0018-001  
`phase_boundary`: execute → qa

**Next:** `/qa` in fresh subagent/chat (role: qa). Do not begin QA in this subagent.
