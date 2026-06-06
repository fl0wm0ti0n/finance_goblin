# QA Findings — Quick Q0009 / BUG-0003

**Work item:** BUG-0003 (defect)  
**Quick task:** Q0009  
**QA phase:** `/qa`  
**Date:** 2026-06-05  
**Verdict:** **PASS** (ready for `/verify-work`; omniflow operator F1 + deploy deferred)

## Scope

Omniflow production defects per `architecture-20260605-bug0003` (`handoffs/tl_to_dev.md`):

- **F2** — Env guard docs (`.env.example`, runbook mis-host table, compose comment)
- **G1** — `ExchangeService::build_connectors` uses `effective_enabled()` for binance/bybit/bitunix
- **F1** — Operator runbook steps (host `.env` + recreate; runtime at verify-work)
- **G2** — R-0058 futures auth spike (skipped — gated; no pre-deploy auth smoke)

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0009/summary.md`, `sprints/quick/Q0009/plan-verify.json`, `docs/product/acceptance.md` (BUG-0003 rows F/G/H), `docs/engineering/architecture.md` (§ BUG-0003), `backend/src/exchanges/service.rs`, `backend/src/config/mod.rs`, `.env.example` (comments only), `docs/engineering/runbook.md` (§ Omniflow mis-host, F1 steps), `docker-compose.external.yml`, `sprints/quick/Q0009/task.json`, `sprints/quick/Q0009/progress.md`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Backend unit tests | `cd backend && cargo test --lib` | **PASS** (89/89) |
| T-2 | Frontend unit tests | `cd frontend && npm test` | **PASS** (2/2) |
| T-3 | Frontend production build | `cd frontend && npm run build` | **PASS** |
| T-4 | G1 architecture contract | Static review + `build_connectors_registers_bitunix_*` | **PASS** |
| T-5 | F2 architecture contract | Doc review — omniflow block, runbook table, compose `${DATABASE_HOST:-postgres}` | **PASS** |
| T-6 | F1 operator contract | Runbook F1 steps (names only; no committed secrets) | **PASS** (docs) |
| T-7 | G2 skip rationale | `task.json` gate + `progress.md` | **PASS** (gated skip documented) |
| T-8 | Frozen boundaries | Static review — no Traefik/JWT/analytics proxy diff in Q0009 scope | **PASS** |
| T-9 | Rows F/G/H live smoke | Omniflow deploy + operator F1 | **DEFERRED** — verify-work |
| T-10 | Regression footer (OIDC + bundled-firefly) | Operator smoke per acceptance | **DEFERRED** — verify-work (plan-verify ADV-1) |

### Environment dependencies (non-blocking)

- **Operator deploy:** Q0009 backend image to omniflow before live acceptance rows F/G.
- **F1 gates F/H:** `DATABASE_HOST=postgres` in host `.env`; recreate `flow-finance-ai` + `grafana` per runbook F1 steps.
- **G2:** Re-open only if post-F1 smoke shows Bitunix test auth/URL failure with body (not `unknown exchange`).

## Acceptance criteria matrix (BUG-0003)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **(F)** | Representative `GET /api/v1/*` **200** &lt;2s; settings `database_host: postgres`, `database_mode: external` | **PASS** (code/docs) / **DEFERRED** (runtime) | F2: `.env.example` omniflow warnings; runbook symptom table (DEC-0056 / R-0052); compose `${DATABASE_HOST:-postgres}` + comment. F1: runbook steps 1–3. Live GET cascade **DEFERRED** until operator F1 on omniflow |
| **(G)** | `POST …/bitunix/test` not **400** unknown exchange; **200** or documented auth failure | **PASS** (code) / **DEFERRED** (live) | G1: `build_connectors` + unit test; `test_connection` uses registry from connectors. G2 skipped per gate. Live curl **DEFERRED** until deploy |
| **(H)** | `POST …/analytics/grafana/api/ds/query` **200**; Grafana reaches in-network `postgres` | **DEFERRED** | H1 = F1; no separate code slice. Grafana ds/query **DEFERRED** until F1 recreate |
| Regression | OIDC-enabled + bundled-firefly deploy checks | **DEFERRED** | plan-verify ADV-1; verify-work uat.md table |

**Summary:** F2/G1/F1-docs **PASS** on static/automated path; full acceptance runtime deferred with `OPERATOR_F1_PENDING` + `OPERATOR_DEPLOY_PENDING`.

## Architecture compliance

### Sub-defect F — DB host cascade (F2 + F1)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| F2 omniflow block | Warn against `host.docker.internal` on external | `.env.example` lines 7–12, 45–46 | PASS |
| F2 runbook table | Symptom → cause → remediation; cites DEC-0056 / R-0052 | `runbook.md` § Omniflow mis-host | PASS |
| F2 compose | No hardcode; keep `${DATABASE_HOST:-postgres}` | `docker-compose.external.yml` lines 11–12 | PASS |
| F1 operator steps | Names only; recreate app + grafana | runbook F1 steps 1–3 | PASS (docs) |

### Sub-defect G — Bitunix registry (G1; G2 gated)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| `build_connectors` | All three exchanges use `effective_enabled()` | `service.rs` lines 52–60 | PASS |
| `new()` | Delegates to `build_connectors` | `service.rs` lines 37–38 | PASS |
| Unit test | Configured + TOML `enabled=false` → bitunix registered | `build_connectors_registers_bitunix_*` | PASS |
| `test_connection` | Unknown exchange only when not in registry | lines 118–123 | PASS |
| G2 gate | Skip until post-G1+F1 auth smoke | `task.json`, `progress.md` | PASS |

### Frozen boundaries

| Boundary | Verdict |
|----------|---------|
| No hardcode `DATABASE_HOST` in compose | PASS |
| No Traefik/JWT/analytics proxy changes in Q0009 deliverables | PASS |
| No merge with BUG-0002 / Q0008 | PASS |
| G2 deferred unless post-G1+F1 auth failure (not unknown exchange) | PASS |
| H2 UID dedupe out of scope | PASS |

## Generated test evidence

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` + `typescript` |
| `generated_test_command` | `cd backend && cargo test --lib && cd ../frontend && npm test && npm run build` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-06-05 — 89/89 lib tests; vitest 2/2; tsc + vite build exit 0 |
| `generated_test_paths_ref` | `backend/src/exchanges/service.rs` (G1), `backend/src/config/mod.rs` (effective_enabled), `.env.example`, `docs/engineering/runbook.md`, `docker-compose.external.yml` |

## Runtime QA evidence (omniflow)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed (operator-owned deploy + F1) |
| `runtime_stack_profile` | `docker-compose` external profile |
| `runtime_mode` | `deferred` |
| `runtime_health_target` | BUG-0003 rows F/G/H + regression on `https://financegnome.omniflow.cc` |
| `runtime_health_result` | `deferred` |
| `runtime_final_verdict` | `deferred` (verify-work / operator) |
| `runtime_reason_code` | `OPERATOR_F1_PENDING` |
| `runtime_evidence_refs` | `handoffs/dev_to_qa.md`; discovery baseline `architecture.md` § BUG-0003; `sprints/quick/Q0009/uat.md` |

## Findings

### Blockers

None for proceeding to `/verify-work`.

### Advisories (non-blocking)

1. **Deploy Q0009** image to omniflow before verify-work closes live BUG-0003 acceptance.
2. **F1 operator:** Set `DATABASE_HOST=postgres` on host `.env`; force-recreate `flow-finance-ai` + `grafana` per runbook.
3. **G2:** Only if check 6 in uat passes registry but returns auth/URL failure — otherwise remain skipped.
4. **Acceptance checkbox:** `docs/product/acceptance.md` BUG-0003 remains unchecked until verify-work smoke passes.

## Verdict

**PASS** — proceed to `/verify-work` in a fresh subagent/chat. No dev rework required; do not populate `handoffs/qa_to_dev.md`.
