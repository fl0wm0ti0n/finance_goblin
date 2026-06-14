# Q0030 — BUG-0023 Crypto Wealth EUR values missing (live regression)

| Field | Value |
|-------|-------|
| **ID** | Q0030 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0023 |
| **Created** | 2026-06-12 |
| **Architecture** | `docs/engineering/architecture.md` § BUG-0023 (extends DEC-0064, DEC-0080, DEC-0081, DEC-0038) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260612-q0030-bug0023`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0023 rows **BO**, **BP**, **BQ** |
| **Task count** | 9 mandatory (9/12 under `SPRINT_MAX_TASKS=12`) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0023 live regression on Bitunix-connected Wealth **Crypto** tab: restore
futures wallet ingest and observability (**BO1**–**BO3**); add display-only linear
`exposure_eur` from `entryValue` without amending **DEC-0064** subtotal contract
(**BP1**, **BP2**); verify baseline + `total_return_pct` when wallet priced (**BQ1**);
integration tests + automated gate + operator verify-work (**T1**, **G1**, **V1**).

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| BO — wallet ingest + observability (P0) | BO1, BO2, BO3 | `backend/src/exchanges/bitunix.rs` |
| BP — linear Value EUR display (P1) | BP1, BP2 | `migrations/`, `repository.rs`, `pnl.rs`, `wealth/service.rs` |
| BQ — total return verify (P1) | BQ1 | `portfolio/service.rs`, tests |
| Regression + gates | T1, G1, V1 | `backend/tests/`, progress, uat |

**Ops-only (not execute tasks):** Operator **BACKEND_DEPLOY** (migration 017),
**EXCHANGE_SYNC** (Bitunix full sync), **PNL_RECOMPUTE** (post-sync scheduler path),
optional **AP1_SQL_PROBE** per architecture § BUG-0023.

**Out of scope:** Tier-2 `ExchangePriceBook` / mark-price feed; merge linear notional
into subtotal; **BUG-0014** AP2 defensive subtotal; Grafana panel edits;
`holdings_count` UX footnote (P2 optional).

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| BO1 | Equity + unrealized key parse fix | 2h | — | **BO** | P0 |
| BO2 | `code==0` validation + parse-skip logging | 1h | BO1 | **BO** | P0 |
| BO3 | OpenAPI wiremock + unit tests | 2h | BO2 | **BO** | P0 |
| BP1 | `exposure_eur` migration + pnl `entryValue` persist | 2.5h | BO3 | **BP** | P1 |
| BP2 | `holdings_all.value_eur` from `exposure_eur` | 1h | BP1 | **BP** | P1 |
| BQ1 | Baseline + `total_return_pct` integration verify | 1.5h | BP2 | **BQ** | P1 |
| T1 | Regression tests BO/BP/BQ | 2.5h | BQ1 | **BO**, **BP**, **BQ** | P0 |
| G1 | Automated gate (cargo test + npm build) | 0.5h | T1 | **BO**, **BP**, **BQ** | P0 |
| V1 | verify-work operator smoke | 2h | G1 + deploy | **BO**, **BP**, **BQ** | P0 |

**Total estimate:** ~15.5h dev + ~2h operator V1.

## Deploy order

```text
BO1 → BO2 → BO3 (bitunix.rs wallet hardening)
  → BP1 (migration 017 + pnl exposure_eur persist)
  → BP2 (wealth/service.rs value_eur map)
  → BQ1 (baseline + total_return verify)
  → T1 → G1
  → operator: BACKEND_DEPLOY (includes migration 017)
  → operator: EXCHANGE_SYNC (Bitunix)
  → operator: PNL_RECOMPUTE
  → V1 verify-work
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **BO** | BO1, BO2, BO3, T1, G1, V1 | `futures` wallet row priced; `crypto.subtotal_eur` ~€2000 order of magnitude; Bitunix card not €0 |
| **BP** | BP1, BP2, T1, G1, V1 | Linear `holdings_all[].value_eur` non-null from `exposure_eur`; subtotal unchanged (wallet-only) |
| **BQ** | BQ1, T1, G1, V1 | `pnl.total_return_pct` non-null when baseline captured and wallet priced; OIDC smoke |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| BO1 | Task **BO1** |
| BO2 | Task **BO2** |
| BO3 | Task **BO3** |
| BP1 | Task **BP1** |
| BP2 | Task **BP2** |
| BQ1 | Task **BQ1** |
| T1 | Task **T1** |
| G1 | Task **G1** |
| BO/BP/BQ runtime gates | Task **V1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.

## User guide (USER_GUIDE_MODE=1)

**Waived** — bug regression fix under existing Wealth Crypto UX; no new operator
workflow. Release note for `entryValue` vs mark-to-market gap documented in V1 if
operator questions display.
