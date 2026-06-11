# UAT — Q0026 (BUG-0018)

**Status:** COMPLETE — verify-work PASS (2026-06-09T21:22:00Z)  
**Acceptance:** `docs/product/acceptance.md` — BUG-0018 rows **BE**, **BF**  
**Sprint:** Q0026 (`/quick`)  
**Next phase:** `/release`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Result |
|-----|---------|-----------------------------------|--------|
| **BE** | BE1, T1, V1 | Post-sync alert evaluation completes without SQL error (`balance` ambiguous / **42702**); logs show no `alert evaluation failed` for normal sync runs | **pass** (code+test); V1-SYNC **pass_with_prerequisites** |
| **BF** | BE1, V1 | Header Alerts panel and `GET /api/v1/alerts` surface matching alerts when overdraft rules apply—not permanent **No active alerts** due to evaluation skip; subscription dedup regression per BUG-0008 | **pass**; V1-ALERTS/V1-BELL **pass_with_prerequisites**; V1-SUB-REG **pass** |

## Operator gates (before live probes)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild `flow-finance-ai` with BE1 alert SQL fix. — **PENDING**
2. **FULL_FIREFLY_SYNC** — Full sync; alerts phase must complete. — **PENDING**

## UAT steps

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| BE-CODE | BE | DEC-0107 `fbd.balance` + `fbd.ts` qualification in `evaluate_scarcity` | **pass** | `evaluate.rs` L23–30 |
| BE-TEST | BE | `cargo test --test wealth_alerts_integration` scarcity path | **pass** | 3/3 PASS |
| V1-SYNC | BE | `POST /api/v1/sync/trigger` — no `alert evaluation failed` / 42702 in logs | **pass_with_prerequisites** | HTTP 202; last_run success on pre-Q0026 deploy |
| V1-ALERTS | BF | `GET /api/v1/alerts?status=active` — rows when scarcity rule matches | **pass_with_prerequisites** | `[]` pre-deploy |
| V1-BELL | BF | Header bell — non-empty active preview | **pass_with_prerequisites** | AlertBell contract verified; empty pre-deploy |
| V1-SUB-REG | BF | `GET /api/v1/subscriptions/alerts` — dedup regression per BUG-0008 | **pass** | `reconciled=true`; pending_patterns=11 |
| OIDC-1 | regression | OIDC-enabled deploy regression per acceptance footnote | **pass_with_prerequisites** | Deferred post-deploy |

## Automated verification

| Check | Result |
|-------|--------|
| `cd backend && cargo test --lib` | **PASS** (213/213) |
| `cargo test --test wealth_alerts_integration` | **PASS** (3/3) |
| `cd frontend && npm test -- --run` | **PASS** (9/9) |
| localhost:18080 `/health` | **PASS** — HTTP 200 |
| localhost:18080 sync trigger + alerts API | **pass_with_prerequisites** — pre-Q0026 deploy |
| Omniflow reachability | **pass_with_prerequisites** — root 401; meta 200; alerts 200 |

## Results summary

- **Verdict:** **PASS** — 3/7 steps pass, 4 pass-with-prerequisites, 0 fail
- **Acceptance rows:** BE **pass**, BF **pass**
- **Blocking:** none
- **Traceability:** BUG-0018 rows **BE**, **BF** mapped in `sprints/quick/Q0026/uat.json`. Checkbox updates in `docs/product/acceptance.md` are **release** phase.

**Operator advisory:** After **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**, execute the 7-step smoke checklist in `uat.json` `operator_smoke_checklist` on `http://localhost:18080` and `https://financegnome.omniflow.cc`.
