# Backlog

## Bug issues (canonical)

### BUG-0001 — Omniflow production regressions (auth + Grafana analytics)

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0007, 2026-06-04

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; Traefik `auth` middleware on UI; `AUTH_DEV_BYPASS=true`; OIDC env vars unset (no IdP configured). Operator report 2026-06-03.

**steps_to_reproduce:**

1. **Defect A (auth):** Deploy external profile with OIDC unset (post-Q0005). Open `https://financegnome.omniflow.cc`, pass Traefik basic-auth, open browser console, click header **AI** or **Chat** nav.
2. **Defect B (Grafana):** From same session, open each Analytics route: `/analytics/platform-health`, `/analytics/cashflow`, `/analytics/subscriptions`, `/analytics/budgets`, `/analytics/portfolio`, `/analytics/forecast-horizons`. Inspect Network tab for static asset requests.

**expected:**

- **A:** App shell, sidebar nav, and Chat/AI usable without OIDC when dev bypass is enabled; no `useAuth()` calls outside `AuthProvider`.
- **B:** Grafana dashboards embed successfully; static assets resolve under same-origin prefix `/analytics/grafana/public/...` per DEC-0057 / US-0011 (not site-root `/public/...`).

**actual:**

- **A:** Console: `AuthProvider context is undefined, please verify you are calling useAuth() as child of a <AuthProvider>`; `TypeError: Cannot read properties of undefined (reading 'user')` on AI/Chat interaction; nav may be affected.
- **B:** Grafana displays "Grafana has failed to load its application files"; browser requests `https://financegnome.omniflow.cc/public/build/...` and `/public/img/...` → **404** (should be `/analytics/grafana/public/...`).

**evidence_refs:** `sprints/quick/Q0005/summary.md`, `decisions/DEC-0057.md`, `docs/engineering/research.md#r-0056`, operator report 2026-06-03, `handoffs/intake_evidence/intake-20260604-omniflow-prod-regressions.json`

#### Known code cause (intake notes)

- **A:** Q0005 made `AuthProvider` conditional in `frontend/src/main.tsx` when `!isOidcConfigured`, but `AppLayout.tsx`, `ChatPanel.tsx` still call `useAuth()` unconditionally; `App.tsx` `ProtectedRoute` guard was split — other consumers were not.
- **B:** iframe `src` uses `/analytics/grafana/d/...` but Grafana HTML emits absolute asset paths from site root `/public/...` without subpath awareness; may require `GF_SERVER_ROOT_URL` and/or proxy path rewrite. DEC-0057 rejected `GF_SERVER_SERVE_FROM_SUB_PATH` in sprint — intake captures as regression vs US-0011 omniflow smoke.

#### Out of scope

- `contentscript.js` / ObjectMultiplex — browser wallet extension console noise (not product defect)

#### Intake evidence (BUG-0001)

- `intake_run_id`: `intake-20260604-omniflow-prod-regressions`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260604-omniflow-prod-regressions.json`
- **Split decision:** single bug with sub-defects A+B (same operator session/environment); discovery may decompose fix tasks independently

#### Discovery refinements (2026-06-04)

**Sub-defect A — AuthProvider / useAuth (frontend)**

| Item | Detail |
|------|--------|
| Root cause | Q0005 made `AuthProvider` conditional in `main.tsx` when `!isOidcConfigured`, but `AppLayout.tsx:30` and `ChatPanel.tsx:16` still call `useAuth()` unconditionally. `App.tsx` `ProtectedRoute` split is correct; display-only `isOidcConfigured` guards in sidebar do not prevent hook execution. |
| Blast radius | `main.tsx`, `AppLayout.tsx`, `ChatPanel.tsx`, `AiSheet.tsx` (via `ChatPanel`). `OidcCallback.tsx` only on `/callback` when OIDC enabled — not in hot path. |
| Recommended fix | **Option A (PO):** Introduce `DevBypassAuthProvider` stub in `frontend/src/auth/` mounted when `!isOidcConfigured`; provides no-op `useAuth()` shape (`user: undefined`, `isAuthenticated: false`, no-op signin/signout). Single mount point in `main.tsx`. |
| Alternatives | **B:** Split `AppLayout` / `ChatPanel` into OIDC vs non-OIDC components (more files, same outcome). **C:** Revert Q0005 — always mount real `AuthProvider` (risk: `oidc-client-ts` init without authority). |
| OIDC regression guard | Stub mounts **only** when `!isOidcConfigured`; existing `OidcProtectedRoute` + `setAccessTokenProvider` unchanged. |
| Tests | Vitest: stub provider satisfies `ChatPanel` mount without warn. Smoke: external profile OIDC-unset — no console warn, AI/Chat opens. Regression: OIDC-configured build still redirects to IdP. |

**Sub-defect B — Grafana `/public/` 404 under iframe proxy (backend + compose)**

| Item | Detail |
|------|--------|
| Root cause | DEC-0057 Pattern A prefix-strip proxy forwards Grafana HTML unchanged. Grafana emits root-absolute asset URLs (`/public/build/…`, `/public/img/…`); browser resolves against site origin → 404. iframe `src` (`AnalyticsEmbedPage.tsx`) is correct; proxy does not rewrite HTML bodies. |
| Blast radius | `docker-compose.yml` Grafana env, `.env.example` operator override; optionally `backend/src/analytics/proxy.rs` if env-only fix insufficient. |
| Recommended fix | **Primary — Option A:** Add `GF_SERVER_ROOT_URL` to Grafana compose (default `https://financegnome.omniflow.cc/analytics/grafana/`; operator-overridable). Keep `GF_SERVER_SERVE_FROM_SUB_PATH=false` per DEC-0057 / R-0056 §3. **Fallback — Option B:** Proxy `text/html` body rewrite — prefix root-absolute `/public/` hrefs/src with `/analytics/grafana`. **Rejected without new DEC:** `GF_SERVER_SERVE_FROM_SUB_PATH=true` (DEC-0057 Option 3). |
| Tests | Extend `analytics_proxy_integration.rs`: mock HTML with `/public/build/` → assert prefixed URLs (if rewrite) or document env contract. Operator smoke: all six `/analytics/{slug}` routes; Network tab shows `/analytics/grafana/public/…` 200. curl HEAD on sample asset path. US-0011 deferred omniflow smoke becomes acceptance gate. |

**Fix-task decomposition (independent vertical slices):**

| Task | Sub | Owner layer | Blocks |
|------|-----|-------------|--------|
| A1 — Auth stub provider + consumer audit | A | frontend | — |
| B1 — `GF_SERVER_ROOT_URL` compose + `.env.example` | B | compose/docs | — |
| B2 — Proxy HTML rewrite (conditional) | B | backend | B1 smoke fail only |

**Discovery risks:**

| Risk | Mitigation |
|------|------------|
| Stub auth masks missing OIDC in production | Stub only when `!isOidcConfigured`; runbook warns OIDC for strict deploys |
| `GF_SERVER_ROOT_URL` host mismatch on non-omniflow deploys | Operator override via `.env`; document canonical US-0010 default |
| HTML rewrite breaks binary responses or Grafana upgrades | Env-first; rewrite scoped to `text/html` only if B1 insufficient |
| OIDC path regression | Acceptance includes OIDC-enabled deploy check |
| Grafana Live WS after root_url change | Existing proxy WS forwarding; operator smoke per `docs/user-guides/US-0011.md` |

**Research status:** R-0056 §3 resolves `GF_SERVER_ROOT_URL` vs prefix-strip — **no standalone `/research` phase** unless architecture proves env-only insufficient (then bounded B2 spike only).

**Recommended next phase:** `/architecture` (confirm auth stub contract + Grafana env/rewrite decision; map to quick sprint or S00xx bug-fix sprint).

---

### BUG-0002 — Omniflow production integration defects (Firefly sync + API 404 + exchange settings)

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0008, 2026-06-05

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; Traefik split routers (`financegnome-api` for `/api` without basic-auth, UI router with `auth`); `AUTH_DEV_BYPASS=true`; OIDC unset; post-BUG-0001 / Q0007 (2026-06-04). Operator requires `FIREFLY_BASE_URL=http://firefly:8080` and `FIREFLY_PERSONAL_ACCESS_TOKEN` in operator `.env` (names only — do not read secrets).

**steps_to_reproduce:**

1. **Defect C (Firefly sync):** Deploy external profile with Firefly PAT configured. Open `https://financegnome.omniflow.cc/sync`, pass Traefik basic-auth, inspect Sync Status and browser console; click manual sync / wait for scheduler.
2. **Defect D (API 404):** From same session (or curl via public host), `GET https://financegnome.omniflow.cc/api/v1/plans/risk-score` (Planning page also requests this endpoint).
3. **Defect E (exchange settings):** Configure only Bitunix credentials in operator `.env` (`BITUNIX_API_KEY` / `BITUNIX_API_SECRET`); open **Settings → Crypto exchanges** table.

**expected:**

- **C:** Firefly connector pulls accounts/transactions; Sync Status shows successful run and non-zero entity counts; no blocking 404 on sync API calls.
- **D:** `GET /api/v1/plans/risk-score` returns **200** with JSON risk score payload or documented empty-state (not **404**).
- **E:** Settings table shows **Bitunix** `enabled=yes` and `configured=yes` when only Bitunix env vars are set; Binance/Bybit reflect actual operator configuration (not default `enabled=true` for Binance when unset).

**actual:**

- **C:** Sync page / manual sync does not pull Firefly data; prior verify-work showed sync state `failed`; console reports `sync:1 Failed to load resource: 404` (SPA route or API path).
- **D:** `GET https://financegnome.omniflow.cc/api/v1/plans/risk-score` → **404** (route may exist in codebase — suspect Traefik routing, stale deploy, or handler registration gap on omniflow).
- **E:** Operator configured Bitunix (`BITUNIX_API_KEY`) but UI shows **Binance** enabled + configured, **Bitunix** no + configured — suggests `default.toml` `enabled` defaults (`binance.enabled=true`, `bitunix.enabled=false`) not aligned with env presence.

