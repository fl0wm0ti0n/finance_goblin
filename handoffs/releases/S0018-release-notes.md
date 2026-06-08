# Sprint Release Notes — S0018

**Sprint:** S0018  
**Date:** 2026-06-09  
**Stories:** US-0019  
**Queue status:** released  
**Orchestrator:** `auto-20260608-us0019-001`  
**Decisions:** DEC-0091, DEC-0092, DEC-0093, DEC-0094, DEC-0095, DEC-0096, DEC-0097

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --lib` 204/204; `npm test -- --run` 9/9 @ 2026-06-09 (qa + verify-work evidence); `validate_doc_profile --no-template-parity` exit 0 @ release
2. **QA completion gate:** PASS — `sprints/S0018/qa-findings.md` (AC-1–AC-6 verified; 0 blocking findings)
3. **UAT completeness gate:** PASS-with-prerequisites — `sprints/S0018/uat.md`, `sprints/S0018/uat.json`; code/test PASS; runtime omniflow OIDC goal-plan smoke deferred pending **BACKEND_FRONTEND_DEPLOY**, **FULL_FIREFLY_SYNC** (US-0014/US-0015/US-0018 precedent)
4. **Isolation compliance gate:** PASS — intake through verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — verify-work tuple `runtime-proof-verify-work-20260609-us0019-001`; release tuple at finalization
6. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai`
- `runtime_mode`: remote (omniflow external US-0010); backend + frontend delta (goal plans, goal-stats API, category overlay, savings modal)
- `runtime_context_ref`: `docs/engineering/runbook.md` § 27 US-0019; `docs/user-guides/US-0019.md`

**Profile rule:** use **`external` only** — do not combine with `minimal`, `standard`, `full`, or `bundled-firefly`.

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-6 OIDC smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) so `category_id` mirror and aggregates are current for overlay cap + savings ranking.

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (Traefik + basic-auth)
- `service_port`: 443 (HTTPS)
- `health_endpoint`: `GET /health`
- Goal surfaces: `/planning` Scenarios (goal template + savings modal), Compare (goal-stats strip), PVA (active plan unchanged)

## Verify

- `verification_steps`:
  1. Deploy backend + frontend per operator gate above.
  2. Confirm app health: `curl -sf https://financegnome.omniflow.cc/health`.
  3. Full Firefly sync so category mirror and aggregates are current.
  4. **AC-1:** `/planning` Scenarios — Goal balance template; create plan with target balance + date; plan appears in list.
  5. **AC-2:** Select goal plan; `GoalStatsStrip` shows monthly delta vs baseline, yearly rollup, projected balance at target — not household-wide on detail.
  6. **AC-3:** Add category `remove_outflow` adjustment; recompute; Compare/PVA reflect change after cap clamp.
  7. **AC-4:** Open savings modal; ranked categories with evidence; checkbox select → apply creates adjustment lines; no auto-apply.
  8. **AC-5:** Savings API aggregate-only path; audit log on apply; optional `get_category_savings` AI tool (7-tool registry).
  9. **AC-6:** OIDC 9-step checklist in `sprints/S0018/uat.md` § OIDC smoke checklist; US-0014 template grid + PVA guided card regression unchanged.
  10. Automated: `cd backend && cargo test --lib` (204/204); `cd frontend && npm test -- --run` (9/9).
- `expected_health_signal`: backend `/health` OK; goal plan create + goal-stats strip render on Planning; savings modal checkbox apply path works

## Credentials

- `credential_source_refs` (env names only):
  - US-0010 block: `COMPOSE_FILE`, `COMPOSE_PROFILES`, `DATABASE_*`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, `TRAEFIK_HOST`
- `expected_value_source`: operator `.env` at repo root (from `.env.example`)

## Known Issues

- Omniflow OIDC goal-plan smoke **PENDING** operator post-deploy — AC-6 pass-with-prerequisites at release (`BACKEND_FRONTEND_DEPLOY`, `FULL_FIREFLY_SYNC`).
- Integration tests require operator `DATABASE_URL` (TimescaleDB-enabled PostgreSQL) (carry-forward).

## Deliverables (US-0019)

| Slice | Tasks | Outcome |
|-------|-------|---------|
| Goal schema + create API | T-0186, T-0187 | Migration `013_goal_balance.sql`; `goal_balance` template; `target_balance_eur`, `target_date`, `goal_account_id` (DEC-0091, DEC-0095) |
| Goal-stats service + UI | T-0188, T-0189 | `GET /api/v1/plans/{id}/goal-stats`; `GoalStatsStrip` on Scenarios + Compare (DEC-0092, DEC-0096) |
| Category overlay cap + account fork | T-0190, T-0191 | `remove_outflow` 3-month avg cap; goal account projection fork (DEC-0093) |
| Savings suggestions + modal | T-0192, T-0193 | `GET …/category-savings-suggestions`; `CategorySavingsModal` checkbox apply (DEC-0094) |
| Docs + regression tests | T-0194, T-0195 | `docs/user-guides/US-0019.md`; US-0014 regression checklist |
| AI tool (optional) | T-0196 | `get_category_savings` seventh tool; aggregate-only path (DEC-0097) |
| UAT template | T-0197 | OIDC 9-step smoke checklist |

**Key files:** `backend/migrations/013_goal_balance.sql`, `backend/src/plan/goal_stats.rs`, `backend/src/plan/overlay.rs`, `backend/src/plan/savings_service.rs`, `frontend/src/components/planning/GoalStatsStrip.tsx`, `frontend/src/components/planning/CategorySavingsModal.tsx`

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0018 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.19.0-us0019`

## Milestone

**US-0019 released** — Goal-driven planning: target-balance plans with per-plan stats strip, category overlay cap, operator-confirmed savings suggestions, optional AI category tool (DEC-0091..DEC-0097).
