# Q0034 — BUG-0025 Firefly Stromkosten mirror lag (sync overlap + status UX)

| Field | Value |
|-------|-------|
| **ID** | Q0034 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0025 |
| **Created** | 2026-06-14 |
| **Architecture** | `docs/engineering/architecture.md` § BUG-0025 (extends **DEC-0002** manual 365d lookback; **GATE-DEC-1** no new DEC) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260614-bug0025-q0034`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0025 rows **BW**, **BX**, **BY** |
| **Task count** | 7 mandatory (7/12 under `SPRINT_MAX_TASKS=12`) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0025: backdated Firefly imports (operator repro **Wohnen - Stromkosten**) skipped by **DEC-0002** incremental window (`watermark − 7d` by transaction date) while Sync Status hero misleadingly shows exchange-only success. Fix combines **manual Full 365-day lookback** (**B1**), **`last_firefly_run`** status split (**B2**/**F1**), operator runbook remediation (**D1**), integration repro (**T1**), and verify-work (**V1**).

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| **BW** — manual 365d ingest (P0) | B1, T1, G1, V1 | `firefly/mod.rs`, `sync/mod.rs`, tests |
| **BX** — transparency + remediation (P0) | B1, D1, F1, G1, V1 | runbook, SyncStatusPage callout |
| **BY** — status API + UI split (P0) | B2, F1, G1, V1 | `sync/mod.rs`, `SyncStatusPage.tsx`, `api.ts` |
| Regression + gates | G1, V1 | cargo, npm, uat |

**Ops-only (not execute tasks):** Operator **BACKEND_REBUILD** + **FRONTEND_DEPLOY** before **V1**.

**Out of scope:** Global `overlap_days` config; Firefly Search API; admin cursor-reset API; expense-series SQL; CategoryTrendChart; HomePage F3 (optional P1); new DEC.

## Task summary

| ID | Title | Est. | Depends | Acceptance | Gate |
|----|-------|------|---------|------------|------|
| B1 | Manual 365d lookback wiring | 2h | — | **BW**, **BX** | GATE-OVERLAP-1 |
| B2 | `last_firefly_run` API split | 1.5h | — | **BY** | GATE-SYNC-UX-1 |
| F1 | Sync Status hero + DEC-0002 callout | 2h | B2 | **BX**, **BY** | GATE-SYNC-UX-1 |
| D1 | Runbook backdated-import remediation | 1h | B1 | **BX** | GATE-REMED-1 |
| T1 | Integration backdated-window repro | 3h | B1 | **BW** | GATE-TEST-1 |
| G1 | Automated gate | 1h | B1, B2, F1, D1, T1 | all | — |
| V1 | verify-work BW/BX/BY + OIDC smoke | 2h | G1 + deploy | all | — |

**Total estimate:** ~12.5h dev + ~2h operator V1.

## Deploy order

```text
B1 (firefly/mod.rs — sync_transactions trigger + MANUAL_LOOKBACK_DAYS=365)
  → B2 (sync/mod.rs — SyncStatusResponse.last_firefly_run)
  → F1 (SyncStatusPage hero, badge, exchange secondary, callout)
  → D1 (runbook.md § backdated Firefly imports)
  → T1 (backend integration test)
  → G1 (cargo test + npm test + build)
  → operator: BACKEND_REBUILD + FRONTEND_DEPLOY
  → V1 verify-work
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **BW** | B1, T1, G1, V1 | Manual **Sync now** ingests multi-month Stromkosten; expense-series category **146** shows bars per month — not **2026-05** only |
| **BX** | B1, D1, F1, G1, V1 | Backdated ingest on manual Full **or** Sync Status callout + runbook cursor reset — not silent skip |
| **BY** | B2, F1, G1, V1 | Hero **Last Firefly sync** from `last_firefly_run`; history `trigger` distinguishes Firefly vs exchange runs |

## Architecture → sprint mapping

| Architecture ID | Disposition | Gate |
|-----------------|-------------|------|
| B1 | Task **B1** | GATE-OVERLAP-1 |
| B2 | Task **B2** | GATE-SYNC-UX-1 |
| F1 | Task **F1** | GATE-SYNC-UX-1 |
| D1 | Task **D1** | GATE-REMED-1 |
| T1 | Task **T1** | GATE-TEST-1 |
| G1 | Task **G1** | — |
| BW/BX/BY runtime | Task **V1** | — |
| F3 (HomePage) | **Out of scope** (optional P1) | — |

## Frozen boundaries

See `task.json` `frozen_boundaries`.

## User guide (USER_GUIDE_MODE=1)

**Waived** — operator guidance delivered via runbook (**D1**) and Sync Status callout (**F1**); no new `docs/user-guides/US-xxxx.md` workflow.
