# Sprint Release Notes — S0017

**Sprint:** S0017  
**Date:** 2026-06-09  
**Stories:** US-0018  
**Queue status:** released  
**Orchestrator:** `auto-20260608-us0018-001`  
**Decisions:** DEC-0087, DEC-0088, DEC-0089, DEC-0090

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --lib` 193/193; `npm test -- --run` 7/7 @ 2026-06-08 (qa + verify-work evidence); `validate_doc_profile --no-template-parity` exit 0 @ release
2. **QA completion gate:** PASS — `sprints/S0017/qa-findings.md` (AC-1–AC-6 verified; 0 blocking findings)
3. **UAT completeness gate:** PASS-with-prerequisites — `sprints/S0017/uat.md`, `sprints/S0017/uat.json`; code/test PASS; runtime omniflow OIDC category-filter smoke deferred pending **BACKEND_FRONTEND_DEPLOY**, **FULL_FIREFLY_SYNC**, **GRAFANA_PROVISIONING_RELOAD** (US-0015/BUG-0013 precedent)
4. **Isolation compliance gate:** PASS — intake through verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — verify-work tuple `runtime-proof-verify-work-20260608-us0018-001`; release tuple at finalization
6. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai`
- `runtime_mode`: remote (omniflow external US-0010); backend + frontend delta (category filters & expense-series API)
- `runtime_context_ref`: `docs/engineering/runbook.md` § 26 US-0018; `docs/user-guides/US-0018.md`

**Profile rule:** use **`external` only** — do not combine with `minimal`, `standard`, `full`, or `bundled-firefly`.

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-6 OIDC smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) so `category_id` mirror is current.

**Operator gate — GRAFANA_PROVISIONING_RELOAD (required before AC-1 Grafana `$category` smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate grafana
```

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (Traefik + basic-auth)
- `service_port`: 443 (HTTPS)
- `health_endpoint`: `GET /health`
- Category surfaces: `/forecast` Monthly tab, `/planning` Compare, `/wealth` Overview, `/analytics/cashflow`, `/analytics/budgets`

## Verify

- `verification_steps`:
  1. Deploy backend + frontend per operator gate above.
  2. Confirm app health: `curl -sf https://financegnome.omniflow.cc/health`.
  3. Full Firefly sync so `category_id` mirror is current.
  4. Reload Grafana provisioning (cashflow + budgets dashboards).
  5. **AC-1:** `/forecast` Monthly — `CategoryFilter` visible; select category; trend chart loads; household cards unchanged.
  6. **AC-2:** `GET /api/v1/categories/expense-series?category_id=<id>` — per-month EUR spine; default 12 months, max 24.
  7. **AC-3:** `CategoryTrendChart` bar chart with month labels + EUR amounts; empty-state when no selection.
  8. **AC-4:** MoM / Highest / Lowest month stat cards from server `summary`.
  9. **AC-5:** `GET ...?category_id=__uncategorized__` returns `uncategorized: true`; full spine with €0 months.
  10. **AC-6:** OIDC 10-step checklist in `sprints/S0017/uat.md` § OIDC smoke checklist; US-0015 AI-mapped badge regression unchanged.
  11. Automated: `cd backend && cargo test --lib` (193/193); `cd frontend && npm test -- --run` (7/7).
- `expected_health_signal`: backend `/health` OK; category filter + trend chart render on Forecast Monthly; Grafana `$category` variable present on cashflow + budgets

## Credentials

- `credential_source_refs` (env names only):
  - US-0010 block: `COMPOSE_FILE`, `COMPOSE_PROFILES`, `DATABASE_*`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, `TRAEFIK_HOST`
- `expected_value_source`: operator `.env` at repo root (from `.env.example`)

## Known Issues

- Omniflow OIDC category-filter smoke **PENDING** operator post-deploy — AC-6 pass-with-prerequisites at release (`BACKEND_FRONTEND_DEPLOY`, `FULL_FIREFLY_SYNC`, `GRAFANA_PROVISIONING_RELOAD`).
- T-0185 EXPLAIN probe / conditional index **deferred** per DEC-0090 — no operator mirror; MVP sequential scan accepted.
- Integration tests require operator `DATABASE_URL` (carry-forward).

## Deliverables (US-0018)

| Slice | Tasks | Outcome |
|-------|-------|---------|
| Backend API | T-0175, T-0176 | `GET /api/v1/categories` catalog; `GET /api/v1/categories/expense-series` month-spine + server `summary`; `__uncategorized__` sentinel (DEC-0087) |
| Frontend SPA | T-0177..T-0180 | Shared `CategoryFilter` + `CategoryTrendChart` on Forecast Monthly, Planning Compare, Wealth Overview (DEC-0088, DEC-0089) |
| Grafana | T-0181, T-0182 | `$category` variable on cashflow + budgets dashboards (DEC-0089) |
| Docs + UAT | T-0183, T-0184 | `docs/user-guides/US-0018.md`; OIDC smoke template |

**Key files:** `backend/src/api/categories.rs`, `backend/src/transactions/repository.rs`, `frontend/src/components/CategoryFilter.tsx`, `frontend/src/components/CategoryTrendChart.tsx`, `grafana/provisioning/dashboards/cashflow.json`, `grafana/provisioning/dashboards/budgets.json`

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0017 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.18.0-us0018`

## Milestone

**US-0018 released** — Category filters & expense trend analytics: shared filter contract, expense-series API, bar trend chart with MoM/best/worst summary, Grafana `$category` on cashflow + budgets (DEC-0087..DEC-0090).
