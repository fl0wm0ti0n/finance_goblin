# Quick Release Notes — Q0010 / BUG-0006

**Quick task:** Q0010  
**Bug:** BUG-0006 — AI get_transactions sees no expenses despite synced transactions  
**Date:** 2026-06-05  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0006)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (123/123), `cd frontend && npm test` (2/2), `npm run build` @ 2026-06-05 (release)
2. **QA completion gate:** PASS — `sprints/quick/Q0010/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0010/uat.json`, `handoffs/verify_work_to_release.md`; omniflow rows P/Q/R live curl PASS (OIDC browser regression deferred)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Release finalization gate:** PASS

---

## Run

**Target services (external profile):** `flow-finance-ai` — rebuild/recreate for Q1–R1 ingest + aggregate fixes; operator **Full Firefly sync backfill** required for acceptance P/Q/R.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator prerequisite (P1 — required for P/Q/R acceptance):**

1. Deploy Q0010 backend image to omniflow.
2. Reset Firefly transaction cursor if needed; trigger **Manual Full Firefly sync** — upsert backfill per DEC-0002 (no SQL migration).
3. Confirm mirror probes: non-NULL `category_id`, `date`, negative `amount` rows.

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

- `runtime_mode`: omniflow external (US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§ Omniflow)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- **P:** AI Chat answers category/spending questions via `get_transactions` for populated ledger months
- **Q:** Firefly sync persists `category_id`, ISO dates, and signed amounts (DEC-0059) on mirror rows
- **R:** Aggregate JSON includes period totals/counts and `period_status` under `allow_raw_transactions=false`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Full Firefly sync gate |
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite from BUG-0003) |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **P** | `POST /api/v1/chat/completions` (spending question, populated month) | Non-zero category/spending aggregates; not "no expenses" when mirror has rows |
| **Q** | Operator SQL on mirror `transactions` | `category_id` / `date` / `amount < 0` counts > 0 post-sync |
| **R** | AI audit `get_transactions` | Totals + empty vs populated period distinction |
| Stack | `GET /health` | 200 |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd frontend && npm test && npm run build
```

**Live (verify-work re-run 2):** sync `2ef16cfe`; operator SQL 917/919/865 of 922; May 2026 AI 75 tx / $5,692.71.

---

## Changes

| Area | Summary |
|------|---------|
| Firefly ingest | Q1 category_id from splits; Q2 ISO date parse; Q3 DEC-0059 amount sign |
| Transactions | R1 aggregate contract: totals, `period_status`, Uncategorized label |
| AI tool | `get_transactions` description passthrough for aggregate semantics |

**Linked decisions:** DEC-0059  
**Research fulfilled:** R-0060

---

## Regression scope

- Privacy redaction (`allow_raw_transactions=false`) preserved
- Six-tool registry unchanged
- OIDC-enabled deploy: deferred browser smoke (non-blocking per prior bug releases)
