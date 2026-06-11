# Q0024 summary — BUG-0016

**Sprint:** Q0024 (quick)  
**Bug:** BUG-0016 — SPA deep links return HTTP 404  
**Orchestrator:** `intake-20260609-ui-audit`  
**Execute completed:** 2026-06-09

## Goal

Close BUG-0016 by implementing **DEC-0104** Axum SPA `index.html` fallback in `build_router` (HTTP 200 for deep links) and integration regression tests.

## Tasks completed

| ID | Title | Status | Evidence |
|----|-------|--------|----------|
| AX1 | SPA `index.html` fallback in `build_router` | **done** | `backend/src/lib.rs` — `attach_spa_fallback` + `ServeDir::fallback(ServeFile)` |
| AX2 | Integration tests — deep links + protected prefixes | **done** | `backend/tests/spa_fallback_integration.rs`, `backend/tests/fixtures/spa/` |
| V1 | verify-work curl + browser smoke | **open** | Deferred to `/verify-work` after **BACKEND_FRONTEND_DEPLOY** |

## Implementation notes

- **DEC-0104:** `ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")))` returns HTTP 200 for missing non-API paths.
- **DEC-0057:** Route merge order unchanged — health → Grafana proxy → API → SPA fallback.
- **Frozen boundaries:** No Traefik label change; no backend `/callback` redirect; no `not_found_service`.
- Exported `attach_spa_fallback` for integration tests mirroring `build_router` prefix ordering.

## Test results

| Check | Result |
|-------|--------|
| `cargo test --lib` | **213/213 PASS** |
| `cargo test --test spa_fallback_integration` | **5/5 PASS** |
| `npm test -- --run` | **9/9 PASS** |

## Operator gates (before V1)

| Gate | Status |
|------|--------|
| **BACKEND_FRONTEND_DEPLOY** | pending — rebuild `flow-finance-ai` image with AX1 SPA fallback |

## Next phase

**`/qa`** — code review + test verification; V1 runtime smoke deferred to verify-work.
