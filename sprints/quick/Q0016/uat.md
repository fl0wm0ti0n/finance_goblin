# UAT — Q0016 (BUG-0009)

**Status:** Verify-work **PASS** (2026-06-06)  
**Acceptance:** `docs/product/acceptance.md` — BUG-0009 rows **(Y)**, **(Z)** — **checked**  
**Plan-verify:** PASS (`sprints/quick/Q0016/plan-verify.json`, 2026-06-06)  
**Execute:** COMPLETE (`sprints/quick/Q0016/summary.md`, 2026-06-06)  
**QA:** PASS (`sprints/quick/Q0016/qa-findings.md`, 2026-06-06)  
**Verify-work:** PASS (`sprints/quick/Q0016/verify-work-findings.md`, 2026-06-06)  
**Orchestrator:** `auto-20260606-bug0009-001`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Verify-work |
|-----|---------|-----------------------------------|-------------|
| **(Y)** | Y1, Y2, T1, V1 | Grafana ds/query **200** with non-empty panel values at default `$account_id`; ML honest empty-state | **PASS** |
| **(Z)** | Z1, Z2, T1, V1 | Cross-account overview in analytics; portfolio shows all synced asset accounts | **PASS** |
| Regression | V1 | Six `/analytics/{slug}` routes smoke; ds/query **200** (BUG-0003 H, BUG-0004 K) | **PASS** |

## Operator gate

1. Deploy image with Z1–Y2 + T1 merged. — **DONE**
2. **GRAFANA_PROVISIONING_RELOAD** — `docker compose … up -d --force-recreate grafana`. — **DONE**
3. Do **not** save dashboard variables in Grafana UI (bakes `current` into DB — see runbook § BUG-0009).

## Smoke checklist (omniflow — `financegnome.omniflow.cc`)

| Step | Probe | Pass criteria | Result |
|------|-------|---------------|--------|
| Y-1 | `/analytics/cashflow` default load (no manual account pick) | Non-flat forecast series (acct 114 pattern) | **PASS** — default acct **114**; min **-132348.57** ≠ max **-3395.75** (731 rows) |
| Y-2 | `/analytics/forecast-horizons` | ML status banner visible; ML panels show `ML unavailable` not blank confusion | **PASS** — panel 13 banner in live dashboard API; `ml_enhanced` count **0** |
| Y-3 | `POST /analytics/grafana/api/ds/query` cashflow + forecast panels | **200** | **PASS** |
| Z-1 | `/analytics/portfolio` | Overview table **3 rows** (name, role, currency, balance); `total_eur` stat visible above fold | **PASS** — 3 rows; `total_eur` **-3395.75** |
| Z-2 | Portfolio breakdown ds/query | Returns 3 account rows (not 1) | **PASS** |
| Z-3 | Six analytics routes | cashflow, subscriptions, budgets, portfolio, forecast-horizons, platform-health render in embed | **PASS** — Grafana embed `d/{uid}` all **200** |
| REG-1 | ds/query portfolio pie (UNION) | **200** — BUG-0004 K regression | **PASS** — 2 slices |
| SUP-1 | `/wealth` | Documented supplementary view (Z3) — not AC substitute | **N/A** |

## Local gates (execute — completed)

| Step | Description | Result |
|------|-------------|--------|
| T-1 | `cargo test --test grafana_provisioning_bug0009` | **PASS** (6/6) |
| T-2 | Provisioning JSON contract (ABS sort, LATERAL SQL, ML banner, no `current`) | **PASS** |

## Notes

- SPA `/analytics/{slug}` paths return **404** on unauthenticated curl (client-side nav); embed routes are the smoke contract (US-0011 precedent).
- US-0013 ML enablement explicitly out of scope — banner sets honest degraded posture per DEC-0066.

## Next phase

**`/release`** — Q0016 release notes, backlog BUG-0009 → DONE.
