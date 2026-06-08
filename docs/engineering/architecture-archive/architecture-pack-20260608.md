# Architecture archive pack (2026-06-08)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## US-0010 — External Firefly/Postgres & Traefik deployment on omniflow host`
- Last archived heading: `## US-0010 — External Firefly/Postgres & Traefik deployment on omniflow host`
- Verification tuple (mandatory):
  - archived_body_lines=219
  - preamble_lines=10
  - retained_body_lines=2983

---

## US-0010 — External Firefly/Postgres & Traefik deployment on omniflow host

**Status:** architecture complete (2026-06-02)  
**Research:** R-0052, R-0053 (extends R-0004, R-0005)  
**Decisions:** DEC-0056  
**Depends on:** US-0001 Compose profiles, external DB wiring, Grafana provisioning

### System context (omniflow external profile)

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Operator browser ──HTTPS──► Traefik (host stack, network traefik)          │
│         │ basic-auth middleware `auth`                                       │
│         ▼                                                                    │
│  https://financegnome.omniflow.cc ──► flow-finance-ai:8080 (no host ports)  │
└─────────────────────────────────────────────────────────────────────────────┘
         │
         │  finance_goblin project (profile external + overlay merge only)
         │
         ├── flow-finance-ai ──GET──► firefly:8080 (host container, DNS on traefik)
         │                      └──► postgres:5432 / flow_finance_ai (TimescaleDB)
         │
         └── grafana (internal-only default — traefik network, no public router)

Host stacks (read-only alignment — not modified by finance_goblin):
  /workdir/firefly/docker-compose.yml     → container `firefly`, Host finance.omniflow.cc
  /workdir/services/docker-compose.yml    → container `postgres`
  /workdir/networking/docker-compose.yml  → Traefik, middleware `auth`, certresolver myresolver
```

**Scope:** deployment wiring only — no application feature changes, no host stack edits in-repo.

### Compose architecture (DEC-0056)

#### Two-file merge pattern

| File | Role |
|------|------|
| `docker-compose.yml` | Base stack: images, healthchecks, profiles, dev defaults (`host.docker.internal`) |
| `docker-compose.external.yml` | Merge overlay only: external `traefik` network, in-network DNS overrides, Traefik labels, port `!reset` |

**Canonical omniflow invocation:**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d
```

Operator `.env` may set `COMPOSE_FILE=docker-compose.yml:docker-compose.external.yml` and `COMPOSE_PROFILES=external`.

**Alternative considered:** env-conditional single compose — rejected (overlay keeps Traefik labels out of local minimal runs; discovery 2026-06-01).

#### Profile model (post `bundled-firefly` split)

| Profile | Services | Use case |
|---------|----------|----------|
| `minimal` | `flow-finance-ai`, `grafana` | Dev/CI baseline **without** bundled Firefly |
| `bundled-firefly` | `firefly-iii` | Greenfield Firefly container alongside minimal |
| `standard` | minimal + bundled-firefly + `redis` | Extended dev |
| `full` | standard + `ollama`, `stats-forecast` | ML/AI sidecar path (remap `STATS_FORECAST_PORT=8091` on omniflow — host 8090 taken) |
| `external` | `flow-finance-ai`, `grafana` | Omniflow attach — **no** `firefly-iii`, **no** `postgres` |
| `oidc` | Authentik stack | Unchanged; optional |

**Greenfield dev (replaces prior `--profile minimal` alone):**

```bash
docker compose --profile minimal --profile bundled-firefly up --build
```

**Profile union rule:** Compose profiles are a union. **Never** combine `external` with `minimal`, `standard`, `full`, or `bundled-firefly` on omniflow — CI must assert `minimal+external` does not list `firefly-iii` after split.

#### External overlay contract

**`flow-finance-ai` overrides:**

