# Tasks — Sprint S0011

**Story:** US-0011  
**Task count:** 11 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0119 | GRAFANA_UPSTREAM AppConfig and allowlist | done | AC-2, AC-3 |
| T-0120 | Analytics proxy route /analytics/grafana/ | done | AC-2, AC-3 |
| T-0121 | Grafana anonymous embed compose env | done | AC-2, AC-3 |
| T-0122 | .env.example embed env contract | done | AC-7 |
| T-0123 | AnalyticsEmbedPage and six dashboard routes | done | AC-1, AC-2 |
| T-0124 | AppLayout Analytics nav group | done | AC-1 |
| T-0125 | WealthPage deprecate external Grafana link | done | AC-5 |
| T-0126 | SPA CSP frame-src self | done | AC-2, AC-3 |
| T-0127 | Proxy and WebSocket integration tests | done | AC-2 |
| T-0128 | ECharts product page regression tests | done | AC-4 |
| T-0129 | Operator user guide US-0011 | done | AC-6, AC-7 |

---

## T-0119 — GRAFANA_UPSTREAM AppConfig and allowlist

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0057, R-0056

### Description

Add `GRAFANA_UPSTREAM` to backend `AppConfig` (default `http://grafana:3000`). Validate upstream host at startup:

- Allowlist: `grafana`, `localhost`, `127.0.0.1` (dev); reject arbitrary external hosts (SSRF guard per DEC-0057)
- Parse URL scheme (`http`/`https` only) and port
- Wire env var through compose `flow-finance-ai` service environment

Pass validated upstream URL to analytics proxy module (T-0120).

### Done when

- [ ] `GRAFANA_UPSTREAM` loads from env with documented default
- [ ] Invalid or non-allowlisted host fails fast at startup with actionable error
- [ ] Compose passes `GRAFANA_UPSTREAM` to backend container

---

## T-0120 — Analytics proxy route /analytics/grafana/

**Status:** open  
**Depends on:** T-0119  
**Decisions:** DEC-0057, R-0056 §3–§4

### Description

Implement backend analytics proxy module:

| Behavior | Requirement |
|----------|-------------|
| Mount path | `/analytics/grafana/` (and `/analytics/grafana/*`) |
| Upstream join | Strip prefix; forward to `{GRAFANA_UPSTREAM}/` |
| Methods | GET, POST, PUT, DELETE, PATCH, OPTIONS |
| WebSocket | Forward `Upgrade` / `Connection` for Grafana Live (`/api/live/`) |
| Framing | Strip or replace `X-Frame-Options: deny` on proxied responses |
| Cookies | Do not forward Grafana `Set-Cookie` to browser |
| Router order | Merge in `build_router` **before** SPA static fallback |
| Auth | **Outside** `/api/v1` JWT middleware — edge Traefik auth is the gate |

Use existing HTTP client patterns in backend; no `/api/v1/analytics/*` namespace.

### Done when

- [ ] `GET /analytics/grafana/` (or known static asset path) returns upstream 200 in Docker profile
- [ ] Prefix strip verified — upstream receives root paths
- [ ] WebSocket upgrade forwarded (smoke in T-0127)
- [ ] Proxied responses omit blocking `X-Frame-Options: deny`
- [ ] Route not behind `/api/v1` JWT stack

---

## T-0121 — Grafana anonymous embed compose env

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0057, R-0056 §1

### Description

Add to `grafana` service in `docker-compose.yml`:

```yaml
environment:
  GF_AUTH_ANONYMOUS_ENABLED: "true"
  GF_AUTH_ANONYMOUS_ORG_ROLE: Viewer
  GF_SECURITY_ALLOW_EMBEDDING: "true"
  # GF_USERS_ALLOW_SIGN_UP: "false" — retain if already present
```

Do **not** enable `GF_SERVER_SERVE_FROM_SUB_PATH`. Do **not** add default public Traefik router for embed acceptance path (DEC-0056).

Document that anonymous Viewer is acceptable because Grafana is internal-only behind proxy + Traefik boundary.

### Done when

- [ ] Grafana service env block includes anonymous Viewer + allow_embedding
- [ ] `GF_USERS_ALLOW_SIGN_UP` remains false
- [ ] No `GF_SERVER_SERVE_FROM_SUB_PATH` added
- [ ] Merged compose config renders expected Grafana env

---

## T-0122 — .env.example embed env contract

**Status:** open  
**Depends on:** T-0119, T-0121  
**Decisions:** DEC-0057

### Description

Update `.env.example`:

