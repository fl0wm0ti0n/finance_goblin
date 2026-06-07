# Quick Release Notes — Q0020 / BUG-0013

**Quick task:** Q0020  
**Bug:** BUG-0013 — Omniflow analytics regression cluster (budgets MTD, crypto pricing, Grafana copy)  
**Date:** 2026-06-09  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0013 rows AI–AN)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (174/174) @ 2026-06-09 release
2. **QA completion gate:** PASS — `sprints/quick/Q0020/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0020/uat.json`, `sprints/quick/Q0020/uat.md`, `sprints/quick/Q0020/verify-work-findings.md`; 12 steps — 5 pass, 7 pass_with_prerequisites, 0 fail
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260609-bug0013-q0020-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Omniflow analytics regression fixes per **DEC-0079** (AL1 budgets MTD upper date bound) and **DEC-0080** (AN1 Bitunix wallet array parse + linear unrealized USDT→EUR) plus optional Grafana copy (**AJ1**, **AK2**) on US-0010 external profile:

| Scope | Fix |
|-------|-----|
| **AL** | Panel 5 `planned` CTE: `pdc.ts::date <= CURRENT_DATE`; deviation uses capped planned; mid-month footnote |
| **AN/AK** | `resolve_futures_account` array shape; `unrealizedPNL` keys; linear unrealized USDT→EUR excluded from `crypto_value_eur` |
| **AJ** | Subscriptions price-changes panel documents 90d empty-state expectation |
| **AK2** | Portfolio performance % `noValue` "Needs ≥2 snapshots" + description |
| **AM** | Waived per R-0077 (curl ds/query 200; browser repro deferred) |
| **AI** | Ops regression only — no Q0020 code change; baseline acct 114 smoke after Full sync |

**Code proof:** `cargo test --lib` 174/174; 5 new Bitunix/PnL unit tests PASS.

**Operator post-release:** Deploy backend + Grafana provisioning reload + Full Firefly sync; run AL-1 through REG-1 per `sprints/quick/Q0020/uat.md`.

---

## Run

**Target services (external profile):** `flow-finance-ai` (AL1 + AN1 backend) and `grafana` (AL1/AJ1/AK2 dashboard JSON).

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AN/AK wealth probes):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — GRAFANA_PROVISIONING_RELOAD (required before AL/AJ/AK live panels):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate grafana
```

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) + forecast recompute before AI/AN/AK wealth probes.

- `start_command`: docker compose commands above + Full sync from Settings UI
- `runtime_mode`: remote (omniflow external US-0010)
- `runtime_context_ref`: `docs/engineering/runbook.md` (§23 BUG-0013 hotfix)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

**Grafana warning:** Do **not** click **Save** on analytics dashboards after variable changes — persisted `current` blocks override provisioning JSON (see runbook §17).

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- `service_port`: 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health`
- Analytics routes: `/analytics/budgets`, `/analytics/portfolio`, `/analytics/subscriptions`, `/analytics/cashflow`, `/analytics/forecast-horizons`
- Wealth API: `GET /api/v1/wealth`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite §11) |
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Mirror data for forecast baseline |
| `BITUNIX_*` | Read-only exchange keys when testing AN/AK crypto valuation |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(AL)** | `/analytics/budgets` MTD summary | Planned/actual/deviation plausible — not −€150K artifact |
| **(AN)** | `GET /api/v1/wealth` | `crypto.subtotal_eur` > 0 when Bitunix equity > 0 post-sync |
| **(AK)** | `/analytics/portfolio` crypto stat | Non-zero after sync + recompute when holdings priced |
| **(AK)** | Performance % panel | "Needs ≥2 snapshots" or data when history exists |
| **(AJ)** | `/analytics/subscriptions` price changes | Event rows or documented empty-state copy |
| **(AI)** | `/analytics/cashflow` + `/analytics/forecast-horizons` acct 114 | Non-empty signed baseline balances post Full sync |
| **(AM)** | `POST /analytics/grafana/api/ds/query` | HTTP **200** (waived unless HAR shows failure) |
| Regression | Six `/analytics/{slug}` routes | Embed without transport errors |

**Automated (release):**

```bash
cd backend && cargo test --lib
```

**Live (operator post-deploy):** AL-1, AN-1, AK-1, AK-2, AI-1, AJ-1, REG-1 per `sprints/quick/Q0020/uat.md` after all three operator gates complete.

**Expected health signal:** `GET /health` → HTTP 200; `GET /api/v1/wealth` → `crypto.subtotal_eur` populated when exchange sync succeeded; budgets MTD planned magnitude matches elapsed month days.

---

## Credentials

- Traefik basic auth — operator shell / password manager
- `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` — operator `.env` only; never inline in artifacts
- No inline secrets in release artifacts

---

## Changes

| Area | Summary |
|------|---------|
| `grafana/provisioning/dashboards/analytics/budgets.json` | AL1 — MTD planned `<= CURRENT_DATE` + footnote |
| `grafana/provisioning/dashboards/analytics/subscriptions.json` | AJ1 — price-changes empty-state description |
| `grafana/provisioning/dashboards/analytics/portfolio.json` | AK2 — performance % min-snapshot footnote |
| `backend/src/exchanges/bitunix.rs` | AN1 — `resolve_futures_account` array wallet shape |
| `backend/src/exchanges/repository.rs` | AN1 — futures wallet ingest path |
| `backend/src/portfolio/pnl.rs` | AN1/AK — linear unrealized USDT→EUR |
| Runbook | §23 BUG-0013 operator smoke |

**Linked decisions:** DEC-0079, DEC-0080  
**Research fulfilled:** R-0076, R-0077  
**Deferred:** AM browser annotation/live WS repro unless operator HAR shows failure

---

## Known Issues

- V1 omniflow runtime probes pass-with-prerequisites until **BACKEND_FRONTEND_DEPLOY**, **GRAFANA_PROVISIONING_RELOAD**, and **FULL_FIREFLY_SYNC**
- Row **AI** is ops regression smoke only — discovery refuted code regression; zeros likely pre-recompute or ML-only panels
- Performance % requires ≥2 wealth snapshots — honest empty-state until history accumulates
- MetaMask `contentscript.js` console noise out of scope

---

## Regression scope

- US-0015 AI bucket mapping unchanged
- BUG-0011 planning mode unchanged
- BUG-0009 account default / overview panels unchanged (coordinate-only)
- DEC-0064 crypto subtotal rules preserved
