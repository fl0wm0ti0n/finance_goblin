# Verify-work Findings — Quick Q0010 / BUG-0006 (re-run 2)

**Work item:** BUG-0006 (defect)  
**Quick task:** Q0010  
**Phase:** `/verify-work` (re-run after operator backfill)  
**Orchestrator:** `auto-20260605-bug0006-002`  
**Date:** 2026-06-05  
**Verdict:** **PASS** — rows **(P)/(Q)/(R)** evidenced on production; proceed to `/release`

## Summary

Re-ran verify-work after operator cursor reset + full Firefly sync **`2ef16cfe`** (`finished_at`: **2026-06-05T15:41:20Z**). Operator mirror SQL attestation: **922** total, **917** `category_id`, **919** `date`, **865** negative amounts. Local gates **PASS** (`cargo test --lib` 123/123, vitest 2/2, build PASS). Live omniflow smoke confirms Q1–R1 ingest + aggregate fixes within the operator ledger window (~June 2025–May 2026). AI Chat + `get_transactions` return non-zero category/spending aggregates for populated months; empty responses for pre-ledger months (Oct 2023, Jan–May 2025) are correct (`no_rows`), not ingest regressions.

## Per-row verdict (acceptance P / Q / R)

| Row | Verdict | Summary |
|-----|---------|---------|
| **(P)** | **PASS** | May 2026 (75 tx, $5,692.71), Dec 2025 (76 tx, $5,970.42), Jan 2026 (70 tx, $3,820.53), Jun 2025 (64 tx, $3,871.34) — AI answers with aggregate spending data via `get_transactions`. Oct 2023 / May 2025 return “no transactions” because mirror has **no dated rows** in those periods (ledger starts ~Jun 2025); not a false “no expenses” when data exists. |
| **(Q)** | **PASS** | Operator SQL post-sync: 917/922 `category_id`, 919/922 `date`, 865/922 `amount < 0`. Live category aggregates for populated months imply non-NULL category mapping on ingest path. Minor residual gaps (5 without category_id, 3 without date) within acceptable MVP tolerance. |
| **(R)** | **PASS** | Populated periods return explicit totals/counts via AI interpretation of aggregate JSON (`allow_raw_transactions=false`). Empty pre-ledger periods correctly distinguish `no_rows` (zero tx, $0) vs populated `with_outflow` periods. Privacy redaction active (`redact_counterparties: true`). |

**Release:** Check BUG-0006 acceptance checkbox during `/release`.

## Deploy / sync gate status

| Gate | Prior run | Re-run 2 |
|------|-----------|----------|
| `OPERATOR_DEPLOY_PENDING` | CLEARED | **CLEARED** |
| `OPERATOR_SYNC_PENDING` | CLEARED | **CLEARED** — run `2ef16cfe` success 2026-06-05T15:41:20Z |
| `OPERATOR_FULL_FIREFLY_BACKFILL_PENDING` | OPEN | **CLEARED** — cursor reset + full sync; operator SQL 917/919/865 |

## Operator mirror SQL (attested, names only)

| Probe | Count | Expectation | Verdict |
|-------|-------|-------------|---------|
| `COUNT(*)` | 922 | ≥922 | **PASS** |
| `category_id IS NOT NULL` | 917 | >0, ≈mirror | **PASS** |
| `date IS NOT NULL` | 919 | ≈mirror | **PASS** |
| `amount < 0` | 865 | >0 | **PASS** |

## Automated verification

| Check | Result |
|-------|--------|
| `cd backend && cargo test --lib` | **PASS** (123/123) |
| `cd frontend && npm test` | **PASS** (2/2) |
| `cd frontend && npm run build` | **PASS** |
| Code path Q1–Q3 + R1 | **PASS** (unchanged from QA) |
| Rows P/Q/R runtime | **PASS** |

## Live curl evidence (2026-06-05 re-run 2, no Traefik credentials)

| Endpoint / probe | HTTP | Notes |
|------------------|------|-------|
| `GET /health` | 200 | Stack reachable |
| `GET /api/v1/settings` | 200 | `database_mode: external`, `allow_raw_transactions: false`, `redact_counterparties: true` |
| `GET /api/v1/sync/entities` | 200 | `transactions.count: 922`, `categories.count: 75` |
| `GET /api/v1/sync/status` | 200 | `last_run.id: 2ef16cfe`, `finished_at: 2026-06-05T15:41:20Z`, `status: success`, `trigger: manual` |
| `POST /api/v1/chat/completions` May 2026 | 200 | **PASS P/R** — 75 tx, $5,692.71 outflow, top buckets |
| `POST /api/v1/chat/completions` Dec 2025 | 200 | **PASS P/R** — 76 tx, $5,970.42 |
| `POST /api/v1/chat/completions` Jan 2026 | 200 | **PASS P/R** — 70 tx, $3,820.53 |
| `POST /api/v1/chat/completions` Jun 2025 | 200 | **PASS P/R** — 64 tx, $3,871.34 (ledger start month) |
| `POST /api/v1/chat/completions` May 2025 | 200 | **PASS P/R** — 0 tx (no mirror rows; correct `no_rows`) |
| `POST /api/v1/chat/completions` Oct 2023 | 200 | **PASS P/R** — 0 tx (pre-ledger; correct `no_rows`) |
| `GET /api/v1/ai/audit` | 200 | `get_transactions` calls for populated + empty periods; `result_status: ok` |
| `GET /api/v1/subscriptions` | 200 | 11 pending patterns — regression OK |

## Acceptance impact

| Row | Verify-work re-run 2 | `acceptance.md` checkbox |
|-----|---------------------|--------------------------|
| **(P)** | **PASS** | Release phase |
| **(Q)** | **PASS** | Release phase |
| **(R)** | **PASS** | Release phase |
| Regression | **PARTIAL** — privacy + six-tool PASS; OIDC deferred (`oidc_issuer_url` empty, dev-bypass profile) | N/A |

## Release gate

| Gate | Status |
|------|--------|
| QA PASS | yes |
| Verify-work PASS | **yes** |
| Acceptance checked | release phase |
| Release proceed | **yes** |

## Operator advisory (non-blocking)

1. **365-day fetch window:** NULL cursor sync fetches Firefly transactions from `now - 365 days` only (DEC-0002). Operator ledger spans ~Jun 2025–May 2026; months before Jun 2025 correctly empty.
2. **Residual NULL rows:** 3 without `date`, 5 without `category_id` — monitor on next sync; not blocking MVP.
3. OIDC browser smoke deferred (non-blocking).

## Next steps

1. **`/release`** — check BUG-0006 acceptance; publish release notes
2. Operator browser smoke (OIDC) — advisory only

---

**Stop here.** Continue in a **new** subagent/chat with **`/release`**.
