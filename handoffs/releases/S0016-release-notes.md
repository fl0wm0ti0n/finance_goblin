# Sprint Release Notes — S0016

**Sprint:** S0016  
**Date:** 2026-06-06  
**Stories:** US-0015  
**Queue status:** released  
**Orchestrator:** `auto-20260606-us0015-001`  
**Decision:** DEC-0078

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --lib` 169/169; `npm test` 5/5 @ 2026-06-06 release (qa + verify-work evidence)
2. **QA completion gate:** PASS — `sprints/S0016/qa-findings.md` (AC-1–AC-6 verified; 0 blocking findings)
3. **UAT completeness gate:** PASS-with-prerequisites — `sprints/S0016/uat.md`, `sprints/S0016/uat.json`; code/test PASS; runtime omniflow OIDC `/forecast` Monthly smoke deferred pending **BACKEND_FRONTEND_DEPLOY** (S0015/US-0014 precedent)
4. **Isolation compliance gate:** PASS — intake through verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — verify-work tuple `runtime-proof-verify-work-20260606-us0015-s0016-001`; release tuple at finalization
6. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai`
- `runtime_mode`: remote (omniflow external US-0010); backend + frontend delta (AI bucket cascade S1–S3)
- `runtime_context_ref`: `docs/engineering/runbook.md` § 22 US-0015; `docs/user-guides/US-0015.md`

**Profile rule:** use **`external` only** — do not combine with `minimal`, `standard`, `full`, or `bundled-firefly`.

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-7 OIDC smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

After deploy: Full Firefly sync + forecast recompute before Monthly tab smoke.

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (Traefik + basic-auth)
- `service_port`: 443 (HTTPS)
- `health_endpoint`: `GET /health`
- Forecast UI: `/forecast` → **Monthly** tab

## Verify

- `verification_steps`:
  1. Deploy backend + frontend per operator gate above.
  2. Confirm app health: `curl -sf https://financegnome.omniflow.cc/health`.
  3. Full Firefly sync + forecast recompute — `GET /api/v1/forecast/meta` shows `computation_id`.
  4. **AC-1:** Config-mapped buckets never AI-overridden — verify `bucket_sources.income == "config"` for salary month.
  5. **AC-2/AC-3:** AI cascade + privacy — ambiguous recurring rows get AI bucket when confidence ≥0.75; raw payee stripped under default privacy.
  6. **AC-4:** `GET /api/v1/forecast/monthly` exposes `bucket_sources` + `ai_mapped` per bucket.
  7. **AC-5:** Monthly tab **AI-mapped** badge when `ai_mapped=true`; config-only months show no badge.
  8. **AC-6:** `forecast_bucket_assignment` audit rows on recompute; no raw merchant in payload.
  9. **AC-7:** OIDC `/forecast` Monthly smoke per `sprints/S0016/uat.md` steps 1–8; chat six-tool + US-0013 ML overlay regression unchanged.
  10. Automated: `cd backend && cargo test --lib` (169/169); `cd frontend && npm test` (5/5).
- `expected_health_signal`: backend `/health` OK; Monthly tab stat cards render; AI-mapped badge visible when provenance includes AI mass

## Credentials

- `credential_source_refs` (env names only):
  - US-0010 block: `COMPOSE_FILE`, `COMPOSE_PROFILES`, `DATABASE_*`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, `TRAEFIK_HOST`
  - AI provider (when enabled): `OPENAI_API_KEY` or configured provider env per `backend/config/default.toml`
- `expected_value_source`: operator `.env` at repo root (from `.env.example`)

## Known Issues

- Omniflow OIDC `/forecast` Monthly smoke **PENDING** operator post-deploy — AC-7 pass-with-prerequisites at release (`BACKEND_FRONTEND_DEPLOY`).
- Integration tests require operator `DATABASE_URL` (carry-forward).
- Rolling residual remains Variable-only in MVP (DEC-0078).

## Deliverables (US-0015)

| Slice | Tasks | Outcome |
|-------|-------|---------|
| S1 | T-0163..T-0166 | `BucketInferenceService` cascade (rule→LLM→Variable); `prepare_bucket_features` privacy allowlist; threshold tests |
| S2 | T-0167..T-0170 | `resolve_bucket_with_ai` + provenance; config guard never overridden; recurring AI on config-map miss |
| S3 | T-0171..T-0174 | `bucket_sources` + `ai_mapped` API; ForecastPage AI-mapped badge; `forecast_bucket_assignment` audit; user guide + UAT template |

**Key files:** `backend/src/forecast/bucket_inference.rs`, `backend/src/forecast/project.rs`, `backend/src/api/forecast.rs`, `frontend/src/pages/ForecastPage.tsx`, migration `011_forecast_bucket_provenance.sql`

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0016 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.16.0-us0015`

## Milestone

**US-0015 released** — AI-assisted forecast category bucket mapping: config→rule→LLM→Variable cascade, monthly provenance API, AI-mapped badge, audit trail (DEC-0078).
