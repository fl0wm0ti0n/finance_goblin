# Q0020 — BUG-0013 omniflow analytics regression (budgets MTD, crypto pricing)

| Field | Value |
|-------|-------|
| **ID** | Q0020 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0013 |
| **Created** | 2026-06-08 |
| **Architecture** | `architecture-20260608-bug0013` (`docs/engineering/architecture.md` § BUG-0013) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260608-q0020-bug0013`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0013 rows **AI**–**AN** |
| **Task count** | 5 (3 mandatory + 2 optional P2) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0013 confirmed code defects on US-0010 external omniflow: **DEC-0079** AL1 budgets MTD upper date bound (**AL1**), **DEC-0080** AN1 Bitunix wallet array parse + USDT equity + linear unrealized→EUR (**AN1**), optional UX copy (**AJ1**, **AK2**), operator verify + regression smoke (**V1**) on `financegnome.omniflow.cc` after deploy + Full sync.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| AL — Budgets MTD (DEC-0079) | AL1 | grafana `budgets.json` |
| AN/AK — Crypto valuation (DEC-0080) | AN1 | backend `bitunix.rs`, `pnl.rs` |
| Optional UX (P2) | AJ1, AK2 | grafana `subscriptions.json`, `portfolio.json` |
| Verify | V1 | uat + operator omniflow smoke |

**Out of scope:** US-0013 ML overlay; MetaMask extension noise; AM1 unless HAR non-200; AI/AJ code fixes (discovery refuted); DEC-0064 `exposure_eur` display tier 2.

## Task summary

| ID | Title | Est. | Depends | Acceptance hook | Priority |
|----|-------|------|---------|-----------------|----------|
| AL1 | MTD planned `<= CURRENT_DATE` + optional footnote | 1h | — | **AL** | P0 |
| AN1 | Wallet parse + linear unrealized EUR + tests | 4h | — | **AN**, **AK** | P0 |
| AJ1 | Price-changes empty-state copy | 0.5h | — | **AJ** | P2 optional |
| AK2 | Performance % min-snapshot footnote | 0.5h | — | **AK** | P2 optional |
| V1 | verify-work omniflow smoke | 2h | AL1, AN1 deploy | **AI**–**AN** | P0 |

**Total estimate:** ~8h (5h mandatory + 1h optional + 2h operator V1).

## Deploy order

```text
(AL1 + AN1) single release  →  BACKEND_FRONTEND_DEPLOY + GRAFANA_PROVISIONING_RELOAD
                              →  Full Firefly sync + forecast recompute
                              →  optional AJ1 + AK2
                              →  V1 verify-work
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **AI** | V1 | Re-smoke baseline panels acct 114 — refuted code regression |
| **AJ** | AJ1, V1 | Documented empty-state when 0 price-change events |
| **AK** | AN1, AK2, V1 | Crypto stat > 0; FX warning documented; performance % footnote |
| **AL** | AL1, V1 | MTD plan/actual/deviation plausible — capped MTD |
| **AM** | V1 | ds/query + annotations 200 — waived per R-0077 |
| **AN** | AN1, V1 | Wealth/portfolio crypto totals after sync |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| AL1 | Task **AL1** |
| AN1a/b/c | Task **AN1** (combined) |
| AJ1 | Task **AJ1** (optional) |
| AK2 | Task **AK2** (optional) |
| V1 | Task **V1** |
| AM1 | **Waived** per R-0077 |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