**evidence_refs:** operator report 2026-06-04, `handoffs/intake_evidence/intake-20260604-omniflow-prod-integration.json`, [R-0057](docs/engineering/research.md#r-0057--firefly-iii-api-docs-discovery-post-bug-0001), [R-0001](docs/engineering/research.md#r-0001--firefly-iii-rest-api-integration-baseline), [R-0032](docs/engineering/research.md#r-0032--exchange-connector-rest-patterns-binance-bybit-bitunix), `backend/src/api/plans.rs`, `backend/config/default.toml`, `docker-compose.external.yml`

#### Known code cause (intake notes)

- **C:** Firefly PAT/base URL must reach in-network `http://firefly:8080`; sync failure may be connector auth, DNS on `traefik` network, or frontend/API 404 on `/api/v1/sync/*` misrouted as SPA.
- **D:** Route registered in `backend/src/api/plans.rs` at `/api/v1/plans/risk-score` — production 404 likely deploy/routing mismatch (Traefik `financegnome-api` PathPrefix `/api`) or stale image without plans module merge.
- **E:** `ExchangeInstanceConfig::configured()` reads env; `enabled` flags are static TOML defaults — UI conflates `enabled` with “operator intent” when only credentials are set.

#### Out of scope

- `contentscript.js` / ObjectMultiplex — browser wallet extension console noise (not product defect)

#### Intake evidence (BUG-0002)

- `intake_run_id`: `intake-20260604-omniflow-prod-integration`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260604-omniflow-prod-integration.json`
- **Split decision:** single bug with sub-defects C+D+E (same operator session/environment post-BUG-0001); discovery may decompose fix tasks independently

#### Intake decomposition (2026-06-04)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 3 sub-defects (sync, API route, settings display) |
| Cross-cutting | backend sync/firefly, plans API, settings/config, Traefik external overlay |
| Acceptance breadth | 3 sub-rows C/D/E |
| Risk | Firefly connectivity on shared host; false-positive if deploy stale; exchange `enabled` semantics |

**Alternatives considered:** three separate BUG ids — rejected (duplicate env/evidence); fixes independently testable via acceptance sub-rows.

**Recommended next phase:** `/architecture` (confirm fix contracts for C1–C2, D1, E1; map to quick sprint or bug-fix sprint).

#### Discovery refinements (2026-06-04)

**Runtime proof (public host, names-only — no operator `.env` read):**

| Probe | Result |
|-------|--------|
| `GET /api/v1/sync/status` | **200** — `state: failed`, `error_message: unexpected status 401 Unauthorized` |
| `GET /api/v1/plans/risk-score` | **404** — empty body (handler `NOT_FOUND`, route registered) |
| `GET /api/v1/settings` exchanges | `binance: enabled=true configured=false`; `bitunix: enabled=false configured=true` |

**Sub-defect C — Firefly sync 401 / empty PAT**

| Item | Detail |
|------|--------|
| Root cause | **Confirmed:** Firefly connector receives empty/invalid PAT → `Authorization: Bearer ` → Firefly **401** → sync run `failed` (not missing `/api/v1/sync/*` route). Operator note: container `FIREFLY_PERSONAL_ACCESS_TOKEN` empty after recreate. Config loads empty string when env var is present but blank (`config/mod.rs` `set_override` without `!is_empty()` guard). Compose passes `${FIREFLY_PERSONAL_ACCESS_TOKEN:-}`. |
| Ruled out | Traefik misroute for sync APIs (status endpoint **200**). SPA `/sync` console label `sync:1` is misleading vs API 404 — triage on API responses + `last_run.error_message`. |
| Blast radius | Operator `.env` / compose `env_file`, `backend/src/config/mod.rs`, `backend/src/firefly/mod.rs`, optional `/health` PAT preflight, runbook § Omniflow PAT checks. |
| Recommended fix | **C1 (ops):** Non-empty PAT in operator `.env`; verify `printenv FIREFLY_PERSONAL_ACCESS_TOKEN` non-empty after recreate (runbook). **C2 (code):** Treat empty PAT as unset in env overlay; fail-fast or degraded health with explicit message when sync enabled and PAT missing. **C3 (UX):** Sync Status already surfaces `error_message` — ensure operator sees 401 text (no change if already visible). |
| Alternatives | **Reject:** proxy/HTML rewrite for sync (wrong layer). **Defer:** PAT generation inside app (out of scope). |
| Research | [R-0057](docs/engineering/research.md#r-0057--firefly-iii-api-docs-discovery-post-bug-0001) — Bearer PAT contract; **no standalone `/research`**. |

**Sub-defect D — `GET /api/v1/plans/risk-score` → 404**

| Item | Detail |
|------|--------|
| Root cause | **Confirmed:** Route exists (`plans.rs`); production **404** is application `StatusCode::NOT_FOUND` when `PlanRiskService::latest_for_active_plan()` returns `None` (no active plan + successful computation + persisted `plan_risk_scores` row). **Not** Traefik/stale-binary routing (sync API on same host returns 200). |
| Contributing factors | Firefly sync failed (C) → no fresh data; plan risk refresh runs only after successful full sync pipeline (`trigger_plan_risk_refresh`). |
| Blast radius | `backend/src/api/plans.rs` `risk_score`, `backend/src/plan/risk.rs`, `frontend/src/pages/PlanningPage.tsx` (expects 200 or handles empty). |
| Recommended fix | **D1 (API):** Return **200** with documented empty-state JSON when no score (align BUG-0002 acceptance) — e.g. `{ "status": "no_score", "reason": "no_active_plan" \| "not_computed" }`. **D2 (optional):** After C fixed, operator activates plan + successful sync to populate score; no API change beyond empty-state if no plan. |
| Alternatives | **Reject:** Traefik-only fix. **Reject:** rename route to `/api/v1/plan/risk-score` (client uses plural path). |
| Research | None required. |

**Sub-defect E — Exchange enabled vs configured**

| Item | Detail |
|------|--------|
| Root cause | **Confirmed:** `ExchangesConfig::settings_view()` exposes TOML `enabled` flags (`default.toml`: `binance.enabled=true`, `bitunix.enabled=false`) independently of `configured()` (env credential presence). Production API: Bitunix **configured=true, enabled=false**. Settings UI renders both columns verbatim (`SettingsPage.tsx`). `mirror_enabled_at_startup()` seeds DB `enabled` from TOML only. |
| Operator report delta | Live settings show Binance **enabled=yes, configured=no** (not enabled+configured); defect stands for Bitunix enabled mismatch. |
| Blast radius | `backend/config/default.toml`, `backend/src/config/mod.rs` (`settings_view`, defaults), `backend/src/exchanges/service.rs` (`mirror_enabled_at_startup`), Settings + `/api/v1/exchanges` list. |
| Recommended fix | **E1 (code):** **Effective enabled** = `configured() \|\| toml.enabled` for settings view and startup mirror (credentials imply operator intent). **E2 (optional):** Set `exchanges.binance.enabled=false` in `default.toml` to reduce greenfield false positives. |
| Alternatives | **B:** TOML-only — operator must edit `default.toml` (poor omniflow UX). **C:** UI-only override (API/exchange sync still wrong). PO recommends **E1**. |
| Research | [R-0032](docs/engineering/research.md#r-0032--exchange-connector-rest-patterns-binance-bybit-bitunix) baseline unchanged. |

**Fix-task decomposition (independent vertical slices):**

| Task | Sub | Owner layer | Blocks |
|------|-----|-------------|--------|
| C1 — Operator PAT + compose env verification | C | ops/docs | — |
| C2 — Empty PAT guard + health/sync preflight message | C | backend/config | — |
| D1 — Risk-score 200 empty-state contract | D | backend API + Planning UI | — |
| E1 — Effective `enabled` from `configured()` | E | backend config/exchanges | — |
| E2 — `default.toml` binance.enabled=false (optional) | E | config | E1 |

Tasks C2, D1, E1 are independently testable; **C1** must pass on omniflow before C acceptance smoke.

**Discovery risks:**

| Risk | Mitigation |
|------|------------|
| PAT set in `.env` but not loaded into container | Document compose cwd + `env_file`; C1 smoke `printenv` |
| Empty-state shape breaks Planning UI | Coordinate D1 response schema with frontend |
| Auto-enable exchange when creds present | E1 only affects display/DB enabled mirror; exchange sync still requires credentials |
| OIDC / bundled-firefly regression | Acceptance rows unchanged — explicit regression checks |

**Research status:** R-0057 sufficient for C; **skip standalone `/research`**.

**Recommended next phase:** `/architecture` (DEC/contracts for empty-state + effective enabled; quick sprint vs S00xx).

---

### BUG-0003 — Omniflow production API 500 cascade, Bitunix test, Grafana SQL

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0009, 2026-06-05

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; Traefik split routers (`financegnome-api` for `/api` and `/analytics`); `AUTH_DEV_BYPASS=true`; OIDC unset; post-BUG-0001 / Q0007; operator report 2026-06-05. Container `DATABASE_HOST=host.docker.internal` (operator `.env` overrides external overlay default `postgres`). Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. **Defect F (API 500 cascade):** Deploy external profile; open financegnome UI or curl public host. Observe many `GET /api/v1/*` return **500** after ~30s (e.g. alerts/unread-count, sync/entities, sync/runs, exchanges, forecast/*, subscriptions/*, ai/audit). Inspect `flow-finance-ai` logs for `response failed` **500** and ~30000ms latency.
2. **Defect G (Bitunix test 400):** With `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` set in operator env (names only), `POST https://financegnome.omniflow.cc/api/v1/exchanges/bitunix/test` → **400** empty body. Settings may show Bitunix configured via Q0008 E1 mirror while connector absent from in-memory registry.
3. **Defect H (Grafana analytics SQL 400):** Open Analytics routes; `POST .../analytics/grafana/api/ds/query` → **400** `error when executing the sql query`. Grafana datasource `grafana/provisioning/datasources/postgres.yaml` uses `${DATABASE_HOST}` — same wrong host if env wrong. Grafana logs may show duplicate dashboard UID warnings and provisioning write blocked.

**expected:**

- **F:** Product APIs return **200** (or documented empty-state) within normal latency; backend uses in-network `DATABASE_HOST=postgres` on traefik network per DEC-0056 / US-0010 / R-0052; settings API reports `database_host: postgres`, `database_mode: external`.
- **G:** Bitunix test registers connector when credentials present (`effective_enabled()` or equivalent); test returns **200** with connection result or explicit auth/URL error — not **400** unknown exchange.
- **H:** Grafana SQL panels query Flow DB via `postgres` host; ds/query returns **200** for provisioned dashboards.

**actual:**

- **F:** Widespread **500** on `/api/v1/*` with ~30s latency (DB query timeout pattern); startup may connect at boot but runtime queries hang; `DATABASE_HOST=host.docker.internal` in container vs overlay default `postgres`.
- **G:** `POST /api/v1/exchanges/bitunix/test` → **400**; `ExchangeService::new` registers Bitunix only if `config.bitunix.enabled` (TOML), not `effective_enabled()` — unknown exchange when connector not in list.
- **H:** Grafana ds/query **400** SQL execution error; datasource host follows wrong `DATABASE_HOST`; duplicate UID provisioning noise in logs.

**evidence_refs:** operator investigation 2026-06-05, `handoffs/intake_evidence/intake-20260605-omniflow-prod-api-500.json`, [R-0052](docs/engineering/research.md#r-0052--external-compose-integration-on-omniflow-traefik-host), [R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation), `docker-compose.external.yml`, `backend/src/exchanges/service.rs`, `grafana/provisioning/datasources/postgres.yaml`

#### Known code cause (intake notes)

- **F:** Operator `.env` sets `DATABASE_HOST=host.docker.internal`, overriding `docker-compose.external.yml` `${DATABASE_HOST:-postgres}`; pool targets unreachable host from traefik network → query timeout → **500**.
- **G:** Connector registration gate uses TOML `bitunix.enabled` in `ExchangeService::new`; Q0008 E1 may mirror DB `enabled` from credentials but leave connector out of runtime map → test handler returns unknown exchange. Implementation uses `openapi.bitunix.com` + query `sign`; futures docs specify `fapi.bitunix.com` + header auth (R-0058).
- **H:** Grafana Postgres datasource env-interpolates same `DATABASE_HOST`; wrong host breaks all SQL panels; duplicate dashboard UIDs may block provisioning updates (secondary).

#### Out of scope

- `contentscript.js` / ObjectMultiplex — browser wallet extension console noise (not product defect)

#### Intake evidence (BUG-0003)

- `intake_run_id`: `intake-20260605-omniflow-prod-api-500`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260605-omniflow-prod-api-500.json`
- **Split decision:** single bug with sub-defects F+G+H (same operator session/environment; shared `DATABASE_HOST` misconfiguration for F+H); discovery may decompose fix tasks independently

#### Intake decomposition (2026-06-05)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 3 sub-defects (DB host cascade, Bitunix test, Grafana SQL) |
| Cross-cutting | compose/env, backend DB pool, exchange service registry, Bitunix HTTP client, Grafana datasource provisioning |
| Acceptance breadth | 3 sub-rows F/G/H |
| Risk | F blocks all product data; G may be separate connector/auth fix after F; H may resolve with F alone |

**Alternatives considered:** three separate BUG ids — rejected (duplicate env/evidence); merge into BUG-0002 — rejected (distinct symptom cluster: API 500 cascade vs Firefly PAT / risk-score / settings display).

**Related work:** BUG-0002 DONE (Q0008 released 2026-06-05) — separate unless PO extends.

#### Discovery notes (2026-06-05)

| Sub | Confirmed root cause | Runtime evidence (no host `.env` read) | Fix task |
|-----|---------------------|----------------------------------------|----------|
| **F** | Operator `.env` sets `DATABASE_HOST=host.docker.internal`, overriding `docker-compose.external.yml` `${DATABASE_HOST:-postgres}`; backend pool cannot reach DB on `traefik` network → ~30s query timeout → **500** | Container env: `DATABASE_HOST=host.docker.internal` (names only). `GET /api/v1/settings` **200** reports `database_host: host.docker.internal`. Probes: `alerts/unread-count`, `sync/entities`, `sync/runs`, `exchanges`, `subscriptions`, `ai/audit` → **500** in **~30.06s** each. Logs: `tower_http::trace::on_failure` **500** `latency=30001 ms`. | **F1** Operator ops: set `DATABASE_HOST=postgres` for external profile; recreate `flow-finance-ai` + `grafana`. **F2** (architecture) runbook / `.env.example` guard — external block must not inherit dev default `host.docker.internal`. |
| **G** | `ExchangeService::new` registers `BitunixConnector` only when TOML `config.bitunix.enabled` (`default.toml` `enabled=false`), not `effective_enabled()`; credentials present → settings `bitunix.enabled=true` but connector absent → **400** before HTTP | `POST /api/v1/exchanges/bitunix/test` → **400** empty body in **&lt;0.2s** (not DB timeout). Settings: `bitunix.configured=true`, `enabled=true` (effective), `spot_base_url=openapi.bitunix.com`. Code: `service.rs` L40–48 vs `mirror_enabled_at_startup` L67–69. Secondary: spot query-sign vs futures header auth ([R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation)). | **G1** Register connectors with `effective_enabled()` parity. **G2** If test still fails after G1+F1: futures `fapi.bitunix.com` header-auth spike per R-0058. |
| **H** | Grafana Postgres datasource interpolates `${DATABASE_HOST}` — same wrong host as F; SQL panels fail fast with DB error (not 30s cascade). Duplicate dashboard UIDs block provisioning writes (secondary) | `POST .../analytics/grafana/api/ds/query` → **400** with `db query error` (fast). Container `DATABASE_HOST=host.docker.internal`. Logs: duplicate UID warnings (`budgets`, `cashflow`, `forecast-horizons`, …) and `dashboards provisioning provider has no database write permissions because of duplicates`. | **H1** Resolves with **F1** for datasource host. **H2** (optional) dedupe Grafana dashboard UIDs across providers if provisioning updates required. |

**Discovery verdict:** F confirmed P0 ops misconfiguration; G confirmed P0 code registry gap (+ deferred G2 auth); H primary = F1, H2 secondary provisioning hygiene.

**Recommended next phase:** `/plan-verify` on **`/quick` Q0009** (F1/F2/G1/G2-gated); operator F1 before verify-work.

---

### BUG-0004 — Post-sync pipeline empty analytics (stuck exchange sync, subscriptions, Grafana SQL)

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0011, 2026-06-05

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; post-BUG-0003 stack recovery (`DATABASE_HOST=postgres`, Firefly sync success, **922 transactions** loaded); operator report 2026-06-05. Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. **Defect I (stuck manual_exchanges):** With stack healthy and exchange credentials configured, trigger manual exchange sync from Sync Status. Observe `GET /api/v1/sync/status` shows `state: running`, `last_run.trigger: manual_exchanges`, `finished_at: null` indefinitely despite exchange phase completing in logs.
2. **Defect J (empty subscriptions):** After full Firefly sync with 922+ transactions including recurring merchants, open Subscriptions page or `GET /api/v1/subscriptions` — returns `[]` / zero confirmed patterns despite operator expecting 10+ subscriptions.
3. **Defect K (Grafana UNION SQL):** Open Analytics portfolio dashboard; panel query fails with **500** `db query error: pq: syntax error at or near "UNION"`. Other dashboards mostly empty or return **400** on ds/query.
4. **Defect L (empty forecast/wealth/analytics):** Open Forecast and Wealth UI routes — no data at any account; Grafana dashboards show no values despite sync entity counts showing transactions loaded.

**expected:**

- **I:** `RunMode::ExchangesOnly` / `manual_exchanges` run calls `finish_sync_run` with terminal status; sync status returns `idle` or `success` with `finished_at` set after exchange phase completes.
- **J:** Subscription detection surfaces recurring expense patterns from synced transactions (≥ operator-known recurring payees) or documents explicit detection thresholds in empty-state UX.
- **K:** Provisioned Grafana SQL panels execute without PostgreSQL syntax errors; portfolio allocation pie chart returns **200**.
- **L:** Forecast API serves balance series per account after successful recompute; wealth snapshots populate `net_worth_snapshots`; analytics dashboards bind to synced account/forecast data when `account_id` variable set.

**actual:**

- **I:** Exchange sync logs show success (~1s) but DB sync run row remains `status: running`, `trigger: manual_exchanges`, `finished_at: null`; UI shows endless "running manual_exchanges". Root cause hypothesis: `execute_run` **ExchangesOnly** branch never calls `finish_sync_run` (unlike **Full** Firefly path).
- **J:** `GET /api/v1/subscriptions` → `[]`; detection phase reports `confirmed_count=0` despite 922 txs. Algorithm groups by normalized `payee_key` from transaction **description** only, requires ≥3 expenses with stable interval and ≥60% confidence — may miss user's subscription merchants or use wrong field.
- **K:** Grafana panel SQL in `grafana/provisioning/dashboards/analytics/portfolio.json` uses invalid `UNION ALL` with per-branch `ORDER BY ... LIMIT 1` without subquery parentheses → PostgreSQL syntax error at `UNION`.
- **L:** Forecast recompute may succeed in logs but UI/Grafana panels empty — possible missing `forecast_balance_daily` rows for selected account, empty `net_worth_snapshots`, dashboard `account_id` variable mismatch, or alerts/wealth phase skipped when `forecast_id` is `None`.

**evidence_refs:** operator report 2026-06-05, `handoffs/intake_evidence/intake-20260605-omniflow-post-sync-pipeline.json`, `backend/src/sync/mod.rs` (`RunMode::ExchangesOnly`, L315–330), `grafana/provisioning/dashboards/analytics/portfolio.json` L80, `backend/src/recurrence/group.rs`, `backend/src/subscriptions/detection.rs`

#### Known code cause (intake notes)

- **I:** `execute_run` calls `finish_sync_run(..., "success", ...)` only on **Full** Firefly success path (L236–242); **ExchangesOnly** skips Firefly and never finishes the sync run row before clearing in-memory `active_run`.
- **J:** `by_payee()` keys on `payee_key(description)`; Firefly payee name may live in separate field; min 3 matching txs + interval stability + confidence gate.
- **K:** Invalid SQL: `SELECT ... ORDER BY ... LIMIT 1 UNION ALL SELECT ... ORDER BY ... LIMIT 1` — requires wrapped subqueries or single-query rewrite.
- **L:** Cross-cutting: forecast recompute tied to full sync path; wealth `upsert_daily_snapshot` in `run_exchanges_and_alerts` only when `forecast_id` is `Some`; Grafana variables may default to account with no forecast rows.

#### Out of scope

- `contentscript.js` / ObjectMultiplex / MaxListenersExceededWarning — browser wallet extension console noise (not product defect)
- `GET /api/v1/plans/active/plan-vs-actual` **404** when no active plan — documented empty-state (not defect I–L)

#### Intake evidence (BUG-0004)

- `intake_run_id`: `intake-20260605-omniflow-post-sync-pipeline`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260605-omniflow-post-sync-pipeline.json`
- **Split decision:** single bug with sub-defects I+J+K+L (same operator session/post-recovery environment); discovery may decompose fix tasks independently

#### Intake decomposition (2026-06-05)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 4 sub-defects (stuck exchange sync, empty subscriptions, Grafana UNION SQL, empty forecast/wealth/analytics) |
| Cross-cutting | sync run lifecycle, subscription detection pipeline, Grafana provisioning SQL, forecast/wealth snapshot writers, dashboard variable binding |
| Acceptance breadth | 4 sub-rows I/J/K/L |
| Risk | I misleads operators on sync health; J+L block core product value despite data ingest; K breaks portfolio analytics panel |

**Alternatives considered:** four separate BUG ids — rejected (duplicate env/evidence); merge into BUG-0003 — rejected (BUG-0003 targets DB host / Bitunix registry / datasource connectivity, not post-sync analytics pipeline).

**Related work:** BUG-0002 DONE (Q0008 released 2026-06-05); BUG-0003 DONE (Q0009 released 2026-06-05) — separate tracks unless discovery finds shared root cause.

**Recommended next phase:** `/architecture` (fix contracts for I sync lifecycle, J payee extraction, L Firefly balance parse, K Grafana SQL).

#### Discovery (2026-06-05)

**Orchestrator:** `auto-20260605-bug0004-001`  
**Evidence:** code trace + public curl probes on `financegnome.omniflow.cc` (no secrets). Intake bundle: `handoffs/intake_evidence/intake-20260605-omniflow-post-sync-pipeline.json`. Research: [R-0061](docs/engineering/research.md#r-0061--post-sync-analytics-pipeline-empty-data-paths).

| Sub | Verdict | Confirmed root cause | Live probe (2026-06-05) |
|-----|---------|----------------------|-------------------------|
| **I** | **CONFIRMED** | `RunMode::ExchangesOnly` branch (`manual_exchanges`, `scheduled_exchanges`) runs `run_exchanges_and_alerts` then clears in-memory `active_run` but **never** calls `finish_sync_run` — only `RunMode::Full` Firefly path does (`backend/src/sync/mod.rs` L236–242 vs L315–330). DB rows stay `status: running`, `finished_at: null`. | `GET /api/v1/sync/status` → `state: success` (last **manual** Full run); `GET /api/v1/sync/runs` → **10** stuck `scheduled_exchanges` + **2** stuck `manual_exchanges` with `finished_at: null`. |
| **J** | **CONFIRMED (refined)** | (1) `by_payee()` keys **only** on normalized `description` (`recurrence/group.rs` L17) — misses merchant identity in payload counterparty/payee fields when description is generic bank text. (2) Detection runs **only** on `RunMode::Full` (L260–273); `ExchangesOnly` skips subscription phase. (3) `confirmed_count=0` until operator confirms; UI “Standing orders” tab filters `status=confirmed` + `kind=standing_order`. | `GET /api/v1/subscriptions` → **11 pending**, **0 confirmed** (not `[]` post–Full sync); payee keys are long bank-memo strings (e.g. Strom Teilbetrag), not clean merchant names. |
| **K** | **CONFIRMED** | `portfolio.json` panel id **8** uses invalid PostgreSQL: per-branch `ORDER BY … LIMIT 1` before `UNION ALL` without subquery parentheses (`grafana/provisioning/dashboards/analytics/portfolio.json` L80). | `POST /analytics/grafana/api/ds/query` with portfolio pie SQL → **500** `pq: syntax error at or near "UNION"`. |
| **L** | **CONFIRMED (multi-cause)** | **L1:** Firefly `attributes.current_balance` parsed with `.as_f64()` only (`firefly/mod.rs` L261) — Firefly returns **string** balances → mirror `accounts.balance` **NULL** for all asset accounts. **L2:** Wealth query `balance >= 0` excludes NULL (`wealth/repository.rs` L36) → `compute_extended` returns `accounts: []`, `account_count: 0`. **L3:** Forecast `starting_balance = account.balance.unwrap_or(0.0)` → **731** `forecast_balance_daily` rows per account but flat **0.00** milestones/series. **L4:** `net_worth_snapshots` exist but totals zero. Grafana `account_id` variable resolves (asset firefly_ids 114/115/116); emptiness is data not variable mismatch. Exchange-only path skips wealth/alerts when `forecast_id` is stale — secondary to L1–L3 on Full sync. | `GET /api/v1/wealth` → `accounts: []`, `total_eur: 0`; `GET /api/v1/forecast/daily?account_id=116` → 26-point series all `0.00`; `GET /api/v1/wealth/history` → snapshots with `account_count: 0`; 922 txs / 919 dated. |

**Fix task decomposition (recommended sprint order):**

| Order | Task | Sub | Primary files | Acceptance hook |
|-------|------|-----|---------------|-----------------|
| 1 | **I1** — Call `finish_sync_run(success\|failed)` on `ExchangesOnly` terminal path (mirror Full error handling) | I | `backend/src/sync/mod.rs` | Row I |
| 2 | **K1** — Rewrite portfolio pie SQL with wrapped subqueries or single-row pivot | K | `grafana/provisioning/dashboards/analytics/portfolio.json` | Row K |
| 3 | **L1** — Parse Firefly account `current_balance` string/number; backfill on next sync | L | `backend/src/firefly/mod.rs` | Row L |
| 4 | **L2** — Wealth asset query: `COALESCE(balance, 0)` and/or drop NULL-excluding filter | L | `backend/src/wealth/repository.rs` | Row L |
| 5 | **J1** — Extend payee grouping: description + payload counterparty/payee/`destination_name` fallbacks | J | `backend/src/recurrence/group.rs`, `backend/src/firefly/mod.rs` (if mirror field added) | Row J |
| 6 | **J2** — Subscriptions empty-state: surface pending count + detection thresholds (≥3 txs, ≥60% confidence, description key) | J | `frontend/src/pages/SubscriptionsPage.tsx` | Row J |
| 7 | **L3** — verify-work: recompute + omniflow probes (wealth non-empty, forecast milestones non-zero for funded account, Grafana portfolio 200) | L,I,J,K | verify-work checklist | Rows I–L |

**Alternatives considered:** merge L1 into BUG-0006 Q3 — rejected (distinct surface: account balance vs transaction amount sign); defer J2 UX — rejected (acceptance allows documented empty-state).

**Related (do not merge):** BUG-0006 Q2/Q3 overlap on transaction date/amount ingest may improve subscription expense filter — coordinate in architecture, separate sprint.

---

### BUG-0005 — Exchange sync spot-only (Bitunix futures / multi-product accounts)

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0012, 2026-06-05

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; Bitunix credentials configured; operator report 2026-06-05. Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. **Defect M (Bitunix spot-only sync):** Configure `BITUNIX_API_KEY` / `BITUNIX_API_SECRET`; run exchange sync. Inspect `exchange_holdings` or Wealth UI — only spot balances from `GET /api/spot/v1/user/account` appear; futures/margin wallet balances absent even when operator holds material futures account value.
2. **Defect N (futures API not implemented):** Enable or inspect `enabled_futures` in config — default `false` in `default.toml`; `BitunixConnector::sync_positions` and `sync_funding` return empty stubs; futures require `fapi.bitunix.com` header auth per [R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation), not implemented alongside spot query-sign client.
3. **Defect O (wealth under-report):** After Firefly + exchange sync, Wealth / portfolio crypto allocation reflects spot holdings only — total net worth and Grafana portfolio panels under-report when operator's primary crypto exposure is in futures/margin accounts.

**expected:**

- **M:** Exchange sync ingests **all configured read-only account types** for Bitunix (spot + futures/margin at minimum), tagged with appropriate `product_type`.
- **N:** Futures private REST uses `fapi.bitunix.com` with header auth (`api-key`, `nonce`, `timestamp`, `sign`) per official docs; balances/positions/funding endpoints populate holdings when keys have read permission.
- **O:** Wealth snapshot `crypto_value_eur` and exchange holdings aggregate spot + futures (and other supported wallets) so operator sees complete exchange exposure.

**actual:**

- **M:** `sync_balances` calls only `/api/spot/v1/user/account`; all holdings tagged `product_type: "spot"`.
- **N:** `enabled_futures = false` default; `sync_positions`/`sync_funding` no-op even when flag true; spot `signed_get` uses query `sign` on `openapi.bitunix.com` — incompatible with futures API contract.
- **O:** Wealth/portfolio crypto totals omit futures wallet — operator sees incomplete picture despite successful spot sync (e.g. BUG-0003 G spot test **200** "Spot balance read OK").

**evidence_refs:** operator report 2026-06-05, `handoffs/intake_evidence/intake-20260605-exchange-futures-multi-product.json`, [R-0032](docs/engineering/research.md#r-0032--exchange-connector-rest-patterns-binance-bybit-bitunix), [R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation), [R-0059](docs/engineering/research.md#r-0059--exchange-multi-product-sync-scope-bitunix-futures), `backend/src/exchanges/bitunix.rs`, `backend/config/default.toml`

#### Known code cause (intake notes)

- **M:** US-0007 spot-first MVP (`R-0032` § Bitunix spot-first; `enabled_futures=false`).
- **N:** Separate auth/base URL for futures vs spot; G2 in BUG-0003 scoped to test-connection spike, not holdings ingestion.
- **O:** `PortfolioEngine` / `upsert_daily_snapshot` consume connector holdings — spot-only input → incomplete wealth.

#### Out of scope

- Trading execution, withdraw, or non-read API permissions
- New exchange integrations beyond Bitunix/Binance/Bybit MVP set unless discovery expands
- Tax reporting methodology (R-0034 boundary unchanged)

#### Intake evidence (BUG-0005)

- `intake_run_id`: `intake-20260605-exchange-futures-multi-product`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260605-exchange-futures-multi-product.json`
- **Split decision:** single bug with sub-defects M+N+O (Bitunix multi-product sync scope)

#### Intake decomposition (2026-06-05)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 3 sub-defects (spot-only ingestion, futures API gap, wealth under-report) |
| Cross-cutting | Bitunix connector, futures auth client, config flags, wealth/portfolio aggregation |
| Acceptance breadth | 3 sub-rows M/N/O |
| Risk | Operator trust in wealth view; may block meaningful crypto allocation analytics |

**Alternatives considered:** fold into BUG-0003 G2 — rejected (G2 is registry/auth for test endpoint, not multi-product sync product requirement).

**Related work:** BUG-0003 OPEN (G1/G2); BUG-0004 OPEN (analytics pipeline); Binance already has partial `fapi` in `sync_positions` — Bitunix is primary gap.

**Recommended next phase:** `/architecture` (futures auth client contract, `enabled_futures` default policy, endpoint mapping frozen for quick sprint).

#### Discovery notes (2026-06-05)

| Sub | Confirmed root cause | Confidence | Runtime evidence (no host `.env` read) | Fix task |
|-----|---------------------|------------|----------------------------------------|----------|
| **M** | `BitunixConnector::sync_balances` calls only `GET /api/spot/v1/user/account` on `openapi.bitunix.com`; every ingested row tagged `product_type: "spot"` — no futures/margin wallet path | **confirmed (code)** | `bitunix.rs` L77–119; sync pipeline upserts balances then positions (`service.rs` L151–155). Live: `GET /api/v1/exchanges` → bitunix `holdings: 0`; spot test **200** `"Spot balance read OK"` but no futures rows possible. | **M1** |
| **N** | `enabled_futures = false` in `default.toml` + `BitunixConfig`; `sync_positions`/`sync_funding` return `Ok(vec![])` even when flag true (L122–129, L197–204). Spot client uses query `sign` on `openapi.bitunix.com` — **incompatible** with futures header auth on `fapi.bitunix.com` per [R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation). No `futures_base_url` in config. | **confirmed (code + settings)** | Settings: `enabled_futures: false`, `spot_base_url: openapi.bitunix.com`. Futures endpoints documented: account `GET /api/v1/futures/account?marginCoin=USDT`, positions `GET /api/v1/futures/position/get_pending_positions` ([R-0059](docs/engineering/research.md#r-0059--exchange-multi-product-sync-scope-bitunix-futures)). Binance pattern: separate fapi host in `sync_positions` (`binance.rs` L127–131). | **N1**, **N2**, **N3**, **N4** |
| **O** | Wealth/portfolio aggregate **all** `exchange_holdings` via `load_all_holdings` — no product_type filter (`wealth/service.rs` L114–157; `portfolio/pnl.rs` L25–57). Spot-only connector output → `crypto.subtotal_eur` and `crypto_value_eur` snapshot omit futures exposure. | **confirmed (code + live symptom)** | Live: `GET /api/v1/wealth` → bitunix connected, `holdings_count: 0`, `crypto.subtotal_eur: 0` despite successful spot connection test. Downstream of M+N — no separate wealth filter bug. | **O1** (verify-work) |

**Futures endpoint map (discovery canonical):**

| Purpose | Method | Path | Auth |
|---------|--------|------|------|
| Futures wallet balance | GET | `/api/v1/futures/account?marginCoin=USDT` | Header: `api-key`, `nonce`, `timestamp`, `sign` on `https://fapi.bitunix.com` |
| Open positions | GET | `/api/v1/futures/position/get_pending_positions` | Same header auth |
| Spot wallet (existing) | GET | `/api/spot/v1/user/account` | Query `timestamp` + `sign` on `openapi.bitunix.com` |

**Discovery verdict:** M/N/O root causes **confirmed**. Primary gap is Bitunix futures client + ingestion; wealth under-report (O) resolves when futures holdings persist with EUR pricing. Bybit/Binance parity out of scope unless architecture expands.

**Fix decomposition (recommended quick sprint):** N1 futures header-auth client → M1 futures account balances → N2 pending positions → N3 config/`enabled_futures` policy → N4 dual-path test_connection → O1 verify-work rows M/N/O on US-0010.

**Discovery decomposition evidence:** single-bug retained; 2 connector gaps (M spot-only, N auth/stubs/config) + 1 downstream symptom (O).

---

### BUG-0006 — AI get_transactions sees no expenses despite synced transactions

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0010 re-run 2, 2026-06-05

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; Firefly sync reports **922 transactions**; AI Chat enabled; operator report 2026-06-05 (~23:30). Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. **Defect P (AI claims no data):** After successful Firefly sync (Sync Status shows hundreds+ transactions), open AI Chat and ask about current-month category spending (German operator example). Model responds: *"Es wurden keine Ausgaben für diesen Monat in den Kategorien aufgezeichnet. Möglicherweise gab es keine Transaktionen oder die Daten sind nicht verfügbar."* Audit shows tool **`get_transactions`** invoked (~23:30:13).
2. **Defect Q (category mapping gap):** Inspect mirror `transactions` rows — `category_id` column may be **NULL** for all rows because `upsert_transaction` never writes `category_id` from Firefly payload despite column existing in schema.
3. **Defect R (aggregate contract):** `get_transactions` returns `TransactionAggregates` with `by_category` groups only; no top-level `total_transaction_count` / explicit empty-state when period has rows but zero outflow (amount sign) or all uncategorized — LLM interprets as "no expenses in categories".

**expected:**

- **P:** AI answers spending questions with non-empty aggregates when mirror contains transactions for the requested `period_start`/`period_end`.
- **Q:** Firefly sync maps category (and related metadata) into mirror columns used by `TransactionsRepository::aggregates_by_category`.
- **R:** Tool JSON includes period totals / transaction counts and distinguishes **no rows in period** vs **rows present but uncategorized/zero outflow** (DEC-0032 aggregate-only mode preserved).

**actual:**

- **P:** Operator sees German "no expenses / data unavailable" message after `get_transactions` despite sync entity count **922**.
- **Q:** `backend/src/db/mod.rs` `upsert_transaction` INSERT/UPDATE omits `category_id`; `sync_transactions` does not extract category from Firefly journal payload.
- **R:** Aggregates may return empty `by_category: []` for wrong date window, all-null categories with misleading zeros, or uncategorized bucket without labels — model fills gap with "no transactions" narrative.

**evidence_refs:** operator report 2026-06-05 (German AI response + `get_transactions` audit ~23:30:13), `handoffs/intake_evidence/intake-20260605-ai-get-transactions-empty.json`, [R-0060](docs/engineering/research.md#r-0060--ai-get_transactions-empty-aggregates-vs-mirror-sync), `backend/src/ai/tools/transactions.rs`, `backend/src/transactions/repository.rs`, `backend/src/firefly/mod.rs`, `backend/src/db/mod.rs`

#### Known code cause (intake notes)

- **Q (confirmed in code):** Mirror ingest gap — category never persisted.
- **P/R (hypothesis):** Wrong LLM date window, Firefly amount sign vs outflow SUM (`amount < 0`), and/or missing aggregate summary fields for model consumption.

#### Discovery notes (2026-06-05)

| Sub | Root cause (confirmed) | Confidence | Fix task | Evidence |
|-----|------------------------|------------|----------|----------|
| **Q** | `sync_transactions` never extracts `category_id` from Firefly journal splits; `upsert_transaction` INSERT/UPDATE omits `category_id` despite schema column — all mirror rows NULL category | **confirmed (code)** | **Q1** | `backend/src/firefly/mod.rs` L301–323; `backend/src/db/mod.rs` L211–219; Firefly split field `attributes.transactions[].category_id` (string) per API |
| **Q2** | Date ingest uses strict `%Y-%m-%d` parse on Firefly ISO datetimes (`2026-06-01T00:00:00+09:00`) → parse fails → `transactions.date` NULL → period-filtered aggregates return **zero rows** | **confirmed (code)** | **Q2** | `backend/src/firefly/mod.rs` L305–308; strict-parse probe fails ISO strings |
| **Q3** | Amount stored as Firefly positive split value; `aggregates_by_category` outflow = `SUM WHERE amount < 0` — positive mirror amounts yield **zero outflow** while `COUNT(*) > 0` | **confirmed (code)** | **Q3** | `backend/src/firefly/mod.rs` L309; `backend/src/transactions/repository.rs` L25–27; Firefly convention: positive amount + `type` metadata |
| **R** | `TransactionAggregates` has only `by_category`/`by_month` arrays — no top-level `total_transaction_count`, `total_outflow`, `uncategorized_count`, or explicit empty-state semantics for LLM under `allow_raw_transactions=false` | **confirmed (code)** | **R1** | `backend/src/transactions/types.rs` L46–53; `backend/src/transactions/service.rs` L43–50 |
| **P** | Downstream symptom: tool returns `by_category: []` (NULL dates + period filter) and/or rows with `category_name: null`, `total_outflow: 0`, `transaction_count > 0` without period summary — LLM narrates "no expenses / unavailable" | **confirmed (chain)** | **P1** | Operator report; intake audit `get_transactions` ~23:30:13; [R-0060](docs/engineering/research.md#r-0060--ai-get_transactions-empty-aggregates-vs-mirror-sync) |

**SQL probe (deferred — no local `DATABASE_URL`):** run on production mirror after deploy prep: `SELECT COUNT(*) total, COUNT(category_id) with_cat, COUNT(date) with_date, COUNT(*) FILTER (WHERE amount < 0) neg_amt, COUNT(*) FILTER (WHERE amount > 0) pos_amt, MIN(date), MAX(date) FROM transactions;` compare vs `get_transactions` JSON for operator month.

**Fix decomposition (recommended quick sprint):** Q1 category sync → Q2 date parse → Q3 amount sign normalization → R1 aggregate contract → P1 operator E2E re-test on `financegnome.omniflow.cc`.

**Discovery decomposition evidence:** single-bug retained; 3 ingest gaps (Q/Q2/Q3) + 1 contract gap (R) explain sub-defect P without merging BUG-0002–0005 scope.

#### Out of scope

- Changing `allow_raw_transactions` default to expose raw rows (privacy regression without explicit operator opt-in)
- LLM prompt tuning alone without fixing mirror/aggregate data path
- Firefly write-back or category editing in product UI

#### Intake evidence (BUG-0006)

- `intake_run_id`: `intake-20260605-ai-get-transactions-empty`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260605-ai-get-transactions-empty.json`
- **Split decision:** single bug with sub-defects P+Q+R

#### Intake decomposition (2026-06-05)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 3 sub-defects (AI empty answer, sync category mapping, aggregate contract) |
| Cross-cutting | Firefly sync ingest, transactions mirror schema, TransactionsService aggregates, AI tool registry |
| Acceptance breadth | 3 sub-rows P/Q/R |
| Risk | AI feature unusable for core household Q&A despite successful data sync |

**Alternatives considered:** merge into BUG-0004 — rejected (forecast/subscriptions/Grafana scope); extend US-0006 — rejected (defect not new feature).

**Related work:** BUG-0004 OPEN (broader "transaction data not used in features"); US-0006 DONE (tool exists but data path broken).

**Recommended next phase:** `/architecture` (DEC for amount normalization + aggregate contract; then quick sprint Q0010).

---

### BUG-0007 — AI merchant/category discovery fails despite mirror data

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0017 + release PASS, 2026-06-07

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; post-BUG-0006 deploy (aggregate/category ingest fixes DONE); Firefly sync reports **922+ transactions**; AI Chat enabled; operator report 2026-06-05 (production chat on `financegnome.omniflow.cc`). Do not read `.env` / `.env_prod` secrets. Privacy: `allow_raw_transactions=false` (aggregate-only default).

**steps_to_reproduce:**

1. **Defect S (subscription enumeration):** Open AI Chat. Ask about cancelable streaming/subscription savings. AI reports ~**200 €/month** cancelable using **`get_subscriptions`** (2× calls). Follow up: *"liste mir die dienste auf"*. AI responds it **cannot retrieve specific streaming services** from subscriptions; mentions only **Netflix ~6,37 €**; gives generic industry list (Netflix, Prime, Disney+, etc.) and asks user to state which services they have.
2. **Defect T (merchant/category keyword misses):** Ask about **electricity costs (Strom)** for a period with known utility bookings. AI reports **no recorded electricity expenses** — possibly miscategorized or no bookings; offers other period/category. Separately ask **Amazon spend Jan–Oct 2023**. AI reports **no Amazon expenses** in period — miscategorized or no bookings; offers other period.
3. **Defect U (cross-signal discovery gap):** Observe that AI does not fuse **category + transaction name/description + account + amounts** to infer merchants (streaming, utilities, Amazon) without the user naming them explicitly.
4. **Discovery note V (architecture — not intake fix):** Operator asks whether **RAG** is needed; intake captures as discovery/architecture question (tool contract vs retrieval layer) — **no prescriptive fix at intake**.

**expected:**

- **S:** When mirror/subscription detection holds recurring streaming patterns, AI enumerates **named merchants/payees** from `get_subscriptions` and/or `get_transactions` — not generic-only lists or "cannot retrieve" after reporting cancelable totals.
- **T:** Merchant/category queries (**Strom**, **Amazon**, streaming) return **data-backed amounts** or explicit empty-state with evidence that category/description/account filters were applied — not blanket "no expenses" when mirror plausibly contains matches.
- **U:** AI reasons across **categories, payee/description, account, amounts** in tool orchestration without requiring the user to supply merchant names.
- **V:** Discovery documents **RAG vs tool-enhancement** tradeoff; intake does not mandate either approach.

**actual:**

- **S:** Cancelable total (~200 €/month) from `get_subscriptions`, but follow-up enumeration fails; only Netflix ~6,37 € cited; generic list + user prompt.
- **T:** Strom and Amazon Jan–Oct 2023 queries return "no expenses / miscategorized / no bookings" narratives despite operator expectation that mirror holds relevant transactions.
- **U:** AI defers to user to name services instead of inferring from transaction signals.
- **V:** Open question — deferred to discovery.

**evidence_refs:** operator report 2026-06-05 (German production chat — streaming list, Strom, Amazon examples); `handoffs/intake_evidence/intake-20260605-ai-merchant-category-discovery.json`; tools audited: **`get_subscriptions`**, **`get_transactions`**; post-BUG-0004 (11 pending subscription patterns), post-BUG-0006 (aggregate ingest fixes DONE)

#### Known code cause (intake notes)

- **S (hypothesis):** `get_subscriptions` may expose totals without merchant/payee enumeration for pending vs confirmed states; AI may not chain `get_transactions` by recurring payee after subscription aggregate.
- **T (hypothesis):** Aggregate-only mode may lack description/counterparty search dimensions; category labels may not match operator keywords (Strom, Amazon); period window or miscategorization handling weak.
- **U (hypothesis):** Tool contracts and/or orchestrator prompts do not require cross-field fusion (category + description + account + amount).
- **V:** Architecture discovery only — RAG vs richer tool payloads.

#### Discovery notes (2026-06-07 — PO discovery, orchestrator `auto-20260607-bug0007-001`)

**Probe host:** `https://financegnome.omniflow.cc` (US-0010 external profile). No `.env` / `.env_prod` secrets read. Live probes: public API + Grafana `FlowFinancePostgreSQL` ds/query.

| Sub | Verdict | Confidence |
|-----|---------|------------|
| **S** | **CONFIRMED** — mirror holds named subscription merchants; AI fails to enumerate despite `get_subscriptions` success | high |
| **T** | **SPLIT** — Amazon Jan–Oct 2023 is **true empty period** (mirror has no 2023 rows); Strom/Amazon in valid mirror window are **category-ID tool-contract misses**, not missing mirror data | high |
| **U** | **CONFIRMED** — no cross-signal fusion path under aggregate-only + privacy defaults + weak orchestration | high |
| **V** | **NOTE** — no RAG layer in codebase; tradeoff deferred to research ([R-0065](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag)) | high |

**Live mirror facts (runtime proof):**

- `GET /api/v1/sync/entities` → **922** transactions, **75** categories; sync `state: success` (last run 2026-06-05T21:39Z).
- Mirror transaction **date range:** `2025-06-05` … `2026-05-22` only — **0 rows** for `2023-01-01`…`2023-10-31`.
- `GET /api/v1/settings` → `privacy.allow_raw_transactions=false`, `redact_counterparties=true` (DEC-0032 default).
- Category totals in mirror (all-time): `Shopping - Amazon` **28 tx / 1079.35 €** (`firefly_id=47`); `Wohnen - Stromkosten` **4 tx / 465.53 €** (`firefly_id=146`); `Hobby & Freizeit - Streaming` **20 tx / 350.51 €** (`firefly_id=18`). May 2026 alone: Stromkosten **465.53 €**.
- `GET /api/v1/subscriptions` → **12** patterns: **3 confirmed** (Netflix, YouTube, Mitgliedsbeitrag), **6 pending** (incl. Apple, Cursor), **3 rejected** (incl. Strom standing_order **-357.60 €**). Description search for `amazon` in `transactions.description` → **0** rows (merchant signal lives in categories / subscription `display_name`, not description keyword).

**Sub-defect S — subscription enumeration (root causes):**

1. **Data present:** `SubscriptionService::list_patterns` (AI tool path) returns `display_name`, `status`, `kind`, `current_amount` for all patterns — REST API also exposes `payee_key` but AI tool JSON **omits `payee_key`** (`backend/src/subscriptions/service.rs` vs `backend/src/api/subscriptions.rs`).
2. **LLM synthesis failure:** Audit `GET /api/v1/ai/audit` (2026-06-05 session) shows `get_subscriptions` **`result_status: ok`** with `{}` args, yet operator chat reports "cannot retrieve" + generic Netflix/Disney list — defect is **orchestration/LLM interpretation**, not empty mirror.
3. **Malformed follow-up tool args:** Same session shows `get_subscriptions` errors with `kind: Counterparty-*` and `status: Counterparty-*` — LLM confuses **privacy hashes** with enum filters (`pending|confirmed|rejected`, `subscription|standing_order`).
4. **Status/kind filtering gap:** Operator expects streaming list after ~200 €/month cancelable total; confirmed subscriptions sum **~40 €/mo**; pending subscriptions **~56 €/mo**; larger totals come from **standing_order** pending patterns — tool schema does not guide LLM to filter `kind=subscription` or surface pending merchant names.

**Sub-defect T — merchant/category keyword misses (root causes):**

1. **Period mismatch (Amazon 2023):** Operator query window 2023 has **no mirror rows** — correct empty-state is "no transactions in period", not miscategorization. Acceptance row T still requires explicit empty-state evidence (period + filters attempted).
2. **category_id misuse:** Audit shows `get_transactions` with `category_id: "amazon"` and `category_id: "Strom"` — tool expects **Firefly `category_id`** (e.g. `47`, `146`), not natural-language keywords (`backend/src/ai/tools/transactions.rs` schema lacks category-name resolution).
3. **No merchant/description dimension:** `get_transactions` supports only `period_start/end`, optional `category_id`, `group_by: category|month` — **no** `description`, `payee`, or `merchant` filter; `aggregates_by_category` SQL joins `categories.name` but LLM cannot discover IDs without a category catalog tool.
4. **Privacy gate:** `allow_raw_transactions=false` suppresses `raw_rows` (description-bearing); even with opt-in, `redact_counterparties=true` hashes `description`/`payee` fields (`backend/src/ai/privacy.rs`).

**Sub-defect U — cross-signal discovery gap (root causes):**

1. **System prompt bias:** `SYSTEM_PROMPT` instructs "Prefer aggregates when raw transactions are disabled" (`backend/src/ai/orchestrator.rs`) — discourages merchant-level reasoning.
2. **No fusion tool path:** Six-tool registry has no bridge from natural-language merchant intent → `category_id` / subscription `display_name` / account signals.
3. **Audit observability gap:** `ai_tool_audit` stores `args_summary` only — **no result row counts or payload shape** (`result_rows` always `None`) — operator cannot distinguish empty mirror vs mis-parameterized tool vs LLM ignore from audit alone.

**Sub-defect V — RAG vs tools (discovery note only):**

- **No RAG/embedding/vector store** in `backend/` — merchant discovery must flow through tool contracts + orchestrator today.
- Research should compare: (a) richer `get_transactions`/`get_subscriptions` payloads + category catalog tool, (b) optional `allow_raw_transactions` policy, (c) future RAG over mirror text — see [R-0065](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag). **No architecture prescription at discovery.**

**Discovery decomposition evidence:**

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 3 confirmed sub-defects + 1 note; T split into period-empty vs tool-contract |
| Cross-cutting | US-0006 orchestrator, `get_transactions`, `get_subscriptions`, privacy layer, subscription detection, category ingest (post-BUG-0006) |
| Acceptance breadth | S/T/U unchanged; T operator repro partially explained by mirror date range |
| Risk | Fix may span tool schema, orchestrator prompt, category resolution — research required before architecture |

**Recommended next phase:** `/research` ([R-0065](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag)) → `/architecture` (no prescriptive fix at discovery).

#### Out of scope

- Reverting BUG-0006 aggregate/category ingest fixes
- Changing `allow_raw_transactions` default without explicit operator opt-in
- Prescriptive RAG implementation at intake
- Firefly write-back or in-app category editing

#### Intake evidence (BUG-0007)

- `intake_run_id`: `intake-20260605-ai-merchant-category-discovery`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260605-ai-merchant-category-discovery.json`
- **Split decision:** single bug with sub-defects S+T+U (+ discovery note V)

#### Intake decomposition (2026-06-05)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 3 testable sub-defects (subscription enumeration, merchant/category misses, cross-signal discovery) + 1 architecture note |
| Cross-cutting | US-0006 AI orchestrator, `get_subscriptions`, `get_transactions`, subscription detection, privacy aggregates |
| Acceptance breadth | 3 sub-rows S/T/U |
| Risk | Core household Q&A unusable for "what services do I pay for?" and merchant-specific spend despite synced mirror |

**Alternatives considered:** extend BUG-0006 — rejected (DONE; fixed empty aggregates — new failures are merchant enumeration and cross-field discovery); new US story — rejected (defect-shaped production chat failures).

**Related work:** BUG-0004 DONE (11 pending subscription patterns); BUG-0006 DONE (category/aggregate ingest); US-0006 DONE (six tools exist — intelligence path broken).

**Recommended next phase:** _(closed — see BUG-0008)_

#### Sprint plan (2026-06-07 — `sprint-plan-20260607-q0017-bug0007`)

**Quick sprint:** **Q0017** — 7 tasks (~13.5h); no split (7 ≤ `SPRINT_MAX_TASKS` 12).  
**sprint_id:** Q0017

| Order | Task | Acceptance hook |
|-------|------|-----------------|
| 1 | **A1** — Category search SQL + mirror bounds | **(T)**, **(U)** |
| 2 | **A2** — Tool schema + response assembly | **(T)**, **(U)** |
| 3 | **F1** — Subscriptions schema + response + guard | **(S)** |
| 4 | **E1** — SYSTEM_PROMPT + audit result_rows | **(S)**, **(T)**, **(U)** |
| 5 | **E2** — Parameter schema descriptions | **(S)**, **(T)** |
| 6 | **T1** — Unit/integration tests | regression |
| 7 | **V1** — verify-work omniflow AI Chat smoke | **(S)**, **(T)**, **(U)** |

**Operator gates:** deploy A1–E2+T1 single backend PR → **backend image on omniflow** → V1.

**Artifacts:** `sprints/quick/Q0017/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260607-q0017-bug0007`)

**Recommended next phase:** `/plan-verify` on **Q0017** → `/execute`

### BUG-0008 — Subscription alerts vs list mismatch & under-detection

Status: DONE
Priority: P1

**closure_note:** verify-work PASS Q0018 + release PASS, 2026-06-08

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; post-BUG-0004 deploy (11 pending subscription patterns, subscription UX partial fix); Firefly sync **922+ transactions**; operator report 2026-06-05. Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. **Defect W (alert count vs list mismatch):** After sync and subscription detection, observe UI shows **33** subscription-related alerts (badge/unread or alerts page) but Subscriptions list (`/subscriptions` or `GET /api/v1/subscriptions`) shows only **11** rows (pending patterns).
2. **Defect X (under-detection):** Operator expects materially more recurring subscriptions from 922+ transaction ledger; detection surfaces too few candidates overall. Operator suggests improving detection rules and/or AI API for better merchant identification.

**expected:**

- **W:** Subscription-scoped alert unread count reconciles with visible subscription list rows (pending + confirmed + standing orders per US-0003 contract); no orphan alerts without corresponding list entry or documented dismissed/rejected state.
- **X:** Detection engine surfaces recurring expense patterns for operator-known subscription merchants at recall materially above current **11 pending** without alert spam; optional AI-assisted enrichment documented if used.

**actual:**

- **W:** UI reports **33 alerts** vs **11** subscriptions in list — count mismatch breaks operator trust in alert inbox vs subscriptions page.
- **X:** Too few subscriptions detected overall despite large synced ledger; BUG-0004 J partial fix left 11 pending with bank-memo payee keys; operator wants higher recall via improved rules and/or AI API.

**evidence_refs:** operator report 2026-06-05 (33 alerts vs 11 list); `handoffs/intake_evidence/intake-20260605-subscription-alerts-detection.json`; post-BUG-0004 (11 pending, 0 confirmed); US-0003 subscription alerts boundary

#### Known code cause (intake notes)

- **W (hypothesis):** Alert entity may count historical/dismissed/subscription-scoped events separately from current `GET /api/v1/subscriptions` filter; unread-count API may include price-change + new-detection alerts not visible on default list tab.
- **X (hypothesis):** `by_payee()` / confidence gates still miss merchants; BUG-0004 J1 payee fallback may be incomplete; AI path not wired into detection pipeline (operator suggestion only at intake).

#### Out of scope

- BUG-0007 AI chat merchant enumeration (separate surface — coordinate don't merge)
- Firefly write-back or in-app subscription editing
- Prescriptive AI detection implementation at intake

#### Intake evidence (BUG-0008)

- `intake_run_id`: `intake-20260605-subscription-alerts-detection`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260605-subscription-alerts-detection.json`
- **Split decision:** single bug W+X (subscription alert + detection pipeline)

#### Intake decomposition (2026-06-05)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 2 sub-defects (alert/list mismatch, under-detection) |
| Cross-cutting | subscription detection, alert entity, `/subscriptions` UI, optional AI enrichment |
| Acceptance breadth | 2 sub-rows W/X |
| Risk | Operator distrust of subscription alerts; missed recurring charges |

**Alternatives considered:** extend BUG-0004 J — rejected (DONE; new alert-count mismatch); merge BUG-0007 — rejected (AI chat vs UI alert surface).

**Related work:** BUG-0004 DONE (J partial — 11 pending); BUG-0007 DONE (AI enumeration — coordinate only; additive JSON shipped).

#### Discovery notes (2026-06-08 — `discovery-20260608-bug0008`, orchestrator `auto-20260608-bug0008-001`)

**Probe environment:** `https://financegnome.omniflow.cc` public API (no `.env` / `.env_prod` secrets). Post-BUG-0004/BUG-0007 deploy; intake evidence `intake-20260605-subscription-alerts-detection`.

| Sub | Verdict | Confidence |
|-----|---------|------------|
| **W** | **CONFIRMED** — unread subscription alerts accumulate without dedup; count diverges from visible list rows | high |
| **X** | **CONFIRMED** — detection recall below operator expectation from 922+ txs; gates/th grouping limit candidates | high |

**Live probe facts (2026-06-08):**

- `GET /api/v1/subscriptions?status=pending` → **6** rows (operator intake **11** — counts drift as patterns confirm/reject).
- `GET /api/v1/subscriptions` (all statuses) → **12** patterns (**3 confirmed**, **6 pending**, **3 rejected**).
- `GET /api/v1/subscriptions/alerts?unread=true` → **83** unread alerts, all `alert_type=new_detection` (operator intake **33** — same failure mode, worsened with sync history).
- `GET /api/v1/alerts/unread-count` → **0** (US-0005 unified inbox — not the operator-reported subscription banner).

**Sub-defect W — alert vs list mismatch (root causes):**

1. **No alert dedup:** `DetectionPipeline::run_candidates` calls `insert_alert` for every detected group every sync; `subscription_patterns` upserts on fingerprint but `subscription_alerts` always inserts (`detection.rs`, `repository.rs`).
2. **Count semantics split:** Subscriptions page banner = unread **alert rows**; list tabs = **pattern rows** filtered by status/kind — no shared unread-count contract or API.
3. **Header bell adjacent:** `AlertBell` badge uses unified `/api/v1/alerts/unread-count`; subscription unread only in popover link — unlikely primary W symptom but documents dual alert surfaces.

**Sub-defect X — under-detection (root causes):**

1. **Hard gates:** ≥3 txs per payee group, cadence stability, `min_emit_confidence: 60`, `detection_window_days: 365` — unchanged since US-0003/S0003.
2. **Payee grouping:** `extract_payee_source` description → counterparty → destination; bank-memo/SEPA strings may fragment or over-merge (live pending includes long SEPA descriptor payees).
3. **Category unused in grouping:** merchant signal in Firefly categories (see BUG-0007 probe: Amazon/Strom category totals) not used by `by_payee()` recurrence core.
4. **AI not in pipeline:** `get_subscriptions` / orchestrator enrichment (BUG-0007) does not feed detection — operator AI suggestion remains optional research path only.

**Discovery decomposition evidence:**

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 2 sub-defects W/X; shared detection+alert pipeline |
| Cross-cutting | `subscription_alerts`, detection engine, `/subscriptions` UI, optional AI enrichment |
| Acceptance breadth | W/X unchanged; live probe confirms failure mode |
| Risk | W fix without dedup may re-break trust; X recall without dedup may amplify W |

#### Sprint plan (2026-06-08 — `sprint-plan-20260608-q0018-bug0008`)

**Quick sprint:** **Q0018** — 12 tasks (~24h); no split (12 = `SPRINT_MAX_TASKS` 12).  
**quick_task_id:** Q0018  
**sprint_id:** Q0018

| Order | Task | Acceptance hook |
|-------|------|-----------------|
| 1 | **W1** — Fingerprint migration + backfill dedupe | **W** |
| 2 | **W2** — `upsert_alert` repository | **W** |
| 3 | **W3** — Detection emit gate | **W** |
| 4 | **W4** — Unread-count API route | **W** |
| 5 | **W5** — Orphan lifecycle hooks | **W** |
| 6 | **W6** — Frontend banner + toast | **W** |
| 7 | **W7** — Backend dedup + lifecycle tests | **W** regression |
| 8 | **X1** — Payee normalization | **X** |
| 9 | **X2** — Transfer counterparty priority | **X** |
| 10 | **X3** — `detection_window_days` 730 | **X** |
| 11 | **X4** — Forecast + subscription integration tests | **X** regression |
| 12 | **V1** — verify-work omniflow smoke | **W**, **X** |

**Operator gates:** deploy W1–X4+W7 backend + W6 frontend → **BACKEND_FRONTEND_DEPLOY** → V1.

**Artifacts:** `sprints/quick/Q0018/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260608-q0018-bug0008`)

**Recommended next phase:** _(closed — release PASS Q0018)_

---

### BUG-0009 — Grafana empty panels & missing account value overview

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0016 + release PASS, 2026-06-06

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; post-BUG-0004 deploy (UNION SQL fix, balance parse); Firefly sync **922+ transactions**; operator report 2026-06-05. Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. **Defect Y (Grafana still empty):** Open Analytics section — six `/analytics/{slug}` routes (cashflow, subscriptions, budgets, portfolio, forecast, platform health). Despite 922+ synced transactions, Grafana panels show **no data/values** for representative dashboards.
2. **Defect Z (no account overview):** Operator lacks **cross-account value overview** in analytics — cannot see all account balances/values aggregated in Grafana or linked analytics UX.

**expected:**

- **Y:** Provisioned Grafana dashboards return **200** on `POST /analytics/grafana/api/ds/query`; panels show non-empty values when `account_id` variable matches synced Firefly asset accounts.
- **Z:** Operator has account-level value overview (Grafana stat/table/pie or documented link to React wealth with equivalent totals) across synced asset accounts.

**actual:**

- **Y:** Grafana still shows empty panels despite BUG-0004 K/L fixes — possible remaining SQL, variable binding, or snapshot data gaps.
- **Z:** No usable account overview of all values in analytics section.

**evidence_refs:** operator report 2026-06-05; `handoffs/intake_evidence/intake-20260605-grafana-account-overview.json`; post-BUG-0004 K UNION fix DONE; US-0011 embedded analytics routes

#### Discovery notes (2026-06-06 — `discovery-20260606-bug0009`)

**Probe environment:** `https://financegnome.omniflow.cc` public curl (no secrets). Post-BUG-0004 K/L + post-BUG-0010/0012 deploy; latest sync `success` `2026-06-05T21:39:31Z`; 922+ transactions; 3 asset accounts (114 Giro **-3395.75**, 115/116 **0.00**). Orchestrator: `auto-20260606-bug0009-001`.

| Sub-defect | Status | Confirmed root cause | Live evidence |
|------------|--------|----------------------|---------------|
| **Y** | **CONFIRMED (multi-factor)** | **Not a datasource/UNION regression.** `POST /analytics/grafana/api/ds/query` **200** for all probed panels; Postgres datasource OK (BUG-0003 H). Emptiness is primarily **default `$account_id` = 116 (Cash wallet)** — first alphabetically — with **flat zero** `forecast_balance_daily` (731 rows min=max=0). Account **114** returns non-empty cashflow/forecast series. Secondary: **forecast-horizons ML panels** query `model_kind='ml_enhanced'` — **0** computations on omniflow → panels empty by design until US-0013. Portfolio/subscriptions/budgets/platform-health panels return data without account variable. | `account_variable` → 116,114,115; acct **116** forecast cnt=731 all 0; acct **114** daily balances negative non-zero; portfolio `total_eur` **-3395.75**; subscriptions **3** confirmed / **6** pending; UNION pie **200** (post-K fix) |
| **Z** | **CONFIRMED** | **Portfolio account-breakdown SQL truncates to one row** — `ORDER BY snapshot_date DESC LIMIT 1` on cross-join with `jsonb_array_elements` returns **1 of 3** accounts (Cash wallet 0 only). **No dedicated cross-account overview panel** in analytics dashboards. React `/wealth` shows per-account totals + portfolio link (post BUG-0010) — partial equivalent but **not embedded in Grafana analytics shell** per AC Z. | Broken query → 1 row; fixed subquery pattern → 3 rows (0, -3395.75, 0); `GET /api/v1/wealth` lists 3 accounts |

**Panel-by-panel probe summary (`POST /analytics/grafana/api/ds/query`):**

| Dashboard | Representative panel | Needs `$account_id` | Default-var probe | Data present? |
|-----------|---------------------|---------------------|-------------------|---------------|
| cashflow | daily balance series | yes | acct **116** → flat 0 | **Empty appearance** at default; **114 → non-empty** |
| cashflow | monthly decomposition | yes | acct **116** → zeros | same |
| forecast-horizons | tomorrow / horizon stats | yes | acct **116** → 0 | same |
| forecast-horizons | ML compare / bands | yes + `ml_enhanced` | 0 ML computations | **Empty** (US-0013) |
| portfolio | total wealth stat | no | — | **-3395.75** |
| portfolio | UNION allocation pie | no | — | **200**, 2 slices |
| portfolio | account breakdown table | no | SQL bug | **1 row** (should be 3) |
| subscriptions | confirmed / pending counts | no | — | **3 / 6** |
| budgets | active plan + MTD table | no | — | plan **test v1**; MTD rows present |
| platform-health | sync run status | no | — | failed/running/success counts |

**Infrastructure (not primary Y cause):** Grafana health **200**; datasource `FlowFinancePostgreSQL` → `postgres:5432/flow_finance_ai`; embed HTML `<base href="/analytics/grafana/">`; assets `/analytics/grafana/public/build/*.js` **200**; `net_worth_snapshots` latest `account_count: 3`, `total_eur: -3395.75`.

**Advisory:** Unauthenticated curl `/analytics/{slug}` SPA routes → **404** (no static file; client-side nav from authenticated shell expected). Grafana proxy `/analytics/grafana/d/*` → **200**.

#### Fix tasks (discovery decomposition)

| ID | Sub-defect | Task | Primary files / surface |
|----|------------|------|-------------------------|
| **Y1** | Y | Fix `$account_id` variable default — prefer account with non-zero forecast series or latest non-zero mirror balance; avoid alphabetical default to zero-balance wallet | `grafana/provisioning/dashboards/analytics/cashflow.json`, `forecast-horizons.json` |
| **Y2** | Y | Forecast-horizons ML panels: hide or show documented empty-state when no `ml_enhanced` computation (baseline-only omniflow profile) | `forecast-horizons.json` |
| **Y3** | Y | verify-work: operator smoke with `$account_id=114` **and** corrected default — cashflow + forecast panels non-empty | acceptance row Y |
| **Z1** | Z | Fix portfolio account-breakdown SQL — subquery latest snapshot then `jsonb_array_elements` (remove erroneous global `LIMIT 1`) | `portfolio.json` |
| **Z2** | Z | Add cross-account value overview panel (stat row or table of all asset accounts from latest `net_worth_snapshots.payload`) to portfolio dashboard or analytics landing | `portfolio.json`, optional `AnalyticsEmbedPage.tsx` / sidebar copy |
| **Z3** | Z | verify-work: overview shows all synced asset account balances; document React `/wealth` link as supplementary if Grafana panel satisfies AC Z | acceptance row Z |

**US-0013 boundary:** Y2 ML panel emptiness closes with honest empty-state at bug fix; enabling ML overlay on omniflow remains **US-0013** epic.

#### Known code cause (intake notes — superseded by discovery)

- **Y (was hypothesis):** Dashboard variable defaults to zero-balance account — **confirmed Y1**; datasource/UNION gaps — **ruled out**.
- **Z (was hypothesis):** No cross-account summary + broken breakdown SQL — **confirmed Z1/Z2**.

#### Out of scope

- React `/forecast` and `/wealth` product pages (BUG-0010)
- OIDC/auth middleware regressions (BUG-0001 DONE)

#### Intake evidence (BUG-0009)

- `intake_run_id`: `intake-20260605-grafana-account-overview`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260605-grafana-account-overview.json`
- **Split decision:** single bug Y+Z (Grafana analytics surface)

#### Intake decomposition (2026-06-05)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 2 sub-defects (panel emptiness, account overview gap) |
| Cross-cutting | Grafana provisioning, datasource SQL, analytics proxy, account variables |
| Acceptance breadth | 2 sub-rows Y/Z |
| Risk | Core analytics unusable on production despite synced data |

**Alternatives considered:** merge BUG-0004 K/L — rejected (DONE; operator re-smoke shows persistent emptiness).

**Related work:** BUG-0004 DONE (K UNION, L balance parse); US-0011 DONE (embed routes).

**Recommended next phase:** `/research` → `/architecture` (variable default strategy; overview panel UX; ML empty-state vs US-0013 split)

#### Sprint plan (2026-06-06 — `sprint-plan-20260606-q0016-bug0009`)

**Quick sprint:** **Q0016** — 6 tasks (~7h); no split (6 ≤ `SPRINT_MAX_TASKS` 12).  
**sprint_id:** Q0016

| Order | Task | Acceptance hook |
|-------|------|-----------------|
| 1 | **Z1** — Portfolio breakdown SQL subquery + LATERAL | **(Z)** |
| 2 | **Z2** — Cross-account overview table + grid layout | **(Z)** |
| 3 | **Y1** — `$account_id` ABS(balance) variable query | **(Y)** |
| 4 | **Y2** — ML banner + noValue on ML panels | **(Y)** |
| 5 | **T1** — SQL fixtures + provisioning snapshot test | **(Y)**, **(Z)** |
| 6 | **V1** — verify-work omniflow smoke | **(Y)**, **(Z)** |

**Operator gates:** deploy Z1–Y2+T1 → **Grafana provisioning reload** → V1.

**Artifacts:** `sprints/quick/Q0016/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260606-q0016-bug0009`)

**Recommended next phase:** `/plan-verify` on **Q0016** → `/execute`

---

### BUG-0010 — Forecast & Wealth empty/wrong numbers; ML skipped

Status: DONE
Priority: P0

**closure_note:** verify-work PASS Q0013 + release PASS, 2026-06-05

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; post-BUG-0004 deploy; Firefly sync **922+ transactions**; operator report 2026-06-05. Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. **Defect AA (forecast wrong numbers):** Open `/forecast`, select account and 3-month horizon. **End balance (3 months)** shows **-25365.78** — implausible/wrong for operator funded accounts.
2. **Defect AB (wealth empty/unusable):** Open `/wealth` — no usable net-worth data despite sync success.
3. **Defect AC (ML skipped):** Forecast UI shows message **"ML skipped: ML forecast unavailable. Baseline DEC-0007 forecast remains authoritative for alerts."** Operator expects ML forecast feature to work fully (US-0009).

**expected:**

- **AA:** Forecast milestones and series show plausible signed balances aligned with synced account funding and expense patterns; 3-month end balance not wildly negative without explicit scenario.
- **AB:** Wealth API and UI show non-empty account breakdown and `total_eur` for synced Firefly asset accounts.
- **AC:** ML forecast path runs when US-0009 sidecar/config available, or UI/API accurately states why baseline-only mode applies; baseline DEC-0007 forecast still numerically correct.

**actual:**

- **AA:** End balance (3 months) **-25365.78** — wrong/unusable forecast output.
- **AB:** Forecast and Wealth have no usable data per operator.
- **AC:** ML skipped banner; operator wants full ML implementation.

**evidence_refs:** operator report 2026-06-05 (-25365.78, ML skipped message); `handoffs/intake_evidence/intake-20260605-forecast-wealth-ml.json`; post-BUG-0004 L partial fix; US-0009 DONE; DEC-0007 baseline forecast

#### Discovery notes (2026-06-05 — `discovery-20260605-bug0010`)

**Probe environment:** `https://financegnome.omniflow.cc` public curl (no secrets). Post-BUG-0004 L (DEC-0060 parse) + post-BUG-0006 Q3 (DEC-0059 tx sign). Latest sync: `scheduled_exchanges` success `2026-06-05T16:28:57Z`; 922 transactions, 375 accounts.

| Sub-defect | Status | Confirmed root cause | Live evidence |
|------------|--------|----------------------|---------------|
| **AA** | **CONFIRMED** | **Wrong mirror starting balances**, not projection math bug. Acct **114** (Raiffeisenbank Giro): daily series starts **-3395.75** (2026-06-05); 3mo end **-25365.78** (`GET /forecast/long-term?account_id=114&horizon=3`). Monthly variable outflows ~**6029 EUR** — consistent drift from negative start + recurring/rolling. Accts **115/116**: flat **0.00** series (mirror `balance` 0.0). DEC-0060 fixed string parse but **values still 0 or negative** vs operator-funded expectation. | `forecast/meta` computation `d372cfec` @ `2026-06-05T15:41:24Z`; `low_confidence: true` |
| **AB** | **CONFIRMED** | **Zero mirror balances + negative-account exclusion.** `GET /api/v1/wealth` → 2 accounts (116, 115), `total_eur: 0.0`; **114 excluded** by `load_asset_accounts` `COALESCE(balance,0) >= 0` (Giro negative in mirror). Snapshots exist (`wealth/history` `account_count: 2`) but totals zero — data not writer skip. Exchange-only sync still upserts snapshot with stale/zero balances. | Forecast lists 3 accounts; wealth lists 2 |
| **AC** | **CONFIRMED (split BUG vs epic)** | **ML never runs on omniflow external profile.** `[forecast_ml] enabled = false` (DEC-0049); `stats-forecast` Compose `profiles: [full]` only — not on US-0010 external merge. Sync skips ML block when disabled → **no** `record_skip_on_baseline` → meta `ml_skipped_reason: null`, `ml_computation_id: null`. **UX defect:** `ForecastPage` shows **"ML skipped: ML forecast unavailable"** whenever `!mlAvailable` (default reason) — misleading when ML was **not configured**, not skipped. Compare endpoint **400** (no ML row). | `forecast/meta` all ML fields null |

**AA math check:** -3395.75 + ~90d × ~200 EUR/day outflow ≈ -25365 — engine behaves per DEC-0007 given inputs.

**Post-BUG-0004 L residual:** Q0011 verify noted zero balances "structural pass"; BUG-0010 is **wrong-number regression** on Giro + unusable zero wealth.

#### Fix tasks (discovery decomposition)

| ID | Sub-defect | Task | Primary files / surface |
|----|------------|------|-------------------------|
| **AA1** | AA | Fix Firefly account `current_balance` mirror — verify Firefly source vs DB for accts 114/115/116; extend ingest if wrong attribute/sign for asset accounts (beyond DEC-0060 parse-only) | `backend/src/firefly/mod.rs`, account upsert |
| **AA2** | AA | Operator gate: **manual Full Firefly sync** + forecast recompute after AA1; verify plausible starting balances before series | sync/forecast pipeline |
| **AA3** | AA | Forecast degraded UX when asset account `starting_balance <= 0` with tx history — surface warning in API meta or UI (avoid silent -25k) | `backend/src/api/forecast.rs`, `ForecastPage.tsx` |
| **AB1** | AB | Wealth visibility for excluded negative-balance asset accounts (114) — include with flag or separate row; don't silently omit primary checking account | `backend/src/wealth/repository.rs`, `WealthPage.tsx` |
| **AB2** | AB | Wealth empty-state when `total_eur == 0` but synced asset accounts exist — operator guidance (re-sync / check Firefly balances) | `frontend/src/pages/WealthPage.tsx` |
| **AB3** | AB | _(blocked by AA1)_ Snapshot totals follow mirror fix — re-verify `net_worth_snapshots` post Full sync | `backend/src/wealth/service.rs` |
| **AC1** | AC | Persist accurate ML posture in meta: when `forecast_ml.enabled=false`, set `ml_skipped_reason: sidecar_disabled` on baseline recompute **or** derive in `/forecast/meta` from config | `backend/src/sync/mod.rs`, `backend/src/api/forecast.rs` |
| **AC2** | AC | UI: distinguish **ML not enabled** vs **ML skipped** vs **ML available**; remove default "ML forecast unavailable" skip copy when reason null | `frontend/src/pages/ForecastPage.tsx` |
| **AC3** | AC → **US-0013** | Production ML on omniflow: `stats-forecast` on external profile + `FORECAST_ML_ENABLED=true` + sidecar health — **epic**, not BUG-0010 closure | compose external, runbook |

**US-0013 boundary:** BUG-0010 **AC** closes accurate baseline numbers (AA) + honest degraded ML messaging (AC1/AC2). **US-0013** closes ML overlay actually running on omniflow (AC3, sidecar ops, compare tab).

#### Out of scope

- Grafana panel emptiness (BUG-0009)
- New ML model research (defer epic scope to **US-0013**)

#### Intake evidence (BUG-0010)

- `intake_run_id`: `intake-20260605-forecast-wealth-ml`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260605-forecast-wealth-ml.json`
- **Split decision:** single bug AA+AB+AC; epic ML hardening → **US-0013**

#### Intake decomposition (2026-06-05)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 3 sub-defects (wrong forecast, empty wealth, ML skip) |
| Cross-cutting | forecast recompute, wealth snapshots, ML sidecar, React forecast/wealth pages |
| Acceptance breadth | 3 sub-rows AA/AB/AC |
| Risk | Core cashflow/wealth product unusable; wrong negative balance misleads alerts |

**Alternatives considered:** merge BUG-0004 L — rejected (DONE; new wrong-number symptom); force all ML into one bug — rejected (**US-0013** epic).

**Related work:** BUG-0004 DONE (L balance parse); US-0009 DONE (ML feature — production path broken); **US-0013** OPEN (epic hardening).

#### Sprint plan (2026-06-05 — `sprint-plan-20260605-q0013-bug0010`)

**Quick sprint:** **Q0013** — 7 tasks (~11h); no split (7 ≤ `SPRINT_MAX_TASKS` 12).

| Order | Task | Acceptance hook |
|-------|------|-----------------|
| 1 | **AA1** — Balance mirror ingest + diagnostics | **(AA)** |
| 2 | **AB1** — Negative asset wealth visibility (DEC-0065) | **(AB)** |
| 3 | **AC1** — `sidecar_disabled` metadata (DEC-0066) | **(AC)** |
| 4 | **AA3** — Negative starting balance warning | **(AA)** |
| 5 | **AB2** — Wealth zero-total empty-state | **(AB)** |
| 6 | **AC2** — Forecast ML three-state UI | **(AC)** |
| 7 | **V1** — verify-work omniflow probes | **(AA)(AB)(AC)** |

**Operator gate (discovery AA2 / AB3):** deploy AA1–AC2 → **manual Full Firefly sync** → V1. AB3 snapshot re-verify in V1 via `wealth` + `wealth/history`.

**Artifacts:** `sprints/quick/Q0013/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260605-q0013-bug0010`)

**Recommended next phase:** `/plan-verify` on **Q0013** → `/execute`

---

### BUG-0011 — Planning mode broken (empty plan, compare sums, plan-vs-actual 404)

Status: DONE
Priority: P1

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; post-BUG-0004 deploy; operator report 2026-06-05. Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. **Defect AD (4a — empty plan click no-op):** Open `/planning` → Scenarios → custom plan **"Start empty and add lines"** — click produces **no action** (no editable plan lines).
2. **Defect AE (4b — compare illogical sums):** Open Compare tab with empty/minimal plan. **v1 Monthly delta sum** shows **-127489.44**; **Projected month-end** **-4042.41** — illogical for empty plan.
3. **Defect AF (4c — plan-vs-actual 404):** `GET /api/v1/plans/active/plan-vs-actual` returns **404**; Plan vs Actual tab broken or empty.

**expected:**

- **AD:** "Start empty and add lines" creates editable plan draft with add-line UX.
- **AE:** Compare with empty plan shows zero/neutral deltas, not large negative aggregates from missing baseline guards.
- **AF:** Plan-vs-actual returns **200** with JSON payload when active plan exists, or **200 documented empty-state** / guided UX when none — not raw **404** breaking the tab.

**actual:**

- **AD:** Nothing happens on click — planning entry path non-functional.
- **AE:** Compare shows **-127489.44** monthly delta and **-4042.41** projected month-end for empty plan.
- **AF:** `GET /api/v1/plans/active/plan-vs-actual` → **404** (BUG-0004 documented as expected when no active plan — operator rejects as broken UX).

**evidence_refs:** operator report 2026-06-05 (4a/4b/4c); `handoffs/intake_evidence/intake-20260605-planning-mode-broken.json`; US-0004 DONE; BUG-0004 out-of-scope 404 note

#### Known code cause (intake notes — superseded by discovery 2026-06-08)

- **AD (hypothesis):** Frontend handler for empty-plan template not wired or API create-plan fails silently.
- **AE (hypothesis):** Compare aggregates uncapped forecast baseline without empty-plan guard.
- **AF (hypothesis):** Route returns 404 instead of structured empty-state; no active plan seeded on first visit.

#### Discovery notes (2026-06-08 — `discovery-20260608-bug0011`, orchestrator `auto-20260608-bug0011-001`)

**Probe environment:** Code audit only (no `.env` / `.env_prod` secrets; no live omniflow API probe this phase). Intake evidence `intake-20260605-planning-mode-broken`.

| Sub | Verdict | Confidence |
|-----|---------|------------|
| **AD** | **CONFIRMED** — no add-adjustment UI; custom Apply clears lines with no follow-up; first-run empty state lacks custom/empty create path | high |
| **AE** | **CONFIRMED** — compare metrics sum full `planned_net` (baseline + overlay), not adjustment-only delta; mislabeled "Monthly delta sum" | high |
| **AF** | **CONFIRMED** — `NoActivePlan` → HTTP 404; plans default `is_active=false`; frontend tab has no empty-state/error handling | high |

**Sub-defect AD — empty plan click no-op (root causes):**

1. **Missing frontend add-line UX:** `PlanningPage.tsx` adjustments table is read-only except delete; backend `POST .../adjustments` unwired (`add_adjustment` in `plans.rs`).
2. **Custom template silent success:** `Apply` on **Custom** calls `apply-template` → `PlanTemplate::Custom` replaces adjustments with `[]` — no visible change when already empty.
3. **First-run CTA gap:** `empty` branch only offers **Create from Leasing template** — operator cannot reach "Start empty and add lines" without an existing plan.

**Sub-defect AE — compare illogical sums (root causes):**

1. **Metric definition bug:** `repository.rs` `version_metrics` sums `plan_daily_cashflow.planned_net` values for the month — includes US-0002 forecast baseline, not overlay-only delta (`project.rs`: `planned_net = baseline_net + overlay_delta`).
2. **Same bug in-memory path:** `service.rs` `project_adjustments_in_memory` filters `p.planned_net` for monthly sum — identical semantic mismatch.
3. **Empty-plan guard absent:** zero adjustments still produce large negative totals from baseline forecast (operator **-127489.44** / **-4042.41**).

**Sub-defect AF — plan-vs-actual 404 (root causes):**

1. **404 on no active plan:** `plan_vs_actual` → `PlanError::NoActivePlan` → `plan_error_status` returns **404** `{ "error": "no_active_plan" }`.
2. **No auto-activate:** `create_plan` inserts `is_active = false`; operator must click **Set active** — easy to miss on first visit.
3. **Frontend silent failure:** `pvaQuery` enabled on tab switch only; no `retry: false` error UI or guided empty-state (contrast `risk-score` **200** `no_score` pattern).

**Discovery decomposition evidence:**

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 3 sub-defects AD/AE/AF; shared `/planning` page, distinct layers (UI wiring, compare math, API contract) |
| Cross-cutting | `PlanningPage.tsx`, plans API, Plan Engine compare + plan-vs-actual |
| Acceptance breadth | AD/AE/AF unchanged; code audit confirms intake failure modes |
| Risk | AE metric fix may change compare semantics for non-empty plans — research must lock delta definition |

**Recommended next phase:** `/research` (compare delta contract; plan-vs-actual empty-state JSON shape; first-run activate/create UX).

#### Research resolution (2026-06-08) — [R-0070](docs/engineering/research.md#r-0070--bug-0011-planning-mode-compare-delta-empty-state-api-first-run-ux)

| Question | Resolution |
|----------|------------|
| Compare delta semantics | Overlay-only sum via `build_overlay_deltas`; zero adjustments → **0.00** (**DEC-0073**) |
| PVA empty contract | HTTP **200** tagged `{ status: "no_active_plan" }` — mirror risk-score (**DEC-0074**) |
| First-run UX | Inline add form + Create empty plan (`POST template=custom`); no auto-activate |
| Grafana scope | Dashboard 3 unchanged — compare fix is `/compare` + React Compare tab only (**R-0020**) |
| Regression | OIDC `/planning` three-tab smoke mandatory |

**Architecture:** `docs/engineering/architecture.md` § **BUG-0011**; **DEC-0073**, **DEC-0074**.

#### Sprint plan (2026-06-08 — `sprint-plan-20260608-q0019-bug0011`)

**Quick sprint:** **Q0019** — 11 tasks (~20h); no split (< `SPRINT_MAX_TASKS` 12).  
**quick_task_id:** Q0019  
**sprint_id:** Q0019

| Order | Task | Acceptance hook |
|-------|------|-----------------|
| 1 | **AE1** — Overlay delta helper | **AE** |
| 2 | **AE2** — Wire repository + service compare paths | **AE** |
| 3 | **AE3** — Compare metric unit tests | **AE** |
| 4 | **AF1** — Tagged PVA API 200 `no_active_plan` | **AF** |
| 5 | **AF2** — PVA guided empty state | **AF** |
| 6 | **AD1** — First-run Create empty plan | **AD** |
| 7 | **AD2** — Inline add/edit adjustment form | **AD** |
| 8 | **AD3** — Custom Apply toast + invalidation | **AD** |
| 9 | **AD4** — Compare footnote + Set active banner | **AD**, **AE** |
| 10 | **T1** — Compare + PVA integration tests | **AD/AE/AF** |
| 11 | **V1** — verify-work OIDC `/planning` smoke | footer |

**Sequencing:** AE-before-AF frozen; AD after AF1 API contract.  
**Artifacts:** `sprints/quick/Q0019/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260608-q0019-bug0011`)

**Recommended next phase:** `/plan-verify`

#### Release closure (2026-06-08 — Q0019 / `auto-20260608-bug0011-001`)

- **Status:** DONE
- **Quick task:** Q0019
- **Acceptance:** AD/AE/AF checked (`docs/product/acceptance.md`)
- **Release notes:** `handoffs/releases/Q0019-release-notes.md`
- **Decisions:** DEC-0073 (AE overlay-only compare), DEC-0074 (AF PVA 200 no_active_plan)
- **Operator follow-up:** **BACKEND_FRONTEND_DEPLOY** then omniflow smoke AD/AE/AF per `sprints/quick/Q0019/uat.md`

#### Out of scope

- AI `simulate_plan` chat (US-0006)
- Full planning UX redesign (defer epic to **US-0014** if discovery exceeds defect scope)

#### Intake evidence (BUG-0011)

- `intake_run_id`: `intake-20260605-planning-mode-broken`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260605-planning-mode-broken.json`
- **Split decision:** single bug AD+AE+AF; epic UX → **US-0014**

#### Intake decomposition (2026-06-05)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 3 sub-defects (empty plan no-op, compare sums, plan-vs-actual 404) |
| Cross-cutting | Plan Engine, plans API, React `/planning` tabs |
| Acceptance breadth | 3 sub-rows AD/AE/AF |
| Risk | Planning feature appears shipped (US-0004 DONE) but unusable in production |

**Alternatives considered:** treat 404 as documented empty-state only — rejected (operator reports non-functional mode); US-only — rejected (defect-shaped clicks and wrong sums).

**Related work:** US-0004 DONE; BUG-0004 out-of-scope 404 note superseded by operator report; **US-0014** OPEN (epic UX).

---

### BUG-0012 — Forecast monthly Income/Fixed buckets always zero

Status: DONE
Priority: P1

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; post-BUG-0010 deploy (baseline balances fixed Q0013); Firefly sync **922+ transactions** with categorized income and fixed-cost bookings; operator report 2026-06-05. Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. **Defect AG (Income always 0):** After successful Full Firefly sync and forecast recompute, open `/forecast` → **Monthly** tab (or `GET /api/v1/forecast/monthly?account_id=<funded>`). **Income** stat/card shows **0** despite mirror transactions with salary/income categories in the selected forecast month.
2. **Defect AH (Fixed always 0):** Same view — **Fixed** stat/card shows **0** despite mirror transactions with rent, utilities, subscription, or other fixed-cost categories mapped in DEC-0007 config; all outflow appears under **Variable** only.

**expected:**

- **AG:** Monthly forecast **Income** reflects categorized inflows (salary, refunds, etc.) per DEC-0007 category→bucket mapping and/or transaction-derived projection — not permanently **0** when mirror holds income-category rows for the account.
- **AH:** Monthly forecast **Fixed** reflects categorized fixed-cost outflows (rent, standing orders, subscription-class expenses) — not permanently **0** while Variable absorbs 100% of spend.

**actual:**

- **AG:** Income **0** on monthly view despite real income transactions in Firefly mirror.
- **AH:** Fixed **0** on monthly view; Variable carries all projected outflow; category split unusable for operator budgeting.

**evidence_refs:** operator report 2026-06-05 (Income 0, Fixed 0 on monthly forecast); `handoffs/intake_evidence/intake-20260605-forecast-monthly-buckets.json`; code gap `backend/src/forecast/project.rs` `categorize_delta` → `map_category(None, config)` for negative deltas; DEC-0007 category config design; post-BUG-0006 `category_id` mirror ingest

#### Known code cause (intake notes)

- **AG/AH (confirmed in code):** `project_account` daily loop calls `categorize_delta(delta, category_names, config)` but function ignores `category_names` and passes **`None`** to `map_category` for all negative deltas → **Variable** bucket only; positive net daily delta alone routes to Income — recurring salary/fixed patterns not decomposed by category.
- **DEC-0007:** TOML `[forecast.categories]` bucket map exists in design/config but projection path does not apply per-transaction/per-pattern category labels.

#### Out of scope

- ML overlay on external profile (**US-0013** — operator issue 1; no new BUG)
- AI chat merchant/category discovery (**BUG-0007** — coordinate only)
- AI-assisted bucket inference beyond config map (**US-0015** epic)

#### Intake evidence (BUG-0012)

- `intake_run_id`: `intake-20260605-forecast-monthly-buckets`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260605-forecast-monthly-buckets.json`
- **Split decision:** single bug AG+AH; AI-assisted category mapping → **US-0015**

#### Intake decomposition (2026-06-05)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 2 sub-defects (Income 0, Fixed 0) + operator AI expectation deferred |
| Cross-cutting | forecast projection engine, category config, monthly API/UI |
| Acceptance breadth | 2 sub-rows AG/AH |
| Risk | Monthly cashflow breakdown core US-0002 promise broken; operator cannot trust income/fixed/variable split |

**Alternatives considered:** merge BUG-0010 AA — rejected (DONE; distinct monthly bucket symptom); merge BUG-0007 — rejected (chat vs forecast projection); force AI into BUG-0012 — rejected (**US-0015** epic).

**Related work:** BUG-0010 DONE (baseline balances); BUG-0006 DONE (category_id ingest); BUG-0007 OPEN (AI chat — coordinate); **US-0013** OPEN (ML external — update only); **US-0015** OPEN (AI forecast buckets epic).

#### Discovery findings (2026-06-05)

| Sub-defect | Verdict | Root cause (code-confirmed) |
|------------|---------|----------------------------|
| **AG** (Income always 0) | CONFIRMED | Monthly buckets accumulate **net daily delta** only. `categorize_delta` assigns Income only when `delta >= 0` at day level. Heavy recurring/rolling outflows make most days net-negative; salary/income recurring amounts are **not attributed separately** — they net into rolling average or lose to negative net days. `category_names` loaded in `service.rs` but **unused** in projection. |
| **AH** (Fixed always 0) | CONFIRMED | For `delta < 0`, `categorize_delta` passes **`None`** to `map_category` → always **Variable**. Rent/utilities recurring patterns fire on due days but bucket as Variable. `RecurringPattern` has **no `category_id`** — detection uses description only (`recurring.rs`). |

**Aggregation path (traced):**

1. `run_projection` → `category_name_map()` (`categories.firefly_id` → `name`) + `fetch_transactions_for_account` (`category_id` present post-BUG-0006).
2. `project_account` daily loop: `delta = rolling.daily_rate + Σ recurring due amounts`.
3. Single `categorize_delta(delta, category_names, config)` per day → monthly_map accumulation.
4. `bulk_insert_monthly` → `forecast_cashflow_monthly` hypertable.
5. `GET /api/v1/forecast/monthly` reads latest baseline computation; UI `ForecastPage` Monthly tab shows `series[0]` stat cards + `MonthlyChart`.

**DEC-0007 config:** `[forecast.category_buckets]` in `backend/config/default.toml` maps names (`salary`, `rent`, `utilities`, …) to buckets; `map_category` unit tests pass — projection path never supplies category names.

**Impacted surfaces:**

| Surface | Role |
|---------|------|
| `backend/src/forecast/project.rs` | `categorize_delta`, daily/monthly accumulation |
| `backend/src/forecast/types.rs` | `RecurringPattern` (missing category) |
| `backend/src/forecast/recurring.rs` | Pattern detection drops `category_id` |
| `backend/src/forecast/categories.rs` | `map_category` (working; uncalled with names) |
| `backend/src/forecast/service.rs` | Loads `category_names`; passes through |
| `backend/src/forecast/repository.rs` | `category_name_map`, `fetch_monthly_series` |
| `backend/src/api/forecast.rs` | Monthly API (read path OK) |
| `frontend/src/pages/ForecastPage.tsx` | Monthly stat cards (display only) |
| `frontend/src/components/forecast/MonthlyChart.tsx` | Chart series (display only) |

**Fix tasks (for research/architecture — not execute):**

| ID | Sub-defect | Task |
|----|------------|------|
| AG1 | AG | Attribute **income-category recurring inflows** to Income bucket separately from net daily delta (decompose rolling vs recurring components before bucket assignment). |
| AG2 | AG | Resolve `category_id` → name via `category_names` for positive flows; apply `map_category` per DEC-0007 / TOML buckets. |
| AH1 | AH | Attribute **fixed-category recurring outflows** (rent, utilities, standing orders) to Fixed bucket via category map — not Variable. |
| AH2 | AH | Extend `RecurringPattern` (or due-date lookup) with dominant `category_id` from grouped transactions. |
| AH3 | AG/AH | Replace `map_category(None, config)` path; remove `let _ = category_names` dead binding. |
| REG1 | AG/AH | Regression: Variable still non-zero for discretionary spend; `free_cashflow` math preserved. |

**Test hints:**

- Unit: `project_account` with salary (+) and rent (−) recurring txs carrying `category_id` → first month `income > 0`, `fixed_costs > 0`, `variable_costs > 0`.
- Unit: extend `categories.rs` / `project.rs` tests beyond current `map_category` name-only cases.
- Integration: `forecast_integration` seeded mirror with categorized salary/rent → `GET /api/v1/forecast/monthly` non-zero Income/Fixed.
- Operator smoke: Full Firefly sync + recompute on omniflow; `/forecast` Monthly tab stat cards non-zero when mirror has categorized inflows/outflows (acceptance AG/AH).

**Out of scope (unchanged):** US-0015 AI bucket inference; US-0013 ML overlay; BUG-0007 AI chat categorization.

**Recommended next phase:** `/research` (component-level bucket attribution patterns; Firefly category name ↔ TOML bucket map alignment).

#### Sprint plan (2026-06-05 — `sprint-plan-20260605-q0014-bug0012`)

**Quick sprint:** **Q0014** — 5 tasks (~7.5h); no split (5 ≤ `SPRINT_MAX_TASKS` 12).

| Order | Task | Acceptance hook |
|-------|------|-----------------|
| 1 | **AH1** — RecurringPattern.category_id + recurring bucket path | **(AH)** |
| 2 | **AG1** — Income from categorized recurring inflows | **(AG)**, **(AH)** |
| 3 | **T1** — Unit tests monthly_map component attribution | **(AG)**, **(AH)** |
| 4 | **D1** — Retire net-delta categorize_delta for monthly buckets | **(AG)**, **(AH)** |
| 5 | **V1** — verify-work + runbook TOML checklist | **(AG)**, **(AH)** |

**Operator gates:** deploy AH1–D1 → **manual Full Firefly sync + recompute** → extend `[forecast.category_buckets]` if German/custom labels miss default keys → V1.

**Artifacts:** `sprints/quick/Q0014/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260605-q0014-bug0012`)

**Recommended next phase:** `/plan-verify` on **Q0014** → `/execute`

---

### BUG-0013 — Omniflow analytics regression cluster (Grafana zeros, crypto, budgets)

Status: DONE
Priority: P0

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; post-**US-0015** release (`0.16.0-us0015`, S0016); operator may still need **BACKEND_FRONTEND_DEPLOY** + Full Firefly sync + forecast recompute before smoke. Firefly sync **922+ transactions**; 3 asset accounts. Operator report 2026-06-06. Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. **Defect AI (cashflow + forecast scarcity empty):** Open `/analytics/cashflow` and `/analytics/forecast-horizons`. **Balance forecast with scarcity threshold** chart empty or flat zero; forecast-horizons panels show **0 €** despite funded Giro account (114).
2. **Defect AJ (subscriptions price changes):** Open `/analytics/subscriptions` → **Price changes (90 days)** panel → **No data** when operator expects subscription amount changes.
3. **Defect AK (portfolio crypto/FX/performance):** Open `/analytics/portfolio`. **Crypto value** shows **€0**; **FX incomplete** warning; **performance total return %** → **No data**; mixed-currency warning present.
4. **Defect AL (budgets MTD implausible):** Open `/analytics/budgets`. **Planned MTD −€150K**, **Actual €0**, **Deviation €150K** — operator cannot interpret panel; sums appear wrong vs active plan.
5. **Defect AM (Grafana fetch failures):** Browser console on analytics routes: `handleAnnotationQueryRunnerError TypeError: Failed to fetch` on Grafana `ds/query` or annotation proxy (in-scope). _(MetaMask `contentscript.js` MaxListenersExceededWarning / ObjectMultiplex — **out of scope**, extension noise.)_
6. **Defect AN (crypto not parsed):** After exchange sync with read-only keys configured, crypto holdings do not appear in wealth/portfolio totals — operator sees no crypto value anywhere in analytics.

**expected:**

- **AI:** Cashflow scarcity-threshold chart and forecast-horizons baseline panels show **non-empty signed balances** for representative funded account after sync+recompute — not persistent zeros post-BUG-0009/0010 fixes.
- **AJ:** Subscriptions price-changes panel shows rows when confirmed subscriptions had amount changes in the last 90 days, or documented empty-state when none exist.
- **AK:** Portfolio crypto stat reflects combined exchange holdings when sync populated positions; FX warning only when pricing gaps remain with documented partial totals; performance return % populated when snapshot history exists.
- **AL:** Budgets MTD plan/actual/deviation rows are **plausible** for active plan currency and period — not unexplained **−€150K** planned with **€0** actual unless plan truly defines that magnitude.
- **AM:** `POST /analytics/grafana/api/ds/query` and annotation queries return **200** without browser **Failed to fetch** on omniflow external profile.
- **AN:** Exchange crypto balances flow into `net_worth_snapshots` / portfolio panels when Bitunix (or configured venue) sync succeeds — not permanently **€0** when operator has spot/futures exposure per BUG-0005 contract.

**actual:**

- **AI:** Empty scarcity chart; forecast panels **0 €**; seasonal panel shows **ML unavailable** (may be acceptable when ML off — baseline must still be non-zero).
- **AJ:** Price changes (90d) → **No data**.
- **AK:** Crypto **€0**; FX incomplete; performance **No data**.
- **AL:** Planned MTD **−€150K**, Actual **€0**, Deviation **€150K** — operator reports panel is unclear and sums wrong.
- **AM:** `handleAnnotationQueryRunnerError` **Failed to fetch** in Grafana query path.
- **AN:** Crypto generally not parsed — no visible value in portfolio/wealth analytics.

**evidence_refs:** operator report 2026-06-06; `handoffs/intake_evidence/intake-20260606-omniflow-regression-readme.json`; post-BUG-0009 Y/Z DONE; post-BUG-0010 AA/AB DONE; post-US-0015 release S0016; [R-0076](docs/engineering/research.md#r-0076--omniflow-analytics-regression-hypotheses-post-us-0015)

#### Sub-defect hypothesis table (intake — validate in discovery)

| Symptom | Prior art | Discovery hypothesis |
|---------|-----------|---------------------|
| Failed to fetch | BUG-0001/0003 | Grafana proxy, auth middleware, `DATABASE_HOST`, container not recreated post-deploy |
| Forecast 0 € | BUG-0010 L3 | `starting_balance` NULL → 0; needs re-smoke after US-0015 deploy + recompute |
| Crypto €0 | BUG-0005 O | Futures vs spot; exchange sync phase skipped or pricing gap |
| FX incomplete | wealth/FX service | Missing FX rates for crypto asset codes |
| Budget MTD −150K | budgets dashboard SQL | Plan vs actual join bug, wrong currency, or MTD window |
| Cashflow empty | BUG-0009 Y1 | Default `$account_id` regression or zero-balance account default |

#### Discovery notes (2026-06-08 — `discovery-20260608-bug0013`)

**Probe environment:** `https://financegnome.omniflow.cc` public curl (no secrets). Post-US-0015 release (`0.16.0-us0015`, S0016); latest sync **success** `2026-06-06T18:29:40Z` (`manual_exchanges`); prior Full run **success** `2026-06-06T18:16:31Z` (forecast recompute `18:16:58Z`, baseline). 922+ transactions; 3 asset accounts (114 Giro **−3395.75**, 115/116 **0.00**). Research: [R-0076](docs/engineering/research.md#r-0076--omniflow-analytics-regression-hypotheses-post-us-0015).

| Sub-defect | Verdict | Root cause (confirmed / refuted) | Live evidence |
|------------|---------|----------------------------------|---------------|
| **AI** | **REFUTED (ops/stale)** | **Not a post-US-0015 code regression on baseline panels.** BUG-0009 **Y1** fix live: `$account_id` default **114** (max ABS balance). Account **114** `forecast_balance_daily` cnt=**731**, min=**−141733.35**, max=**−3395.75**; scarcity series non-zero (e.g. **−3395.75** today). Account **116** still flat **0** if manually selected. ML panels empty by design (no `ml_enhanced` computation). Operator zeros likely **pre-recompute** or **ML-only panels** — not missing starting_balance. | `POST /analytics/grafana/api/ds/query` **200**; variable query → 114 first; acct **114** tomorrow stat **−3563.93**; acct **116** all **0**; latest baseline computation `2026-06-06T18:16:58Z` |
| **AJ** | **REFUTED (expected empty)** | Panel SQL correct. **0** `price_increase`/`price_decrease` events in 90d; **54** `billing` events only. Confirmed subscriptions exist but no material price-change events per DEC-0017 thresholds. | price-change COUNT=**0**; patterns confirmed/pending present |
| **AK** | **CONFIRMED (pricing pipeline)** | **7** Bitunix **linear** holdings synced (`INJUSDT`, `SOLUSDT`, …) but **`market_value_eur` NULL** on all rows → crypto stat **€0**. `fx.to_eur` keys on base asset; holdings store **full symbol** as `asset` → `Unpriced`. Snapshot `fx_incomplete=false` but `holdings_top` empty. **Performance %** NULL — only **3** snapshots, no return baseline. Mixed-currency panel shows single-currency OK. | `GET /api/v1/wealth` crypto subtotal **0**, holdings_count **7**; snapshot crypto **0**; `total_return_pct` **NULL** |
| **AL** | **CONFIRMED (SQL semantics)** | **MTD planned sums entire future plan horizon**, not month-to-date. `budgets.json` panel id **5** filter `pdc.ts >= date_trunc('month', CURRENT_DATE)` lacks upper bound `<= CURRENT_DATE` → **730** days summed → **−150337.6**. Correct capped MTD **0** (plan starts **2026-06-07**, today **2026-06-06**; no plan days elapsed). Actual MTD **€0** correct (0 June non-transfer txs). | broken MTD **−150337.6** / actual **0** / deviation **150337.6**; broken day-count **730** vs correct **0** |
| **AM** | **NOT REPRODUCED (deferred)** | **Not a transport regression via curl.** Grafana health **200**; `POST …/api/ds/query` **200**; `GET …/api/annotations` **200**; hashed JS/CSS assets **200**; `<base href="/analytics/grafana/">` present. Browser **Failed to fetch** may be intermittent **WebSocket live**, embed session, or operator-side network — needs authenticated browser repro. MetaMask contentscript **out of scope**. | ds/query + annotations **200**; `app.*.js` **200**; ws without upgrade **400** (expected) |
| **AN** | **CONFIRMED (same as AK)** | Exchange sync **succeeds** (Bitunix **connected**, 7 positions) but **pricing/valuation phase** does not populate EUR columns. `upsert_holdings` stores qty+payload only; `recompute_pnl` skips unpriced symbols. **Not** missing sync — **parse/price gap** for linear futures symbols (BUG-0005 residual on omniflow). | holdings: 7 rows, all `product_type=linear`, all `market_value_eur` **NULL**; last sync `18:29:40Z` |

**Isolation vs US-0015:** Forecast monthly API non-zero buckets post-deploy (`/api/v1/forecast/monthly?account_id=114` income/fixed populated; `ai_mapped=true`). Cluster is **multi-factor**: one **confirmed SQL bug (AL)**, one **confirmed crypto pricing gap (AK/AN)**, remainder **ops/expected-empty/not-reproduced** — **not** a single US-0015 regression.

**Operator gates (before code sprint):**

1. **Full Firefly sync** (not exchanges-only) if operator has not run since US-0015 deploy — confirms forecast/subscription mirror freshness.
2. **Forecast recompute** — baseline panels already non-zero post `18:16:58Z` Full run; re-smoke cashflow/forecast-horizons with `$account_id=114`.
3. **Exchanges-only sync insufficient** for wealth snapshot refresh when no prior forecast_id path — prefer **Full sync** after deploy.

#### Fix tasks (discovery decomposition)

| ID | Sub-defect | Task | Primary files / surface |
|----|------------|------|-------------------------|
| **AL1** | AL | MTD summary SQL: add `AND pdc.ts::date <= CURRENT_DATE` (and mirror in deviation joins if needed); add panel footnote when plan horizon starts mid-month | `grafana/provisioning/dashboards/analytics/budgets.json` |
| **AN1** | AN/AK | Linear futures valuation: map symbol→base asset for FX **or** persist exchange notional/`unrealized_pnl` from Bitunix payload on upsert; ensure `recompute_pnl` updates `market_value_eur` | `backend/src/exchanges/bitunix.rs`, `backend/src/portfolio/pnl.rs`, `backend/src/exchanges/repository.rs` |
| **AK2** | AK | Portfolio performance panel: document min snapshot count empty-state; optional seed from first priced crypto sync | `grafana/provisioning/dashboards/analytics/portfolio.json`, wealth snapshot job |
| **AJ1** | AJ | Subscriptions price-changes empty-state copy when 0 events (optional UX — not defect if no changes) | `grafana/provisioning/dashboards/analytics/subscriptions.json` |
| **AM1** | AM | Research: browser repro for annotation/live WS failures on embed shell; compare Traefik vs same-origin paths | `/research` phase — defer execute |
| **V1** | all | verify-work omniflow smoke post AL1+AN1 deploy + Full sync | acceptance AI–AN |

**Architecture:** [docs/engineering/architecture.md § BUG-0013](../engineering/architecture.md#bug-0013--omniflow-analytics-regression-cluster-budgets-mtd-crypto-pricing) — **DEC-0079** (AL1 MTD SQL), **DEC-0080** (AN1 wallet parse + linear unrealized EUR); execute **AL1**, **AN1**, **V1**; AM waived per R-0077

#### Sprint plan (2026-06-08 — `sprint-plan-20260608-q0020-bug0013`)

**Quick sprint:** **Q0020** — 5 tasks (~8h); 3 mandatory + 2 optional P2; no split (< `SPRINT_MAX_TASKS` 12).  
**quick_task_id:** Q0020  
**sprint_id:** Q0020

| Order | Task | Acceptance hook | Priority |
|-------|------|-----------------|----------|
| 1 | **AL1** — MTD planned `<= CURRENT_DATE` + optional footnote | **AL** | P0 |
| 2 | **AN1** — Wallet parse + linear unrealized EUR + tests | **AN**, **AK** | P0 |
| 3 | **AJ1** — Price-changes empty-state copy | **AJ** | P2 optional |
| 4 | **AK2** — Performance % min-snapshot footnote | **AK** | P2 optional |
| 5 | **V1** — verify-work omniflow smoke | **AI**–**AN** | P0 |

**Sequencing:** AL1 ∥ AN1 parallel; operator gates (deploy, Grafana reload, Full sync) before V1.  
**Artifacts:** `sprints/quick/Q0020/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260608-q0020-bug0013`)

**Recommended next phase:** `/plan-verify`

#### Out of scope

- MetaMask / browser extension `contentscript.js` console warnings
- US-0013 ML overlay production enablement (honest ML-unavailable banner acceptable when ML off)
- Greenfield README creation (**US-0016** DONE)

#### Intake evidence (BUG-0013)

- `intake_run_id`: `intake-20260606-omniflow-regression-readme`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug` (paired with **US-0017** story in same intake run)
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260606-omniflow-regression-readme.json`
- Research: [R-0076](docs/engineering/research.md#r-0076--omniflow-analytics-regression-hypotheses-post-us-0015)

#### Intake decomposition (2026-06-06)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 6 sub-defects (AI–AN) + operator crypto call-out |
| Cross-cutting | Grafana SQL, wealth/FX, exchange sync, forecast engine, budgets SQL, analytics proxy |
| Acceptance breadth | 6 sub-rows AI–AN |
| Risk | Post-US-0015 deploy smoke gap; possible multi-factor regression vs single root cause |

**Alternatives considered:** reopen BUG-0009 — rejected (DONE Q0016; no closure-evidence); reopen BUG-0010 — rejected (DONE Q0013; distinct post-release cluster); merge into US-0017 — rejected (independent user value).

**Related work:** BUG-0009 DONE (panel defaults); BUG-0010 DONE (baseline balances); BUG-0005 DONE (futures ingest); US-0015 DONE (bucket mapping — may need sync+recompute); US-0013 OPEN (ML overlay).

**Recommended next phase:** `/sprint-plan`

---

### BUG-0014 — Post-rebuild omniflow cluster (ML sidecar, crypto display, Grafana zeros, planning delete)

Status: DONE
Priority: P0

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; operator rebuilt `flow-finance-ai` + `grafana` 2026-06-07; operator `.env` sets `FORECAST_ML_ENABLED=true`, `STATS_FORECAST_URL=http://stats-forecast:8090`, `BITUNIX_ENABLED_FUTURES=true`, Grafana embed vars per DEC-0057. Post-**BUG-0013** Q0020 / **US-0013** Q0014 releases. Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. **Defect AO (ML still disabled):** Set `FORECAST_ML_ENABLED=true` and `STATS_FORECAST_URL=http://stats-forecast:8090` in operator env; rebuild external profile. Open Forecast UI or `/analytics/forecast-horizons`. Banner: **"ML forecast not enabled on this deployment… Enable ML-enhanced projections via US-0013"** or equivalent skip copy. `GET /api/v1/forecast/meta` → `ml_status: skipped`, `ml_skipped_reason: sidecar_unavailable`, `ml_computation_id: null`.
2. **Defect AP (crypto portfolio €0):** With Bitunix connected and exchange sync success, Wealth **Crypto** tab and portfolio Grafana panels show **€0** crypto subtotal while exchange summary shows **holdings count 7** (operator: *"exchange bitunix 7 don't know what it means"*). `GET /api/v1/wealth` → `crypto.subtotal_eur: 0`, `holdings_top: []` despite `pnl.unrealized_eur` non-zero when linear positions priced.
3. **Defect AQ (FX incomplete / native currency):** Crypto surfaces show **"FX incomplete: one or more crypto assets could not be priced in EUR"** (Wealth tab and/or portfolio forecast warning) while operator expects **native asset amounts** displayed with **EUR valuation at sync/display time** via public FX or connected CEX (USDT→fiat) pricing — not empty portfolio cards.
4. **Defect AR (Grafana cashflow zeros):** Open `/analytics/cashflow`. **Balance forecast with scarcity threshold** flat at **0**; **Recent daily forecast balances** table shows **balance 0** for current dates; monthly decomposition may show income without costs — operator reports *"Grafana is still missing data"* (screenshot 2026-06-07).
5. **Defect AS (planning delete / limited targets):** Open `/planning`. Operator cannot **delete** unwanted plans from UI (only adjustment delete exists); adjustment **target type** limited to **household / subscription / account** feels unintuitive; flows remain confusing post-**US-0014** — operator reports *"can't delete set up plans"* and *"why only 3 categories?"*.
6. **Defect AT (operator gate gap):** Rebuild command recreates `flow-finance-ai` + `grafana` only — **`stats-forecast` container not started** even though external overlay defines `stats-forecast` with `profiles: [external]` per **US-0013** / **DEC-0076**.

**expected:**

- **AO:** When `FORECAST_ML_ENABLED=true` and sidecar healthy on external profile, ML overlay runs after Full sync + recompute (`ml_computation_id` set) or UI shows accurate **sidecar unreachable** ops guidance with compose service name — not permanent **US-0013 not enabled** copy when env opts in.
- **AP:** Wealth crypto subtotal, exchange cards, and `holdings_top` reflect Bitunix futures wallet equity + priced exposure per **DEC-0080** / **DEC-0064** — not **€0** with **7** holdings when unrealized/wallet equity exists.
- **AQ:** Operator sees **native quantities** (USDT, contract symbols) where applicable and **EUR equivalents** from CEX or public FX at valuation time; **FX incomplete** banner only when assets truly lack price — with listed `unpriced_assets` or documented partial totals.
- **AR:** Cashflow Grafana panels show **non-zero signed balances** for funded account **114** (default `$account_id`) after Full sync + forecast recompute — not persistent flat **0** when API forecast series non-zero.
- **AS:** Operator can **delete** a plan from `/planning` (or documented archive flow); target-type UX documented or expanded beyond three enums if product requires; no silent failures on plan mutations.
- **AT:** External profile `docker compose up` documentation/runbook includes **`stats-forecast`** alongside app + Grafana when ML enabled; operator rebuild smoke starts all three services.

**actual:**

- **AO:** `FORECAST_ML_ENABLED=true` in running container; **no** `stats-forecast` container (`docker ps` empty); `forecast/meta` → `ml_skipped_reason: sidecar_unavailable`; UI shows ML-not-enabled banner.
- **AP:** `GET /api/v1/wealth` → `crypto.subtotal_eur: 0`, Bitunix `holdings_count: 7`, `holdings_top: []`, `pnl.unrealized_eur: 411.74` (live probe 2026-06-07).
- **AQ:** Operator sees FX incomplete copy on crypto surfaces; native-currency + point-in-time EUR display not met for futures/linear symbols.
- **AR:** Screenshot: cashflow chart and daily balance table **0** for June 2028 window; monthly income **3266** without aligned costs/balances — operator perceives missing data.
- **AS:** `DELETE /api/v1/plans/:id` exists in backend; **no** delete-plan control in `PlanningPage.tsx`; adjustment form offers only **household / subscription / account**.
- **AT:** Prior rebuild recreated **two** services only; sidecar never brought up despite **US-0013** external overlay.

**evidence_refs:** operator report 2026-06-07; screenshot cashflow zeros; `handoffs/intake_evidence/intake-20260607-post-rebuild-omniflow.json`; live probes `forecast/meta`, `GET /api/v1/wealth`; [R-0079](docs/engineering/research.md#r-0079--bug-0014-post-rebuild-omniflow-ml-sidecar-crypto-display-grafana-planning); post-**BUG-0013** Q0020; post-**US-0013** S0014; post-**US-0014** S0015

#### Sub-defect hypothesis table (discovery verdicts — 2026-06-09)

| ID | Intake hypothesis | Discovery verdict | Root cause | Boundary | Next |
|----|-------------------|-------------------|------------|----------|------|
| **AO** | Sidecar not in compose set | **CONFIRMED (ops)** | `FORECAST_ML_ENABLED=true` but `stats-forecast` container absent → `ml_skipped_reason: sidecar_unavailable` | Ops gate — not US-0013 code gap | Start sidecar + Full sync; improve Grafana static ML banner copy (research) |
| **AT** | Operator `up` omits third service | **CONFIRMED (ops)** | Rebuild scoped to `flow-finance-ai` + `grafana` only; `docker-compose.external.yml` defines `stats-forecast` `profiles: [external]` | Ops/runbook — [US-0013](docs/user-guides/US-0013.md) documents three services | Runbook/README rebuild smoke (research) |
| **AP** | Wallet/subtotal gap post-DEC-0080 | **CONFIRMED (code residual)** | `crypto.subtotal_eur` sums `market_value_eur` only; linear rows excluded per **DEC-0064**/**DEC-0080**; live probe: `unrealized_eur: 411.74` but `subtotal_eur: 0`, `holdings_top: []`, `holdings_count: 7` — wallet row not surfacing in wealth aggregation or Q0020 image not on rebuilt host | Code — wealth `build_breakdown` + wallet `market_value_eur` persistence | **AP1** research: verify deploy tag + DB wallet row; subtotal vs unrealized display contract |
| **AQ** | FX + native currency | **CONFIRMED (product gap)** | `wealth/service.rs` never populates `unpriced_assets` (always `[]`) → wealth `fx_incomplete: false` while crypto tab `portfolioForecast.fx_incomplete_warning` may still fire; holdings table driven by `holdings_top` (priced rows only) — operator sees count **7** but empty table, no native+EUR pairs | Code + UX — separate from ops | **AQ1** research: wire unpriced list; native qty + EUR column for all holdings; banner gating |
| **AR** | Grafana cashflow zeros | **LIKELY (ops/stale)** — code regression **not confirmed** | `cashflow.json` SQL unchanged; **BUG-0013 AI** refuted for acct **114** after Full sync + recompute; operator screenshot (June 2028 window) suggests wrong time range or unfunded `$account_id` — monthly income **3266** without aligned balances points to stale/mismatched account not panel SQL | Data/account — re-verify before code | Operator smoke acct **114** + Full sync + recompute; **AR1** only if API non-zero but Grafana zero |
| **AS** | Plan delete UI missing | **CONFIRMED (UI gap)** | `DELETE /api/v1/plans/:id` wired in backend; `PlanningPage.tsx` has adjustment delete only — no plan delete mutation/control | Code — React UX; out of **US-0014** scope | **AS1** delete plan with confirmation; **AS2** target-type help copy (enum frozen unless DEC) |

**Discovery boundary split (mandatory):**

| Class | Sub-defects | Action |
|-------|-------------|--------|
| **Ops gate** | AO, AT | `compose up` includes `stats-forecast`; Full sync + forecast recompute before attributing ML/crypto/Grafana symptoms to code |
| **Code gaps** | AP, AQ, AS | Execute after research contracts; AP may need deploy verification gate first |
| **Data/account** | AR | Operator re-smoke acct **114**; defer code unless API/Grafana divergence proven |

**Operator gates (mandatory before sprint):**

1. **BACKEND_FRONTEND_DEPLOY** — confirm Q0020 (`DEC-0080`) image on host rebuilt 2026-06-07.
2. **Start `stats-forecast`** on external profile when `FORECAST_ML_ENABLED=true`.
3. **Full Firefly sync** — not exchanges-only.
4. **Forecast recompute** — baseline panels on acct **114**.

**Quick sprint:** **Q0022** — 8 tasks (~14h); 5 mandatory P0 + AS2 P1 optional + AP2/AR1 conditional; no split (< `SPRINT_MAX_TASKS` 12).  
**quick_task_id:** Q0022  
**sprint_id:** Q0022

| Order | Task | Acceptance hook | Priority |
|-------|------|-----------------|----------|
| 1 | **AO1** — Grafana panel 13 dual-scenario ML copy | **AO** | P0 |
| 2 | **AQ1** — holdings_all + unpriced_assets / fx_incomplete | **AQ** | P0 |
| 3 | **AQ2** — WealthPage native+EUR + unified FX banner | **AQ** | P0 |
| 4 | **AS1** — Delete plan UI + active 409 guard | **AS** | P0 |
| 5 | **AS2** — target_type select + help copy | **AS** | P1 optional |
| 6 | **AP2** — Defensive subtotal + count annotation | **AP** | P0 conditional (AP1 gate) |
| 7 | **AR1** — Cashflow Grafana variable fix | **AR** | P2 conditional (V1 AR gate) |
| 8 | **V1** — verify-work omniflow smoke | **AO**–**AT** | P0 |

**Recommended next phase:** `/plan-verify`

#### Discovery evidence (2026-06-09)

- `orchestrator_run_id`: `auto-20260607-bug0014-001`
- `phase_id`: discovery
- Code audit: `docker-compose.external.yml`, `backend/src/wealth/service.rs`, `backend/src/portfolio/pnl.rs`, `backend/src/exchanges/bitunix.rs`, `frontend/src/pages/PlanningPage.tsx`, `grafana/provisioning/dashboards/analytics/cashflow.json`, `grafana/provisioning/dashboards/analytics/forecast-horizons.json`
- Intake probes: `handoffs/intake_evidence/intake-20260607-post-rebuild-omniflow.json`
- Prior art: **BUG-0013** discovery (AI refuted), **DEC-0080**, **US-0013**/**DEC-0076**, **US-0014**

#### Intake decomposition (2026-06-07)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 6 sub-defects (AO–AT) |
| Cross-cutting | Compose external profile, ML sidecar, wealth/FX pricing, Grafana cashflow SQL, planning React UX |
| Acceptance breadth | 6 sub-rows AO–AT |
| Risk | Mix of ops gate (AO/AT) and residual code gaps (AP/AQ/AR/AS) |

**Alternatives considered:** reopen **BUG-0013** — rejected (DONE Q0020; operator post-rebuild residual); merge into **US-0013** — rejected (epic DONE; runtime deployment gap); single story for native currency — deferred to discovery under **AQ**.

**Related work:** **BUG-0013** DONE (DEC-0079/0080); **US-0013** DONE (DEC-0076 sidecar on external); **US-0014** DONE (planning UX — delete plan out of scope); **BUG-0011** DONE.

#### Intake evidence (BUG-0014)

- `intake_run_id`: `intake-20260607-post-rebuild-omniflow`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260607-post-rebuild-omniflow.json`
- Research: [R-0079](docs/engineering/research.md#r-0079--bug-0014-post-rebuild-omniflow-ml-sidecar-crypto-display-grafana-planning)

---

### BUG-0015 — Confirmed subscriptions reappear as pending after rebuild

Status: DONE
Priority: P1

**environment:** US-0010 external Compose profile on `financegnome.omniflow.cc`; operator rebuilt containers 2026-06-07 (related to **BUG-0014** work); postgres expected to persist on external volume across app/grafana rebuild. Operator previously confirmed subscriptions including **CURSOR, AI POWERED IDE, CURSOR.COM** (95%, Monthly · €17.18) and **APPLE.COM/BILL, ITUNES.COM** (60%, Monthly · €9.99). Do not read `.env` / `.env_prod` secrets.

**steps_to_reproduce:**

1. On omniflow external profile, confirm subscription patterns for recurring merchants (e.g. Cursor, Apple) via `/subscriptions` Confirm action.
2. Rebuild application containers (`flow-finance-ai` and/or related services per operator rebuild scope from **BUG-0014**).
3. Start stack; allow or trigger **Full Firefly sync** and subscription detection phase.
4. Open `/subscriptions` → **Pending review** tab (or **All**).

**expected:**

- Previously **confirmed** patterns remain `status=confirmed` in API and UI — no Confirm/Reject cards for Cursor or Apple.
- Post-sync detection **skips** groups whose fingerprint is in the confirmed set (`load_confirmed_fingerprints` / `upsert_pending_pattern` status preservation).
- Subscription-scoped alert unread count reconciles with visible pending list — no orphan `new_detection` alerts forcing re-review of confirmed merchants.

**actual:**

- Operator reports confirmed subscriptions **reappearing** with **Confirm/Reject** buttons after rebuild and container start.
- Examples: **CURSOR, AI POWERED IDE, CURSOR.COM** — 95%, Monthly · €17.18; **APPLE.COM/BILL, ITUNES.COM** — 60%, Monthly · €9.99.
- Symptom surfaced post-rebuild on `financegnome.omniflow.cc` external profile.

**evidence_refs:** operator report 2026-06-07; `handoffs/intake_evidence/intake-20260607-subscription-reconfirm.json`; code context `backend/src/subscriptions/repository.rs`, `detection.rs`; prior **BUG-0004** (empty subscriptions), **BUG-0008** (alert vs list mismatch); post-**BUG-0014** rebuild; [R-0081](docs/engineering/research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild)

#### Sub-defect hypothesis table (discovery verdicts — 2026-06-07)

| ID | Intake hypothesis | Maps to AC | Discovery verdict | Root cause | Boundary | Next |
|----|-------------------|------------|-------------------|------------|----------|------|
| **H1** | **Fingerprint drift** | AU, AV | **LIKELY PRIMARY (code-confirmed)** | `compute_fingerprint(payee_key, interval_days, median_amount)` hashes all three (`detect.rs` L45–49). `upsert_pending_pattern` preserves `confirmed`/`rejected` **only on same fingerprint** (`repository.rs` L144–147). Post-rebuild Full sync re-groups txs via `extract_payee_source` → `payee_key` (`group.rs` L30–43) and recomputes `median_amount` from recent 6 txs (`detect.rs` L95–97) — any drift yields **new** pending INSERT while prior confirmed row remains on **old** fingerprint. Cursor/Apple long display names are consistent with payee-normalization variance, not DB wipe. | **Code** — fingerprint / merchant-identity contract | **Research:** payee-level confirm inheritance vs stable fingerprint (exclude amount from hash); operator SQL probe to confirm duplicate rows |
| **H2** | **DB ephemeral (ops)** | AU | **UNLIKELY sole cause** | US-0010 external profile binds `DATABASE_HOST=postgres` on traefik network (`docker-compose.external.yml`); **BUG-0014** rebuild scoped app/grafana — postgres not in compose set. Total volume loss would wipe **all** confirms, not merchant-specific re-prompts. Cannot fully refute without operator DB probe. | **Ops gate** — verify persistence | Operator SQL: `SELECT status, COUNT(*) FROM subscription_patterns GROUP BY status` immediately after rebuild **before** Full sync |
| **H3** | **Alert/UI desync** | AW | **REFUTED as primary** | `/subscriptions` Confirm/Reject cards render only for `GET /api/v1/subscriptions?status=pending` rows (`SubscriptionsPage.tsx` L240–261) — not alert-driven. Symptom requires `status=pending` in DB. Post-**BUG-0008** `upsert_alert` dedupes unread by alert fingerprint (`repository.rs` L286–290) but **new** `pattern_id` from H1 drift still emits `new_detection`. H3 may contribute as **secondary** (banner noise) only. | Secondary — alert lifecycle if H1 fixed | AW satisfied when H1 fix prevents duplicate pending rows |
| **H4** | **Detection re-run post-sync** | AV | **CONFIRMED mechanism, subsumed by H1** | Pipeline trace: `sync/mod.rs` L261–264 → `SubscriptionService::run_detection` loads `confirmed_fps` **before** emit (`service.rs` L40–44); `detection.rs` L43–44 skips exact fingerprint match. Re-run is **expected** after rebuild; skip logic works when fingerprint stable. Failure mode is fingerprint **change** on re-group (H1), not missing skip. | **Code** (same as H1) | No separate fix — address via H1 research |

**Discovery verdict (summary):** H1 **LIKELY PRIMARY**; H2 **UNLIKELY sole** (ops verify gate); H3 **REFUTED primary**; H4 **subsumed by H1**. Recommended fix boundary: **code** (fingerprint stability or payee-level confirm propagation); **ops** pre-fix gate to rule out H2.

**Discovery notes (2026-06-07 — `discovery-20260607-bug0015`, orchestrator `auto-20260607-bug0015-001`):**

**Probe environment:** Code-path audit in isolated PO context; no `.env` / `.env_prod` secrets; operator omniflow report + intake evidence. Runtime DB probe **deferred to research** (operator gate).

**Code anchors:**

- Fingerprint: `backend/src/recurrence/detect.rs` `compute_fingerprint` — `payee_key:interval_days:rounded_amount`
- Grouping: `backend/src/recurrence/group.rs` `extract_payee_source` — description priority (counterparty for transfer-shaped memos)
- Normalize: `backend/src/recurrence/normalize.rs` `payee_key` — SEPA strip, trailing reference codes, legal suffix collapse
- Preserve: `backend/src/subscriptions/repository.rs` `upsert_pending_pattern` ON CONFLICT status CASE
- Skip: `backend/src/subscriptions/detection.rs` L42–44 `confirmed_fps.contains`
- Orchestration: `backend/src/sync/mod.rs` subscriptions phase after successful Firefly sync

**Recommended next phase:** `/research`

#### Intake decomposition (2026-06-07)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 1 operator workflow (rebuild → subscriptions review) |
| Cross-cutting | Postgres persistence, fingerprint contract, detection pipeline, subscriptions UI, alert dedup |
| Acceptance breadth | 3 AC rows (AU–AW) + 4 intake hypotheses (H1–H4) for discovery |
| Risk | Data integrity + operator trust; may be ops (volume) or code (fingerprint drift) |

**Alternatives considered:** reopen **BUG-0008** — rejected (DONE; distinct confirm-persistence-after-rebuild symptom); merge into **BUG-0014** — rejected (BUG-0014 DONE; independent US-0003 contract); split fingerprint vs DB into separate BUG ids — rejected (jointly testable on same rebuild smoke).

**Related work:** **US-0003** (subscription confirm contract); **BUG-0008** DONE (alert vs list — coordinate AW hypothesis); **BUG-0014** DONE (operator rebuild context); **BUG-0004** DONE (detection baseline).

#### Intake evidence (BUG-0015)

- `intake_run_id`: `intake-20260607-subscription-reconfirm`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260607-subscription-reconfirm.json`
- Research: [R-0081](docs/engineering/research.md#r-0081--bug-0015-confirmed-subscription-reconfirm-after-rebuild)

**Quick sprint:** **Q0023** — 5 tasks (~13h); all P0 mandatory; no split (< `SPRINT_MAX_TASKS` 12).  
**quick_task_id:** Q0023  
**sprint_id:** Q0023

| Order | Task | Acceptance hook | Priority |
|-------|------|-----------------|----------|
| 1 | **AU1** — Card billing `payee_key` normalization | **AU**, **AV** | P0 |
| 2 | **AU2** — Payee+interval maps + merge upsert + index | **AU**, **AV** | P0 |
| 3 | **AU3** — Detection skip + merge path | **AU**, **AV**, **AW** | P0 |
| 4 | **AU4** — Stale inactive by payee+interval | **AV** | P0 |
| 5 | **V1** — verify-work rebuild smoke | **AU**–**AW** | P0 |

**Operator gates (before V1):**

1. **BACKEND_FRONTEND_DEPLOY** — ship Q0023 backend on omniflow.
2. **POSTGRES_PERSISTENCE_PROBE** — H2 SQL on `subscription_patterns` after rebuild, before Full sync.
3. **FULL_FIREFLY_SYNC** — Full sync + subscription detection phase.

**Recommended next phase:** `/plan-verify`

#### Sprint-plan evidence (2026-06-07)

- `orchestrator_run_id`: `auto-20260607-bug0015-001`
- `phase_id`: sprint-plan
- Architecture: DEC-0084, DEC-0085, DEC-0086
- Sprint artifacts: `sprints/quick/Q0023/`
- Handoff: `handoffs/tl_to_dev.md` (`sprint-plan-20260607-q0023-bug0015`)

---

## User stories (canonical)

### US-0001 — Self-hosted platform foundation & Firefly read-only integration

Status: DONE
Priority: P0

As a self-hosting user with Firefly III,
I want a deployable Flow Finance AI stack that syncs my Firefly data read-only,
So that I have a secure foundation for all analytics without altering my ledger.

#### Scope

- In: Docker Compose (minimal + standard profiles), external PostgreSQL config (never embedded), Rust/Axum/Tokio backend skeleton, React/TypeScript/Tailwind/shadcn UI shell, OIDC auth wiring, Firefly Connector (accounts, transactions, categories, budgets, tags, piggy banks), sync scheduler, read-only guarantee
- Out: Forecasting logic, subscription detection, planning, AI, crypto, Grafana dashboard content

#### Constraints

- Firefly III is the only transaction source; no writes to Firefly
- PostgreSQL must be configurable as external via TOML/env (per Projectplan)
- Dev environment: Docker available, Firefly III running and API-testable (per operator note)

#### Intake decomposition

- Split decision: **multi-story** (9 vertical slices aligned with Projectplan roadmap phases)
- Rationale: broad cross-cutting system; vertical slices preserve independent user value per phase
- Boundaries: US-0001 delivers deployable platform + data ingest only; downstream stories consume synced data

#### Discovery refinements (2026-05-31)

- **UI shell pages (in scope):** Home/Dashboard placeholder, Sync Status, Settings (Firefly connection + sync interval display)
- **Nav placeholders (disabled):** Forecast, Subscriptions, Planning, Wealth, AI — visible in sidebar with "Coming soon" badge; routes not implemented
- **Grafana (in scope):** Service provisioning + datasource wiring in minimal Compose profile; optional Platform Health dashboard (sync metrics, API health). Analytics dashboards 1–5 explicitly out of scope (US-0002+)
- **Compose profiles:** `minimal` = flow-finance-ai + firefly-iii + grafana; `standard` adds redis; `full` adds ollama (per Projectplan)
- **Firefly auth:** Personal Access Token preferred for connector (per R-0001); OAuth2 supported as alternative
- **Sync Status UX:** Entity counts by Firefly type, last sync timestamp, manual sync trigger, sync history log with status badges
- **Read-only indicator:** Persistent header badge; verified via integration test or audit log (acceptance unchanged)
- **OIDC:** Redirect login + session in sidebar footer; provider config via Compose/env only (no in-app OIDC admin in US-0001)

---

### US-0002 — Cashflow forecasting & Grafana analytics (MVP)

Status: DONE
Priority: P0

As a household budgeter,
I want daily, monthly, and long-term cashflow forecasts with Grafana dashboards,
So that I can anticipate account balances and plan spending.

#### Scope

- In: Forecast Engine (daily balance, weekly, month-end; monthly income/fix/variable/free cashflow; 3/6/12/24 month horizons), React forecast views, Grafana Dashboard 1 (Cashflow) and Dashboard 5 (Forecast), TimescaleDB time-series storage
- Out: ML-based forecasting (US-0009), subscription-driven forecast adjustments (US-0003), plan scenario overlays (US-0004)

#### Constraints

- Forecasts derived from Firefly-synced data (US-0001 dependency)
- Read-only toward Firefly
- TimescaleDB hypertables on external PostgreSQL (operator prerequisite per R-0004)
- Forecast recompute triggered on successful Firefly sync completion

#### Discovery refinements (2026-05-31)

- **Forecast Engine outputs:** Daily (tomorrow, next week, month-end balance per account); monthly (income, fixed costs, variable costs, free cashflow); long-term (3 / 6 / 12 / 24 month balance projections)
- **React UI — `/forecast` route:** Enable Forecast nav item (replace US-0001 disabled placeholder); account selector; Daily | Monthly | Long-term horizon tabs; ECharts charts per view; last-computed timestamp
- **Backend API:** Forecast endpoints serving precomputed series from TimescaleDB; recompute job/post-sync hook after Firefly sync
- **TimescaleDB:** Hypertables for forecast snapshot time series (per R-0004 pattern); migration in US-0002 scope
- **Grafana Dashboard 1 (Cashflow):** Balance + forecast overlay + scarcity threshold reference lines (visual markers only; Alert Engine US-0005)
- **Grafana Dashboard 5 (Forecast):** Horizon panels 1 / 3 / 6 / 12 months (Projectplan); optional 24-month panel to match React long-term selector
- **Grafana provisioning:** Dashboard JSON alongside existing Platform Health dashboard; datasource reuse from US-0001
- **Out of scope unchanged:** ML forecasting (US-0009), subscription-adjusted forecasts (US-0003), plan scenario overlays (US-0004), Dashboards 2–4

#### Discovery decomposition evidence

- Feature/workflow count: 3 forecast granularities + React page + 2 Grafana dashboards + hypertable storage (moderate breadth, single vertical slice)
- Cross-cutting impact: backend engine, DB migration, React UI, Grafana provisioning
- Acceptance breadth: unchanged (8 criteria in `docs/product/acceptance.md`)
- Risk surface: forecast algorithm accuracy on sparse history, hypertable migration on external TimescaleDB (R-0004), sync-triggered recompute latency

---

### US-0003 — Subscription detection, price changes & alerts

Status: DONE
Priority: P1

As a subscriber to many services,
I want automatic detection of recurring payments with confirm/reject and price-change alerts,
So that I control my subscriptions and spot cost increases early.

#### Scope

- In: Subscription Detection Engine (amount, payee, text, regularity, intervals), confidence scores (95/80/60%), user confirm/reject UI, standing-order (Dauerauftrag) detection, price change detection (increases/decreases/frequency), alerts for new subscriptions and price changes, Grafana Dashboard 2 (Subscriptions)
- Out: Plan scenario impact of cancelling subscriptions (US-0004), AI natural-language queries (US-0006)

#### Constraints

- User confirmation required before treating a pattern as a confirmed subscription
- Detection runs on synced Firefly transactions only

#### Discovery refinements (2026-05-31)

- **Subscription Detection Engine:** Analyze synced transactions by amount, payee, description text, regularity, and interval stability; emit confidence tiers **95% / 80% / 60%** (Projectplan); extend US-0002 `detect_patterns` heuristics with persisted candidate/confirmed/rejected states
- **React UI — `/subscriptions` route:** Enable Subscriptions nav (replace US-0001 disabled placeholder); tabs **All | Pending review | Standing orders**; confirm/reject cards for pending; confirmed table with interval, amount, type badge
- **Standing orders (Dauerauftrag):** Separate classification/tab for fixed recurring outflows (rent, insurance, utilities) vs discretionary subscriptions
- **Price-change detection:** Track amount history per confirmed pattern; surface increases, decreases, and interval/frequency shifts; detail drawer with ECharts price history
- **Subscription-scoped alerts:** In-app notification for new detection and price change on confirmed subscription (banner/toast + optional on-page alert strip); not US-0005 Alert Engine inbox
- **Forecast integration:** Confirmed subscriptions override forecast recurring heuristics for matching payee/description; **rejected** patterns excluded from forecast projection and alerts (acceptance AC-8)
- **Detection trigger:** Run after successful Firefly sync (same boundary as forecast recompute per DEC-0010); pending queue refreshed on sync completion
- **Grafana Dashboard 2 (Subscriptions):** Panels — all subscriptions (count + monthly spend), price changes, new/pending detections; uid `subscriptions` per DEC-0012 pattern
- **Out of scope unchanged:** Plan scenario cancel-impact (US-0004), AI `get_subscriptions` tool wiring detail (US-0006 implements consumer), full scarcity/budget alert inbox (US-0005)

#### Discovery decomposition evidence

- Feature/workflow count: detection engine + pending confirm/reject + confirmed list + standing-order tab + price-change history + subscription alerts + Grafana Dashboard 2 (moderate-high breadth — single vertical slice retained)
- Cross-cutting impact: backend detection service, DB migration for subscription entities, React `/subscriptions` UI, forecast engine integration, Grafana provisioning
- Acceptance breadth: unchanged (8 criteria in `docs/product/acceptance.md`)
- Risk surface: false-positive detection before confirm, Dauerauftrag vs subscription disambiguation, price-change sensitivity, sync-triggered detection latency, confidence tier calibration

---

### US-0004 — Financial planning, scenarios & plan-vs-actual

Status: DONE
Priority: P1

As a financial planner,
I want to create versioned scenarios and compare plan vs actual daily,
So that I can evaluate life decisions (leasing, savings mode, major purchases) with evidence.

#### Scope

- In: Plan Engine (scenarios: current, leasing, savings mode, house purchase, custom adjustments), plan versioning (v1/v2/v3 compare), plan-vs-Ist daily comparison with deviation, Grafana Dashboard 3 (Budgets: plan/ist/deviation)
- Out: AI-driven plan simulation chat (US-0006), crypto allocation scenarios (US-0007)

#### Constraints

- Plans stored in Flow Finance AI DB; Firefly remains read-only source for actuals
- Scenario changes are explicit user-defined deltas (e.g. +300 €/month leasing)

#### Discovery refinements (2026-05-31)

- **Plan Engine:** User-defined scenario deltas applied to US-0002 forecast baseline; plans persisted in Flow DB; one **active plan** drives plan-vs-Ist and Grafana Dashboard 3
- **Built-in scenario templates:** **Current (Ist)** — baseline without deltas; **Leasing** — add recurring outflow (e.g. +300 €/month); **Savings mode** — remove selected confirmed subscriptions (US-0003) and/or discretionary cuts (e.g. −100 €/month dining); **House purchase** — increase savings rate / recurring transfer to savings
- **Custom adjustments:** Named plan with explicit delta lines (amount, frequency monthly default, target: subscription pick-list / Firefly category / custom label)
- **Plan versioning:** Each named plan supports versions **v1 / v2 / v3**; create new version from prior; **Compare** view shows side-by-side metrics (monthly delta sum, projected month-end balance per version)
- **React UI — `/planning` route:** Enable Planning nav (replace US-0001 disabled placeholder); Tabs **Scenarios** | **Compare** | **Plan vs Actual**; active plan selector in page header
- **Plan vs Actual (daily):** For active plan, show daily **planned**, **actual (Ist)**, and **deviation** amounts; default view current calendar month; actuals aggregated from synced Firefly transactions (read-only)
- **Forecast integration:** Current (Ist) scenario aligns with latest successful forecast computation; scenario deltas adjust projected recurring/cashflow layer (not Firefly data)
- **Grafana Dashboard 3 (Budgets):** Panels — **Plan**, **Ist**, **Abweichung** (deviation) for active plan; uid `budgets` per DEC-0012 pattern; datasource reuse from US-0001
- **Out of scope unchanged:** AI `simulate_plan` chat (US-0006), crypto allocation scenarios (US-0007), plan viability / budget-drift Alert Engine inbox (US-0005)

#### Discovery decomposition evidence

- Feature/workflow count: Plan Engine + 4 templates + custom deltas + version compare + daily plan-vs-Ist + React page + Grafana Dashboard 3 (moderate-high breadth — single vertical slice retained)
- Cross-cutting impact: backend Plan Engine, DB migration for plan entities, React `/planning` UI, forecast baseline integration, Grafana provisioning
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md`)
- Risk surface: scenario delta modeling vs forecast baseline, daily Ist aggregation when sync lags, v1/v2/v3 compare UX breadth, Dashboard 3 SQL joining plan snapshots + Firefly mirror actuals, read-only actuals constraint

---

### US-0005 — Wealth analysis, budget drift & scarcity alerts

Status: DONE
Priority: P1

As a wealth-conscious user,
I want aggregated net worth and proactive alerts for scarcity and budget drift,
So that I notice problems before they become crises.

#### Scope

- In: Net worth aggregation (giro, savings, Firefly accounts; crypto placeholder until US-0007), Alert Engine (scarcity threshold, budget drift %, plan viability warnings), Grafana Dashboard 4 (Portfolio/wealth partial), React alert inbox
- Out: Full crypto portfolio PnL (US-0007), Grafana alert rule provisioning details (may extend in execute)

#### Constraints

- Alerts must be actionable with clear threshold configuration
- Wealth view excludes crypto balances until US-0007 integrates exchange data

#### Discovery refinements (2026-05-31)

- **Net worth aggregation:** Sum synced Firefly **asset** accounts (giro/checking, savings, cash, other asset types); household total in reporting currency (EUR default per DEC-0021); per-account breakdown table with native currency labels; **crypto placeholder row** ("Connect exchanges — US-0007") excluded from total until US-0007
- **Alert Engine:** Centralized evaluation after post-sync pipeline completes (forecast recompute + active-plan hook per DEC-0010 / DEC-0023); persist alerts in Flow DB with type, severity, entity reference, and lifecycle (active / acknowledged / dismissed)
- **Alert types (MVP):** **Scarcity** — projected balance (US-0002 forecast path) falls below configurable threshold (default €200, Projectplan); **Budget drift** — category actual spend exceeds active-plan category target by configurable % (default +20%, Projectplan "Lebensmittel +20%"); **Plan viability** — active plan scenario projects infeasible month-end balance or sustained deficit per forecast overlay (Projectplan "Leasing Plan nicht tragfähig")
- **Threshold config:** TOML `[alerts]` — `scarcity_threshold_eur`, `budget_drift_pct`, optional per-category overrides deferred; **centralize** Dashboard 1 scarcity reference line to same config source (replace DEC-0012 static €200 hardcode in execute)
- **React UI — `/wealth` route:** Enable Wealth nav (replace US-0001 disabled placeholder); **Overview** tab — net worth stat card, account breakdown table, optional ECharts stacked bar by account type; link to Grafana Dashboard 4
- **React alert inbox:** Header **notification bell** with unread badge (deferred from US-0003 per R-0011); dropdown preview (latest 5) + full **`/alerts`** page — list active alerts with type icon, message, timestamp; **Acknowledge** (read) and **Dismiss** (suppress until condition clears / re-triggers)
- **US-0003 boundary:** Subscription-scoped alerts (new abo, price change) remain on `/subscriptions` banner/toast only — **not migrated** to unified inbox; optional "View subscriptions" link from wealth header when unread subscription alerts exist (no duplicate firing)
- **Grafana Dashboard 4 (Portfolio partial):** uid `portfolio` per DEC-0012 pattern; panels — **total wealth** stat (non-crypto sum), account breakdown table/pie, optional wealth-over-time from account balance snapshots; **crypto** and **performance** panels deferred to US-0007
- **Out of scope unchanged:** Full crypto PnL and exchange connectors (US-0007), Grafana Alertmanager rule provisioning, AI `get_budget_status` / `get_portfolio` tool implementation detail (US-0006), subscription alert entity migration, multi-currency conversion beyond reporting-currency display

#### Discovery decomposition evidence

- Feature/workflow count: net worth view + 3 alert types + threshold config centralization + header bell + `/alerts` inbox + `/wealth` page + Grafana Dashboard 4 partial (moderate-high breadth — single vertical slice retained)
- Cross-cutting impact: backend Alert Engine, DB migration for alerts + net-worth snapshots, React UI (wealth + alerts + header), forecast/plan integration, Grafana Dashboard 4 + Dashboard 1 threshold wiring
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md`)
- Risk surface: alert dedup vs US-0003 subscription alerts, budget-drift metric grain (category vs household), plan-viability rule definition, Dashboard 1 threshold migration, net-worth multi-currency MVP, post-sync evaluation latency

---

### US-0006 — AI financial assistant with privacy-safe tool layer

Status: DONE
Priority: P2

As a user exploring my finances,
I want to ask natural-language questions answered via structured tools,
So that I get insights without exposing raw data to uncontrolled AI database access.

#### Scope

- In: AI Tool Layer (`get_transactions`, `get_subscriptions`, `get_forecast`, `get_budget_status`, `get_portfolio`, `simulate_plan`), OpenAI provider integration, chat UI, Privacy Layer (redact IBAN/counterparties, optional raw transaction suppression)
- Out: Local/self-hosted AI providers (US-0008), advanced ML forecasts (US-0009)

#### Scope refinements (discovery 2026-05-31)

- **AI Tool Layer:** Six registered tools exposed via OpenAI function calling; each wraps existing Flow REST/services (forecast, subscriptions, wealth, plans) — no SQL from AI execution path
- **OpenAI provider:** Configurable via TOML `[ai]` section + env override; API key in self-hosted secrets only; model name configurable
- **Chat UI:** Header **AI** button opens `Sheet` drawer with shared `ChatPanel`; full-page **`/chat`** route for extended sessions; sidebar AI nav enabled
- **Suggested prompts:** Empty-state chips for Projectplan example queries (affordability, subscription price changes, budget overrun, cancel savings, top categories)
- **Tool transparency:** Collapsible "Tools used" row under assistant messages (tool name + timestamp; no raw JSON)
- **Privacy Layer:** TOML `[privacy]` defaults per Projectplan — `allow_raw_transactions=false`, `redact_iban=true`, `redact_counterparties=true`; chat header privacy badge; Settings **AI & Privacy** section
- **Tool audit log:** Operator-visible recent invocations (timestamp, tool, session/user, redacted arg summary, duration, status)
- **Out of scope unchanged:** Local/self-hosted providers (US-0008), ML forecasts (US-0009), Grafana AI dashboard, inline ECharts in chat (defer if sprint tight)

#### Constraints

- AI must **never** access PostgreSQL directly — tools only
- Default privacy-redaction settings per Projectplan TOML options
- `simulate_plan` read-only — invokes plan APIs only; no Firefly mutation (DEC-0004)

#### Discovery decomposition evidence

- Feature/workflow count: 6 tools + OpenAI integration + privacy redaction + chat drawer + `/chat` page + audit log + Settings privacy + example queries (moderate-high breadth — single vertical slice retained)
- Cross-cutting impact: backend AI orchestration + tool registry, privacy middleware, React chat UI (header + route + Settings), OpenAI client, audit persistence, existing API consumers (forecast/subscriptions/wealth/plans)
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md`)
- Risk surface: privacy redaction bypass, OpenAI key handling, tool payload size vs context window, prompt injection, streaming auth (DEC-0006), scope creep into US-0008 local providers

---

### US-0007 — Crypto exchange portfolio integration

Status: DONE
Priority: P2

As a crypto investor,
I want Binance, Bybit, and Bitunix balances and PnL integrated into my net worth,
So that my total wealth picture is complete and plan-able.

#### Scope

- In: Exchange connectors (Binance, Bybit, Bitunix — start set), import wallets/balances/positions/trades/transfers/funding/PnL, Portfolio Engine (realized/unrealized gains, total return), crypto slice in wealth analysis (US-0005 extension), portfolio allocation planning scenarios, Grafana Dashboard 4 completion
- Out: Additional exchanges (Kraken, Coinbase, Bitpanda, OKX — future), on-chain wallet tracking

#### Constraints

- Phase 2 per Projectplan; depends on US-0001 platform and US-0005 wealth framework
- API keys stored securely in self-hosted config (never cloud)

#### Discovery refinements (2026-06-01)

- **Exchange connectors (start set):** **Binance**, **Bybit**, **Bitunix** — read-only API keys; import wallets/balances, positions, trades, transfers, funding, exchange-reported PnL where available; unified connector trait per exchange (Projectplan Phase 5)
- **Portfolio Engine:** Compute **realized gains** (closed positions / sold lots), **unrealized gains** (open positions mark-to-market), **total return** (% and absolute since first sync baseline); persist holdings snapshots and trade history in Flow DB — not Firefly
- **Sync integration:** Exchange sync runs on configurable interval + manual trigger; extends **Sync Status** page with per-exchange rows (connection state, last sync, entity counts); post-sync pipeline adds **exchanges** phase before net-worth snapshot (after forecast/plan hook, before alerts per DEC-0028 extension)
- **Net worth extension (US-0005):** Replace crypto placeholder row with live exchange totals; **headline net worth includes crypto** converted to reporting currency (EUR default); Firefly asset subtotal + crypto subtotal stat cards; mixed-currency banner retained when FX incomplete
- **React UI — `/wealth` extension:** Enable **Crypto** tab (Overview | Crypto); Overview shows combined total; Crypto tab — per-exchange stat cards, holdings table (asset, quantity, value EUR, unrealized PnL), PnL summary row (realized / unrealized / total return); link to Settings when no exchanges connected
- **Settings — Exchange connections:** New section **Crypto exchanges** — add/edit Binance/Bybit/Bitunix credentials (masked display), test connection, enable/disable per exchange; secrets via TOML/env only (never browser storage)
- **Planning allocation scenarios:** Extend Plan Engine with **Allocation target** template — user sets target weights (e.g. 50% ETF / 50% crypto, 70% ETF / 20% crypto / 10% cash per Projectplan); compare **current allocation** (from wealth + crypto breakdown) vs target on Planning **Scenarios** tab or dedicated allocation card on `/wealth` (MVP: read-only gap display + link to `/planning`)
- **Grafana Dashboard 4 completion:** Replace crypto placeholder panel with **Crypto value** stat, **allocation pie** (Firefly assets vs crypto by asset), **portfolio performance** time series (total return from snapshots); extend `net_worth_snapshots` payload to include crypto slice
- **AI tool boundary:** `get_portfolio` narrative includes crypto totals and top holdings once connected (extends R-0031); no new AI tools in US-0007
- **Out of scope unchanged:** Kraken, Coinbase, Bitpanda, OKX; on-chain wallet tracking; tax reporting; trading execution; futures-only advanced analytics beyond exchange API import

#### Discovery decomposition evidence

- Feature/workflow count: 3 exchange connectors + portfolio engine + PnL + wealth UI extension + Settings credentials + sync status + allocation scenario template + Grafana Dashboard 4 completion (moderate-high breadth — single vertical slice retained per intake plan-area `crypto-portfolio`)
- Cross-cutting impact: backend exchange connectors, portfolio engine, DB migration, React UI (wealth + settings + sync status), plan allocation template, wealth/AI tool extension, Grafana provisioning, post-sync pipeline
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md`)
- Risk surface: API key security and rotation, exchange rate limits, FX conversion source for crypto→EUR, PnL methodology vs exchange reports, spot vs futures/funding scope, sync mutex latency growth, mixed-currency headline accuracy

---

### US-0008 — Local & self-hosted AI provider support

Status: DONE
Priority: P3

As a privacy-first operator,
I want to use Ollama, LM Studio, or OpenAI-compatible local endpoints for the AI assistant,
So that financial Q&A never leaves my infrastructure.

#### Scope

- In: Extend stub `AiProvider` trait over US-0006 orchestrator (Chat Completions + tools + SSE unchanged at tool layer); three provider modes — `openai` (existing), `ollama` (native base URL default `http://ollama:11434/v1`), `openai_compatible` (LM Studio / LocalAI / vLLM via configurable `base_url`); TOML `[ai]` schema extension (`provider`, `base_url`, optional `api_key_env`); Settings **AI & Privacy** read-only provider table + status badge + **Test AI provider** button; chat **Local vs Cloud** provider badge; Docker Compose `full` profile Ollama service wiring documented with backend dependency when `provider = "ollama"`; operator user guide for model pull + local E2E verification
- Out: Model fine-tuning; GPU orchestration beyond Compose profiles; in-app model catalog/pull UI; runtime provider switching without restart; user message pre-redaction; token vault / NER rehydration (DEC-0032 deferral)

#### Constraints

- Must reuse US-0006 tool registry, PrivacyLayer, orchestrator loop, and six tools unchanged — provider swap at HTTP client layer only
- OpenAI fallback remains available when `provider = "openai"` and API key configured
- Config edit + backend restart required to change provider (same pattern as US-0006 privacy toggles and US-0007 exchange secrets)
- Local E2E must produce no outbound call to `api.openai.com` when local provider selected and configured

#### Discovery decomposition evidence (2026-06-02)

- Feature/workflow count: provider factory + 3 provider paths + config schema + settings display/test + chat provider badge + compose Ollama wiring + operator guide + local E2E verification (moderate breadth — single vertical slice retained; split axis would be "provider backend" vs "operator UX" but neither slice independently satisfies AC5 alone)
- Cross-cutting impact: `backend/src/ai/provider`, config/settings API, orchestrator factory, React Settings + ChatPanel badges, docker-compose docs, user guide US-0008
- Acceptance breadth: unchanged (5 criteria in `docs/product/acceptance.md#US-0008`)
- Risk surface: local model tool-calling reliability vs OpenAI (R-0027); OpenAI-compatible endpoint variance (`tool_choice`, parallel tools, streaming deltas); smaller context windows vs 8 KB tool payloads (DEC-0035); SSE auth across providers (R-0029); scope creep into model management or outbound redaction

#### Discovery open questions (carry to `/research`)

- Recommended Ollama models for reliable function calling with six finance tools?
- Orchestrator fallback when local model omits `tool_calls` or returns malformed JSON arguments?
- Single `OpenAiCompatibleProvider` implementation vs separate Ollama type given shared `/v1/chat/completions` contract?
- Optional vs required `api_key_env` for local providers (Ollama dummy key pattern)?
- Settings API shape: `provider_configured`, `provider_label`, `base_url`, `is_local` flags?
- Compose: conditional `depends_on: ollama` vs always-on in `full` profile?
- Audit log: persist provider name per invocation for operator traceability?
- Network isolation test strategy for AC5 (mock server vs compose profile assertion)?

---

### US-0009 — Advanced forecasting with ML & risk assessment

Status: DONE
Priority: P3

As a long-term planner,
I want ML-enhanced forecasts with seasonal patterns and portfolio risk views,
So that I can make informed multi-year financial decisions.

#### Scope

- In: Seasonal forecasting models, ML-enhanced cashflow projections, portfolio performance forecasts, risk assessment scoring, integration with forecast UI and Grafana Dashboard 5 extensions
- Out: Real-time trading signals, tax optimization

#### Constraints

- Phase 7 per Projectplan; depends on US-0002 baseline forecasting and US-0007 portfolio data
- Models must be explainable enough for user trust in self-hosted context

#### Discovery scope refinements (2026-06-01)

- **Forecast extension (not replacement):** US-0002 rule-based baseline (`DEC-0007`) remains default and always available; ML overlay stored as separate successful `forecast_computations` row tagged `model_kind = ml_enhanced` (or equivalent) linked to same sync cycle
- **Seasonal layer:** Detect and apply monthly/annual seasonality on aggregated net-cashflow series (household or per-account); surface detected periods + strength in API metadata and optional badge on `/forecast` Monthly tab
- **Long-term ML projections:** 6 / 12 / 24 month horizons with **p10–p90 confidence bands** on React long-term chart (vision deferred band now in scope); re-fetch on horizon pill change
- **Baseline vs ML compare:** Toggle or segmented control on `/forecast` **Long-term** tab — **Baseline** (latest `DEC-0007` computation) vs **ML-enhanced** (latest ML computation); dual series + band shading; stat row shows delta at selected horizon end
- **Portfolio performance forecast:** When US-0007 exchange snapshots exist, project portfolio EUR value 3 / 6 / 12 months on **`/wealth` Crypto tab** (or Forecast sub-section) using historical snapshot series — not trade execution
- **Risk assessment score:** 0–100 score for **active plan scenario** (US-0004) from projected month-end balance stress, plan viability signals (extends `plan_viability` concept per R-0022), and optional portfolio volatility proxy when crypto data present; display on Planning **Scenarios** tab and Compare stat cards
- **Grafana Dashboard 5:** Extend `forecast-horizons.json` — ML vs baseline overlay panel, confidence band time series, seasonal flag stat, optional portfolio forecast row when data exists; uid unchanged per DEC-0012
- **Sync pipeline:** ML recompute runs after successful baseline forecast in sync mutex (extends `DEC-0010`); skip ML pass when history below minimum months (configurable) with `ml_skipped` metadata
- **Explainability:** API returns `model_family`, `seasonal_periods`, `backtest_wmape` (or holdout metric), `low_confidence` flags; UI "How this forecast works" collapsible — no black-box-only chart
- **Out of scope unchanged:** real-time trading signals, tax optimization; **added explicit:** external cloud ML APIs (OpenAI/Azure ML/SageMaker), GPU training pipelines, in-app model training UI, replacing US-0003 subscription engine or US-0005 alert rules, new Grafana dashboards beyond Dashboard 5 extensions, changes to six AI chat tools

#### Discovery decomposition evidence (2026-06-01)

- Feature/workflow count: seasonal detection + ML 6–24mo bands + baseline/ML compare UI + portfolio projection + plan risk score + Dashboard 5 panels + sync ML pass (high breadth — **single vertical slice retained**)
- Split axis considered: "ML forecast core" vs "portfolio + risk" — rejected because risk score and portfolio panels depend on same ML/baseline computation IDs and sync ordering; Compare AC requires ML core in same release
- Cross-cutting impact: `backend/src/forecast`, hypertable schema extension, React `/forecast` + `/planning` + `/wealth`, Grafana `forecast-horizons.json`, optional Python/stats sidecar (research), `get_forecast` tool read path (display only — no new tools)
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md#US-0009`)
- Risk surface: sparse history → unstable seasonality; ML vs baseline divergence confuses operators; portfolio forecast without FX completeness (R-0034); scope creep into MLOps or cloud APIs; sync mutex latency growth (DEC-0010 precedent)

#### Discovery open questions (carry to `/research`)

- Self-hosted execution model: embedded Rust stats crate vs Python **StatsForecast** sidecar vs subprocess — latency and Compose footprint?
- Minimum transaction/month history before ML pass runs (90 days? 12 monthly points?) and fallback behavior?
- Hypertable schema: separate `forecast_ml_daily` vs columns on existing tables vs second `computation_id` with `model_kind` discriminator?
- Seasonal detection: MSTL/AutoETS vs rule-based month-of-year factors on mirror aggregates?
- Portfolio forecast input series: `portfolio_snapshots` EUR total vs per-asset — alignment with US-0007 hybrid PnL?
- Risk score formula: deterministic weighted index from plan viability + balance path vs separate ML classifier?
- Grafana: single dashboard JSON version bump vs new row of panels only?
- `get_forecast` AI tool: return both baselines in one payload or `variant` query param?

---

### US-0010 — External Firefly/Postgres & Traefik deployment on omniflow host

Status: DONE
Priority: P0

As an operator with an existing Firefly III and PostgreSQL stack on Debian,
I want Flow Finance AI Compose to attach to those services and publish via Traefik,
So that I do not run duplicate Firefly or Postgres containers and can reach the app at `financegnome.omniflow.cc` behind basic auth.

#### Scope

- In: Refactor `docker-compose.yml` to **external** mode — remove or gate bundled `firefly-iii`; connect `flow-finance-ai` and `grafana` to host `traefik` network; `DATABASE_*` → existing `postgres` service; `FIREFLY_BASE_URL` → existing `firefly` container (`http://firefly:8080`); Traefik labels on Flow Finance AI for `Host(\`financegnome.omniflow.cc\`)`, `websecure`, `tls.certresolver=myresolver`, `middlewares=auth`; update `.env.example` with external-host variables; operator runbook notes referencing `/workdir/firefly/docker-compose.yml` and `/workdir/services/docker-compose.yml` (read-only alignment, no edits required on host stacks unless documented); smoke verification on target host
- Out: Changing Firefly version, migrating Firefly data, modifying Traefik ACME/DNS, replacing host `auth` middleware, OIDC IdP setup (document redirect URI only)

#### Constraints

- **No new** `firefly-iii` or `postgres` services in finance_goblin Compose when external profile is used
- Existing host containers: `firefly` (fireflyiii/core:6.2.12), `postgres` (postgres:latest), `traefik` on Docker network `traefik` (subnet 172.20.0.0/16)
- Reuse global Traefik basic-auth middleware `auth` (`credentials.passwd` on Traefik container) — same pattern as `finance.omniflow.cc`
- Host port **8090** already bound by `firefly_product_manager` — avoid publishing `stats-forecast` on 8090 in external deployments or remap via env
- Flow Finance AI database (`flow_finance_ai`) must exist on shared Postgres (operator creates DB/user if missing)

#### Intake decomposition

- Split decision: **single story** (bounded infra integration)
- Rationale: narrow follow-up to US-0001 Compose; all changes are deploy wiring and verification
- Boundaries: application feature work unchanged; only deployment/integration artifacts

#### Discovery scope refinements (2026-05-31)

- **Compose profile:** add `external` (or equivalent) where `minimal` services exclude `firefly-iii` and join `networks.traefik.external: true`
- **Firefly API URL:** in-container `http://firefly:8080`; public ledger remains `https://finance.omniflow.cc` (unchanged)
- **Postgres:** `DATABASE_HOST=postgres`, credentials from operator `.env` (not committed)
- **Traefik router name:** e.g. `financegnome` — must not collide with `firefly` router on same host
- **Grafana:** optional separate Traefik host or internal-only; default internal on traefik network without public host unless operator requests
- **Env vars operator must supply:** `DATABASE_PASSWORD`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, optional `FIREFLY_APP_KEY` only if local Firefly profile used; `VITE_OIDC_*` / `OIDC_*` if auth enabled for public URL

#### Intake evidence (US-0010)

- `intake_run_id`: `intake-20260531-external-compose`
- `selected_pack`: `small-intake-pack`
- Evidence bundle: `handoffs/intake_evidence/intake-20260531-external-compose.json`

#### Discovery refinements (2026-06-01)

- **Two-file Compose pattern (bundled base + external overlay):**
  - `docker-compose.yml` — **base stack** for greenfield/dev/CI: defines service images, healthchecks, profile membership (`minimal` / `standard` / `full` / `external`), and defaults that reach host Postgres/Firefly via `host.docker.internal` when no overlay is used.
  - `docker-compose.external.yml` — **merge overlay only** for the omniflow operator host: joins external Docker network `traefik`, overrides in-network DNS targets (`postgres`, `firefly`), strips published host ports on `flow-finance-ai`, and adds Traefik router labels. Never defines new `postgres` or `firefly` services.
  - **Canonical omniflow invocation:** `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d` (document `COMPOSE_FILE` / `COMPOSE_PROFILES=external` in operator `.env`).
  - **Alternative considered:** single monolithic compose with env-conditional networks/labels — rejected because overlay keeps dev profiles readable and avoids Traefik label leakage into local minimal runs.
- **No duplicate Firefly on external profile:**
  - Today: `firefly-iii` is on profiles `[minimal, standard, full]` only; `external` profile does **not** include it — partial implementation already satisfies AC-1 when operator uses **`--profile external` alone**.
  - **Risk:** combining profiles (e.g. `--profile minimal --profile external`) would still start bundled `firefly-iii` — execute must prevent or loudly warn.
  - **PO recommendation (execute):** move bundled Firefly to dedicated profile `bundled-firefly`; document greenfield dev as `--profile minimal --profile bundled-firefly`; keep `external` free of any Firefly service definition. **Alternative (doc-only):** runbook forbids profile combination + add compose config test asserting zero `firefly-iii`/`postgres` services under external-only merge.
- **Credentials policy (compose + env):**
  - All secrets via operator `.env` referenced as `${VAR}` or `${VAR:?message}` — **no literal passwords** in committed YAML.
  - Required for external smoke: `DATABASE_PASSWORD`, `FIREFLY_PERSONAL_ACCESS_TOKEN`; `FIREFLY_APP_KEY` / `FIREFLY_DB_*` only when `bundled-firefly` profile used.
  - **Gap in partial impl:** base file still has weak Grafana defaults (`GRAFANA_ADMIN_PASSWORD:-admin`); external overlay should require `${GRAFANA_ADMIN_PASSWORD:?…}` or document operator override. Traefik `auth` middleware credentials remain on host Traefik stack only (out of scope).
  - Hardcoded host in overlay labels (`financegnome.omniflow.cc`) — execute should parameterize `${TRAEFIK_HOST:-financegnome.omniflow.cc}` and `${TRAEFIK_MIDDLEWARE:-auth}` for reuse without editing YAML.
- **Postgres topology (shared container, separate app DB):**
  - Flow Finance AI uses **`flow_finance_ai`** database on existing container **`postgres`** (Docker DNS on network `traefik`); **`DATABASE_HOST=postgres`** in external mode.
  - Firefly ledger DB remains separate (host Firefly stack); connector uses **`FIREFLY_BASE_URL=http://firefly:8080`** (in-container); public UI stays **`https://finance.omniflow.cc`** unchanged.
  - Operator one-time bootstrap (documented, not automated): create DB + user/grants on shared Postgres; verify **TimescaleDB extension** before hypertable migrations (open question — see below).
- **Traefik routing:**
  - Router id **`financegnome`** (must not collide with host `firefly` router); entrypoint `websecure`; cert resolver `myresolver`; middleware **`auth`** (global basic-auth on host Traefik per R-0052).
  - `flow-finance-ai`: public at **`https://financegnome.omniflow.cc`** behind auth.
  - **Grafana:** default **internal-only** on `traefik` network (no public Host rule in overlay); optional second subdomain only if operator explicitly requests (out of MVP unless research recommends otherwise).
- **Port / profile notes:**
  - Host port **8090** bound by `firefly_product_manager` — external/full deployments must use **`STATS_FORECAST_PORT=8091`** (already noted in `.env.example`).
  - `external` profile includes `flow-finance-ai` + `grafana` only (no redis/ollama/stats-forecast unless operator adds other profiles deliberately).

#### Partial implementation review (2026-06-01)

| Area | Status | Execute follow-up |
|------|--------|-------------------|
| `docker-compose.external.yml` overlay + `traefik` external network | present | validate `!reset` ports on target Compose version |
| `flow-finance-ai` on `external` profile; `firefly-iii` excluded from `external` | present | add profile-combination guard or `bundled-firefly` split |
| `DATABASE_HOST` / `FIREFLY_BASE_URL` overrides in overlay | present | align `.env.example` omniflow block with required vars |
| Traefik labels (Host, TLS, `auth` middleware) | present | env-parameterize host/middleware |
| `.env.example` omniflow section + DB bootstrap SQL comment | present | expand operator runbook + OIDC redirect notes for public URL |
| No `postgres` service in finance_goblin compose | satisfied | regression test in CI |
| Operator smoke test on Debian host | **not recorded** | required for AC-6 |

#### Discovery decomposition evidence (2026-06-01)

- Feature/workflow count: 2-file compose pattern + profile isolation + env/Traefik wiring + operator runbook + host smoke test (low–moderate breadth — **single story retained**)
- Cross-cutting impact: `docker-compose.yml`, `docker-compose.external.yml`, `.env.example`, operator docs; no application code changes
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md#US-0010`)
- Risk surface: profile combination starting duplicate Firefly; TimescaleDB extension missing on shared Postgres; hardcoded Traefik host; weak Grafana defaults; OIDC redirect URI mismatch for public SPA URL; compose merge `!reset` portability

#### Discovery open questions (carry to `/research`)

- Is **TimescaleDB** (`CREATE EXTENSION timescaledb`) available on shared host `postgres`? If not, operator procedure vs migration failure mode?
- **`bundled-firefly` profile split** vs doc-only guard — impact on US-0001 minimal command docs and CI compose tests?
- Should **`TRAEFIK_HOST`** / **`TRAEFIK_MIDDLEWARE`** be required env vars in external overlay?
- Grafana on omniflow: remain internal-only or expose via separate Traefik host (e.g. `grafana-financegnome.omniflow.cc`) with same `auth` middleware?
- OIDC: required env updates (`VITE_OIDC_REDIRECT_URI`, `OIDC_ISSUER_URL` audience) when SPA served at `https://financegnome.omniflow.cc` — document-only or validation script?
- Smoke test checklist: exact commands from traefik network namespace for DB/Firefly/PAT/health/TLS/auth (record template for AC-6)?
- Compose CI test: `docker compose … --profile external config` asserts services list excludes `firefly-iii` and `postgres`?

#### Research resolution (2026-06-01) — [R-0053](docs/engineering/research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci)

| Question | Resolution |
|----------|------------|
| TimescaleDB on shared `postgres` | Not assumed on `postgres:latest`; operator preflight `CREATE EXTENSION timescaledb` on `flow_finance_ai` after server-level TimescaleDB install + restart; migration 001 fail-fast if missing (R-0004) |
| `bundled-firefly` vs doc-only | **Prefer profile split** — move `firefly-iii` to `bundled-firefly`; greenfield `--profile minimal --profile bundled-firefly`; doc warning + CI service-list guard required either way |
| `TRAEFIK_HOST` / `TRAEFIK_MIDDLEWARE` | **Defaults in overlay** (`financegnome.omniflow.cc`, `auth`); optional operator override via `.env` — not required vars |
| Grafana on omniflow | **Internal-only default**; add `grafana.ports: !reset []` in external overlay; optional `${GRAFANA_TRAEFIK_HOST}` for second public host behind `auth` |
| OIDC for public SPA URL | **Document-only** IdP registration (`https://financegnome.omniflow.cc/callback`); runtime origin fallback in `oidc.ts` when build vars unset; optional preflight script |
| Smoke test (AC-6) | Template in R-0053 §6 — TimescaleDB check, Firefly DNS, PAT, in-network `/health`, Traefik TLS + 401 without auth |
| Compose CI guard | `docker compose -f … -f … --profile external config --services` → exactly `flow-finance-ai`, `grafana`; extend `tests/run-tests.sh` / runbook |

---

### US-0011 — Unified analytics UI in financegnome (Grafana in-app)

Status: DONE
Priority: P1

As a financegnome user,
I want all analytics charts and Grafana dashboards inside one web UI at financegnome.omniflow.cc,
So that I never need a separate Grafana site and all future charts live in the same shell.

#### Scope

- In: **Analytics** section in React shell (sidebar/nav); embed or same-origin proxy for all provisioned Grafana dashboards (Platform Health, Cashflow, Subscriptions, Budgets, Portfolio, Forecast horizons per DEC-0012 / US-0002–US-0009); keep existing ECharts product pages (Forecast, Wealth, Planning, etc.); replace Wealth “open Grafana in new tab” with in-app routes; optional backend reverse-proxy route for Grafana with OIDC/Traefik-aware auth; **future-chart contract** — new product charts default to in-app (React + API); document when Grafana embed is allowed for ops-only panels
- Out: Removing Grafana container entirely in this story; rewriting all Grafana SQL panels to ECharts (follow-up migration); changing dashboard SQL definitions; Firefly/Compose/Traefik work (US-0010)

#### Constraints

- Must work on omniflow external deploy (US-0010 / DEC-0056): Grafana often internal-only on `traefik` network — embed/proxy must not require separate `GRAFANA_TRAEFIK_HOST` for default UX
- OIDC session and Traefik `auth` middleware must not break iframe or proxied views (CSP, `X-Frame-Options`, anonymous Grafana access policy)
- Do not duplicate chart logic unnecessarily — ECharts pages remain source for interactive product flows; Grafana embed is for SQL operational dashboards until migrated

#### Intake decomposition

- Split decision: **single story** (one unified shell deliverable)
- Rationale: navigation + embed/proxy + auth is one vertical slice; full SQL→React port is a separate future effort
- Boundaries: in-app shell and routes for **all** current dashboards + extensibility guideline; not full Grafana deprecation

#### Approach options (intake recommendation)

| Option | Summary | Trade-off |
|--------|---------|-----------|
| **A — Grafana embed (recommended MVP)** | iframe or Grafana `d-solo` / kiosk routes under `/analytics/*` via backend proxy | Fastest; keeps DEC-0012 dashboards; auth/CSP work required |
| **B — Full React port** | Reimplement each dashboard panel via API + ECharts | Single stack; large effort; out of scope for US-0011 |
| **C — Hybrid** | Embed legacy dashboards; new features React-only | Matches “future charts in financegnome” — **recommended long-term** with A for existing |

#### Discovery scope refinements (2026-06-02)

- Map each `grafana/provisioning/dashboards/**/*.json` to a financegnome route (uid/slug table in architecture)
- Proxy path e.g. `/api/v1/analytics/grafana/*` or static embed base URL from env **`VITE_GRAFANA_EMBED_BASE`** (same-origin preferred)
- Mobile/narrow layout: full-width iframe or responsive panel chrome matching shadcn shell
- Deprecate **`VITE_GRAFANA_URL`** external-tab pattern on WealthPage

#### Discovery refinements (2026-06-02) — route map + partial impl

**Partial implementation review (repo-only; no host `.env`):**

| Area | Status | Notes |
|------|--------|-------|
| Grafana dashboards | **Done** (US-0001–US-0009) | Six JSON files provisioned; stable uids per DEC-0012 |
| External deploy | **Done** (US-0010 / DEC-0056) | `flow-finance-ai` + `grafana` on `traefik`; Grafana host port `!reset`; public route gated by empty `GRAFANA_TRAEFIK_HOST` |
| React `/analytics/*` routes | **Missing** | `App.tsx` has no analytics routes |
| Sidebar **Analytics** section | **Missing** | `AppLayout.tsx` flat nav only |
| Backend Grafana proxy | **Missing** | `build_router` serves API + static SPA only |
| Wealth Grafana link | **External tab** | `WealthPage.tsx` → `VITE_GRAFANA_URL` + `target=_blank` `/d/portfolio` |

**Dashboard → financegnome route map (canonical for architecture):**

| Provisioned JSON | Grafana folder | uid | Title | Route slug | React path | Proxied Grafana embed (kiosk) |
|------------------|----------------|-----|-------|------------|------------|--------------------------------|
| `platform-health.json` | Platform | `platform-health` | Platform Health | `platform-health` | `/analytics/platform-health` | `/d/platform-health/platform-health?kiosk=tv` |
| `analytics/cashflow.json` | Analytics | `cashflow` | Cashflow | `cashflow` | `/analytics/cashflow` | `/d/cashflow/cashflow?kiosk=tv` |
| `analytics/subscriptions.json` | Analytics | `subscriptions` | Subscriptions | `subscriptions` | `/analytics/subscriptions` | `/d/subscriptions/subscriptions?kiosk=tv` |
| `analytics/budgets.json` | Analytics | `budgets` | Budgets | `budgets` | `/analytics/budgets` | `/d/budgets/budgets?kiosk=tv` |
| `analytics/portfolio.json` | Analytics | `portfolio` | Portfolio | `portfolio` | `/analytics/portfolio` | `/d/portfolio/portfolio?kiosk=tv` |
| `analytics/forecast-horizons.json` | Analytics | `forecast-horizons` | Forecast Horizons | `forecast-horizons` | `/analytics/forecast-horizons` | `/d/forecast-horizons/forecast-horizons?kiosk=tv` |

- **Nav IA:** sidebar group **Analytics** with six links (slug labels match Title); optional **Platform** subgroup or single “Platform Health” entry above Analytics dashboards.
- **Proxy contract (discovery recommendation, [R-0054](docs/engineering/research.md#r-0054--unified-financegnome-analytics-shell-grafana-embed-vs-react-port)):** backend reverse-proxy `http://grafana:3000` at same-origin prefix **`/analytics/grafana/`** (preferred over public `GRAFANA_TRAEFIK_HOST`); iframe `src` = ``${VITE_GRAFANA_EMBED_BASE:-/analytics/grafana}/d/{uid}/{slug}?kiosk=tv``; deprecate **`VITE_GRAFANA_URL`**.
- **In-app cross-links (execute):** Wealth portfolio card → `/analytics/portfolio` (not new tab); optional secondary links from Forecast → `forecast-horizons`, Planning → `budgets`, Subscriptions → `subscriptions` — product pages stay ECharts-primary per constraints.
- **ECharts pages unchanged:** `/forecast`, `/wealth`, `/planning`, `/subscriptions`, `/alerts` remain authoritative interactive flows; Grafana routes are SQL operational views.

**Discovery decomposition evidence:**

- Feature/workflow count: nav section + 6 iframe pages + proxy + Wealth link migration + future-chart doc (moderate — **single story retained**)
- Cross-cutting impact: `frontend` routes/layout, `backend` proxy middleware, `.env.example`, operator user guide
- Acceptance breadth: 7 criteria unchanged (`docs/product/acceptance.md#US-0011`)
- Risk surface: Traefik `auth` + OIDC + Grafana framing/auth; proxy WebSocket path; duplicate metrics UX vs ECharts

**Open questions (carry to research/architecture):**

- Grafana **anonymous vs auth-proxy** when financegnome OIDC is enabled — can iframe rely on Traefik/session boundary only? → **Resolved in [R-0056 §1](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik): anonymous Viewer MVP**
- **CSP / X-Frame-Options** — strip or rewrite on proxied responses; required `frame-src` for SPA → **[R-0056 §2](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik)**
- **`GF_SERVER_ROOT_URL` / subpath** — root proxy vs `/analytics/grafana` subpath serve (Grafana 11) → **[R-0056 §3](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik): prefix strip, no `serve_from_sub_path`**
- **WebSocket** through reverse proxy for live panel refresh → **[R-0056 §4](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik)**
- **Theme/kiosk** — match shell chrome; hide Grafana top nav via `kiosk=tv` sufficient? → execute smoke; cosmetic only
- **Traefik basic auth + iframe** → **[R-0056 §5](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik): same-origin reuses edge auth**

#### Intake evidence (US-0011)

- `intake_run_id`: `intake-20260602-unified-charts-ui`
- `selected_pack`: `small-intake-pack`
- Evidence bundle: `handoffs/intake_evidence/intake-20260602-unified-charts-ui.json`
- Research: **[R-0054](docs/engineering/research.md#r-0054--unified-financegnome-analytics-shell-grafana-embed-vs-react-port)**, **[R-0056](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik)**

---

### US-0012 — Auto-provision application database on first start

Status: DONE
Priority: P1

As an operator deploying Flow Finance AI on external PostgreSQL,
I want the application database created automatically on first startup when it does not exist,
So that I do not run manual `CREATE DATABASE` SQL before `docker compose up`.

#### Scope

- In: Idempotent bootstrap before SQLx migrations — detect missing `DATABASE_NAME` (default `flow_finance_ai`) on configured host; connect via optional admin/maintenance URL (e.g. database `postgres`) or elevated `DATABASE_USER` when it has `CREATEDB`; `CREATE DATABASE` when absent; grant usage to app role; attempt `CREATE EXTENSION IF NOT EXISTS timescaledb` on new DB when extension is available on server; clear structured logs and fail-closed errors when bootstrap lacks privilege; update `.env.example` and runbook (US-0010 omniflow) to make bootstrap env vars explicit
- Out: Installing TimescaleDB OS packages or editing `postgresql.conf` on host; auto-creating the PostgreSQL server/container; auto-creating unrelated Firefly databases; bundled/embedded Postgres service in Compose

#### Constraints

- Must remain compatible with **US-0010** external profile (shared `postgres` on `traefik` network)
- Must not drop or recreate an existing database (detect-only create)
- Wrong `DATABASE_PASSWORD` still fails after DB exists — bootstrap does not fix credential typos
- Shared-host deployments may require separate bootstrap credentials (`DATABASE_BOOTSTRAP_URL` or equivalent) — document security guidance (admin URL not committed)

#### Intake decomposition

- Split decision: **single story** (startup bootstrap vertical slice)
- Rationale: narrow operator UX improvement; touches db layer + docs + tests only
- Boundaries: database (+ extension attempt) auto-provision; user/role creation optional follow-up unless discovery expands

#### Approach options (intake recommendation)

| Option | Summary | Trade-off |
|--------|---------|-----------|
| **A — In-app bootstrap (recommended)** | Rust pre-migration hook: maintenance connection → create DB → extension → migrate | One container; needs admin URL env when app user lacks `CREATEDB` |
| **B — Compose one-shot init** | `db-bootstrap` service runs SQL then exits | Clear separation; extra service in compose |
| **C — Script only** | `scripts/bootstrap-db.sh` | Not automatic on first start — rejected for this story |

#### Discovery scope refinements (2026-06-02)

- Define env contract: `DATABASE_BOOTSTRAP_URL` (optional) vs reuse `DATABASE_*` when role has `CREATEDB`
- PostgreSQL version note: use `SELECT 1 FROM pg_database WHERE datname = $1` for idempotency (portable)
- Log lines: `database_bootstrap_created`, `database_bootstrap_skipped_exists`, `database_bootstrap_failed_privilege`
- Relate to migration 001 `CREATE EXTENSION timescaledb` — avoid duplicate failure modes

#### Discovery refinements (2026-06-03)

- **Startup ordering:** `ensure_database` → `connect_with_retry` → `run_migrations` in `backend/src/lib.rs` (new hook in `backend/src/db/`)
- **Env contract:** runtime `DATABASE_*` unchanged; optional **`DATABASE_BOOTSTRAP_URL`** (maintenance DB `postgres`, env-only); when unset derive maintenance URL from `DATABASE_*` with db name `postgres`
- **Idempotency:** `SELECT 1 FROM pg_database WHERE datname = $1`; create only when absent; **never** drop/recreate; grants/owner to `DATABASE_USER` when bootstrap creds ≠ app creds
- **TimescaleDB:** bootstrap runs `CREATE EXTENSION IF NOT EXISTS timescaledb` on app DB before migrations (new + existing DB missing extension); host package install remains operator + runbook US-0010 §1 (R-0053)
- **Migration 001:** keep `CREATE EXTENSION IF NOT EXISTS timescaledb` — duplicate safe; bootstrap fails first with actionable code when extension unavailable
- **Log reason codes (`bootstrap_reason`):** `database_bootstrap_started`, `database_bootstrap_created`, `database_bootstrap_skipped_exists`, `database_bootstrap_grants_applied`, `database_bootstrap_extension_ok`, `database_bootstrap_failed_privilege`, `database_bootstrap_failed_timescaledb`, `database_bootstrap_failed_connect` — no secrets in logs
- **US-0010 runbook delta:** replace manual `CREATE DATABASE flow_finance_ai` with bootstrap env docs; omniflow shared `postgres` expects `DATABASE_BOOTSTRAP_URL` when app role lacks `CREATEDB`
- **Partial impl:** no bootstrap in repo today; `.env.example` still documents manual SQL only

#### Discovery decomposition evidence (2026-06-03)

- Feature/workflow count: bootstrap hook + config + docs + test (low–moderate — **single story retained**)
- Cross-cutting impact: `backend/src/db`, `config`, `.env.example`, runbook; no Compose service added
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md#US-0012`)
- Risk surface: bootstrap URL logging; shared-host grants; TimescaleDB missing after DB create; wrong password still fails post-bootstrap

#### Discovery open questions (carry to `/research` / `/architecture`)

- `CREATE DATABASE … OWNER` vs superuser create + `GRANT ALL` on shared homelab `postgres`?
- Extension step privilege: app role vs bootstrap admin when `CREATE EXTENSION` restricted?
- CI fixture: vanilla Postgres for create path vs TimescaleDB image for extension path?
- `DATABASE_NAME` identifier validation for dynamic SQL safety?
- Maintenance connect retry budget vs app `startup_retry_*` sharing?

#### Intake evidence (US-0012)

- `intake_run_id`: `intake-20260602-auto-provision-db`
- `selected_pack`: `small-intake-pack`
- Evidence bundle: `handoffs/intake_evidence/intake-20260602-auto-provision-db.json`

---

### US-0013 — Production ML forecast & wealth analytics hardening

Status: DONE
Priority: P0
**sprint_id:** S0014
**released:** 2026-06-08 (`0.14.0-us0013`, S0014)

As an operator on the omniflow production profile,
I want ML-enhanced forecasting and wealth analytics to run reliably end-to-end,
So that Forecast and Wealth pages deliver the full US-0009 capability—not baseline-only fallback after BUG-0010 baseline fixes.

#### Scope

- In: `stats-forecast` sidecar on US-0010 **external** profile (`docker-compose.external.yml` merge — today `profiles: [full]` only); `FORECAST_ML_ENABLED=true` + `[forecast_ml] enabled` on omniflow; sync `forecast_ml` phase gates (DEC-0052); `model_kind=ml_enhanced` overlay persistence (DEC-0050); React Compare + wealth ML overlay; Grafana `$forecast_variant=ml_enhanced` panels with data; operator runbook (health, min history, degraded-mode); CI sidecar fixture
- Out: New model research beyond US-0009 (R-0043 ladder); raw transaction ML training; monthly bucket attribution (**BUG-0012** / **US-0015**); Grafana empty-state-only work (**BUG-0009** DONE — banner remains when ML off)

#### Constraints

- **BUG-0010** AA/AB/AC **DONE** Q0013 — baseline numbers prerequisite satisfied; US-0013 closes AC3 ML production path deferred from bug
- DEC-0049 default-off preserved until operator explicitly enables — no silent ML on minimal/external without config
- DEC-0052: ML failure must not fail sync; skip metadata per DEC-0066 when disabled
- DEC-0007 baseline authoritative when ML unavailable — no misleading balances
- `STATS_FORECAST_PORT=8091` on omniflow (host 8090 clash per US-0010 intake)
- No host secrets read during intake/execute

#### Intake decomposition (2026-06-08 re-intake)

- **Evaluator:** high breadth — compose + backend sync + API + React + Grafana + runbook + CI (5+ workflow steps, 4+ component surfaces)
- **Split decision:** **single epic retained**; defer vertical slices to **`/sprint-plan`** (SPRINT_AUTO_SPLIT=1, SPRINT_MAX_TASKS=12)
- **Recommended slices (sprint-plan, not new US IDs):**
  - **US-0013-S1** — External compose + ML config enablement
  - **US-0013-S2** — Sync ML pipeline + API persistence
  - **US-0013-S3** — UI + Grafana ML parity
  - **US-0013-S4** — Runbook + CI sidecar fixture
- **Alternatives rejected:** split into US-0017..0020 now (breaks epic continuity); undifferentiated mega-sprint (exceeds task budget)
- **Boundaries:** BUG-0010 closed baseline defects; US-0013 closes ML production parity with US-0009 on omniflow external profile

#### Intake evidence

- `intake_run_id`: `intake-20260608-us0013`
- `orchestrator_run_id`: `auto-20260608-us0013-001`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: (none)
- `assumptions_confirmed`: (none)
- `prior_intake_ref`: `intake-20260605-forecast-wealth-ml` (BUG-0010 deferral origin)
- Evidence bundle: `handoffs/intake_evidence/intake-20260608-us0013.json`
- Research: [R-0071](docs/engineering/research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile)

#### Discovery notes (2026-06-08)

- **Root cause confirmed:** feature-complete US-0009 ML stack + **missing external sidecar wiring** — not projection/UI defect (BUG-0010 baseline **DONE**)
- **Compose audit:** `stats-forecast` `profiles: [full]` only; `docker-compose.external.yml` has no sidecar service; `flow-finance-ai` on external uses **traefik network only** — sidecar must attach same network for `http://stats-forecast:8090`
- **Config audit:** `[forecast_ml] enabled=false` default (DEC-0049); `FORECAST_ML_ENABLED` / `STATS_FORECAST_URL` env merge in `backend/src/config/mod.rs` — **ready**; `.env.example` documents `STATS_FORECAST_PORT=8091` only (gap: ML enable vars)
- **Backend audit:** sync `forecast_ml` phase + `record_skip_on_baseline` + sidecar `health_ok()` gate — **implemented** (`backend/src/sync/mod.rs`, `backend/src/forecast_ml/`)
- **Frontend audit:** ForecastPage Compare + `sidecar_disabled` copy; WealthPage `portfolio-forecast` horizons + FX warning — **implemented**; needs data post-enablement
- **Grafana audit:** forecast-horizons ML panels query `ml_enhanced` — **implemented**; 0 computations on omniflow today
- **Runbook gap:** no omniflow external-profile ML enablement section (compose union, env, health probe, min history)
- **CI partial:** `forecast_ml_integration` + wiremock; AC row wants external-profile compose or CI fixture extension
- **Slice boundaries unchanged:** US-0013-S1 compose/config → S2 sync/API → S3 UI/Grafana verify → S4 runbook/CI
- **Acceptance:** 10 rows unchanged (9 open + BUG-0010 prerequisite checked)
- **Research resolved (2026-06-08):** overlay profile-merge, traefik-only network, runtime health SLO, min-history gate unchanged, dual CI guard — see [R-0071](docs/engineering/research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile); **DEC-0076** formalized

#### Sprint plan (2026-06-08 — `sprint-plan-20260608-s0014-us0013`)

**Standard sprint:** **S0014** — 11 tasks; no split (11 < `SPRINT_MAX_TASKS` 12).  
**sprint_id:** S0014  
**orchestrator_run_id:** `auto-20260608-us0013-001`

| Order | Task | Slice | Acceptance hook |
|-------|------|-------|-----------------|
| 1 | **T-0144** — External overlay stats-forecast sidecar | S1 | AC-1 |
| 2 | **T-0145** — flow-finance-ai ML env passthrough | S1 | AC-1 |
| 3 | **T-0146** — .env.example omniflow ML block | S1 | AC-1 |
| 4 | **T-0147** — Compose config CI guard update | S1 | AC-1 |
| 5 | **T-0148** — Sync ML phase + health gate verify | S2 | AC-2, AC-3 |
| 6 | **T-0149** — ml_enhanced API persistence verify | S2 | AC-3, AC-4 |
| 7 | **T-0150** — ForecastPage Compare + degraded UX verify | S3 | AC-5 |
| 8 | **T-0151** — WealthPage portfolio-forecast verify | S3 | AC-6 |
| 9 | **T-0152** — Grafana forecast-horizons ML panels verify | S3 | AC-7 |
| 10 | **T-0153** — Runbook Omniflow ML enablement | S4 | AC-8 |
| 11 | **T-0154** — CI dual guard — wiremock + compose assert | S4 | AC-9 |

**Sequencing:** S1 → S2 → S3; S4 after S1 (runbook/CI may parallel S3).  
**Operator gates:** deploy S1 overlay + env → **BACKEND_COMPOSE_DEPLOY** → Full Firefly sync → UAT omniflow smoke.

**Artifacts:** `sprints/S0014/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260608-s0014-us0013`)

**Recommended next phase:** `/plan-verify` on **S0014** → `/execute`

---

### US-0015 — AI-assisted forecast category bucket mapping

Status: DONE
Priority: P2
**sprint_id:** S0016

As an operator reviewing forecast monthly breakdown,
I want AI to help classify transactions and merchants into income/fixed/variable buckets when Firefly categories are missing or ambiguous,
So that forecast decomposition reflects real spending patterns—not hardcoded map gaps or silent zeros.

#### Scope

- In: AI-assisted inference layer for forecast bucket assignment (merchant/description/category fusion); projection fallback chain after DEC-0007 config map miss; operator-visible confidence or **AI-mapped** badge on monthly buckets; audit trail under existing AI privacy constraints (US-0006 / DEC-0032)
- Out: Firefly write-back or in-app category editing; duplicate BUG-0007 chat enumeration scope; new ML forecast models (US-0013); RAG/vector index (deferred per R-0074)

#### Constraints

- **BUG-0012 DONE** (Q0014) — DEC-0007 config-driven category→bucket projection (AG/AH) is prerequisite baseline; AI layer **extends**, does not replace, config map
- Reuse US-0006 tool/privacy patterns; `allow_raw_transactions=false` default preserved unless operator opts in
- Coordinate with **BUG-0007 DONE** (DEC-0069) — shared category intelligence patterns; forecast projection surface only (no chat tool registry changes)
- No host `.env` / operator secrets read

#### Intake decomposition (2026-06-06 re-intake)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 4 surfaces (AI inference, projection integration, monthly API/UI badge, audit trail) |
| Cross-cutting | `forecast/project.rs`, new AI bucket module, monthly API, React ForecastPage Monthly tab |
| Acceptance breadth | 8 rows (1 prerequisite checked + AC-1–AC-7) |
| Risk | Medium-high — privacy defaults, projection correctness, operator trust in AI-mapped buckets |

**Split decision:** **single epic** — deferred to sprint-plan vertical slices **US-0015-S1..S3**

**Rationale:** BUG-0012 released config baseline; four surfaces share projection engine but are independently testable; sprint-plan owns slice IDs (mirrors US-0014 pattern).

**Recommended slices:**

| Slice | Title | Boundary |
|-------|-------|----------|
| **US-0015-S1** | AI bucket inference service | Merchant/description/category fusion; confidence scoring; privacy-safe feature extraction under DEC-0032 defaults |
| **US-0015-S2** | Projection integration | Fallback chain: DEC-0007 config map → AI proposal → Variable default; recurring pattern labels; no silent Variable-only absorption |
| **US-0015-S3** | Operator visibility + audit | Monthly API `bucket_source` fields; UI AI-mapped badge/tooltip; audit log; user guide; OIDC smoke |

**Alternatives considered:**

- Split into US-0017..0019 backlog stories now — **rejected** (operator epic continuity from BUG-0012 deferral)
- Merge into BUG-0007 — **rejected** (chat discovery vs forecast projection)
- RAG embedding index — **rejected for MVP** (R-0074; rule+LLM cascade sufficient)

#### Intake evidence

- `intake_run_id`: `intake-20260606-us0015`
- `selected_pack`: `small-intake-pack`
- `orchestrator_run_id`: `auto-20260606-us0015-001`
- `writer_id`: `po`
- `parent_bug`: BUG-0012 (DONE Q0014)
- `prior_intake_ref`: `handoffs/intake_evidence/intake-20260605-forecast-monthly-buckets.json`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260606-us0015.json`

#### Discovery findings (2026-06-06 — `discovery-20260606-us0015`, orchestrator `auto-20260606-us0015-001`)

**Probe environment:** Code audit of `backend/src/forecast/{categories,project,service}.rs`, `backend/src/api/forecast.rs`, `frontend/src/pages/ForecastPage.tsx`, `backend/config/default.toml` (no `.env` / host secrets; no live omniflow probe this phase).

| AC | Verdict | Execute note |
|----|---------|--------------|
| Prerequisite BUG-0012 | **Shipped** | DEC-0007 config map + category_id resolution in projection — verify only |
| AC-1 Baseline precedence | **Partial** | Config path wired; AI override guard + tests missing (S2) |
| AC-2 AI inference | **Gap** | No inference module; uncategorized → Variable silent (S1) |
| AC-3 Privacy defaults | **Gap** | `PrivacyLayer` chat-only; forecast feature allowlist unwired (S1) |
| AC-4 API visibility | **Gap** | `MonthlyPointResponse` lacks `bucket_source` (S3) |
| AC-5 UI badge | **Gap** | Monthly cards present; no AI-mapped badge/tooltip (S3) |
| AC-6 Audit trail | **Gap** | No forecast-bucket audit rows (S3) |
| AC-7 Regression | **Verify** | OIDC smoke post-deploy; BUG-0007 / US-0013 surfaces unchanged |

**Critical discovery finding:** BUG-0012 shipped config-driven bucketing for **recurring** flows; **rolling residual** (`variable_residual` daily rate) is **hardcoded Variable** with no AI hook — primary enrichment surface alongside uncategorized recurring patterns.

**Partial implementation matrix:**

| Surface | Status |
|---------|--------|
| `categories.rs` `resolve_bucket` | **Done** |
| `project.rs` recurring bucket assignment | **Done** |
| `project.rs` rolling → Variable only | **Gap** (AI target) |
| AI inference service | **Missing** (S1) |
| Monthly API `bucket_source` | **Missing** (S3) |
| ForecastPage AI-mapped badge | **Missing** (S3) |
| AI audit for bucket assignments | **Missing** (S3) |

**Slice boundaries (confirmed):**

| Slice | Primary AC | Discovery note |
|-------|------------|----------------|
| US-0015-S1 | AC-2, AC-3 | New inference module; privacy-safe features; US-0008 provider reuse |
| US-0015-S2 | AC-1 | Fallback chain in `project_account`; guard config precedence; rolling + recurring paths |
| US-0015-S3 | AC-4, AC-5, AC-6, AC-7 | API provenance, badge (seasonal callout pattern), audit, user guide, OIDC smoke |

- **Acceptance:** 8 rows unchanged (1 prerequisite checked + AC-1–AC-7 open until execute)
- **Research:** [R-0074](docs/engineering/research.md#r-0074--us-0015-ai-forecast-bucket-mapping-rulellm-cascade-privacy) fulfilled + [R-0075](docs/engineering/research.md#r-0075--us-0015-forecast-bucket-privacy-feature-allowlist) — 6/6 questions resolved; **DEC-0078** accepted at architecture
- **UX references:** Finanzguru four-card monthly layout (retain); seasonal badge pattern in `ForecastPage.tsx` as AI-mapped chrome template; R-0074 rule+LLM cascade
- **Decision gates:** none blocking — rolling-residual aggregate AI split deferred stage-2; merchant aliases TOML post-MVP

#### Architecture notes (2026-06-06 — `architecture-20260606-us0015`, orchestrator `auto-20260606-us0015-001`)

**Formalized:** **DEC-0078** — config→rule→LLM→Variable cascade; `ai_bucket_min_confidence=0.75`; `PrivacyLayer::prepare_bucket_features` (R-0075); `bucket_sources` + `ai_mapped` on `MonthlyPointResponse`; US-0008 `build_provider()` reuse; `ai_tool_audit` forecast rows.

| Slice | Architecture contract | Primary files |
|-------|----------------------|---------------|
| **US-0015-S1** | `BucketInferenceService`; privacy allowlist; batch cap 100 | `forecast/bucket_inference.rs`, `ai/privacy.rs` |
| **US-0015-S2** | `resolve_bucket_with_ai`; AC-1 config guard; provenance aggregation | `forecast/project.rs`, `forecast/categories.rs` |
| **US-0015-S3** | API provenance; AI-mapped badge; audit; user guide; OIDC smoke | `api/forecast.rs`, `ForecastPage.tsx` |

**MVP limitation (documented):** `variable_residual` daily rate stays Variable — AI enriches recurring dues only.

**Recommended sprint:** S0016 — S1+S2 before S3.

#### Sprint plan (2026-06-06 — `sprint-plan-20260606-s0016-us0015`)

**Standard sprint:** **S0016** — 12 tasks (~32h); no split (12 = `SPRINT_MAX_TASKS` 12).  
**sprint_id:** S0016  
**orchestrator_run_id:** `auto-20260606-us0015-001`

| Order | Task | Slice | Est. | Acceptance hook |
|-------|------|-------|------|-----------------|
| 1 | **T-0163** — BucketInferenceService module (rule→LLM cascade) | S1 | 4h | AC-2 |
| 2 | **T-0164** — PrivacyLayer::prepare_bucket_features + BucketFeatureRow | S1 | 3h | AC-3 |
| 3 | **T-0165** — Structured LLM I/O + ai_bucket_min_confidence TOML | S1 | 3h | AC-2 |
| 4 | **T-0166** — S1 unit tests: privacy, threshold, provider_unavailable | S1 | 2h | AC-2, AC-3 |
| 5 | **T-0167** — resolve_bucket_with_ai + config precedence guard | S2 | 3h | AC-1 |
| 6 | **T-0168** — Recurring dues AI on config-map miss | S2 | 4h | AC-1, AC-2 |
| 7 | **T-0169** — Provenance tracking per monthly accumulation | S2 | 3h | AC-4 |
| 8 | **T-0170** — S2 integration tests: config never overridden | S2 | 3h | AC-1 |
| 9 | **T-0171** — MonthlyPointResponse bucket_sources + ai_mapped | S3 | 2h | AC-4 |
| 10 | **T-0172** — ForecastPage AI-mapped badge | S3 | 2h | AC-5 |
| 11 | **T-0173** — ai_tool_audit forecast_bucket_assignment persistence | S3 | 2h | AC-6 |
| 12 | **T-0174** — User guide US-0015 + UAT OIDC smoke template | S3 | 2h | AC-7 |

**Sequencing:** S1 (T-0163→T-0166) → S2 (T-0167→T-0170) → S3 (T-0171→T-0174). S1+S2 before S3 API/UI.  
**Operator gates:** deploy S1–S3 backend+frontend → **BACKEND_FRONTEND_DEPLOY** → UAT omniflow `/forecast` Monthly OIDC smoke.

**Artifacts:** `sprints/S0016/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260606-s0016-us0015`)

**Recommended next phase:** `/plan-verify` on **S0016** → `/execute`

---

### US-0014 — Planning mode intuitive UX completion

Status: DONE
Priority: P2
**sprint_id:** S0015
**release_version:** 0.15.0-us0014
**released_at:** 2026-06-08

As a household budgeter using Flow Finance AI,
I want planning mode to be fully functional and intuitive from first visit,
So that I can create scenarios, compare versions, and track plan-vs-actual without broken clicks or confusing numbers.

#### Scope

- In: First-run planning onboarding (template grid + Create empty plan); empty-plan → add-lines flow polish; Compare tab contextual help (overlay-only delta); plan-vs-actual guided UX polish (DEC-0074); template discoverability; Set-active guidance banner; operator-visible error surfaces
- Out: AI-driven plan simulation chat (US-0006); crypto allocation scenarios (US-0007); compare metric formula changes (frozen DEC-0073); PVA API contract changes (frozen DEC-0074); auto-activate first plan

#### Constraints

- **BUG-0011 DONE** (Q0019 released 2026-06-08) — AD/AE/AF functional gates satisfied; prerequisite acceptance row checked
- **DEC-0073** / **DEC-0074** frozen — US-0014 is frontend/guided UX polish only unless discovery finds contract gap
- US-0004 acceptance remains baseline; Firefly data read-only unchanged
- No host `.env` / operator secrets read

#### Intake decomposition (2026-06-08 re-intake)

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | 6 UX surfaces (onboarding, add-lines, Compare copy, PVA guidance, templates, errors) |
| Cross-cutting | React `PlanningPage.tsx` primary; plans API contracts frozen |
| Acceptance breadth | 9 rows (1 prerequisite checked + 8 open AC-1–AC-8) |
| Risk | Medium — functional fixes shipped; epic is guided UX and discoverability |

**Split decision:** **single epic** — deferred to sprint-plan vertical slices **US-0014-S1..S3**

**Rationale:** BUG-0011 released functional gates; six UX surfaces share PlanningPage but are independently testable slices; sprint-plan owns slice IDs (mirrors US-0013 pattern).

**Recommended slices:**

| Slice | Title | Boundary |
|-------|-------|----------|
| US-0014-S1 | First-run onboarding + template discoverability | Empty-state template grid, Create empty plan CTA, Set-active banner |
| US-0014-S2 | Add-lines polish + error surfaces | Inline add form visibility, mutation toasts, API error surfacing |
| US-0014-S3 | Compare + PVA contextual UX | Compare help copy; PVA guided card polish; user guide |

**Alternatives considered:** split into US-0017..0019 now — rejected (epic continuity from BUG-0011 deferral); merge into BUG-0011 — rejected (BUG-0011 DONE).

#### Intake evidence

- `intake_run_id`: `intake-20260608-us0014`
- `orchestrator_run_id`: `auto-20260608-us0014-001`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `story`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260608-us0014.json`
- `prior_intake_ref`: `handoffs/intake_evidence/intake-20260605-planning-mode-broken.json`
- `parent_bug`: BUG-0011 (DONE)
- Research: [R-0070](docs/engineering/research.md#r-0070--bug-0011-planning-mode-compare-delta-empty-state-api-first-run-ux) (fulfilled), [R-0072](docs/engineering/research.md#r-0072--us-0014-planning-ux-epic-gap-beyond-bug-0011)
- `next_phase`: `/research`

#### Discovery notes (2026-06-08 — `discovery-20260608-us0014`, orchestrator `auto-20260608-us0014-001`)

**Probe environment:** Code audit of `frontend/src/pages/PlanningPage.tsx` post-Q0019 (no `.env` / host secrets; no live omniflow probe this phase).

| AC | Verdict | Execute note |
|----|---------|--------------|
| AC-1 Onboarding | **Shipped** | Empty-state template grid + primary **Create empty plan** — verify only |
| AC-2 Add-lines | **Partial** | Form wired; add success/error feedback + PVA/detail invalidation gap |
| AC-3 Compare UX | **Shipped** | Overlay footnote present (L600–603); verify DEC-0073 **0.00** display |
| AC-4 PVA guided | **Shipped** | `no_active_plan` guided card with Scenarios + Set active — verify only |
| AC-5 Templates | **Partial** | Discoverability done; add confirmation toast on empty-state **Create from** / create mutations |
| AC-6 Set-active | **Partial** | Banner after create; extend copy for **Grafana Dashboard 3** active-plan requirement |
| AC-7 Errors | **Gap** | No mutation `onError` surfaces — primary S2 execute work |
| AC-8 OIDC | **Verify** | Omniflow `/planning` three-tab smoke; pass-with-prerequisites **BACKEND_FRONTEND_DEPLOY** |

**Critical discovery finding:** Q0019 (BUG-0011) shipped **5 of 8** epic AC rows in code; intake gap table overstated remaining work. US-0014 is **polish + error surfaces + operator smoke**, not first-run greenfield.

**Slice boundaries (adjusted):**

| Slice | Primary AC | Discovery note |
|-------|------------|----------------|
| US-0014-S1 | AC-1, AC-5, AC-6 | Mostly verify; banner Dashboard 3 text + create confirmation toasts |
| US-0014-S2 | AC-2, AC-7 | **Primary gap** — shared error helper, add-line success feedback, query invalidation |
| US-0014-S3 | AC-3, AC-4, AC-8 | Verify shipped help/guided UX; OIDC smoke; user guide US-0014 |

- **Acceptance:** 9 rows unchanged (prerequisite checked + AC-1–AC-8 open until execute)
- **Research:** [R-0072](docs/engineering/research.md#r-0072--us-0014-planning-ux-epic-gap-beyond-bug-0011) discovery matrix confirmed; close open item #1 (Q0019 overlap)
- **Architecture (2026-06-08):** **DEC-0077** formalized — page-local mutation feedback; spec-pack trio + user guide created; see `docs/engineering/architecture.md` § US-0014

#### Sprint plan (2026-06-08 — `sprint-plan-20260608-s0015-us0014`)

**Standard sprint:** **S0015** — 8 tasks; no split (8 < `SPRINT_MAX_TASKS` 12).  
**sprint_id:** S0015  
**orchestrator_run_id:** `auto-20260608-us0014-001`

| Order | Task | Slice | Acceptance hook |
|-------|------|-------|-----------------|
| 1 | **T-0158** — planningFeedback helper module | S2 | AC-7 |
| 2 | **T-0159** — onError on all 7 planning mutations | S2 | AC-7 |
| 3 | **T-0160** — addAdjustment success + plan-vs-actual invalidation | S2 | AC-2 |
| 4 | **T-0155** — AC-1 empty-state onboarding regression verify | S1 | AC-1 |
| 5 | **T-0156** — Set-active banner Dashboard 3 copy | S1 | AC-6 |
| 6 | **T-0157** — Create/template success confirmations | S1 | AC-5 |
| 7 | **T-0161** — Compare/PVA shipped UX verify | S3 | AC-3, AC-4 |
| 8 | **T-0162** — User guide US-0014 finalize + UAT OIDC template | S3 | AC-8 |

**Sequencing:** S2 foundation (T-0158 → T-0159 → T-0160); S1 after T-0158 for toasts; S3 after S2. S2-weighted (AC-7 primary).  
**Operator gates:** deploy S1–S2 frontend → **BACKEND_FRONTEND_DEPLOY** → UAT omniflow `/planning` OIDC smoke.

**Artifacts:** `sprints/S0015/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260608-s0015-us0014`)

**Recommended next phase:** `/plan-verify` on **S0015** → `/execute`

---

### US-0016 — Root README for operators and contributors (living documentation)

Status: DONE
Priority: P2
**sprint_id:** S0013
**closure_note:** verify-work PASS S0013 + release PASS, 2026-06-08

As a new operator or contributor cloning Flow Finance AI,
I want a root `README.md` that explains what the product is, how to run it, and where deeper docs live,
So that I do not hunt through `docs/` folders or fail `validate_doc_profile` on first clone.

#### Scope

- In: Create and populate root `README.md` per **DEC-0059** split layout (`DOC_AUDIENCE_PROFILE=both`, `DOC_DETAIL_LEVEL=balanced`): user-channel H2 sections (Purpose, Quickstart, Examples, Limitations, Related documentation) with **Flow Finance AI**-specific content; `## Contributing` pointer to `docs/developer/README.md`; cross-links to `docs/user-guides/`, `docs/engineering/runbook.md`, and compose profiles (minimal / bundled-firefly / external omniflow)
- In: Align `docs/developer/README.md` only where gaps remain (workflow pointer to release/refresh-context README maintenance)
- In: **Living-doc maintenance contract** — when a **US** or **BUG** closes in release or refresh-context, update root README **Product status** (or equivalent) subsection with the closed id + one-line outcome; run `python scripts/validate_doc_profile.py --repo .` at release gate
- In: `template/README.md` parity when `template/` tree exists; document maintenance in runbook § documentation profile
- Out: Replacing per-story `docs/user-guides/US-xxxx.md` (US-0032); full its-magic framework manual (see `its_magic/README.md`); auto-generated README from backlog on every commit (manual curated updates at phase boundaries only)

#### Constraints

- **US-0077** / **DEC-0059**: no `DEV_*` H2 titles in root README; developer depth stays in `docs/developer/README.md`
- `USER_GUIDE_MODE=1` and `SPEC_PACK_MODE=1`: root README must mention `docs/user-guides` and engineering/spec paths in Related documentation
- Incremental updates — avoid full README rewrites each sprint; cap root H2 count per profile budget

#### Intake decomposition

- Split decision: **single story** (documentation surface + maintenance process are one deliverable)
- Rationale: Missing root README blocks `validate_doc_profile`; upkeep cadence is part of the same operator/contributor outcome
- Boundaries: Feature how-tos remain in user guides; engineering contracts remain in architecture/decisions

#### Intake evidence

- `intake_run_id`: `intake-20260607-root-readme`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `story`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260607-root-readme.json`
- Research: [R-0066](docs/engineering/research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance)

#### Discovery notes (2026-06-08)

- **Split-layout target (DEC-0059, profile `both`/`balanced`):** root README required user H2s = `Purpose`, `Quickstart`, `Examples`, `Limitations`, `Related documentation`; single `## Contributing` pointer; **no `DEV_*` H2 titles** in root; profile-scoped root H2 budget ≤ **8** (5 user H2s fit) — resolved from `scripts/doc_profile_lib.py`.
- **Developer keys (live in `docs/developer/README.md`, not root):** `DEV_PREREQS`, `DEV_WORKFLOW`, `DEV_QUALITY_GATES`, `DEV_ARCHITECTURE` — validator already enforces shard presence; align only where gaps remain.
- **Optional-mode crosslinks (validator-checked):** `USER_GUIDE_MODE=1` → mention `docs/user-guides`; `SPEC_PACK_MODE=1` → mention engineering/spec paths in Related documentation.
- **Template parity (`template/` absent today):** AC-6 is conditional; discovery defers stub-vs-`--no-template-parity` decision to `/research`/`/architecture`.
- **Living-doc placement:** discovery leans a budget-safe `###` Product status subsection (vs a new `## Product status` H2 that consumes H2 budget); confirm in research.
- Acceptance rows unchanged (6 in `docs/product/acceptance.md` § US-0016); validator PASS is release gate.
- `next_phase`: `/research`

---

### US-0017 — README living-doc expansion and troubleshooting (post-US-0016)

Status: DONE
Priority: P2
**sprint_id:** Q0021
**release_version:** `0.17.0-us0017`
**released:** 2026-06-09

As an operator or contributor using Flow Finance AI,
I want the root README expanded with troubleshooting guidance and a enforced upkeep hook when stories and bugs close,
So that onboarding and production smoke paths stay accurate without hunting runbook fragments.

#### Scope

- In: Expand root `README.md` **Examples** with omniflow smoke paths (sync trigger, recompute, analytics route table, exchange sync sanity)
- In: Expand **Limitations** or add budget-safe **Troubleshooting** subsection (empty Grafana panels, BACKEND_FRONTEND_DEPLOY cadence, ML-unavailable vs data-missing distinction) per `DOC_DETAIL_LEVEL=balanced`
- In: Strengthen **living-doc maintenance contract** — release and refresh-context checklists in `docs/developer/README.md` and runbook § documentation profile require Product status bullet for each closed US/BUG in the release segment
- In: Update **Product status** to include **US-0015** and other post-US-0016 closures when this story ships
- Out: Replacing per-story user guides; auto-generated README on every commit; analytics defect fixes (**BUG-0013**)

#### Constraints

- **US-0016** / **DEC-0059** split layout preserved — no `DEV_*` H2 in root; H2 budget ≤ 8
- `python scripts/validate_doc_profile.py --repo .` must remain PASS
- Incremental edits only — no full README rewrite

#### Intake decomposition

- Split decision: **dual work item** with **BUG-0013** (operator bundled concerns)
- Rationale: Documentation upkeep is independently valuable from analytics regression remediation
- Boundaries: US-0017 = README/runbook/checklist only; BUG-0013 = data pipeline correctness

#### Intake evidence

- `intake_run_id`: `intake-20260606-omniflow-regression-readme`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `story` (paired with **BUG-0013** in same intake run)
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260606-omniflow-regression-readme.json`
- Research: [R-0066](docs/engineering/research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance) (living-doc patterns; no new research required)

#### Discovery notes (2026-06-09)

- **Orchestrator run:** `auto-20260609-us0017-001`
- **Baseline audit:** US-0016 shipped root README (S0013); `validate_doc_profile.py --no-template-parity` **PASS** today; 6 root H2s (5 user + Contributing); H2 budget headroom **2** per DEC-0059.
- **Product status:** refresh-context post-Q0020 already lists **US-0015**, **BUG-0013**, and other post-US-0016 closures — AC row satisfied at execute verification only (no further ids expected unless segment closes more work).
- **Examples gap:** localhost-only `curl` samples (`:8080`); analytics route **table** present but no omniflow base URL or Traefik basic-auth pattern; sync trigger lacks external-profile variant; **forecast recompute** and **exchange sync sanity** (`GET /api/v1/wealth` crypto probe) deferred entirely to runbook §23.
- **Troubleshooting gap:** `## Limitations` has ML-unavailable bullet but lacks: (1) **empty Grafana panels vs ML-unavailable banner** distinction (BUG-0013 **AI** refuted on acct **114** after Full sync; zeros on **116** or stale deploy ≠ ML-off); (2) **`BACKEND_FRONTEND_DEPLOY`** + **`GRAFANA_PROVISIONING_RELOAD`** + **`FULL_FIREFLY_SYNC`** gate sequence from Q0020; (3) sync+recompute prerequisite before attributing non-zero analytics failures to code.
- **Maintenance contract gap:** runbook § README maintenance (US-0016) and `docs/developer/README.md` Quality gates mention Product status updates but do not yet say **each closed US/BUG in the release segment** — US-0017 execute should tighten wording in both surfaces.
- **Recommended doc placement (discovery):** budget-safe **`### Omniflow smoke (external profile)`** under `## Examples`; **`### Troubleshooting`** under `## Limitations` (not a new H2 — preserves DEC-0059 layout; aligns R-0067 Product-status placement precedent).

#### Architecture notes (2026-06-09)

- **Orchestrator run:** `auto-20260609-us0017-001`
- **Decision:** **DEC-0070** extension (no DEC-0081) — H3 layout + per-segment maintenance frozen in `docs/engineering/architecture.md` § US-0017
- **Research:** [R-0078](docs/engineering/research.md#r-0078--us-0017-readme-omniflow-smoke-templates-h3-layout-validate_doc_profile-gates)
- **Execute slices:** E1 README Examples H3; E2 README Troubleshooting H3; E3 Product status verify; E4 `docs/developer/README.md`; E5 runbook § README maintenance; E6 `validate_doc_profile --no-template-parity`; UG1 user guide (`USER_GUIDE_MODE=1`)
- **Sprint:** **Q0021** (`/quick`, 7 tasks, PLANNED 2026-06-09)
- `next_phase`: `/plan-verify`
- **Troubleshooting topic map (from BUG-0013 / Q0020):**

| Symptom | Likely cause | Operator action |
|---------|--------------|-----------------|
| All analytics panels flat **0 €** after deploy | Stale image / gates skipped | **BACKEND_FRONTEND_DEPLOY** → **GRAFANA_PROVISIONING_RELOAD** → **FULL_FIREFLY_SYNC** + recompute |
| Budgets MTD **−€150K** planned, **€0** actual | Pre-AL1 MTD SQL artifact | Deploy **DEC-0079** build + Grafana reload; see runbook §23 Row AL |
| Crypto **€0** in wealth/portfolio | Pre-AN1 pricing gap or exchanges-only sync | Deploy **DEC-0080** build + Full sync + manual exchange sync; `crypto.subtotal_eur` probe |
| Forecast **0 €** on default panels | Wrong `$account_id` or exchanges-only sync | Full sync + recompute; verify acct **114** (not **116**); see BUG-0013 **AI** verdict |
| **ML unavailable** banner on forecast-horizons | ML overlay off (US-0013) | Expected — baseline statistical forecast still applies; not a data-missing defect |
| Grafana **Failed to fetch** (browser) | Embed/WS edge case | curl ds/query **200** per R-0077 — check Traefik session + do not Save dashboard overrides |

- **Out of scope unchanged:** analytics code; MetaMask console noise; new research entry (R-0066/R-0067 sufficient).
- **Decomposition:** single story retained — doc surfaces + maintenance hooks are one deliverable.
- `next_phase`: `/research` (lightweight: omniflow curl template + Troubleshooting H3 contract; no web research required)

**Recommended next phase:** `/research`

---

### US-0018 — Category filters & expense trend analytics

Status: DONE
Priority: P1
**sprint_id:** S0017

As a household budgeter using Firefly categories,
I want to filter by category across product views and see how each category's spending changes month over month,
So that I can spot where I save or overspend and use categories in forecasts and planning what-ifs.

#### Scope

- In: Shared **category filter** contract (API + React) on forecast monthly/long-term views, planning compare context, wealth/firefly breakdown, and Grafana dashboards where category breakdown applies
- In: **Per-category monthly expense series** API (rolling 12–24 months) from mirror `transactions` + `categories`
- In: React **category trend chart** (bar default, line optional) with EUR totals per month; **single-category MVP** (multi-overlay deferred)
- In: Category performance summary (month-over-month delta, best/worst month indicator) for selected period
- In: Grafana **`$category`** variable + category-scoped panel on **`cashflow`** and **`budgets`** dashboards (AC-1 minimum two)
- In: Shared **`CategoryFilter`** React component + `GET /api/v1/categories` catalog + `GET /api/v1/categories/expense-series` monthly series
- Out: Firefly category editing; ML category auto-labeling (US-0015 bucket mapping unchanged); tax reporting; Grafana↔SPA bidirectional filter sync; multi-category chart overlay (stretch)

#### Constraints

- Firefly read-only — categories sourced from mirror sync (post-BUG-0006 `category_id` ingest)
- Reporting currency EUR with native Firefly account currency noted where mixed
- Privacy: aggregate category series only in new public REST endpoints (no raw rows)
- Forecast monthly category filter scopes **display/breakdown** in MVP; full forecast re-projection by category is architecture follow-on (DEC-0007 join)

#### Discovery (2026-06-08)

- **Surface map:** `/forecast` monthly tab, `/planning` compare toolbar, `/wealth` overview subsection, Grafana `cashflow` + `budgets` (see `docs/product/vision.md` US-0018 discovery)
- **Partial impl:** mirror + period `aggregates_by_category` done; monthly per-category series API, REST routes, filter component, trend chart, Grafana `$category` missing
- **Multi-category:** deferred — single select satisfies AC-3; architecture may add ≤3 overlay later
- **Uncategorized:** explicit labeled bucket per AC-5 (reuse AI aggregate labeling pattern)
- **Decomposition:** single story retained — filter contract + API + chart + Grafana wiring are one vertical slice

#### Intake decomposition

- Split decision: **multi-story** (3 stories — see intake evidence)
- Boundaries: US-0018 = category analytics foundation; US-0019 = planning/goals; US-0020 = subscriptions/tags
- Recommended sequence: **US-0018** first (enables category what-ifs in US-0019)

#### Intake evidence

- `intake_run_id`: `intake-20260607-category-planning-subscriptions`
- `selected_pack`: `first-intake-pack`
- `plan_area_id`: `category-analytics`
- Evidence bundle: `handoffs/intake_evidence/intake-20260607-category-planning-subscriptions.json`
- Research: [R-0080](docs/engineering/research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake), [R-0083](docs/engineering/research.md#r-0083--us-0018-category-filters-expense-series-api--trend-analytics)

**Artifacts:** `sprints/S0017/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260608-s0017-us0018`), `handoffs/releases/S0017-release-notes.md`

**Recommended next phase:** _(closed — release PASS S0017 `0.18.0-us0018`)_

---

### US-0019 — Goal-driven planning with per-plan stats & AI savings suggestions

Status: DONE
Priority: P1
**sprint_id:** S0018

As a household planner,
I want goal-based plans (e.g. **€10k balance in 5 months**) with per-plan statistics and AI-assisted savings ideas by category,
So that I see monthly/yearly deltas and projected balance at my target date—not only whole-household aggregates.

#### Scope

- In: **Target balance + target date** plan type (extends US-0004 plan engine)
- In: **Per-plan statistics** view: monthly cash delta vs baseline, yearly rollup, projected balance at target date
- In: **Category-scoped adjustments** in plan builder (e.g. reduce spend in category "crypto") — builds on US-0018 filter contract when available
- In: **AI savings suggestions**: propose reducible expenses/categories; operator selects suggestions to materialize plan adjustment lines (US-0006 tool layer, `allow_raw_transactions=false`)
- In: Per-plan compare/PVA scoped to selected plan version — not mixed with other plans' stats
- Out: Firefly write-back; automatic plan execution; replacing US-0014 template onboarding (extends it)

#### Constraints

- Read-only Firefly; plan mutations in product DB only
- AI suggestions must cite aggregate/category signals per DEC-0032; operator confirms each adopted line
- US-0014 / BUG-0011 functional gates remain baseline

#### Intake decomposition

- `plan_area_id`: `goal-planning`
- Depends on US-0018 for richest category what-if UX; MVP may ship with category picker from mirror catalog

#### Intake evidence

- Same bundle as US-0018 (`intake-20260607-category-planning-subscriptions`)
- Research: [R-0080](docs/engineering/research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake)

#### Discovery (2026-06-09)

- **Surface map:** `/planning` Scenarios (goal template + metadata + category adjustments + AI savings modal), Compare (per-plan goal stats strip + existing version table), PVA unchanged (active plan); see `docs/product/vision.md` US-0019 discovery
- **US-0018 dependency (released):** `CategoryFilter` + `GET /api/v1/categories` + expense-series for actuals preview; **DEC-0089** — compare API household-level; category overlay in plan engine is US-0019 scope
- **Partial impl (pre-S0018):** templates + category enum + subscription savings modal done; goal metadata, per-plan stats, category overlay, AI category suggestions shipped in S0018
- **Goal template:** new **`goal_balance`** card with `target_balance_eur`, `target_date`, optional `account_id` (discovery draft)
- **AI savings:** ranked category candidates with evidence summary; operator checkbox select → adjustment lines (`allow_raw_transactions=false`)
- **Decomposition:** single story retained — goal type + stats + category overlay + AI picker are one planning vertical slice

**Artifacts:** `sprints/S0018/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260609-s0018-us0019`), `handoffs/releases/S0018-release-notes.md`

**Recommended next phase:** _(closed — release PASS S0018 `0.19.0-us0019`)_

---

### US-0020 — Subscription manual discovery, majority category & operator tags

Status: DONE
Priority: P2
**sprint_id:** S0019

As a subscription manager,
I want to search for potential subscriptions myself, assign majority categories from transaction history, and tag subscriptions with my own labels,
So that detection automation plus manual control both work and I can group services (e.g. luxus, important).

#### Scope

- In: **Search/filter UI** for subscription candidates: account, title/payee substring, repeating interval (months), amount band (discovery refines)
- In: Operator **confirm/reject** candidate without relying solely on auto-detection pipeline (US-0003)
- In: On confirm, set subscription **display category** = **majority category** of linked transactions (mode on tie — document in architecture)
- In: Operator-defined **tags** (create/rename/delete) attachable to confirmed subscriptions; filter list by tag
- In: Tags stored in product DB; API for list/filter; optional Grafana subscriptions dashboard tag variable (discovery)
- Out: Firefly tag write-back; changing Firefly category on source transactions

#### Constraints

- Read-only Firefly; tags are Flow Finance AI overlay metadata
- Majority-category rule must handle miscategorized single transactions (operator example: 1 of 12 wrong)
- US-0008 detection/dedup contracts preserved

#### Intake decomposition

- `plan_area_id`: `subscription-ops`
- Independent delivery slice — can parallel US-0018

#### Intake evidence

- Same bundle as US-0018 (`intake-20260607-category-planning-subscriptions`)
- Research: [R-0080](docs/engineering/research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake)

#### Discovery (2026-06-09)

- **Surface map:** `/subscriptions` **Discover** tab (search form + candidate table), confirmed list + detail drawer (majority category badge), tag manager + tag filter chips; optional Grafana `subscriptions` **`$tag`** variable (stretch); see `docs/product/vision.md` US-0020 discovery
- **US-0003 dependency (released):** auto-detection + Pending confirm/reject preserved; manual path additive — does not replace sync-triggered detection
- **DEC-0084**..**DEC-0086** (released): manual confirms must use same payee normalization, payee+interval inheritance, ±3d interval tolerance as auto-detected confirms
- **US-0018 dependency (released):** `GET /api/v1/categories` for display names on majority category — no new category ingest
- **Partial impl:** `/subscriptions` All/Pending/Standing tabs + confirm/reject on pending done; explorer API/UI, `display_category_id`, tag CRUD/assign/filter, manual confirm-from-search **missing**
- **Majority category:** mode of linked tx `category_id` on confirm; tie-break most-recent category documented in UI tooltip
- **Decomposition:** single story retained — discover search + manual confirm + majority category + operator tags are one subscription-ops vertical slice

**Artifacts:** `sprints/S0019/*`, `handoffs/tl_to_dev.md` (`sprint-plan-20260610-s0019-us0020`), `handoffs/releases/S0019-release-notes.md`

**Recommended next phase:** _(closed — release PASS S0019 `0.20.0-us0020`; last story in intake bundle)_

---

## Intake evidence (2026-05-31)

- `intake_run_id`: `intake-20260531-flow-finance`
- `selected_pack`: `first-intake-pack`
- `asked_topics`: users_problem, runtime_target_environment, language_framework_runtime, architecture_preference, ui_design_expectations, security_compliance, non_functional_priorities, scope_timeline
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- `coverage_complete`: `true`
- Evidence bundle: `handoffs/intake_evidence/intake-20260531-flow-finance.json`
- `plan_area_inventory`: platform-foundation, firefly-integration, forecasting, grafana-analytics, subscriptions, planning, alerts-wealth, ai-assistant, crypto-portfolio, local-ai, advanced-forecasting
- `plan_area_coverage`: see evidence bundle JSON
