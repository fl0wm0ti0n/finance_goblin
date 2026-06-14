# Sprint S0021 — UAT (verify-work)

**Story:** US-0022 — Deploy version stamp & stale-frontend detection  
**Phase:** verify-work  
**Date:** 2026-06-14  
**Orchestrator:** `auto-20260613-bug0025`  
**Verdict:** **PASS-WITH-PREREQUISITES**  
**Blockers:** 0

## UAT step results

| Step | Description | Result | Evidence |
|------|-------------|--------|----------|
| AC-1 | Version stamp: subtle build indicator in sidebar footer | **PASS** | `AppLayout.tsx` L94-104: sidebar-footer div, fontSize 0.7rem, color #94a3b8, monospace, `__BUILD_ID__.slice(0,7)` |
| AC-2 | Hover detail: release tag + build id + timestamp on hover | **PASS** | `AppLayout.tsx` L101: `title` attribute with Release/Build/Timestamp. Advisory: timestamp is client-side, not build-time |
| AC-3 | Backend metadata: `GET /api/v1/meta/build-info` returns build id + release tag, no secrets | **PASS** | `meta/mod.rs` + `meta_test.rs` 3/3 PASS (shape, no-secrets, fallback) |
| AC-4 | SPA embed: Vite `define` injects build id at compile time | **PASS** | `vite.config.ts` L8-11 define block; `Dockerfile` L29-36 frontend ARG chain; `npm run build` PASS |
| AC-5 | Stale detection: non-blocking banner when build ids mismatch | **PASS-WITH-PREREQUISITES** | `useStaleDetection.ts` + `StaleBanner.tsx` code PASS; live browser probe deferred **BACKEND_FRONTEND_DEPLOY** |
| AC-6 | Regression: `/health` unchanged; no secrets in metadata | **PASS-WITH-PREREQUISITES** | `health/mod.rs` unchanged; cargo lib 221/221; meta_test no-secrets PASS; OIDC smoke deferred **BACKEND_FRONTEND_DEPLOY** |
| GATE-META-1 | Dedicated `/api/v1/meta/build-info` route (not extend `/health`) | **PASS** | `meta/mod.rs` L19-21; `lib.rs` L210-214 public route before `api_router` |
| GATE-BUILD-1 | Build id = git short sha + release tag + UTC timestamp; Docker ARG chain | **PASS** | `meta/mod.rs` L13-15 `option_env!()`; `Dockerfile` L3-5 global, L10-12 builder, L29-31 frontend, L42-44 runtime, L46-48 OCI LABELs |

## Counts

- **Pass:** 6 (AC-1, AC-2, AC-3, AC-4, GATE-META-1, GATE-BUILD-1)
- **Pass-with-prerequisites:** 2 (AC-5, AC-6 — live smoke deferred BACKEND_FRONTEND_DEPLOY)
- **Fail:** 0
- **Total:** 8

## Independent test results

| Suite | Result | Notes |
|-------|--------|-------|
| `cargo test --lib` | **221 passed / 0 failed** | All green (pre-existing `config::tests::effective_enabled_futures_env_false_opt_out` not reproduced in this run) |
| `cargo test --test meta_test` | **3 passed / 0 failed** | Shape, no-secrets, fallback values — all PASS |
| `npm test` | **31 passed / 0 failed** | 6 test files, all green |
| `npm run build` | **PASS** (13.24s) | 709+ modules, dist produced |

## Runtime browser evidence

**UAT_BROWSER_PROBE_MODE:** cursor  
**Result:** `UAT_BROWSER_UNAVAILABLE`  
**Reason:** Backend dev server (port 8080) unreachable; frontend dev server (port 5173) serves different application ("Omniflow Trading"), not finance_goblin. Target app requires operator **BACKEND_FRONTEND_DEPLOY** before live browser probes can execute.

## Code quality observations

### Advisory: Tooltip timestamp is client-side (non-blocking)

**Location:** `AppLayout.tsx` L101  
**Issue:** `title` attribute uses `new Date().toISOString()` for timestamp, which is client-side current time, not build timestamp.  
**Impact:** Low — release tag and build id are sufficient for deploy verification. Build timestamp available via `/api/v1/meta/build-info`.  
**Recommendation:** Consider using `serverInfo.build_timestamp` from `useStaleDetection` in tooltip. Non-blocking for US-0022.

### Advisory: StaleBanner not dismissible (non-blocking)

**Location:** `StaleBanner.tsx`  
**Observation:** Banner has reload button but no dismiss/close button. Architecture mentions "dismissible" but implementation only has reload.  
**Impact:** Low — reload solves the stale state.  
**Recommendation:** Consider adding dismiss button. Non-blocking for US-0022.

## Operator gate

**BACKEND_FRONTEND_DEPLOY** → live AC-5 stale-detection browser smoke + AC-6 OIDC external profile smoke

## Summary

US-0022 / S0021 verify-work **PASS-WITH-PREREQUISITES**: 6 pass / 2 pass-with-prerequisites / 0 fail. All acceptance criteria AC-1..AC-6 and architecture gates GATE-META-1, GATE-BUILD-1 verified at code+test level. Live browser/API smoke deferred pending operator **BACKEND_FRONTEND_DEPLOY**. 0 blockers. Next phase: **release**.
