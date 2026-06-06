# Q0008 — BUG-0002 omniflow production integration defects

| Field | Value |
|-------|-------|
| **ID** | Q0008 |
| **Type** | `/quick` |
| **Status** | PLAN-VERIFIED |
| **Bug** | BUG-0002 |
| **Created** | 2026-06-04 |
| **Architecture** | `architecture-20260604-bug0002` (`docs/engineering/architecture.md` § BUG-0002) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260604-q0008-bug0002`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0002 rows **(C)**, **(D)**, **(E)** |
| **Task count** | 5 (≤ `SPRINT_MAX_TASKS` 12; no split) |
| **Next phase** | `/execute` |

## Goal

Close BUG-0002 on US-0010 external omniflow: non-empty Firefly PAT + empty-PAT guard (C), risk-score **200** tagged empty-state (D), effective exchange `enabled` + greenfield Binance default off (E). **No new DEC** — extends DEC-0004, DEC-0054, R-0032.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| C — Firefly sync | C1, C2 | ops/docs + backend |
| D — Risk-score API | D1 | backend + frontend |
| E — Exchange settings | E1, E2 | backend + config |

**Out of scope:** Traefik/router fixes for sync or risk-score (discovery refuted); new DEC; `GF_SERVER_SERVE_FROM_SUB_PATH`; wallet extension console noise.

## Task summary

| ID | Title | Est. | Depends | Acceptance hook |
|----|-------|------|---------|-----------------|
| C2 | Empty PAT env guard + sync fail-fast | 3h | — | **(C)** code path — no blank Bearer 401 |
| D1 | Risk-score 200 tagged empty-state + Planning types | 4h | — | **(D)** `GET …/risk-score` → 200 JSON |
| E1 | Effective `enabled` in settings_view + startup mirror | 2h | — | **(E)** Bitunix-only → enabled+configured |
| E2 | `default.toml` binance.enabled=false | 0.5h | E1 (same PR) | **(E)** greenfield Binance row |
| C1 | Operator PAT + runbook/compose verification | 1h | C2 deploy | **(C)** operator — sync success, counts > 0 |

**Total estimate:** ~10.5h (dev ~9.5h + operator C1 ~1h).

## Deploy order

1. **Code image** — C2 + D1 + E1 + E2 (single PR recommended; tasks independently mergeable).
2. **Operator C1** — non-empty `FIREFLY_PERSONAL_ACCESS_TOKEN` in container **before** omniflow acceptance row **(C)**.
3. **Smoke** — acceptance rows C, D, E + OIDC/bundled-firefly regression footer.

## Execute order (dev)

```text
C2 ∥ D1 ∥ (E1 + E2)  →  deploy  →  C1 (operator)  →  omniflow smoke (C/D/E + regression)
```

## Acceptance mapping

| `acceptance.md` row | Primary tasks | Verify |
|-------------------|---------------|--------|
| **(C)** Firefly sync completes; no blocking 404 on sync APIs | C1, C2 | `printenv` name non-empty; manual sync; no 401 in status |
| **(D)** `GET /api/v1/plans/risk-score` → **200** (payload or empty-state) | D1 | curl/Rust test + Planning UI no hard error |
| **(E)** Bitunix enabled+configured when only Bitunix env set | E1, E2 | `GET /api/v1/settings` + Settings UI |
| Footer — OIDC + bundled-firefly regression | all (post-deploy) | Operator smoke per BUG-0002 acceptance |

## Risks

| Risk | Mitigation | Tasks |
|------|------------|-------|
| PAT in `.env` not mounted in container | C1 runbook + compose cwd | C1 |
| Empty-state client drift | Tagged union in same PR as API | D1 |
| False auto-enable with creds | Sync still validates keys | E1 |
| C1 without C2 deploy | Acceptance C needs both | Deploy order |

## Definition of Done

- All 5 tasks `done` in `task.json` / `progress.md`
- `/plan-verify` — no gaps vs BUG-0002 rows C, D, E
- `cargo test` / frontend build pass for changed paths
- Operator omniflow smoke recorded (or verify-work handoff)
- BUG-0002 acceptance rows C, D, E checkable

## Frozen boundaries

- Do not return **404** for empty risk-score (D1 contract)
- Do not log PAT values
- Do not enable `GF_SERVER_SERVE_FROM_SUB_PATH`
- Do not change Traefik proxy prefix or JWT stack for analytics

## Architecture references

- `docs/engineering/architecture.md` — BUG-0002
- `handoffs/tl_to_dev.md` — architecture-20260604-bug0002
- `handoffs/po_to_tl.md` — discovery-20260604-bug0002
- `handoffs/intake_evidence/intake-20260604-omniflow-prod-integration.json`
- DEC-0004, DEC-0054; R-0057, R-0032

## Artifacts

- `sprints/quick/Q0008/sprint.md` (this file)
- `sprints/quick/Q0008/sprint.json`, `tasks.md`, `task.json`, `progress.md`
- `sprints/quick/Q0008/uat.md` (PENDING — QA/execute)
