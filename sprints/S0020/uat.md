# UAT — Sprint S0020 / US-0021

**Sprint:** S0020  
**Story:** US-0021 — Subscription transaction explorer with rich filters  
**Phase:** verify-work (populated)  
**Status:** **PASS-WITH-PREREQUISITES**  
**Verified at:** 2026-06-13T08:10:01Z  
**Orchestrator:** `auto-20260613-us0021`  
**Decisions:** DEC-0112, DEC-0113, DEC-0114 (extends DEC-0098, DEC-0099, DEC-0111)

## UAT steps

| ID | AC | Description | Result |
|----|-----|-------------|--------|
| AC1-TX-SEARCH | AC-1 | Transactions mode lists individual expense txs with pagination/cap — not candidate-only | pass_with_prerequisites |
| AC2-FILTERS | AC-2 | Filters: account, payee, category, Geldbereich, date range | pass_with_prerequisites |
| AC3-HINT | AC-3 | Hint badge on filtered txs (account 114 SEPA-Lastschrift fixture) | pass_with_prerequisites |
| AC4-CONFIRM | AC-4 | Multi-select txs → preview-group → confirm as subscription/standing order | pass_with_prerequisites |
| AC5-REGRESSION | AC-5 | Patterns tab unchanged; US-0020 tags/majority; US-0003/US-0008 regression | **pass** |
| AC6-OIDC | AC-6 | OIDC external profile smoke on Discover tx search + confirm | pass_with_prerequisites |

## Target acceptance criteria

Source: `docs/product/acceptance.md` § US-0021

| ID | Criterion | Verify-work verdict |
|----|-----------|---------------------|
| AC-1 | Individual mirror expense txs, paginated/capped | pass_with_prerequisites |
| AC-2 | Rich filters: account, payee, category, Geldbereich, date | pass_with_prerequisites |
| AC-3 | Pattern hint on filtered txs below auto threshold | pass_with_prerequisites |
| AC-4 | Manual activate via tx group confirm (DEC-0085/0099) | pass_with_prerequisites |
| AC-5 | US-0020 + US-0003/US-0008 unchanged | **pass** |
| AC-6 | OIDC external profile smoke pass | pass_with_prerequisites |

## Operator gate (pre-runtime smoke)

| Gate | Action | Status |
|------|--------|--------|
| **BACKEND_FRONTEND_DEPLOY** | Deploy S0020 backend + frontend on US-0010 external profile (`financegnome.omniflow.cc`) | **pending** |

## Automated checks (verify-work re-run)

| Suite | Result |
|-------|--------|
| `cargo test --lib` | **221/221 PASS** |
| `cargo test --test us0021_transaction_search` | **6/6 PASS** |
| `npm test` | **17/17 PASS** |
| `npm run build` | **PASS** |

## Live probe baseline (pre-deploy)

| Probe | HTTP | Notes |
|-------|------|-------|
| `GET /health` | 200 | Stack reachable |
| `GET /api/v1/subscriptions/discover?account_id=114` | 200 | Legacy discover — 11-tx SEPA-Lastschrift @ 31d/95% |
| `GET /api/v1/subscriptions/transactions/search?account_id=114` | **404** | New route not deployed |
| `GET /subscriptions` | **404** | SPA shell not deployed |

## Results summary

| Result | Count |
|--------|-------|
| pass | **1** |
| pass_with_prerequisites | **5** |
| fail | **0** |
| total | **6** |

**Verdict:** PASS-WITH-PREREQUISITES — implementation verified by code review + automated tests; live AC-1..AC-4 and AC-6 browser probes deferred on operator **BACKEND_FRONTEND_DEPLOY**. AC-5 regression confirmed by `reg_discover_candidate_pass_unchanged_ac5` integration test.

**Next:** `/release` (role: release)
