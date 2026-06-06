# QA Findings — Quick Q0008 / BUG-0002

**Work item:** BUG-0002 (defect)  
**Quick task:** Q0008  
**QA phase:** `/qa`  
**Date:** 2026-06-04  
**Verdict:** **PASS** (ready for `/verify-work`; omniflow operator smoke deferred)

## Scope

Omniflow production integration defects per `architecture-20260604-bug0002` (`handoffs/tl_to_dev.md`):

- **C2** — Empty PAT env guard; `pat_configured()`; sync preflight fail-fast
- **C1** — Operator PAT runbook / `.env.example` (docs only; runtime at verify-work)
- **D1** — `GET /api/v1/plans/risk-score` always **200** tagged `ok` | `no_score`
- **E1** — `effective_enabled = configured() || enabled`
- **E2** — `default.toml` `[exchanges.binance] enabled = false`

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0008/summary.md`, `sprints/quick/Q0008/plan-verify.json`, `docs/product/acceptance.md` (BUG-0002 rows C/D/E), `docs/engineering/architecture.md` (§ BUG-0002), `handoffs/tl_to_dev.md`, `backend/src/config/mod.rs`, `backend/src/sync/mod.rs`, `backend/src/api/plans.rs`, `backend/src/health/mod.rs`, `backend/src/exchanges/service.rs`, `backend/config/default.toml`, `frontend/src/lib/api.ts`, `frontend/src/pages/PlanningPage.tsx`, `docs/engineering/runbook.md`, `.env.example` (comments only; no secret values read).

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Backend unit tests | `cd backend && cargo test --lib` | **PASS** (88/88) |
| T-2 | Frontend unit tests | `cd frontend && npm test` | **PASS** (2/2) |
| T-3 | Frontend production build | `cd frontend && npm run build` | **PASS** |
| T-4 | C2 architecture contract | Static review + `firefly_pat_configured_*` tests | **PASS** |
| T-5 | D1 architecture contract | Static review + `risk_score_tests` serialization | **PASS** |
| T-6 | E1/E2 architecture contract | Static review + `effective_enabled` / settings_view tests | **PASS** |
| T-7 | Frozen boundaries | Repo grep (no PAT in error strings beyond stable code; no `SERVE_FROM_SUB_PATH` in Q0008 diff scope) | **PASS** |
| T-8 | C1 operator PAT + sync success | Omniflow deploy + `printenv` / manual sync | **DEFERRED** — verify-work after deploy |
| T-9 | Row D live curl/UI | `curl` risk-score on financegnome.omniflow.cc | **DEFERRED** — requires Q0008 image deploy |
| T-10 | Row E live settings | `GET /api/v1/settings` on omniflow | **DEFERRED** — requires deploy |
| T-11 | Regression footer (OIDC + bundled-firefly) | Operator smoke per acceptance | **DEFERRED** — verify-work (ADV-1) |

### Environment dependencies (non-blocking)

- **Operator deploy:** Q0008 backend + frontend image to omniflow before live acceptance rows C/D/E and regression can close.
- **C1 gates row C:** Non-empty `FIREFLY_PERSONAL_ACCESS_TOKEN` in container; sync success without 401 from blank Bearer.

## Acceptance criteria matrix (BUG-0002)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **(C)** | PAT configured; sync completes; Sync Status entity counts; no blocking sync 404 | **PASS** (code/docs) / **DEFERRED** (runtime) | C2: env overlay skips empty trim; sync preflight `firefly_personal_access_token_missing`; `/health/ready` `firefly_pat_configured`; runbook + `.env.example` C1. Live sync **DEFERRED** until operator PAT on omniflow |
| **(D)** | `GET /api/v1/plans/risk-score` → **200** (ok or documented empty-state) | **PASS** (code) / **DEFERRED** (live) | Handler returns 200 `NoScore` / `Ok`; serde tag tests; `PlanRiskScoreResponse` union; Planning badge only when `status === "ok"`. Live curl **DEFERRED** until deploy |
| **(E)** | Bitunix enabled+configured when only Bitunix env; Binance/Bybit match env | **PASS** (code) / **DEFERRED** (live) | `effective_enabled()` in settings_view + `mirror_enabled_at_startup`; binance `enabled=false` default; unit tests. Live settings **DEFERRED** until deploy |
| Regression | OIDC-enabled + bundled-firefly deploy checks | **DEFERRED** | plan-verify ADV-1; verify-work uat.md table |

**Summary:** 3/3 rows **PASS** on static/automated path; full acceptance runtime deferred with `OPERATOR_DEPLOY_PENDING` (C1 + omniflow smoke).

## Architecture compliance

### Sub-defect C — Firefly PAT guard (C2)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Env overlay | Skip override when PAT trim empty | `config/mod.rs` lines 909–912 | PASS |
| `pat_configured()` | Non-empty trimmed token | `FireflyConfig::pat_configured()` + unit tests | PASS |
| Sync preflight | Fail before Firefly HTTP with stable code | `sync/mod.rs` `execute_run` Full mode | PASS |
| No PAT in logs/responses | Stable message only | Error string cites runbook; no token value | PASS |
| Readiness (optional) | `firefly_pat_configured: bool` | `health/mod.rs` `ReadyResponse` | PASS |

### Sub-defect D — risk-score 200 (D1)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| HTTP status | Always 200 | `risk_score` returns `Ok(Json(...))` or 500 on DB only | PASS |
| Empty state | `status: no_score` + reason | `no_active_plan` / `not_computed` | PASS |
| Populated | `status: ok` + score fields | `RiskScoreApiResponse::Ok` | PASS |
| Frontend | Union type; no hard error on empty | `api.ts` + `PlanningPage` badge gating | PASS |
| No 404 empty | Handler never NOT_FOUND for empty score | 404 path removed | PASS |

### Sub-defect E — exchange settings (E1 + E2)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| `effective_enabled` | `configured() \|\| enabled` | Binance/Bybit/Bitunix impls | PASS |
| `settings_view` | Rows use effective enabled | `ExchangesConfig::settings_view` | PASS |
| Startup mirror | `mirror_enabled_at_startup` uses effective | `exchanges/service.rs` | PASS |
| E2 greenfield | Binance default off | `default.toml` `[exchanges.binance] enabled = false` | PASS |

### Frozen boundaries

| Boundary | Verdict |
|----------|---------|
| No 404 for empty risk-score | PASS |
| No PAT values in logs/responses | PASS |
| No `GF_SERVER_SERVE_FROM_SUB_PATH` in Q0008 scope | PASS |
| No Traefik/JWT analytics changes | PASS |

## Generated test evidence

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` + `typescript` |
| `generated_test_command` | `cd backend && cargo test --lib && cd ../frontend && npm test && npm run build` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-06-04 — 88/88 lib tests; vitest 2/2; tsc + vite build exit 0 |
| `generated_test_paths_ref` | `backend/src/config/mod.rs` (pat/effective_enabled), `backend/src/api/plans.rs` (risk_score_tests), `frontend/src/lib/api.ts`, `frontend/src/pages/PlanningPage.tsx` |

