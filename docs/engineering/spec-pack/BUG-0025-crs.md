# CRS — BUG-0025 Firefly Stromkosten mirror lag

## Purpose

Restore mirror fidelity for backdated Firefly category transactions and eliminate misleading sync freshness signals. Operators expect **Sync now** to pull recent ledger changes including bulk backdated imports; scheduled incremental sync remains bounded per **DEC-0002**.

## Scope

### In scope

- **B1:** `sync_transactions` manual-trigger **365d** lookback; pass `trigger` from `execute_run`
- **B2:** `SyncStatusResponse.last_firefly_run` query (`trigger IN ('manual','scheduled')`)
- **F1:** `SyncStatusPage` — hero **Last Firefly sync**, trigger badge, exchange secondary line, DEC-0002 info callout
- **F3 (P1):** `HomePage` prefer `last_firefly_run` for dashboard stat
- **D1:** `runbook.md` — backdated import symptom, **Sync now**, cursor-reset SQL
- **T1:** Integration test — tx before incremental window skipped on scheduled, ingested on manual
- **G1:** `cargo test` + `npm test` + build
- **V1:** verify-work **BW**/**BX**/**BY**; OIDC smoke

### Out of scope

- Global overlap_days TOML change
- Admin cursor-reset endpoint
- `expense-series` SQL / CategoryTrendChart changes
- Firefly direct oracle in CI (mirror assert sufficient)

## Acceptance criteria ref

`docs/product/acceptance.md` — BUG-0025:

- **(BW)** Multi-month Stromkosten bars after manual Full sync — not 2026-05-only
- **(BX)** Backdated imports ingest **or** documented DEC-0002 limitation + remediation
- **(BY)** Manual **Sync now** = Full Firefly; summary distinguishes Firefly vs exchange-only; OIDC regression pass

## Dependencies

- **DEC-0002** (sync watermark — extended for manual lookback)
- **DEC-0088** / **US-0018** (Category spending trend surface)
- [R-0097](docs/engineering/research.md#r-0097--bug-0025-firefly-category-transactions-not-updating-stromkosten)
- [R-0089](docs/engineering/research.md#r-0089--bug-0019-grafana-cashflow-zeros-account_id-default--sync-entity-counts-per-run-cursor)
