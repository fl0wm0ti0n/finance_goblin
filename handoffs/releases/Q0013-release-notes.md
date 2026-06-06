# Quick Release Notes — Q0013 / BUG-0010

**Quick task:** Q0013  
**Bug:** BUG-0010 — Forecast & Wealth empty/wrong numbers; ML skipped  
**Date:** 2026-06-05  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0010)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (131/131), `cd frontend && npm test` (2/2), `npm run build` @ 2026-06-05 (release)
2. **QA completion gate:** PASS — `sprints/quick/Q0013/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0013/uat.md`, `handoffs/verify_work_to_release.md`; omniflow rows AA/AB/AC live curl PASS (OIDC browser regression deferred)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

---

## Run

**Target services (external profile):** `flow-finance-ai` — rebuild/recreate for AA1–AC2 forecast/wealth/ML posture fixes; operator **Manual Full Firefly sync** required for acceptance AA/AB.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator prerequisite (V1 — required for AA/AB/AC acceptance):**

1. Deploy Q0013 backend + frontend image to omniflow.
2. Trigger **Manual Full Firefly sync** — mirror balance backfill per DEC-0002 (no SQL migration).
3. Confirm forecast recompute + wealth snapshot after sync.

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

- `runtime_mode`: omniflow external (US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§ Omniflow §15)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- **AA:** Forecast shows signed starting balances matching mirror; daily/monthly/long-term series populated; `balance_warnings` for overdrawn accounts
- **AB:** Wealth includes synced asset accounts (including overdrawn Giro); honest signed `total_eur`
- **AC:** Meta `ml_skipped_reason: sidecar_disabled` when ML off; UI "not enabled" copy — not misleading generic skip banner

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Full Firefly sync gate |
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite from §11) |

**Out of scope (US-0013 epic):** AC3 — ML sidecar production on external profile (`stats-forecast`, `FORECAST_ML_ENABLED=true`).

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **AA** | `GET /api/v1/forecast/long-term?account_id=114&horizon=3` | Start balance matches mirror (e.g. -3395.75); series populated; not silent -25365.78 without warning |
| **AA** | `GET /api/v1/forecast/meta` | `balance_warnings` for acct 114 `negative_starting_balance` when applicable |
| **AB** | `GET /api/v1/wealth` | ≥3 asset accounts; Giro 114 visible with `is_overdrawn: true`; signed `total_eur` |
| **AB** | `GET /api/v1/wealth/history?days=30` | Post-sync snapshot matches current total |
| **AC** | `GET /api/v1/forecast/meta` | `ml_skipped_reason: sidecar_disabled`; baseline authoritative |
| Stack | `GET /health` | 200 |
| Sync | `GET /api/v1/sync/status` | Last manual success run recorded |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd frontend && npm test && npm run build
```

**Live (verify-work):** sync `3e44fbfb`; forecast start -3395.75, 3mo end -23590.16; wealth total -3395.75; meta sidecar_disabled.

---

## Changes

| Area | Summary |
|------|---------|
| Firefly ingest | AA1 balance mirror diagnostics (`balance_ingest` logs) |
| Wealth | AB1 DEC-0065 negative asset visibility + `is_overdrawn`; AB2 zero-total callout |
| Forecast meta | AA3 `balance_warnings`; AC1 DEC-0066 `sidecar_disabled` on baseline |
| Forecast UI | AA3 negative-start banner; AC2 ML three-state copy |
| Sync | AC1 persist ML posture when `forecast_ml.enabled=false` |

**Linked decisions:** DEC-0065, DEC-0066  
**Research fulfilled:** R-0062  
**Deferred epic:** US-0013 (AC3 ML production on omniflow)

---

## Regression scope

- DEC-0007 baseline forecast math unchanged
- DEC-0049 ML disabled by default on external profile preserved
- OIDC-enabled deploy: deferred browser smoke (non-blocking per prior bug releases)

## Known issues (non-blocking)

- Large negative 3-month projection mathematically consistent with overdrawn start + outflows — AA3 warning satisfies honest deficit disclosure
- Wealth history pre-sync zero row expected; post-sync snapshot correct
- AC3 ML overlay on external profile → **US-0013** OPEN epic

## Operator advisory

After deploy, run **Manual Full Firefly sync**. Confirm rows AA/AB/AC via forecast meta, long-term/daily/monthly series, wealth breakdown, and sync status. Optional: browser smoke for OIDC regression.
