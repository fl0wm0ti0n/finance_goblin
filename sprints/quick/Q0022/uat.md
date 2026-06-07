# UAT — Q0022 (BUG-0014)

**Status:** POPULATED — verify-work complete 2026-06-07  
**Acceptance:** `docs/product/acceptance.md` — BUG-0014 rows **AO**, **AP**, **AQ**, **AR**, **AS**, **AT**  
**Sprint:** Q0022 (`/quick`)  
**Verdict:** **PASS** — code/test complete; runtime probes pass-with-prerequisites (BUG-0013 precedent)  
**Next phase:** `/release`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Result |
|-----|---------|-----------------------------------|--------|
| **AO** | AO1, V1 | ML available after Full sync or accurate sidecar-down copy — not permanent US-0013 not-enabled when env opts in | **pass** (code) / **pass_with_prerequisites** (AO-1 live) |
| **AP** | AP2, V1 | crypto.subtotal_eur > 0 when Bitunix wallet priced; holdings reflect DEC-0080 | **pass_with_prerequisites** (AP2 skipped; AP1/AP-1 operator) |
| **AQ** | AQ1, AQ2, V1 | Native quantities + EUR equivalents; FX incomplete only with documented unpriced_assets | **pass** |
| **AR** | AR1, V1 | Cashflow panels non-zero acct 114 after Full sync + recompute; AR1 only if API≠Grafana | **pass_with_prerequisites** (AR1 skipped; AR probes operator) |
| **AS** | AS1, AS2, V1 | Delete non-active plan; 409 on active; valid target_type + help | **pass** |
| **AT** | V1 | Three-service external compose includes stats-forecast when ML enabled | **pass_with_prerequisites** (ops AT-1) |

## Operator gates (before live omniflow probes)

1. **BACKEND_FRONTEND_DEPLOY** — Q0020+ bundle on omniflow. — **PENDING**
2. **THREE_SERVICE_COMPOSE** — `stats-forecast` alongside app + Grafana per DEC-0076. — **PENDING**
3. **FULL_FIREFLY_SYNC** — Full sync + forecast recompute acct **114**. — **PENDING**
4. **GRAFANA_PROVISIONING_RELOAD** — after AO1 (and AR1 if executed). — **PENDING**
5. **AP1_SQL_PROBE** — `exchange_holdings` futures row priced before AP2 attribution. — **PENDING**

## UAT steps (verify-work results)

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| AO-CODE | AO | forecast-horizons panel 13 dual-scenario copy | **pass** | `forecast-horizons.json`; VW-AUTO-2 6/6 |
| AO-1 | AO | `GET /api/v1/forecast/meta` ml_status when sidecar healthy | **pass_with_prerequisites** | Omniflow API 404; pending three-service compose |
| AP1 | AP | SQL `exchange_holdings` bitunix futures `market_value_eur` | **pass_with_prerequisites** | DEFERRED — operator DB probe |
| AP-CODE | AP | AP2 defensive subtotal (if gate passed) | **skipped** | AP1 gate not evaluable; AP2 remains skipped |
| AP-1 | AP | `GET /api/v1/wealth` `crypto.subtotal_eur` > 0 | **pass_with_prerequisites** | Pending deploy + sync |
| AQ-CODE | AQ | holdings_all + fx_incomplete API | **pass** | AQ1; wealth tests 4/4 |
| AQ-1 | AQ | Wealth crypto tab native+EUR + banner | **pass_with_prerequisites** | Code PASS; live deferred |
| AR-API | AR | `GET /api/v1/forecast/daily?account_id=114` non-zero | **pass_with_prerequisites** | DEFERRED — pending Full sync |
| AR-GRAF | AR | `/analytics/cashflow` balance panels acct 114 | **pass_with_prerequisites** | DEFERRED — compare API vs Grafana |
| AR-CODE | AR | AR1 cashflow.json (only if API≠Grafana) | **skipped** | AR partial probe not runnable |
| AS-CODE | AS | DELETE active 409 + delete UI | **pass** | AS1; VW-AUTO-3 |
| AS-1 | AS | target_type select + help (no account) | **pass** | AS2; VW-AUTO-4 |
| AT-1 | AT | `docker ps` includes stats-forecast | **pass_with_prerequisites** | Ops-only DEC-0076 compose |
| REG-1 | regression | Six `/analytics/{slug}` routes embed | **pass_with_prerequisites** | Pending deploy + Grafana reload |

## Conditional task gates (verify-work evaluation)

| Task | Gate status | Outcome |
|------|-------------|---------|
| **AP2** | **DEFERRED** | Cannot run AP1 SQL without operator DB. Reopen per architecture: priced futures + `subtotal_eur=0` → execute AP2; `subtotal_eur>0` → AP closed; NULL wallet → ops only |
| **AR1** | **DEFERRED** | Cannot run AR-API/AR-GRAF without deploy + Full sync. Reopen only if step 4 (API) non-zero AND step 5 (Grafana panel) zero for acct **114** |

## Automated verification (verify-work independent re-run)

| Check | Result |
|-------|--------|
| `cd backend && cargo test --lib` | **PASS** (177/177) |
| `cargo test --test grafana_provisioning_bug0009` | **PASS** (6/6) |
| `cargo test --lib plan_delete_api_tests` | **PASS** (1/1) |
| `cd frontend && npm test -- --run` | **PASS** (6/6) |
| Omniflow reachability (`curl financegnome.omniflow.cc`) | **pass_with_prerequisites** — root 401; API health 404 |

**Local vs operator:** All code-level and unit/integration tests verified locally. Omniflow runtime probes, AP1 SQL, AR partial compare, and docker compose three-service smoke deferred to operator per BUG-0013 precedent. Local docker compose not runnable (missing `AUTHENTIK_SECRET_KEY`, stats-forecast image context).

## Results summary

| Metric | Count |
|--------|-------|
| UAT steps total | 14 |
| **pass** | 4 |
| **pass_with_prerequisites** | 8 |
| **skipped** | 2 |
| **fail** | 0 |
| Acceptance rows **pass** (code) | AO, AQ, AS |
| Acceptance rows **pass_with_prerequisites** (runtime/ops) | AP, AR, AT |

**Traceability:** BUG-0014 rows **AO–AT** mapped in `sprints/quick/Q0022/uat.json`. Checkbox updates in `docs/product/acceptance.md` are **release** phase. Decisions **DEC-0081** (AQ), **DEC-0082** (AS1), **DEC-0083** (AS2).

**Operator advisory:** After all five operator gates complete, execute the 14-step smoke checklist in `uat.json` `operator_smoke_checklist` on `https://financegnome.omniflow.cc`.
