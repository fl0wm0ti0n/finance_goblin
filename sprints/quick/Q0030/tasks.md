# Tasks — Q0030 (BUG-0023)

**Bug:** BUG-0023  
**Task count:** 9 mandatory (9/12 under `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260612-q0030-bug0023`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **BO1** | Task **BO1** | Equity fallback sum + unrealized key aliases |
| **BO2** | Task **BO2** | JSON `code==0` + parse-skip `warn!` |
| **BO3** | Task **BO3** | OpenAPI wiremock + unit tests |
| **BP1** | Task **BP1** | Migration 017 + `entryValue` → `exposure_eur` |
| **BP2** | Task **BP2** | `value_eur = market_value_eur.or(exposure_eur)` |
| **BQ1** | Task **BQ1** | Baseline + `total_return_pct` verify |
| **T1** | Task **T1** | `backend/tests/bug0023_*.rs` regression |
| **G1** | Task **G1** | `cargo test` + `npm run build` |
| **BO/BP/BQ runtime** | Task **V1** | verify-work after deploy |

## Execute order

```text
BO1 → BO2 → BO3
  → BP1 → BP2
  → BQ1
  → T1 → G1
  → operator: BACKEND_DEPLOY
  → operator: EXCHANGE_SYNC
  → operator: PNL_RECOMPUTE
  → V1 verify-work
```

**Parallelism:** BO slice sequential in `bitunix.rs`. BP1 touches migration + pnl +
repository — sequence after BO3 tests green. BQ1 verify-only unless baseline bug
found. T1 blocked on BO1–BQ1.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **BO** | BO1, BO2, BO3, T1, G1, V1 | `futures` wallet priced; subtotal ~€2000; card not €0 |
| **BP** | BP1, BP2, T1, G1, V1 | Linear `value_eur` from `exposure_eur`; subtotal wallet-only |
| **BQ** | BQ1, T1, G1, V1 | `total_return_pct` when baseline + priced wallet |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| BO1 | Equity + unrealized key parse fix | 2h | open | **BO** | P0 |
| BO2 | `code==0` validation + parse-skip logging | 1h | open | **BO** | P0 |
| BO3 | OpenAPI wiremock + unit tests | 2h | open | **BO** | P0 |
| BP1 | `exposure_eur` migration + pnl persist | 2.5h | open | **BP** | P1 |
| BP2 | `holdings_all.value_eur` map | 1h | open | **BP** | P1 |
| BQ1 | Baseline + total_return verify | 1.5h | open | **BQ** | P1 |
| T1 | Regression tests BO/BP/BQ | 2.5h | open | **BO**, **BP**, **BQ** | P0 |
| G1 | Automated gate | 0.5h | open | **BO**, **BP**, **BQ** | P0 |
| V1 | verify-work operator smoke | 2h | open | **BO**, **BP**, **BQ** | P0 |

---

## BO1 — Equity + unrealized key parse fix

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0023 **BO** — **DEC-0080**

### Description

In `backend/src/exchanges/bitunix.rs` `parse_futures_wallet`:

1. **Equity key scan** — retain `accountEquity`, `totalEquity`, `equity`, `balance` first.
2. **Equity fallback sum** — when direct keys absent:
   `available + frozen + margin + crossUnrealizedPNL + isolationUnrealizedPNL`.
3. **Unrealized keys** — add `crossUnrealizedPNL`, `isolationUnrealizedPNL` alongside
   existing aliases.
4. **Persist row** — `product_type=futures`, `asset=marginCoin` (default USDT),
   `quantity=equity`, `market_value_usd=Some(qty)` for USDT.

**Files:** `backend/src/exchanges/bitunix.rs`

### Done when

- [ ] Equity derived from fallback sum when direct keys absent
- [ ] Unrealized key aliases cover OpenAPI sample shape
- [ ] Parse yields non-zero equity for official Get Single Account fixture

### Verification

Unit test with OpenAPI array sample → `product_type=futures` row with non-zero quantity.

---

## BO2 — `code==0` validation + parse-skip logging

**Status:** open  
**Depends on:** BO1  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0023 **BO**

### Description

In `backend/src/exchanges/bitunix.rs` (HTTP helper if shared):

1. Reject body when JSON `code != 0` (or missing on error responses) before parse.
2. When `parse_futures_wallet` returns `None`, emit `warn!` with redacted shape
   diagnostic: marginCoin present, equity keys tried, derived sum components.
3. Positions sync unchanged — wallet failure must be observable in logs, not silent.

**Files:** `backend/src/exchanges/bitunix.rs`

### Done when

- [ ] `code != 0` responses do not persist wallet row
- [ ] Parse skip emits structured warn (no silent `None`)
- [ ] Positions sync path unaffected

### Verification

Test `code != 0` → no row; empty equity → warn path logged.

---

## BO3 — OpenAPI wiremock + unit tests

**Status:** open  
**Depends on:** BO2  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0023 **BO** — **R-0093**

### Description

Add regression tests in `backend/src/exchanges/bitunix.rs` tests (wiremock fixture
per R-0093 web refs):

1. **OpenAPI sample** — official Get Single Account array shape → futures row with
   non-zero equity.
2. **Zero-equity skip** — empty `data: []` → warn path, no row.
3. **`code != 0`** — no row persisted; error surfaced.

**Files:** `backend/src/exchanges/bitunix.rs` (tests module)

### Done when

- [ ] Wiremock fixture matches Bitunix OpenAPI Get Single Account shape
- [ ] All three test cases PASS
- [ ] No regression in existing bitunix exchange tests

### Verification

`cargo test bitunix` (wallet parse tests) → all PASS.

---

## BP1 — `exposure_eur` migration + pnl `entryValue` persist

**Status:** open  
**Depends on:** BO3  
**Estimate:** 2.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0023 **BP** — **DEC-0064**, **DEC-0081**

### Description

1. **Migration** — `backend/migrations/017_bug0023_exposure_eur.sql`: nullable
   `exposure_eur` on `exchange_holdings`.
2. **Parse** — `entryValue` from linear position `payload` (Bitunix pending-positions API).
3. **Convert** — `fx.to_eur(entryValue, "USDT", price_book)`.
4. **Persist** — set `exposure_eur`; **do not** set `market_value_eur` for linear.
5. **Recompute** — extend linear branch of `compute_hybrid_pnl` / `update_holding_eur`.

**Files:** `backend/migrations/017_bug0023_exposure_eur.sql`,
`backend/src/exchanges/repository.rs`, `backend/src/portfolio/pnl.rs`

### Done when

- [ ] Migration applies cleanly (nullable column)
- [ ] Linear recompute persists `exposure_eur` from `entryValue`
- [ ] `market_value_eur` remains NULL for linear rows

### Verification

Integration seed: linear holding with `entryValue` in payload → `exposure_eur` non-null
in DB after recompute.

---

## BP2 — `holdings_all.value_eur` from `exposure_eur`

**Status:** open  
**Depends on:** BP1  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0023 **BP** — **DEC-0064**

### Description

In `backend/src/wealth/service.rs`:

1. `holdings_all[].value_eur` = `market_value_eur.or(exposure_eur)` per row.
2. `holdings_top` — priced wallet rows only (unchanged).
3. `crypto.subtotal_eur` = `sum(market_value_eur)` only — **no** `exposure_eur` merge.

Frontend `WealthPage.tsx` pass-through only if API shape unchanged.

**Files:** `backend/src/wealth/service.rs` (minimal); `frontend/src/pages/WealthPage.tsx` if needed

### Done when

- [ ] API returns non-null `value_eur` for linear rows with `exposure_eur`
- [ ] Subtotal unchanged (wallet-only sum)
- [ ] `holdings_top` still wallet-priced only

### Verification

`GET /api/v1/wealth` integration test: linear `value_eur` populated; `subtotal_eur`
excludes `exposure_eur`.

---

## BQ1 — Baseline + `total_return_pct` integration verify

**Status:** open  
**Depends on:** BP2  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0023 **BQ** — **DEC-0038**

### Description

Verify (and fix only if bug found) in `backend/src/portfolio/service.rs`:

1. `compute_hybrid_pnl` — futures row priced → `crypto_value_eur > 0`.
2. `capture_if_missing` — baseline captured when exchange `sum(market_value_eur) > 0`.
3. `total_return_pct` — `(crypto_value_eur - baseline) / baseline` when `baseline > 0`.

Add integration test or SQL probe path: wallet row priced → baseline exists → API
`pnl.total_return_pct` non-null with non-zero unrealized.

**Rejected:** Drive `total_return_pct` from unrealized alone — violates **DEC-0038**.

**Files:** `backend/src/portfolio/service.rs`, `backend/tests/`

### Done when

- [ ] Integration test: priced wallet + baseline → `total_return_pct` non-null
- [ ] No code change unless baseline capture bug discovered
- [ ] Unrealized alone does not drive return %

### Verification

`cargo test` BQ integration case → PASS.

---

## T1 — Regression tests BO/BP/BQ

**Status:** open  
**Depends on:** BQ1  
**Estimate:** 2.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0023 **BO**, **BP**, **BQ**

### Description

Add `backend/tests/bug0023_crypto_wealth_eur.rs` (or split modules) covering:

1. **BO** — wiremock wallet ingest → `futures` row + non-zero `market_value_eur` after recompute.
2. **BP** — linear `entryValue` seed → `exposure_eur` + API `value_eur` non-null; subtotal wallet-only.
3. **BQ** — priced wallet + baseline → `total_return_pct` non-null.
4. **Regression** — wealth list shape unchanged; OIDC route smoke if harness supports.

**Files:** `backend/tests/bug0023_crypto_wealth_eur.rs`

### Done when

- [ ] BO/BP/BQ integration cases PASS
- [ ] DEC-0064 subtotal contract asserted in tests
- [ ] No regression in existing wealth/portfolio tests

### Verification

`cargo test --test bug0023_crypto_wealth_eur` → all PASS.

---

## G1 — Automated gate

**Status:** open  
**Depends on:** T1  
**Estimate:** 0.5h  
**Acceptance hook:** DEC-0064/0080/0081/0038 automated verification

### Description

Run and record automated checks in `sprints/quick/Q0030/progress.md`:

1. `cargo test --test bug0023_crypto_wealth_eur` → PASS.
2. `cargo test --lib` (or relevant suites) → no regression.
3. `npm run build` (frontend) → PASS.
4. `git diff --stat` blast radius matches frozen file list.

**Files:** `sprints/quick/Q0030/progress.md`

### Done when

- [ ] All automated checks PASS, recorded in progress.md
- [ ] No forbidden paths touched (subtotal merge, mark-price tier-2, Grafana)

### Verification

Test output pasted in progress.md; diff stat confirms scope.

---

## V1 — verify-work operator smoke

**Status:** open  
**Depends on:** G1 + operator BACKEND_DEPLOY + EXCHANGE_SYNC + PNL_RECOMPUTE  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0023 **BO**, **BP**, **BQ**

### Description

Populate `sprints/quick/Q0030/uat.md` and `uat.json` after deploy on
localhost:18080 (and optional omniflow OIDC):

1. **BO-API** — `GET /api/v1/wealth` — `crypto.subtotal_eur` ~€2000; `bitunix.subtotal_eur` not €0.
2. **BO-UI** — Wealth → Crypto — Bitunix card not €-0,00.
3. **BO-SQL** — `futures` row `market_value_eur` populated (AP1 probe).
4. **BP-API** — linear `holdings_all[].value_eur` non-null.
5. **BP-UI** — Value EUR column not all em dash.
6. **BP-SUBTOTAL** — subtotal wallet-only; linear `market_value_eur` NULL in DB.
7. **BQ-API** — `pnl.total_return_pct` non-null when baseline exists.
8. **BQ-UI** — Total return % not em dash with non-zero unrealized.
9. **OIDC-1** — `/wealth`, `/api/v1/wealth` smoke on omniflow profile.

**Files:** `sprints/quick/Q0030/uat.md`, `sprints/quick/Q0030/uat.json`

### Done when

- [ ] Rows **BO**, **BP**, **BQ** probed per acceptance.md matrix
- [ ] Regression gates documented
- [ ] `uat.md` and `uat.json` populated with results

**Operator gates:** **BACKEND_DEPLOY**; **EXCHANGE_SYNC**; **PNL_RECOMPUTE**; optional **AP1_SQL_PROBE**.

**Release note:** If operator questions `entryValue` vs mark-to-market gap, document D1
contract per architecture GATE-BP-1.
