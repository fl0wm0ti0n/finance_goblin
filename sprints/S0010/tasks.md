# Tasks — Sprint S0010

**Story:** US-0010  
**Task count:** 10 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0109 | bundled-firefly profile split | open | AC-1 |
| T-0110 | Traefik env-parameterized labels | open | AC-3 |
| T-0111 | Grafana external overlay hardening | open | AC-1, AC-2 |
| T-0112 | .env.example omniflow block | open | AC-5 |
| T-0113 | Compose config CI guard | open | AC-1 |
| T-0114 | Runbook deploy section updates | open | AC-1, AC-2, AC-3, AC-4 |
| T-0115 | US-0001 user guide start commands | open | AC-1 |
| T-0116 | Operator user guide US-0010 | open | AC-5, AC-6 |
| T-0117 | External overlay config validation | open | AC-2 |
| T-0118 | Operator AC-6 smoke evidence | open | AC-4, AC-6 |

---

## T-0109 — bundled-firefly profile split

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0056, R-0053 §2

### Description

Move `firefly-iii` profile membership in `docker-compose.yml`:

```yaml
# Before
profiles: [minimal, standard, full]

# After
profiles: [bundled-firefly, standard, full]
```

Update file header comments:

- Greenfield dev: `docker compose --profile minimal --profile bundled-firefly up --build`
- Omniflow: `--profile external` only (no `bundled-firefly`)

**Frozen:** do not add `postgres` or host `firefly` services; do not put `firefly-iii` on `external` profile.

### Done when

- [ ] `firefly-iii` profiles are `[bundled-firefly, standard, full]` only
- [ ] `docker compose --profile minimal config --services` lists `flow-finance-ai`, `grafana` — no `firefly-iii`
- [ ] `docker compose --profile minimal --profile bundled-firefly config --services` includes `firefly-iii`
- [ ] Compose file comments reflect new greenfield invocation

---

## T-0110 — Traefik env-parameterized labels

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0056, R-0053 §3, R-0052

### Description

Replace hardcoded Traefik labels in `docker-compose.external.yml`:

```yaml
labels:
  - traefik.http.routers.financegnome.rule=Host(`${TRAEFIK_HOST:-financegnome.omniflow.cc}`)
  - traefik.http.routers.financegnome.middlewares=${TRAEFIK_MIDDLEWARE:-auth}
```

Keep fixed router/service id **`financegnome`** (collision guard vs host `firefly` router).

Other labels unchanged: `websecure`, `tls=true`, `tls.certresolver=myresolver`, `loadbalancer.server.port=8080`.

Verify merged config renders expected host when `TRAEFIK_HOST` unset and when overridden.

### Done when

- [ ] No hardcoded `financegnome.omniflow.cc` in overlay (uses interpolation default)
- [ ] No hardcoded `middlewares=auth` without env default syntax
- [ ] Router id remains `financegnome`
- [ ] `docker compose … config` shows expected rule with placeholder env

---

## T-0111 — Grafana external overlay hardening

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0056, R-0053 §4

### Description

Extend `docker-compose.external.yml` for `grafana`:

```yaml
grafana:
  ports: !reset []
  networks:
    traefik:
```

**Optional public router** — only when `${GRAFANA_TRAEFIK_HOST}` is non-empty at deploy time. Pattern options (pick simplest that works with Compose ≥2.24):

- Conditional label block using extension field / override file fragment, **or**
- Document that operator sets `GRAFANA_TRAEFIK_HOST` and overlay includes router labels referencing `${GRAFANA_TRAEFIK_HOST}` with empty default suppressing route (if Compose omits empty Host rules — validate)

When public gate active, reuse `${TRAEFIK_MIDDLEWARE:-auth}` on Grafana router; distinct router name (e.g. `grafana-financegnome`) to avoid collision.

Runbook must state internal-only default and SSH tunnel / traefik-network curl access paths.

### Done when

- [ ] External merge clears Grafana host port publish (`!reset []`)
- [ ] Grafana joins external `traefik` network
- [ ] No Traefik Host router labels by default (empty `GRAFANA_TRAEFIK_HOST`)
- [ ] Optional public router documented and wired when env set
- [ ] Runbook Grafana access section matches overlay behavior

---

## T-0112 — .env.example omniflow block

**Status:** open  
**Depends on:** T-0110, T-0111  
**Decisions:** DEC-0056

### Description

Expand `.env.example` omniflow section:

