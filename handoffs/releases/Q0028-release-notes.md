# Quick Release Notes — Q0028 / BUG-0020

**Quick task:** Q0028  
**Bug:** BUG-0020 — Subscriptions list quality (duplicates, uncategorized)  
**Date:** 2026-06-11  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0020 rows **BI**, **BJ**; All-tab UI deploy operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --test bug0020_subscription_list_quality` (7/7); regression `bug0008_subscription_alerts` (8/8); `subscriptions_integration` (1/1); `sprints/quick/Q0028/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/quick/Q0028/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS-WITH-PREREQUISITES — `sprints/quick/Q0028/uat.json`, `sprints/quick/Q0028/uat.md`, `handoffs/verify_work_to_release.md`; 11 steps — 9 pass, 2 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260610-bug0020-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Subscription list data-quality fix per **DEC-0109** — one-time migration 016 reconcile (YouTube confirmed merge, Strom pending collapse), confirmed `display_category_id` backfill (DEC-0100 RANK), All-tab default excludes `rejected`/`inactive`, and forward pending guard in detection. Restores BI dedup contract and BJ category display for legacy confirms.

| Scope | Fix |
|-------|-----|
| **DA1** | `backend/migrations/016_bug0020_subscription_list_quality.sql` — YouTube merge + Strom pending collapse |
| **DB1** | Migration 016 — confirmed `display_category_id` backfill (6/6 non-null post-apply) |
| **DA2** | `frontend/src/pages/SubscriptionsPage.tsx` — All tab `pending`+`confirmed` only |
| **DA3** | `backend/src/subscriptions/detection.rs` — skip pending when confirmed/rejected match exists |
| **T1/G1** | `bug0020_subscription_list_quality` 7/7; regression suites PASS |
| **V1** | verify-work BI/BJ oracles — pass; container rebuild blocked TS6133 |

**Code proof:** bug0020 7/7; bug0008 8/8; subscriptions_integration 1/1; BI-API 6 confirmed / 0 dup payee_key; BJ R-0090 oracle 6/6.

**Operator post-release:** Fix `ForecastPage.tsx` TS6133, rebuild `flow-finance-ai`, register migration 016 in `_sqlx_migrations`, re-run BI-ALL browser smoke.

---

## Run

**Target service:** `flow-finance-ai` (backend migration + detection guard + frontend All-tab filter).

**Prerequisite — fix docker build blocker (required before image build):**

```bash
# Remove unused `hasForecast` in frontend/src/pages/ForecastPage.tsx (TS6133)
cd frontend && npm run build   # must exit 0 before docker build
```

**Deploy (backend + frontend):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

- `start_command`: docker compose commands above (after ForecastPage TS6133 fix)
- `runtime_mode`: local (`:18080`) and remote (omniflow external US-0010)
- `runtime_context_ref`: `docs/engineering/runbook.md` (§33 BUG-0020 hotfix)

**Operator gate — MIGRATION_016_APPLY (required before BI/BJ runtime probes):**

Migration 016 was applied manually via psql at verify-work (2026-06-11). For clean environments:

```bash
cd backend && sqlx migrate run
# If migration 15 checksum conflict: resolve _sqlx_migrations row 15, then re-run
```

Confirm: 6 confirmed rows, 6/6 `display_category_id` non-null; ≤1 YouTube confirmed; Strom pending collapsed.

**Operator gate — FULL_FIREFLY_SYNC (required for detection regression):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
# Confirm no new duplicate confirmed YouTube after sync (REG-DETECT)
```

---

## Connect

- `service_url`: `http://localhost:18080` (local) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 18080 (local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET http://localhost:18080/health` → JSON 200
- Subscriptions UI: `http://localhost:18080/subscriptions`
- BI API oracle: `GET /api/v1/subscriptions?status=confirmed`
- Discover regression: `GET /api/v1/subscriptions/discover`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | External PostgreSQL with TimescaleDB — migration 016 target |
| OIDC provider config | Omniflow OIDC-1 regression — Compose/env only |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(BI)** | `GET /api/v1/subscriptions?status=confirmed` | ≤1 row per `payee_key`; 1 YouTube survivor |
| **(BI)** | `/subscriptions` **All** tab | No triplicate Strom / duplicate YouTube visible rows (post-deploy DA2 filter) |
| **(BJ)** | netflix, kindle → `display_category_id` | `18` |
| **(BJ)** | youtube survivor → `display_category_id` | `66` |
| **(BJ)** | hgp patterns → `display_category_id` | `56` |
| **(BJ)** | florian gabriel → `display_category_id` | `3` |
| Regression | discover, tags, detection | HTTP 200; no new YouTube dup post-sync |

**Automated (release):**

```bash
cd backend && cargo test --test bug0020_subscription_list_quality
cd backend && cargo test --test bug0008_subscription_alerts
cd backend && cargo test --test subscriptions_integration
```

**Live (operator):** UAT steps in `sprints/quick/Q0028/uat.json`.

**Scope change (DEC-0109):** **All** tab now shows `pending` + `confirmed` only — `rejected` and `inactive` hidden by default. Pending tab unchanged. Unfiltered API list unchanged for ops.

---

## Credentials

- `DATABASE_URL` — external PostgreSQL (operator shell only — no inline secrets)
- OIDC provider config via Compose/env only

---

## Changes

| Area | Summary |
|------|---------|
| `backend/migrations/016_bug0020_subscription_list_quality.sql` | DA1/DB1 reconcile + backfill |
| `backend/src/subscriptions/detection.rs` | DA3 forward pending guard |
| `frontend/src/pages/SubscriptionsPage.tsx` | DA2 All-tab filter |
| `backend/tests/bug0020_subscription_list_quality.rs` | T1/G1 regression (7/7) |
| Runbook | §33 BUG-0020 operator smoke |

**Linked decisions:** DEC-0109 (reconcile + backfill contract); extends DEC-0085, DEC-0086, DEC-0100  
**Research fulfilled:** R-0090  
**Deferred:** BI-ALL browser visual until **BACKEND_FRONTEND_DEPLOY** (ForecastPage TS6133 fix)

---

## Known Issues

- `docker build` blocked by unrelated `ForecastPage.tsx` TS6133 (`hasForecast` unused) — operator must fix before image rebuild
- Running container serves pre-Q0028 static/binary — DA2 UI filter + DA3 guard not live until rebuild
- Migration 016 applied manually; `_sqlx_migrations` version 16 not registered (sqlx checksum conflict on migration 15)
- All-tab scope change — rejected/inactive no longer visible on All tab (documented per DEC-0109)

---

## Regression scope

- Discover/tags API unchanged
- Unfiltered `GET /api/v1/subscriptions` unchanged (full fidelity)
- Alert evaluation (BUG-0018) unchanged
- Grafana provisioning (BUG-0019) unchanged

---

## Rollback

```bash
git revert <Q0028-migration-and-code-commits>
# Restore pre-migration DB from backup if reconcile ran in production — migration is destructive
docker compose up -d --build flow-finance-ai
```

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0028 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0020-q0028`

## Milestone

**BUG-0020 released** — DEC-0109 subscription list reconcile + display_category backfill; BI/BJ runtime oracles PASS; All-tab UI deploy operator-deferred per pass-with-prerequisites.
