# QA Findings — Quick Q0029 / BUG-0021

**Work item:** BUG-0021 (defect)  
**Quick task:** Q0029  
**QA phase:** `/qa`  
**Date:** 2026-06-11  
**Orchestrator:** `auto-20260611-bug0021`  
**Decisions:** DEC-0110, DEC-0111  
**QA agent:** fresh subagent (`qa-20260611-bug0021-qa-fresh`)

## Verdict

**PASS** — Independent re-run confirms DEC-0110/DEC-0111 implementation: static `CategoryFilter` on Forecast Monthly, Wealth Overview, and Planning parity (EA3); `COALESCE(attributes, root)` SQL in `load_asset_accounts`; `formatAccountRole` label map matches canonical table; automated gates **213/213** lib, **9/9** npm, **PASS** build. Live `:18080` API still returns `account_role: null` (pre-deploy backend) — BL runtime oracles correctly deferred to verify-work after **BACKEND_FRONTEND_DEPLOY**.

## Scope

**Inputs reviewed:** `handoffs/dev_to_qa.md` (Q0029 top section), `sprints/quick/Q0029/progress.md`, `decisions/DEC-0110.md`, `decisions/DEC-0111.md`, `docs/product/acceptance.md` BUG-0021 row, `frontend/src/pages/{ForecastPage,WealthPage,PlanningPage}.tsx`, `frontend/src/lib/accountRole.ts`, `backend/src/wealth/repository.rs`, `backend/tests/bug0021_wealth_account_role.rs`. Read-only DB probes on `flow_finance_ai` postgres; `:18080` wealth API probe. No host `.env`/secret files read.

## Test plan

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | Lib unit suite | `cargo test --lib` | **PASS** — 213/213 (1.51s) |
| T-2 | BUG-0021 integration suite | `DATABASE_URL=… cargo test --test bug0021_wealth_account_role` | **PASS** — 4/4 exit 0; **live seed path skipped** — `sqlx::migrate!` fails checksum on migration 015 (`migration 15 was previously applied but has been modified`); misleading SKIP log when migrate fails |
| T-3 | EB1 SQL shape unit test | `cargo test load_asset_accounts_includes_negative_balances --lib` | **PASS** — COALESCE + attributes path asserted |
| T-4 | Frontend unit suite | `npm test` | **PASS** — 9/9 |
| T-5 | Frontend build | `npm run build` | **PASS** — no TS6133; `CategoryTrendChart` remains separate lazy chunk; no `CategoryFilter` lazy chunk |
| T-6 | DEC-0110 EA1 ForecastPage | Static import L4; Monthly tab L273–277 no Suspense wrapper | **PASS** — `CategoryTrendChart` lazy+Suspense unchanged L339–341 |
| T-7 | DEC-0110 EA2 WealthPage | Static import L4; Overview CategoryFilter L178 no Suspense | **PASS** — `CategoryTrendChart` lazy+Suspense unchanged L181–183 |
| T-8 | DEC-0110 EA3 PlanningPage | Static import L17; CategoryFilter L852 no Suspense | **PASS** — P2 parity delivered |
| T-9 | DEC-0111 EB1 SQL | `repository.rs` L31–34 + test constant L136–139 | **PASS** — `COALESCE(payload->'attributes'->>'account_role', payload->>'account_role')` |
| T-10 | DEC-0111 EB2 labels | `accountRole.ts` map + `WealthPage` L217 | **PASS** — five canonical labels; unknown → raw; null → em dash |
| T-11 | Live mirror COALESCE probe | Read-only SQL on 3 asset accounts | **PASS** — `effective_role` = `cashWalletAsset` / `defaultAsset` / `savingAsset`; root path null |
| T-12 | Live API pre-deploy baseline | `GET :18080/api/v1/wealth` | **EXPECTED** — `account_role: null` on all rows (deployed image lacks EB1) |
| T-13 | V1 operator smoke | BACKEND_FRONTEND_DEPLOY + optional SNAPSHOT_UPSERT_OR_SYNC | **DEFERRED** — verify-work |

### T-1 output

```
test result: ok. 213 passed; 0 failed; 0 ignored; 0 measured
```

### T-2 note

With `DATABASE_URL` set, all four integration tests print `SKIP: DATABASE_URL not set…` because `setup_db()` treats migrate failure as None. Environment has `_sqlx_migrations` max version **15** with checksum drift on 015. Code path verified via T-3, T-9, T-11 instead. Recommend operator reconcile migration checksums before relying on integration seed path in CI.

## Acceptance row status (qa-stage)

| Row | qa-stage evidence | Status |
|-----|-------------------|--------|
| **BK** | Static CategoryFilter on Forecast Monthly + Wealth Overview; no `Loading category filter…` string in repo; CategoryTrendChart lazy boundary preserved; build chunk audit | **PASS** at qa — browser ≤1 s interactive deferred V1 |
| **BL** | COALESCE SQL + `formatAccountRole` + lib SQL assertion + live mirror effective_role probe | **PASS** at qa — live API/UI/snapshot deferred V1 (pre-deploy null API) |

## Non-blocking notes (carry to verify-work)

- `bug0021_wealth_account_role` integration seeds did not execute in this environment due to migration 015 checksum mismatch — not a code defect; document for operator/CI.
- New files `accountRole.ts`, `bug0021_wealth_account_role.rs` are **untracked** (sprint policy: uncommitted).
- Deployed `:18080` stack returns null `account_role` until **BACKEND_FRONTEND_DEPLOY** ships EB1.

## Handoff

- **Next phase:** `/verify-work` (role: qa) — see `handoffs/qa_to_verify_work.md`
- **No return items**

`fresh_context_marker`: qa-20260611-bug0021-qa-fresh  
`runtime_proof_id`: runtime-proof-qa-20260611-bug0021-001  
`phase_boundary`: qa → verify-work
