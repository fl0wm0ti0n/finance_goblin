# Sprint S0021

**ID:** S0021  
**Story:** US-0022 — Deploy version stamp & stale-frontend detection  
**Status:** PLANNED  
**Created:** 2026-06-14  
**Orchestrator:** `auto-20260613-bug0025`

## Goal

Deliver compile-time build provenance oracle: backend `GET /api/v1/meta/build-info` endpoint with `option_env!()` metadata, Docker `ARG` chain propagation through 3-stage build, Vite `define` injection for SPA build id, subtle UI stamp in `AppLayout` sidebar footer with hover tooltip, and on-mount stale-bundle detection with non-blocking reload banner. Eliminate operator need for `docker inspect` or behavioral guesswork to confirm running release.

## Scope

| Slice | Tasks | Primary files |
|-------|-------|---------------|
| **US-0022-S1** — Backend metadata endpoint | B1, T1 | `backend/src/meta/mod.rs`, `backend/src/api/mod.rs`, `backend/tests/meta_test.rs` |
| **US-0022-S2** — Docker build-arg chain | B2 | `backend/Dockerfile` |
| **US-0022-S3** — Frontend build id embed | F1, F2 | `frontend/vite.config.ts`, `frontend/src/vite-env.d.ts` |
| **US-0022-S4** — UI stamp + stale detection | F3, F4, F5 | `frontend/src/components/AppLayout.tsx`, `frontend/src/hooks/useStaleDetection.ts`, `frontend/src/components/StaleBanner.tsx` |
| **US-0022-S5** — Tests + docs | G1, R1, V1 | cargo/npm test, `docs/user-guides/US-0022.md` |

**Out of scope:** Periodic polling for stale detection (GATE-STALE-1); Service Worker integration; changes to `/health` or `/health/ready`; release-management UI; Grafana metadata panel; semver auto-bump.

## Task table

| ID | Title | Slice | Est. | Acceptance |
|----|-------|-------|------|------------|
| B1 | Backend `meta` module + route registration | S1 | 3h | AC-3 |
| B2 | Dockerfile `ARG`/`ENV`/`LABEL` chain | S2 | 2h | AC-4 |
| F1 | Vite `define` block injection | S3 | 1h | AC-4 |
| F2 | TypeScript declarations (`vite-env.d.ts`) | S3 | 0.5h | AC-4 |
| F3 | `AppLayout` sidebar-footer stamp + tooltip | S4 | 3h | AC-1, AC-2 |
| F4 | `useStaleDetection` hook | S4 | 2h | AC-5 |
| F5 | `StaleBanner` component | S4 | 2h | AC-5 |
| T1 | Integration test — meta endpoint shape | S1 | 1.5h | AC-3 |
| G1 | Automated gate — cargo test + npm test + build | S5 | 1h | all |
| R1 | User guide US-0022 | S5 | 2h | — |
| V1 | Verify-work AC-1..AC-6 + OIDC smoke | S5 | 2h | AC-6 |

**Total estimate:** ~20h across 11 mandatory tasks (all P0).

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Secrets in metadata response | Allowlist fields only; `option_env!()` never echoes `.env` | AC-6, B1 |
| Backend-only deploy (no frontend rebuild) | Stale banner explains "New version available — reload"; expected behavior | AC-5, F4 |
| Traefik/browser cache on meta endpoint | `cache: 'no-store'` header; operator hard refresh hint in tooltip | AC-5, F4 |
| Docker `ARG` scope confusion (not re-declared) | Document pattern; test in CI; `option_env!()` fallback to `"dev"` | B2 |
| Local dev without `--build-arg` | `option_env!()` returns `"dev"`; stale detection skips dev mode | B1, F4 |
| Compile-time `env!()` breaks local dev | Use `option_env!().unwrap_or("dev")` — never breaks build | B1 |
| OIDC deploy regression | **V1** smoke on `/sync` or `/` | AC-6, V1 |

## Definition of Done

- All 11 sprint tasks complete (`B1`, `B2`, `F1`, `F2`, `F3`, `F4`, `F5`, `T1`, `G1`, `R1`, `V1`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` § US-0022 (AC-1..AC-6)
- `GET /api/v1/meta/build-info` returns `{build_id, release_tag, build_timestamp}` with no secrets (AC-3)
- Frontend bundle embeds same build id at compile time via Vite `define` + Dockerfile `ARG` (AC-4)
- Subtle version stamp visible in `AppLayout` sidebar footer; hover reveals release tag + build id + build timestamp (UTC) (AC-1, AC-2)
- On-mount stale detection: SPA build id ≠ server → non-blocking banner + reload CTA; no false positive when ids match (AC-5)
- `/health` liveness unchanged (`{status: ok}`); OIDC external profile smoke pass; no env secrets in metadata (AC-6)
- `docs/user-guides/US-0022.md` published (`USER_GUIDE_MODE=1`)

## Architecture references

- `docs/engineering/architecture.md` § US-0022
- Research: [R-0095](docs/engineering/research.md#r-0095--us-0022-deploy-version-stamp--stale-frontend-detection) §6–§12
- No new DEC (GATE-DEC-1 closed; all gates are implementation-level)
- Spec-pack: `docs/engineering/spec-pack/US-0022-{design-concept,crs,technical-specification}.md`
- User guide: `docs/user-guides/US-0022.md`
- Discovery: `handoffs/po_to_tl.md#discovery-20260614-us0022`
- Acceptance: `docs/product/acceptance.md` § US-0022
- Handoff: `handoffs/tl_to_dev.md` (`sprint-plan-20260614-us0022-s0021`)

## Sequencing (frozen)

```text
S1: B1 → T1
S2: B2 (parallel with S1)
S3: F1 → F2 (after B2)
S4: F3 (after F2); F4 → F5 (after F3)
S5: G1 after B1+T1+F1+F2+F3+F4+F5; R1 after F3+F5; V1 after G1+R1
Operator: BACKEND_FRONTEND_DEPLOY → verify-work omniflow smoke (UAT)
```

## Acceptance coverage map

| Row | Tasks | Notes |
|-----|-------|-------|
| AC-1 | F3, G1, V1 | Subtle stamp in sidebar footer; does not dominate primary UX |
| AC-2 | F3, V1 | Hover tooltip: release tag + build id + build timestamp (UTC) |
| AC-3 | B1, T1, G1, V1 | `GET /api/v1/meta/build-info` returns authoritative metadata; no secrets |
| AC-4 | B2, F1, F2, G1 | Docker `ARG` chain + Vite `define` inject build id at compile time |
| AC-5 | F4, F5, V1 | On-mount stale detection: mismatch → banner + reload CTA; no false positive |
| AC-6 | B1, G1, V1 | `/health` unchanged; OIDC smoke pass; no secrets in metadata |

## Split decision

- **Why 11 tasks:** Architecture B1 + B2 + F1 + F2 + F3 + F4 + F5 + T1 + G1 + R1 + V1 = 11 < `SPRINT_MAX_TASKS` 12.
- **Why not split S0021a/b:** Single deploy-observability vertical slice; backend metadata + frontend stamp + stale detection are tightly coupled (operator cannot see value without both).
- **No P1/P2 stretch:** All tasks are P0 (mandatory for operator value).
- **User guide in R1:** `USER_GUIDE_MODE=1` — separate task avoids bundling with V1.

## Next phase

**PLANNED** — next **plan-verify** (qa) → **execute** (dev) → **qa** → **verify-work** → **release** (`0.22.0-us0022`). Operator: **BACKEND_FRONTEND_DEPLOY** → verify-work omniflow smoke (UAT).
