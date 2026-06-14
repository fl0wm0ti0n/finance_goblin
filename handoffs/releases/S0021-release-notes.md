# Release Notes — S0021 / US-0022

**Sprint:** S0021
**Story:** US-0022 — Deploy version stamp & stale-frontend detection
**Date:** 2026-06-14
**Release version:** `0.22.0-us0022`
**Backlog status:** DONE
**Acceptance:** checked (`docs/product/acceptance.md` US-0022 rows AC-1..AC-6; live AC-5/AC-6 operator-deferred BACKEND_FRONTEND_DEPLOY)
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --lib` (221/221); `cargo test --test meta_test` (3/3); `npm test` (31/31); `npm run build` PASS (709 modules, 13.24s); `sprints/S0021/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/S0021/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS-WITH-PREREQUISITES — `sprints/S0021/uat.json`, `sprints/S0021/uat.md`, `sprints/S0021/verify-work-findings.md`; 8 steps — 6 pass, 2 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260614-us0022-001`; release tuple at finalization
6. **Doc gates:**
   - **3e Legacy drift:** PASS — no drift detected for US-0022
   - **3f README feature coverage:** PASS (advisory) — `coverage_missing: []`; framework template parity advisory (non-blocking; no story coverage gaps)
   - **3g Project README coverage:** PASS — `validate_project_readme_coverage.py --enforce` exit 0
   - **User guide (US-0032):** PASS — `docs/user-guides/US-0022.md` present with required sections
7. **Release finalization gate:** PASS

---

## Summary

Deploy version stamp and stale-frontend detection per architecture § US-0022 — five gates frozen (GATE-META-1 dedicated public route; GATE-BUILD-1 Docker ARG chain; GATE-STALE-1 on-mount only; GATE-UI-1 sidebar-footer placement; GATE-DEC-1 no new DEC).

| Scope | Implementation |
|-------|---------------|
| **B1** | `backend/src/meta/mod.rs` (new) — `GET /api/v1/meta/build-info` returns `{build_id, release_tag, build_timestamp}` via `option_env!()` with safe fallbacks |
| **B2** | `backend/Dockerfile` — ARG/ENV/LABEL chain (global L3-5, builder L10-12, frontend L29-36, runtime L42-44, OCI LABELs L46-48) |
| **F1** | `frontend/vite.config.ts` — `define` block injects `__BUILD_ID__` and `__RELEASE_TAG__` at compile time |
| **F2** | `frontend/src/vite-env.d.ts` — TypeScript declarations for build constants |
| **F3** | `frontend/src/components/AppLayout.tsx` — sidebar-footer monospace stamp + hover tooltip with release/build/timestamp |
| **F4** | `frontend/src/hooks/useStaleDetection.ts` (new) — on-mount fetch `/api/v1/meta/build-info`, compare SPA vs server build id |
| **F5** | `frontend/src/components/StaleBanner.tsx` (new) — non-blocking stale warning with reload CTA |
| **T1** | `backend/tests/meta_test.rs` (new) — 3 integration tests (shape, no-secrets, fallback values) |
| **G1** | cargo lib 221/221; meta_test 3/3; npm 31/31; build PASS |
| **R1** | `docs/user-guides/US-0022.md` (new) — operator user guide |
| **V1** | verify-work AC-1..AC-6 — pass-with-prerequisites; deferred **BACKEND_FRONTEND_DEPLOY** |

**Code proof:** meta_test 3/3 (shape, no-secrets, fallback); cargo lib 221/221; npm 31/31; build PASS (709 modules).

**Operator post-release:** Rebuild backend + frontend (**BACKEND_FRONTEND_DEPLOY**) → verify sidebar-footer stamp visible → hover tooltip shows release tag + build id → `GET /api/v1/meta/build-info` returns matching metadata → stale detection triggers when SPA build id ≠ backend.

---

## Run

**Target service:** `flow-finance-ai` (backend + frontend — metadata endpoint + SPA stamp + stale detection).

**Deploy (backend + frontend rebuild — no migration):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

- `start_command`: docker compose commands above
- `runtime_mode`: local (`:18080`) and remote (omniflow external US-0010)
- `runtime_context_ref`: `docs/engineering/runbook.md`

**Build args (Docker ARG chain propagates to all stages):**

| ARG | Source | Fallback |
|-----|--------|----------|
| `BUILD_ID` | git short sha or CI pipeline | `"dev"` |
| `RELEASE_TAG` | release label (e.g. `0.22.0-us0022`) | `"dev"` |
| `BUILD_TIMESTAMP` | UTC ISO timestamp at build | `"unknown"` |

---

## Connect

- `service_url`: `http://localhost:18080` (local) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 18080 (local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET http://localhost:18080/health` → JSON 200
- `metadata_endpoint`: `GET http://localhost:18080/api/v1/meta/build-info` → JSON 200 (public, no auth)
- Version stamp: sidebar footer (visible after login)
- Stale banner: above content area (only when build id mismatch detected)

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | External PostgreSQL — mirror + sync cursors |
| `AUTHENTIK_SECRET_KEY` | External compose profile build gate (set dummy for local external profile) |
| OIDC provider config | Omniflow OIDC-1 regression — Compose/env only |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(AC-1)** | Sidebar footer stamp visible | Subtle monospace 7-char build id fragment; low visual noise |
| **(AC-2)** | Hover tooltip on stamp | Shows Release tag + Build id + Timestamp |
| **(AC-3)** | `GET /api/v1/meta/build-info` | 200 JSON `{build_id, release_tag, build_timestamp}`; no secrets |
| **(AC-4)** | Frontend bundle embed | `__BUILD_ID__` matches Docker build arg after rebuild |
| **(AC-5)** | Stale detection | After deploy with new build id, old SPA shows non-blocking banner with reload CTA |
| **(AC-6)** | Regression | `/health` unchanged; no secrets in metadata; OIDC smoke pass |

