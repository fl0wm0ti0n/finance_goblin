# Quick Release Notes — Q0023 / BUG-0015

**Quick task:** Q0023  
**Bug:** BUG-0015 — Confirmed subscriptions reappear as pending after rebuild  
**Date:** 2026-06-07  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0015 rows AU–AW; runtime operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (187/187) @ 2026-06-07 release
2. **QA completion gate:** PASS — `sprints/quick/Q0023/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0023/uat.json`, `sprints/quick/Q0023/uat.md`, `handoffs/verify_work_to_release.md`; 10 steps — 3 pass, 7 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260607-bug0015-q0023-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Confirm-once subscription persistence after container rebuild per **DEC-0084** (AU1 card `payee_key` normalization), **DEC-0085** (AU2–AU3 payee+interval confirm inheritance skip+merge), **DEC-0086** (±3d interval tolerance + in-place fingerprint rotation on merge). **V1** omniflow rebuild smoke deferred pending operator gates.

| Scope | Fix |
|-------|-----|
| **AU** | Card billing `payee_key` rules; payee+interval merge before pending upsert; confirmed Cursor/Apple remain confirmed after rebuild + Full sync |
| **AV** | `load_confirmed_payee_intervals` + `merge_confirmed_pattern`; detection skip by payee+interval; stale inactive by payee+interval (AU4) |
| **AW** | Merge path suppresses `new_detection` alert; unread count reconciles with pending tab |

**Code proof:** `cargo test --lib` 187/187; card_billing 4/4; interval_matches 2/2; build_active_payee 1/1; frontend vitest 6/6.

**Operator post-release:** Deploy Q0023 backend; run 10-step rebuild smoke per `sprints/quick/Q0023/uat.json` `operator_smoke_checklist`.

---

## Run

**Target service (external profile):** `flow-finance-ai` (AU1–AU4 backend).

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before runtime probes):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — POSTGRES_PERSISTENCE_PROBE (H2 SQL after app rebuild, before Full sync):**

```sql
SELECT status, COUNT(*) FROM subscription_patterns GROUP BY status;
SELECT fingerprint, status, payee_key, interval_days, current_amount
FROM subscription_patterns
WHERE payee_key ILIKE '%cursor%' OR payee_key ILIKE '%apple%'
ORDER BY updated_at DESC;
```

| Outcome | Action |
|---------|--------|
| Zero `confirmed` rows after rebuild (no operator action) | **Ops** — volume/DB target (H2); do not run V1 until resolved |
| `confirmed` rows present; drift after Full sync | AU1–AU4 path validated |
| Single confirmed per merchant; still pending in UI | Reopen discovery — unlikely per H3 refutation |

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) + subscription detection phase.

- `start_command`: docker compose commands above + Full sync from Settings UI
- `runtime_mode`: remote (omniflow external US-0010)
- `runtime_context_ref`: `docs/engineering/runbook.md` (§25 BUG-0015 hotfix)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

**Rebuild scope:** Recreate `flow-finance-ai` only — postgres volume untouched.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- `service_port`: 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health`
- Subscriptions UI: `/subscriptions`
- Subscriptions API: `GET /api/v1/subscriptions`, `GET /api/v1/subscriptions/alerts/unread-count`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile |
| `AUTH_DEV_BYPASS` | Local API-only dev only — not omniflow |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(AU)** | Confirm Cursor + Apple → rebuild app → Full sync | `GET /api/v1/subscriptions?status=confirmed` includes merchants; `/subscriptions` no Confirm/Reject for confirmed |
| **(AV)** | Post-sync detection | No duplicate `status=pending` for same payee+interval; merge/skip path active |
| **(AW)** | Unread alerts | `GET /api/v1/subscriptions/alerts/unread-count` reconciles with pending tab; no spurious `new_detection` |
| **H2-1** | H2 SQL before Full sync | `confirmed` rows present after rebuild (rules out volume wipe) |
| Regression | OIDC-enabled deploy | Auth + `/subscriptions` load without transport errors |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd backend && cargo test --lib card_billing
cd backend && cargo test --lib interval_matches
cd frontend && npm test -- --run
```

**Live (operator post-deploy):** 10-step checklist in `sprints/quick/Q0023/uat.json` after all operator gates complete.

**Expected health signal:** `GET /health` → HTTP 200; confirmed subscriptions persist across app container rebuild when postgres persists.

---

## Credentials

- Traefik basic auth — operator shell / password manager
- No inline secrets in release artifacts

---

## Changes

| Area | Summary |
|------|---------|
| `backend/src/recurrence/normalize.rs` | AU1 — card billing `payee_key` rules (DEC-0084) |
| `backend/src/subscriptions/repository.rs` | AU2 — payee+interval maps, merge upsert |
| `backend/src/subscriptions/detection.rs` | AU3/AU4 — skip+merge path; stale inactive by payee+interval |
| `backend/src/subscriptions/service.rs` | AU4 — wire stale inactive post-candidates |
| `backend/src/subscriptions/types.rs` | `ConfirmedPayeeInterval` type |
| `backend/migrations/012_subscription_patterns_payee_status.sql` | Index `idx_subscription_patterns_payee_status` |
| Runbook | §25 BUG-0015 operator smoke |

**Linked decisions:** DEC-0084, DEC-0085, DEC-0086 (extends DEC-0071, DEC-0072)  
**Research fulfilled:** R-0081, R-0082  
**Deferred:** V1 omniflow runtime smoke (operator gates)

---

## Known Issues

- V1 omniflow runtime probes pass-with-prerequisites until **BACKEND_FRONTEND_DEPLOY**, **POSTGRES_PERSISTENCE_PROBE**, **FULL_FIREFLY_SYNC**
- Omniflow API health returned 404 at verify-work — deploy pending per BUG-0013/0014 precedent
- H2 false negative possible — mandatory POSTGRES_PERSISTENCE_PROBE before Full sync

---

## Regression scope

- BUG-0008 alert dedup contract preserved
- US-0003 subscription confirm/reject UX unchanged
- DEC-0072 SEPA normalization paths unchanged for non-card payees

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0023 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0015-q0023`

## Milestone

**BUG-0015 released** — confirm-once subscription persistence after rebuild via payee normalization and payee+interval inheritance; operator rebuild smoke deferred per pass-with-prerequisites.
