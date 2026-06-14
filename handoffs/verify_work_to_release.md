# Verify-work → Release handoff

**Story:** US-0021  
**Sprint:** S0020  
**Verify-work verdict:** **PASS-WITH-PREREQUISITES** (2026-06-13T08:10:01Z)  
**Orchestrator:** `auto-20260613-us0021`  
**Decisions:** DEC-0112, DEC-0113, DEC-0114  
**Next phase:** `/release`

## UAT summary

- **Verdict:** PASS-WITH-PREREQUISITES — 1/6 steps pass, 5 pass-with-prerequisites, 0 fail
- **Automated:** us0021 6/6; cargo lib 221/221; npm 17/17; build PASS
- **Runtime:** discover API 200 (account 114); tx-search 404 + /subscriptions 404 pre-deploy
- **Blocking:** none

## Acceptance row verdicts

| Row | Verdict | Notes |
|-----|---------|-------|
| **AC-1** | pass_with_prerequisites | Integration + UI code PASS; live tx-search 404 pre-deploy |
| **AC-2** | pass_with_prerequisites | Rich filters in code + integration PASS; live blocked |
| **AC-3** | pass_with_prerequisites | Hint test + badge UI PASS; account 114 live deferred |
| **AC-4** | pass_with_prerequisites | preview-group + multi-select UI PASS; live blocked |
| **AC-5** | pass | `run_discover` regression test PASS; patterns tab preserved |
| **AC-6** | pass_with_prerequisites | Health + discover API 200; OIDC browser deferred deploy |

## Deliverables verified

| Task | Status | Evidence |
|------|--------|----------|
| TX1–TX3 — tx-search API | PASS | `repository.rs`, `transaction_search.rs`, `subscriptions.rs` routes |
| UI1–UI4 — dual-mode Discover | PASS | `SubscriptionsPage.tsx` Transactions default + filters + multi-select |
| PT1 — patterns sub-tab | PASS | DEC-0098 frozen discover path |
| T1/T2 — integration + regression | PASS | us0021 6/6; AC-5 `reg_discover_candidate_pass_unchanged_ac5` |
| R1 — user guide | PASS | `docs/user-guides/US-0021.md` |
| V1 — operator smoke | pass_with_prerequisites | BACKEND_FRONTEND_DEPLOY deferred |

## Operator notes (release)

1. Rebuild + restart `flow-finance-ai` (**BACKEND_FRONTEND_DEPLOY**) to ship S0020 tx-search API + dual-mode Discover UI
2. After deploy: `/subscriptions` Discover → Transactions mode — account 114 SEPA-Lastschrift 11 txs @ 31d/95% with hint badges
3. Post-deploy OIDC smoke on external profile (`financegnome.omniflow.cc`)
4. Release notes: subscription transaction explorer (DEC-0112/0113/0114); dual-mode Discover; rich filters; manual tx-group activate

## Artifacts

- `sprints/S0020/uat.json`
- `sprints/S0020/uat.md`
- `sprints/S0020/verify-work-findings.md`
- `sprints/S0020/qa-findings.md`
- `decisions/DEC-0112.md`, `DEC-0113.md`, `DEC-0114.md`
- `docs/product/acceptance.md` (US-0021 rows AC-1..AC-6)

## Release checklist

1. Check `docs/product/acceptance.md` US-0021 rows **AC-1**..**AC-6**
2. Include **BACKEND_FRONTEND_DEPLOY** prerequisite in release notes
3. Document dual-mode Discover UX and tx-search API contract

`fresh_context_marker`: verify-work-20260613-us0021-qa-fresh  
`runtime_proof_id`: runtime-proof-verify-work-20260613-us0021-001  
`phase_boundary`: verify-work → release

**Next:** `/release` (role: release) in fresh subagent/chat.

---

# Verify-work → Release handoff

**Bug:** BUG-0021  
**Quick task:** Q0029  
**Verify-work verdict:** **PASS-WITH-PREREQUISITES** (2026-06-11T12:50:00Z)  
**Orchestrator:** `auto-20260611-bug0021`  
**Decisions:** DEC-0110, DEC-0111  
**Next phase:** `/release`

## UAT summary

- **Verdict:** PASS-WITH-PREREQUISITES — 1/7 steps pass, 6 pass-with-prerequisites, 0 fail
- **Automated:** bug0021 4/4; cargo lib 213/213; npm 9/9; build PASS; wealth_alerts 3/3
- **Runtime:** mirror COALESCE 3/3 PASS; live API/UI/snapshot null pre-deploy; omniflow wealth API HTTP 200
- **Blocking:** none

## Acceptance row verdicts

| Row | Verdict | Notes |
|-----|---------|-------|
| **BK** | pass_with_prerequisites | Static CategoryFilter + chunk audit PASS; browser ≤1 s deferred deploy |
| **BL** | pass_with_prerequisites | COALESCE SQL + label map + mirror probe PASS; API/UI/snapshot deferred deploy |

## Deliverables verified

| Task | Status | Evidence |
|------|--------|----------|
| EA1 — ForecastPage static CategoryFilter | PASS | Static import; Monthly tab no Suspense wrapper |
| EA2 — WealthPage static CategoryFilter | PASS | Static import; Overview no Suspense wrapper |
| EA3 — PlanningPage parity | PASS | Static import P2 parity |
| EB1 — COALESCE account_role SQL | PASS | `repository.rs` + mirror probe 3/3 |
| EB2 — formatAccountRole label map | PASS | `accountRole.ts` five canonical labels |
| T1/G1 — integration + automated gate | PASS | bug0021 4/4; lib 213/213; npm 9/9 |
| V1 — verify-work operator smoke | pass_with_prerequisites | Deploy deferred; DB oracle PASS |

## Operator notes (release)

1. Rebuild + restart `flow-finance-ai` (**BACKEND_FRONTEND_DEPLOY**) to ship Q0029 EA/EB changes
2. After deploy: confirm `GET /api/v1/wealth` returns non-null `account_role` for Giro/savings/cash wallet
3. Optional: trigger Full sync or wait for daily snapshot upsert (**SNAPSHOT_UPSERT_OR_SYNC**) before BL-SNAPSHOT/BL-GRAFANA oracle
4. Release notes: CategoryFilter loads synchronously on Forecast/Wealth (DEC-0110); Role column shows Firefly account type (DEC-0111)

## Artifacts

- `sprints/quick/Q0029/uat.json`
- `sprints/quick/Q0029/uat.md`
- `sprints/quick/Q0029/qa-findings.md`
- `handoffs/verify_work_report.md`
- `handoffs/qa_to_verify_work.md`
- `decisions/DEC-0110.md`, `decisions/DEC-0111.md`
- `docs/product/acceptance.md` (BUG-0021 rows BK, BL)

## Release checklist

1. Check `docs/product/acceptance.md` BUG-0021 rows **BK**, **BL**
2. Include deploy prerequisite in release notes
3. Document CategoryFilter static import + account_role COALESCE behavior

`fresh_context_marker`: verify-work-20260611-bug0021-qa-fresh  
`runtime_proof_id`: runtime-proof-verify-work-20260611-bug0021-001  
`phase_boundary`: verify-work → release

**Next:** `/release` (role: release) in fresh subagent/chat.
