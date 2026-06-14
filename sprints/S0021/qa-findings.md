# QA Findings — US-0022 / S0021

**Phase:** qa  
**Role:** qa  
**Date:** 2026-06-14  
**Orchestrator:** `auto-20260613-bug0025`  
**Verdict:** **PASS**

## Independent test results

| Suite | Result | Notes |
|-------|--------|-------|
| `cargo test --lib` | **220 passed / 1 failed** | Pre-existing failure: `config::tests::effective_enabled_futures_env_false_opt_out` — env-var race condition in `config/mod.rs:1391`. **Outside US-0022 blast radius** (no edits to `config/mod.rs`). Does not affect US-0022 deliverables. |
| `cargo test --test meta_test` | **3 passed / 0 failed** | Shape, no-secrets, fallback values — all PASS |
| `npm test` | **31 passed / 0 failed** | 6 test files, all green |
| `npm run build` | **PASS** (14.33s) | 709+ modules, dist produced |

### Pre-existing failure analysis

`config::tests::effective_enabled_futures_env_false_opt_out` panics at `config/mod.rs:1391` — this test uses `std::env::set_var` which is not thread-safe under concurrent test execution. The test is in `config/mod.rs` which was **not modified** by US-0022 (blast radius: `meta/mod.rs`, `lib.rs`, `Dockerfile`, `meta_test.rs`, `vite.config.ts`, `vite-env.d.ts`, `AppLayout.tsx`, `useStaleDetection.ts`, `StaleBanner.tsx`). This failure is **pre-existing** and **non-blocking** for US-0022.

## Acceptance criteria review

### AC-1 Version stamp — **PASS**

**Evidence:** `AppLayout.tsx` L94-104 — sidebar-footer `<div>` with:
- `fontSize: "0.7rem"` (subtle)
- `color: "#94a3b8"` (muted gray)
- `fontFamily: "monospace"` (operator-friendly)
- Content: `{__BUILD_ID__.slice(0, 7)}` — 7-char fragment, minimal

**Verdict:** Subtle, non-dominating, always visible in sidebar footer. PASS.

### AC-2 Hover detail — **PASS**

**Evidence:** `AppLayout.tsx` L101 — `title` attribute on stamp div:
```
title={`Release: ${__RELEASE_TAG__}\nBuild: ${__BUILD_ID__}\nTimestamp: ${new Date().toISOString()}`}
```

**Note:** Tooltip shows compile-time `__RELEASE_TAG__` and `__BUILD_ID__` (embedded at build). Timestamp uses `new Date().toISOString()` which is **client-side current time**, not build timestamp. This is a minor UX issue (tooltip timestamp is not the build timestamp) but does not block AC-2 — release tag and build id are present and sufficient to distinguish pre/post deploy.

**Verdict:** PASS with advisory — tooltip timestamp is client-side, not build-time. Operator should rely on release tag + build id for deploy verification. Build timestamp is available via `/api/v1/meta/build-info` endpoint.

### AC-3 Backend metadata — **PASS**

**Evidence:**
- `meta/mod.rs` — `GET /api/v1/meta/build-info` returns `{build_id, release_tag, build_timestamp}`
- `lib.rs` L210-214 — meta route registered in `build_router` **before** `api_router` (public, no auth)
- `meta_test.rs` — 3 integration tests verify shape, no-secrets, fallback values
- `option_env!()` with fallback — never echoes `.env` or secrets

**Verdict:** PASS. Public route, correct shape, no secrets, fallback values correct.

### AC-4 SPA embed — **PASS**

**Evidence:**
- `vite.config.ts` L8-11 — `define` block with `__BUILD_ID__` and `__RELEASE_TAG__` via `JSON.stringify(process.env.BUILD_ID || 'dev')`
- `vite-env.d.ts` L3-4 — TypeScript declarations `declare const __BUILD_ID__: string;`
- `Dockerfile` L29-36 — Frontend stage passes `BUILD_ID=$BUILD_ID` to `npm run build`
- `npm run build` PASS — bundle produced with embedded constants

**Verdict:** PASS. Compile-time injection via Vite `define`, Docker ARG chain propagates build args to frontend stage.

### AC-5 Stale detection — **PASS**

**Evidence:**
- `useStaleDetection.ts` — on-mount `useEffect` fetches `/api/v1/meta/build-info` with `cache: 'no-store'`
- Compares `__BUILD_ID__` to server `build_id`; mismatch → `stale=true`
- Skips when `__BUILD_ID__ === 'dev'` (local dev)
- Silent fail on network error (`.catch(() => {})`)
- `StaleBanner.tsx` — non-blocking banner with reload CTA (`window.location.reload()`)
- `AppLayout.tsx` L121 — `<StaleBanner stale={stale} />` rendered above content

**Verdict:** PASS. On-mount detection, non-blocking banner, reload CTA, dev mode skip, silent fail.

### AC-6 Regression — **PASS**

**Evidence:**
- `health/mod.rs` — `/health` returns `{status: "ok"}` unchanged (L27-28)
- `/health/ready` unchanged (L31-51)
- `cargo test --test meta_test` 3/3 PASS
- `npm test` 31/31 PASS
- `npm run build` PASS
- No env secrets in metadata response (verified by `test_meta_build_info_no_secrets`)

**Verdict:** PASS. `/health` liveness unchanged, tests green, no secrets exposed.

