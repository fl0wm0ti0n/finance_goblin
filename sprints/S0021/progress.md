# Sprint S0021 — Progress

## Status: EXECUTE COMPLETE

**Created:** 2026-06-14  
**Execute completed:** 2026-06-14T19:05:00Z  
**Next phase:** `/qa` (qa)

## Task progress

| ID | Title | Status | Started | Completed | Notes |
|----|-------|--------|---------|-----------|-------|
| B1 | Backend `meta` module + route registration | DONE | 2026-06-14 | 2026-06-14 | `meta/mod.rs` created; registered in `lib.rs` `build_router` (public, no auth); `option_env!()` with fallback |
| B2 | Dockerfile `ARG`/`ENV`/`LABEL` chain | DONE | 2026-06-14 | 2026-06-14 | 3-stage ARG propagation; builder ENV; frontend RUN env; runtime OCI LABELs |
| F1 | Vite `define` block injection | DONE | 2026-06-14 | 2026-06-14 | `__BUILD_ID__` + `__RELEASE_TAG__` via `JSON.stringify(process.env.*)` |
| F2 | TypeScript declarations (`vite-env.d.ts`) | DONE | 2026-06-14 | 2026-06-14 | `declare const __BUILD_ID__: string;` + `__RELEASE_TAG__` |
| F3 | `AppLayout` sidebar-footer stamp + tooltip | DONE | 2026-06-14 | 2026-06-14 | Monospace build id fragment; title tooltip with release/build/timestamp |
| F4 | `useStaleDetection` hook | DONE | 2026-06-14 | 2026-06-14 | On-mount fetch `cache: 'no-store'`; skip dev; silent fail |
| F5 | `StaleBanner` component | DONE | 2026-06-14 | 2026-06-14 | Non-blocking banner; reload CTA; `role="alert"` |
| T1 | Integration test — meta endpoint shape | DONE | 2026-06-14 | 2026-06-14 | 3 tests: shape, no-secrets, fallback values |
| G1 | Automated gate — cargo test + npm test + build | DONE | 2026-06-14 | 2026-06-14 | cargo lib 221/221; meta_test 3/3; npm 31/31; build PASS |
| R1 | User guide US-0022 | DONE | 2026-06-14 | 2026-06-14 | `docs/user-guides/US-0022.md` created |
| V1 | Verify-work AC-1..AC-6 + OIDC smoke | DEFERRED | — | — | verify-work scope; requires BACKEND_FRONTEND_DEPLOY |

## Phase checkpoints

**SPRINT-PLAN COMPLETE** (2026-06-14) — 11 tasks materialized (B1, B2, F1, F2, F3, F4, F5, T1, G1, R1, V1); AC-1..AC-6 traced; traceability PLANNED; USER_GUIDE_MODE=1; UAT placeholders created; next **plan-verify** (qa).

**PLAN-VERIFY COMPLETE** (2026-06-14T18:51:00Z) — PASS verdict; 6/6 acceptance covered; 11/11 tasks traced; 4/4 gates traced; 0 gaps; 0 orphans; next **execute** (dev).

**EXECUTE COMPLETE** (2026-06-14T19:05:00Z) — 10/11 tasks DONE (V1 deferred to verify-work); G1 all green (cargo lib 221/221, meta_test 3/3, npm 31/31, build PASS); meta route registered as public (outside auth middleware); user guide published; isolation evidence: role=dev, fresh context; next **qa**.
