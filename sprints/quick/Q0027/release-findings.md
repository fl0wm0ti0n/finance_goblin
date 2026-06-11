# Release Findings — Quick Q0027 / BUG-0019

**Quick task:** Q0027  
**Bug:** BUG-0019  
**Phase:** `/release`  
**Date:** 2026-06-10  
**Orchestrator:** `auto-20260610-bug0019`  
**Gate status:** **PASS**

## Gate audit (US-0039)

| gate | verdict | reason_code | remediation | evidence_refs |
|------|---------|-------------|-------------|---------------|
| check-in_test | pass | — | — | `cargo test --test grafana_provisioning_bug0009` 6/6; static guard 21/21; `sprints/quick/Q0027/qa-findings.md` |
| qa | pass | — | — | `sprints/quick/Q0027/qa-findings.md` (0 blockers; cycle 2 PASS) |
| uat | pass-with-prerequisites | — | — | `sprints/quick/Q0027/uat.json`, `handoffs/verify_work_to_release.md` — 5 pass, 2 pass_with_prerequisites, 0 fail |
| isolation | pass | — | — | `docs/engineering/state.md` (execute→verify-work checkpoints) |
| runtime_proof | pass | — | — | `runtime-proof-verify-work-20260610-bug0019-001`; release tuple at finalization |
| finalization | pass | — | — | `handoffs/releases/Q0027-release-notes.md`, backlog BUG-0019 DONE |
| publish | skipped | — | — | `RELEASE_PUBLISH_MODE=disabled` |

**Doc gates:** README feature coverage `skipped` (`README_FEATURE_COVERAGE_ENFORCE=0`)

## Release verdict

**PASS** — BUG-0019 finalized; acceptance **BG**, **BH** checked; Grafana provisioning reload documented; OIDC/kiosk visual operator-deferred.

## Blocking findings

None.

## Non-blocking findings

- Duplicate-UID provisioning warning — pre-existing triple provider scan; live API serves fixed dashboards; recommend follow-up bug for provider dedupe
- BG-EMBED kiosk browser visual — pass-with-prerequisites; config + datasource oracles sufficient for release
- OIDC-1 omniflow browser BG/BH smoke — pass-with-prerequisites; API reachability OK (401/200)
- 43 `ml_enhanced` computations stuck `running` — out of scope; recommend new backlog bug

## Deployment steps

1. **Apply JSON edits** (already in repo): `cashflow.json`, `forecast-horizons.json`, `platform-health.json`
2. **Re-provision Grafana:**

```bash
docker compose restart grafana
```

3. **Verify operator gates** (verify-work already executed locally):
   - GRAFANA_PROVISIONING_RELOAD — container restart; API serves sort:0 + current + mirror SQL
   - FULL_FIREFLY_SYNC_PLUS_INCREMENTAL_RERUN — mirror 922; incremental 0-new-tx regression confirmed

## Rollback

```bash
git revert <Q0027-dashboard-json-commits>
docker compose restart grafana
```

No migration or backend rollback required.

## Operator follow-up (post-release, optional)

1. **OIDC omniflow smoke** — repeat BG/BH on `https://financegnome.omniflow.cc` after Grafana restart on external profile
2. **Kiosk visual** — open Analytics → Cashflow embed; confirm default account 114 and non-zero panels
3. **Duplicate-UID follow-up** — file backlog bug to dedupe Grafana provider scan paths (three YAMLs overlap)
4. **ml_enhanced stuck computations** — file backlog bug per DEC-0108 out-of-scope note

## Operator smoke checklist (8 steps)

1. `docker compose restart grafana` — **DONE** (2026-06-10T20:41:55Z at verify-work)
2. Kiosk embed Cashflow — default 114, non-zero panels — ORACLE PASS; browser visual optional
3. Direct Grafana Cashflow (no `var-account_id`) — **PASS**
4. `curl GET /api/v1/forecast/monthly?account_id=114` — **PASS**
5. Forecast Horizons default account 114 — **PASS**
6. Full sync; Platform Health panel 2 transactions = 922 — **PASS**
7. Incremental sync 0-new-tx; panel 2 still 922 — **PASS** (verify-work executed)
8. Repeat BG/BH on omniflow — **DEFERRED** (operator OIDC browser)

## Rerun criteria

N/A — release finalization PASS.
