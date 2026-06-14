# QA Findings — Quick Q0034 / BUG-0025

**Work item:** BUG-0025 (defect)  
**Quick task:** Q0034  
**QA phase:** `/qa`  
**Date:** 2026-06-14  
**Orchestrator:** `auto-20260613-bug0025`  
**Decisions:** DEC-0002 (extends; GATE-DEC-1 no new DEC)  
**QA agent:** fresh subagent (`qa-20260614-bug0025-qa-fresh`)

> Canonical segment checkpoint: `sprints/S0001/qa-findings.md`

## Verdict

**PASS** — Independent re-run confirms **BW**/**BX**/**BY** implementation under frozen architecture gates **GATE-OVERLAP-1** / **GATE-SYNC-UX-1** / **GATE-REMED-1** / **GATE-TEST-1** / **GATE-DEC-1**: manual `trigger=manual` uses **365-day** lookback; scheduled path unchanged (`watermark − overlap_days`); `GET /api/v1/sync/status` exposes `last_firefly_run` distinct from exchange-only `last_run`; Sync Status hero + DEC-0002 callout + runbook remediation documented. Automated gates **cargo lib 221/221**, **bug0025 integration 3/3**, **npm 31/31**, **build PASS**. **V1** runtime BW/BX/BY smoke on `/sync` and `/forecast` deferred to verify-work — pass-with-prerequisites (**BACKEND_REBUILD** + **FRONTEND_DEPLOY**).

**Blockers:** 0

## Scope

**Inputs reviewed:** `handoffs/dev_to_qa.md` (Q0034 top section), `sprints/quick/Q0034/{summary,progress,tasks,uat}.md`, `docs/product/acceptance.md` BUG-0025 row (BW/BX/BY), `docs/engineering/architecture.md` § BUG-0025, `backend/src/firefly/mod.rs`, `backend/src/sync/mod.rs`, `backend/tests/bug0025_sync_transaction_window.rs`, `frontend/src/lib/api.ts`, `frontend/src/pages/SyncStatusPage.tsx`, `docs/engineering/runbook.md` § Backdated Firefly imports. No host `.env`/secret files read.

## Test plan

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | Backend unit suite | `cargo test --lib` | **PASS** — 221/221 (2.02s) |
| T-2 | BUG-0025 integration | `cargo test --test bug0025_sync_transaction_window` | **PASS** — 3/3 (0.00s) |
| T-3 | Frontend unit suite | `npm test` | **PASS** — 31/31 (2.45s) |
| T-4 | Frontend build | `npm run build` | **PASS** — tsc + vite build (13.35s) |
| T-5 | GATE-OVERLAP-1 manual 365d | `manual_sync_uses_365_day_lookback_and_ingests_backdated_tx` | **PASS** — start param 365d; backdated tx ingested |
| T-6 | GATE-OVERLAP-1 scheduled unchanged | `scheduled_sync_uses_watermark_minus_overlap_start` | **PASS** — `watermark − 7d` start param |
| T-7 | GATE-TEST-1 narrow-window skip | `scheduled_sync_does_not_ingest_when_firefly_returns_empty_for_narrow_window` | **PASS** — 0 mirror rows on empty response |
| T-8 | GATE-SYNC-UX-1 API split | `SyncStatusResponse.last_firefly_run` + `latest_firefly_run()` | **PASS** — filters `trigger IN ('manual','scheduled')` |
| T-9 | GATE-SYNC-UX-1 hero UX | `SyncStatusPage.tsx` code review | **PASS** — hero uses `last_firefly_run`; exchange secondary when newer exchange-only run |
| T-10 | GATE-REMED-1 runbook | `#backdated-firefly-imports` anchor + cursor-reset SQL | **PASS** — `DELETE FROM sync_cursors WHERE entity_type = 'transactions'` documented |
| T-11 | BX callout | SyncStatusPage DEC-0002 callout + runbook link | **PASS** — 7-day overlap + Sync now 365d + runbook href |
| T-12 | BY history trigger column | Sync history table `run.trigger` | **PASS** — unchanged raw trigger column; manual/scheduled vs exchange-only distinguished |
| T-13 | User-visible metadata guard | `python3 scripts/check-user-visible-metadata.py` | **skipped** — entrypoint missing (`METADATA_SANITIZATION_POLICY_MISSING`); manual review of changed strings shows no internal metadata tokens |
| T-14 | V1 operator smoke | BACKEND_REBUILD + FRONTEND_DEPLOY + `/sync` + expense-series | **DEFERRED** — verify-work |

