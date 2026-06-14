# Design Concept — BUG-0025

## Summary

BUG-0025 closes a mirror lag gap for backdated Firefly imports (operator repro: **Wohnen - Stromkosten** shows only **2026-05** on `/forecast` Category spending trend). Root cause is **DEC-0002** incremental window (`watermark − 7d` by transaction date) plus misleading Sync Status hero when exchange-only runs succeed. Fix combines **manual Full 365-day lookback**, **last_firefly_run** status split, and operator documentation.

## Goals

- **BW:** Multi-month category outflow bars after **Sync now** when Firefly holds rows in those months
- **BX:** Backdated imports ingest on manual Full **or** documented DEC-0002 limitation + cursor-reset remediation — not silent omission
- **BY:** Sync Status distinguishes Firefly full runs from exchange-only; hero reflects Firefly freshness
- Integration test for backdated-window skip/ingest
- OIDC-enabled deploy regression pass

## Non-goals

- Global `overlap_days` increase
- Firefly Search API / `updated_at` incremental signal
- Admin cursor-reset API (runbook SQL only)
- expense-series API or chart component changes
- New DEC record (extend **DEC-0002** in architecture)

## Key decisions

| Gate | Choice | Rationale |
|------|--------|-----------|
| GATE-OVERLAP-1 | Doc + manual **365d** lookback; scheduled unchanged | **BW** + **BX** without scheduled cost inflation |
| GATE-SYNC-UX-1 | `last_firefly_run` hero + trigger badge + exchange secondary line | Fixes **BY** without hiding exchange signal |
| GATE-REMED-1 | Runbook cursor-reset SQL | **>365d** backfill path |
| GATE-TEST-1 | Rust integration repro | Deterministic pre/post-fix assert |
| GATE-DEC-1 | Extend **DEC-0002** | Same upsert contract; trigger-specific window |

**Spec-pack traceability:** `docs/engineering/spec-pack/BUG-0025-crs.md`, `docs/engineering/spec-pack/BUG-0025-technical-specification.md`
