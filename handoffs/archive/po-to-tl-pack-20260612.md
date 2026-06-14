# PO to TL archive pack (2026-06-12)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 9
- Retained units in hot file: 40
- First archived heading: `## Architecture summary`
- Last archived heading: `## Key findings`
- Verification tuple (mandatory):
  - archived_body_lines=63
  - retained_body_lines=357

---

## Architecture summary

Research verdicts **frozen**. Root cause: **wallet ingest silent failure** → no futures row → subtotal €0 → Value EUR null → total return blocked. Fix stack preserves **DEC-0064** / **DEC-0080** subtotal contract — **no new DEC** (GATE-DEC-1 closed).

## Gates resolved

| Gate | Decision |
|------|----------|
| **GATE-BO-1** | Wallet hardening — equity keys (`crossUnrealizedPNL`/`isolationUnrealizedPNL`), `code==0`, parse-skip logging, OpenAPI wiremock |
| **GATE-BP-1** | D1 — `entryValue` → `exposure_eur` display-only; **not** merged into subtotal |
| **GATE-AGG-1** | Subtotal = `sum(market_value_eur)` wallet-only; linear excluded |
| **GATE-BQ-1** | Return denominator = wallet-priced `crypto_value_eur`; auto-resolves when BO fixed |
| **GATE-DEC-1** | **No new DEC** — contract unchanged |

## Fix stack (P0→P1)

1. **BO (P0):** `bitunix.rs` wallet parse + sync observability → `recompute_pnl` prices futures row
2. **BP (P1):** Migration `exposure_eur` + `pnl.rs` `entryValue` persist + `wealth/service.rs` map to `holdings_all.value_eur`
3. **BQ (P1):** Verify baseline + `total_return_pct` when wallet priced (no separate logic slice)
4. **Rejected:** AP2 defensive subtotal; linear notional in subtotal; tier-2 mark-price

## Sprint sizing hint

9 mandatory tasks (BO1–BO3, BP1–BP2, BQ1, T1, G1, V1) ≤ `SPRINT_MAX_TASKS` (12) → **`/quick`** recommended.

## Acceptance rows (unchanged)

- **(BO)** Bitunix card + `crypto.subtotal_eur` ~operator portfolio — not €0 with 11 positions
- **(BP)** Value EUR populated when prices available — not all em dash
- **(BQ)** Total return % when baseline exists — not — with non-zero unrealized

## Artifacts

- Architecture: `docs/engineering/architecture.md` § **BUG-0023**
- Spec-pack: `docs/engineering/spec-pack/BUG-0023-{design-concept,crs,technical-specification}.md`
- Research: [R-0093 §5](docs/engineering/research.md#r-0093--bug-0023-crypto-wealth-eur-values-live-regression)

## Recommended next phase

`/sprint-plan` — materialize `/quick` sprint from architecture task table.

---

# research-20260612-bug0023 — BUG-0023 Crypto Wealth EUR values missing (live regression)

**From:** Tech Lead **To:** Dev (via architecture) **Bug:** BUG-0023 **Run:** `auto-20260612-bug0023`
**Date:** 2026-06-12 **Next phase:** `/architecture` (role: tech-lead)
**Intake evidence:** `handoffs/intake_evidence/intake-20260612-crypto-eur-values.json` (read-only)

## Research summary

Discovery verdicts **confirmed** (BO/BP/BQ). Research narrows root cause to **wallet ingest silent failure** on live Bitunix account API — not deploy gap (H4 ruled out), not wealth aggregation logic error when wallet is priced.

## Key findings

| Objective | Verdict | Evidence |
|-----------|---------|----------|
| Wallet parse path | **Silent skip most likely** | `parse_futures_wallet` returns `None` without log; HTTP client ignores JSON `code`; equity formula omits `crossUnrealizedPNL`/`isolationUnrealizedPNL`; positions sync uses separate endpoint and succeeds |
| Value EUR (BP) | **Fixable via `entryValue`** | Bitunix position payload already stored; display-only USDT→EUR per DEC-0064 (no subtotal merge) |
| Subtotal contract | **Wallet equity only** | `wealth/service.rs` `sum(market_value_eur)` correct when futures row priced; double-count guard unchanged |
| Total return (BQ) | **Downstream of BO** | `crypto_value_eur=0` → `total_return_pct=null`; unrealized path independent and working |
| DEC-0064 amend | **Not required (default)** | Only if architecture merges position notional into subtotal — rejected |

