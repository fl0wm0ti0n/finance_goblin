# QA Findings — Quick Q0013 / BUG-0010

**Work item:** BUG-0010 (defect)  
**Quick task:** Q0013  
**QA phase:** `/qa`  
**Date:** 2026-06-05  
**Orchestrator:** `auto-20260605-bug0010-001`  
**Verdict:** **PASS** (ready for `/verify-work`; V1 operator deploy + Full Firefly sync deferred)

## Scope

Forecast wrong numbers, empty wealth, misleading ML skip per `architecture-20260605-bug0010` (`handoffs/tl_to_dev.md`, `docs/engineering/architecture.md` § BUG-0010):

- **AA1** — `balance_ingest` structured logs on account sync; parse-failure warn (DEC-0060)
- **AB1** — Remove `>= 0` wealth filter; `is_overdrawn`; signed `firefly.subtotal_eur` (DEC-0065)
- **AC1** — `record_skip_on_baseline(Disabled)` when ML off; meta derive `sidecar_disabled` (DEC-0066)
- **AA3** — `balance_warnings` in baseline metadata + meta API; ForecastPage negative-start banner
- **AB2** — WealthPage zero-total callout with Full Firefly sync + reconcile guidance
- **AC2** — ForecastPage three-state ML copy (available / not enabled / skipped / baseline-only)
- **V1** — Omniflow probes after deploy + manual Full Firefly sync — **DEFERRED**

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0013/summary.md`, `sprints/quick/Q0013/plan-verify.json`, `sprints/quick/Q0013/plan-verify.md`, `docs/product/acceptance.md` (BUG-0010 rows AA/AB/AC), `docs/engineering/architecture.md` (§ BUG-0010), `decisions/DEC-0065.md`, `decisions/DEC-0066.md`, `backend/src/firefly/mod.rs`, `backend/src/wealth/repository.rs`, `backend/src/wealth/types.rs`, `backend/src/wealth/service.rs`, `backend/src/sync/mod.rs`, `backend/src/forecast/service.rs`, `backend/src/forecast/repository.rs`, `backend/src/api/forecast.rs`, `backend/src/forecast_ml/service.rs`, `frontend/src/pages/WealthPage.tsx`, `frontend/src/pages/ForecastPage.tsx`, `frontend/src/lib/api.ts`, `backend/config/default.toml`, `sprints/quick/Q0013/uat.md`, `sprints/quick/Q0013/progress.md`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Backend unit tests | `cd backend && cargo test --lib` | **PASS** (131/131) |
| T-2 | Frontend unit tests | `cd frontend && npm test` | **PASS** (2/2) |
| T-3 | Frontend build | `cd frontend && npm run build` | **PASS** |
| T-4 | AA1 balance_ingest logs | Static review + `account_current_balance_*` tests | **PASS** |
| T-5 | AA1 parse failure → NULL upsert | `account_current_balance_parse_failure_returns_none` | **PASS** |
| T-6 | AB1 negative asset rows included | SQL review + `load_asset_accounts_includes_negative_balances` | **PASS** |
| T-7 | AB1 is_overdrawn + signed subtotal | Static review + `overdrawn_flag_when_balance_negative` | **PASS** |
| T-8 | AC1 sidecar_disabled persist | `sync/mod.rs` Disabled path + `record_skip_on_baseline` | **PASS** |
| T-9 | AC1 meta derive fallback | `derive_sidecar_disabled_when_ml_off_and_metadata_null` | **PASS** |
| T-10 | AC1 enabled path unchanged | `derive_none_when_ml_enabled_and_metadata_null` | **PASS** |
| T-11 | AA3 balance_warnings meta | `balance_warning_*` + `build_metadata_includes_balance_warnings` | **PASS** |
| T-12 | AA3 zero balance + no txs → no warning | `balance_warning_absent_for_zero_balance_without_txs` | **PASS** |
| T-13 | AA3 ForecastPage banner | Static review `ForecastPage.tsx` L198–205 | **PASS** |
| T-14 | AB2 zero-total callout | Static review `WealthPage.tsx` L86–100 | **PASS** |
| T-15 | AC2 three-state ML copy | Static review `ForecastPage.tsx` L47–75 | **PASS** |
| T-16 | Frozen boundaries | No tx-sum balance recompute; ML default off; no BUG-0009/0011 merge | **PASS** |
| T-17 | Rows AA/AB/AC live smoke | Omniflow deploy + Full Firefly sync | **DEFERRED** — verify-work (V1) |
| T-18 | Regression footer (OIDC + bundled-firefly) | Operator smoke per acceptance | **DEFERRED** — verify-work (plan-verify ADV-1) |

### Environment dependencies (non-blocking)

- **Operator deploy:** Q0013 backend + frontend image to omniflow before live acceptance rows AA/AB/AC.
- **FULL_FIREFLY_SYNC gate:** Manual Full Firefly sync after deploy to backfill mirror balances (DEC-0002; AA2 operator gate).
- **Forecast recompute:** Confirm `/api/v1/forecast/meta` `computed_at` updates post-sync before V1 probes.

## Acceptance criteria matrix (BUG-0010)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **(AA)** | Plausible signed forecast; not silent -25365 without warning; series after recompute | **PASS** (code) / **DEFERRED** (runtime) | AA1: `balance_ingest` logs + DEC-0060 parse unchanged; AA3: `balance_warning_entry` + meta API + UI banner. Live mirror backfill + forecast probes **DEFERRED** until operator Full sync |
| **(AB)** | Non-empty wealth breakdown; synced asset accounts visible; honest `total_eur` | **PASS** (code) / **DEFERRED** (runtime) | AB1: `>= 0` filter removed; `is_overdrawn`; signed sum; AB2: zero-total callout. Live acct 114 + wealth/history probes **DEFERRED** until Full sync |
| **(AC)** | Honest ML posture when disabled; not-enabled vs skipped distinguished | **PASS** (code) / **DEFERRED** (runtime) | AC1: `record_skip_on_baseline(Disabled)` + `derive_ml_skipped_reason`; AC2: three-state UI; `forecast_ml.enabled = false` in default.toml. Live meta/UI confirm **DEFERRED** until deploy |
| Regression | OIDC + bundled-firefly deploy checks | **PASS** (unit) / **DEFERRED** (live) | No ML enablement change; frozen boundaries intact; full OIDC smoke at verify-work |

**Summary:** AA1–AC2 **PASS** on static/automated path; full acceptance runtime deferred with `OPERATOR_DEPLOY_PENDING` + `OPERATOR_FULL_FIREFLY_SYNC_PENDING`.

## Architecture compliance

### AA1 — Balance mirror ingest

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Canonical field | `attributes.current_balance` via `parse_split_amount` | `sync_accounts` L255–256 | PASS |
| Structured log | `balance_ingest` with firefly_id, name, raw, parsed, role | `tracing::info!` L259–266 | PASS |
| Parse failure | Warn, upsert NULL balance | `tracing::warn!` L268–276 + unit test | PASS |
| No tx recompute | Trust Firefly mirror only | No transaction-sum balance logic added | PASS |

### AB1 — Wealth negative visibility (DEC-0065)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| SQL filter | Remove `>= 0`; keep active + include_net_worth | `wealth/repository.rs` L33–36 | PASS |
| `is_overdrawn` | `true` when `balance < 0` | `wealth/service.rs` L98 + types | PASS |
| Signed subtotal | Sum of included balances | `firefly_subtotal` L78 | PASS |
| UI styling | Overdrawn row highlight | `WealthPage.tsx` L174–189 | PASS |

### AC1 — ML disabled metadata (DEC-0066)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Disabled path | `record_skip_on_baseline(Disabled)` after baseline | `sync/mod.rs` L314–318 | PASS |
| Persist reason | `sidecar_disabled` in baseline metadata | `forecast_ml/service.rs` L288–297 | PASS |
| Meta derive | Fallback when ML off and metadata null | `derive_ml_skipped_reason` L603–617 | PASS |
| Enabled path | No false derive when ML on | Unit test L635–637 | PASS |

### AA3 — Negative starting balance warning

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Warning condition | `balance <= 0` AND tx count > 0 | `balance_warning_entry` L626–634 | PASS |
| Zero + no txs | No warning | Unit test L649–650 | PASS |
| Meta cache | Stored in baseline metadata | `build_metadata` L608–615 | PASS |
| UI banner | ForecastPage warning card | L198–205 | PASS |

### AB2 / AC2 — UI guidance

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Zero-total callout | Full sync + reconcile guidance | `WealthPage.tsx` L86–100 | PASS |
| ML not enabled | `sidecar_disabled` → not-enabled copy | `ForecastPage.tsx` L53–68 | PASS |
| ML skipped other | Shows reason, not default unavailable | L69–72 | PASS |
| No false skip | Null reason + ML off → baseline-only, not "ML skipped" | L73–74 | PASS |

### Frozen boundaries

| Boundary | Verdict |
|----------|---------|
| No transaction-sum balance recompute | PASS |
| No ML default enable on external profile (`forecast_ml.enabled = false`) | PASS |
| AC3 / US-0013 out of scope | PASS |
| No merge with BUG-0009/0011 | PASS |
| DEC-0060 parse regression preserved | PASS |

## Plan-verify alignment

Plan-verify PASS (2026-06-06): 3/3 acceptance rows AA/AB/AC covered; 7/7 tasks mapped; 0 gaps; 6 low advisories. QA confirms all six code tasks (AA1, AB1, AC1, AA3, AB2, AC2) implemented per frozen architecture contracts. V1 correctly deferred to verify-work per operator gate.

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

## Advisories (carry-forward from plan-verify)

| ID | Note |
|----|------|
| ADV-1 | OIDC + bundled-firefly regression footer — operator verify-work smoke post-V1 |
| ADV-2 | Rows AA/AB blocked until operator manual Full Firefly sync backfills mirror balances |
| ADV-3 | AB3 snapshot re-verify via `wealth/history` consolidated into V1 |
| ADV-4 | AC3 (ML production on omniflow) → US-0013 epic; BUG-0010 AC closes honest degraded messaging only |
| ADV-5 | Legitimate Firefly overdraft shows signed truth + warnings — not invented positive balance |
| ADV-6 | Parallel open bugs (0007–0009, 0011) must not merge into Q0013 scope |

## Next phase

**`/verify-work`** — after operator deploys AA1–AC2 and runs manual Full Firefly sync:

1. `GET /api/v1/forecast/long-term?account_id=114&horizon=3` — plausible end OR negative-start warning
2. `GET /api/v1/forecast/meta` — `balance_warnings` + `ml_skipped_reason: sidecar_disabled`
3. `GET /api/v1/wealth` — acct 114 present; `is_overdrawn`; honest signed `total_eur`
4. `GET /api/v1/wealth/history?days=30` — post-sync snapshot totals (AB3)
5. `/forecast` UI — not-enabled copy; negative-start banner when applicable
6. Regression footer: OIDC + bundled-firefly

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
