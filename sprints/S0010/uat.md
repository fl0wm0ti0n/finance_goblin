# UAT — Sprint S0010 / US-0010

**Sprint:** S0010  
**Story:** US-0010  
**Phase:** `/verify-work`  
**Date:** 2026-06-02  
**Verdict:** **PASS** (ready for `/release`)  
**Ready for release:** yes (operator runtime prerequisites documented below)

## Inputs

- QA PASS: `sprints/S0010/qa-findings.md`, `sprints/S0010/qa.json`
- Acceptance: `docs/product/acceptance.md` (US-0010)
- Operator guide: `docs/user-guides/US-0010.md`
- Implementation: `docker-compose.yml`, `docker-compose.external.yml`, `.env.example`, `scripts/compose-config-check.sh`, `tests/run-tests.sh`
- Smoke evidence: `sprints/S0010/smoke-evidence.md`
- Architecture: `docs/engineering/architecture.md#US-0010`, `decisions/DEC-0056.md`

## Environment

| Prerequisite | Status in verify-work |
|--------------|----------------------|
| Debian omniflow host (`firefly`, `postgres`, `traefik`) | **Not available** — operator prerequisite |
| TimescaleDB on `flow_finance_ai` | **Not verified** — operator preflight (smoke step 1) |
| Operator `.env` with secrets | **Not present** in workspace — no `.env` read |
| Docker Compose ≥2.24 | **Required** — `!reset` merge validated in CI |
| Traefik basic-auth credentials (host stack) | **Out of scope** — operator-managed |

Per workflow policy (US-0001 pattern): code-level and automated verification **pass**; omniflow host runtime steps recorded as **PASS-with-prerequisites** or **operator-verified** where external infra is required.

## Automated checks

| ID | Check | Command | Result |
|----|-------|---------|--------|
| AUTO-1 | Test harness | `bash tests/run-tests.sh` | **PASS** (verify-work run 2026-06-02, exit 0) |
| AUTO-2 | External-only services | `compose-config-check.sh` external profile | **PASS** — `flow-finance-ai grafana` only |
| AUTO-3 | Anti-combination guard | `minimal+external` merge | **PASS** — no `firefly-iii` or `postgres` |
| AUTO-4 | Greenfield bundled-firefly | `minimal+bundled-firefly` | **PASS** — `firefly-iii` present (regression) |
| AUTO-5 | Merged overlay validation | traefik network, DNS overrides, port reset | **PASS** |
| AUTO-6 | Traefik label interpolation | `TRAEFIK_HOST` / `TRAEFIK_MIDDLEWARE` defaults + override | **PASS** |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | External profile does **not** create `firefly-iii` or `postgres` containers | **PASS** | `compose-config-check.sh`: external-only services; anti-combination guard; smoke step 7 local PASS (`config --services` → `flow-finance-ai`, `grafana`). `docker-compose.external.yml` adds no `postgres`/`firefly-iii` service definitions. |
| UAT-2 | AC-2 | `flow-finance-ai` joins external `traefik` network; reaches `postgres` and `firefly` by Docker DNS | **PASS-with-prerequisites** | Merged config: external `traefik` network, `DATABASE_HOST: postgres`, `FIREFLY_BASE_URL: http://firefly:8080`, host ports `!reset`. Runbook + US-0010 guide. **Operator prerequisite:** in-network curls smoke steps 2, 4 on omniflow host. |
| UAT-3 | AC-3 | Traefik routes `https://financegnome.omniflow.cc` with `auth` middleware and valid TLS | **PASS-with-prerequisites** | `docker-compose.external.yml` router `financegnome`, defaults `${TRAEFIK_HOST:-financegnome.omniflow.cc}` / `${TRAEFIK_MIDDLEWARE:-auth}`, certresolver `myresolver`; override test in compose-config-check. **Operator prerequisite:** HTTPS/TLS/auth curls smoke steps 5–6 on omniflow host. |
| UAT-4 | AC-4 | Backend `/health` returns success when wired to external DB and Firefly PAT is configured | **PASS-with-prerequisites** | `.env.example` documents `DATABASE_PASSWORD`, `FIREFLY_PERSONAL_ACCESS_TOKEN`; base compose healthcheck defined. **Operator prerequisite:** live deploy + TimescaleDB + PAT; smoke steps 3–4, 8 on omniflow host. Optional `AUTH_DEV_BYPASS=true` for API-only smoke. |
| UAT-5 | AC-5 | `.env.example` documents all operator-required variables for external mode | **PASS** | Omniflow block (lines 33–48): `COMPOSE_FILE`, `COMPOSE_PROFILES`, `TRAEFIK_HOST`, `TRAEFIK_MIDDLEWARE`, `GRAFANA_TRAEFIK_HOST`, TimescaleDB bootstrap comments, OIDC production URIs; mirrors US-0010 guide required vs optional table. |
| UAT-6 | AC-6 | Operator smoke test recorded (compose up + health + Traefik route check on Debian host) | **PASS-with-prerequisites** | `sprints/S0010/smoke-evidence.md`: local/CI PASS (harness, compose-config-check, build, step 7); steps 1–6, 8 **operator-verified** pending with copy-paste commands per runbook §6 — acceptable per DEC-0056 / US-0001 policy. |

