# QA Findings — Sprint S0010 / US-0010

**Sprint:** S0010  
**Story:** US-0010  
**QA phase:** `/qa`  
**Date:** 2026-06-02  
**Verdict:** **PASS** (ready for `/verify-work`; host smoke advisories)

## Scope

External Firefly/Postgres + Traefik deployment on omniflow host: `bundled-firefly` profile split, `docker-compose.external.yml` overlay (Traefik labels, `!reset` ports, DNS overrides), `.env.example` omniflow block, `scripts/compose-config-check.sh` in CI harness, operator runbook and user guides, smoke evidence (local PASS + omniflow host PENDING).

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0010/summary.md`, `sprints/S0010/smoke-evidence.md`, `sprints/S0010/plan-verify.json`, `docs/product/acceptance.md` (US-0010), `docker-compose.yml`, `docker-compose.external.yml`, `.env.example`, `scripts/compose-config-check.sh`, `tests/run-tests.sh`, `docs/engineering/runbook.md` (Omniflow section), `docs/user-guides/US-0010.md`, `docs/user-guides/US-0001.md`, `decisions/DEC-0056.md`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Automated harness | `bash tests/run-tests.sh` (QA run 2026-06-02) | **PASS** (exit 0, `All tests passed`) |
| T-2 | Compose config regression | `scripts/compose-config-check.sh` (via harness) | **PASS** — external-only services, anti-combination, greenfield regression, Traefik defaults/override, overlay network/DNS/ports, Grafana router gate |
| T-3 | Firefly GET-only unit test | `cargo test --test firefly_readonly` | **PASS** (1/1) |
| T-4 | Backend unit tests | `cargo test --lib` | **PASS** (67/67) |
| T-5 | US-0009 ML integration (carry-forward) | `cargo test --test forecast_ml_integration` | **PASS** (3/3) |
| T-6 | AI frozen modules | `cargo test --test ai_frozen_modules` | **PASS** (2/2) |
| T-7 | Frontend build | `npm run build` (via harness) | **PASS** |
| T-8 | Integration suites | firefly/forecast/subscriptions/plans/wealth_alerts/ai_assistant/exchanges_portfolio | **SKIP** — `DATABASE_URL` not set (S0001–S0009 pattern; US-0010 is compose/docs-only) |
| T-9 | External profile image build | Not re-run in QA (dev handoff PASS) | **Accepted** — dev evidence + config checks sufficient for infra story |
| T-10 | Omniflow host smoke (AC-6 steps 1–6, 8) | Not executable in QA sandbox | **PENDING** — documented in `sprints/S0010/smoke-evidence.md`; close at verify-work / operator post-merge |
| T-11 | Operator guide | Static review `docs/user-guides/US-0010.md` | **PASS** — profile rule, env table, deploy, edge auth, eight-step UAT cross-ref |
| T-12 | `.env.example` omniflow block | Static review | **PASS** — `COMPOSE_FILE`, `COMPOSE_PROFILES`, Traefik vars, Grafana gate, TimescaleDB bootstrap comments, OIDC production URIs |

### Environment dependencies (non-blocking)

- **Omniflow Debian host:** Required for live TLS, Traefik basic-auth, in-network Firefly/health curls, TimescaleDB preflight, migration log check (smoke steps 1–6, 8).
- **`DATABASE_URL`:** Not required for US-0010 QA scope; integration suites skipped per harness convention.

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | External profile does not create `firefly-iii` or `postgres` | **PASS** | `compose-config-check.sh`: external-only `flow-finance-ai grafana`; anti-combination; no forbidden services in merged config; smoke step 7 local PASS (`config --services`) |
| AC-2 | `flow-finance-ai` joins `traefik` network; reaches `postgres` and `firefly` by DNS | **PASS** (static/CI) | Merged config asserts external `traefik` network, `DATABASE_HOST: postgres`, `FIREFLY_BASE_URL: http://firefly:8080`, host ports `!reset`; runbook + US-0010 guide. Live in-network curls **PENDING** host (smoke steps 2, 4) |
| AC-3 | Traefik routes `https://financegnome.omniflow.cc` with `auth` middleware and valid TLS | **PASS** (static) | `docker-compose.external.yml` router `financegnome`, defaults `${TRAEFIK_HOST}` / `${TRAEFIK_MIDDLEWARE:-auth}`, certresolver `myresolver`; override test in compose-config-check. HTTPS/TLS/auth curls **PENDING** host (smoke steps 5–6) |
| AC-4 | Backend `/health` success with external DB + Firefly PAT | **PASS** (static/docs) | `.env.example` + operator guide document `DATABASE_PASSWORD`, `FIREFLY_PERSONAL_ACCESS_TOKEN`; healthcheck in base compose. Live `/health` **PENDING** host (smoke steps 3–4, 8) |
| AC-5 | `.env.example` documents operator-required external variables | **PASS** | Omniflow block lines 33–48; mirrors US-0010 guide required vs optional table |
| AC-6 | Operator smoke recorded on Debian host | **PASS-with-prerequisites** | `sprints/S0010/smoke-evidence.md`: local/CI PASS (config, harness, build, step 7); steps 1–6, 8 PENDING with operator commands and blocker note — acceptable per DEC-0056 / execute handoff |

**Summary:** 6/6 PASS on static/CI path; runtime omniflow smoke deferred with documented operator checklist (not a QA blocker).

## Generated baseline test evidence

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `bash` + `docker-compose` + `rust` + `typescript` |
| `generated_test_command` | `bash tests/run-tests.sh` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-06-02 — exit 0, message `All tests passed`; compose-config-check all checks passed |
| `generated_test_paths_ref` | `scripts/compose-config-check.sh`, `tests/run-tests.sh`, `docker-compose.yml`, `docker-compose.external.yml` |
| `generated_test_reason_code` | — |

## Runtime QA evidence

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed (`docker compose … up` on omniflow host) |
| `runtime_stack_profile` | `docker-compose` external merge |
| `runtime_mode` | `deferred` |
| `runtime_health_target` | Omniflow eight-step smoke (TimescaleDB, Firefly about, PAT, in-network/public `/health`, Traefik TLS/auth, migration logs) |
| `runtime_health_result` | `deferred` |
| `runtime_final_verdict` | `deferred` (verify-work / operator) |
| `runtime_reason_code` | `OMNIFLOW_HOST_UNAVAILABLE` |
| `runtime_evidence_refs` | `sprints/S0010/smoke-evidence.md`, `docs/user-guides/US-0010.md`, `handoffs/dev_to_qa.md` |

## Findings

### Blockers

None.

### Advisories (non-blocking)

1. **Omniflow host smoke PENDING:** Steps 1–6 and 8 in `smoke-evidence.md` require Debian host with shared `traefik`/`postgres`/`firefly`. Operator post-merge checklist is complete and copy-paste ready.
2. **Compose GRAFANA_TRAEFIK_HOST warning:** Blank `GRAFANA_TRAEFIK_HOST` emits Compose “variable is not set” warnings during `config` — cosmetic; gating behavior verified (Traefik disabled by default).
3. **Integration tests skipped:** No `DATABASE_URL` in QA env — unchanged from prior sprints; US-0010 does not add backend feature code.
4. **External image build not re-run in QA:** Relied on dev handoff + config regression; acceptable for compose-contract story.

## Decision gates

No critical issues requiring dev rework. Proceed to **`/verify-work`**.

## Next phase

Run `/verify-work` in a fresh subagent. Close omniflow smoke steps 1–6 and 8 on host when available; optional `AUTH_DEV_BYPASS=true` for API checks per smoke-evidence.md.
