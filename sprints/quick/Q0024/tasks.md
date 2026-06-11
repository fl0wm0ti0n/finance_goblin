# Tasks — Q0024 (BUG-0016)

**Bug:** BUG-0016  
**Task count:** 3 (all P0 mandatory; < `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260609-q0024-bug0016`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **AX1** | Task **AX1** | DEC-0104 `ServeDir::fallback(ServeFile::new(index.html))` in `build_router` |
| **AX2** | Task **AX2** | Integration: deep links 200 HTML; API/Grafana/assets non-HTML |
| **V1** | Task **V1** | verify-work AX curl matrix + browser smoke + OIDC `/callback` |

## Execute order

```text
AX1 (lib.rs build_router SPA fallback)
  → AX2 (integration tests)
  → single backend release deploy
  → operator: BACKEND_FRONTEND_DEPLOY
  → V1 verify-work
```

**Parallelism:** None — AX2 depends on AX1; V1 blocked on deploy.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **AX** | AX1, AX2, V1 | `/forecast`, `/subscriptions`, `/planning`, `/sync`, `/analytics/{slug}` return 200 HTML shell on `:18080` and omniflow; hard-refresh and bookmarks work; OIDC `/callback` completes |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| AX1 | SPA `index.html` fallback in `build_router` | 2h | done | **AX** | P0 |
| AX2 | Integration tests — deep links + protected prefixes | 2h | done | **AX** | P0 |
| V1 | verify-work curl + browser smoke | 1.5h | open | **AX** | P0 |

---

## AX1 — SPA `index.html` fallback in `build_router`

**Status:** done  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0016 **AX** — **DEC-0104**

### Description

Replace plain `ServeDir` fallback with `ServeDir::fallback(ServeFile::new(index.html))` returning **HTTP 200** for missing non-API paths. Preserve merge order per **DEC-0057**: health → Grafana proxy → API → SPA.

```rust
use tower_http::services::{ServeDir, ServeFile};

let index = static_dir.join("index.html");
let spa = ServeDir::new(static_dir).fallback(ServeFile::new(index));
router = router.fallback_service(spa);
```

Apply identically to prod (`/app/static`) and local dev (`frontend/dist`) branches. Do **not** use `not_found_service` (404 status fails AX curl gate). Do **not** add backend `/callback` redirect.

**Files:** `backend/src/lib.rs`

### Done when

- [ ] `ServeDir::fallback(ServeFile::new(index.html))` wired for both static-dir branches
- [ ] Route order unchanged: `/health`, `/analytics/grafana/*`, `/api/v1/*` before SPA fallback
- [ ] `/callback` serves SPA shell (React `OidcCallback` handles token exchange)
- [ ] `cargo build` PASS

---

## AX2 — Integration tests — deep links + protected prefixes

**Status:** done  
**Depends on:** AX1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0016 **AX** — **DEC-0104**

### Description

Add `build_router` integration tests per R-0086 §6 and architecture regression matrix:

**Deep links (200 HTML):**

| Path | Assert |
|------|--------|
| `GET /forecast` | 200, `text/html`, body contains `#root` or Vite shell marker |
| `GET /subscriptions` | 200 HTML shell |
| `GET /planning` | 200 HTML shell |
| `GET /sync` | 200 HTML shell |
| `GET /analytics/cashflow` | 200 HTML shell |

**Protected prefixes (must not return SPA index):**

| Path | Assert |
|------|--------|
| `GET /api/v1/health` (or representative API) | JSON, not HTML |
| `GET /api/v1/nonexistent` | JSON 404, not `index.html` |
| `GET /assets/{fixture}.js` (when present) | Static file with correct `Content-Type` |

Grafana proxy path (`/analytics/grafana/*`) may be smoke-tested in V1 if test harness lacks upstream; document in test module if deferred.

**Files:** `backend/tests/` (new SPA fallback test module) or `backend/src/lib.rs` `#[cfg(test)]`

### Done when

- [ ] Primary AX paths assert 200 + HTML shell
- [ ] API paths assert non-HTML response
- [ ] `cargo test` SPA fallback paths PASS

---

## V1 — verify-work curl + browser smoke

**Status:** open  
**Depends on:** AX1, AX2 + deploy  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0016 **AX**

### Description

Prepare `sprints/quick/Q0024/uat.md` smoke checklist. After deploy:

**Curl matrix (`:18080`, `AUTH_DEV_BYPASS`):**

- Primary AX paths: `/forecast`, `/subscriptions`, `/planning`, `/sync`, `/analytics/cashflow`
- Expanded same-contract: `/wealth`, `/alerts`, `/chat`, `/settings`, `/analytics/{platform-health,budgets,portfolio,subscriptions,forecast-horizons}`
- Protected: `/api/v1/health`, `/api/v1/nonexistent`, `/assets/*`

**Browser smoke (operator omniflow):**

- Hard-refresh Forecast, Planning, Analytics embed routes
- Bookmark reopen
- Traefik `auth` + optional OIDC
- `/callback?code=…&state=…` completes without backend redirect

### Done when

- [ ] Row **AX** probed per acceptance.md curl + browser matrix
- [ ] `uat.md` and `uat.json` populated with results
- [ ] Operator gate documented: **BACKEND_FRONTEND_DEPLOY** before omniflow probes

**Operator gate:** **BACKEND_FRONTEND_DEPLOY** — rebuild `flow-finance-ai` image before runtime probes.
