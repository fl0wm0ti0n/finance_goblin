# UAT — Q0031 (BUG-0022)

**Status:** COMPLETE  
**Verdict:** **PASS-WITH-PREREQUISITES**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0022 rows **BM**, **BN**  
**Sprint:** Q0031 (`/quick`)  
**Orchestrator:** `auto-20260613-bug0022`  
**Verified at:** 2026-06-13T09:50:00Z

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Result |
|-----|---------|-----------------------------------|--------|
| **BM** | BM1, T1, G1, V1 | Select non-active plan → Delete enabled → confirm removes plan; list refreshes | **pass_with_prerequisites** |
| **BN** | BM1, T1, G1, V1 | Active plan delete disabled + tooltip; API DELETE active → **409** per **DEC-0082**; OIDC smoke | **pass_with_prerequisites** (BN-API live **pass**) |

## Operator gates (before live probes)

| Gate | Status | Notes |
|------|--------|-------|
| FRONTEND_DEPLOY | pending | Frontend rebuild only — no migration; BM1 selector fix must be live |

## UAT steps

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| BM-UI | BM | `/planning` 2+ plans, one global active: select non-active → Delete enabled → confirm → plan removed | pass_with_prerequisites | Code+vitest PASS; 1 plan + FRONTEND_DEPLOY pending |
| BM-API | BM | `DELETE /api/v1/plans/:id` non-active → **204** | pass_with_prerequisites | Vitest matrix PASS; no non-active plan live |
| BN-UI | BN | Select globally active plan → delete disabled + tooltip | pass_with_prerequisites | Code review PASS; browser deferred FRONTEND_DEPLOY |
| BN-API | BN | `DELETE /api/v1/plans/:id` active → **409** `active_plan_delete_forbidden` | **pass** | Live DELETE → 409; cargo unit 1/1 |
| OIDC-1 | regression | `/planning`, `/api/v1/plans` smoke on omniflow profile | pass_with_prerequisites | `/api/v1/plans` 200; `/planning` 404 pre-deploy |

## Automated checks

| Check | Result |
|-------|--------|
| `npm test` (planSelector + planningFeedback) | **17/17 PASS** |
| `npm run build` | **PASS** |

## Results summary

| Result | Count |
|--------|-------|
| pass | 1 |
| pass_with_prerequisites | 4 |
| fail | 0 |

**Next phase:** `/release` (role: release)
