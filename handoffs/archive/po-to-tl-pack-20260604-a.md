# PO to TL archive pack (2026-06-04)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 7
- First archived heading: `## intake-20260605-bug0004 — BUG-0004 post-sync pipeline empty analytics`
- Last archived heading: `## discovery-20260605-bug0003 — BUG-0003 omniflow production API 500 / Bitunix / Grafana SQL`
- Verification tuple (mandatory):
  - archived_body_lines=110
  - retained_body_lines=485

---

## intake-20260605-bug0004 — BUG-0004 post-sync pipeline empty analytics

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Work item:** BUG-0004 (defect)  
**Next phase:** `/discovery`

### Summary

Operator report on `financegnome.omniflow.cc` after BUG-0003 stack recovery: **922 transactions** synced successfully, but product analytics appear empty and exchange-only sync UI stuck **running**. Single bug **BUG-0004** with sub-defects **I–L**; do not merge with BUG-0002 (Firefly PAT) or BUG-0003 (DB host / Bitunix / datasource).

| Sub | Symptom | Intake hypothesis |
|-----|---------|-------------------|
| **I** | `manual_exchanges` sync status stuck `running`, `finished_at: null` | `RunMode::ExchangesOnly` never calls `finish_sync_run` — only Full Firefly path does (`backend/src/sync/mod.rs` L236–242 vs L315–330) |
| **J** | Subscriptions empty despite 10+ expected | Detection keys on `payee_key(description)`; ≥3 txs + interval + confidence gates; payee field mismatch possible |
| **K** | Grafana **500** `syntax error at or near "UNION"` | `portfolio.json` L80 invalid `UNION ALL` with per-branch `ORDER BY LIMIT` |
| **L** | Forecast / wealth / Grafana mostly empty | Forecast rows or `net_worth_snapshots` missing; dashboard `account_id` binding; wealth phase gated on `forecast_id` |

### Out of scope

- Wallet extension `contentscript.js` console noise
- `GET /api/v1/plans/active/plan-vs-actual` **404** when no active plan (expected empty-state)

### Intake evidence

- `intake_run_id`: `intake-20260605-omniflow-post-sync-pipeline`
- `selected_pack`: `small-intake-pack`
- Bundle: `handoffs/intake_evidence/intake-20260605-omniflow-post-sync-pipeline.json`
- Validation: `[INTAKE_EVIDENCE_VALIDATION_OK]`

### Discovery guidance

1. **I:** Trace `ExchangesOnly` / `manual_exchanges` through `execute_run` → confirm missing `finish_sync_run`; fix should mirror Full path terminal status without re-running Firefly.
2. **J:** Sample synced transaction rows — compare Firefly payee vs description fields; tune detection or document thresholds.
3. **K:** Fix portfolio SQL (subquery wrap or CTE); scan other dashboards for similar UNION patterns (`budgets.json`).
4. **L:** Verify `forecast_balance_daily`, `net_worth_snapshots`, subscription_patterns row counts post full sync; confirm Grafana template variables.

### Related

- **BUG-0003** OPEN — verify-work may PASS after operator recovery; distinct from BUG-0004 pipeline defects
- **BUG-0002** OPEN — Firefly sync verify separate

---

## discovery-20260605-bug0003 — BUG-0003 omniflow production API 500 / Bitunix / Grafana SQL

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Work item:** BUG-0003 (defect)  
**Next phase:** `/architecture`

### Summary

Discovery **confirms** intake hypotheses for sub-defects **F**, **G**, and **H** on `financegnome.omniflow.cc` (US-0010 external). **Do not merge with BUG-0002** (separate OPEN track: Firefly PAT / risk-score / settings).

