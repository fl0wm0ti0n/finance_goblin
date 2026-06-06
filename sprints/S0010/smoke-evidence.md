# AC-6 Operator Smoke Evidence — Sprint S0010

**Story:** US-0010  
**Recorded:** 2026-06-02 (execute phase)  
**Environment:** CI/dev sandbox + operator omniflow host (pending)

> No credentials committed. Placeholders only.

## Local execute evidence (config-only + build)

| Check | Result | Notes |
|-------|--------|-------|
| `scripts/compose-config-check.sh` | **PASS** | External-only services, anti-combination, greenfield regression, overlay validation |
| `bash tests/run-tests.sh` | **PASS** | Includes compose-config-check integration |
| `docker compose … --profile external build flow-finance-ai` | **PASS** | Image `finance_goblin-flow-finance-ai` built successfully |
| Step 7 (no duplicate Firefly) | **PASS** | `config --services` → `flow-finance-ai`, `grafana` only |

## Eight-step operator checklist (runbook §6)

| Step | Check | Result | Evidence |
|------|-------|--------|----------|
| 1 | TimescaleDB version non-null on `flow_finance_ai` | **PENDING** | Requires omniflow host `postgres` access |
| 2 | `curl http://firefly:8080/api/v1/about` from traefik network → 200 | **PENDING** | Requires host `firefly` on `traefik` network |
| 3 | `FIREFLY_PERSONAL_ACCESS_TOKEN` non-empty in container | **PENDING** | Requires live deploy + operator PAT |
| 4 | `curl http://flow-finance-ai:8080/health` from traefik network → OK | **PENDING** | Requires live deploy + TimescaleDB + PAT |
| 5 | `curl https://financegnome.omniflow.cc/health` with basic-auth → 200 + valid TLS | **PENDING** | Requires omniflow Traefik + `<basic-auth-user>:<pass>` |
| 6 | Same URL without credentials → 401 | **PENDING** | Requires live Traefik `auth` middleware |
| 7 | `docker compose … ps --services` → no `firefly-iii` | **PASS** | Local: `config --services` → flow-finance-ai, grafana |
| 8 | Backend logs — no migration panic | **PENDING** | Requires live deploy against TimescaleDB-enabled DB |

## Auth path notes

- **Auth-off smoke:** operator may set `AUTH_DEV_BYPASS=true` for steps 4–5 API checks; Traefik basic-auth (steps 5–6) is independent.
- **Auth-on production:** register OIDC URIs per `.env.example` and runbook §4.

## Operator post-merge steps (omniflow host)

```bash
# On Debian omniflow host — do NOT commit credentials
cd /path/to/finance_goblin
cp .env.example .env   # set DATABASE_PASSWORD, FIREFLY_PERSONAL_ACCESS_TOKEN, etc.
export COMPOSE_FILE=docker-compose.yml:docker-compose.external.yml
export COMPOSE_PROFILES=external

# Preflight TimescaleDB (step 1)
psql -h postgres -U finance -d flow_finance_ai -c \
  "SELECT extversion FROM pg_extension WHERE extname='timescaledb';"

docker compose --profile external up -d --build

# From traefik network container (steps 2, 4)
docker run --rm --network traefik curlimages/curl:latest \
  -sf http://firefly:8080/api/v1/about
docker run --rm --network traefik curlimages/curl:latest \
  -sf http://flow-finance-ai:8080/health

# Edge TLS + auth (steps 5–6) — use host basic-auth credentials
curl -sfI https://financegnome.omniflow.cc/health -u '<user>:<pass>'
curl -sfI https://financegnome.omniflow.cc/health   # expect 401

docker compose --profile external ps --services   # step 7
docker compose --profile external logs flow-finance-ai | tail -50   # step 8
```

## Blocker

Live omniflow host (Debian + shared `traefik`/`postgres`/`firefly`) not available in execute sandbox. Steps 1–6 and 8 remain **PENDING** for operator post-merge verification (QA: PASS-with-prerequisites).
