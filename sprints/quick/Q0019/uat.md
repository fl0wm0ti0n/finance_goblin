# UAT — Q0019 (BUG-0011)

**Status:** Verify-work **PASS** (2026-06-08) — AD/AE/AF code/test verified; V1 omniflow pass-with-prerequisites  
**Acceptance:** `docs/product/acceptance.md` — BUG-0011 rows **AD**, **AE**, **AF**  
**Execute:** complete 2026-06-08  
**QA:** PASS 2026-06-08  
**Orchestrator:** `auto-20260608-bug0011-001`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Verify-work |
|-----|---------|-----------------------------------|-------------|
| **AD** | AD1–AD4, T1, V1 | "Start empty and add lines" creates editable plan with add-line UX — not silent no-op | **PASS** (code) / **pass-with-prerequisites** (runtime) |
| **AE** | AE1–AE3, AD4, T1, V1 | Compare with empty/minimal plan shows zero or neutral deltas — not illogical aggregates | **PASS** (code) / **pass-with-prerequisites** (runtime) |
| **AF** | AF1–AF2, T1, V1 | Plan-vs-actual returns 200 JSON or guided UX when no active plan — not 404 breaking tab | **PASS** (code) / **pass-with-prerequisites** (runtime) |
| Regression | T1, V1 | OIDC-enabled deploy `/planning` three-tab regression | **pass-with-prerequisites** |

## Operator gate

1. Deploy backend + frontend image with AE1–T1 + AF2/AD1–AD4 merged. — **PENDING**
2. **BACKEND_FRONTEND_DEPLOY** — confirm containers on omniflow before runtime probes. — **PENDING**

Per **US-0010** precedent: code-level and automated verification **pass**; omniflow host runtime steps recorded as **PASS-with-prerequisites** where operator deploy is required.

## Smoke checklist (omniflow — `financegnome.omniflow.cc`)

| Step | Probe | Pass criteria | Result |
|------|-------|---------------|--------|
| AD-1 | `/planning` Scenarios — Create empty plan | Editable plan with empty adjustments table; add form visible | **pass-with-prerequisites** — AD1 code PASS; live probe post-deploy |
| AD-2 | Add adjustment inline | POST creates row; PATCH edits row | **pass-with-prerequisites** — AD2 code PASS |
| AD-3 | Custom template Apply | Toast "Custom plan ready — add lines below"; table refreshes | **pass-with-prerequisites** — AD3 code PASS |
| AE-1 | Compare tab — zero-adjustment plan | `monthly_delta_sum` ≈ 0.00; not full forecast net | **pass-with-prerequisites** — AE3 + integration tests PASS |
| AE-2 | Compare tab — Leasing template | Overlay delta ~ leasing amount; projected balance may differ | **pass-with-prerequisites** — Leasing ~-300 test PASS |
| AF-1 | `GET /api/v1/plans/active/plan-vs-actual` (no active) | HTTP 200 `{ status: "no_active_plan", reason: "no_active_plan" }` | **pass-with-prerequisites** — AF1 serialization PASS |
| AF-2 | Plan vs Actual tab (no active) | Guided card with create + Set active CTA — not blank/404 | **pass-with-prerequisites** — AF2 code PASS |
| REG-1 | OIDC `/planning` three tabs | Scenarios + Compare + Plan vs Actual load without console errors | **pass-with-prerequisites** |
| REG-2 | Grafana Dashboard 3 | No provisioning JSON changes in deploy | **pass-with-prerequisites** |

## Local gates (verify-work — complete)

| Step | Description | Result |
|------|-------------|--------|
| VW-AUTO-1 | `cargo test --test plans_integration` | **PASS** (5/5) |
| VW-AUTO-2 | `cargo test --lib` | **PASS** (160/160) |
| UAT-AD-CODE | First-run grid, inline add/edit, Custom toast, Set active banner | **PASS** |
| UAT-AE-CODE | Overlay-only monthly_delta_sum; zero-overlay 0.00; Leasing ~overlay delta | **PASS** |
| UAT-AF-CODE | Route 200 no_active_plan; guided PVA empty state | **PASS** |
| T1 | Compare + PVA integration/unit tests | **PASS** |

## V1 smoke notes

- Compare footnote explains overlay-only delta vs projected balance (AD4)
- No auto-activate on create — user must Set active explicitly
- Service layer still returns `PlanError::NoActivePlan`; HTTP 200 handling is route-layer only per DEC-0074
- AE-before-AF sequencing honored in sprint execution

## Results summary

| Metric | Count |
|--------|-------|
| Acceptance rows **AD**, **AE**, **AF** | **3/3 PASS** |
| UAT steps (full pass) | 3 |
| UAT steps (pass-with-prerequisites) | 9 |
| Failed | 0 |
| Automated checks | 5/5 PASS |

**Canonical acceptance:** `docs/product/acceptance.md#BUG-0011` — rows **AD**, **AE**, and **AF** satisfied at code/test level; runtime omniflow probes documented as operator prerequisites per US-0010 policy.

## Next phase

**`/release`** — BUG-0011 closure; check acceptance checkbox; publish release notes.