| Aspect | Base default | External overlay |
|--------|--------------|------------------|
| Host ports | `${FLOW_PORT:-8080}:8080` | `ports: !reset []` |
| Networks | implicit default | `traefik` (external) |
| `DATABASE_HOST` | `host.docker.internal` | `postgres` |
| `FIREFLY_BASE_URL` | `http://firefly-iii:8080` | `http://firefly:8080` |
| Traefik | none | router `financegnome` (see below) |

**`grafana` overrides:**

| Aspect | Base default | External overlay |
|--------|--------------|------------------|
| Host ports | `${GRAFANA_PORT:-3000}:3000` | `ports: !reset []` (execute) |
| Networks | implicit default | `traefik` (external) |
| Public Traefik router | none | only when `${GRAFANA_TRAEFIK_HOST}` non-empty (opt-in) |

**Forbidden in overlay:** new `postgres`, `firefly`, or `firefly-iii` service definitions (AC-1).

#### Traefik routing (env-parameterized)

Fixed router/service id **`financegnome`** — must not collide with host `firefly` router (R-0052).

```yaml
labels:
  - traefik.enable=true
  - traefik.docker.network=traefik
  - traefik.http.routers.financegnome.rule=Host(`${TRAEFIK_HOST:-financegnome.omniflow.cc}`)
  - traefik.http.routers.financegnome.entrypoints=websecure
  - traefik.http.routers.financegnome.tls=true
  - traefik.http.routers.financegnome.tls.certresolver=myresolver
  - traefik.http.routers.financegnome.middlewares=${TRAEFIK_MIDDLEWARE:-auth}
  - traefik.http.services.financegnome.loadbalancer.server.port=8080
```

Reuse host global basic-auth middleware **`auth`** (`credentials.passwd` on Traefik container — out of scope). TLS via existing `myresolver` wildcard `*.omniflow.cc`.

**Public Firefly UI** remains `https://finance.omniflow.cc` (unchanged). Connector uses in-container `http://firefly:8080`.

#### PostgreSQL / TimescaleDB preflight

Flow Finance AI requires TimescaleDB (migration `001_initial.sql` → `CREATE EXTENSION timescaledb`; US-0002+ hypertables). Shared host container `postgres:latest` does **not** guarantee extension availability (R-0053 §1).

**Operator steps before first `compose up`:**

1. Create database `flow_finance_ai` and role `finance` on shared `postgres` (grants documented in `.env.example`).
2. Verify server packages + `shared_preload_libraries = 'timescaledb'` + Postgres restart if extension missing.
3. On `flow_finance_ai`: `CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;`
4. Preflight SQL: `SELECT extversion FROM pg_extension WHERE extname='timescaledb';` — non-null required.

**Failure mode:** backend migration panic; `/health` never OK until fixed. Firefly DB on same container does **not** imply TimescaleDB on `flow_finance_ai`.

**Alternative considered:** skip extension in migration 001 for external mode — rejected (breaks US-0002–US-0009; violates released architecture).

#### Environment variables (operator `.env` — names only)

| Variable | External mode | Notes |
|----------|---------------|-------|
| `DATABASE_HOST` | `postgres` | Overlay default |
| `DATABASE_PASSWORD` | required | `${DATABASE_PASSWORD:?}` in compose |
| `FIREFLY_BASE_URL` | `http://firefly:8080` | Overlay default |
| `FIREFLY_PERSONAL_ACCESS_TOKEN` | required for health/sync | Server-side only |
| `COMPOSE_FILE` | `docker-compose.yml:docker-compose.external.yml` | Optional convenience |
| `COMPOSE_PROFILES` | `external` | Must not combine with other profiles |
| `TRAEFIK_HOST` | default `financegnome.omniflow.cc` | Optional override |
| `TRAEFIK_MIDDLEWARE` | default `auth` | Host Traefik middleware name |
| `GRAFANA_TRAEFIK_HOST` | empty (internal-only) | Set only for optional public Grafana |
| `GRAFANA_ADMIN_PASSWORD` | operator-set | Replace weak base default |
| `STATS_FORECAST_PORT` | `8091` if `full` on same host | Host 8090 used by `firefly_product_manager` |
| `FIREFLY_APP_KEY`, `FIREFLY_DB_*` | only when `bundled-firefly` profile | Not required for external |
| `VITE_OIDC_*`, `OIDC_*` | when auth enabled | See OIDC section |

