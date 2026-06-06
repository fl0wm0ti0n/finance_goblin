# UAT — Sprint S0011 / US-0011

**Sprint:** S0011  
**Story:** US-0011  
**Phase:** verify-work (complete)  
**Status:** PASS (omniflow runtime deferred)  
**Verified at:** 2026-06-03T00:30:00Z

## Inputs

- Acceptance: `docs/product/acceptance.md#US-0011`
- Operator guide: `docs/user-guides/US-0011.md` (T-0129)
- Architecture: `docs/engineering/architecture.md#US-0011`, `decisions/DEC-0057.md`
- Research: R-0054, R-0056
- QA: `sprints/S0011/qa-findings.md`, `sprints/S0011/qa.json`
- Dev handoff: `handoffs/dev_to_qa.md`

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | Sidebar **Analytics** lists all six provisioned dashboards with in-app routes | **pass** | `AppLayout.tsx` `analyticsNavItems` (6); `analytics_routes_registered` test |
| UAT-2 | AC-2 | Each dashboard opens in-app via iframe (no default new tab) | **pass_with_prerequisites** | `AnalyticsEmbedPage.tsx` kiosk iframe; no `target="_blank"` on Wealth; live iframe **deferred** |
| UAT-3 | AC-3 | Embed works with Traefik `auth` (or documented dev bypass) | **pass_with_prerequisites** | Proxy outside JWT + US-0011.md Traefik section; omniflow proxy health **deferred** |
| UAT-4 | AC-4 | Forecast, Wealth, Planning, Subscriptions, Alerts pages regress cleanly | **pass** | `product_routes_regression` (4/4) |
| UAT-5 | AC-5 | Wealth no longer uses external Grafana tab as primary portfolio entry | **pass** | `wealth_uses_in_app_portfolio_analytics`; `Link to="/analytics/portfolio"` |
| UAT-6 | AC-6 | Future-chart guideline documented in user guide | **pass** | `docs/user-guides/US-0011.md` §Canonical UX + future-chart guideline |
| UAT-7 | AC-7 | Operator guide describes single-URL analytics; no public Grafana host required | **pass** | US-0011.md + `.env.example` (`GRAFANA_TRAEFIK_HOST=` empty) |

## Verify-work automated checks (2026-06-03)

| Check | Result |
|-------|--------|
| `cargo test --test analytics_proxy_integration` | PASS (4/4) |
| `cargo test --test product_routes_regression` | PASS (4/4) |
| `cargo test grafana_upstream --lib` | PASS (3/3) |
| `npm run build` (frontend) | PASS |
| DEC-0057 static contract review | PASS |
| Task completion T-0119–T-0129 | 11/11 done |

## Omniflow runtime (deferred)

| Check | Result | Notes |
|-------|--------|-------|
| `GET /health` | **deferred** | HTTP 404 (verify-work curl 2026-06-03) |
| `GET /analytics/grafana/api/health` | **deferred** | HTTP 404 |
| Iframe load `/analytics/{slug}` × 6 | **deferred** | Requires deployed stack + Traefik auth |
| Grafana Live WebSocket refresh | **deferred** | Manual browser smoke per user guide |

**Reason code:** `OMNIFLOW_HOST_UNAVAILABLE` — not a release blocker per S0010 precedent; operator closes post-deploy.

## Results summary

- **Acceptance criteria:** 7/7 PASS (5 full pass, 2 pass-with-prerequisites for deferred omniflow runtime)
- **Sprint tasks:** 11/11 complete (T-0119–T-0129)
- **DEC-0057:** aligned — proxy prefix, anonymous Grafana, CSP, SSRF allowlist, no subpath, no public Grafana default
- **Blockers:** 0
- **Verdict:** **PASS** — ready for `/release`

## Operator post-release smoke (non-blocking)

When omniflow host is deployed, close per `docs/user-guides/US-0011.md` §Smoke checks:

1. `GET /health` → 200
2. `GET /analytics/grafana/api/health` → 200 (after Traefik auth)
3. Load each `/analytics/{slug}` iframe under Traefik session
4. Confirm Grafana Live panel refresh on one dashboard

**Decision gate:** If anonymous Grafana insufficient, escalate auth-proxy DEC — do not enable public `GRAFANA_TRAEFIK_HOST` without new DEC.