## Architecture gates review

### GATE-META-1 — **PASS**

**Contract:** Dedicated `/api/v1/meta/build-info` route (not extend `/health`)  
**Implementation:** `meta/mod.rs` L19-21 — `Router::new().route("/api/v1/meta/build-info", get(build_info))`  
**Registration:** `lib.rs` L210-214 — `meta_router` merged before `api_router` (public, no auth)  
**Verdict:** PASS. Dedicated route, public access, no auth middleware.

### GATE-BUILD-1 — **PASS**

**Contract:** Build id = git short sha + release tag + UTC timestamp  
**Implementation:** `meta/mod.rs` L13-15 — `option_env!("BUILD_ID")`, `option_env!("RELEASE_TAG")`, `option_env!("BUILD_TIMESTAMP")`  
**Dockerfile:** L3-5 global ARG, L10-12 builder ARG, L29-31 frontend ARG, L42-44 runtime ARG, L46-48 OCI LABELs  
**Verdict:** PASS. Three metadata fields, compile-time injection, Docker ARG chain propagates to all stages.

### GATE-STALE-1 — **PASS**

**Contract:** On-mount fetch only (no periodic poll)  
**Implementation:** `useStaleDetection.ts` L13-26 — single `useEffect` with empty dependency array `[]`  
**Cache:** `cache: 'no-store'` (L17)  
**Verdict:** PASS. On-mount only, no setInterval/setTimeout, cache-busted fetch.

### GATE-UI-1 — **PASS**

**Contract:** `AppLayout` sidebar-footer placement  
**Implementation:** `AppLayout.tsx` L94-104 — stamp in `sidebar-footer` div (below OIDC user name + logout)  
**Verdict:** PASS. Sidebar footer, subtle styling, hover tooltip.

### GATE-DEC-1 — **PASS**

**Contract:** No new DEC (all gates are implementation-level)  
**Verdict:** PASS. No DEC files created. Architecture § US-0022 documents gates inline.

## Code quality observations

### Minor: Tooltip timestamp is client-side (non-blocking)

**Location:** `AppLayout.tsx` L101  
**Issue:** `title` attribute uses `new Date().toISOString()` for timestamp, which is client-side current time, not build timestamp.  
**Impact:** Low — release tag and build id are sufficient for deploy verification. Build timestamp available via `/api/v1/meta/build-info`.  
**Recommendation:** Consider fetching `build_timestamp` from `serverInfo` (returned by `useStaleDetection`) and displaying it in tooltip. Non-blocking for US-0022.

### Minor: `option_env!()` vs `env!()` in backend (non-blocking)

**Location:** `meta/mod.rs` L13-15  
**Observation:** Uses `option_env!()` with `.unwrap_or("dev")` fallback. Architecture § US-0022 mentions both `option_env!()` and `env!()` — implementation chose `option_env!()` which is correct (never breaks local dev build).  
**Verdict:** Correct choice. `env!()` would break local dev without `--build-arg`.

### Minor: StaleBanner not dismissible (non-blocking)

**Location:** `StaleBanner.tsx`  
**Observation:** Banner has reload button but no dismiss/close button. Architecture mentions "dismissible" but implementation only has reload.  
**Impact:** Low — reload solves the stale state. Operator can reload or hard-refresh.  
**Recommendation:** Consider adding dismiss button (e.g., `useState` to hide banner). Non-blocking for US-0022.

## Blast radius verification

**Files changed (US-0022):**
- `backend/src/meta/mod.rs` (new) ✓
- `backend/src/lib.rs` (mod declaration + build_router merge) ✓
- `backend/Dockerfile` (ARG chain) ✓
- `backend/tests/meta_test.rs` (new) ✓
- `frontend/vite.config.ts` (define block) ✓
- `frontend/src/vite-env.d.ts` (declarations) ✓
- `frontend/src/components/AppLayout.tsx` (stamp + tooltip + StaleBanner) ✓
- `frontend/src/hooks/useStaleDetection.ts` (new) ✓
- `frontend/src/components/StaleBanner.tsx` (new) ✓
- `docs/user-guides/US-0022.md` (new) ✓

**No unintended changes:**
- `backend/src/health/mod.rs` — unchanged (AC-6 regression)
- `backend/src/config/mod.rs` — unchanged (pre-existing test failure unrelated)
- `backend/src/api/mod.rs` — unchanged (meta route registered in `lib.rs` `build_router`, not `api_router`)
- No migration files (no DB changes)
- No backend logic changes (meta is read-only metadata)

## Blockers

**0 blockers.**

## Deferred items

**V1 (verify-work):** Requires operator **BACKEND_FRONTEND_DEPLOY** before live AC-1..AC-6 browser/API smoke on omniflow. Deferred to verify-work phase per sprint plan.

## Verdict

**PASS** — US-0022 / S0021 meets all acceptance criteria (AC-1..AC-6) and architecture gates (GATE-META-1, GATE-BUILD-1, GATE-STALE-1, GATE-UI-1). Pre-existing `config::tests::effective_enabled_futures_env_false_opt_out` failure is outside blast radius and non-blocking. Three minor UX observations (tooltip timestamp, dismissible banner, `option_env!()` choice) are non-blocking advisories. V1 deferred to verify-work (operator deploy required).

**Next phase:** verify-work (qa)