| Variable | Purpose |
|----------|---------|
| `GRAFANA_UPSTREAM=http://grafana:3000` | Backend proxy upstream (Docker default) |
| `VITE_GRAFANA_EMBED_BASE=/analytics/grafana` | SPA iframe base (build-time) |
| Deprecate `VITE_GRAFANA_URL` | Comment as deprecated; Wealth no longer uses external tab |

Add dev note: local non-Docker may use `GRAFANA_UPSTREAM=http://localhost:3000` when Grafana published on host port.

No literal secrets.

### Done when

- [ ] `GRAFANA_UPSTREAM` and `VITE_GRAFANA_EMBED_BASE` documented with defaults
- [ ] `VITE_GRAFANA_URL` marked deprecated with migration pointer
- [ ] Dev vs Docker upstream examples present

---

## T-0123 — AnalyticsEmbedPage and six dashboard routes

**Status:** open  
**Depends on:** T-0122  
**Decisions:** DEC-0057

### Description

Create `AnalyticsEmbedPage` (or equivalent) rendering full-viewport kiosk iframe:

```tsx
src={`${embedBase}/d/${uid}/${slug}?kiosk=tv`}
```

Register six routes in `App.tsx` per discovery map:

| slug | uid |
|------|-----|
| `platform-health` | `platform-health` |
| `cashflow` | `cashflow` |
| `subscriptions` | `subscriptions` |
| `budgets` | `budgets` |
| `portfolio` | `portfolio` |
| `forecast-horizons` | `forecast-horizons` |

Use `VITE_GRAFANA_EMBED_BASE` at build time (default `/analytics/grafana`). No `target="_blank"` on default navigation.

### Done when

- [ ] Six `/analytics/{slug}` routes render iframe shell
- [ ] Iframe `src` uses embed base + kiosk query per route map
- [ ] Page titles match dashboard names
- [ ] Frontend build succeeds with new routes

---

## T-0124 — AppLayout Analytics nav group

**Status:** open  
**Depends on:** T-0123  
**Decisions:** DEC-0057

### Description

Add **Analytics** sidebar section in `AppLayout` with six in-app links:

- Platform Health → `/analytics/platform-health`
- Cashflow → `/analytics/cashflow`
- Subscriptions → `/analytics/subscriptions`
- Budgets → `/analytics/budgets`
- Portfolio → `/analytics/portfolio`
- Forecast Horizons → `/analytics/forecast-horizons`

Match existing nav styling conventions. All links use in-app routing (no external Grafana URL).

### Done when

- [ ] Analytics nav group visible in sidebar
- [ ] Six links route to correct `/analytics/{slug}` paths
- [ ] Active route highlighting works
- [ ] No default `target="_blank"` Grafana links in nav

---

## T-0125 — WealthPage deprecate external Grafana link

**Status:** open  
**Depends on:** T-0123  
**Decisions:** DEC-0057

### Description

Update `WealthPage` portfolio analytics entry:

- Replace external `VITE_GRAFANA_URL` new-tab link with in-app navigation to `/analytics/portfolio`
- Remove default `target="_blank"` for primary portfolio Grafana view
- Preserve ECharts product charts as authoritative interactive surface (DEC-0057 UX table)

Optional: secondary text link “SQL view” → `/analytics/portfolio` if card already exists.

### Done when

- [ ] Primary portfolio Grafana entry is in-app `/analytics/portfolio`
- [ ] No dependency on `VITE_GRAFANA_URL` for default Wealth flow
- [ ] ECharts wealth charts unchanged

---

## T-0126 — SPA CSP frame-src self

**Status:** open  
**Depends on:** T-0120  
**Decisions:** DEC-0057, R-0056 §2

### Description

Ensure SPA Content-Security-Policy allows same-origin iframe embed:

- Set or extend `frame-src 'self'` (or equivalent) on financegnome HTML/responses
- Do not add third-party Grafana host to CSP in default build
- Complement proxy `X-Frame-Options` rewrite (T-0120) — both layers required per R-0056

Locate CSP in existing frontend build or backend static middleware — match project convention.

### Done when

- [ ] CSP includes `frame-src 'self'` (or policy equivalent for same-origin iframes)
- [ ] No third-party Grafana origin in default CSP
- [ ] Documented in code comment referencing R-0056 / DEC-0057 if non-obvious

---

## T-0127 — Proxy and WebSocket integration tests

**Status:** open  
**Depends on:** T-0120, T-0121  
**Decisions:** DEC-0057, R-0056 §4

### Description

