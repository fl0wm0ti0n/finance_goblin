# Verify-work Findings — Quick Q0024 / BUG-0016

**Work item:** BUG-0016 (defect)  
**Quick task:** Q0024  
**Phase:** `/verify-work`  
**Orchestrator:** `intake-20260609-ui-audit`  
**Date:** 2026-06-09  
**Decisions:** DEC-0104, DEC-0057  
**Verify-work agent:** fresh subagent (`verify-work-20260609-bug0016-qa-fresh`)  
**Verdict:** **PASS** — row **AX** satisfied at code/test level; V1 runtime probes pass-with-prerequisites per BUG-0013/0014/0015 precedent; proceed to `/release`

## Summary

Verify-work populated UAT artifacts from QA PASS evidence (`sprints/quick/Q0024/qa-findings.md`, `handoffs/dev_to_qa.md`). Independent re-run confirms **213/213** lib tests, **5/5** SPA fallback integration tests, and **9/9** frontend vitest. Acceptance row **AX** (DEC-0104 SPA `index.html` fallback, DEC-0057 route ordering) passes at code/test level. V1 curl matrix on `:18080`, omniflow browser hard-refresh/bookmarks, and OIDC `/callback` live smoke recorded as **pass-with-prerequisites** pending **BACKEND_FRONTEND_DEPLOY**. Zero blocking findings.

## Per-row verdict (acceptance AX)

| Row | Verdict | Summary |
|-----|---------|---------|
| **AX** | **pass** | AX1 `ServeDir::fallback(ServeFile)` per DEC-0104; AX2 integration 5/5. Live AX-CURL-1/2, AX-BROWSER-1/2, OIDC-1 **pass-with-prerequisites** — pre-Q0024 deploy on :18080 returns 404 for deep links; omniflow auth barrier. |

## Operator gate

| Gate | Status |
|------|--------|
| Code (AX1–AX2) | **CLEARED** |
| `cargo test --lib` | **CLEARED** — 213/213 PASS |
| `cargo test --test spa_fallback_integration` | **CLEARED** — 5/5 PASS |
| `npm test -- --run` | **CLEARED** — 9/9 PASS |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** |
| V1 curl matrix on `:18080` | **PENDING** — pass-with-prerequisites (pre-deploy 404) |
| V1 omniflow browser smoke | **PENDING** — pass-with-prerequisites (401 auth barrier) |
| OIDC `/callback` live regression | **PENDING** — pass-with-prerequisites |

## Automated verification (verify-work independent re-run)

| Check | Result |
|-------|--------|
| `cd backend && cargo test --lib` | **PASS** (213/213) |
| `cargo test --test spa_fallback_integration` | **PASS** (5/5) |
| `cd frontend && npm test -- --run` | **PASS** (9/9) |
| localhost:18080 `/health` | **PASS** — HTTP 200 (stack reachable) |
| localhost:18080 deep-link curl matrix | **pass_with_prerequisites** — `/forecast` etc. HTTP 404 (AX1 not deployed) |
| Omniflow reachability | **pass_with_prerequisites** — root `/forecast` 401; `/api/v1/health` 404 |

### Test output (lib suite)

```
test result: ok. 213 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Test output (SPA fallback integration)

```
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Code contract verification

| Contract | Evidence |
|----------|----------|
| **DEC-0104** — `ServeDir::fallback(ServeFile::new(index.html))` HTTP 200 | `lib.rs` `attach_spa_fallback`; `deep_links_return_200_html_shell` |
| **DEC-0057** — health → Grafana → API → SPA merge order | `build_router` lines 209–212; integration tests |
| **AX2** — deep links 200 HTML | `/forecast`, `/subscriptions`, `/planning`, `/sync`, `/analytics/cashflow`, `/callback` |
| **AX2** — protected prefixes non-HTML | `/health` JSON, `/api/v1/*` JSON, Grafana proxy, `/assets/*` static |
| Frozen boundaries — no Traefik/backend `/callback` change | Code review per qa-findings T-7 |

## Acceptance impact

| Row | Verify-work | `acceptance.md` checkbox |
|-----|-------------|--------------------------|
| **AX** | **PASS** | Release phase |

## Release gate

| Gate | Status |
|------|--------|
| QA PASS (dev handoff) | yes |
| Verify-work PASS | **yes** |
| UAT populated (DEC-0009) | **yes** — `uat.json`, `uat.md` |
| Acceptance AX | **PASS** (code) + runtime prerequisites documented |
| Isolation evidence (verify-work) | **yes** |
| Release proceed | **yes** |

## Operator advisory (non-blocking)

1. **BACKEND_FRONTEND_DEPLOY:** Rebuild `flow-finance-ai` with Q0024 AX1 SPA fallback.
2. **AX-CURL-1/2:** Curl deep-link matrix on `:18080` — expect HTTP 200 + `text/html` SPA shell.
3. **AX-CURL-3:** Confirm `/health` JSON, `/api/v1/*` JSON responses, static `/assets/*` unchanged.
4. **AX-BROWSER-1/2:** Hard-refresh and bookmark reopen on `financegnome.omniflow.cc`.
5. **OIDC-1:** Complete OIDC login; verify `/callback` serves SPA shell and session establishes.
6. **Reopen criteria:** Deep links still 404 after rebuild → reopen execute; API/Grafana paths return HTML → DEC-0057 regression.

## Artifacts

- `sprints/quick/Q0024/uat.json`
- `sprints/quick/Q0024/uat.md`
- `sprints/quick/Q0024/qa-findings.md`
- `handoffs/dev_to_qa.md`
- `handoffs/verify_work_to_release.md`

## Next steps

1. **`/release`** — check BUG-0016 acceptance AX; publish release notes; update backlog → DONE

---

**Stop here.** Continue in a **new** subagent/chat with **`/release`**.
