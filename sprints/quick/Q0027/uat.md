# UAT — Q0027 (BUG-0019)

**Status:** COMPLETE — verify-work 2026-06-10  
**Verdict:** **PASS-WITH-PREREQUISITES**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0019 rows **BG**, **BH**  
**Sprint:** Q0027 (`/quick`)  
**Orchestrator:** `auto-20260610-bug0019`

## Operator gates (before live probes)

1. **GRAFANA_PROVISIONING_RELOAD** — `docker compose restart grafana` after the three JSON edits. — **PASS** (container restart 2026-06-10T20:41:55Z; live Grafana API serves DEC-0108 dashboards)
2. **FULL_FIREFLY_SYNC_PLUS_INCREMENTAL_RERUN** — Full sync (BH baseline 922), then incremental with 0 new transactions. — **PASS** (mirror 922; verify-work triggered incremental sync; panel oracle holds 922 while `sync_cursors.records_synced=0`)

## UAT steps

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| BG-EMBED | BG | Kiosk embed Cashflow: default account = 114; panels 1–2 non-zero (negative) series | pass_with_prerequisites | Variable SQL first=114; ds query 731/731 non-zero; browser visual optional |
| BG-DIRECT | BG | Direct Grafana Cashflow URL (no `var-account_id`): same default + non-zero | **PASS** | HTTP 200; sort:0 + current; 731/731 non-zero series |
| BG-API | BG | `GET /api/v1/forecast/monthly?account_id=114` — 25 points, non-zero from Jul 2026 | **PASS** | 25 months with non-zero buckets |
| BG-FH | BG | Forecast Horizons dashboard default account = 114 | **PASS** | sort:0 + empty current; HTTP 200 |
| BH-FULL | BH | Platform Health panel 2 `transactions` = mirror COUNT (922) after Full sync | **PASS** | Panel SQL + Grafana ds query → 922 |
| BH-INCR | BH | Panel 2 `transactions` still = mirror count after 0-new-tx incremental run | **PASS** | POST /sync/trigger success; oracle still 922 (old cursor path would show 0) |
| OIDC-1 | regression | Re-run BG/BH on omniflow profile | pass_with_prerequisites | root 401; /api/v1/forecast/meta 200 — browser deferred |

## Automated checks

| Check | Result |
|-------|--------|
| Static JSON guard (python3) | **21/21 PASS** |
| `cargo test --test grafana_provisioning_bug0009` | **6/6 PASS** |

## Duplicate-UID provisioning caveat

**Classification:** pre-existing (unchanged since first commit); **not introduced by Q0027**; **not blocking** BG/BH UAT.

Three provider YAMLs scan overlapping paths → Grafana logs `"the same UID is used more than once"`. Live API already serves fixed dashboard content. V1 risk: future re-provision may not persist to DB. Recommend follow-up bug to dedupe provider scan paths.

## Results summary

- **Row BG:** PASS (runtime oracles + Grafana config; embed visual pass-with-prerequisites)
- **Row BH:** PASS (mirror count 922 full + incremental regression confirmed)
- **Overall:** PASS-WITH-PREREQUISITES — OIDC browser smoke + optional kiosk visual deferred; no blocking failures
- **Next phase:** `/release`
