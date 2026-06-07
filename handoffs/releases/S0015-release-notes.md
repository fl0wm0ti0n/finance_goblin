# Sprint Release Notes — S0015

**Sprint:** S0015  
**Date:** 2026-06-08  
**Stories:** US-0014  
**Queue status:** released  
**Orchestrator:** `auto-20260608-us0014-001`  
**Decision:** DEC-0077

---

## Gate results

1. **Check-in test gate:** PASS — `npm test` 5/5; `cargo test --test plans_integration` 5/5 @ 2026-06-08 release
2. **QA completion gate:** PASS — `sprints/S0015/qa-findings.md` (AC-1–AC-8 verified; 0 blocking findings)
3. **UAT completeness gate:** PASS-with-prerequisites — `sprints/S0015/uat.md`, `sprints/S0015/uat.json`; code/test PASS; runtime omniflow OIDC smoke deferred pending **BACKEND_FRONTEND_DEPLOY** (Q0019/S0010 precedent)
4. **Isolation compliance gate:** PASS — intake through verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — qa tuple `runtime-proof-qa-20260608-us0014-s0015-001`; release tuple at finalization
6. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai`
- `runtime_mode`: remote (omniflow external US-0010); frontend-only delta (planning UX polish)
- `runtime_context_ref`: `docs/engineering/runbook.md` § 21 US-0014; `docs/user-guides/US-0014.md`

**Profile rule:** use **`external` only** — do not combine with `minimal`, `standard`, `full`, or `bundled-firefly`.

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-8 OIDC smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (Traefik + basic-auth)
- `health_endpoint`: `GET /health`
- Planning UI: `/planning` (Scenarios | Compare | Plan vs Actual)

## Verify

- `verification_steps`:
  1. Deploy frontend per operator gate above.
  2. Confirm app health: `curl -sf https://financegnome.omniflow.cc/health`.
  3. **AC-1:** Empty state — template card grid (6 templates), plan name field, **Create empty plan** CTA; inline add form visible.
  4. **AC-2/AC-5:** Create empty / from template — green success confirmation; add adjustment updates Compare/PVA after recompute.
  5. **AC-3:** Compare tab — zero-adjustment plan shows **0.00** monthly delta; overlay-only footnote present.
  6. **AC-4:** Plan vs Actual — guided card with Set active / Scenarios when no active plan (HTTP 200 `no_active_plan`).
  7. **AC-6:** Yellow set-active banner mentions Plan vs Actual + Grafana Dashboard 3 (Budgets).
  8. **AC-7:** Force mutation failure — red error card with Dismiss (not console-only).
  9. **AC-8:** OIDC three-tab smoke per `sprints/S0015/uat.md`.
  10. Automated: `cd frontend && npm test` (5/5); `cd backend && cargo test --test plans_integration` (5/5).
- `expected_health_signal`: backend `/health` OK; planning mutations surface visible success/error feedback per DEC-0077

## Credentials

- `credential_source_refs` (env names only):
  - US-0010 block: `COMPOSE_FILE`, `COMPOSE_PROFILES`, `DATABASE_*`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, `TRAEFIK_HOST`
- `expected_value_source`: operator `.env` at repo root (from `.env.example`)

## Known Issues

- Omniflow OIDC smoke **PENDING** operator post-deploy — AC-8 pass-with-prerequisites at release (`BACKEND_FRONTEND_DEPLOY`).
- Integration tests require operator `DATABASE_URL` (carry-forward).
- US-0013 omniflow ML smoke still pending **BACKEND_COMPOSE_DEPLOY** (carry-forward).

## Deliverables (US-0014)

| Slice | Tasks | Outcome |
|-------|-------|---------|
| S2 | T-0158, T-0159, T-0160 | `planningFeedback.tsx` helper; `onError` on all 7 mutations; success toasts + plan-vs-actual invalidation |
| S1 | T-0155, T-0156, T-0157 | Empty-state verify; set-active banner Dashboard 3 copy; create/template success confirmations |
| S3 | T-0161, T-0162 | Compare/PVA verify-only; user guide finalized; UAT OIDC template |

**Files changed:** `frontend/src/pages/planningFeedback.tsx`, `frontend/src/pages/planningFeedback.test.ts`, `frontend/src/pages/PlanningPage.tsx`, `docs/user-guides/US-0014.md`

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0015 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.15.0-us0014`

## Milestone

**US-0014 released** — planning mode intuitive UX completion: mutation feedback, onboarding polish, operator-visible errors (DEC-0077).
