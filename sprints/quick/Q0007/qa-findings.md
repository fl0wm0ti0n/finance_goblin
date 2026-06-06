# QA Findings — Quick Q0007 / BUG-0001

**Work item:** BUG-0001 (defect)  
**Quick task:** Q0007  
**QA phase:** `/qa`  
**Date:** 2026-06-04  
**Verdict:** **PASS** (ready for `/verify-work`; omniflow post-redeploy smoke deferred)

## Scope

Omniflow production regressions per `architecture-20260604-bug0001` (`handoffs/tl_to_dev.md`):

- **A1** — `DevBypassAuthProvider` stub on `AuthContext` when `!isOidcConfigured`
- **B1** — `GF_SERVER_ROOT_URL` in `docker-compose.yml` + `.env.example`
- **B2** — Proxy HTML rewrite (skipped; gated on B1 smoke fail)

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0007/summary.md`, `docs/product/acceptance.md` (BUG-0001), `handoffs/tl_to_dev.md`, `frontend/src/auth/DevBypassAuthProvider.tsx`, `frontend/src/main.tsx`, `frontend/src/App.tsx`, `frontend/src/components/AppLayout.tsx`, `frontend/src/components/chat/ChatPanel.tsx`, `frontend/src/components/chat/ChatPanel.test.tsx`, `docker-compose.yml`, `.env.example`, `decisions/DEC-0057.md`, `docs/engineering/research.md` (R-0056 §3).

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | DevBypass stub contract | `cd frontend && npm test` | **PASS** (2/2) |
| T-2 | Frontend production build | `cd frontend && npm run build` | **PASS** |
| T-3 | Architecture A1 contract | Static review vs `handoffs/tl_to_dev.md` | **PASS** |
| T-4 | Architecture B1 contract | Static review compose + `.env.example` | **PASS** |
| T-5 | Frozen boundaries | Repo grep (no `SERVE_FROM_SUB_PATH`, no B2 rewrite) | **PASS** |
| T-6 | OIDC regression guard | Static review `main.tsx` + `ProtectedRoute` | **PASS** |
| T-7 | Omniflow row A smoke | Browser/console on deployed stack | **DEFERRED** — requires frontend image redeploy (A1) |
| T-8 | Omniflow row B smoke | Six analytics routes + Network tab assets | **DEFERRED** — requires Grafana recreate with `GF_SERVER_ROOT_URL` (B1) |

### Environment dependencies (non-blocking)

- **Operator redeploy:** `flow-finance-ai` + `grafana` per `handoffs/dev_to_qa.md` before live acceptance rows A+B can close.
- **Traefik basic-auth:** Unauthenticated `curl` to asset paths returns 401; not interpreted as acceptance failure.

## Acceptance criteria matrix (BUG-0001)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **(A)** | OIDC-unset + `AUTH_DEV_BYPASS=true`: no AuthProvider/useAuth errors; Chat opens | **PASS** (static/unit) | `main.tsx` ternary; `DevBypassAuthProvider` on `AuthContext`; Vitest no `useAuth` warn; `ChatPanel` `getToken` → `null`. Live console **DEFERRED** until frontend redeploy |
| **(B)** | Six `/analytics/{slug}` routes; no site-root `/public/build/` or `/public/img/` 404 | **PASS** (static) | `GF_SERVER_ROOT_URL` with trailing slash in compose + `.env.example`; DEC-0057 / R-0056 Pattern A. Live iframe/asset **DEFERRED** until Grafana redeploy |
| OIDC regression | OIDC-configured build redirects to IdP | **PASS** (static) | Stub only when `!isOidcConfigured`; real `AuthProvider` + `OidcProtectedRoute` unchanged |

**Summary:** 3/3 PASS on static/automated path; rows A+B live runtime deferred with `OPERATOR_REDEPLOY_PENDING` (host reachable; fix not yet witnessed on omniflow).

## Architecture compliance

### Sub-defect A — DevBypassAuthProvider

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Mount gate | `!isOidcConfigured` only | `main.tsx` ternary | PASS |
| Context | Same `AuthContext` from `react-oidc-context` | `DevBypassAuthProvider.tsx` | PASS |
| `user` | `undefined` | Factory + test assert | PASS |
| Consumer audit | AppLayout gated; ChatPanel token null | No changes required per architecture | PASS |
| OIDC path | Real `AuthProvider` when authority set | Unchanged `oidcConfig` spread | PASS |
| Tests | Vitest stub mount | 2/2 PASS | PASS |

### Sub-defect B — GF_SERVER_ROOT_URL

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Compose env | Default omniflow URL + trailing slash | `docker-compose.yml` line 73 | PASS |
| Operator override | `${GF_SERVER_ROOT_URL:-…}` | Compose interpolation | PASS |
| `.env.example` | Documented with local override comment | Lines 77–81 | PASS |
| `GF_SERVER_SERVE_FROM_SUB_PATH` | Not enabled | Absent from compose | PASS |
| Proxy / iframe | Unchanged prefix strip | No `proxy.rs` diff in Q0007 | PASS |
| B2 rewrite | Skipped unless B1 smoke fails | Not implemented | PASS (gated skip) |

## Generated test evidence

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `typescript` |
| `generated_test_command` | `cd frontend && npm test && npm run build` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-06-04 — vitest 2/2; tsc + vite build exit 0 |
| `generated_test_paths_ref` | `frontend/src/components/chat/ChatPanel.test.tsx`, `frontend/src/auth/DevBypassAuthProvider.tsx` |

## Runtime QA evidence (omniflow)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed (operator-owned redeploy) |
| `runtime_stack_profile` | `docker-compose` external profile |
| `runtime_mode` | `deferred` |
| `runtime_health_target` | BUG-0001 rows A+B on `https://financegnome.omniflow.cc` |
| `runtime_health_result` | `deferred` |
| `runtime_final_verdict` | `deferred` (verify-work / operator) |
| `runtime_reason_code` | `OPERATOR_REDEPLOY_PENDING` |
| `runtime_evidence_refs` | `handoffs/dev_to_qa.md` redeploy block; partial curl: `/health` 200, `/analytics/grafana/api/health` 200; asset paths not verified post-B1 |

## Findings

### Blockers

None.

### Advisories (non-blocking)

1. **Operator redeploy** `flow-finance-ai` + `grafana` before verify-work closes live BUG-0001 acceptance.
2. If post-redeploy B smoke still shows site-root `/public/` 404, open **B2** bounded spike per architecture gate.
3. Hard refresh browser after deploy; confirm six analytics slugs in Network tab.

## Verdict

**PASS** — proceed to `/verify-work` in fresh subagent. No `handoffs/qa_to_dev.md`.
