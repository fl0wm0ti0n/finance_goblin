# Sprint S0010

**ID:** S0010  
**Story:** US-0010 — External Firefly/Postgres & Traefik deployment on omniflow host  
**Status:** PLANNED  
**Created:** 2026-06-02

## Goal

Complete the omniflow external deployment contract per **DEC-0056**: `bundled-firefly` profile split, env-parameterized Traefik labels, Grafana internal-only default with optional public gate, `.env.example` alignment, compose CI guards, operator documentation, and recorded AC-6 smoke evidence — attaching Flow Finance AI to existing host `firefly`, `postgres`, and `traefik` without duplicate containers.

## Scope

- `bundled-firefly` profile split — move `firefly-iii` off `minimal` (DEC-0056, R-0053 §2)
- `docker-compose.external.yml` — `${TRAEFIK_HOST}`, `${TRAEFIK_MIDDLEWARE}`, Grafana `ports: !reset []`, optional `${GRAFANA_TRAEFIK_HOST}` router gate
- `.env.example` — omniflow block: `COMPOSE_FILE`, `COMPOSE_PROFILES`, `TRAEFIK_*`, `GRAFANA_TRAEFIK_HOST`, OIDC redirect notes, greenfield `bundled-firefly` start
- `tests/run-tests.sh` — external-only service assert; anti-combination guard (`minimal+external` must not list `firefly-iii`); greenfield `minimal+bundled-firefly` regression
- Merged compose config validation — traefik network, DNS overrides, port `!reset` (AC-2)
- Runbook — greenfield start commands, profile union warnings, Compose ≥2.24, TimescaleDB preflight (R-0053 §1)
- User guides — update `docs/user-guides/US-0001.md` start commands; create `docs/user-guides/US-0010.md`
- Operator AC-6 smoke execution + evidence record (R-0053 §6 runbook table)

**Out of scope:** Host Firefly/Postgres/Traefik stack edits; Traefik ACME/DNS; replacing host `auth` middleware; OIDC IdP provisioning; application feature code; Firefly data migration.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| TimescaleDB absent on shared Postgres | Operator preflight in runbook; migration 001 fail-fast | R-0053 §1 |
| Profile union starts duplicate Firefly | `bundled-firefly` split + CI guard + runbook warning | DEC-0056 |
| Compose `!reset` unsupported | Document Compose ≥2.24 minimum | R-0053 |
| Hardcoded Traefik host in overlay | T-0110 env labels with stable defaults | DEC-0056 |
| Grafana port leak on external merge | T-0111 `ports: !reset []` | DEC-0056, R-0053 §4 |
| Breaking greenfield dev start | US-0001/US-0010 guides show `minimal --profile bundled-firefly` | DEC-0056 |
| OIDC misconfig masked by dev bypass | Smoke template distinguishes auth-off vs auth-on | R-0053 §5 |
| AC-6 requires live Debian host | Operator task T-0118; CI covers compose config only | AC-6 |

## Definition of Done

- All 10 sprint tasks complete (`T-0109` … `T-0118`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0010
- `docker compose … --profile external config --services` → `flow-finance-ai`, `grafana` only
- `minimal+external` merge does **not** include `firefly-iii` or `postgres`
- Traefik labels use `${TRAEFIK_HOST:-financegnome.omniflow.cc}` and `${TRAEFIK_MIDDLEWARE:-auth}`
- Grafana external overlay has no host port publish by default
- `.env.example` documents all operator-required external variables
- Operator AC-6 smoke evidence recorded (eight-step runbook checklist)
- No Firefly write operations introduced; no new `postgres`/`firefly` services in external profile

## Architecture references

- `docs/engineering/architecture.md` — US-0010
- `decisions/DEC-0056.md`
- Research: R-0052, R-0053
- Depends on: US-0001 Compose profiles (released), partial `docker-compose.external.yml`