| Sub | Confirmed root cause | Fix tasks |
|-----|---------------------|-----------|
| **F** | `DATABASE_HOST=host.docker.internal` in `flow-finance-ai` and `grafana` containers overrides external overlay default `postgres` → DB pool timeout ~30s → widespread **500** | **F1** Operator: `DATABASE_HOST=postgres`, recreate services. **F2** Architecture: external-profile env guard in runbook / `.env.example` (DEC-0056 / R-0052) |
| **G** | `ExchangeService::new` gates on TOML `bitunix.enabled` (`false` in `default.toml`) not `effective_enabled()` → **400** unknown exchange despite credentials + settings `enabled:true` | **G1** Register connectors with `effective_enabled()`. **G2** If still failing: R-0058 futures header auth on `fapi.bitunix.com` |
| **H** | Grafana datasource `${DATABASE_HOST}` shares F misconfig; duplicate dashboard UIDs → provisioning write blocked (secondary) | **H1** Same as F1. **H2** Optional UID dedupe in provisioning |

### Runtime proof (2026-06-05)

| Probe | HTTP | Latency | Notes |
|-------|------|---------|-------|
| `GET /api/v1/settings` | 200 | ~0.08s | `database_host: host.docker.internal`, `database_mode: external` |
| `GET /api/v1/alerts/unread-count` | 500 | ~30.07s | DB timeout pattern |
| `GET /api/v1/sync/entities` | 500 | ~30.12s | |
| `GET /api/v1/sync/runs` | 500 | ~30.06s | |
| `GET /api/v1/exchanges` | 500 | ~30.06s | |
| `GET /api/v1/subscriptions` | 500 | ~30.06s | |
| `GET /api/v1/ai/audit` | 500 | ~30.06s | |
| `POST /api/v1/exchanges/bitunix/test` | 400 | &lt;0.2s | Empty body — registry gap, not DB timeout |
| `POST .../analytics/grafana/api/ds/query` | 400 | ~0.36s | `db query error` on `SELECT 1` |

Container env (names only, no values): `DATABASE_HOST=host.docker.internal` on `finance_goblin-flow-finance-ai-1` and `finance_goblin-grafana-1`; `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` present on backend.

Backend logs: repeated `response failed` **500** `latency=30001 ms`. Grafana logs: duplicate UID warnings + `no database write permissions because of duplicates`.

`isolation_scope`: artifact + repo code + public HTTPS curl + docker logs/env names-only; **no** operator `.env` / `.env_prod` read.

### Code anchors

| Area | Files |
|------|-------|
| F | `docker-compose.external.yml` (`DATABASE_HOST:-postgres`), `.env.example` L7–8, `backend` DB pool |
| G | `backend/src/exchanges/service.rs` (`new` L40–48 vs `mirror_enabled_at_startup` L64–69), `backend/config/default.toml` `[exchanges.bitunix] enabled=false`, `backend/src/exchanges/bitunix.rs` spot `openapi.bitunix.com` query-sign |
| H | `grafana/provisioning/datasources/postgres.yaml`, dashboard provisioning providers |

### Architecture guidance

1. **F2:** Document external-profile invariant `DATABASE_HOST=postgres` on `traefik` network; warn against copying greenfield `.env.example` default into omniflow deploy.
2. **G1:** Single contract — connector registration uses same `effective_enabled()` as `settings_view` / `mirror_enabled_at_startup` (Q0008 E1 partial — gap in `new()` only).
3. **G2:** Defer unless G1+F1 smoke still returns auth failure with body; follow [R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation).
4. **H:** Treat H1 as F1 verification step in acceptance row H; scope H2 only if operator needs provisioning refresh after UID fix.
5. **Sprint shape:** Recommend **`/quick`** sprint (3–4 tasks F1/F2/G1/G2) after architecture unless TL prefers bundled execute with BUG-0002 (keep separate).

### Research refs

- [R-0052](docs/engineering/research.md#r-0052--external-compose-integration-on-omniflow-traefik-host)
- [R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation)

### Related

- **BUG-0002** OPEN — do not block BUG-0003 architecture on PAT verify

---