| Addition | Purpose |
|----------|---------|
| Top comment fix | Greenfield: `--profile minimal --profile bundled-firefly up` |
| `COMPOSE_FILE=docker-compose.yml:docker-compose.external.yml` | Uncommented example |
| `COMPOSE_PROFILES=external` | Profile-only rule |
| `TRAEFIK_HOST=financegnome.omniflow.cc` | Optional override (default in overlay) |
| `TRAEFIK_MIDDLEWARE=auth` | Host middleware name |
| `GRAFANA_TRAEFIK_HOST=` | Empty = internal-only |
| OIDC notes | Redirect/post-logout/origin for `https://financegnome.omniflow.cc` |
| TimescaleDB bootstrap SQL comment | Retain/create DB grants on shared `postgres` |
| `STATS_FORECAST_PORT=8091` | When `full` on omniflow host |

Clarify which vars are **not** required for external (`FIREFLY_APP_KEY`, `FIREFLY_DB_*` — only for `bundled-firefly`).

No literal operator secrets.

### Done when

- [ ] All operator-required external variables documented with comments
- [ ] Greenfield start command uses `bundled-firefly`
- [ ] OIDC redirect URI examples for public omniflow URL included
- [ ] Profile-only rule for external documented inline

---

## T-0113 — Compose config CI guard

**Status:** open  
**Depends on:** T-0109  
**Decisions:** DEC-0056, R-0053 §7

### Description

Extend `tests/run-tests.sh` (or add `scripts/compose-config-check.sh` invoked by harness):

```bash
export DATABASE_PASSWORD=ci FIREFLY_APP_KEY=base64:32RandomCharactersMinimumRequired== \
       FIREFLY_DB_PASSWORD=ci AUTHENTIK_SECRET_KEY=ci

# AC-1: external-only services
services=$(docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external config --services | sort)
# expect exactly: flow-finance-ai, grafana

# Anti-combination guard (post bundled-firefly split)
bad=$(docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile minimal --profile external config --services)
# must NOT contain firefly-iii or postgres

# Greenfield regression
good=$(docker compose --profile minimal --profile bundled-firefly config --services)
# must contain firefly-iii
```

Fail harness with clear message on mismatch. Document in runbook §7.

### Done when

- [ ] External-only assert passes in `bash tests/run-tests.sh`
- [ ] Anti-combination assert fails CI if `firefly-iii` appears in `minimal+external` merge
- [ ] Greenfield `minimal+bundled-firefly` assert includes `firefly-iii`
- [ ] No live `docker up` required — config-only checks

---

## T-0114 — Runbook deploy section updates

**Status:** open  
**Depends on:** T-0109, T-0110, T-0111, T-0112  
**Decisions:** DEC-0056, R-0053

### Description

Update `docs/engineering/runbook.md`:

- Local/greenfield start: `--profile minimal --profile bundled-firefly` (replace bare `--profile minimal` where it implied bundled Firefly)
- Omniflow section: canonical two-file invocation; profile union **never combine external with minimal/standard/full/bundled-firefly**
- Compose **≥2.24** requirement for `!reset`
- TimescaleDB preflight block cross-linked to `.env.example`
- Traefik env overrides (`TRAEFIK_HOST`, `TRAEFIK_MIDDLEWARE`)
- Grafana internal access paths; optional `GRAFANA_TRAEFIK_HOST`
- OIDC IdP registration table (doc-only)
- AC-6 smoke table reference (full eight steps — align with T-0118)
- CI guard command cross-ref to `tests/run-tests.sh`

**Security:** env var names only — no host `.env` or Traefik credential paths committed.

### Done when

- [ ] Greenfield and omniflow start commands accurate post split
- [ ] Profile union warning prominent
- [ ] TimescaleDB preflight steps present before first `up`
- [ ] AC-6 smoke table complete (eight steps)
- [ ] Compose version minimum documented

---

## T-0115 — US-0001 user guide start commands

**Status:** open  
**Depends on:** T-0109  
**Decisions:** DEC-0056

### Description

Update `docs/user-guides/US-0001.md` Compose profile start commands:

- Replace `--profile minimal up` (when Firefly bundled) with `--profile minimal --profile bundled-firefly up`
- Note that `minimal` alone starts app + Grafana only (external DB + optional external Firefly)
- Cross-link to `docs/user-guides/US-0010.md` for omniflow external deploy

Minimal diff — preserve existing US-0001 operator workflows.

### Done when

- [ ] US-0001 guide reflects `bundled-firefly` profile
- [ ] Omniflow external pointer to US-0010 guide added
- [ ] No contradictory start commands remain

---

## T-0116 — Operator user guide US-0010

**Status:** open  
**Depends on:** T-0112, T-0114  
**Decisions:** DEC-0056, USER_GUIDE_MODE=1

### Description

Create `docs/user-guides/US-0010.md`:

- Prerequisites: host `firefly`, `postgres`, `traefik` on network `traefik`
- TimescaleDB preflight on `flow_finance_ai`
- `.env` setup from `.env.example` (required vs optional vars)
- Deploy command (two-file, `--profile external` only)
- Traefik URL `https://financegnome.omniflow.cc` + basic-auth note (host middleware — credentials not in repo)
- Backend `/health` and Firefly PAT requirement
- Grafana internal access; optional public host
- OIDC redirect registration when auth enabled
- Troubleshooting: TimescaleDB missing, profile union duplicate Firefly, migration panic, 401 without auth
- Operator UAT checklist mirroring six acceptance criteria

### Done when

- [ ] User guide covers all six acceptance criteria from operator perspective
- [ ] Profile-only rule and two-file pattern documented step-by-step
- [ ] TimescaleDB and PAT prerequisites explicit
- [ ] Troubleshooting for common failure modes included

---

## T-0117 — External overlay config validation

**Status:** open  
**Depends on:** T-0110, T-0111  
**Decisions:** DEC-0056, R-0053

### Description

Add compose config assertions (in `tests/run-tests.sh` alongside T-0113 or dedicated function) validating merged external overlay:

| Check | Expected in merged config |
|-------|---------------------------|
| `flow-finance-ai` networks | includes `traefik` |
| `flow-finance-ai` ports | empty / reset (no host bind) |
| `flow-finance-ai` environment | `DATABASE_HOST=postgres`, `FIREFLY_BASE_URL=http://firefly:8080` |
| `grafana` networks | includes `traefik` |
| External network | `traefik.external: true` |
| Services absent | no `postgres`, `firefly`, `firefly-iii` service definitions in external merge |

Use `docker compose … config` output parsing (yaml or grep) — no container runtime required.

### Done when

- [ ] Traefik network membership asserted for app + Grafana
- [ ] DNS override env vars present in merged config
- [ ] Host ports cleared on external merge for both services
- [ ] No forbidden services in external project service list
- [ ] Checks run in CI harness

---

## T-0118 — Operator AC-6 smoke evidence

**Status:** open  
**Depends on:** T-0109, T-0110, T-0111, T-0112, T-0113, T-0114  
**Decisions:** DEC-0056, R-0053 §6

### Description

Execute the eight-step operator smoke checklist on the Debian omniflow host (or operator-managed equivalent) per runbook §6:

| Step | Check |
|------|-------|
| 1 | TimescaleDB version non-null on `flow_finance_ai` |
| 2 | `curl http://firefly:8080/api/v1/about` from traefik network → 200 |
| 3 | `FIREFLY_PERSONAL_ACCESS_TOKEN` non-empty in container |
| 4 | `curl http://flow-finance-ai:8080/health` from traefik network → OK |
| 5 | `curl https://financegnome.omniflow.cc/health` with basic-auth → 200 + valid TLS |
| 6 | Same URL without credentials → 401 |
| 7 | `docker compose … ps --services` → no `firefly-iii` |
| 8 | Backend logs — no migration panic |

Record pass/fail per step in `sprints/S0010/smoke-evidence.md` (or release notes handoff). **Never commit credentials** — use placeholders in committed evidence.

If live host unavailable during execute, document blocker and leave checklist template with PENDING steps for operator post-merge (QA must flag PASS-with-prerequisites).

### Done when

- [ ] Eight-step checklist recorded with pass/fail per step
- [ ] Evidence file committed without secrets
- [ ] AC-4 (`/health` OK) and AC-6 (recorded smoke) satisfied or blocker documented
- [ ] Auth-off (`AUTH_DEV_BYPASS`) vs auth-on paths noted if applicable

---

## Execution order (recommended)

1. **Profile split:** T-0109
2. **Overlay hardening (parallel):** T-0110 ∥ T-0111
3. **Env template:** T-0112 (after T-0110, T-0111)
4. **CI guards:** T-0113 (after T-0109) → T-0117 (after T-0110, T-0111)
5. **Documentation:** T-0114 (after overlay + env) → T-0115 ∥ T-0116
6. **Smoke:** T-0118 (last — requires deploy-ready stack)

## Acceptance coverage map

| AC | Tasks |
|----|-------|
| AC1 External profile does not create `firefly-iii` or `postgres` | T-0109, T-0111, T-0113, T-0115, T-0118 |
| AC2 App joins `traefik`; reaches `postgres` and `firefly` by DNS | T-0111, T-0114, T-0117, T-0118 |
| AC3 Traefik routes `financegnome.omniflow.cc` with `auth` + TLS | T-0110, T-0112, T-0114, T-0116, T-0118 |
| AC4 Backend `/health` OK with external DB + PAT | T-0112, T-0114, T-0116, T-0118 |
| AC5 `.env.example` documents operator-required external vars | T-0112, T-0116 |
| AC6 Operator smoke test recorded on Debian host | T-0114, T-0116, T-0118 |
