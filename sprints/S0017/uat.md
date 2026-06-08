# UAT — Sprint S0017 / US-0018

**Sprint:** S0017  
**Story:** US-0018 — Category filters & expense trend analytics  
**Phase:** verify-work (populated)  
**Status:** **PASS** — 5 code PASS + 1 pass-with-prerequisites  
**Plan-verified at:** 2026-06-08T22:00:00Z  
**QA verified at:** 2026-06-08T23:20:00Z  
**Verified at:** 2026-06-08T23:30:00Z  
**Orchestrator:** `auto-20260608-us0018-001`  
**Decisions:** DEC-0087, DEC-0088, DEC-0089, DEC-0090

## UAT steps

| ID | AC | Description | Result |
|----|-----|-------------|--------|
| UAT-1 | AC-1 | Shared category filter on forecast, planning, wealth + Grafana cashflow/budgets | **PASS** |
| UAT-2 | AC-2 | Monthly expense-series API 12 default / 24 max | **PASS** |
| UAT-3 | AC-3 | Bar trend chart; empty-state | **PASS** |
| UAT-4 | AC-4 | MoM + best/worst from API summary | **PASS** |
| UAT-5 | AC-5 | `__uncategorized__` explicit bucket | **PASS** |
| UAT-6 | AC-6 | OIDC smoke; US-0015 unchanged; read-only Firefly | **pass-with-prerequisites** |

## Target acceptance criteria

Source: `docs/product/acceptance.md` § US-0018

| ID | Criterion | Verify-work verdict |
|----|-----------|---------------------|
| AC-1 | Category filter on forecast, planning, wealth + Grafana cashflow/budgets | **PASS** (code) |
| AC-2 | Monthly expense-series API 12 default / 24 max | **PASS** |
| AC-3 | Bar trend chart; empty-state | **PASS** |
| AC-4 | MoM + best/worst from API summary | **PASS** |
| AC-5 | `__uncategorized__` explicit bucket | **PASS** |
| AC-6 | OIDC smoke; US-0015 unchanged; read-only Firefly | **pass-with-prerequisites** |

## Operator gate (pre-runtime smoke)

| Gate | Action |
|------|--------|
| **BACKEND_FRONTEND_DEPLOY** | Deploy S0017 backend + frontend on US-0010 external profile (`financegnome.omniflow.cc`) |
| **FULL_FIREFLY_SYNC** | Ensure `category_id` mirror current |
| **GRAFANA_PROVISIONING_RELOAD** | Reload cashflow + budgets dashboards after deploy |

## OIDC smoke checklist (AC-1, AC-6) — operator post-deploy

Profile: **US-0010 external** (Traefik auth + OIDC on SPA).

1. **`/forecast` Monthly** — Category filter visible; select a category with mirror spend; trend chart loads with month labels and EUR amounts; Income/Fixed/Variable cards unchanged vs before filter.
2. **US-0015 regression** — AI-mapped badge still appears when applicable; no changes to `bucket_sources` / `project.rs` behavior.
3. **`/planning` Compare** — Category filter + Actual spending trend widget; version compare table unchanged when category changes.
4. **`/wealth` Overview** — Category spending subsection; household total unchanged when category selected.
5. **`/analytics/cashflow`** — `$category` variable present; panel filters when set; **All categories** matches pre-US-0018 view.
6. **`/analytics/budgets`** — `$category` on Ist/deviation/MTD actual; planned series unchanged when category set.
7. **API** — `GET /api/v1/categories` returns catalog; `GET /api/v1/categories/expense-series?category_id=__uncategorized__` returns `uncategorized: true`.
8. **404 path** — Unknown `category_id` returns 404 JSON with hint (no silent empty household data).
9. **Read-only Firefly** — No POST/PUT/PATCH/DELETE to Firefly during smoke.
10. **Regression scope** — No `project.rs` edits in S0017; forecast monthly API response shape unchanged when category filter used in UI only.

## Automated checks (verify-work re-run)

| Check | Result | Notes |
|-------|--------|-------|
| `cargo test --lib` | **PASS** | 193/193 |
| `npm test -- --run` | **PASS** | 7/7 |
| Integration `us0018_categories` | **SKIP** | Requires `DATABASE_URL` (sandbox) |

## EXPLAIN probe (T-0185)

**Deferred** — no operator mirror in execute sandbox; DEC-0090 gate not triggered; no index migration shipped.

## Results summary

| Metric | Count |
|--------|-------|
| UAT steps total | 6 |
| Passed (code) | 5 |
| Pass-with-prerequisites | 1 |
| Failed | 0 |
| Operator checklist deferred | 9 |

**Verdict:** **PASS** — release unblocked; operator omniflow smoke advisory post-deploy per BUG-0013/0014/0015 precedent.

**Evidence:** `sprints/S0017/uat.json`, `sprints/S0017/verify-work-findings.md`, `docs/product/acceptance.md#US-0018`
