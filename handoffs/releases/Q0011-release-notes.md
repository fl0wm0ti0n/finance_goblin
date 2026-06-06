# Quick Release Notes — Q0011 / BUG-0004

**Quick task:** Q0011  
**Bug:** BUG-0004 — Post-sync pipeline empty analytics (stuck exchange sync, subscriptions, Grafana SQL, wealth/forecast)  
**Date:** 2026-06-05  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0004)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (110/110), `cd frontend && npm test` (2/2), `npm run build` @ 2026-06-05 (release)
2. **QA completion gate:** PASS — `sprints/quick/Q0011/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0011/uat.md`, `handoffs/verify_work_to_release.md`; omniflow rows I/J/K/L live curl PASS (OIDC/J2 UI regression deferred)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

---

## Run

**Target services (external profile):** `flow-finance-ai` + `grafana` — rebuild/recreate for I1/K1/L1/L2/J1/J2 code; operator **Full Firefly sync** + **manual exchange sync** required for acceptance I/J/L.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana
```

**Operator prerequisite (L3 — required for I/J/K/L acceptance):**

1. Deploy Q0011 image to omniflow.
2. Trigger **Manual Full Firefly sync** — account balance backfill via DEC-0060 upsert.
3. Trigger **Manual exchange sync** — verify I1 terminal status path.

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai grafana
```

- `runtime_mode`: omniflow external (US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§ Omniflow §12 BUG-0004)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- **I:** Exchange sync terminal status via `finish_sync_run` on `RunMode::ExchangesOnly`
- **J:** Subscription detection with DEC-0061 payee key fallbacks + empty-state thresholds
- **K:** Portfolio Grafana UNION SQL fixed in provisioned dashboard
- **L:** Firefly account balance parse (DEC-0060) + wealth NULL handling; forecast series after recompute

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite from BUG-0003) |
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Full Firefly sync gate (L3) |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **I** | `GET /api/v1/sync/status` after manual exchange sync | `state: success`; `last_run.trigger: manual_exchanges`; `finished_at` set |
| **I** | `GET /api/v1/sync/runs?limit=25` | Latest `manual_exchanges` run terminal `success` |
| **J** | `GET /api/v1/subscriptions` | Non-empty pending patterns with `payee_key`, `confidence_pct` |
| **K** | `POST /analytics/grafana/api/ds/query` (portfolio pie) | **200**; no `pq: syntax error at or near "UNION"` |
| **L** | `GET /api/v1/wealth` | Firefly asset accounts populated post–Full sync |
| **L** | `GET /api/v1/forecast/daily?account_id=<id>` | **200** with populated series |
| Stack | `GET /health` | 200 |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd frontend && npm test
cd frontend && npm run build
```

**Boundaries preserved:** no Traefik routing changes; 10 historical pre-I1 stuck `scheduled_exchanges` rows out of scope; OIDC/J2 UI regression deferred to operator browser smoke.

---

## Deliverables

| Slice | Change |
|-------|--------|
| **I1** | `finish_sync_run` on `RunMode::ExchangesOnly` terminal path in `sync/mod.rs` |
| **K1** | Portfolio pie panel UNION branches wrapped in parentheses |
| **L1** | `parse_split_amount` for Firefly `current_balance` (DEC-0060) |
| **L2** | `COALESCE(balance, 0) >= 0` in `load_asset_accounts` |
| **J1** | `extract_payee_source` + DEC-0061 fallbacks in `by_payee()` |
| **J2** | Subscriptions empty-state thresholds + pending-review banner |
| **L3** | Operator verify-work — deploy + Full Firefly sync + exchange sync |

## Evidence refs

- `sprints/quick/Q0011/summary.md`, `qa-findings.md`, `uat.md`, `verify-work-findings.md`
- `handoffs/verify_work_to_release.md`, `dev_to_qa.md`
- `handoffs/tl_to_dev.md` (architecture-20260605-bug0004)

## Known issues (non-blocking)

- OIDC-enabled deploy regression: operator browser smoke recommended
- J2 threshold empty-state copy not curl-verifiable (401); API satisfies **J**
- Wealth/forecast/Grafana values may show `0.00` — structural population met; verify Firefly source balances if non-zero expected
- 10 historical pre-I1 `scheduled_exchanges` stuck rows remain — frozen out of scope

## Operator advisory

After deploy, run **Full Firefly sync** then **manual exchange sync**. Confirm rows I/J/K/L via settings API, sync status, subscriptions, portfolio ds/query, wealth, and forecast daily endpoints. Optional: browser smoke for J2 threshold copy and OIDC regression.
