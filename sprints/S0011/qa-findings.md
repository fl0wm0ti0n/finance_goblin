# QA Findings — Sprint S0011 / US-0011

**Sprint:** S0011  
**Story:** US-0011  
**QA phase:** `/qa`  
**Date:** 2026-06-02  
**Verdict:** **PASS** (ready for `/verify-work`; omniflow iframe/runtime deferred)

## Scope

Unified analytics per DEC-0057: same-origin Grafana reverse proxy (`/analytics/grafana/`), six kiosk iframe routes, Analytics sidebar, Wealth migration to in-app portfolio, compose anonymous Grafana env, CSP `frame-src 'self'`, env contract, proxy/regression tests, operator guide.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0011/summary.md`, `sprints/S0011/uat.md`, `docs/product/acceptance.md` (US-0011), `decisions/DEC-0057.md`, `docs/user-guides/US-0011.md`, `backend/src/analytics/proxy.rs`, `backend/src/lib.rs`, `frontend/src/pages/AnalyticsEmbedPage.tsx`, `frontend/src/components/AppLayout.tsx`, `frontend/src/pages/WealthPage.tsx`, `docker-compose.yml`, `.env.example`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Analytics proxy integration | `cargo test --test analytics_proxy_integration` | **PASS** (4/4) |
| T-2 | Product routes + analytics regression | `cargo test --test product_routes_regression` | **PASS** (4/4) |
| T-3 | Grafana upstream SSRF allowlist | `cargo test grafana_upstream --lib` | **PASS** (3/3) |
| T-4 | Frontend production build | `npm run build` (frontend) | **PASS** |
| T-5 | DEC-0057 contract review | Static code/config review | **PASS** (see matrix) |
| T-6 | Operator guide AC-6/AC-7 | Static review `docs/user-guides/US-0011.md` | **PASS** |
| T-7 | Omniflow iframe + Traefik auth (AC-2/AC-3) | `curl` smoke to `https://financegnome.omniflow.cc` | **DEFERRED** — host returns HTTP 404 for `/health`, `/`, `/analytics/grafana/api/health` |
| T-8 | Grafana Live WebSocket panel refresh | Manual browser smoke | **DEFERRED** — requires deployed stack + live Grafana |
| T-9 | Full harness `tests/run-tests.sh` | Not re-run (US-0011 scoped tests sufficient; compose-config-check env-dependent per dev handoff) | **Advisory** — dev noted `DATABASE_HOST` external merge assert failure unrelated to US-0011 |

### Environment dependencies (non-blocking)

- **Omniflow host:** Required for live iframe load, Traefik `auth` session reuse, Grafana health through proxy, WebSocket `/api/live/` smoke.
- **`DATABASE_URL`:** Not required for US-0011 QA scope (no new integration DB tests).

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | Analytics sidebar lists six dashboards with in-app routes | **PASS** | `AppLayout.tsx` `analyticsNavItems` (6); `AnalyticsEmbedPage.tsx` slug map; `analytics_routes_registered` test |
| AC-2 | Each dashboard opens in-app via iframe (no default new tab) | **PASS** (static) | `AnalyticsEmbedPage` iframe `src` kiosk pattern; no `target="_blank"` on Wealth; live iframe **DEFERRED** omniflow |
| AC-3 | Embed works with Traefik `auth` or documented dev bypass | **PASS** (static/docs) | Proxy outside `/api/v1` JWT; user guide §Traefik auth; anonymous Grafana compose env. Live Traefik **DEFERRED** (omniflow 404) |
| AC-4 | ECharts product pages regression | **PASS** | `product_routes_registered_in_app`, `echarts_pages_still_import_chart_surfaces` |
| AC-5 | Wealth uses in-app portfolio analytics | **PASS** | `wealth_uses_in_app_portfolio_analytics`; `Link to="/analytics/portfolio"`; no `VITE_GRAFANA_URL` |
| AC-6 | Future-chart guideline documented | **PASS** | `docs/user-guides/US-0011.md` §Canonical UX + future-chart guideline |
| AC-7 | Single-URL operator guide; no public Grafana host required | **PASS** | User guide §Single-URL UX; `.env.example` `GRAFANA_TRAEFIK_HOST=` empty; embed base `/analytics/grafana` |