**Automated (release):**

```bash
cd backend && cargo test --lib && cargo test --test meta_test
cd frontend && npm test && npm run build
```

**Live (operator):** UAT steps in `sprints/S0021/uat.json`.

---

## Credentials

- `DATABASE_URL` — external PostgreSQL (operator shell only — no inline secrets)
- OIDC provider config via Compose/env only
- No secrets exposed in `/api/v1/meta/build-info` (verified by `test_meta_build_info_no_secrets`)

---

## Changes

| Area | Summary |
|------|---------|
| `backend/src/meta/mod.rs` | B1 — new metadata endpoint with `option_env!()` compile-time injection |
| `backend/src/lib.rs` | mod declaration + `build_router` merge (public route before `api_router`) |
| `backend/Dockerfile` | B2 — ARG/ENV/LABEL chain for BUILD_ID, RELEASE_TAG, BUILD_TIMESTAMP |
| `backend/tests/meta_test.rs` | T1 — 3 integration tests (shape, no-secrets, fallback) |
| `frontend/vite.config.ts` | F1 — `define` block for compile-time constant injection |
| `frontend/src/vite-env.d.ts` | F2 — TypeScript declarations |
| `frontend/src/components/AppLayout.tsx` | F3 — sidebar-footer stamp + hover tooltip + StaleBanner integration |
| `frontend/src/hooks/useStaleDetection.ts` | F4 — on-mount stale detection hook (new) |
| `frontend/src/components/StaleBanner.tsx` | F5 — non-blocking stale warning banner (new) |
| `docs/user-guides/US-0022.md` | R1 — operator user guide (new) |

**Architecture gates:** GATE-META-1 (dedicated route), GATE-BUILD-1 (ARG chain), GATE-STALE-1 (on-mount only), GATE-UI-1 (sidebar-footer), GATE-DEC-1 (no new DEC)
**Research fulfilled:** R-0095
**Deferred:** AC-5/AC-6 live browser/API smoke until **BACKEND_FRONTEND_DEPLOY**

---

## Known Issues

- Tooltip timestamp uses client-side `Date.now()`, not build timestamp (advisory — release tag + build id sufficient for deploy verification; build timestamp available via `/api/v1/meta/build-info`)
- StaleBanner not dismissible (advisory — reload solves stale state; no close button)
- Live stale-detection browser smoke deferred pending **BACKEND_FRONTEND_DEPLOY**
- OIDC external profile smoke deferred pending **BACKEND_FRONTEND_DEPLOY**

---

## Regression scope

- Firefly sync window (BUG-0025 / Q0034) unchanged
- Plan delete hint (BUG-0024 / Q0033) unchanged
- Forecast Income card (BUG-0026 / Q0032) unchanged
- Subscription transaction explorer (US-0021 / S0020) unchanged
- `/health` liveness endpoint unchanged (AC-6 regression verified)

---

## Rollback

```bash
git revert <S0021-code-commits>
docker compose up -d --build flow-finance-ai
```

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0021 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.22.0-us0022`

## Milestone

**US-0022 released** — backend metadata endpoint + Docker ARG chain + SPA compile-time stamp + on-mount stale detection; code/integration oracles PASS; deploy operator-deferred per pass-with-prerequisites.
