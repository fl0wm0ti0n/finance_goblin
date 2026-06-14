# Sprint Release Notes — S0020

**Sprint:** S0020  
**Date:** 2026-06-13  
**Stories:** US-0021  
**Queue status:** released  
**Orchestrator:** `auto-20260613-us0021`  
**Decisions:** DEC-0112, DEC-0113, DEC-0114 (extends DEC-0098, DEC-0099, DEC-0111)

---

## Gate results

1. **Check-in test gate:** PASS — `cargo test --lib` 221/221; `cargo test --test us0021_transaction_search` 6/6; `npm test` 17/17; `npm run build` PASS @ verify-work
2. **QA completion gate:** PASS — `sprints/S0020/qa-findings.md` (AC-1..AC-5 verified; 0 blocking findings)
3. **UAT completeness gate:** PASS-with-prerequisites — `sprints/S0020/uat.md`, `sprints/S0020/uat.json`; 1 pass / 5 pass-with-prerequisites / 0 fail; live tx-search 404 + `/subscriptions` 404 pre-deploy
4. **Isolation compliance gate:** PASS — intake through verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — verify-work tuple `runtime-proof-verify-work-20260613-us0021-001`; release tuple at finalization
6. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai`
- `runtime_mode`: remote (omniflow external US-0010); backend + frontend delta (tx-search API, dual-mode Discover UI)
- `runtime_context_ref`: `docs/engineering/runbook.md` § 37 US-0021; `docs/user-guides/US-0021.md`

**Profile rule:** use **`external` only** — do not combine with `minimal`, `standard`, `full`, or `bundled-firefly`.

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-1..AC-4 and AC-6 live smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

Confirm tests pass before docker build:

```bash
cd backend && cargo test --lib && cargo test --test us0021_transaction_search
cd frontend && npm test && npm run build
```

No migration. Backend + frontend both change.

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (Traefik + basic-auth) / `http://localhost:18080` (local)
- `service_port`: 443 (HTTPS) / 18080 (local)
- `health_endpoint`: `GET /health`
- Subscription surfaces: `/subscriptions` Discover tab — **Transactions** (default) | **Suggested patterns**
- New API: `GET /api/v1/subscriptions/transactions/search`, `POST /api/v1/subscriptions/transactions/preview-group`

## Verify

- `verification_steps`:
  1. Deploy backend + frontend per operator gate above.
  2. Confirm app health: `curl -sf https://financegnome.omniflow.cc/health`.
  3. **AC-1:** `/subscriptions` Discover → **Transactions** mode lists individual expense rows (paginated 100/page) — not candidate-only.
  4. **AC-2:** Rich filters: account, payee, category (`CategoryFilter`), Geldbereich (`account_role`), date range.
  5. **AC-3:** Hint badges on filtered subset (account **114**, payee **SEPA-Lastschrift** — 11 txs @ 31d/95% fixture).
  6. **AC-4:** Multi-select ≥2 txs → **Activate selected** → preview-group modal → confirm subscription/standing order (DEC-0085 merge).
  7. **AC-5:** **Suggested patterns** sub-tab unchanged; US-0020 tags/majority; US-0003/US-0008 regression (`reg_discover_candidate_pass_unchanged_ac5` PASS).
  8. **AC-6:** OIDC external profile smoke on Discover tx search + confirm flow.
  9. Automated: `cd backend && cargo test --lib` (221/221); `cargo test --test us0021_transaction_search` (6/6); `cd frontend && npm test` (17/17).
- `expected_health_signal`: backend `/health` OK; tx-search API 200; Discover Transactions mode renders; Patterns tab preserved

## Credentials

- `credential_source_refs` (env names only):
  - US-0010 block: `COMPOSE_FILE`, `COMPOSE_PROFILES`, `DATABASE_*`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, `TRAEFIK_HOST`
- `expected_value_source`: operator `.env` at repo root (from `.env.example`)

## Known Issues

- Live AC-1..AC-4 and AC-6 browser probes **PENDING** operator post-deploy — pass-with-prerequisites at release (`BACKEND_FRONTEND_DEPLOY`).
- V1 operator smoke deferred; account 114 fixture requires deployed stack.
- Integration tests require operator `DATABASE_URL` (TimescaleDB-enabled PostgreSQL) (carry-forward).

## Deliverables (US-0021)

| Slice | Tasks | Outcome |
|-------|-------|---------|
| Tx-search API | TX1–TX3 | `GET /api/v1/subscriptions/transactions/search` + `POST /transactions/preview-group` (DEC-0112) |
| Dual-mode Discover UI | UI1–UI4 | Transactions default \| Suggested patterns segmented control (DEC-0113) |
| Patterns sub-tab | PT1 | US-0020 discover candidates unchanged (DEC-0098) |
| Hint pass | TX2/UI3 | Row metadata only on filtered subset; no auto-emit (DEC-0114) |
| Integration + regression | T1–T2 | `us0021_transaction_search.rs` 6/6; AC-5 `run_discover` regression |
| User guide | R1 | `docs/user-guides/US-0021.md` |
| Operator smoke | V1 | Deferred — **BACKEND_FRONTEND_DEPLOY** |

**Key files:** `backend/src/subscriptions/transaction_search.rs`, `backend/src/subscriptions/repository.rs`, `backend/src/api/subscriptions.rs`, `backend/tests/us0021_transaction_search.rs`, `frontend/src/pages/SubscriptionsPage.tsx`, `frontend/src/lib/api.ts`

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0020 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.21.0-us0021`

## Milestone

**US-0021 released** — Subscription transaction explorer: dual-mode Discover (Transactions default + Suggested patterns), rich filters, hint badges on filtered subset, multi-select manual activate via preview-group (DEC-0112..DEC-0114). **Intake bundle story drain complete** — open stories queue empty.