## Runtime QA evidence (omniflow)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed (operator-owned deploy) |
| `runtime_stack_profile` | `docker-compose` external profile |
| `runtime_mode` | `deferred` |
| `runtime_health_target` | BUG-0002 rows C/D/E + regression on `https://financegnome.omniflow.cc` |
| `runtime_health_result` | `deferred` |
| `runtime_final_verdict` | `deferred` (verify-work / operator) |
| `runtime_reason_code` | `OPERATOR_DEPLOY_PENDING` |
| `runtime_evidence_refs` | `handoffs/dev_to_qa.md`; discovery baseline in `architecture.md` § BUG-0002 |

## Findings

### Blockers

None for proceeding to `/verify-work`.

### Advisories (non-blocking)

1. **Deploy Q0008** to omniflow before verify-work closes live BUG-0002 acceptance.
2. **C1 operator:** Confirm non-empty `FIREFLY_PERSONAL_ACCESS_TOKEN` in container; manual sync success (no 401 from blank Bearer).
3. **Sync preflight integration test:** No dedicated `sync::tests` for preflight message — covered by `pat_configured` unit tests + static review; optional follow-up test if desired.
4. **Full `cargo test`:** `firefly_integration` / `firefly_readonly_test` fail on stale `AppConfig` fixtures (pre-existing per dev handoff; not Q0008 regression).

## Verdict

**PASS** — proceed to `/verify-work` in fresh subagent. No dev rework required; do not populate `handoffs/qa_to_dev.md` (no code defects).
