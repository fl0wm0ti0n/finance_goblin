# Tasks — Q0012 (BUG-0005)

**Bug:** BUG-0005  
**Task count:** 6 (within `SPRINT_MAX_TASKS=12`)  
**Sprint-plan ref:** `sprint-plan-20260605-q0012-bug0005`

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **(M)** | M1, N2, O1 | Non-spot `product_type` holdings after sync |
| **(N)** | N1, N2, N3, N4, O1 | fapi header auth; data populated when enabled |
| **(O)** | M1, N2, O1 | Wealth crypto subtotal includes futures wallet |

| ID | Title | Est. | Status | Acceptance |
|----|-------|------|--------|------------|
| N1 | Futures header-auth client (DEC-0062) | 2.5h | done | **(N)** |
| N3 | effective_enabled_futures (DEC-0063) | 1h | done | **(N)** |
| M1 | Futures wallet balance ingestion | 2h | done | **(M)** |
| N2 | sync_positions (DEC-0064) | 2h | done | **(N)** |
| N4 | Dual-path test_connection | 1.5h | done | **(N)** |
| O1 | verify-work omniflow probes | 1h | open | **(M)(N)(O)** |

---

## N1 — Futures header-auth client (DEC-0062)

**Status:** done  
**Depends on:** —  
**Estimate:** 2.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0005 **(N)**

### Description

Add `bitunix_futures_sign` and `BitunixConnector::futures_signed_get` per R-0058. Add `futures_base_url` config default `https://fapi.bitunix.com`. Keep spot `signed_get` unchanged.

**Files:** `backend/src/exchanges/http.rs`, `backend/src/exchanges/bitunix.rs`, `backend/src/config/mod.rs`, `backend/config/default.toml`

### Done when

- [x] Unit: futures sign canonical string matches fixture
- [x] Unit: spot `bitunix_sign` unchanged (regression)
- [x] Mock HTTP: futures GET sends `api-key`, `nonce`, `timestamp`, `sign` headers
- [x] `cargo test --lib` exchanges/http PASS

---

## N3 — effective_enabled_futures (DEC-0063)

**Status:** done  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0005 **(N)**

### Description

Implement `BitunixConfig::effective_enabled_futures()` per DEC-0063. Settings view exposes effective value. Document `BITUNIX_ENABLED_FUTURES` in `.env.example`.

**Files:** `backend/src/config/mod.rs`, `.env.example`

### Done when

- [x] Unit: creds + TOML `enabled_futures=false` → effective true
- [x] Unit: env `BITUNIX_ENABLED_FUTURES=false` → effective false
- [x] Settings API returns effective `enabled_futures`

---

## M1 — Futures wallet balance ingestion

**Status:** done  
**Depends on:** N1, N3  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0005 **(M)**

### Description

When `effective_enabled_futures()`, fetch `/api/v1/futures/account?marginCoin=USDT` and append holdings with `product_type: "futures"`. Spot path unchanged when futures disabled.

**Files:** `backend/src/exchanges/bitunix.rs`

### Done when

- [x] Mock HTTP: account JSON → holding `product_type: "futures"`
- [x] Regression: futures disabled → spot-only sync unchanged
- [x] `cargo test --lib` bitunix PASS

---

## N2 — sync_positions (DEC-0064)

**Status:** done  
**Depends on:** N1, N3  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0005 **(M)**, **(N)**

### Description

Implement `sync_positions` via `/api/v1/futures/position/get_pending_positions`. Emit `product_type: "linear"`, `market_value_usd: None` per DEC-0064.

**Files:** `backend/src/exchanges/bitunix.rs`

### Done when

- [x] Mock HTTP: positions JSON → linear holdings with null market value
- [x] Non-zero positions update `state.active_symbols`
- [x] Disabled futures → `Ok(vec![])`

---

## N4 — Dual-path test_connection

**Status:** done  
**Depends on:** N1, N3  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0005 **(N)**

### Description

Probe spot always; probe futures when `effective_enabled_futures()`. Message pattern `Spot: OK; Futures: OK` or partial failure per architecture § N4.

**Files:** `backend/src/exchanges/bitunix.rs`

### Done when

- [x] Unit: spot OK + futures fail → `ok: true`, message contains both
- [x] Unit: spot fail → `ok: false`
- [x] Futures disabled → spot-only message unchanged

---

## O1 — verify-work omniflow probes

**Status:** open  
**Depends on:** N1, N2, N3, N4 deploy + exchange sync  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0005 **(M)(N)(O)**

### Description

Prepare `sprints/quick/Q0012/uat.md` smoke checklist. After deploy + manual exchange sync, probe settings, bitunix test, exchanges holdings, wealth crypto on `financegnome.omniflow.cc`.

**Files:** `sprints/quick/Q0012/uat.md`

### Done when

- [ ] Row **(M)**: non-spot holdings when operator has futures exposure
- [ ] Row **(N)**: test message mentions Futures; effective enabled_futures true
- [ ] Row **(O)**: wealth crypto subtotal reflects futures wallet
- [ ] Regression footer documented (OIDC + bundled-firefly)

**Operator gate:** Manual exchange sync required before O1 probes.
