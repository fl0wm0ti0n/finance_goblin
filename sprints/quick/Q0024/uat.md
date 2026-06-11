# UAT — Q0024 (BUG-0016)

**Status:** POPULATED — verify-work complete 2026-06-09  
**Acceptance:** `docs/product/acceptance.md` — BUG-0016 row **AX**  
**Sprint:** Q0024 (`/quick`)  
**Verdict:** **PASS** — code/test complete; runtime probes pass-with-prerequisites (BUG-0013/0014/0015 precedent)  
**Next phase:** `/release`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Result |
|-----|---------|-----------------------------------|--------|
| **AX** | AX1, AX2, V1 | Direct navigation, hard-refresh, and bookmarks to `/forecast`, `/subscriptions`, `/planning`, `/sync`, `/analytics/{slug}` return **HTTP 200** with SPA shell — not **404** blank body. Applies on `:18080` and `financegnome.omniflow.cc`. OIDC-enabled deploy regression checks pass. | **pass** (code) / **pass_with_prerequisites** (live) |

## Operator gates (before live omniflow probes)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild `flow-finance-ai` with Q0024 AX1 SPA fallback. — **PENDING**

## UAT steps (verify-work results)

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| AX-CODE | AX | AX1 `ServeDir::fallback(ServeFile)` in `build_router` | **pass** | `lib.rs` attach_spa_fallback; DEC-0104; qa-findings T-4 |
| AX-TEST | AX | AX2 integration tests — deep links 200; API non-HTML | **pass** | `spa_fallback_integration.rs` 5/5; VW-AUTO-2 |
| AX-CURL-1 | AX | `curl :18080/forecast` → 200 HTML shell | **pass_with_prerequisites** | Pre-deploy: HTTP 404; integration test proves 200 contract |
| AX-CURL-2 | AX | Primary AX paths curl matrix | **pass_with_prerequisites** | All deep-link paths 404 on :18080 pre-deploy |
| AX-CURL-3 | AX | Protected prefixes — API JSON, assets static | **pass** | Integration tests: health, API, Grafana proxy, assets |
| AX-BROWSER-1 | AX | Hard-refresh Forecast, Planning, Analytics on omniflow | **pass_with_prerequisites** | Omniflow 401 auth barrier; deploy pending |
| AX-BROWSER-2 | AX | Bookmark reopen client routes | **pass_with_prerequisites** | Browser smoke deferred post-deploy |
| OIDC-1 | regression | `/callback?code=…&state=…` completes — SPA shell only | **pass_with_prerequisites** | Integration test covers /callback; live OIDC deferred |

## Automated verification (verify-work independent re-run)

| Check | Result |
|-------|--------|
| `cd backend && cargo test --lib` | **PASS** (213/213) |
| `cargo test --test spa_fallback_integration` | **PASS** (5/5) |
| `cd frontend && npm test -- --run` | **PASS** (9/9) |
| localhost:18080 `/health` | **PASS** — HTTP 200 |
| localhost:18080 deep-link curl matrix | **pass_with_prerequisites** — `/forecast` etc. HTTP 404 (pre-Q0024 deploy) |
| Omniflow reachability | **pass_with_prerequisites** — root `/forecast` 401; `/api/v1/health` 404 |

### Curl probe output (localhost:18080, verify-work 2026-06-09)

```
localhost:18080/health → HTTP 200
localhost:18080/forecast → HTTP 404
localhost:18080/subscriptions → HTTP 404
localhost:18080/planning → HTTP 404
localhost:18080/sync → HTTP 404
localhost:18080/analytics/cashflow → HTTP 404
localhost:18080/callback → HTTP 404
```

### Omniflow probe output

```
omniflow root → HTTP 401
omniflow /forecast → HTTP 401
omniflow /api/v1/health → HTTP 404
```

## Results summary

- **Verdict:** PASS — 3/8 UAT steps pass (code), 5 pass-with-prerequisites (runtime/ops), 0 fail
- **Acceptance rows:** AX **pass** (code); live curl/browser/OIDC smoke deferred to operator
- **Blocking:** none
- **Traceability:** BUG-0016 row **AX** mapped in `sprints/quick/Q0024/uat.json`. Checkbox updates in `docs/product/acceptance.md` are **release** phase.

**Operator advisory:** After **BACKEND_FRONTEND_DEPLOY**, execute the 7-step smoke checklist in `uat.json` `operator_smoke_checklist` on `http://localhost:18080` and `https://financegnome.omniflow.cc`.