## AC-6 smoke step detail

| Step | Check | Result | Notes |
|------|-------|--------|-------|
| 1 | TimescaleDB version non-null | **operator-verified** | Requires omniflow host `postgres` access |
| 2 | `curl http://firefly:8080/api/v1/about` from traefik network → 200 | **operator-verified** | Requires live host `firefly` |
| 3 | `FIREFLY_PERSONAL_ACCESS_TOKEN` non-empty in container | **operator-verified** | Requires live deploy + operator PAT |
| 4 | `curl http://flow-finance-ai:8080/health` from traefik network → OK | **operator-verified** | Requires live deploy + TimescaleDB + PAT |
| 5 | `curl https://financegnome.omniflow.cc/health` with basic-auth → 200 + valid TLS | **operator-verified** | Requires omniflow Traefik |
| 6 | Same URL without credentials → 401 | **operator-verified** | Requires live Traefik `auth` middleware |
| 7 | No duplicate `firefly-iii` in project services | **PASS** | Local CI: `config --services` → `flow-finance-ai`, `grafana` |
| 8 | Backend logs — no migration panic | **operator-verified** | Requires live deploy against TimescaleDB-enabled DB |

## Summary

| Metric | Count |
|--------|-------|
| Acceptance criteria passed (code/automated) | 6/6 |
| Full omniflow runtime E2E executed | 0/6 (blocked by missing operator host) |
| Automated checks passed | 6/6 |
| Failed | 0 |

## Operator prerequisites (post-release smoke)

1. TimescaleDB preflight on shared `postgres` for database `flow_finance_ai` (smoke step 1).
2. Copy `.env.example` → `.env`; set `COMPOSE_FILE`, `COMPOSE_PROFILES=external`, `DATABASE_PASSWORD`, `FIREFLY_PERSONAL_ACCESS_TOKEN`.
3. `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build`
4. Verify services: `flow-finance-ai`, `grafana` only (smoke step 7).
5. Run eight-step smoke checklist in runbook §6; record PASS timestamps in `sprints/S0010/smoke-evidence.md`.
6. Optional: OIDC IdP registration for `https://financegnome.omniflow.cc/callback`.

## Findings

### Blockers

None.

### Observations

1. Omniflow host unavailable in verify-work sandbox — structural and automated verification sufficient for release gate (US-0001 pattern).
2. Compose `GRAFANA_TRAEFIK_HOST` unset warning during `config` is cosmetic; gating behavior verified (Traefik disabled by default).
3. Integration tests skip without `DATABASE_URL` — unchanged harness convention; US-0010 is compose/docs-only.

## Next phase

Run `/release` in a fresh subagent (or confirm release artifacts if already published).
