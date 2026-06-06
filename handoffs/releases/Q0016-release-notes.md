# Quick Release Notes — Q0016 / BUG-0009

**Quick task:** Q0016  
**Bug:** BUG-0009 — Grafana empty panels & missing account value overview  
**Date:** 2026-06-06  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0009 rows Y/Z)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --test grafana_provisioning_bug0009` (6/6 @ 2026-06-06 release)
2. **QA completion gate:** PASS — `sprints/quick/Q0016/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0016/uat.md`, `handoffs/verify_work_to_release.md`; omniflow rows Y/Z live curl PASS after Grafana provisioning reload
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — verify-work tuple `runtime-proof-verify-work-20260606-bug0009-q0016-001`
6. **Release finalization gate:** PASS

---

## Summary

Provisioning-only Grafana analytics fix per **DEC-0068** on US-0010 external profile (`financegnome.omniflow.cc`):

| Scope | Fix |
|-------|-----|
| **Y1** | `$account_id` defaults to highest ABS(balance) asset account (114 Giro, not zero wallet 116) |
| **Y2** | ML status banner + `noValue: "ML unavailable"` on ML panels when `ml_enhanced` absent |
| **Z1** | Portfolio breakdown SQL — latest-snapshot subquery + `LATERAL jsonb_array_elements` (3 rows) |
| **Z2** | Cross-account overview table **"All accounts (latest snapshot)"** below stat row |

**Production proof:** Default account **114**; cashflow **731** non-flat rows; portfolio overview **3** rows; `total_eur` **-3395.75**; ML banner panel 13 live.

---

## Run

**Target service (external profile):** `grafana` — provisioning reload only; **no backend rebuild required** unless image bundles JSON.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build grafana
```

**Operator gate — GRAFANA_PROVISIONING_RELOAD (required before smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate grafana
```

**Grafana UI warning:** Do **not** click **Save** on analytics dashboards after changing variables — persisted `current` blocks override provisioning JSON.

- `start_command`: docker compose commands above
- `runtime_mode`: remote (omniflow external US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§17 BUG-0009 hotfix)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- `service_port`: 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health`
- Grafana embed: `/analytics/grafana/d/{uid}`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite §11) |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(Y)** | `/analytics/cashflow` default load (no manual account pick) | Non-flat series at funded account 114 |
| **(Y)** | `/analytics/forecast-horizons` | ML status banner visible; ML panels show `ML unavailable` |
| **(Z)** | `/analytics/portfolio` | Overview table **3** account rows; `total_eur` stat **-3395.75** |
| Regression | Six `/analytics/{slug}` embed routes + `POST …/ds/query` | **200** (BUG-0003 H, BUG-0004 K) |
| T1 | `cd backend && cargo test --test grafana_provisioning_bug0009` | **6/6 PASS** |

**Automated (release):**

```bash
cd backend && cargo test --test grafana_provisioning_bug0009
```

**Live (verify-work):** account 114 ABS sort; cashflow min **-132348.57** ≠ max **-3395.75**; portfolio **3** rows; six embed routes **200**.

**Supplementary:** React `/wealth` per-account detail — not AC Z substitute (Z3).

---

## Credentials

- Traefik basic auth — operator shell / password manager
- No inline secrets in release artifacts

---

## Changes

| Area | Summary |
|------|---------|
| `grafana/.../portfolio.json` | Z1 LATERAL breakdown SQL; Z2 overview table + layout |
| `grafana/.../cashflow.json` | Y1 ABS(balance) variable query |
| `grafana/.../forecast-horizons.json` | Y1 variable query; Y2 ML banner + noValue |
| `backend/tests/grafana_provisioning_bug0009.rs` | T1 JSON contract + SQL fixtures (6 tests) |
| Runbook | §17 BUG-0009 operator smoke + `current` save warning |

**Linked decisions:** DEC-0068  
**Deferred epic:** US-0013 (ML overlay enablement on omniflow)

---

## Known Issues

- SPA paths `/analytics/{slug}` return **404** on unauthenticated curl (client-side routing — same advisory as BUG-0004/BUG-0012)
- ML overlay remains disabled on external profile until US-0013
- Browser OIDC regression smoke deferred (API PASS)

---

## Regression scope

- Portfolio UNION pie, subscriptions, budgets, platform-health unchanged
- Backend forecast/wealth/React pages unchanged
- BUG-0004 K/L fixes preserved
