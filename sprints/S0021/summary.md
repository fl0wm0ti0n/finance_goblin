# Sprint S0021 — Execute Summary

**Story:** US-0022 — Deploy version stamp & stale-frontend detection  
**Status:** EXECUTE COMPLETE  
**Date:** 2026-06-14  
**Orchestrator:** `auto-20260613-bug0025`

## Tasks completed

| ID | Title | Status | Files |
|----|-------|--------|-------|
| B1 | Backend `meta` module + route registration | DONE | `backend/src/meta/mod.rs` (new), `backend/src/lib.rs` (mod + build_router merge) |
| B2 | Dockerfile `ARG`/`ENV`/`LABEL` chain | DONE | `backend/Dockerfile` |
| F1 | Vite `define` block injection | DONE | `frontend/vite.config.ts` |
| F2 | TypeScript declarations | DONE | `frontend/src/vite-env.d.ts` |
| F3 | `AppLayout` sidebar-footer stamp + tooltip | DONE | `frontend/src/components/AppLayout.tsx` |
| F4 | `useStaleDetection` hook | DONE | `frontend/src/hooks/useStaleDetection.ts` (new) |
| F5 | `StaleBanner` component | DONE | `frontend/src/components/StaleBanner.tsx` (new) |
| T1 | Integration test — meta endpoint shape | DONE | `backend/tests/meta_test.rs` (new) |
| G1 | Automated gate | DONE | All tests green |
| R1 | User guide US-0022 | DONE | `docs/user-guides/US-0022.md` (new) |
| V1 | Verify-work AC-1..AC-6 + OIDC smoke | DEFERRED | verify-work scope |

## Test results (G1)

| Suite | Result |
|-------|--------|
| `cargo test --lib` | **221 passed** / 0 failed |
| `cargo test --test meta_test` | **3 passed** / 0 failed |
| `npm test` | **31 passed** / 0 failed |
| `npm run build` | **PASS** (709 modules, 12.90s) |

## Key implementation decisions

1. **Meta route is public (no auth):** Registered in `build_router` before `api_router` (which has auth middleware). Architecture § US-0022 mandates public route — metadata is non-sensitive.

2. **`option_env!()` fallback:** `BUILD_ID` → `"dev"`, `RELEASE_TAG` → `"dev"`, `BUILD_TIMESTAMP` → `"unknown"`. Never breaks local dev build.

3. **Vite `define` (not `import.meta.env`):** Canonical for CI/Docker build-time injection. `JSON.stringify()` ensures string literals.

4. **On-mount stale detection only (GATE-STALE-1):** No periodic poll. Sufficient for operator use case (long-lived tabs rare).

5. **StaleBanner receives `stale` prop:** Hook lives in `AppLayout`; banner is pure presentational component.

## Files changed

- `backend/src/meta/mod.rs` (new)
- `backend/src/lib.rs` (mod declaration + build_router merge)
- `backend/Dockerfile` (ARG chain)
- `backend/tests/meta_test.rs` (new)
- `frontend/vite.config.ts` (define block)
- `frontend/src/vite-env.d.ts` (declarations)
- `frontend/src/components/AppLayout.tsx` (stamp + tooltip + StaleBanner integration)
- `frontend/src/hooks/useStaleDetection.ts` (new)
- `frontend/src/components/StaleBanner.tsx` (new)
- `docs/user-guides/US-0022.md` (new)
- `sprints/S0021/progress.md` (updated)
- `sprints/S0021/summary.md` (this file)

## Acceptance coverage

| Row | Status | Evidence |
|-----|--------|----------|
| AC-1 (subtle stamp) | IMPLEMENTED | F3 — sidebar-footer monospace stamp, low visual noise |
| AC-2 (hover tooltip) | IMPLEMENTED | F3 — title attribute with release + build + timestamp |
| AC-3 (backend metadata) | IMPLEMENTED + TESTED | B1 + T1 — 200 OK, shape verified, no secrets |
| AC-4 (SPA embed) | IMPLEMENTED | F1 + F2 + B2 — Vite define + Docker ARG chain |
| AC-5 (stale detection) | IMPLEMENTED | F4 + F5 — on-mount hook + non-blocking banner |
| AC-6 (regression) | VERIFIED | G1 — `/health` unchanged; cargo/npm/build all green |

## Next phase

**qa** — ready for `/qa` in new subagent/chat. V1 (verify-work) deferred to verify-work phase — requires operator **BACKEND_FRONTEND_DEPLOY**.
