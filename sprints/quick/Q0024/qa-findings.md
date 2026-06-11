# QA Findings — Quick Q0024 / BUG-0016

**Work item:** BUG-0016 (defect)  
**Quick task:** Q0024  
**QA phase:** `/qa`  
**Date:** 2026-06-09  
**Orchestrator:** `intake-20260609-ui-audit`  
**Decisions:** DEC-0104, DEC-0057  
**QA agent:** fresh subagent (`qa-20260609-bug0016-qa-fresh`)  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — Implemented tasks **AX1** and **AX2** satisfy **DEC-0104** and **DEC-0057** at code and test level. Zero blocking findings. **V1** correctly deferred (operator **BACKEND_FRONTEND_DEPLOY** gate). Hand off to **`/verify-work`** for curl/browser/OIDC runtime probes.

## Scope

BUG-0016 SPA deep links return HTTP 404: Axum `index.html` fallback in `build_router` (**AX1** / DEC-0104), integration regression tests for deep links and protected prefixes (**AX2**). Runtime **V1** curl + browser smoke deferred per operator deploy gate.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0024/summary.md`, `sprints/quick/Q0024/tasks.md`, `sprints/quick/Q0024/uat.json`, `backend/src/lib.rs`, `backend/tests/spa_fallback_integration.rs`, `backend/tests/fixtures/spa/`, `decisions/DEC-0104.md`, `docs/product/acceptance.md` row **AX**. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | Full lib regression | `cargo test --lib` | **PASS** (213/213) |
| T-2 | SPA fallback integration | `cargo test --test spa_fallback_integration` | **PASS** (5/5) |
| T-3 | Frontend vitest regression | `cd frontend && npm test -- --run` | **PASS** (9/9) |
| T-4 | **AX1** — DEC-0104 fallback API + route order | Code review `lib.rs` | **PASS** |
| T-5 | **AX2** — deep links 200 HTML + protected prefixes | Code review + T-2 | **PASS** |
| T-6 | **DEC-0057** — Grafana proxy before SPA fallback | Code review + `grafana_proxy_not_spa_html` | **PASS** |
| T-7 | Frozen boundaries — no Traefik/backend `/callback` change | Code review | **PASS** |
| T-8 | V1 curl + browser + OIDC smoke | `sprints/quick/Q0024/uat.md` | **DEFERRED** — verify-work |
| T-9 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |
| T-10 | LINT / TYPECHECK | runbook keys blank | **SKIP** |

### Test output (T-1)

```
test result: ok. 213 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Test output (T-2)

