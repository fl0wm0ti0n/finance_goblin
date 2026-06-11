# Q0024 — BUG-0016 SPA deep links return HTTP 404

| Field | Value |
|-------|-------|
| **ID** | Q0024 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0016 |
| **Created** | 2026-06-09 |
| **Architecture** | `architecture-20260609-bug0016` (`docs/engineering/architecture.md` § BUG-0016) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260609-q0024-bug0016`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0016 row **AX** |
| **Task count** | 3 (all P0 mandatory) |
| **Next phase** | `/execute` |

## Goal

Close BUG-0016 SPA deep-link 404: **DEC-0104** Axum `ServeDir::fallback(ServeFile::new(index.html))` returning HTTP 200 in `build_router` (**AX1**), integration tests for deep links and protected prefixes (**AX2**), operator verify-work curl + browser smoke (**V1**) on `:18080` and `financegnome.omniflow.cc` after backend deploy.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| SPA fallback (DEC-0104) | AX1 | `backend/src/lib.rs` `build_router` |
| Regression tests | AX2 | `backend/tests/` or `lib` integration |
| Verify | V1 | uat + curl matrix + browser smoke |

**Ops-only (not execute tasks):** Operator rebuild `flow-finance-ai` image on omniflow.

**Out of scope:** Traefik label changes, React route edits, backend `/callback` redirect handler, Grafana panel data (BUG-0009 **Y**), `not_found_service` (404 status fails AX).

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| AX1 | SPA `index.html` fallback in `build_router` | 2h | — | **AX** | P0 |
| AX2 | Integration tests — deep links + protected prefixes | 2h | AX1 | **AX** | P0 |
| V1 | verify-work curl + browser smoke | 1.5h | AX1, AX2 + deploy | **AX** | P0 |

**Total estimate:** ~5.5h (4h dev + ~1.5h operator V1).

## Deploy order

```text
AX1 → AX2 → single backend release
  → operator: BACKEND_FRONTEND_DEPLOY (rebuild flow-finance-ai)
  → V1 verify-work (curl :18080 + omniflow hard-refresh + OIDC /callback)
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **AX** | AX1, AX2, V1 | Direct nav, hard-refresh, bookmarks return HTTP 200 SPA shell on `:18080` and omniflow; OIDC `/callback` regression |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| AX1 | Task **AX1** |
| AX2 | Task **AX2** |
| V1 | Task **V1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