No literal passwords in committed YAML. Traefik basic-auth credentials remain on host Traefik stack only.

#### OIDC on public URL (document-only)

SPA (`frontend/src/auth/oidc.ts`) defaults `redirect_uri` to `${window.location.origin}/callback` when `VITE_OIDC_REDIRECT_URI` unset — works for omniflow without rebuild if IdP allows.

**Operator IdP registration (out of scope to automate):**

| Setting | Value |
|---------|-------|
| Redirect URI | `https://financegnome.omniflow.cc/callback` |
| Post-logout redirect | `https://financegnome.omniflow.cc/` |
| Web origin / CORS | `https://financegnome.omniflow.cc` |

AC-6 smoke may use `AUTH_DEV_BYPASS=true`; auth-on deployments must register IdP URIs explicitly. Traefik basic-auth and OIDC are orthogonal (edge vs app session).

#### CI / config guard (R-0053 §7)

Extend compose config check (no live `docker up` required):

```bash
export DATABASE_PASSWORD=ci FIREFLY_APP_KEY=base64:32RandomCharactersMinimumRequired== \
       FIREFLY_DB_PASSWORD=ci AUTHENTIK_SECRET_KEY=ci
services=$(docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external config --services | sort)
# expect: flow-finance-ai, grafana only
docker compose --profile minimal --profile bundled-firefly config --services
# guard: minimal+external must NOT include firefly-iii after bundled-firefly split
```

Wire through `tests/run-tests.sh` or `scripts/compose-config-check.sh`; CI reads `TEST_COMMAND` from runbook.

#### Operator smoke test (AC-6)

Eight-step checklist on Debian host — full table in `docs/engineering/runbook.md` (omniflow section). Record pass/fail per step; **never commit operator credentials**.

Key pass criteria: TimescaleDB version non-null; `http://firefly:8080/api/v1/about` from traefik network; backend `/health` OK; `https://financegnome.omniflow.cc/health` with basic-auth → 200; unauthenticated → 401; no `firefly-iii` in project services.

### Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| TimescaleDB missing on shared Postgres | Operator preflight block; fail-fast migrations | R-0053 §1, R-0004 |
| Profile union starts duplicate Firefly | `bundled-firefly` split + CI guard + runbook warning | DEC-0056, R-0053 §2 |
| Traefik router name collision | Fixed router id `financegnome` | R-0052 |
| Grafana admin exposed | Internal-only default; `!reset` ports; opt-in host only | DEC-0056, R-0053 §4 |
| Hardcoded Traefik host | `TRAEFIK_HOST` / `TRAEFIK_MIDDLEWARE` defaults | DEC-0056 |
| Compose `!reset` unsupported | Document Compose ≥2.24 minimum | R-0053 |
| OIDC misconfig masked by dev bypass | Smoke documents auth-off vs auth-on paths | R-0053 §5 |
| Port 8090 conflict with host service | `STATS_FORECAST_PORT=8091` when using `full` | R-0052 |
| Weak Grafana defaults | Require operator `GRAFANA_ADMIN_PASSWORD` in external docs | backlog discovery |

### Decisions (US-0010)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0056 | Omniflow external deploy | `bundled-firefly` split; two-file overlay contract; env Traefik labels; Grafana internal-only default |

Full record: `decisions/DEC-0056.md`

### Out of scope (US-0010)

- Editing host Firefly/Postgres/Traefik compose files in-repo
- Changing Firefly version or migrating Firefly data
- Modifying Traefik ACME/DNS or replacing host `auth` middleware
- OIDC IdP provisioning (redirect URI documentation only)
- Application code changes (connector, sync, UI features unchanged)

### Next phase

`/sprint-plan` — decompose 6 acceptance criteria (infra-only; expect smaller task count than feature stories; no sprint split unless >12 tasks).

---

