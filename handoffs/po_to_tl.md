## research-20260602-us0011 — US-0011 unified analytics embed technical research

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-02  
**Story:** US-0011  
**Next phase:** `/architecture`

### Summary

Web research completed for US-0011 Grafana in-app embed. Added **[R-0056](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik)** resolving all discovery open questions: anonymous Viewer auth behind internal Grafana + Traefik boundary, CSP/X-Frame-Options handling on proxied responses, prefix-strip proxy (no `GF_SERVER_SERVE_FROM_SUB_PATH`), WebSocket upgrade for Grafana Live through Axum proxy, and same-origin Traefik basic-auth behavior for iframes. Extends R-0054; aligns with DEC-0057 contract elements. No host `.env` or secrets read.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Grafana auth** | [R-0056 §1](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik) | **Anonymous Viewer** (`GF_AUTH_ANONYMOUS_*`, `GF_SECURITY_ALLOW_EMBEDDING`); auth-proxy deferred; no token-in-URL |
| **CSP / framing** | [R-0056 §2](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik) | Proxy strips `X-Frame-Options: deny`; SPA `frame-src 'self'`; same-origin — no cross-host cookie hacks |
| **Subpath serve** | [R-0056 §3](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik) | **Prefix strip** at `/analytics/grafana/` → root upstream; **`serve_from_sub_path=false`** |
| **WebSocket** | [R-0056 §4](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik) | Forward upgrade on `/api/live/`; proxy outside JWT stack; QA smoke live refresh |
| **Traefik + iframe** | [R-0056 §5](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik) | Same-origin embed reuses Traefik `auth`; reject public `GRAFANA_TRAEFIK_HOST` default |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Anonymous vs auth-proxy with OIDC/Traefik? | **Anonymous Viewer** — trust boundary = Traefik + internal-only Grafana (DEC-0056) |
| CSP / X-Frame-Options? | `allow_embedding` + proxy header rewrite; `frame-src 'self'` on SPA |
| Subpath / `GF_SERVER_*`? | Prefix strip; do not enable `GF_SERVER_SERVE_FROM_SUB_PATH` for MVP |
| WebSocket live panels? | Explicit upgrade forwarding; verify in QA |
| Theme/kiosk? | `?kiosk=tv` sufficient for chrome hide; theme match cosmetic |
| Traefik basic auth + iframe? | Same host — browser reuses basic auth; no second prompt |

### Risks surfaced (carry to architecture)

1. **Anonymous Grafana** — Viewer API exposed to anyone passing edge auth; keep Grafana off public internet
2. **WebSocket proxy gaps** — stale panels without obvious failure; mandatory QA smoke
3. **`GF_SERVER_ROOT_URL` mis-set** — broken dashboard asset links after execute
4. **Auth-proxy escalation path** — if Viewer too permissive, new DEC sub-decision (do not open public Grafana host)

### Recommended next steps

1. `/architecture` — confirm DEC-0057 proxy contract (prefix, env, Grafana `GF_*`, canonical UX table)
2. `/sprint-plan` — Decompose 7 AC after architecture

---

## discovery-20260602-us0011 — US-0011 unified analytics UI discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-02  
**Story:** US-0011  
**Next phase:** `/research` → `/architecture`

### Summary

