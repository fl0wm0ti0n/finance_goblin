# Design Concept — BUG-0023

## Summary

BUG-0023 restores Bitunix crypto Wealth EUR display on the **Crypto** tab: exchange card subtotal (~operator portfolio), per-position **Value EUR**, and **Total return %**. Root cause is **wallet ingest silent failure** (no `futures` row), not aggregation logic. Fix under existing **DEC-0064** / **DEC-0080** contracts — **no new DEC**.

## Goals

- **BO:** Bitunix card + `crypto.subtotal_eur` reflect wallet equity (~€2000 order of magnitude) — not €0 with 11 positions
- **BP:** Holdings **Value EUR** populated from exchange `entryValue` + FX — not all em dash
- **BQ:** **Total return %** when baseline exists after priced wallet sync — not — with non-zero unrealized
- Observable wallet parse failures (logging + `code==0` validation)
- Regression tests with Bitunix OpenAPI sample shapes

## Non-goals

- Merge linear position notional into wealth subtotal (**DEC-0064** preserved)
- Tier-2 mark-price / external price book
- **BUG-0014** AP2 defensive subtotal from portfolio snapshot
- Grafana or frontend routing changes
- `holdings_count` UX footnote (P2 optional)

## Key decisions

| Gate | Choice | Rationale |
|------|--------|-----------|
| GATE-BO-1 | Wallet parse hardening | Equity keys + validation + logging + wiremock |
| GATE-BP-1 | D1 `entryValue` → `exposure_eur` display | No **DEC-0064** amend; separate from subtotal |
| GATE-AGG-1 | `sum(market_value_eur)` wallet-only | **DEC-0080** double-count guard |
| GATE-BQ-1 | Wallet-priced `crypto_value_eur` | Auto-resolves when BO fixed |
| GATE-DEC-1 | No new DEC | Subtotal contract unchanged |

**Spec-pack traceability:** `docs/engineering/spec-pack/BUG-0023-crs.md`, `docs/engineering/spec-pack/BUG-0023-technical-specification.md`
