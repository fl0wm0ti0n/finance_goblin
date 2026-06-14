# Technical Specification — BUG-0025

## Overview

Backend + frontend fix for backdated Firefly transaction ingest on manual Full sync and Sync Status UX split. Extends **DEC-0002** with a manual-trigger 365-day lookback exception; scheduled incremental path unchanged.

## Components

| Layer | Change | Gate |
|-------|--------|------|
| `backend/src/firefly/mod.rs` | `sync_transactions(..., trigger)` + frozen start contract | B1 |
| `backend/src/sync/mod.rs` | Pass trigger; `last_firefly_run` on `SyncStatusResponse` | B1, B2 |
| `frontend/src/lib/api.ts` | `SyncStatus.last_firefly_run` | B2 |
| `frontend/src/pages/SyncStatusPage.tsx` | Hero, badge, exchange line, info callout | F1 |
| `frontend/src/pages/HomePage.tsx` | Optional `last_firefly_run` stat (P1) | F3 |
| `docs/engineering/runbook.md` | Backdated import remediation | D1 |
| `backend/tests/*` | Integration backdated-window repro | T1 |

## Sync start contract (frozen)

```rust
const MANUAL_LOOKBACK_DAYS: i64 = 365;

fn transaction_start_date(watermark: Option<DateTime<Utc>>, overlap_days: i64, trigger: &str) -> String {
    let start = if trigger == "manual" {
        Utc::now() - Duration::days(MANUAL_LOOKBACK_DAYS)
    } else if let Some(w) = watermark {
        w - Duration::days(overlap_days)
    } else {
        Utc::now() - Duration::days(MANUAL_LOOKBACK_DAYS)
    };
    start.format("%Y-%m-%d").to_string()
}
```

## Sync status API (additive)

```typescript
export interface SyncStatus {
  state: string;
  phase: string | null;
  active_run_id: string | null;
  last_run: SyncRun | null;
  last_firefly_run: SyncRun | null; // NEW
}
```

**Query:** latest `sync_runs` row where `trigger IN ('manual', 'scheduled')`.

## UI behavior

| Operator view | Expected |
|---------------|----------|
| Hero on `/sync` | **Last Firefly sync:** timestamp from `last_firefly_run` |
| Trigger badge | **Manual** / **Scheduled** from `last_firefly_run.trigger` |
| Exchange newer than Firefly | Secondary **Last exchange sync:** from `last_run` when exchange-only |
| Info callout | DEC-0002 overlap explanation + runbook link |
| **Sync now** | Full Firefly + 365d lookback post-fix |

## Runbook (D1)

- Symptom: missing months on category trend after Firefly backdated import
- Fix ≤365d: **Sync now**
- Fix >365d: `DELETE FROM sync_cursors WHERE entity_type = 'transactions';` then **Sync now**

## Non-functional

- **Compatibility:** localhost `:18080`, omniflow; backend + frontend co-deploy
- **Testing:** Rust integration repro; `cargo test`; frontend build unchanged unless F3
- **Deploy:** Backend + frontend rebuild; no migration

## Traceability

- [R-0097](docs/engineering/research.md#r-0097--bug-0025-firefly-category-transactions-not-updating-stromkosten)
- `docs/engineering/architecture.md` § **BUG-0025**
- **DEC-0002** (extended — no new DEC)
