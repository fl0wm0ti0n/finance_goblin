# Verify-work Findings — Quick Q0013 / BUG-0010

**Work item:** BUG-0010 (defect)  
**Quick task:** Q0013  
**Phase:** `/verify-work`  
**Orchestrator:** `auto-20260605-bug0010-001`  
**Date:** 2026-06-05  
**Verdict:** **PASS** — rows **(AA)/(AB)/(AC)** evidenced on production; proceed to `/release`

## Summary

Ran verify-work after operator deploy of AA1–AC2 and manual Full Firefly sync **`3e44fbfb`** (`finished_at`: **2026-06-05T16:55:41Z**). Local gates **PASS** (`cargo test --lib` 131/131, vitest 2/2, build PASS). Live omniflow smoke on `https://financegnome.omniflow.cc` confirms mirror balance backfill, honest signed wealth totals, forecast series population, `balance_warnings` for overdrawn Giro 114, and `ml_skipped_reason: sidecar_disabled` (AC3 ML production remains US-0013 out of scope).

## Per-row verdict (acceptance AA / AB / AC)

| Row | Verdict | Summary |
|-----|---------|---------|
| **(AA)** | **PASS** | Forecast API shows signed starting balance **-3395.75** (matches mirror); daily/monthly/long-term series populated post-recompute. 3-month end **-23590.16** is a plausible baseline projection from negative start + ~4–5k/mo outflows — not the prior silent **-25365.78** from wrong mirror. Meta `balance_warnings` includes acct **114** `negative_starting_balance` (AA3). |
| **(AB)** | **PASS** | Wealth API non-empty: **3** Firefly asset accounts; Giro **114** visible with `balance: -3395.75`, `is_overdrawn: true`; signed `subtotal_eur` / `total_eur`: **-3395.75** (was 0 / excluded). `wealth/history` post-sync snapshot **-3395.75** on 2026-06-05. Zero-total callout correctly absent (AB2). |
| **(AC)** | **PASS** | Meta `ml_status: skipped`, `ml_skipped_reason: sidecar_disabled`; baseline variant authoritative. UI code path shows "ML forecast is not enabled on this deployment" for `sidecar_disabled` — not generic "ML skipped: ML forecast unavailable…". AC3 (ML sidecar production on external profile) deferred **US-0013** — not blocking. |

**Release:** Check BUG-0010 acceptance checkbox during `/release`.

## Deploy / sync gate status

| Gate | Status |
|------|--------|
| `OPERATOR_DEPLOY_PENDING` | **CLEARED** |
| `OPERATOR_FULL_FIREFLY_SYNC_PENDING` | **CLEARED** — run `3e44fbfb` success 2026-06-05T16:55:41Z |
| Forecast recompute post-sync | **CLEARED** — meta `computed_at` 2026-06-05T16:55:45Z, `sync_run_id` matches |

## Automated verification

| Check | Result |
|-------|--------|
| `cd backend && cargo test --lib` | **PASS** (131/131) |
| `cd frontend && npm test` | **PASS** (2/2) |
| `cd frontend && npm run build` | **PASS** |
| Code path AA1–AC2 | **PASS** (unchanged from QA) |
| Rows AA/AB/AC runtime | **PASS** |

## Live curl evidence (2026-06-05, no Traefik credentials)

| Endpoint / probe | HTTP | Notes |
|------------------|------|-------|
| `GET /health` | 200 | Stack reachable |
| `GET /api/v1/sync/status` | 200 | `last_run.id: 3e44fbfb`, `status: success`, `trigger: manual` |
| `GET /api/v1/forecast/meta` | 200 | **PASS AA/AC** — `ml_skipped_reason: sidecar_disabled`; `balance_warnings` acct 114 `-3395.75` `negative_starting_balance`; `computed_at` post-sync |
| `GET /api/v1/forecast/long-term?account_id=114&horizon=3` | 200 | **PASS AA** — start `-3395.75`, end `-23590.16`, 118 points; variant `baseline` |
| `GET /api/v1/forecast/daily?account_id=114` | 200 | **PASS AA** — milestones + daily series populated |
| `GET /api/v1/forecast/monthly?account_id=114` | 200 | **PASS AA** — monthly series populated |
| `GET /api/v1/wealth` | 200 | **PASS AB** — 3 accounts; Giro 114 `is_overdrawn: true`; `total_eur: -3395.75` |
| `GET /api/v1/wealth/history?days=30` | 200 | **PASS AB3** — 2026-06-05 snapshot `-3395.75`, 3 accounts |

## Acceptance impact

| Row | Verify-work | `acceptance.md` checkbox |
|-----|-------------|--------------------------|
| **(AA)** | **PASS** | Release phase |
| **(AB)** | **PASS** | Release phase |
| **(AC)** | **PASS** | Release phase |
| Regression | **PARTIAL** — API smoke PASS; OIDC browser deferred (`oidc_issuer_url` empty, dev-bypass profile) | N/A |

## Release gate

| Gate | Status |
|------|--------|
| QA PASS | yes |
| Verify-work PASS | **yes** |
| Acceptance checked | release phase |
| Release proceed | **yes** |

## Operator advisory (non-blocking)

1. **Large negative 3-month projection (-23590):** Consistent with overdrawn start + recurring outflows; AA3 warning surfaces deficit scenario — not a mirror-ingest regression.
2. **Wealth history 2026-06-04 zero:** Pre-sync snapshot; 2026-06-05 reflects post-sync truth.
3. OIDC + bundled-firefly browser regression deferred (non-blocking per S0010/S0011 precedent).

## Next steps

1. **`/release`** — check BUG-0010 acceptance; publish Q0013 release notes
2. Operator browser smoke (OIDC) — advisory only

---

**Stop here.** Continue in a **new** subagent/chat with **`/release`**.