**Summary:** 7/7 PASS on static/automated path; AC-2/AC-3 live runtime deferred with `OMNIFLOW_HOST_UNAVAILABLE` (not a QA blocker per S0010 precedent).

## DEC-0057 compliance review

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Proxy prefix | `/analytics/grafana/` | `proxy.rs` `PREFIX`, nested router | PASS |
| Upstream env | `GRAFANA_UPSTREAM` default `http://grafana:3000` | `docker-compose.yml`, `default.toml`, allowlist | PASS |
| SPA embed base | `VITE_GRAFANA_EMBED_BASE` `/analytics/grafana` | `.env.example`, `AnalyticsEmbedPage.tsx` | PASS |
| Six slugs / uids | Discovery map | `DASHBOARDS` + sidebar + tests | PASS |
| Deprecate `VITE_GRAFANA_URL` | No Wealth external tab | Regression test + Wealth `Link` | PASS |
| Anonymous Grafana | GF_AUTH_ANONYMOUS + Viewer + ALLOW_EMBEDDING | `docker-compose.yml` lines 70–72 | PASS |
| No subpath serve | No `GF_SERVER_SERVE_FROM_SUB_PATH` | Repo grep — absent | PASS |
| No public Grafana default | `GRAFANA_TRAEFIK_HOST` empty | `.env.example`; external overlay gated | PASS |
| Proxy outside JWT | Separate `analytics_router` merge | `build_router` before `api_router` | PASS |
| Framing / cookies | Strip `X-Frame-Options`, `Set-Cookie` | `filter_response_headers` + integration tests | PASS |
| WebSocket live | Upgrade on `/api/live/` | `proxy_entry` + `websocket_live_route_registered` | PASS (unit); live smoke deferred |
| CSP | `frame-src 'self'` | `frontend/index.html` | PASS |
| SSRF guard | Host allowlist | `validate_grafana_upstream` + 3 unit tests | PASS |

## Generated baseline test evidence

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` + `typescript` |
| `generated_test_command` | `cargo test --test analytics_proxy_integration --test product_routes_regression && cargo test grafana_upstream --lib && cd frontend && npm run build` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-06-02 — proxy 4/4, regression 4/4, allowlist 3/3, frontend build exit 0 |
| `generated_test_paths_ref` | `backend/tests/analytics_proxy_integration.rs`, `backend/tests/product_routes_regression.rs`, `backend/src/config/mod.rs`, `frontend/` |
| `generated_test_reason_code` | — |

## Runtime QA evidence

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed (`docker compose … up` on omniflow host) |
| `runtime_stack_profile` | `docker-compose` external + analytics proxy |
| `runtime_mode` | `deferred` |
| `runtime_health_target` | `https://financegnome.omniflow.cc/health`, `/analytics/grafana/api/health`, iframe load under Traefik `auth`, Grafana Live WS |
| `runtime_health_result` | `deferred` |
| `runtime_final_verdict` | `deferred` (verify-work / operator) |
| `runtime_reason_code` | `OMNIFLOW_HOST_UNAVAILABLE` |
| `runtime_evidence_refs` | QA curl 2026-06-02: health/root/grafana-proxy all HTTP 404; `docs/user-guides/US-0011.md` §Smoke checks; `handoffs/dev_to_qa.md` |

## Findings

### Blockers

None.

### Advisories (non-blocking)

1. **Omniflow runtime:** Close iframe + Traefik auth + Grafana Live smoke on Debian host when stack is deployed (verify-work or operator).
2. **compose-config-check:** Dev handoff reports env-dependent `DATABASE_HOST` failure — not US-0011 regression; re-run on CI/host with `DATABASE_HOST=postgres`.
3. **Decision gate:** If anonymous Grafana insufficient on omniflow, escalate auth-proxy DEC per DEC-0057 — do not enable public `GRAFANA_TRAEFIK_HOST` without new DEC.

## Verdict

**PASS** — proceed to `/verify-work` in fresh subagent. No `handoffs/qa_to_dev.md`.
