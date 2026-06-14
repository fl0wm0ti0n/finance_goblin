# UAT — Q0033 (BUG-0024)

**Status:** VERIFY-WORK COMPLETE — **PASS-WITH-PREREQUISITES**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0024 rows **BR**, **BS**  
**Sprint:** Q0033 (`/quick`)  
**Orchestrator:** `auto-20260613-bug0024`  
**Verified:** 2026-06-13T15:05:00Z

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Result |
|-----|---------|-----------------------------------|--------|
| **BS** | H1, F1, T1, G1, V1 | Sole active plan: delete disabled + clear create→activate→delete guidance | pass_with_prerequisites |
| **BR** | G1, V1 | 2+ plans: non-active selected → delete enabled → plan removed | **pass** |

## Operator gates (before live BS probe)

| Gate | Status | Notes |
|------|--------|-------|
| FRONTEND_DEPLOY | pending | Sole-plan inline hint must be live; omniflow full BR/BS smoke |

## UAT steps

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| BS-UI | BS | `/planning` 1 sole active plan — inline hint visible, delete disabled | pass_with_prerequisites | Vitest 7/7; hint absent pre-deploy |
| BR-UI | BR | `/planning` 2+ plans — non-active selected → delete enabled | **pass** | Browser MCP localhost |
| BR-API | BR | `DELETE` active plan → **409** `active_plan_delete_forbidden` | **pass** | curl oracle |
| BN-regression | regression | Active plan selected — delete disabled + tooltip (Q0031) | **pass** | Browser MCP |
| OIDC-1 | regression | `/planning` `/api/v1/plans` smoke | pass_with_prerequisites | HTTP 200; BS deferred deploy |

**Next phase:** `/release`
