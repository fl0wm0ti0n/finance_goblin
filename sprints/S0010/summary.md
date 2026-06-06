# Sprint S0010 Summary — US-0010 omniflow external deploy

**Status:** Released (`0.10.0-us0010`, 2026-06-02)  
**Tasks completed:** 10/10 (T-0109 … T-0118)

## Deliverables

| Task | Deliverable |
|------|-------------|
| T-0109 | `bundled-firefly` profile split in `docker-compose.yml` |
| T-0110 | Env-parameterized Traefik labels in `docker-compose.external.yml` |
| T-0111 | Grafana `ports: !reset []`, traefik network, optional public router gate |
| T-0112 | `.env.example` omniflow block (COMPOSE_FILE, TRAEFIK_*, OIDC notes) |
| T-0113 | `scripts/compose-config-check.sh` wired into `tests/run-tests.sh` |
| T-0114 | Runbook §7 cross-ref update |
| T-0115 | `docs/user-guides/US-0001.md` start command update |
| T-0116 | `docs/user-guides/US-0010.md` operator guide |
| T-0117 | External overlay merged config validation in compose-config-check |
| T-0118 | `sprints/S0010/smoke-evidence.md` — local PASS + host PENDING |

## Frozen boundaries preserved

- No `postgres` or host `firefly` services in external merge
- Traefik router id `financegnome` unchanged
- Firefly read-only guarantee (DEC-0004) — no app code changes

## Test results

- `bash tests/run-tests.sh` — PASS
- `scripts/compose-config-check.sh` — PASS (external-only, anti-combination, overlay validation)
- `docker compose … --profile external build flow-finance-ai` — PASS

## Smoke status

- **Local/CI:** PASS (config regression, build, step 7 service list)
- **Omniflow host:** PENDING steps 1–6, 8 — operator post-merge per smoke-evidence.md

## Gate summary

- Execute: complete
- QA: PASS (2026-06-02)
- Verify-work: PASS (2026-06-02)
- Release: `0.10.0-us0010`; publish skipped (`RELEASE_PUBLISH_MODE=disabled`)
- Refresh-context: complete (2026-06-02T24:00:00Z)
- Operator follow-up: omniflow host smoke steps 1–6, 8 per `smoke-evidence.md`
