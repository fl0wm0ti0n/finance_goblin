# Q0034 Summary — BUG-0025 Firefly Stromkosten mirror lag

**Sprint:** Q0034 (`/quick`)  
**Bug:** BUG-0025  
**Orchestrator:** `auto-20260613-bug0025`  
**Phase:** execute **COMPLETE**  
**Date:** 2026-06-14

## Goal

Close BUG-0025: backdated Firefly imports skipped by **DEC-0002** incremental window while Sync Status hero misleadingly shows exchange-only success. Fix combines manual Full **365-day lookback** (**B1**), **`last_firefly_run`** status split (**B2**/**F1**), operator runbook remediation (**D1**), integration repro (**T1**), and automated gate (**G1**). Extends **DEC-0002** — no new DEC.

## Tasks

| ID | Title | Status | Acceptance | Priority |
|----|-------|--------|------------|----------|
| B1 | Manual 365d lookback wiring | **done** | **BW**, **BX** | P0 |
| B2 | `last_firefly_run` API split | **done** | **BY** | P0 |
| F1 | SyncStatusPage hero + DEC-0002 callout | **done** | **BX**, **BY** | P0 |
| D1 | Runbook remediation | **done** | **BX** | P0 |
| T1 | Integration backdated-window repro | **done** | **BW** | P0 |
| G1 | Automated gate | **done** | all | P0 |
| V1 | verify-work BW/BX/BY + OIDC smoke | **deferred** | all | P0 |

**Task count:** 7 mandatory (7/12 under `SPRINT_MAX_TASKS=12`; no split).

## Deliverables

- `backend/src/firefly/mod.rs` — `sync_transactions(..., trigger)` + `MANUAL_LOOKBACK_DAYS=365`
- `backend/src/sync/mod.rs` — pass trigger; `SyncStatusResponse.last_firefly_run` + `latest_firefly_run()`
- `frontend/src/lib/api.ts` — `last_firefly_run: SyncRun | null`
- `frontend/src/pages/SyncStatusPage.tsx` — Last Firefly sync hero, trigger badge, exchange secondary, DEC-0002 callout
- `docs/engineering/runbook.md` — § Backdated Firefly imports (`#backdated-firefly-imports`)
- `backend/tests/bug0025_sync_transaction_window.rs` — 3 integration cases

## Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **BW** | B1, T1, G1, V1 | Manual Sync now → multi-month Stromkosten; expense-series category **146** bars per month |
| **BX** | B1, D1, F1, G1, V1 | Manual Full ingest or callout + runbook DEC-0002 + cursor reset |
| **BY** | B2, F1, G1, V1 | `last_firefly_run` hero; exchange secondary; Sync now → manual in history |

## Frozen boundaries

- **GATE-OVERLAP-1:** manual 365d; scheduled `watermark − overlap_days` unchanged
- **GATE-SYNC-UX-1:** hero uses `last_firefly_run` only; exchange secondary when newer
- **GATE-REMED-1:** runbook cursor-reset SQL documented
- **GATE-TEST-1:** integration repro for scheduled vs manual start window
- **GATE-DEC-1:** extends **DEC-0002** — no new DEC

## Operator gates (before V1)

1. **BACKEND_REBUILD** — B1 + B2 live
2. **FRONTEND_DEPLOY** — F1 Sync Status UX live

## Next phase

**`/qa`** — fresh subagent (role: qa).
