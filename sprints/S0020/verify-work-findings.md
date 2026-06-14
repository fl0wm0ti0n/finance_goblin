# Verify-work findings — Sprint S0020 / US-0021

**Work item:** US-0021 (story)  
**Sprint:** S0020  
**Phase:** `/verify-work`  
**Date:** 2026-06-13  
**Orchestrator:** `auto-20260613-us0021`  
**Decisions:** DEC-0112, DEC-0113, DEC-0114 (extends DEC-0098, DEC-0099, DEC-0111)  
**QA agent:** fresh subagent (`verify-work-20260613-us0021-qa-fresh`)

## Verdict

**PASS-WITH-PREREQUISITES** — Independent re-run confirms US-0021 implementation under frozen gates **DEC-0112** (tx-search API), **DEC-0113** (dual-mode Discover), **DEC-0114** (hint pass boundary): `GET /api/v1/subscriptions/transactions/search`, `POST /transactions/preview-group`, dual-mode UI (Transactions default | Suggested patterns), rich filters, hint badges, multi-select activate flow. Automated gates **221/221** cargo lib, **6/6** us0021 integration, **17/17** npm, **PASS** build; **AC-5** `run_discover` regression **pass**. Live environment returns **404** on new tx-search route and `/subscriptions` SPA — **BACKEND_FRONTEND_DEPLOY** not yet applied. **0 blockers.** Ready for **`/release`**.

## Scope

**Inputs reviewed:** `handoffs/dev_to_qa.md` (S0020 section), `sprints/S0020/qa-findings.md` (PASS — 0 blockers), `sprints/S0020/summary.md`, `docs/product/acceptance.md` § US-0021 AC-1..AC-6, `backend/tests/us0021_transaction_search.rs`, `backend/src/api/subscriptions.rs`, `frontend/src/pages/SubscriptionsPage.tsx`, `frontend/src/lib/api.ts`, `docs/user-guides/US-0021.md`. No host `.env`/secret files read.

**QA baseline:** QA PASS (2026-06-13T10:30:00Z) — AC-1..AC-5 code+test verified; V1 deferred. Verify-work adds live pre-deploy probes and UAT population.

## Operator gates

| Gate | Status | Action | Notes |
|------|--------|--------|-------|
| **BACKEND_FRONTEND_DEPLOY** | pending | Rebuild backend + frontend — new routes + dual-mode Discover UI | Required before live AC-1..AC-6 UI/API probes and OIDC smoke |

**Post-gate smoke:** `/subscriptions` Discover → Transactions mode (default) — account 114 SEPA-Lastschrift: individual tx rows paginated, hint badge 31d/95%, rich filters, multi-select ≥2 → preview-group → confirm; Suggested patterns tab unchanged; OIDC external profile (`financegnome.omniflow.cc`).

## Live probe — pre-deploy baseline (2026-06-13)

| Probe | HTTP | Key fields | Interpretation |
|-------|------|------------|----------------|
| `GET /health` | 200 | OK | Stack reachable |
| `GET /api/v1/subscriptions/discover?account_id=114` | 200 | 11-tx SEPA-Lastschrift candidate @ 31d/95% | Legacy discover endpoint live (AC-5 baseline) |
| `GET /api/v1/subscriptions/transactions/search?account_id=114&limit=5` | **404** | — | **New DEC-0112 route not deployed** |
| `GET /subscriptions` | **404** | — | Pre-deploy SPA shell — dual-mode UI not live |

### Discover API snapshot (pre-deploy, account 114)

| Field | Value |
|-------|-------|
| Top candidate payee | SEPA-Lastschrift R759090719… |
| Interval | 31d |
| Confidence | 95% |
| Transaction count | **11** |
| Account | 114 |

Live baseline confirms operator **BACKEND_FRONTEND_DEPLOY** not yet applied; consistent with execute V1 deferral and Q0031/Q0029 pass-with-prerequisites precedent.

## Per-row verdict (acceptance AC-1..AC-6)

