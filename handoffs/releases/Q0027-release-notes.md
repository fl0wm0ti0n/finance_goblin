# Quick Release Notes — Q0027 / BUG-0019

**Quick task:** Q0027  
**Bug:** BUG-0019 — Grafana metrics wrong (cashflow zeros, sync entity counts)  
**Date:** 2026-06-10  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0019 rows **BG**, **BH**; OIDC browser operator-deferred)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --test grafana_provisioning_bug0009` (6/6); static JSON guard 21/21 @ 2026-06-10 release; `sprints/quick/Q0027/qa-findings.md`
2. **QA completion gate:** PASS — `sprints/quick/Q0027/qa-findings.md` (0 blockers; cycle 2 re-run)
3. **UAT / verify-work gate:** PASS-WITH-PREREQUISITES — `sprints/quick/Q0027/uat.json`, `sprints/quick/Q0027/uat.md`, `handoffs/verify_work_to_release.md`; 7 steps — 5 pass, 2 pass_with_prerequisites, 0 fail; `ready_for_release: true`
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260610-bug0019-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Grafana provisioning-only fix per **DEC-0108** — correct `$account_id` default selection (sort:0 + empty `current`) and mirror-count Platform Health panel SQL. Restores Cashflow non-zero series for funded default account and BH sync entity counts without backend or migration changes.

| Scope | Fix |
|-------|-----|
| **CA1/CA2** | `grafana/provisioning/dashboards/analytics/cashflow.json` — `sort: 0`, `current`, `model_kind = 'baseline'` in panels 1–3 |
| **CA3** | `grafana/provisioning/dashboards/analytics/forecast-horizons.json` — `sort: 0` + `current` on `$account_id` |
| **CB1** | `grafana/provisioning/dashboards/platform-health.json` panel 2 — mirror `COUNT(*)` UNION ALL SQL |
| **G1** | Static JSON guard — 21/21 |
| **V1** | verify-work BG/BH oracles — pass; kiosk visual + OIDC browser deferred |

**Code proof:** `grafana_provisioning_bug0009` 6/6; static guard 21/21; BG API 25 points / 731 non-zero series; BH mirror 922 full + post-incremental.

**Operator post-release:** Optional omniflow OIDC browser BG/BH smoke; optional kiosk embed visual check per `sprints/quick/Q0027/uat.json`.

---

## Run

**Target service:** `grafana` (provisioning reload only — no backend image change).

```bash
docker compose restart grafana
```

Local override (`:13000`):

```bash
docker compose restart grafana
```

- `start_command`: `docker compose restart grafana`
- `runtime_mode`: local (`:13000` Grafana, `:18080` API) and remote (omniflow external US-0010)
- `runtime_context_ref`: `docs/engineering/runbook.md` (§32 BUG-0019 hotfix)

**Profile rule:** Provisioning-only — no `external` profile rebuild required for BG/BH; omniflow may need Grafana restart if dashboards not yet re-provisioned.

**Rebuild scope:** Grafana container restart only — three dashboard JSON files loaded at provisioning scan.

**Operator gate — GRAFANA_PROVISIONING_RELOAD (required before runtime probes):**

```bash
docker compose restart grafana
# Confirm container StartedAt fresh; Grafana API serves sort:0 + current + mirror SQL
```

**Operator gate — FULL_FIREFLY_SYNC_PLUS_INCREMENTAL_RERUN (required for BH incremental regression):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
# Full sync baseline; then incremental with 0 new transactions — panel 2 transactions must match mirror COUNT
```

---

## Connect

