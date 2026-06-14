# PO to TL archive pack (2026-06-14)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=650, PO_TO_TL_HOT_MAX_SECTIONS=60`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 8
- Retained units in hot file: 60
- First archived heading: `## Research summary`
- Last archived heading: `## Isolation evidence (DEC-0038)`
- Verification tuple (mandatory):
  - archived_body_lines=61
  - retained_body_lines=506

---

## Research summary

[R-0095 §6-§12](docs/engineering/research.md#r-0095--us-0022-deploy-version-stamp--stale-frontend-detection) extended with technical patterns (backend meta endpoint, Vite define, stale detection, Docker ARG chain). Four gates frozen; risk low; sprint sizing hint ~8-10 tasks under `SPRINT_MAX_TASKS=12`.

## Frozen gates

| Gate | Decision | Rationale |
|------|----------|-----------|
| **GATE-META-1** | Dedicated `/api/v1/meta/build-info` | Kubernetes /health vs /ready pattern; keep liveness thin |
| **GATE-BUILD-1** | `BUILD_ID` (git short sha) + `RELEASE_TAG` + `BUILD_TIMESTAMP` (UTC ISO 8601) | Standard CI pattern; `git rev-parse --short HEAD` |
| **GATE-STALE-1** | On-mount fetch only (no periodic poll) | Operator tool; on-mount sufficient; Sentry PR #98031 pattern |
| **GATE-UI-1** | `AppLayout` sidebar-footer | Subtle by default; hover/focus for details; natural location |

## Technical patterns validated

| Layer | Pattern | Key decision |
|-------|---------|--------------|
| **Backend** | Axum `Json<BuildInfoResponse>` handler | `option_env!("BUILD_ID").unwrap_or("dev")` compile-time; public route (no auth); flat JSON `{build_id, release_tag, build_timestamp}` |
| **Frontend** | Vite `define` block | `JSON.stringify(process.env.BUILD_ID \|\| 'dev')`; TypeScript `declare const __BUILD_ID__: string;` in `vite-env.d.ts` |
| **Stale detect** | `useStaleDetection()` hook | On-mount fetch `/api/v1/meta/build-info` with `cache: no-store`; compare `__BUILD_ID__` to server `build_id`; skip dev mode |
| **Docker** | 3-stage `ARG` chain | Global `ARG BUILD_ID/RELEASE_TAG/BUILD_TIMESTAMP`; re-declare per stage; `ENV` in builder for Rust `env!()`; `RUN BUILD_ID=$BUILD_ID npm run build` in frontend |

## Risk analysis

**Overall risk:** Low. All risks have clear mitigations.

| Risk | Mitigation |
|------|------------|
| Secrets in metadata | Allowlist fields only; `option_env!()` never echoes `.env` |
| Backend-only deploy | Stale banner explains "New version available — reload" |
| Traefik/browser cache | `cache: no-store` header; hard refresh hint in tooltip |
| Docker ARG scope | Document pattern; `option_env!()` fallback to `"dev"` |
| Local dev without `--build-arg` | `option_env!()` returns `"dev"`; stale detection skips dev mode |

## Acceptance rows (unchanged intent)

- **AC-1** (subtle stamp): `AppLayout` sidebar-footer; default minimal
- **AC-2** (hover details): tooltip release tag + build id + build timestamp (UTC)
- **AC-3** (backend metadata): dedicated `GET /api/v1/meta/build-info`; returns `{build_id, release_tag, build_timestamp}`; no secrets
- **AC-4** (SPA embed): Vite `define` block: `__BUILD_ID__`, `__RELEASE_TAG__`; Dockerfile `ARG BUILD_ID` + `ARG RELEASE_TAG`
- **AC-5** (stale detection): on app mount: fetch `/api/v1/meta/build-info`, compare `__BUILD_ID__` to server `build_id`; mismatch → non-blocking banner + reload CTA
- **AC-6** (regression): `/health` liveness unchanged; OIDC external profile smoke pass; metadata responses contain no env secrets

## Sprint sizing hint

~8-10 tasks (backend meta module, Dockerfile ARG chain, Vite define, TS declarations, AppLayout stamp, stale detection hook, stale banner component, integration test, UAT). Under `SPRINT_MAX_TASKS=12`; no split needed.

## Recommended next phase

`/architecture` (tech-lead role) — freeze implementation contract (backend meta endpoint, Vite define, Dockerfile ARG, AppLayout stamp, stale detection logic).

## Isolation evidence (DEC-0038)

- **Role:** tech-lead (research phase)
- **Fresh context:** yes (no prior chat history; artifact/handoff reads + web research only)
- **Phase boundary:** research complete; stop; hand off to /architecture in new subagent/chat
- **No architecture/sprint-plan executed:** confirmed (research only per DEC-0038)
- **No host secrets read:** confirmed (web research + codebase audit only)

---

