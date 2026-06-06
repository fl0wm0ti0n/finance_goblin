# Sprint S0001

**ID:** S0001  
**Story:** US-0001 — Platform foundation & Firefly read-only integration  
**Status:** PLANNED  
**Created:** 2026-05-31

## Goal

Deliver a deployable Flow Finance AI platform foundation: Docker Compose minimal profile, external PostgreSQL mirror, read-only Firefly connector with sync scheduler, OIDC-protected React UI shell, Grafana datasource provisioning, and read-only verification — ready for US-0002 forecasting.

## Scope

- Docker Compose profiles: `minimal`, `standard`, `full`, `oidc` (no embedded PostgreSQL)
- External PostgreSQL/TimescaleDB configuration and startup retry (DEC-0003)
- Rust/Axum backend: health, JWT-protected API skeleton, sync endpoints
- SQLx migrations and relational mirror schema (DEC-0005)
- Firefly GET-only connector for 6 entity types (DEC-0004, DEC-0002)
- Sync scheduler (cron) + manual trigger with overlap guard
- React/shadcn UI shell: Home, Sync Status, Settings; OIDC login (DEC-0001, DEC-0006)
- Grafana datasource + optional Platform Health dashboard
- Read-only integration test and operator user guide (`docs/user-guides/US-0001.md`)

**Out of scope:** Forecasting, subscriptions, analytics Grafana dashboards 1–5, Redis app dependency, hypertables, AI.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Accidental Firefly writes | GET-only client + audit + integration test (T-0012) | DEC-0004 |
| External TimescaleDB missing | Clear migration error; operator docs in T-0012 | R-0004 |
| Large initial sync duration | Progress in Sync Status UI (T-0010) | R-0002 |
| Linux host DB unreachable | `host.docker.internal:host-gateway` in Compose (T-0001) | R-0005 |
| OIDC callback misconfiguration | Document env vars; optional `oidc` profile only | DEC-0001 |

## Definition of Done

- All 12 sprint tasks complete (`T-0001` … `T-0012`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0001
- Minimal Compose profile starts without embedded PostgreSQL
- Integration test proves Firefly GET-only traffic
- User guide published at `docs/user-guides/US-0001.md`

## Architecture references

- `docs/engineering/architecture.md` — US-0001
- Decisions: DEC-0001 … DEC-0006
- Research: R-0001 … R-0005
