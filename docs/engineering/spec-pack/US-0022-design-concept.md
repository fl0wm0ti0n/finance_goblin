# Design Concept — US-0022

## Summary

US-0022 introduces deploy-observability into Flow Finance AI: a compile-time version stamp baked into both backend binary and frontend SPA, a dedicated metadata endpoint, a subtle UI stamp in the `AppLayout` sidebar footer, and on-mount stale-bundle detection with a non-blocking reload banner. Operator pain originates from BUG-0023 deploy confusion (post-deploy uncertainty without `docker inspect`).

## Goals

- **AC-1:** Subtle version/build indicator in `AppLayout` sidebar footer — minimal default (short label)
- **AC-2:** Hover/focus tooltip reveals release tag, build id (git short sha), build timestamp (UTC)
- **AC-3:** Dedicated `GET /api/v1/meta/build-info` returns authoritative `{build_id, release_tag, build_timestamp}` — no secrets
- **AC-4:** Frontend bundle embeds same build id at compile time via Vite `define` + Dockerfile `ARG`
- **AC-5:** On-mount stale detection: SPA build id ≠ server → non-blocking banner + reload CTA; no false positive when ids match
- **AC-6:** `/health` liveness unchanged; OIDC external profile smoke pass; no env secrets in metadata

## Non-goals

- Full release-management UI; semver auto-bump
- Periodic polling for stale detection (on-mount only per GATE-STALE-1)
- Exposing `.env` secrets, PATs, or exchange keys in metadata
- Service Worker integration (no SW in this project)
- Changes to `/health` or `/health/ready` endpoints

## Key decisions

| Gate | Choice | Rationale |
|------|--------|-----------|
| GATE-META-1 | Dedicated `/api/v1/meta/build-info` | Kubernetes /health vs /ready pattern; keep liveness thin |
| GATE-BUILD-1 | `BUILD_ID` (git short sha) + `RELEASE_TAG` + `BUILD_TIMESTAMP` (UTC ISO 8601) | Standard CI pattern; `git rev-parse --short HEAD` |
| GATE-STALE-1 | On-mount fetch only (no periodic poll) | Operator tool; on-mount sufficient; Sentry PR #98031 pattern |
| GATE-UI-1 | `AppLayout` sidebar-footer | Subtle by default; hover/focus for details; natural location |
| GATE-DEC-1 | No new DEC | All gates are implementation-level; no canonical policy change |

## Spec-pack traceability

- `docs/engineering/spec-pack/US-0022-crs.md`
- `docs/engineering/spec-pack/US-0022-technical-specification.md`
- `docs/engineering/architecture.md` § US-0022
- [R-0095](docs/engineering/research.md#r-0095--us-0022-deploy-version-stamp--stale-frontend-detection) §6–§12