### T-1..T-4 output

```
cargo test --lib
  test result: ok. 221 passed; 0 failed; 0 ignored

cargo test --test bug0025_sync_transaction_window
  test result: ok. 3 passed; 0 failed; 0 ignored

npm test
  Test Files  6 passed (6)
  Tests       31 passed (31)

npm run build
  ✓ built in 13.35s
```

## Generated baseline test evidence (US-0066)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | rust + node (cargo + vitest) |
| `generated_test_command` | `cargo test --lib`; `cargo test --test bug0025_sync_transaction_window`; `npm test`; `npm run build` |
| `generated_test_result` | pass |
| `generated_test_output_ref` | T-1..T-4 output above |
| `generated_test_paths_ref` | `backend/tests/bug0025_sync_transaction_window.rs`; `frontend/src/pages/SyncStatusPage.tsx` |
| `generated_test_reason_code` | (none) |

## Code review vs decisions

| Gate / Decision | Contract | Review |
|-----------------|----------|--------|
| **GATE-OVERLAP-1** | Manual 365d lookback; scheduled `watermark − overlap_days` unchanged | **PASS** — `firefly/mod.rs` L368–383; trigger passed from `sync/mod.rs` L226–230 |
| **GATE-SYNC-UX-1** | Hero uses `last_firefly_run` only; exchange secondary when newer | **PASS** — `latest_firefly_run()` L541–551; `shouldShowExchangeSecondary` L38–49; hero L114–137 |
| **GATE-REMED-1** | Runbook cursor-reset SQL documented | **PASS** — runbook L1986–1998; anchor `#backdated-firefly-imports` resolves |
| **GATE-TEST-1** | Integration repro scheduled vs manual start window | **PASS** — 3/3 wiremock + DB cases |
| **GATE-DEC-1** | Extends DEC-0002 — no new DEC | **PASS** — callout cites DEC-0002; no new decision record |
| **DEC-0002** | Incremental overlap by transaction date; manual exception | **PASS** — callout explains 7-day overlap + manual 365d path |

## Acceptance row status (qa-stage)

| Row | qa-stage evidence | Status |
|-----|-------------------|--------|
| **BW** | Manual sync 365d start param + backdated tx ingest (T1 integration); scheduled narrow window unchanged | **PASS** at qa — live Stromkosten category **146** multi-month bars deferred V1 |
| **BX** | DEC-0002 callout on Sync Status; runbook remediation + cursor-reset SQL; manual Full path documented | **PASS** at qa — operator-visible deployed callout deferred V1 |
| **BY** | `last_firefly_run` API field; hero never shows exchange timestamp as Firefly sync; history `trigger` column preserved | **PASS** at qa — Sync now → manual in history deferred V1 |

## Task verdict matrix

| Task | Status (execute) | QA verdict | Notes |
|------|------------------|------------|-------|
| B1 | done | **PASS** | `MANUAL_LOOKBACK_DAYS=365`; trigger wiring |
| B2 | done | **PASS** | `last_firefly_run` on status response |
| F1 | done | **PASS** | Hero + trigger badge + exchange secondary + callout |
| D1 | done | **PASS** | Runbook § Backdated Firefly imports |
| T1 | done | **PASS** | 3/3 integration |
| G1 | done | **PASS** | Independent 221/221 + 3/3 + 31/31 + build PASS |
| V1 | deferred | **DEFERRED** | BACKEND_REBUILD + FRONTEND_DEPLOY required |

## Non-blocking notes (carry to verify-work)

- V1 requires operator **BACKEND_REBUILD** (B1+B2) then **FRONTEND_DEPLOY** (F1) before `/sync` hero/callout and `/forecast` Category spending trend smoke on category **146** (Stromkosten).
- Integration tests require `DATABASE_URL`; QA environment had DB available (3/3 executed, not skipped).
- Implementation files remain uncommitted per sprint policy.

## Handoff

- **Next phase:** `/verify-work` (role: qa)
- **No return items** — `handoffs/qa_to_dev.md` not written (PASS; 0 blockers)

`fresh_context_marker`: qa-20260614-bug0025-qa-fresh  
`runtime_proof_id`: runtime-proof-qa-20260614-bug0025-001  
`phase_boundary`: qa → verify-work
