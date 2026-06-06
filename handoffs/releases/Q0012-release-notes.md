# Quick Release Notes — Q0012 / BUG-0005

**Quick task:** Q0012  
**Bug:** BUG-0005 — Exchange sync spot-only (Bitunix futures / multi-product accounts)  
**Date:** 2026-06-05  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0005)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (123/123), `cd frontend && npm test` (2/2), `npm run build` @ 2026-06-05 (release)
2. **QA completion gate:** PASS — `sprints/quick/Q0012/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0012/uat.json`, `handoffs/verify_work_to_release.md`; omniflow rows M/N/O live curl PASS (OIDC/browser regression deferred)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

---

## Run

**Target services (external profile):** `flow-finance-ai` — rebuild/recreate for N1/M1/N2/N3/N4 code; operator **manual exchange sync** required for acceptance M/N/O.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator prerequisite (O1 — required for M/N/O acceptance):**

1. Deploy Q0012 backend image to omniflow.
2. Ensure `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` configured (names only).
3. Trigger **Manual exchange sync** — verify futures-enabled ingestion path.

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

- `runtime_mode`: omniflow external (US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§ Omniflow §13 BUG-0005)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- **M:** Bitunix futures wallet + linear positions ingested with non-spot `product_type` rows
- **N:** Futures REST on `fapi.bitunix.com` with header auth (DEC-0062); `effective_enabled_futures()` (DEC-0063); dual-path test (N4)
- **O:** Wealth crypto aggregates combined spot + futures holdings (DEC-0064 wallet vs position pricing)

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` | Bitunix read-only credentials |
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite from §11) |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **M** | `GET /api/v1/exchanges` after manual exchange sync | Bitunix `holdings` > 0; `exchange_bitunix` entity count > 0 |
| **N** | `GET /api/v1/settings` | `futures_base_url: https://fapi.bitunix.com`; `enabled_futures: true` |
| **N** | `POST /api/v1/exchanges/bitunix/test` | **200** — `Spot: OK; Futures: OK` |
| **O** | `GET /api/v1/wealth` | `crypto.holdings_count` > 0; `crypto_placeholder: false` |
| Stack | `GET /health` | 200 |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd frontend && npm test
cd frontend && npm run build
```

**Boundaries preserved:** read-only key constraint; no Traefik routing changes; `crypto.subtotal_eur: 0.0` with unpriced linear positions acceptable per DEC-0064; OIDC/browser regression deferred to operator smoke.

---

## Deliverables

| Slice | Change |
|-------|--------|
| **N1** | Futures header-auth client + `futures_base_url` (DEC-0062) |
| **N3** | `effective_enabled_futures()` policy + settings exposure (DEC-0063) |
| **M1** | Futures wallet balance ingestion (`product_type: futures`) |
| **N2** | `sync_positions` via `get_pending_positions` (`product_type: linear`, DEC-0064) |
| **N4** | Dual-path `test_connection` (spot + futures sub-status) |
| **O1** | verify-work omniflow rows M/N/O — exchange sync `f0906348` success |

## Evidence refs

- `sprints/quick/Q0012/summary.md`, `qa-findings.md`, `uat.md`, `verify-work-findings.md`
- `handoffs/verify_work_to_release.md`, `dev_to_qa.md`
- `handoffs/tl_to_dev.md` (architecture-20260605-bug0005)
- `decisions/DEC-0062.md`, `DEC-0063.md`, `DEC-0064.md`

## Known issues (non-blocking)

- OIDC-enabled deploy regression: operator browser smoke recommended
- `crypto.subtotal_eur: 0.0` with 4 holdings — DEC-0064 expected when linear positions unpriced; verify wallet USDT if non-zero expected
- 10+ historical pre-I1 stuck `scheduled_exchanges` rows remain — frozen out of scope

## Operator advisory

After deploy, run **manual exchange sync**. Confirm rows M/N/O via settings API, Bitunix test endpoint, exchanges holdings, sync entities, and wealth crypto endpoints. Optional: browser smoke for OIDC regression.
