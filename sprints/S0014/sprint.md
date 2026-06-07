# Sprint S0014

**ID:** S0014  
**Story:** US-0013 — Production ML forecast & wealth analytics hardening  
**Status:** PLANNED  
**Created:** 2026-06-08  
**Orchestrator:** `auto-20260608-us0013-001`

## Goal

Deliver **DEC-0076** external-profile ML enablement on omniflow: overlay `stats-forecast` via additive `profiles: [external]` on the existing service (traefik network, host port 8091), passthrough `FORECAST_ML_ENABLED` / `STATS_FORECAST_URL` on `flow-finance-ai` (DEC-0049 default-off preserved), dual CI guard (compose-config-check + `forecast_ml_integration`), verify-first US-0009 sync/API/React/Grafana paths post-enablement, and operator runbook for ML enablement.

## Scope

| Slice | Tasks | Primary files |
|-------|-------|---------------|
| **US-0013-S1** — Compose + env + CI assert | T-0144 … T-0147 | `docker-compose.external.yml`, `.env.example`, `scripts/compose-config-check.sh` |
| **US-0013-S2** — Sync + API verify | T-0148, T-0149 | `backend/src/sync/mod.rs`, `backend/src/forecast_ml/`, `backend/src/api/forecast.rs` |
| **US-0013-S3** — UI + Grafana verify | T-0150 … T-0152 | `ForecastPage.tsx`, `WealthPage.tsx`, `forecast-horizons.json` |
| **US-0013-S4** — Runbook + CI fixture | T-0153, T-0154 | `docs/engineering/runbook.md`, `backend/tests/forecast_ml_integration.rs`, `tests/run-tests.sh` |

**Out of scope:** New ML models (R-0043 ladder); monthly bucket attribution (BUG-0012 / US-0015); Grafana empty-state-only (BUG-0009 DONE — banner remains when ML off); backend/React algorithm changes beyond verify gaps.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Cold-start race (first sync skips ML) | Runbook: re-sync after `/health` OK; DEC-0052 skip acceptable | R-0071, T-0153 |
| CI drift (compose-check not updated with overlay) | Atomic S1 PR: overlay + compose-config-check together | DEC-0076, T-0147 |
| Host memory (sidecar RSS on shared omniflow) | Runbook monitoring note; R-0044 footprint bounded | T-0153 |
| `minimal+external` profile regression | DEC-0056 guard unchanged in compose-config-check | T-0147 |
| Short mirror history | `insufficient_history` skip; Full sync prerequisite in runbook | T-0153 |
| FX incomplete crypto portfolio | `portfolio_forecast_low_confidence` banner — not block | R-0034, T-0151 |

## Definition of Done

- All 11 sprint tasks complete (`T-0144` … `T-0154`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` § US-0013 (9 open AC + prerequisite checked)
- `scripts/compose-config-check.sh` passes with 3-service external set including `stats-forecast`
- `cargo test --test forecast_ml_integration` passes (wiremock sidecar path)
- Operator runbook § Omniflow ML enablement documents compose union, env vars, health probe, min history, degraded troubleshooting
- Operator gate **BACKEND_COMPOSE_DEPLOY** satisfied before omniflow verify-work smoke (documented in UAT)

## Architecture references

- `docs/engineering/architecture.md` § US-0013
- `decisions/DEC-0076.md`
- Research: R-0071; addenda R-0043, R-0044, R-0045, R-0053, R-0062
- Spec-pack: `docs/engineering/spec-pack/US-0013-{design-concept,crs,technical-specification}.md`
- User guide: `docs/user-guides/US-0013.md`
- Discovery: `handoffs/po_to_tl.md#discovery-20260608-us0013`
- Acceptance: `docs/product/acceptance.md` § US-0013
- Handoff: `handoffs/tl_to_dev.md` (`sprint-plan-20260608-s0014-us0013`)

## Sequencing (frozen)

```text
S1: T-0144 → T-0145 → T-0146 → T-0147
S2: T-0148 → T-0149 (after S1 — sidecar must start)
S3: T-0150, T-0151, T-0152 (after S2 — data prerequisite; may parallelize)
S4: T-0153, T-0154 (T-0154 after T-0147; T-0153 may parallel S3 after S1)
Operator: BACKEND_COMPOSE_DEPLOY → Full sync → verify-work omniflow smoke (UAT)
```

## Split decision

- **Why 11 tasks:** Maps architecture slices US-0013-S1..S4 (~4+2+3+2); within `SPRINT_MAX_TASKS=12`.
- **Why not split S0014a/b:** Single epic with S1→S2→S3 dependency chain; splitting would leave omniflow ML path incomplete between sprints.
- **USER_GUIDE_MODE=1:** `docs/user-guides/US-0013.md` created at architecture; T-0153 cross-links runbook.

## Next phase

`/plan-verify` in fresh subagent/chat
