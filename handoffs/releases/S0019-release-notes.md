# Sprint Release Notes — S0019

**Sprint:** S0019  
**Date:** 2026-06-10  
**Stories:** US-0020  
**Queue status:** released  
**Orchestrator:** `auto-20260608-us0020-001`  
**Decisions:** DEC-0098, DEC-0099, DEC-0100, DEC-0101, DEC-0102, DEC-0103

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --lib` 213/213; `npm test -- --run` 9/9 @ 2026-06-10 (qa + verify-work evidence); `validate_doc_profile --no-template-parity` exit 0 @ release
2. **QA completion gate:** PASS — `sprints/S0019/qa-findings.md` (AC-1–AC-6 verified; 0 blocking findings)
3. **UAT completeness gate:** PASS-with-prerequisites — `sprints/S0019/uat.md`, `sprints/S0019/uat.json`; code/test PASS; runtime omniflow OIDC discover/tag smoke deferred pending **BACKEND_FRONTEND_DEPLOY**, **FULL_FIREFLY_SYNC** (US-0018/US-0019 precedent)
4. **Isolation compliance gate:** PASS — intake through verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — verify-work tuple `runtime-proof-verify-work-20260610-us0020-001`; release tuple at finalization
6. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai`
- `runtime_mode`: remote (omniflow external US-0010); backend + frontend delta (discover API/UI, majority category, operator tags)
- `runtime_context_ref`: `docs/engineering/runbook.md` § 28 US-0020; `docs/user-guides/US-0020.md`

**Profile rule:** use **`external` only** — do not combine with `minimal`, `standard`, `full`, or `bundled-firefly`.

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-6 OIDC smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) so mirror transactions and `category_id` are current for discover search + majority category.

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (Traefik + basic-auth)
- `service_port`: 443 (HTTPS)
- `health_endpoint`: `GET /health`
- Subscription surfaces: `/subscriptions` Discover tab, All/Pending/Standing tabs, tag manager + filter chips

## Verify

- `verification_steps`:
  1. Deploy backend + frontend per operator gate above.
  2. Confirm app health: `curl -sf https://financegnome.omniflow.cc/health`.
  3. Full Firefly sync so mirror transactions and categories are current.
  4. **AC-1:** `/subscriptions` Discover — filter by account, payee substring, interval; results capped at 50.
  5. **AC-2:** Confirm discover candidate → appears in All/confirmed list (not pending-only path); DEC-0085 merge on duplicate payee+interval.
  6. **AC-3:** Confirmed row shows majority display category badge; tooltip documents RANK tie-break (latest-date).
  7. **AC-4:** Create tag (e.g. `luxus`); assign multiple tags; filter All tab by tag chip.
  8. **AC-5:** Tags + `display_category_id` in product DB only; no Firefly write-back during smoke.
  9. **AC-6:** OIDC 8-step checklist in `sprints/S0019/uat.md` § OIDC smoke checklist; US-0003 pending confirm/reject + US-0008 alert dedup regression unchanged.
  10. Automated: `cd backend && cargo test --lib` (213/213); `cd frontend && npm test -- --run` (9/9).
- `expected_health_signal`: backend `/health` OK; Discover tab search + confirm path works; tag CRUD + filter chips render on All tab

## Credentials

- `credential_source_refs` (env names only):
  - US-0010 block: `COMPOSE_FILE`, `COMPOSE_PROFILES`, `DATABASE_*`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, `TRAEFIK_HOST`
- `expected_value_source`: operator `.env` at repo root (from `.env.example`)

## Known Issues

- Omniflow OIDC discover/tag smoke **PENDING** operator post-deploy — AC-6 pass-with-prerequisites at release (`BACKEND_FRONTEND_DEPLOY`, `FULL_FIREFLY_SYNC`).
- Integration tests require operator `DATABASE_URL` (TimescaleDB-enabled PostgreSQL) (carry-forward).

## Deliverables (US-0020)

| Slice | Tasks | Outcome |
|-------|-------|---------|
| Migration + types | T-0198 | Migration `014_us0020_display_category_tags.sql`; `display_category_id`, `operator_tags`, `subscription_pattern_tags` |
| Discover service + API | T-0199 | `GET /api/v1/subscriptions/discover`; account + payee + interval; cap 50 (DEC-0098) |
| Discover tab UI | T-0200 | SubscriptionsPage Discover tab with search form + candidate table |
| Confirm from discover + merge | T-0201 | `POST …/discover/confirm`; direct confirmed insert; DEC-0085 merge; no `new_detection` alert (DEC-0099) |
| Majority category compute | T-0202 | RANK majority on confirm + merge; `display_category_id` persisted (DEC-0100) |
| Majority badge + tooltip | T-0203 | Confirmed row badge + tie-break tooltip UI |
| Tag CRUD API | T-0204 | `operator_tags` CRUD; PATCH rename; slug uniqueness (DEC-0101) |
| Tag assign + filter | T-0205 | `PUT …/tags` replace set; list `?tag=` slug filter (DEC-0102) |
| Tag manager + filter chips | T-0206 | Tag manager UI + filter chips on All tab |
| User guide | T-0207 | `docs/user-guides/US-0020.md` |
| Regression tests | T-0208 | US-0003/US-0008 regression audit tests |
| UAT template | T-0209 | OIDC 8-step smoke checklist |
| Grafana `$tag` (P2) | T-0210 | Grafana subscriptions dashboard `$tag` variable (DEC-0103) |

**Key files:** `backend/migrations/014_us0020_display_category_tags.sql`, `backend/src/subscriptions/discovery.rs`, `backend/src/subscriptions/subscription_tags.rs`, `frontend/src/pages/SubscriptionsPage.tsx`, `grafana/provisioning/dashboards/analytics/subscriptions.json`

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0019 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.20.0-us0020`

## Milestone

**US-0020 released** — Subscription manual discovery: operator search/confirm path, majority display category with RANK tie-break, operator-defined tags with CRUD/assign/filter, optional Grafana `$tag` (DEC-0098..DEC-0103). **Last story in intake bundle** — backlog drain complete.
