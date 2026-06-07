# Quick Release Notes — Q0018 / BUG-0008

**Quick task:** Q0018  
**Bug:** BUG-0008 — Subscription alerts vs list mismatch & under-detection  
**Date:** 2026-06-08  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0008 rows W/X)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (156/156); `cargo test --test bug0008_subscription_alerts` (8/8) @ 2026-06-08 release
2. **QA completion gate:** PASS — `sprints/quick/Q0018/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0018/verify-work-findings.md`, `handoffs/verify_work_to_release.md`; W/X code PASS; runtime pass-with-prerequisites (BACKEND_FRONTEND_DEPLOY)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — verify-work tuple `runtime-proof-verify-work-20260608-bug0008-q0018-001`
6. **Release finalization gate:** PASS

---

## Summary

Subscription alert reconciliation and detection recall per **DEC-0071** (W bundle) and **DEC-0072 Phase 1** (X recall) on US-0010 external profile:

| Scope | Fix |
|-------|-----|
| **W** | Alert `fingerprint` dedup migration; `upsert_alert` ON CONFLICT; emit gate (new pending / tier increase only); `GET /api/v1/subscriptions/alerts/unread-count` with reconciled semantics; orphan mark-read on confirm/reject/inactive; frontend banner + toast from unread-count API |
| **X** | SEPA + legal suffix normalization; transfer counterparty priority guard; `detection_window_days = 730`; forecast + subscription regression tests; spam invariant via emit gate + fingerprint dedup |

**Code proof:** `bug0008_subscription_alerts` 8/8; `cargo test --lib` 156/156.

**Operator post-release:** Deploy migration `010_subscription_alert_fingerprint.sql` + W6 frontend; run omniflow smoke W-1–W-4, X-1–X-2 per `sprints/quick/Q0018/uat.md`.

---

## Run

**Target service (external profile):** `flow-finance-ai` — rebuild/recreate for Q0018 backend + embedded frontend changes.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before V1 smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

- `start_command`: docker compose commands above
- `runtime_mode`: remote (omniflow external US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§19 BUG-0008 hotfix)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- `service_port`: 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health`
- Subscriptions UI: `/subscriptions`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite §11) |
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Mirror data gate (922+ txs) |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(W)** | `GET /api/v1/subscriptions/alerts/unread-count` | `reconciled: true`; `unread_new_detection <= pending_patterns` |
| **(W)** | `/subscriptions` banner | Count from `unread_new_detection` — not raw alert list length |
| **(W)** | Post-sync toast | Fires on sessionStorage delta of `unread_new_detection` only |
| **(W)** | Confirm/reject pattern | Orphan alerts marked read; unread-count drops |
| **(X)** | Pattern count after deploy + optional sync | Total patterns > 12 live baseline |
| **(X)** | Resync spam guard | `unread_new_detection` not >> `pending_patterns` |
| Regression | Subscription routes + health | **PASS** post-deploy |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test bug0008_subscription_alerts
```

**Live (operator post-deploy):** W-1–W-4, X-1–X-2, REG-1–REG-2 per `sprints/quick/Q0018/uat.md`.

---

## Credentials

- Traefik basic auth — operator shell / password manager
- No inline secrets in release artifacts

---

## Changes

| Area | Summary |
|------|---------|
| `backend/migrations/010_subscription_alert_fingerprint.sql` | Fingerprint column, backfill, dedupe, partial unique index |
| `backend/src/subscriptions/{repository,detection,service,types}.rs` | Dedup upsert, emit gate, unread-count, orphan lifecycle |
| `backend/src/api/subscriptions.rs` | Unread-count route |
| `backend/src/recurrence/{normalize,group}.rs` | SEPA/legal normalization; transfer counterparty guard |
| `backend/src/config/mod.rs`, `backend/config/default.toml` | `detection_window_days=730` |
| `backend/tests/bug0008_subscription_alerts.rs` | Contract suite (8 tests) |
| `frontend/src/pages/SubscriptionsPage.tsx`, `frontend/src/lib/api.ts` | Banner + toast from unread-count API |
| Runbook | §19 BUG-0008 operator smoke |

**Linked decisions:** DEC-0071, DEC-0072  
**Research fulfilled:** R-0068, R-0069, R-0009–R-0013  
**Deferred:** X Phase 2 category gate; AI-in-pipeline (optional research)

---

## Known Issues

- V1 omniflow runtime probes pass-with-prerequisites until **BACKEND_FRONTEND_DEPLOY**
- OIDC browser regression smoke deferred (API PASS per prior bug releases)
- Live pattern count (X-1) may require optional full Firefly sync post-deploy

---

## Regression scope

- BUG-0007 AI discovery unchanged (coordinate-only)
- US-0005 unified alert bell unchanged (subscription banner uses dedicated unread-count API)
- Six-tool AI registry + `allow_raw_transactions=false` preserved