- `service_url`: `http://localhost:13000` (Grafana local) / `http://localhost:18080` (API) / `https://financegnome.omniflow.cc` (external)
- `service_port`: 13000 (Grafana) / 18080 (API local) / 443 (HTTPS via Traefik)
- `health_endpoint`: `GET http://localhost:18080/health` → JSON 200
- Cashflow dashboard: `http://localhost:13000/d/cashflow/cashflow?kiosk=tv`
- Platform Health: `http://localhost:13000/d/platform-health/platform-health`
- Forecast API oracle: `GET /api/v1/forecast/monthly?account_id=114`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| Traefik basic auth | Omniflow browser smoke — operator shell / password manager |
| OIDC provider config | Omniflow BG/BH regression — Compose/env only |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(BG)** | Cashflow default account | First `$account_id` option = funded account (114 fixture); panels 1–2 non-zero series |
| **(BG)** | `GET /api/v1/forecast/monthly?account_id=114` | 25 points; non-zero from Jul 2026; matches panel oracle |
| **(BG)** | Forecast Horizons | `sort: 0` + empty `current`; default account 114 |
| **(BH)** | Platform Health panel 2 after Full sync | `transactions` = `SELECT COUNT(*) FROM transactions` (922 fixture) |
| **(BH)** | After 0-new-tx incremental sync | Panel 2 count unchanged; `sync_cursors.records_synced` may be 0 |
| Regression | OIDC-enabled deploy | Standard OIDC BG/BH smoke per acceptance footnote |

**Automated (release):**

```bash
cd backend && cargo test --test grafana_provisioning_bug0009
python3 -c "# static JSON guard per sprints/quick/Q0027/qa-findings.md — 21/21"
```

**Live (operator):** 8-step checklist in `sprints/quick/Q0027/uat.json` `operator_smoke_checklist`.

**Expected health signal:** Grafana API serves DEC-0108 dashboards; BG ds query 731/731 non-zero; BH panel oracle `transactions=922`.

---

## Credentials

- Traefik basic auth — operator shell / password manager
- OIDC provider config via Compose/env only — no inline secrets in release artifacts
- `DATABASE_URL` — external PostgreSQL with TimescaleDB (unchanged — no new migration)

---

## Changes

| Area | Summary |
|------|---------|
| `grafana/provisioning/dashboards/analytics/cashflow.json` | CA1/CA2 — sort:0, current, baseline filter |
| `grafana/provisioning/dashboards/analytics/forecast-horizons.json` | CA3 — sort:0 + current |
| `grafana/provisioning/dashboards/platform-health.json` | CB1 — mirror-count UNION ALL panel SQL |
| `backend/tests/grafana_provisioning_bug0009.rs` | Regression guard (6/6) |
| Runbook | §32 BUG-0019 operator smoke |

**Linked decisions:** DEC-0108 (provisioning contract; supersedes DEC-0068 Y1 omit-`current`)  
**Research fulfilled:** R-0089  
**Deferred:** OIDC-1 omniflow browser smoke; BG-EMBED kiosk visual (operator optional)

---

## Known Issues

- Duplicate-UID provisioning warning (three provider YAMLs scan overlapping paths) — pre-existing, non-blocking; recommend follow-up bug for provider dedupe
- 43 `ml_enhanced` computations stuck `running` — out of scope per DEC-0108; recommend new backlog bug
- Omniflow browser/OIDC smoke deferred — auth barrier per prior bug precedent
- Fresh install with all-zero balances → arbitrary default account — acceptable per DEC-0108

---

## Regression scope

- Backend sync/cursor semantics unchanged (`upsert_cursor` untouched)
- `AnalyticsEmbedPage.tsx` unchanged
- Alert evaluation (BUG-0018 / DEC-0107) unchanged
- SPA deep-link fallback (BUG-0016) unchanged
- OIDC flow unchanged

---

## Rollback

```bash
git revert <Q0027-dashboard-json-commits>
docker compose restart grafana
```

No schema, data, or backend state involved.

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0027 `status=released`
- `release_notes_ref`: this file
- `release_version`: `bug0019-q0027`

## Milestone

**BUG-0019 released** — Grafana provisioning DEC-0108; BG/BH runtime oracles PASS; OIDC/kiosk visual operator-deferred per pass-with-prerequisites.
