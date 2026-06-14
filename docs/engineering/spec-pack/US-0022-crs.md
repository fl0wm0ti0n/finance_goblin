# CRS — US-0022 Deploy version stamp & stale-frontend detection

## Purpose

Give operators a compile-time build provenance oracle in the product UI: a subtle version stamp, hover details, and automatic stale-bundle detection after deploys. Eliminates the need for `docker inspect` or behavioral guesswork to confirm which release is running.

## Scope

### In scope

- **B1:** Backend `meta` module — `GET /api/v1/meta/build-info` with `option_env!()` compile-time metadata
- **B2:** Dockerfile `ARG` chain — `BUILD_ID`, `RELEASE_TAG`, `BUILD_TIMESTAMP` propagated through 3-stage build
- **F1:** Vite `define` block — `__BUILD_ID__`, `__RELEASE_TAG__` compile-time constants
- **F2:** TypeScript declarations — `declare const __BUILD_ID__: string;` in `vite-env.d.ts`
- **F3:** `AppLayout` sidebar-footer stamp — subtle label + hover tooltip (release tag, build id, timestamp)
- **F4:** `useStaleDetection()` hook — on-mount fetch + compare + stale state
- **F5:** Stale banner component — non-blocking, dismissible, reload CTA
- **T1:** Integration test — `GET /api/v1/meta/build-info` returns expected shape
- **G1:** `cargo test` + `npm test` + build
- **V1:** verify-work AC-1..AC-6; OIDC smoke

### Out of scope

- Periodic polling for stale detection
- Service Worker integration
- Changes to `/health` or `/health/ready`
- Release tag auto-bump or release-management UI
- Grafana dashboard metadata panel

## Acceptance criteria ref

`docs/product/acceptance.md` — US-0022:

- **AC-1** Version stamp: subtle indicator in sidebar footer
- **AC-2** Hover detail: release tag + build id + build timestamp (UTC)
- **AC-3** Backend metadata: `GET /api/v1/meta/build-info` returns `{build_id, release_tag, build_timestamp}`
- **AC-4** SPA embed: Vite `define` injects build id at compile time
- **AC-5** Stale detection: mismatch → non-blocking banner + reload; no false positive
- **AC-6** Regression: `/health` unchanged; OIDC smoke; no secrets

## Dependencies

- [R-0095](docs/engineering/research.md#r-0095--us-0022-deploy-version-stamp--stale-frontend-detection) §6–§12
- `backend/src/api/mod.rs` (route registration)
- `backend/src/health/mod.rs` (liveness unchanged)
- `backend/Dockerfile` (multi-stage build)
- `frontend/vite.config.ts` (define block)
- `frontend/src/components/AppLayout.tsx` (sidebar-footer)
- `frontend/src/vite-env.d.ts` (TS declarations)