Discovery refined **unified analytics in financegnome**: embed all six provisioned Grafana dashboards inside the SPA at `financegnome.omniflow.cc` without a separate Grafana site for day-to-day use. US-0010 released **internal Grafana** on the `traefik` Docker network (`GRAFANA_TRAEFIK_HOST` empty by default, host port `!reset`). Today only **`WealthPage`** opens Grafana in a **new tab** via `VITE_GRAFANA_URL` → `/d/portfolio`. Intake recommendation **[R-0054](docs/engineering/research.md#r-0054--unified-financegnome-analytics-shell-grafana-embed-vs-react-port)**: same-origin proxy from `flow-finance-ai` to `http://grafana:3000` + React `/analytics/*` kiosk iframes.

### Dashboard → route map (canonical)

| JSON | Folder | uid | Title | Slug | React path | Proxied embed |
|------|--------|-----|-------|------|------------|---------------|
| `platform-health.json` | Platform | `platform-health` | Platform Health | `platform-health` | `/analytics/platform-health` | `/d/platform-health/platform-health?kiosk=tv` |
| `analytics/cashflow.json` | Analytics | `cashflow` | Cashflow | `cashflow` | `/analytics/cashflow` | `/d/cashflow/cashflow?kiosk=tv` |
| `analytics/subscriptions.json` | Analytics | `subscriptions` | Subscriptions | `subscriptions` | `/analytics/subscriptions` | `/d/subscriptions/subscriptions?kiosk=tv` |
| `analytics/budgets.json` | Analytics | `budgets` | Budgets | `budgets` | `/analytics/budgets` | `/d/budgets/budgets?kiosk=tv` |
| `analytics/portfolio.json` | Analytics | `portfolio` | Portfolio | `portfolio` | `/analytics/portfolio` | `/d/portfolio/portfolio?kiosk=tv` |
| `analytics/forecast-horizons.json` | Analytics | `forecast-horizons` | Forecast Horizons | `forecast-horizons` | `/analytics/forecast-horizons` | `/d/forecast-horizons/forecast-horizons?kiosk=tv` |

- **Sidebar:** new **Analytics** nav group with six links (slug = uid).
- **Proxy prefix (recommended):** `/analytics/grafana/` → `http://grafana:3000/`; iframe base `VITE_GRAFANA_EMBED_BASE=/analytics/grafana` (deprecate `VITE_GRAFANA_URL`).
- **Wealth migration:** portfolio card → in-app `/analytics/portfolio` (AC-5).

### Partial implementation review

| Area | Status |
|------|--------|
| Grafana JSON + provisioning | **Done** (DEC-0012 uids) |
| US-0010 external Grafana networking | **Done** (DEC-0056; internal default) |
| React `/analytics/*` + sidebar group | **Missing** |
| Backend reverse proxy | **Missing** |
| Wealth external Grafana tab | **Present** — migrate at execute |

### Discovery decomposition evidence

- Feature/workflow count: nav + 6 iframe pages + proxy + Wealth link + future-chart doc (moderate — **single story retained**)
- Cross-cutting impact: `frontend`, `backend`, `.env.example`, user guide
- Acceptance breadth: 7 AC unchanged (`docs/product/acceptance.md#US-0011`)
- Risk surface: Traefik `auth` + OIDC + Grafana framing; proxy WebSocket/subpath; ECharts vs Grafana duplicate metrics

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#US-0011` | Route map + partial impl + open questions | pass |
| `docs/product/acceptance.md` US-0011 | 7 criteria still valid | pass |
| `grafana/provisioning/dashboards/**` | Six uids match map | pass |
| `frontend` WealthPage / App / AppLayout | Gaps documented | pass |
| R-0054 | Embed/proxy recommendation linked | pass |

`triad_hot_surface`: rollover 4 US-0009 tail sections → `handoffs/archive/po-to-tl-pack-20260602-a.md`; US-0011 discovery prepended; retained_body_lines=221, pack_ref=handoffs/archive/po-to-tl-pack-20260602-a.md

### Open questions (carry to research/architecture)

| Topic | Question |
|-------|----------|
| **Grafana auth** | Anonymous viewer behind financegnome boundary vs auth-proxy headers vs service token — which satisfies OIDC + Traefik `auth`? |
| **CSP / framing** | Required SPA `frame-src`; strip/rewrite `X-Frame-Options` on proxied Grafana responses? |
| **Subpath serve** | `GF_SERVER_ROOT_URL` / `GF_SERVER_SERVE_FROM_SUB_PATH` for `/analytics/grafana` vs root proxy |
| **WebSocket** | Live panel refresh through reverse proxy (Grafana 11) |
| **Theme** | `kiosk=tv` sufficient to hide Grafana chrome vs shell theme match |
| **Canonical UX** | Per-page doc: ECharts product view vs Grafana SQL view (Forecast/Wealth/Planning/Subscriptions) |

### Recommended next steps

1. `/research` — Spike same-origin proxy + iframe on external profile; Grafana anonymous/auth-proxy matrix; CSP sample headers (extends R-0054)
2. `/architecture` — DEC for proxy prefix, env contract (`GRAFANA_UPSTREAM`, `VITE_GRAFANA_EMBED_BASE`), Grafana `GF_*` settings, future-chart guideline
3. `/sprint-plan` — Decompose 7 AC after architecture

---

## intake-20260602-us0012 — Auto-provision application database on first start

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-02  
**Story:** US-0012  
**Next phase:** `/discovery` → `/architecture`

### Summary

Operator request: **create `flow_finance_ai` automatically on first app start** if it does not exist, instead of manual SQL before `docker compose up` (today per US-0010 runbook).

### Split decision

- **Single story** — db bootstrap hook + docs + tests

### Recommended approach (R-0055)

- **In-app pre-migration bootstrap** connecting to maintenance DB `postgres`
- Optional **`DATABASE_BOOTSTRAP_URL`** when `DATABASE_USER` lacks `CREATEDB`
- Idempotent create + `CREATE EXTENSION timescaledb` attempt; then existing SQLx migrations
- **Alternative:** Compose one-shot init job — heavier for external profile

### Risks

- Bootstrap admin URL mishandling (secrets in logs)
- TimescaleDB absent on host — DB exists but migrations still fail (document separately)
- Bootstrap does **not** fix wrong `DATABASE_PASSWORD` (observed on omniflow)

### Out of scope

- Host TimescaleDB package install, Firefly DB, embedded Postgres service

### Intake evidence

- `selected_pack`: `small-intake-pack`
- `intake_run_id`: `intake-20260602-auto-provision-db`
- Bundle: `handoffs/intake_evidence/intake-20260602-auto-provision-db.json`
- Research: **R-0055**

### Recommended next steps

1. `/discovery` — env contract + privilege matrix for shared `postgres`
2. `/architecture` — DEC for bootstrap URL and fail-closed codes

---

## intake-20260602-us0011 — Unified analytics UI in financegnome

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-02  
**Story:** US-0011  
**Next phase:** `/discovery` → `/architecture`

### Summary

Operator request: **all charts and Grafana content inside one web UI** at `financegnome.omniflow.cc`; future charts must follow the same shell. Today Grafana is a separate container/UI; only Wealth links out in a new tab.

### Split decision

- **Single story** — unified Analytics navigation + embed/proxy for existing six Grafana dashboards + future-chart guideline
- **Not in scope:** full SQL→ECharts migration; removing Grafana container

### Recommended approach (R-0054)

- **MVP:** same-origin proxy from `flow-finance-ai` to `http://grafana:3000` + React `/analytics/*` iframe/kiosk routes
- **Future:** React-first for new charts; embed only where SQL/Grafana still justified
- **Alternative rejected for MVP:** public `GRAFANA_TRAEFIK_HOST` + external iframe (double auth, extra host)

### Risks

- Traefik basic auth + OIDC + Grafana anonymous/proxy policy interaction
- CSP / `X-Frame-Options` on proxied responses
- Duplicate UX where ECharts and Grafana show similar metrics (document which is canonical per page)

### Intake evidence

- `selected_pack`: `small-intake-pack`
- `intake_run_id`: `intake-20260602-unified-charts-ui`
- Bundle: `handoffs/intake_evidence/intake-20260602-unified-charts-ui.json`
- Research: **R-0054**

### Optional artifacts (scratchpad flags)

- `SPEC_PACK_MODE=1` — CRS at `docs/spec-pack/US-0011/` during architecture
- `USER_GUIDE_MODE=1` — `docs/user-guides/US-0011.md` at execute

### Recommended next steps

1. `/discovery` — dashboard uid → route map; auth/proxy spike on omniflow
2. `/architecture` — DEC for proxy vs embed; CSP and Grafana auth mode

---

## research-20260601-us0010 — US-0010 omniflow external deploy technical research

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-01  
**Story:** US-0010  
**Next phase:** `/architecture`

### Summary

Web research completed for US-0010 external omniflow deployment. Added **[R-0053](docs/engineering/research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci)** resolving all discovery open questions: TimescaleDB preflight on shared Postgres, `bundled-firefly` profile split recommendation, Traefik label env defaults, Grafana internal-only default, OIDC redirect documentation, AC-6 smoke template, and Compose CI config guard. Local verification: external-only merge yields `flow-finance-ai` + `grafana` only; `minimal+external` still includes `firefly-iii` (confirms guard need). No host `.env` or secrets read outside repo.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **TimescaleDB** | [R-0053 §1](docs/engineering/research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci) | Operator preflight on `flow_finance_ai`; server packages + `shared_preload_libraries` + restart; migration 001 fail-fast |
| **Profile guard** | [R-0053 §2](docs/engineering/research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci) | **`bundled-firefly` split** + CI service-list assert; doc-only warning insufficient |
| **Traefik env labels** | [R-0053 §3](docs/engineering/research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci) | `${TRAEFIK_HOST:-financegnome.omniflow.cc}`, `${TRAEFIK_MIDDLEWARE:-auth}` — defaults, not required |
| **Grafana exposure** | [R-0053 §4](docs/engineering/research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci) | Internal-only default; `grafana.ports: !reset []` in overlay; optional `${GRAFANA_TRAEFIK_HOST}` |
| **OIDC redirects** | [R-0053 §5](docs/engineering/research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci) | Document IdP URIs for `https://financegnome.omniflow.cc/callback`; runtime origin fallback; optional preflight script |
| **Smoke test (AC-6)** | [R-0053 §6](docs/engineering/research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci) | Eight-step checklist: TimescaleDB, Firefly DNS, PAT, health, TLS, 401, no duplicate Firefly |
| **Compose CI** | [R-0053 §7](docs/engineering/research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci) | Extend `tests/run-tests.sh`; external merge `--services` = `flow-finance-ai`, `grafana` only |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| TimescaleDB on shared `postgres`? | **Verify operator-side** — not bundled; preflight SQL + fail-fast migrations |
| `bundled-firefly` vs doc-only? | **Profile split** recommended; update US-0001 start commands; CI guard mandatory |
| Required `TRAEFIK_*` vars? | **No** — sensible defaults with optional override |
| Grafana public? | **Internal-only MVP**; optional second host via env gate |
| OIDC doc vs validation? | **Doc-first** + optional script; IdP redirect URI registration out of scope |
| Smoke template? | **R-0053 §6** table — operator records on Debian host for AC-6 |
| Compose CI guard? | **`config --services` assert** — verified locally on Compose v2.29 |

### Risks surfaced (carry to architecture)

1. **TimescaleDB absent on host Postgres** — blocks startup until operator installs extension (server + DB level)
2. **Profile union** — `minimal+external` starts duplicate Firefly until `bundled-firefly` split lands
3. **Grafana port leak** — external overlay still publishes `:3000` until `!reset` added
4. **OIDC silent skip** — `AUTH_DEV_BYPASS=true` smoke path must not mask IdP misconfig for auth-on deployments
5. **Compose `!reset`** — requires Compose ≥2.24; document operator minimum

### Recommended next steps

1. `/architecture` — DEC-xxxx for `bundled-firefly` split, external overlay contract (Traefik/Grafana env gates), operator runbook structure, TimescaleDB preflight
2. `/sprint-plan` — Decompose 6 AC (infra-only; expect smaller task count than feature stories)

---

## discovery-20260601-us0010 — US-0010 external omniflow deployment discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-01  
**Story:** US-0010  
**Next phase:** `/research`

### Summary

Discovery refined the **omniflow host deployment** story: attach Flow Finance AI to existing host **`firefly`**, **`postgres`**, and **`traefik`** containers without duplicates; publish the app at **`https://financegnome.omniflow.cc`** using the host **`auth`** basic-auth middleware (same pattern as `finance.omniflow.cc`). Partial implementation exists (`docker-compose.external.yml`, `external` profile on `flow-finance-ai`/`grafana`, `.env.example` omniflow block). Discovery addresses operator concerns on **two-file compose pattern**, **Firefly profile isolation**, **credential policy**, and **shared Postgres DB topology**. Builds on intake **R-0052** and released US-0001 Compose profiles.

### Operator concerns addressed

| Concern | Discovery resolution |
|---------|---------------------|
| **Why two YAML files?** | **`docker-compose.yml`** = base dev/greenfield stack; **`docker-compose.external.yml`** = merge overlay for omniflow only (external `traefik` network, DNS overrides, Traefik labels, strip host ports). Invocation: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d`. |
| **Duplicate Firefly risk** | `firefly-iii` is **not** on `external` profile today — safe when operator uses **`--profile external` alone**. **Risk:** combining `minimal` + `external` would start bundled Firefly. **Recommend execute:** dedicated `bundled-firefly` profile **or** doc-only guard + compose config test. |
| **No hardcoded credentials** | Secrets only via `${VAR}` / `${VAR:?}` in operator `.env`. **Gaps:** Grafana `:-admin` defaults in base file; hardcoded Traefik Host in overlay — parameterize `${TRAEFIK_HOST}` / `${TRAEFIK_MIDDLEWARE}`. Host `auth` credentials stay on Traefik stack (out of scope). |
| **Postgres topology** | App DB **`flow_finance_ai`** on shared container **`postgres`** (`DATABASE_HOST=postgres` on `traefik` network). Firefly ledger DB separate; connector **`FIREFLY_BASE_URL=http://firefly:8080`**. Operator bootstrap SQL documented in `.env.example`. |

### Scope refinements (backlog updated)

- **Two-file pattern:** base + external overlay; alternative single-file rejected (readability + Traefik label isolation)
- **External profile services:** `flow-finance-ai` + `grafana` only; no `postgres`/`firefly-iii` service definitions
- **Traefik:** router `financegnome`, `websecure`, `myresolver`, middleware `auth`; Grafana internal-only by default
- **Env required (external):** `DATABASE_PASSWORD`, `FIREFLY_PERSONAL_ACCESS_TOKEN`; optional OIDC/VITE vars for public SPA URL
- **Port note:** `STATS_FORECAST_PORT=8091` when `full` on same host (8090 clash per intake)
- **Out of scope unchanged:** host stack edits, Traefik ACME/DNS, replacing `auth` middleware, Firefly migration

### Partial implementation review

| Artifact | Status |
|----------|--------|
| `docker-compose.external.yml` | Present — traefik network, label set, env overrides, `ports: !reset` |
| `external` profile membership | Present on `flow-finance-ai`, `grafana`; absent on `firefly-iii` |
| `.env.example` omniflow section | Present — needs TRAEFIK_* vars + runbook expansion in execute |
| Operator smoke test (AC-6) | **Not recorded** |

### Discovery decomposition evidence

- Feature/workflow count: compose overlay + profile guard + env/Traefik wiring + runbook + host smoke (low–moderate — **single story retained**)
- Cross-cutting impact: compose files, `.env.example`, operator docs only
- Acceptance breadth: unchanged (6 criteria in `docs/product/acceptance.md#US-0010`)
- Risk surface: profile combination duplicates, TimescaleDB extension on shared Postgres, hardcoded Traefik host, OIDC redirect mismatch, compose `!reset` portability

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#us-0010` | Discovery refinements + partial impl review + open questions | pass |
| `docs/product/acceptance.md` US-0010 | 6 criteria still valid; no AC rewrite at discovery | pass |
| `docker-compose.yml` + `docker-compose.external.yml` | Partial impl aligned; gaps documented for execute | pass |
| `.env.example` | Omniflow placeholders present; no secrets read from host | pass |
| R-0052 | Host stack structural facts referenced; no host `.env` access | pass |

`triad_hot_surface`: rollover 4 US-0008 sections → `handoffs/archive/po-to-tl-pack-20260601-b.md`; US-0010 discovery prepended; retained_body_lines=337, pack_ref=handoffs/archive/po-to-tl-pack-20260601-b.md

### Open questions (carry to research/architecture)

- **TimescaleDB extension** on shared host `postgres` — verify procedure vs migration failure?
- **`bundled-firefly` profile split** vs doc-only guard — impact on US-0001 minimal docs/CI?
- **Env-parameterized Traefik labels** — required vars vs sensible defaults?
- **Grafana public exposure** on omniflow — internal-only vs second Traefik host?
- **OIDC redirect URI** updates for `https://financegnome.omniflow.cc` — doc-only or validation?
- **Smoke test template** for AC-6 — exact commands from traefik network?
- **Compose CI guard** — `docker compose … config` excludes `firefly-iii`/`postgres` under external-only merge?

### Recommended next steps

1. `/research` — TimescaleDB on shared Postgres, compose profile guard pattern, Traefik label env substitution, smoke test checklist (extends R-0052)
2. `/architecture` — `bundled-firefly` vs guard decision, external overlay contract, operator runbook structure, DEC-xxxx for omniflow deploy defaults
3. `/sprint-plan` — Decompose 6 AC after architecture (infra-only; expect smaller task count than feature stories)

---
