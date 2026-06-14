# CRS — BUG-0023 Crypto Wealth EUR values missing (live regression)

## Purpose

Close live regression on Wealth **Crypto** tab where Bitunix-connected operator sees **€0** exchange card, empty **Value EUR** column, and missing **Total return %** despite 11 linear positions and non-zero unrealized PnL.

## Scope

### In scope

- **BO:** `bitunix.rs` wallet parse — equity formula, JSON `code==0`, parse-skip logging, wiremock tests
- **BP:** `exposure_eur` column + `pnl.rs` `entryValue` display persist; `wealth/service.rs` `holdings_all.value_eur` mapping
- **BQ:** Verify baseline capture + `total_return_pct` when wallet priced (downstream of BO)
- Integration/regression tests; operator verify-work on `:18080`

### Out of scope

- Subtotal contract change or new DEC
- Mark-price API / tier-2 price book
- AP2 defensive subtotal (**BUG-0014**)
- UI copy changes beyond pass-through API fields

## Acceptance criteria ref

`docs/product/acceptance.md` — BUG-0023:

- **(BO)** Bitunix card + `crypto.subtotal_eur` ~operator portfolio — not €0 with 11 positions
- **(BP)** Value EUR populated when prices available — not all em dash
- **(BQ)** Total return % when baseline exists — not — with non-zero unrealized; OIDC regression pass

## Dependencies

- **DEC-0064**, **DEC-0080**, **DEC-0081**, **DEC-0038**
- [R-0093 §5](docs/engineering/research.md#r-0093--bug-0023-crypto-wealth-eur-values-live-regression)
- Exchange sync + PnL recompute operator gates