```
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Task verdict matrix

| Task | Status (execute) | QA verdict | Notes |
|------|------------------|------------|-------|
| AX1 | done | **PASS** | `ServeDir::fallback(ServeFile::new(index))`; both static-dir branches; DEC-0057 order preserved |
| AX2 | done | **PASS** | 5 integration tests; deep links + API/health/Grafana/assets protected |
| V1 | open | **DEFERRED** | Operator gate: **BACKEND_FRONTEND_DEPLOY** before `:18080` / omniflow probes |

## Acceptance criteria matrix (BUG-0016)

| Row | Criterion | QA verdict | Evidence |
|-----|-----------|------------|----------|
| **AX** | Direct navigation, hard-refresh, bookmarks to client routes return HTTP 200 SPA shell — not 404 blank body; `:18080` + omniflow; OIDC regression | **PASS** (code) / **DEFERRED** (runtime) | AX1 DEC-0104 fallback; AX2 integration 5/5; T-1/T-2/T-4/T-5 PASS. Live curl/browser/OIDC **DEFERRED** to verify-work |

## Code review vs decisions

### DEC-0104 (AX1)

| Contract | Status | Evidence |
|----------|--------|----------|
| Fallback API: `ServeDir::fallback(ServeFile::new(index.html))` | **PASS** | `attach_spa_fallback` lines 194–197 |
| HTTP **200** for missing non-API paths (not 404-with-body) | **PASS** | `deep_links_return_200_html_shell` asserts `StatusCode::OK` |
| Prod branch `/app/static` | **PASS** | `build_router` lines 216–217 |
| Dev branch `frontend/dist` | **PASS** | `build_router` lines 218–219 |
| No `not_found_service` | **PASS** | grep: none in `lib.rs` |
| No backend `/callback` redirect | **PASS** | `/callback` covered by SPA fallback in AX2 test only |
| Placement in `build_router` only | **PASS** | `backend/src/lib.rs`; no Traefik label changes |

### DEC-0057 (route ordering)

| Contract | Status | Evidence |
|----------|--------|----------|
| Merge order: health → Grafana → API → SPA | **PASS** | `build_router` lines 209–212; `test_router` mirrors order |
| `/health` returns JSON, not HTML | **PASS** | `health_returns_json_not_html` |
| `/api/v1/*` JSON responses preserved | **PASS** | `api_paths_return_json_not_html` |
| `/analytics/grafana/*` proxy preserved | **PASS** | `grafana_proxy_not_spa_html` (wiremock upstream) |
| Hashed `/assets/*` served when present | **PASS** | `static_assets_served_with_js_content_type` |

### AX2 integration coverage

| Path class | Paths tested | Status |
|------------|--------------|--------|
| Deep links (200 HTML) | `/forecast`, `/subscriptions`, `/planning`, `/sync`, `/analytics/cashflow`, `/callback` | **PASS** |
| Protected prefixes | `/health`, `/api/v1/nonexistent`, `/analytics/grafana/api/health`, `/assets/fixture.js` | **PASS** |

## Generated baseline test evidence (US-0066)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | rust + node (vitest) |
| `generated_test_command` | `cargo test --lib`; `cargo test --test spa_fallback_integration`; `cd frontend && npm test -- --run` |
| `generated_test_result` | **pass** |
| `generated_test_output_ref` | T-1/T-2/T-3 output above |
| `generated_test_paths_ref` | `backend/tests/spa_fallback_integration.rs`, `backend/src/lib.rs` |
| `generated_test_reason_code` | — |

## Runtime QA evidence (US-0065)

Runtime autopilot probes **not executed in QA phase** — V1 deferred per sprint contract and **BACKEND_FRONTEND_DEPLOY** operator gate. Verify-work owns curl/browser/OIDC matrix.

| Field | Value |
|-------|-------|
| `runtime_startup_command` | deferred |
| `runtime_stack_profile` | rust axum + react vite |
| `runtime_mode` | deferred (verify-work) |
| `runtime_health_target` | `:18080` + `financegnome.omniflow.cc` |
| `runtime_health_result` | deferred |
| `runtime_log_summary` | n/a |
| `runtime_retry_count` | 0 |
| `runtime_retry_ledger` | [] |
| `runtime_final_verdict` | deferred |
| `runtime_reason_code` | V1_DEFERRED_BACKEND_FRONTEND_DEPLOY |
| `runtime_evidence_refs` | `sprints/quick/Q0024/uat.md` |

## Issues

None blocking. No `handoffs/qa_to_dev.md` required.

**Informational (non-blocking):**

1. Integration tests use `test_router` + exported `attach_spa_fallback` mirroring `build_router` — not a live `build_router` e2e with full `AppState`; acceptable for AX2 contract verification.
2. **V1 runtime** — curl matrix, hard-refresh, bookmarks, OIDC `/callback` live probes pending deploy rebuild.
3. Pre-existing lib compile warnings (unused imports) — unchanged by Q0024; not blocking.

## Verify-work readiness

| Gate | Status |
|------|--------|
| Code QA (AX1–AX2) | **READY** |
| `cargo test --lib` | **READY** — 213/213 PASS |
| `cargo test --test spa_fallback_integration` | **READY** — 5/5 PASS |
| `npm test` | **READY** — 9/9 PASS |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** |
| V1 curl + browser + OIDC (AX) | **PENDING** |

## Next phase

**`/verify-work`** — rebuild `flow-finance-ai`, then V1 probes per `sprints/quick/Q0024/uat.md`.

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