Add automated tests (integration or smoke):

1. **Proxy reachability** — `GET /analytics/grafana/` or known Grafana static asset returns 200 (mock upstream or testcontainers Grafana stub acceptable)
2. **Prefix strip** — upstream receives path without `/analytics/grafana` prefix
3. **Framing** — proxied response does not include blocking `X-Frame-Options: deny`
4. **WebSocket** — upgrade request forwarded (minimal handshake test or documented manual smoke template if full WS E2E impractical in CI)

Wire into `tests/run-tests.sh` harness where applicable.

### Done when

- [ ] At least one proxy smoke test passes in CI harness
- [ ] Framing header assertion included
- [ ] WebSocket forward covered by test or documented smoke step for QA
- [ ] Tests run without requiring public Grafana host

---

## T-0128 — ECharts product page regression tests

**Status:** open  
**Depends on:** T-0123, T-0124, T-0125  
**Decisions:** DEC-0057

### Description

Add or extend frontend/backend regression coverage ensuring existing product chart pages remain functional:

| Route | Check |
|-------|-------|
| `/forecast` | Page renders; primary ECharts surface loads |
| `/wealth` | Page renders; ECharts + migrated portfolio link |
| `/planning` | Page renders |
| `/subscriptions` | Page renders |
| `/alerts` | Page renders |

Prefer existing test patterns (Vitest component smoke or route snapshot). No requirement to E2E Grafana panels on these pages.

### Done when

- [ ] Regression tests cover five product routes
- [ ] Tests pass in `tests/run-tests.sh` / frontend test suite
- [ ] No accidental removal of ECharts primary surfaces

---

## T-0129 — Operator user guide US-0011

**Status:** open  
**Depends on:** T-0122, T-0124, T-0125  
**Decisions:** DEC-0057, USER_GUIDE_MODE=1

### Description

Create `docs/user-guides/US-0011.md`:

- **Single-URL UX** — operators use `https://financegnome.omniflow.cc` only; Grafana not required on public host (AC-7)
- **Analytics sidebar** — six in-app dashboards
- **Env vars** — `GRAFANA_UPSTREAM`, `VITE_GRAFANA_EMBED_BASE`; deprecated `VITE_GRAFANA_URL`
- **Future-chart guideline** (AC-6) — new product charts default to React + API; Grafana embed exception list
- **Canonical UX table** — ECharts primary vs Grafana secondary per product page (from DEC-0057)
- **Troubleshooting** — iframe blank (proxy/CSP), WebSocket stale panels, dev `localhost:3000` upstream, Traefik auth + iframe behavior, DEC-0057 decision gate if anonymous insufficient

Cross-link US-0010 omniflow deploy guide where relevant.

### Done when

- [ ] User guide covers all seven acceptance criteria from operator/user perspective
- [ ] Future-chart guideline and canonical UX table present
- [ ] Single-URL analytics documented; public Grafana host optional only
- [ ] Troubleshooting for proxy/CSP/auth paths included

---

## Execution order (recommended)

1. **Config foundation:** T-0119 ∥ T-0121
2. **Proxy:** T-0120 (after T-0119)
3. **Env template:** T-0122 (after T-0119, T-0121)
4. **Frontend shell:** T-0123 (after T-0122) → T-0124 → T-0125
5. **Security:** T-0126 (after T-0120)
6. **Tests:** T-0127 (after T-0120, T-0121) → T-0128 (after frontend tasks)
7. **Docs:** T-0129 (last)

## Acceptance coverage map

| AC | Tasks |
|----|-------|
| AC1 Analytics sidebar lists all dashboards | T-0124 |
| AC2 In-app open (no default new tab) | T-0120, T-0123, T-0126 |
| AC3 Traefik auth / dev bypass embed works | T-0120, T-0121, T-0126, T-0127 |
| AC4 ECharts pages regression | T-0128 |
| AC5 Wealth no external Grafana primary | T-0125 |
| AC6 Future-chart guideline | T-0129 |
| AC7 Single-URL operator guide | T-0122, T-0129 |

## Split decision

- **Why 11 tasks:** Maps DEC-0057 execute outline plus explicit CSP (R-0056 §2) and split regression vs proxy tests; within SPRINT_MAX_TASKS=12.
- **Why not S0011a/b:** Proxy, compose env, and iframe shell share same deploy contract; splitting proxy from Grafana env would allow broken embed between sprints.
- **Optional deferred:** Secondary ECharts → `/analytics/*` cross-links on Forecast/Planning/Subscriptions — not in sprint scope.