| Row | Verdict | Evidence |
|-----|---------|----------|
| **AC-1** | **pass_with_prerequisites** | `tx_search_sql_filters_and_pagination_meta` PASS; `SEARCH_PAGE_LIMIT` 100/page; UI Transactions mode + pagination in code; live tx-search 404 pre-deploy |
| **AC-2** | **pass_with_prerequisites** | CategoryFilter, Geldbereich, date range, payee, account filters in UI + API query params; integration filter assertions PASS; live blocked |
| **AC-3** | **pass_with_prerequisites** | `tx_search_hint_attachment_without_pending_emit` PASS; hint badge UI L766–776; account 114 fixture deferred post-deploy |
| **AC-4** | **pass_with_prerequisites** | `preview_group_*` tests PASS; multi-select ≥2 → `txPreviewMutation` → confirm modal in UI; live blocked |
| **AC-5** | **pass** | `reg_discover_candidate_pass_unchanged_ac5` PASS; Suggested patterns sub-tab preserved (DEC-0098); tags/majority unchanged in list tabs |
| **AC-6** | **pass_with_prerequisites** | `/health` 200; discover API 200; OIDC browser smoke on tx search + confirm deferred BACKEND_FRONTEND_DEPLOY |

## Automated checks (verify-work re-run)

| Check | Result |
|-------|--------|
| `cargo test --lib` | **221/221 PASS** (2.06s) |
| `cargo test --test us0021_transaction_search` | **6/6 PASS** — search, hints, preview-group, AC-5 regression |
| `npm test` | **17/17 PASS** (2.35s) |
| `npm run build` | **PASS** — tsc + vite build (12.55s) |

### us0021 integration test matrix

| Test | AC | Result |
|------|-----|--------|
| `tx_search_sql_filters_and_pagination_meta` | AC-1, AC-2 | PASS |
| `tx_search_hint_attachment_without_pending_emit` | AC-3 | PASS |
| `preview_group_fingerprint_helper_stable` | AC-4 | PASS |
| `preview_group_median_interval_computation` | AC-4 | PASS |
| `reg_discover_candidate_pass_unchanged_ac5` | AC-5 | PASS |
| `hint_scan_cap_constant_documented` | AC-3 | PASS |

## UAT step matrix

| Step | Row | Result | Evidence |
|------|-----|--------|----------|
| AC1-TX-SEARCH | AC-1 | pass_with_prerequisites | Integration + UI code PASS; live tx-search 404 |
| AC2-FILTERS | AC-2 | pass_with_prerequisites | Filter push-down in code + integration PASS; live blocked |
| AC3-HINT | AC-3 | pass_with_prerequisites | Hint test + badge UI PASS; account 114 live deferred |
| AC4-CONFIRM | AC-4 | pass_with_prerequisites | preview-group tests + multi-select UI PASS; live blocked |
| AC5-REGRESSION | AC-5 | **pass** | `run_discover` regression 6/6 suite; patterns tab code intact |
| AC6-OIDC | AC-6 | pass_with_prerequisites | Health + discover API 200; browser OIDC deferred deploy |

## UAT matrix summary

| Result | Count |
|--------|-------|
| pass | **1** (AC-5) |
| pass_with_prerequisites | **5** (AC-1..AC-4, AC-6) |
| fail | **0** |
| pending | **0** |

## Acceptance impact

| Row | Verify-work | Post-operator (release follow-up) |
|-----|-------------|-----------------------------------|
| **AC-1** | pass_with_prerequisites | Individual tx rows paginated in Transactions mode after deploy |
| **AC-2** | pass_with_prerequisites | Rich filters narrow results live |
| **AC-3** | pass_with_prerequisites | Account 114 SEPA-Lastschrift hint badge 11 txs @ 31d/95% |
| **AC-4** | pass_with_prerequisites | Multi-select activate → confirmed subscription |
| **AC-5** | pass | Patterns tab + detection pipeline unchanged (test-confirmed) |
| **AC-6** | pass_with_prerequisites | OIDC external profile smoke on Discover tx flow |

## Advisories (non-blocking)

1. **V1 deferred** — operator **BACKEND_FRONTEND_DEPLOY** gates all live AC-1..AC-4 and AC-6 browser probes.
2. **DB integration paths** — four us0021 tests skip without `DATABASE_URL` in QA env; verify-work re-run executed 6/6 PASS in current env (DB available).

## Next phase

**`/release`** — release notes; operator gate checklist; backlog US-0021 remains open until post-deploy smoke PASS.

`fresh_context_marker`: verify-work-20260613-us0021-qa-fresh  
`runtime_proof_id`: runtime-proof-verify-work-20260613-us0021-001  
`phase_boundary`: verify-work → release
