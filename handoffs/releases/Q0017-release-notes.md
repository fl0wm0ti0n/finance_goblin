# Quick Release Notes — Q0017 / BUG-0007

**Quick task:** Q0017  
**Bug:** BUG-0007 — AI merchant/category discovery fails despite mirror data  
**Date:** 2026-06-08  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0007 rows S/T/U)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (150/150); `cargo test --test bug0007_ai_discovery` (8/8) @ 2026-06-08 release
2. **QA completion gate:** PASS — `sprints/quick/Q0017/qa-findings.md` (0 blockers after S-privacy re-run)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0017/verify-work-findings.md`, `handoffs/verify_work_to_release.md`; omniflow rows S/U live PASS; T partial (non-blocking)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — verify-work tuple `runtime-proof-verify-work-20260607-bug0007-q0017-002`
6. **Release finalization gate:** PASS

---

## Summary

Backend AI discovery fix per **DEC-0069** (A′+E+F bundle + S-privacy exemption) on US-0010 external profile (`financegnome.omniflow.cc`):

| Scope | Fix |
|-------|-----|
| **S** | `get_subscriptions` exposes `kind`, `merchant_names[]`; PrivacyLayer preserves `display_name` / `merchant_names[]`; orchestrator discovery rules |
| **T** | `get_transactions.category_search` ILIKE resolution (cap 10); mirror date bounds in empty-state; explicit filter evidence |
| **U** | SYSTEM_PROMPT cross-signal fusion; audit `result_rows`; parameter schema descriptions |
| **S-fix** | Privacy exemption for subscription label fields (verify-work blocker resolved) |

**Production proof:** Named merchants in subscription enumeration (YouTube, Netflix, CURSOR, APPLE.COM/BILL, …); Strom **465,53 €** via `category_search`; multi-tool fusion with named merchants; six-tool + `allow_raw_transactions=false` preserved.

---

## Run

**Target service (external profile):** `flow-finance-ai` — rebuild/recreate for Q0017 backend changes.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_DEPLOY (required before V1 smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

- `start_command`: docker compose commands above
- `runtime_mode`: remote (omniflow external US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§18 BUG-0007 hotfix)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- `service_port`: 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health`
- AI Chat: `/chat` or header **AI** nav

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite §11) |
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Mirror data gate (922+ txs) |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(S)** | AI Chat cancelable streaming → *liste mir die dienste auf* | Named merchants (not Counterparty-* or generic-only) |
| **(T)** | Strom / electricity spend query | Data-backed amount (e.g. **465,53 €**) or explicit empty-state with mirror bounds |
| **(T)** | Amazon Jan–Oct 2023 | Empty-state cites mirror date range (2025-06-05..2026-05-22) — true empty period |
| **(U)** | Multi-tool fusion without user merchant names | `get_subscriptions` + `get_transactions` fused response |
| Regression | Six tools; `allow_raw_transactions: false`; payee redaction in `get_transactions` | **PASS** |
| T1 | `cd backend && cargo test --test bug0007_ai_discovery` | **8/8 PASS** |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test bug0007_ai_discovery
```

**Live (verify-work):** S-1/S-2 named merchants; T-b Strom **465,53 €**; U-1 multi-tool fusion; REG payee redaction Counterparty-* in top payees only.

**Advisory (non-blocking):** LLM may pass `group_by: month` inflating category totals — document in operator notes; does not block acceptance.

---

## Credentials

- Traefik basic auth — operator shell / password manager
- No inline secrets in release artifacts

---

## Changes

| Area | Summary |
|------|---------|
| `backend/src/transactions/repository.rs` | A1 ILIKE category search + mirror bounds |
| `backend/src/ai/tools/transactions.rs` | A2 `category_search` param + response assembly |
| `backend/src/ai/tools/subscriptions.rs` | F1 `kind` enum, `merchant_names[]`, Counterparty-* guard |
| `backend/src/ai/orchestrator.rs` | E1 discovery rules + audit `result_rows` |
| `backend/src/ai/privacy.rs` | S-fix subscription label exemption |
| `backend/tests/bug0007_ai_discovery.rs` | T1 contract suite (8 tests) |
| Runbook | §18 BUG-0007 operator smoke |

**Linked decisions:** DEC-0069  
**Research fulfilled:** R-0065  
**Deferred:** RAG (V), payee aggregates (B), BUG-0008 coordinate-only

---

## Known Issues

- T partial: `group_by: month` advisory when LLM chooses monthly grouping (non-blocking)
- OIDC browser regression smoke deferred (API PASS per prior bug releases)
- Amazon 2023 queries correctly empty — mirror has no 2023 rows

---

## Regression scope

- BUG-0006 aggregate/category ingest unchanged
- Six-tool registry preserved
- `allow_raw_transactions=false` default unchanged
- Privacy payee redaction in `get_transactions` preserved
